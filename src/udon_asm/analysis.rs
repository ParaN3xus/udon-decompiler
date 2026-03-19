use crate::decompiler::{
    DecompileContext, DecompileError, ExternFunctionInfo, IrExpression, IrVariableExpression,
    Result, StackValue, UdonModuleInfo, build_extern_ir_expression, is_property_setter,
    render_expression, render_variable_expression,
};

use super::{AsmBindDirective, AsmBindTableDirective, AsmInstructionComment, OpCode};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AsmBindAnalysis {
    pub binds: Vec<AsmBindDirective>,
    pub bind_tables: Vec<AsmBindTableDirective>,
}

pub fn collect_asm_bind_analysis(ctx: &DecompileContext) -> Result<AsmBindAnalysis> {
    let mut bind_by_symbol = std::collections::BTreeMap::<String, u32>::new();
    for (inst_id, address, instruction) in ctx.instructions.iter() {
        if instruction.opcode != OpCode::Jump {
            continue;
        }

        let Some((symbol_name, next_address)) =
            resolve_internal_call_bind(ctx, inst_id, address, instruction)
        else {
            continue;
        };

        match bind_by_symbol.insert(symbol_name, next_address) {
            Some(existing) if existing != next_address => {
                return Err(DecompileError::new(format!(
                    "conflicting call-jump bind addresses: 0x{:08X} vs 0x{:08X}",
                    existing, next_address
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
                return Err(DecompileError::new(format!(
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

pub fn collect_asm_instruction_comments(
    ctx: &DecompileContext,
) -> Result<Vec<AsmInstructionComment>> {
    let module_info = UdonModuleInfo::load_default_cached()?;
    let mut instruction_comments = Vec::<AsmInstructionComment>::new();
    for (inst_id, address, instruction) in ctx.instructions.iter() {
        let text = match instruction.opcode {
            OpCode::Copy => render_copy_instruction_comment(ctx, address),
            OpCode::JumpIfFalse => render_jump_if_false_comment(ctx, address),
            OpCode::Extern => render_extern_instruction_comment(
                ctx,
                address,
                instruction.numeric_operand(),
                module_info,
            ),
            OpCode::Jump => resolve_internal_call_name(ctx, instruction)
                .filter(|_| {
                    resolve_internal_call_bind(ctx, inst_id, address, instruction).is_some()
                })
                .map(|name| format!("call {name}();")),
            OpCode::Nop
            | OpCode::Annotation
            | OpCode::Pop
            | OpCode::Push
            | OpCode::JumpIndirect => None,
        };
        if let Some(text) = text {
            instruction_comments.push(AsmInstructionComment { address, text });
        }
    }
    Ok(instruction_comments)
}

fn resolve_internal_call_bind(
    ctx: &DecompileContext,
    inst_id: crate::decompiler::InstructionId,
    address: u32,
    instruction: &crate::udon_asm::AsmInstruction,
) -> Option<(String, u32)> {
    let state = ctx.stack_simulation.get_instruction_state(address)?;
    let top = state.peek(0)?;
    let top_address = top.maybe_heap_address()?;
    let symbol_name = ctx.symbol_name_by_address.get(&top_address)?.clone();
    let next_address = ctx
        .instructions
        .next_of(inst_id)
        .and_then(|next| ctx.instructions.address_of(next))?;
    if top.resolve_u32_literal(ctx) != Some(next_address) {
        return None;
    }

    let target = instruction.numeric_operand();
    let _entry = ctx.entry_points.iter().find(|entry| {
        let call_target = entry.entry_call_jump_target(ctx);
        entry.address == target || call_target == target
    })?;
    Some((symbol_name, next_address))
}

fn resolve_internal_call_name(
    ctx: &DecompileContext,
    instruction: &crate::udon_asm::AsmInstruction,
) -> Option<String> {
    let target = instruction.numeric_operand();
    let entry = ctx.entry_points.iter().find(|entry| {
        let call_target = entry.entry_call_jump_target(ctx);
        entry.address == target || call_target == target
    })?;
    Some(entry.name.clone())
}

fn render_extern_instruction_comment(
    ctx: &DecompileContext,
    address: u32,
    signature_operand: u32,
    module_info: &UdonModuleInfo,
) -> Option<String> {
    let signature = ctx.heap_string_literals.get(&signature_operand)?;
    let function_info = module_info.get_function_info(signature)?;
    let state = ctx.stack_simulation.get_instruction_state(address)?;
    if state.depth() < function_info.parameter_count() {
        return None;
    }

    let args = (0..function_info.parameter_count())
        .map(|index| {
            let depth = function_info.parameter_count() - 1 - index;
            state.peek(depth).and_then(StackValue::maybe_heap_address)
        })
        .collect::<Option<Vec<_>>>()?;
    Some(render_extern_comment(ctx, &function_info, &args))
}

fn render_copy_instruction_comment(ctx: &DecompileContext, address: u32) -> Option<String> {
    let state = ctx.stack_simulation.get_instruction_state(address)?;
    let target = state.peek(0)?;
    let source = state.peek(1)?;
    let target_name = render_variable_expression(target.maybe_heap_address()?, ctx);
    let source_name = render_variable_expression(source.maybe_heap_address()?, ctx);
    if target_name == source_name {
        return None;
    }
    Some(format!("{target_name} = {source_name};"))
}

fn render_jump_if_false_comment(ctx: &DecompileContext, address: u32) -> Option<String> {
    let state = ctx.stack_simulation.get_instruction_state(address)?;
    let condition = state.peek(0)?;
    let condition_name = render_variable_expression(condition.maybe_heap_address()?, ctx);
    Some(format!("if (!{condition_name})"))
}

fn render_extern_comment(
    ctx: &DecompileContext,
    function_info: &ExternFunctionInfo,
    args: &[u32],
) -> String {
    if is_property_setter(function_info) {
        let value = args
            .last()
            .map(|address| render_variable_expression(*address, ctx))
            .unwrap_or_else(|| panic!("property setter missing value argument"));
        let target_expr = build_extern_comment_ir_expression(
            function_info,
            args.get(..args.len().saturating_sub(1)).unwrap_or(&[]),
        );
        return format!("{} = {value};", render_expression(&target_expr, ctx));
    }

    let expression_args = if function_info.returns_void {
        args.to_vec()
    } else {
        args.get(..args.len().saturating_sub(1))
            .unwrap_or(&[])
            .to_vec()
    };
    let expression = build_extern_comment_ir_expression(function_info, &expression_args);
    if function_info.returns_void {
        format!("{};", render_expression(&expression, ctx))
    } else {
        let target = args
            .last()
            .map(|address| render_variable_expression(*address, ctx))
            .unwrap_or_else(|| panic!("extern comment missing return slot"));
        format!("{target} = {};", render_expression(&expression, ctx))
    }
}

fn build_extern_comment_ir_expression(
    function_info: &ExternFunctionInfo,
    args: &[u32],
) -> IrExpression {
    let arguments = args
        .iter()
        .map(|address| IrExpression::Variable(IrVariableExpression { address: *address }))
        .collect::<Vec<_>>();
    build_extern_ir_expression(
        function_info.clone(),
        function_info.signature.clone(),
        arguments,
    )
}
