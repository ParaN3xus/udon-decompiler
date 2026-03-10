use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrContainerKind, IrExpression, IrFunction, IrIf, IrJump, IrLeave,
    IrOperator, IrOperatorCallExpression, IrStatement, IrSwitch,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct HighLevelLoopTransform;

impl ITransform for HighLevelLoopTransform {

    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let mut state = LoopTransformState::from_context(context);
        rewrite_container(&mut function.body, &mut state);
        state.commit(context);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct LoopTransformState {
    synthetic_next: i64,
}

impl LoopTransformState {
    fn from_context(context: &TransformContext<'_, '_>) -> Self {
        let current = context
            .program_context
            .metadata
            .get("_synthetic_block_addr")
            .copied()
            .unwrap_or(-1);
        Self {
            synthetic_next: current,
        }
    }

    fn next_synthetic_block_address(&mut self) -> u32 {
        let current = self.synthetic_next;
        self.synthetic_next -= 1;
        (current as i32) as u32
    }

    fn commit(self, context: &mut TransformContext<'_, '_>) {
        context
            .program_context
            .metadata
            .insert("_synthetic_block_addr".to_string(), self.synthetic_next);
    }
}

fn rewrite_container(container: &mut IrBlockContainer, state: &mut LoopTransformState) {
    for block in &mut container.blocks {
        for statement in &mut block.statements {
            rewrite_statement(statement, state);
        }
    }
}

fn rewrite_statement(statement: &mut IrStatement, state: &mut LoopTransformState) {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            rewrite_statement(true_statement, state);
            if let Some(false_statement) = false_statement.as_mut() {
                rewrite_statement(false_statement, state);
            }
        }
        IrStatement::Block(block) => {
            for nested in &mut block.statements {
                rewrite_statement(nested, state);
            }
        }
        IrStatement::BlockContainer(container) => {
            rewrite_container(container, state);
            if container.kind == IrContainerKind::Loop && match_while_loop(container) {
                return;
            }

            if container.kind == IrContainerKind::Loop {
                let _ = match_do_while_loop(container, state);
            }
        }
        _ => {}
    }
}

fn match_while_loop(loop_container: &mut IrBlockContainer) -> bool {
    let Some(entry_snapshot) = loop_container.entry_block().cloned() else {
        return false;
    };
    let (if_inst, exit_inst) = if entry_snapshot.statements.len() == 2 {
        let IrStatement::If(if_inst) = &entry_snapshot.statements[0] else {
            return false;
        };
        if if_inst.false_statement.is_some() {
            return false;
        }
        if !is_leave_to_container(&entry_snapshot.statements[1], loop_container.id) {
            return false;
        }
        (if_inst.clone(), entry_snapshot.statements[1].clone())
    } else if entry_snapshot.statements.len() == 1 {
        let IrStatement::If(if_inst) = &entry_snapshot.statements[0] else {
            return false;
        };
        let Some(false_stmt) = if_inst.false_statement.as_ref() else {
            return false;
        };
        if !is_leave_to_container(false_stmt, loop_container.id) {
            return false;
        }
        let mut normalized_if = if_inst.clone();
        normalized_if.false_statement = None;
        (normalized_if, *false_stmt.clone())
    } else {
        return false;
    };

    let mut true_target = as_jump_target_addr(&if_inst.true_statement);
    if true_target.is_none() && matches!(if_inst.true_statement.as_ref(), IrStatement::Block(_)) {
        let IrStatement::Block(block) = if_inst.true_statement.as_ref() else {
            return false;
        };
        true_target = Some(block.start_address);
        if !loop_container
            .blocks
            .iter()
            .any(|x| x.start_address == block.start_address)
        {
            loop_container.blocks.insert(1, block.clone());
        }
        if let Some(loop_body_mut) = loop_container
            .blocks
            .iter_mut()
            .find(|x| x.start_address == block.start_address)
            && !has_unreachable_endpoint(loop_body_mut)
        {
            loop_body_mut.statements.push(IrStatement::Leave(IrLeave {
                target_container_id: loop_container.id,
            }));
        }
    }

    let Some(loop_body_address) = true_target else {
        return false;
    };
    if !loop_container
        .blocks
        .iter()
        .any(|x| x.start_address == loop_body_address)
    {
        return false;
    }

    let mut new_if = if_inst;
    new_if.true_statement = Box::new(IrStatement::Jump(IrJump {
        target_address: loop_body_address,
    }));
    new_if.false_statement = Some(Box::new(exit_inst));

    if let Some(entry_mut) = loop_container.blocks.first_mut() {
        entry_mut.statements = vec![IrStatement::If(new_if)];
    }

    loop_container.kind = IrContainerKind::While;

    while unwrap_while_tail_condition(loop_container, loop_body_address) {}

    true
}

