use std::collections::HashMap;

use crate::decompiler::Result;
use crate::decompiler::ir::{
    ControlFlowGraph, IrBlock, IrBlockContainer, IrExpression, IrFunction, IrHighLevelDoWhile,
    IrHighLevelSwitch, IrHighLevelWhile, IrIf, IrJump, IrOperator, IrOperatorCallExpression,
    IrStatement, IrSwitch,
};
use crate::decompiler::transform::ir_utils::{find_container_mut, iter_block_containers};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};
use crate::decompiler::transform::passes::detect_exit_points::DetectExitPoints;

pub struct ConditionDetection;

impl ITransform for ConditionDetection {

    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let function_body_id = function.body.id;
        let mut state = ConditionDetectionState::from_context(context);
        rewrite_container(function, function_body_id, &mut state);
        state.commit(context);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ConditionDetectionState {
    synthetic_block_address: i64,
}

impl ConditionDetectionState {
    fn from_context(context: &TransformContext<'_, '_>) -> Self {
        let current = context
            .program_context
            .metadata
            .get("_synthetic_block_addr")
            .copied()
            .unwrap_or(-1);
        Self {
            synthetic_block_address: current,
        }
    }

    fn next_synthetic_block_address(&mut self) -> u32 {
        let current = self.synthetic_block_address;
        self.synthetic_block_address -= 1;
        (current as i32) as u32
    }

    fn commit(self, context: &mut TransformContext<'_, '_>) {
        context.program_context.metadata.insert(
            "_synthetic_block_addr".to_string(),
            self.synthetic_block_address,
        );
    }
}

fn rewrite_container(
    function: &mut IrFunction,
    function_body_id: u32,
    state: &mut ConditionDetectionState,
) {
    let container_ids = iter_block_containers(function);
    for container_id in container_ids {
        let Some(container) = find_container_mut(&mut function.body, container_id) else {
            break;
        };

        let cfg = ControlFlowGraph::new(container, function_body_id);
        if cfg.nodes.is_empty() {
            continue;
        }

        let addr_to_node = cfg
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| node.block_start_address.map(|addr| (addr, idx)))
            .collect::<HashMap<_, _>>();

        let mut block_addrs = dominator_post_order_block_addrs(&cfg);
        if block_addrs.is_empty() {
            block_addrs = container
                .blocks
                .iter()
                .map(|block| block.start_address)
                .collect::<Vec<_>>();
        }
        for block_addr in block_addrs {
            let _ = process_block(container, state, block_addr, &cfg, &addr_to_node);
        }
    }
}

fn dominator_post_order_block_addrs(cfg: &ControlFlowGraph) -> Vec<u32> {
    if cfg.nodes.is_empty() {
        return Vec::new();
    }

    fn visit(node: usize, cfg: &ControlFlowGraph, out: &mut Vec<u32>) {
        if let Some(children) = cfg.nodes[node].dominator_tree_children.as_ref() {
            for child in children {
                visit(*child, cfg, out);
            }
        }
        if let Some(addr) = cfg.nodes[node].block_start_address {
            out.push(addr);
        }
    }

    let mut out = Vec::<u32>::new();
    visit(0, cfg, &mut out);
    out
}

