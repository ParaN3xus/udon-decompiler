use crate::decompiler::Result;
use crate::decompiler::ir::IrFunction;

use super::pass_base::{IProgramTransform, ITransform, ProgramTransformContext, run_il_transforms};
use super::passes::{
    class_construction::IrClassConstructionTransform, collect_label_usage::CollectLabelUsage,
    collect_variables::CollectVariables, condition_detection::ConditionDetection,
    const_to_literal::ConstToLiteral, control_flow_simplification::ControlFlowSimplification,
    detect_exit_points::DetectExitPoints,
    high_level_loop_statement_transform::HighLevelLoopStatementTransform,
    high_level_loop_transform::HighLevelLoopTransform,
    high_level_switch_transform::HighLevelSwitchTransform, loop_detection::LoopDetection,
    promote_globals::PromoteGlobals,
    structured_control_flow_cleanup_transform::StructuredControlFlowCleanupTransform,
    temp_variable_inline::TempVariableInline,
};

pub struct TransformPipeline {
    pub il_transforms: Vec<Box<dyn ITransform>>,
    pub program_transforms: Vec<Box<dyn IProgramTransform>>,
}

impl TransformPipeline {
    pub fn new(
        il_transforms: Vec<Box<dyn ITransform>>,
        program_transforms: Vec<Box<dyn IProgramTransform>>,
    ) -> Self {
        Self {
            il_transforms,
            program_transforms,
        }
    }

    pub fn run(
        &self,
        functions: &mut [IrFunction],
        context: &mut ProgramTransformContext<'_>,
    ) -> Result<()> {
        for function in functions.iter_mut() {
            run_il_transforms(function, &self.il_transforms, context)?;
        }

        for transform in &self.program_transforms {
            context.throw_if_cancellation_requested();
            transform.run(functions, context)?;
        }

        Ok(())
    }
}

pub fn build_default_pipeline() -> TransformPipeline {
    TransformPipeline::new(
        vec![
            Box::new(ControlFlowSimplification),
            Box::new(ConstToLiteral),
            Box::new(TempVariableInline),
            Box::new(DetectExitPoints::new(false)),
            Box::new(LoopDetection),
            Box::new(DetectExitPoints::new(true)),
            Box::new(ConditionDetection),
            Box::new(HighLevelLoopTransform),
            Box::new(HighLevelSwitchTransform),
            Box::new(HighLevelLoopStatementTransform),
            Box::new(StructuredControlFlowCleanupTransform),
            Box::new(CollectLabelUsage),
            Box::new(CollectVariables),
        ],
        vec![
            Box::new(IrClassConstructionTransform),
            Box::new(PromoteGlobals),
        ],
    )
}
