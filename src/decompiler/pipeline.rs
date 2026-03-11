use tracing::info;

use super::Result;
use super::basic_block::BasicBlockCollection;
use super::cfg::build_cfgs_and_discover_entries;
use super::codegen::generate_csharp;
use super::context::DecompileContext;
use super::ir::{IrClass, build_ir_functions};
use super::transform::{ProgramTransformContext, build_default_pipeline};
use super::variable::VariableTable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompilePipelineOutput {
    pub ir_class: IrClass,
    pub generated_code: String,
}

pub fn run_decompile_pipeline(ctx: &mut DecompileContext) -> Result<DecompilePipelineOutput> {
    info!("decompile pipeline: identify variables");
    ctx.variables = VariableTable::identify_from_heap(&ctx.heap_entries, &ctx.symbols);
    ctx.rebuild_symbol_address_maps_from_variables();

    info!("decompile pipeline: identify basic blocks");
    ctx.basic_blocks = BasicBlockCollection::identify_from_context(ctx);
    ctx.rebuild_basic_block_address_map();

    info!("decompile pipeline: discover entries + build cfg");
    let cfg_output = build_cfgs_and_discover_entries(ctx)?;
    ctx.cfg_functions = cfg_output.functions;
    ctx.stack_simulation = cfg_output.stack_simulation;

    info!("decompile pipeline: lower cfg to ir functions");
    let mut functions = build_ir_functions(ctx);

    info!("decompile pipeline: run transform pipeline");
    let class_name = ctx.infer_class_name_for_output();
    let mut transform_context = ProgramTransformContext::new(class_name.clone(), ctx);
    build_default_pipeline().run(&mut functions, &mut transform_context)?;

    let ir_class = transform_context.ir_class.unwrap_or(IrClass {
        class_name,
        namespace: None,
        variable_declarations: Vec::new(),
        functions,
    });

    info!("decompile pipeline: generate code");
    let generated_code = generate_csharp(ctx, &ir_class)?;

    Ok(DecompilePipelineOutput {
        ir_class,
        generated_code,
    })
}