fn process_block(
    container: &mut IrBlockContainer,
    state: &mut ConditionDetectionState,
    block_address: u32,
    cfg: &ControlFlowGraph,
    addr_to_node: &HashMap<u32, usize>,
) -> bool {
    let Some(cfg_node_index) = addr_to_node.get(&block_address).copied() else {
        return false;
    };

    let Some(block_index) = find_block_index(container, block_address) else {
        return false;
    };

    let mut changed = false;

    {
        let Some(block) = container.blocks.get_mut(block_index) else {
            return false;
        };

        if let Some((mut if_inst, mut exit_inst, normalized)) = get_if_with_exit(block) {
            if normalized {
                changed = true;
            }

            if should_swap_if_targets(&if_inst.true_statement, &exit_inst, container) {
                let old_true = (*if_inst.true_statement).clone();
                if_inst.true_statement = Box::new(exit_inst.clone());
                exit_inst = old_true;
                if_inst.condition = logic_not(if_inst.condition);
                changed = true;
            }

            let mut true_exit_inst: Option<IrStatement> = None;

            if is_usable_branch_to_child(
                &if_inst.true_statement,
                container,
                cfg,
                cfg_node_index,
                block_address,
            ) {
                let target_addr = as_jump_target_addr(if_inst.true_statement.as_ref())
                    .expect("jump target expected");
                if let Some(mut target_block) = remove_block_by_address(container, target_addr) {
                    changed = true;
                    if_inst.true_statement = Box::new(IrStatement::Block(target_block.clone()));
                    let mut keep_wrapped_target_block = true;

                    while !target_block.statements.is_empty() {
                        let nested_if = target_block.statements.first().cloned();
                        let Some(IrStatement::If(nested_if)) = nested_if else {
                            break;
                        };
                        if nested_if.false_statement.is_some() {
                            break;
                        }

                        let nested_true =
                            unpack_block_containing_only_branch(nested_if.true_statement.as_ref());
                        if DetectExitPoints::compatible_exit_instruction(&exit_inst, &nested_true) {
                            if_inst.condition =
                                logic_and(if_inst.condition, logic_not(nested_if.condition));
                            target_block.statements.remove(0);
                            changed = true;
                            continue;
                        }

                        let mut rewrite_nested = false;
                        if let IrStatement::Block(nested_true_block) = &nested_true
                            && !nested_true_block.statements.is_empty()
                            && target_block.statements.len() > 1
                            && contains_non_exit_statement(&target_block.statements[1..])
                            && nested_true_block.statements.last().is_some_and(|stmt| {
                                DetectExitPoints::compatible_exit_instruction(&exit_inst, stmt)
                            })
                            && has_unreachable_endpoint(&target_block)
                        {
                            rewrite_nested = true;
                        }

                        if rewrite_nested
                            && let Some(IrStatement::If(mut nested_if_new)) =
                                target_block.statements.first().cloned()
                        {
                            nested_if_new.condition = logic_not(nested_if_new.condition);
                            if let IrStatement::Block(mut nested_true_block) =
                                *nested_if_new.true_statement
                            {
                                nested_true_block.statements.pop();
                                let true_path_statements = nested_true_block.statements.clone();
                                let false_path_statements = target_block.statements[1..].to_vec();
                                nested_true_block.statements = false_path_statements;
                                nested_if_new.true_statement =
                                    Box::new(IrStatement::Block(nested_true_block));

                                target_block.statements = vec![IrStatement::If(nested_if_new)];
                                target_block.statements.extend(true_path_statements);
                                changed = true;
                            }
                        }

                        break;
                    }

                    true_exit_inst = last_exit_statement(&target_block);
                    if true_exit_inst.as_ref().is_some_and(|stmt| {
                        DetectExitPoints::compatible_exit_instruction(&exit_inst, stmt)
                    }) {
                        target_block.statements.pop();
                        true_exit_inst = None;
                        changed = true;

                            if target_block.statements.len() == 1
                                && let Some(IrStatement::If(nested_if)) =
                                    target_block.statements.first().cloned()
                                && nested_if.false_statement.is_none()
                        {
                            if_inst.condition = logic_and(if_inst.condition, nested_if.condition);
                            if_inst.true_statement = nested_if.true_statement;
                            keep_wrapped_target_block = false;

                            if let IrStatement::Block(inlined_true_block) =
                                if_inst.true_statement.as_ref()
                            {
                                true_exit_inst = last_exit_statement(inlined_true_block);
                            } else if is_branch_or_leave(if_inst.true_statement.as_ref()) {
                                true_exit_inst = Some((*if_inst.true_statement).clone());
                            }
                            changed = true;
                        }
                    }

                    if keep_wrapped_target_block {
                        if_inst.true_statement = Box::new(IrStatement::Block(target_block));
                    }
                }
            } else if is_branch_or_leave(if_inst.true_statement.as_ref()) {
                true_exit_inst = Some((*if_inst.true_statement).clone());
            }

            let container_view_owned: Option<IrBlockContainer>;
            let container_view =
                if let Some(current_index) = find_block_index(container, block_address) {
                    let mut view = container.clone();
                    if let Some(view_block) = view.blocks.get_mut(current_index)
                        && view_block.statements.len() >= 2
                    {
                        let last = view_block.statements.len() - 1;
                        view_block.statements[last - 1] = IrStatement::If(if_inst.clone());
                        view_block.statements[last] = exit_inst.clone();
                    }
                    container_view_owned = Some(view);
                    container_view_owned.as_ref().expect("view present")
                } else {
                    &*container
                };

            if is_usable_branch_to_child(
                &exit_inst,
                container_view,
                cfg,
                cfg_node_index,
                block_address,
            ) {
                let target_addr = as_jump_target_addr(&exit_inst).expect("jump target expected");
                if let Some(target_block) = get_block_clone_by_address(container, target_addr) {
                    let false_exit_inst = last_exit_statement(&target_block);
                    if let (Some(true_exit_inst), Some(false_exit_inst)) =
                        (true_exit_inst.as_ref(), false_exit_inst.as_ref())
                        && DetectExitPoints::compatible_exit_instruction(
                            true_exit_inst,
                            false_exit_inst,
                        )
                    {
                        let mut target_block =
                            remove_block_by_address(container, target_addr).expect("target exists");
                        target_block.statements.pop();

                        if_inst.false_statement = if target_block.statements.is_empty() {
                            None
                        } else {
                            Some(Box::new(IrStatement::Block(target_block)))
                        };

                        exit_inst = false_exit_inst.clone();

                        if let IrStatement::Block(true_block) = if_inst.true_statement.as_mut() {
                            if true_block
                                .statements
                                .last()
                                .is_some_and(|stmt| stmt == true_exit_inst)
                            {
                                true_block.statements.pop();
                            }
                        } else if if_inst.true_statement.as_ref() == true_exit_inst {
                            if_inst.true_statement = Box::new(empty_block(state));
                        }

                        changed = true;
                    }
                }
            }

            if is_empty(if_inst.true_statement.as_ref()) {
                let old_true = (*if_inst.true_statement).clone();
                if_inst.true_statement = Box::new(
                    if_inst
                        .false_statement
                        .take()
                        .map(|stmt| *stmt)
                        .unwrap_or_else(|| empty_block(state)),
                );
                if_inst.false_statement = Some(Box::new(old_true));
                if_inst.condition = logic_not(if_inst.condition);
                changed = true;

                if let IrStatement::Block(true_block) = if_inst.true_statement.as_ref()
                    && true_block.statements.len() == 1
                    && let Some(IrStatement::If(nested_if)) = true_block.statements.first().cloned()
                    && nested_if.false_statement.is_none()
                {
                    if_inst.condition = logic_and(if_inst.condition, nested_if.condition);
                    if_inst.true_statement = nested_if.true_statement;
                }
            } else if if_inst.false_statement.as_ref().is_some_and(|false_stmt| {
                statement_start(false_stmt.as_ref())
                    < statement_start(if_inst.true_statement.as_ref())
            }) {
                let old_true = (*if_inst.true_statement).clone();
                let new_true = if_inst.false_statement.take().expect("present");
                if_inst.true_statement = new_true;
                if_inst.false_statement = Some(Box::new(old_true));
                if_inst.condition = logic_not(if_inst.condition);
                changed = true;
            }

            let Some(current_index) = find_block_index(container, block_address) else {
                return true;
            };
            if let Some(block) = container.blocks.get_mut(current_index)
                && block.statements.len() >= 2
            {
                let last = block.statements.len() - 1;
                block.statements[last - 1] = IrStatement::If(if_inst);
                block.statements[last] = exit_inst;
            }
        }
    }

    let Some(current_index) = find_block_index(container, block_address) else {
        return true;
    };

    let should_inline_unconditional = container
        .blocks
        .get(current_index)
        .and_then(|block| block.statements.last())
        .is_some_and(|last| {
            is_usable_branch_to_child(last, container, cfg, cfg_node_index, block_address)
        });

    if should_inline_unconditional {
        let target_addr = container
            .blocks
            .get(current_index)
            .and_then(|block| block.statements.last())
            .and_then(as_jump_target_addr)
            .expect("jump target exists");

        if let Some(target_block) = remove_block_by_address(container, target_addr)
            && let Some(block) = container.blocks.get_mut(current_index)
            && matches!(block.statements.last(), Some(IrStatement::Jump(_)))
        {
            block.statements.pop();
            block.statements.extend(target_block.statements);
            changed = true;
        }
    }

    changed
}

