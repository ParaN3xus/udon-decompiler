use std::collections::HashMap;

use crate::decompiler::context::DecompileContext;
use crate::decompiler::ir::{ControlFlowGraph, IrBlock, IrClass, IrFunction};
use crate::decompiler::{DecompileError, Result};

use super::ir_utils::{find_container_mut, iter_block_containers};

#[derive(Debug, Clone)]
pub struct ProgramTransformContext<'ctx> {
    pub class_name: String,
    pub ir_class: Option<IrClass>,
    pub decompile_context: &'ctx DecompileContext,
    pub settings: HashMap<String, String>,
    pub metadata: HashMap<String, i64>,
}

impl<'ctx> ProgramTransformContext<'ctx> {
    pub fn new(class_name: String, decompile_context: &'ctx DecompileContext) -> Self {
        Self {
            class_name,
            ir_class: None,
            decompile_context,
            settings: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn throw_if_cancellation_requested(&self) {}

    pub fn create_il_context(&mut self, function_name: String) -> TransformContext<'_, 'ctx> {
        TransformContext {
            function_name,
            program_context: self,
        }
    }
}

pub struct TransformContext<'a, 'ctx> {
    pub function_name: String,
    pub program_context: &'a mut ProgramTransformContext<'ctx>,
}

impl<'a, 'ctx> TransformContext<'a, 'ctx> {
    pub fn throw_if_cancellation_requested(&self) {
        self.program_context.throw_if_cancellation_requested();
    }
}

pub trait IProgramTransform {
    fn run(
        &self,
        functions: &mut [IrFunction],
        context: &mut ProgramTransformContext<'_>,
    ) -> Result<()>;
}

pub trait ITransform {
    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()>;
}

pub trait IBlockTransform {
    fn run(
        &self,
        block: &mut IrBlock,
        context: &mut BlockTransformContext<'_, '_, '_>,
    ) -> Result<()>;
}

pub trait IStatementTransform {
    fn run(
        &self,
        block: &mut IrBlock,
        pos: usize,
        context: &mut StatementTransformContext<'_, '_, '_, '_>,
    ) -> Result<()>;
}

pub struct BlockTransformContext<'a, 'b, 'ctx> {
    pub transform_context: &'a mut TransformContext<'b, 'ctx>,
    pub container_id: u32,
    pub control_flow_graph: ControlFlowGraph,
    pub block_index: Option<usize>,
    pub control_flow_node_index: Option<usize>,
    dirty: bool,
}

impl<'a, 'b, 'ctx> BlockTransformContext<'a, 'b, 'ctx> {
    fn new(
        transform_context: &'a mut TransformContext<'b, 'ctx>,
        container_id: u32,
        control_flow_graph: ControlFlowGraph,
    ) -> Self {
        Self {
            transform_context,
            container_id,
            control_flow_graph,
            block_index: None,
            control_flow_node_index: None,
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

pub struct StatementTransformContext<'a, 'b, 'c, 'ctx> {
    pub block_context: &'a mut BlockTransformContext<'b, 'c, 'ctx>,
    pub rerun_current_position: bool,
    pub rerun_position: Option<usize>,
}

impl<'a, 'b, 'c, 'ctx> StatementTransformContext<'a, 'b, 'c, 'ctx> {
    fn new(block_context: &'a mut BlockTransformContext<'b, 'c, 'ctx>) -> Self {
        Self {
            block_context,
            rerun_current_position: false,
            rerun_position: None,
        }
    }

    pub fn request_rerun(&mut self, pos: Option<usize>) {
        let Some(pos) = pos else {
            self.rerun_current_position = true;
            return;
        };
        if self.rerun_position.is_none_or(|x| pos > x) {
            self.rerun_position = Some(pos);
        }
    }
}

pub fn run_il_transforms(
    function: &mut IrFunction,
    transforms: &[Box<dyn ITransform>],
    program_context: &mut ProgramTransformContext<'_>,
) -> Result<()> {
    for transform in transforms {
        program_context.throw_if_cancellation_requested();
        let function_name = function.function_name.clone();
        let mut il_context = program_context.create_il_context(function_name);
        transform.run(function, &mut il_context)?;
    }
    Ok(())
}

pub fn run_block_transforms(
    block: &mut IrBlock,
    transforms: &[Box<dyn IBlockTransform>],
    context: &mut BlockTransformContext<'_, '_, '_>,
) -> Result<()> {
    for transform in transforms {
        context
            .transform_context
            .program_context
            .throw_if_cancellation_requested();
        transform.run(block, context)?;
    }
    Ok(())
}

pub struct LoopingBlockTransform {
    transforms: Vec<Box<dyn IBlockTransform>>,
}

impl LoopingBlockTransform {
    pub fn new(transforms: Vec<Box<dyn IBlockTransform>>) -> Self {
        Self { transforms }
    }
}

impl IBlockTransform for LoopingBlockTransform {
    fn run(
        &self,
        block: &mut IrBlock,
        context: &mut BlockTransformContext<'_, '_, '_>,
    ) -> Result<()> {
        loop {
            context.reset_dirty();
            run_block_transforms(block, &self.transforms, context)?;
            if !context.is_dirty() {
                break;
            }
        }
        Ok(())
    }
}

pub struct StatementTransform {
    children: Vec<Box<dyn IStatementTransform>>,
}

impl StatementTransform {
    pub fn new(children: Vec<Box<dyn IStatementTransform>>) -> Self {
        Self { children }
    }
}

impl IBlockTransform for StatementTransform {
    fn run(
        &self,
        block: &mut IrBlock,
        context: &mut BlockTransformContext<'_, '_, '_>,
    ) -> Result<()> {
        if block.statements.is_empty() {
            return Ok(());
        }

        let mut statement_context = StatementTransformContext::new(context);
        let mut pos = block.statements.len() - 1;
        statement_context.rerun_position = Some(pos);

        loop {
            if block.statements.is_empty() {
                break;
            }
            if let Some(rerun_position) = statement_context.rerun_position.take() {
                pos = rerun_position.min(block.statements.len() - 1);
            } else if pos >= block.statements.len() {
                pos = block.statements.len() - 1;
            }

            for transform in &self.children {
                transform.run(block, pos, &mut statement_context)?;
                if statement_context.rerun_current_position {
                    statement_context.rerun_current_position = false;
                    statement_context.request_rerun(Some(pos));
                }
                if statement_context.rerun_position.is_some() {
                    break;
                }
            }

            if statement_context.rerun_position.is_none() {
                if pos == 0 {
                    break;
                }
                pos -= 1;
            }
        }

        Ok(())
    }
}

pub struct BlockTransform {
    pub pre_order_transforms: Vec<Box<dyn IBlockTransform>>,
    pub post_order_transforms: Vec<Box<dyn IBlockTransform>>,
}

impl BlockTransform {
    pub fn new() -> Self {
        Self {
            pre_order_transforms: Vec::new(),
            post_order_transforms: Vec::new(),
        }
    }
}

impl Default for BlockTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl ITransform for BlockTransform {

    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let container_ids = iter_block_containers(function);

        for container_id in container_ids {
            let function_body_container_id = function.body.id;
            let Some(container) = find_container_mut(&mut function.body, container_id) else {
                continue;
            };

            let cfg = ControlFlowGraph::new(container, function_body_container_id);
            if cfg.nodes.is_empty() {
                continue;
            }

            let mut block_context = BlockTransformContext::new(context, container_id, cfg);
            run_dominator_tree_node(
                0,
                container,
                &self.pre_order_transforms,
                &self.post_order_transforms,
                &mut block_context,
            )?;
        }

        Ok(())
    }
}

fn run_dominator_tree_node(
    node_index: usize,
    container: &mut crate::decompiler::ir::IrBlockContainer,
    pre_transforms: &[Box<dyn IBlockTransform>],
    post_transforms: &[Box<dyn IBlockTransform>],
    context: &mut BlockTransformContext<'_, '_, '_>,
) -> Result<()> {
    if node_index >= container.blocks.len() {
        return Err(DecompileError::new(format!(
            "Dominator node index {} out of bounds for container {}",
            node_index, container.id
        )));
    }

    context.block_index = Some(node_index);
    context.control_flow_node_index = Some(node_index);

    {
        let block = container
            .blocks
            .get_mut(node_index)
            .ok_or_else(|| DecompileError::new("missing block for dominator node"))?;
        run_block_transforms(block, pre_transforms, context)?;
    }

    let children = context.control_flow_graph.nodes[node_index]
        .dominator_tree_children
        .clone()
        .unwrap_or_default();
    for child in children {
        run_dominator_tree_node(child, container, pre_transforms, post_transforms, context)?;
    }

    {
        let block = container
            .blocks
            .get_mut(node_index)
            .ok_or_else(|| DecompileError::new("missing block for dominator node"))?;
        run_block_transforms(block, post_transforms, context)?;
    }

    Ok(())
}
