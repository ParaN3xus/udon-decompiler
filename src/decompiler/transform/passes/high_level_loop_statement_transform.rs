use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrContainerKind, IrFunction, IrHighLevelDoWhile, IrHighLevelWhile,
    IrJump, IrLeave, IrStatement,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct HighLevelLoopStatementTransform;

impl ITransform for HighLevelLoopStatementTransform {

    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let mut state = LoopStatementState::from_context(function, context);
        rewrite_container(&mut function.body, &mut state);
        state.commit(context);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct LoopStatementState {
    next_container_id: u32,
    synthetic_block_address: i64,
}

impl LoopStatementState {
    fn from_context(function: &IrFunction, context: &TransformContext<'_, '_>) -> Self {
        let current = context
            .program_context
            .metadata
            .get("_synthetic_block_addr")
            .copied()
            .unwrap_or(-1);
        Self {
            next_container_id: max_container_id(&function.body).saturating_add(1),
            synthetic_block_address: current,
        }
    }

    fn next_container_id(&mut self) -> u32 {
        let current = self.next_container_id;
        self.next_container_id = self.next_container_id.saturating_add(1);
        current
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

fn rewrite_container(container: &mut IrBlockContainer, state: &mut LoopStatementState) {
    for block in &mut container.blocks {
        let mut rewritten = Vec::<IrStatement>::with_capacity(block.statements.len());
        for statement in block.statements.drain(..) {
            rewritten.push(rewrite_statement(statement, state));
        }
        block.statements = rewritten;
    }
}

fn rewrite_statement(statement: IrStatement, state: &mut LoopStatementState) -> IrStatement {
    match statement {
        IrStatement::If(mut if_stmt) => {
            if_stmt.true_statement = Box::new(rewrite_statement(*if_stmt.true_statement, state));
            if let Some(false_statement) = if_stmt.false_statement.take() {
                if_stmt.false_statement =
                    Some(Box::new(rewrite_statement(*false_statement, state)));
            }
            IrStatement::If(if_stmt)
        }
        IrStatement::Block(mut block) => {
            let mut rewritten = Vec::<IrStatement>::with_capacity(block.statements.len());
            for nested in block.statements.drain(..) {
                rewritten.push(rewrite_statement(nested, state));
            }
            block.statements = rewritten;
            IrStatement::Block(block)
        }
        IrStatement::HighLevelSwitch(mut switch_stmt) => {
            for section in &mut switch_stmt.sections {
                let rewritten = rewrite_statement(IrStatement::Block(section.body.clone()), state);
                if let IrStatement::Block(block) = rewritten {
                    section.body = block;
                }
            }
            IrStatement::HighLevelSwitch(switch_stmt)
        }
        IrStatement::BlockContainer(mut container) => {
            rewrite_container(&mut container, state);
            lift_container(container, state)
        }
        _ => statement,
    }
}

fn lift_container(container: IrBlockContainer, state: &mut LoopStatementState) -> IrStatement {
    match container.kind {
        IrContainerKind::Loop => lift_infinite_loop(container, state),
        IrContainerKind::While => lift_while(container, state),
        IrContainerKind::DoWhile => lift_do_while(container, state),
        _ => IrStatement::BlockContainer(container),
    }
}

fn lift_infinite_loop(
    container: IrBlockContainer,
    state: &mut LoopStatementState,
) -> IrStatement {
    let Some(entry) = container.entry_block().cloned() else {
        return IrStatement::BlockContainer(container);
    };

    let mut body = IrBlockContainer {
        id: state.next_container_id(),
        blocks: container.blocks.clone(),
        kind: IrContainerKind::Block,
        should_emit_exit_label: container.should_emit_exit_label,
    };
    remove_trailing_continue_jump(&mut body, entry.start_address);

    IrStatement::HighLevelWhile(IrHighLevelWhile {
        condition: None,
        body,
        continue_target: entry.start_address,
        break_target: container.id,
    })
}

fn lift_while(container: IrBlockContainer, state: &mut LoopStatementState) -> IrStatement {
    let Some(entry) = container.entry_block().cloned() else {
        return IrStatement::BlockContainer(container);
    };
    if entry.statements.len() != 1 {
        return IrStatement::BlockContainer(container);
    }

    let IrStatement::If(if_stmt) = &entry.statements[0] else {
        return IrStatement::BlockContainer(container);
    };
    let Some(false_statement) = if_stmt.false_statement.as_ref() else {
        return IrStatement::BlockContainer(container);
    };
    if !is_leave_to_container(false_statement.as_ref(), container.id) {
        return IrStatement::BlockContainer(container);
    }

    let mut inline_loop_body_block: Option<IrBlock> = None;
    let mut loop_body_target: Option<IrBlock> = None;

    if let Some(target_addr) = as_jump_target_addr(if_stmt.true_statement.as_ref()) {
        if let Some(target_block) = container
            .blocks
            .iter()
            .find(|block| block.start_address == target_addr)
            .cloned()
        {
            loop_body_target = Some(target_block);
        } else {
            return IrStatement::BlockContainer(container);
        }
    } else if let IrStatement::Block(inline_block) = if_stmt.true_statement.as_ref() {
        inline_loop_body_block = Some(IrBlock {
            statements: inline_block.statements.clone(),
            start_address: state.next_synthetic_block_address(),
            should_emit_label: false,
        });
    } else {
        return IrStatement::BlockContainer(container);
    }

    if loop_body_target
        .as_ref()
        .is_some_and(|target| target.start_address == entry.start_address)
    {
        return IrStatement::BlockContainer(container);
    }

    let mut entry_anchor = entry.clone();
    entry_anchor.statements.clear();

    let mut body_blocks = Vec::<IrBlock>::new();
    if let Some(inline_loop_body_block) = inline_loop_body_block {
        body_blocks.push(inline_loop_body_block);
    } else if let Some(loop_body_target) = loop_body_target.as_ref() {
        body_blocks.push(loop_body_target.clone());
    }

    for block in container.blocks.iter().skip(1) {
        if loop_body_target
            .as_ref()
            .is_some_and(|target| target.start_address == block.start_address)
        {
            continue;
        }
        body_blocks.push(block.clone());
    }

    body_blocks.push(entry_anchor.clone());

    let mut body_container = IrBlockContainer {
        id: state.next_container_id(),
        blocks: body_blocks,
        kind: IrContainerKind::Block,
        should_emit_exit_label: container.should_emit_exit_label,
    };
    remove_trailing_continue_jump(&mut body_container, entry_anchor.start_address);

    IrStatement::HighLevelWhile(IrHighLevelWhile {
        condition: Some(if_stmt.condition.clone()),
        body: body_container,
        continue_target: entry_anchor.start_address,
        break_target: container.id,
    })
}

fn lift_do_while(container: IrBlockContainer, state: &mut LoopStatementState) -> IrStatement {
    let Some(entry) = container.entry_block().cloned() else {
        return IrStatement::BlockContainer(container);
    };
    if container.blocks.is_empty() {
        return IrStatement::BlockContainer(container);
    }

    let Some(condition_block) = container.blocks.last().cloned() else {
        return IrStatement::BlockContainer(container);
    };
    if condition_block.statements.is_empty() {
        return IrStatement::BlockContainer(container);
    }

    let Some(IrStatement::If(if_stmt)) = condition_block.statements.last() else {
        return IrStatement::BlockContainer(container);
    };
    let Some(false_statement) = if_stmt.false_statement.as_ref() else {
        return IrStatement::BlockContainer(container);
    };
    if !is_leave_to_container(false_statement.as_ref(), container.id) {
        return IrStatement::BlockContainer(container);
    }
    if !is_jump_to_entry(if_stmt.true_statement.as_ref(), entry.start_address) {
        return IrStatement::BlockContainer(container);
    }

    let mut body_blocks = container.blocks[..container.blocks.len() - 1].to_vec();
    let condition_prefix = condition_block
        .statements
        .iter()
        .take(condition_block.statements.len().saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>();

    if !condition_prefix.is_empty() {
        body_blocks.push(IrBlock {
            statements: condition_prefix,
            start_address: state.next_synthetic_block_address(),
            should_emit_label: false,
        });
    }

    let mut condition_anchor = condition_block.clone();
    condition_anchor.statements.clear();
    body_blocks.push(condition_anchor.clone());

    let mut body_container = IrBlockContainer {
        id: state.next_container_id(),
        blocks: body_blocks,
        kind: IrContainerKind::Block,
        should_emit_exit_label: container.should_emit_exit_label,
    };
    remove_trailing_continue_jump(&mut body_container, condition_anchor.start_address);

    IrStatement::HighLevelDoWhile(IrHighLevelDoWhile {
        condition: if_stmt.condition.clone(),
        body: body_container,
        continue_target: condition_anchor.start_address,
        break_target: container.id,
    })
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

fn is_leave_to_container(statement: &IrStatement, container_id: u32) -> bool {
    matches!(
        statement,
        IrStatement::Leave(IrLeave { target_container_id }) if *target_container_id == container_id
    )
}

fn is_jump_to_entry(statement: &IrStatement, entry_address: u32) -> bool {
    as_jump_target_addr(statement).is_some_and(|target| target == entry_address)
}

fn remove_trailing_continue_jump(body: &mut IrBlockContainer, continue_target: u32) {
    for block in body.blocks.iter_mut().rev() {
        if block.statements.is_empty() {
            continue;
        }
        let remove = matches!(
            block.statements.last(),
            Some(IrStatement::Jump(IrJump { target_address })) if *target_address == continue_target
        );
        if remove {
            block.statements.pop();
        }
        break;
    }
}

fn max_container_id(container: &IrBlockContainer) -> u32 {
    let mut max_id = container.id;
    for block in &container.blocks {
        for statement in &block.statements {
            max_id = max_id.max(max_container_id_in_statement(statement));
        }
    }
    max_id
}

fn max_container_id_in_statement(statement: &IrStatement) -> u32 {
    match statement {
        IrStatement::Block(block) => block
            .statements
            .iter()
            .map(max_container_id_in_statement)
            .max()
            .unwrap_or(0),
        IrStatement::BlockContainer(container) => max_container_id(container),
        IrStatement::If(if_stmt) => {
            let mut max_id = max_container_id_in_statement(if_stmt.true_statement.as_ref());
            if let Some(false_statement) = if_stmt.false_statement.as_ref() {
                max_id = max_id.max(max_container_id_in_statement(false_statement.as_ref()));
            }
            max_id
        }
        IrStatement::HighLevelSwitch(switch_stmt) => switch_stmt
            .sections
            .iter()
            .flat_map(|section| section.body.statements.iter())
            .map(max_container_id_in_statement)
            .max()
            .unwrap_or(0),
        IrStatement::HighLevelWhile(while_stmt) => max_container_id(&while_stmt.body),
        IrStatement::HighLevelDoWhile(do_while_stmt) => max_container_id(&do_while_stmt.body),
        IrStatement::Assignment(_)
        | IrStatement::Expression(_)
        | IrStatement::VariableDeclaration(_)
        | IrStatement::Jump(_)
        | IrStatement::Leave(_)
        | IrStatement::Return(_)
        | IrStatement::Switch(_) => 0,
    }
}