fn get_if_with_exit(block: &mut IrBlock) -> Option<(IrIf, IrStatement, bool)> {
    if block.statements.len() >= 2 {
        let if_stmt = block.statements.get(block.statements.len() - 2).cloned();
        let exit_stmt = block.statements.last().cloned();
        if let (Some(IrStatement::If(if_stmt)), Some(exit_stmt)) = (if_stmt, exit_stmt)
            && if_stmt.false_statement.is_none()
            && is_branch_or_leave(&exit_stmt)
        {
            return Some((if_stmt, exit_stmt, false));
        }
    }

    let len = block.statements.len();
    if len == 0 {
        return None;
    }

    let terminal_if = block.statements[len - 1].clone();
    if let IrStatement::If(mut if_stmt) = terminal_if
        && let Some(false_stmt) = if_stmt.false_statement.clone()
        && is_branch_or_leave(false_stmt.as_ref())
    {
        let exit_stmt = *false_stmt;
        if_stmt.false_statement = None;
        block.statements[len - 1] = IrStatement::If(if_stmt.clone());
        block.statements.push(exit_stmt.clone());
        return Some((if_stmt, exit_stmt, true));
    }

    None
}

fn should_swap_if_targets(
    true_statement: &IrStatement,
    exit_statement: &IrStatement,
    container: &IrBlockContainer,
) -> bool {
    match (true_statement, exit_statement) {
        (
            IrStatement::Jump(IrJump {
                target_address: true_target,
            }),
            IrStatement::Jump(IrJump {
                target_address: exit_target,
            }),
        ) => signed_block_address(*true_target) > signed_block_address(*exit_target),
        (IrStatement::Leave(_), IrStatement::Jump(_)) => true,
        (IrStatement::Jump(IrJump { target_address }), IrStatement::Leave(_)) => {
            incoming_edge_count(container, *target_address) > 1
        }
        _ => false,
    }
}

