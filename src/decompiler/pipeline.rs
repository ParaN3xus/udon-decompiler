use tracing::debug;

use super::Result;
use super::basic_block::BasicBlockCollection;
use super::cfg::build_cfgs_and_discover_entries;
use super::codegen::generate_csharp;
use super::context::DecompileContext;
use super::ir::{IrClass, build_ir_functions};
use super::transform::{ProgramTransformContext, build_default_pipeline};
use super::variable::VariableTable;
use crate::udon_asm::{AsmBindDirective, AsmBindTableDirective, OpCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompilePipelineOutput {
    pub ir_class: IrClass,
    pub generated_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AsmBindAnalysis {
    pub binds: Vec<AsmBindDirective>,
    pub bind_tables: Vec<AsmBindTableDirective>,
}

pub fn run_analysis_pipeline(ctx: &mut DecompileContext) -> Result<()> {
    debug!("decompile pipeline: identify variables");
    ctx.variables = VariableTable::identify_from_heap(&ctx.heap_entries, &ctx.symbols);
    ctx.rebuild_symbol_address_maps_from_variables();

    debug!("decompile pipeline: identify basic blocks");
    ctx.basic_blocks = BasicBlockCollection::identify_from_context(ctx);
    ctx.rebuild_basic_block_address_map();

    debug!("decompile pipeline: discover entries + build cfg");
    let cfg_output = build_cfgs_and_discover_entries(ctx)?;
    ctx.cfg_functions = cfg_output.functions;
    ctx.stack_simulation = cfg_output.stack_simulation;

    Ok(())
}

pub fn run_decompile_pipeline(ctx: &mut DecompileContext) -> Result<DecompilePipelineOutput> {
    run_analysis_pipeline(ctx)?;

    debug!("decompile pipeline: lower cfg to ir functions");
    let mut functions = build_ir_functions(ctx);

    debug!("decompile pipeline: run transform pipeline");
    let class_name = ctx.infer_class_name_for_output();
    let mut transform_context = ProgramTransformContext::new(class_name.clone(), ctx);
    build_default_pipeline().run(&mut functions, &mut transform_context)?;

    let ir_class = transform_context.ir_class.unwrap_or(IrClass {
        class_name,
        namespace: None,
        variable_declarations: Vec::new(),
        functions,
    });

    debug!("decompile pipeline: generate code");
    let generated_code = generate_csharp(ctx, &ir_class)?;

    Ok(DecompilePipelineOutput {
        ir_class,
        generated_code,
    })
}

pub fn collect_asm_bind_analysis(ctx: &DecompileContext) -> Result<AsmBindAnalysis> {
    let mut bind_by_symbol = std::collections::BTreeMap::<String, u32>::new();
    for (inst_id, address, instruction) in ctx.instructions.iter() {
        if instruction.opcode != OpCode::Jump {
            continue;
        }

        let Some(state) = ctx.stack_simulation.get_instruction_state(address) else {
            continue;
        };
        let Some(top) = state.peek(0) else {
            continue;
        };
        let Some(symbol_name) = ctx.symbol_name_by_address.get(&top.value) else {
            continue;
        };

        let Some(next_address) = ctx
            .instructions
            .next_of(inst_id)
            .and_then(|next| ctx.instructions.address_of(next))
        else {
            continue;
        };
        if ctx.heap_u32_literals.get(&top.value).copied() != Some(next_address) {
            continue;
        }

        let target = instruction.numeric_operand();
        let is_internal_call = ctx.entry_points.iter().any(|entry| {
            let call_target = entry.entry_call_jump_target(ctx);
            entry.address == target || call_target == target
        });
        if !is_internal_call {
            continue;
        }

        match bind_by_symbol.insert(symbol_name.clone(), next_address) {
            Some(existing) if existing != next_address => {
                return Err(super::DecompileError::new(format!(
                    "conflicting call-jump bind addresses for symbol '{}': 0x{:08X} vs 0x{:08X}",
                    symbol_name, existing, next_address
                )));
            }
            _ => {}
        }
    }

    let mut bind_table_by_symbol = std::collections::BTreeMap::<String, Vec<u32>>::new();
    for block in &ctx.basic_blocks.blocks {
        let Some(switch_info) = block.switch_info.as_ref() else {
            continue;
        };
        let Some(symbol_name) = ctx.symbol_name_by_address.get(&switch_info.table_operand) else {
            continue;
        };
        match bind_table_by_symbol.insert(symbol_name.clone(), switch_info.targets.clone()) {
            Some(existing) if existing != switch_info.targets => {
                return Err(super::DecompileError::new(format!(
                    "conflicting switch bind-table targets for symbol '{}'",
                    symbol_name
                )));
            }
            _ => {}
        }
    }

    Ok(AsmBindAnalysis {
        binds: bind_by_symbol
            .into_iter()
            .map(|(symbol, address)| AsmBindDirective { symbol, address })
            .collect(),
        bind_tables: bind_table_by_symbol
            .into_iter()
            .map(|(symbol, addresses)| AsmBindTableDirective { symbol, addresses })
            .collect(),
    })
}
