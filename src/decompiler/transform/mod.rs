mod ir_utils;
mod pass_base;
mod passes;
mod pipeline;

pub use pass_base::{
    BlockTransform, BlockTransformContext, IBlockTransform, IProgramTransform, IStatementTransform,
    ITransform, LoopingBlockTransform, ProgramTransformContext, StatementTransform,
    StatementTransformContext, TransformContext, run_block_transforms, run_il_transforms,
};
pub use pipeline::{TransformPipeline, build_default_pipeline};