fn is_usable_branch_to_child(
    statement: &IrStatement,
    container: &IrBlockContainer,
    cfg: &ControlFlowGraph,
    cfg_node_index: usize,
    current_block_addr: u32,
) -> bool {
    let Some(target_addr) = as_jump_target_addr(statement) else {
        return false;
    };

    if target_addr == current_block_addr {
        return false;
    }

    if !container
        .blocks
        .iter()
        .any(|block| block.start_address == target_addr)
    {
        return false;
    }

    if incoming_edge_count(container, target_addr) != 1 {
        return false;
    }

    let Some(target_node_index) = cfg
        .nodes
        .iter()
        .position(|node| node.block_start_address == Some(target_addr))
    else {
        return false;
    };

    dominates(cfg, cfg_node_index, target_node_index)
}

fn dominates(cfg: &ControlFlowGraph, dominator: usize, dominated: usize) -> bool {
    if dominator == dominated {
        return true;
    }

    let mut current = Some(dominated);
    while let Some(index) = current {
        if index == dominator {
            return true;
        }
        current = cfg.nodes[index].immediate_dominator;
    }

    false
}

fn incoming_edge_count(container: &IrBlockContainer, target_address: u32) -> usize {
    container
        .blocks
        .iter()
        .map(|block| count_targets_in_statements(&block.statements, target_address))
        .sum()
}

fn count_targets_in_statements(statements: &[IrStatement], target_address: u32) -> usize {
    statements
        .iter()
        .map(|statement| count_targets_in_statement(statement, target_address))
        .sum()
}