fn unwrap_while_tail_condition(
    loop_container: &mut IrBlockContainer,
    loop_body_address: u32,
) -> bool {
    let Some(body_index) = loop_container
        .blocks
        .iter()
        .position(|x| x.start_address == loop_body_address)
    else {
        return false;
    };

    let body = &mut loop_container.blocks[body_index];
    if body.statements.len() < 2 {
        return false;
    }

    let leave_inst = body.statements.last().cloned();
    let nested = body.statements.get(body.statements.len() - 2).cloned();

    let Some(IrStatement::Leave(IrLeave {
        target_container_id,
    })) = leave_inst.clone()
    else {
        return false;
    };
    if target_container_id != loop_container.id {
        return false;
    }

    let Some(IrStatement::If(mut nested_if)) = nested else {
        return false;
    };
    if nested_if.false_statement.is_some() {
        return false;
    }

    match nested_if.true_statement.as_ref() {
        IrStatement::Block(nested_block) => {
            body.statements.pop();
            body.statements
                .truncate(body.statements.len().saturating_sub(1));
            body.statements.extend(nested_block.statements.clone());
        }
        IrStatement::Jump(jump) => {
            let len = body.statements.len();
            body.statements[len - 1] = IrStatement::Jump(jump.clone());
        }
        _ => return false,
    }

    nested_if.condition = logic_not(nested_if.condition);
    nested_if.true_statement = Box::new(IrStatement::Leave(IrLeave {
        target_container_id: loop_container.id,
    }));

    if !has_unreachable_endpoint(body) {
        body.statements.push(IrStatement::Leave(IrLeave {
            target_container_id: loop_container.id,
        }));
    }

    true
}

fn match_do_while_loop(
    loop_container: &mut IrBlockContainer,
    state: &mut LoopTransformState,
) -> bool {
    if loop_container.blocks.is_empty() {
        return false;
    }

    for idx in (0..loop_container.blocks.len()).rev() {
        if loop_container.blocks[idx].statements.len() < 2 {
            continue;
        }

        let Some(entry_address) = loop_container.entry_block().map(|x| x.start_address) else {
            continue;
        };

        let block_snapshot = loop_container.blocks[idx].clone();
        let len = block_snapshot.statements.len();
        let last = block_snapshot.statements[len - 1].clone();
        let maybe_if = block_snapshot.statements[len - 2].clone();
        let IrStatement::If(base_if) = maybe_if else {
            continue;
        };
        if base_if.false_statement.is_some() {
            continue;
        }

        let swap_branches = if is_jump_to_entry(&last, loop_container, entry_address)
            && is_leave_to_container(base_if.true_statement.as_ref(), loop_container.id)
        {
            true
        } else if is_leave_to_container(&last, loop_container.id)
            && is_jump_to_entry(
                base_if.true_statement.as_ref(),
                loop_container,
                entry_address,
            )
        {
            false
        } else {
            continue;
        };

        let conditions = collect_do_while_conditions(
            &block_snapshot,
            loop_container,
            loop_container.id,
            entry_address,
            swap_branches,
        );
        if conditions.is_empty() {
            continue;
        }

        let split = block_snapshot.start_address == entry_address
            || block_snapshot.statements.len() > conditions.len() + 1;

        let start = block_snapshot
            .statements
            .len()
            .saturating_sub(conditions.len() + 1);

        let exit_statement = last.clone();

        let condition_block_index = if split {
            let new_block_address = state.next_synthetic_block_address();
            let new_block = IrBlock {
                statements: Vec::new(),
                start_address: new_block_address,
                should_emit_label: false,
            };

            if let Some(block_mut) = loop_container.blocks.get_mut(idx) {
                block_mut.statements.truncate(start);
                block_mut.statements.push(IrStatement::Jump(IrJump {
                    target_address: new_block_address,
                }));
            }

            loop_container.blocks.push(new_block);
            loop_container.blocks.len() - 1
        } else if idx + 1 != loop_container.blocks.len() {
            let block = loop_container.blocks.remove(idx);
            loop_container.blocks.push(block);
            loop_container.blocks.len() - 1
        } else {
            idx
        };

        let mut combined_if = conditions[0].clone();
        if swap_branches {
            combined_if.condition = logic_not(combined_if.condition);
            let old_true = combined_if.true_statement;
            combined_if.false_statement = Some(old_true);
            combined_if.true_statement = Box::new(exit_statement.clone());
        } else {
            combined_if.false_statement = Some(Box::new(exit_statement.clone()));
        }

        for inst in conditions.iter().skip(1) {
            if swap_branches {
                combined_if.condition =
                    logic_and(logic_not(inst.condition.clone()), combined_if.condition);
            } else {
                combined_if.condition = logic_and(inst.condition.clone(), combined_if.condition);
            }
        }

        if let Some(condition_block) = loop_container.blocks.get_mut(condition_block_index) {
            condition_block
                .statements
                .push(IrStatement::If(combined_if));
        }

        loop_container.kind = IrContainerKind::DoWhile;
        return true;
    }

    false
}

