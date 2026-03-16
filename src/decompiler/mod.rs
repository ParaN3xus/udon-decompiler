mod basic_block;
mod cfg;
mod codegen;
mod context;
mod indir_jump_analysis;
mod instruction_list;
mod ir;
mod module_info;
mod pipeline;
mod transform;
mod variable;

pub use basic_block::{BasicBlock, BasicBlockCollection, BasicBlockType};
pub use cfg::{
    CfgBuildOutput, FunctionCfg, StackFrame, StackSimulationResult, StackValue,
    build_cfgs_and_discover_entries,
};
pub use codegen::generate_csharp;
pub(crate) use codegen::{render_expression, render_variable_expression};
pub use context::{DecompileContext, DecompileHeapEntry, DecompileSymbol};
pub use instruction_list::{InstructionId, InstructionList};
pub use ir::{
    ControlFlowGraph, ControlFlowNode, IrAssignmentStatement, IrBlock, IrBlockContainer, IrBuilder,
    IrClass, IrConstructorCallExpression, IrContainerKind, IrExpression, IrExpressionStatement,
    IrExternalCallExpression, IrFunction, IrHighLevelDoWhile, IrHighLevelSwitch,
    IrHighLevelSwitchSection, IrHighLevelWhile, IrIf, IrInternalCallExpression, IrJump, IrLeave,
    IrLiteralExpression, IrOperator, IrOperatorCallExpression, IrPropertyAccessExpression,
    IrRawExpression, IrReturn, IrStatement, IrSwitch, IrVariableDeclarationStatement,
    IrVariableExpression, build_ir_functions, compute_dominance, mark_nodes_with_reachable_exits,
};
pub(crate) use ir::{build_extern_ir_expression, is_property_setter};
pub use module_info::{
    ExternFunctionInfo, FunctionDefinitionType, ParameterType, UINT32_ARRAY_GET_METHOD_NAME,
    UdonModuleInfo,
};
pub use pipeline::{DecompilePipelineOutput, run_analysis_pipeline, run_decompile_pipeline};
pub use transform::{
    BlockTransform, BlockTransformContext, IBlockTransform, IProgramTransform, IStatementTransform,
    ITransform, LoopingBlockTransform, ProgramTransformContext, StatementTransform,
    StatementTransformContext, TransformContext, TransformPipeline, build_default_pipeline,
    run_block_transforms, run_il_transforms,
};
pub use variable::{VariableRecord, VariableScope, VariableTable};

pub type Result<T> = std::result::Result<T, DecompileError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompileError {
    message: String,
}

impl DecompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for DecompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecompileError {}

impl From<crate::odin::OdinError> for DecompileError {
    fn from(value: crate::odin::OdinError) -> Self {
        Self::new(value.to_string())
    }
}

impl From<std::io::Error> for DecompileError {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}