fn count_targets_in_statement(statement: &IrStatement, target_address: u32) -> usize {
    match statement {
        IrStatement::Jump(IrJump {
            target_address: addr,
        }) => usize::from(*addr == target_address),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut count = count_targets_in_statement(true_statement, target_address);
            if let Some(false_statement) = false_statement.as_ref() {
                count += count_targets_in_statement(false_statement, target_address);
            }
            count
        }
        IrStatement::Block(block) => count_targets_in_statements(&block.statements, target_address),
        IrStatement::BlockContainer(container) => container
            .blocks
            .iter()
            .map(|block| count_targets_in_statements(&block.statements, target_address))
            .sum(),
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => sections
            .iter()
            .map(|section| count_targets_in_statements(&section.body.statements, target_address))
            .sum(),
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => body
            .blocks
            .iter()
            .map(|block| count_targets_in_statements(&block.statements, target_address))
            .sum(),
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => body
            .blocks
            .iter()
            .map(|block| count_targets_in_statements(&block.statements, target_address))
            .sum(),
        IrStatement::Switch(IrSwitch {
            cases,
            default_target,
            ..
        }) => {
            let case_count = cases
                .values()
                .filter(|addr| **addr == target_address)
                .count();
            let default_count = usize::from(default_target == &Some(target_address));
            case_count + default_count
        }
        _ => 0,
    }
}

fn has_unreachable_endpoint(block: &IrBlock) -> bool {
    block.statements.last().is_some_and(is_branch_or_leave)
}

fn contains_non_exit_statement(statements: &[IrStatement]) -> bool {
    statements
        .iter()
        .any(|statement| !is_branch_or_leave(statement))
}

fn last_exit_statement(block: &IrBlock) -> Option<IrStatement> {
    block.statements.last().cloned().filter(is_branch_or_leave)
}

fn unpack_block_containing_only_branch(statement: &IrStatement) -> IrStatement {
    if let IrStatement::Block(block) = statement
        && block.statements.len() == 1
        && is_branch_or_leave(&block.statements[0])
    {
        return block.statements[0].clone();
    }
    statement.clone()
}

fn is_branch_or_leave(statement: &IrStatement) -> bool {
    matches!(statement, IrStatement::Jump(_) | IrStatement::Leave(_))
}

fn as_jump_target_addr(statement: &IrStatement) -> Option<u32> {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => Some(*target_address),
        _ => None,
    }
}

fn is_empty(statement: &IrStatement) -> bool {
    matches!(statement, IrStatement::Block(block) if block.statements.is_empty())
}

fn empty_block(state: &mut ConditionDetectionState) -> IrStatement {
    IrStatement::Block(IrBlock {
        statements: Vec::new(),
        start_address: state.next_synthetic_block_address(),
        should_emit_label: false,
    })
}

fn statement_start(statement: &IrStatement) -> i32 {
    if let IrStatement::Block(block) = statement {
        signed_block_address(block.start_address)
    } else {
        i32::MAX
    }
}

fn signed_block_address(address: u32) -> i32 {
    address as i32
}

fn logic_not(expression: IrExpression) -> IrExpression {
    if let IrExpression::OperatorCall(IrOperatorCallExpression {
        arguments,
        operator,
        ..
    }) = &expression
        && *operator == IrOperator::UnaryNegation
        && arguments.len() == 1
    {
        return arguments[0].clone();
    }

    IrExpression::OperatorCall(IrOperatorCallExpression {
        arguments: vec![expression],
        operator: IrOperator::UnaryNegation,
    })
}

fn logic_and(lhs: IrExpression, rhs: IrExpression) -> IrExpression {
    IrExpression::OperatorCall(IrOperatorCallExpression {
        arguments: vec![lhs, rhs],
        operator: IrOperator::LogicalAnd,
    })
}

fn find_block_index(container: &IrBlockContainer, start_address: u32) -> Option<usize> {
    container
        .blocks
        .iter()
        .position(|block| block.start_address == start_address)
}

fn get_block_clone_by_address(container: &IrBlockContainer, start_address: u32) -> Option<IrBlock> {
    container
        .blocks
        .iter()
        .find(|block| block.start_address == start_address)
        .cloned()
}

fn remove_block_by_address(
    container: &mut IrBlockContainer,
    start_address: u32,
) -> Option<IrBlock> {
    let index = find_block_index(container, start_address)?;
    Some(container.blocks.remove(index))
}