fn collect_do_while_conditions(
    block: &IrBlock,
    loop_container: &IrBlockContainer,
    loop_container_id: u32,
    entry_address: u32,
    swap_branches: bool,
) -> Vec<IrIf> {
    let mut conditions = Vec::<IrIf>::new();
    let mut index = block.statements.len().saturating_sub(2);
    loop {
        let Some(statement) = block.statements.get(index) else {
            break;
        };
        let IrStatement::If(if_inst) = statement else {
            break;
        };
        if if_inst.false_statement.is_some() {
            break;
        }

        let shape_ok = if swap_branches {
            is_leave_to_container(if_inst.true_statement.as_ref(), loop_container_id)
        } else {
            is_jump_to_entry(
                if_inst.true_statement.as_ref(),
                loop_container,
                entry_address,
            )
        };
        if !shape_ok {
            break;
        }

        conditions.push(if_inst.clone());
        if index == 0 {
            break;
        }
        index -= 1;
    }
    conditions
}

fn as_jump_target_addr(statement: &IrStatement) -> Option<u32> {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => Some(*target_address),
        IrStatement::Block(block) if block.statements.len() == 1 => {
            match block.statements.first() {
                Some(IrStatement::Jump(IrJump { target_address })) => Some(*target_address),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_jump_to_entry(
    statement: &IrStatement,
    loop_container: &IrBlockContainer,
    entry_address: u32,
) -> bool {
    if loop_container
        .blocks
        .iter()
        .filter(|block| block.start_address == entry_address)
        .count()
        != 1
    {
        return false;
    }
    as_jump_target_addr(statement).is_some_and(|x| x == entry_address)
}

fn is_leave_to_container(statement: &IrStatement, container_id: u32) -> bool {
    match statement {
        IrStatement::Leave(IrLeave {
            target_container_id,
        }) => *target_container_id == container_id,
        IrStatement::Block(block) if block.statements.len() == 1 => {
            matches!(
                block.statements.first(),
                Some(IrStatement::Leave(IrLeave { target_container_id })) if *target_container_id == container_id
            )
        }
        _ => false,
    }
}

fn has_unreachable_endpoint(block: &IrBlock) -> bool {
    block
        .statements
        .last()
        .is_some_and(statement_unreachable_endpoint)
}

fn statement_unreachable_endpoint(statement: &IrStatement) -> bool {
    match statement {
        IrStatement::Jump(_)
        | IrStatement::Leave(_)
        | IrStatement::Return(_)
        | IrStatement::Switch(IrSwitch { .. }) => true,
        IrStatement::Block(block) => block
            .statements
            .last()
            .is_some_and(statement_unreachable_endpoint),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => false_statement.as_ref().is_some_and(|x| {
            statement_unreachable_endpoint(true_statement) && statement_unreachable_endpoint(x)
        }),
        _ => false,
    }
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
