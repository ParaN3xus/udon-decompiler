use std::collections::HashSet;

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlockContainer, IrContainerKind, IrFunction, IrHighLevelDoWhile, IrHighLevelSwitch,
    IrHighLevelWhile, IrIf, IrJump, IrLeave, IrReturn, IrStatement,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct CollectLabelUsage;

impl ITransform for CollectLabelUsage {

    fn run(&self, function: &mut IrFunction, _context: &mut TransformContext<'_, '_>) -> Result<()> {
        reset_flags(&mut function.body);

        let mut label_targets = HashSet::<u32>::new();
        let mut exit_label_targets = HashSet::<u32>::new();

        analyze_container(
            &function.body,
            None,
            None,
            function.body.id,
            &mut label_targets,
            &mut exit_label_targets,
        );

        apply_flags(&mut function.body, &label_targets, &exit_label_targets);

        Ok(())
    }
}

fn reset_flags(container: &mut IrBlockContainer) {
    container.should_emit_exit_label = false;
    for block in &mut container.blocks {
        block.should_emit_label = false;
        for statement in &mut block.statements {
            reset_statement_flags(statement);
        }
    }
}

fn reset_statement_flags(statement: &mut IrStatement) {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            reset_statement_flags(true_statement);
            if let Some(false_statement) = false_statement.as_mut() {
                reset_statement_flags(false_statement);
            }
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                section.body.should_emit_label = false;
                for nested in &mut section.body.statements {
                    reset_statement_flags(nested);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => reset_flags(body),
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => reset_flags(body),
        IrStatement::Block(block) => {
            block.should_emit_label = false;
            for nested in &mut block.statements {
                reset_statement_flags(nested);
            }
        }
        IrStatement::BlockContainer(container) => reset_flags(container),
        _ => {}
    }
}

fn analyze_container(
    container: &IrBlockContainer,
    break_target: Option<u32>,
    continue_target: Option<u32>,
    function_body_id: u32,
    label_targets: &mut HashSet<u32>,
    exit_label_targets: &mut HashSet<u32>,
) {
    if analyze_while_like(
        container,
        break_target,
        continue_target,
        function_body_id,
        label_targets,
        exit_label_targets,
    ) {
        return;
    }

    if analyze_do_while_like(
        container,
        break_target,
        continue_target,
        function_body_id,
        label_targets,
        exit_label_targets,
    ) {
        return;
    }

    for index in 0..container.blocks.len() {
        let block = &container.blocks[index];
        let next_block = container.blocks.get(index + 1).map(|x| x.start_address);

        for stmt_index in 0..block.statements.len() {
            let statement = &block.statements[stmt_index];
            let statement_next_block = if stmt_index + 1 == block.statements.len() {
                next_block
            } else {
                None
            };
            analyze_statement(
                statement,
                container,
                statement_next_block,
                break_target,
                continue_target,
                function_body_id,
                label_targets,
                exit_label_targets,
            );
        }
    }
}

fn analyze_while_like(
    container: &IrBlockContainer,
    _break_target: Option<u32>,
    continue_target: Option<u32>,
    function_body_id: u32,
    label_targets: &mut HashSet<u32>,
    exit_label_targets: &mut HashSet<u32>,
) -> bool {
    if container.kind != IrContainerKind::While {
        return false;
    }

    let Some(entry) = container.entry_block() else {
        return false;
    };
    if container.blocks.len() != 1 || entry.statements.is_empty() {
        return false;
    }

    let Some(IrStatement::If(IrIf {
        true_statement,
        false_statement,
        ..
    })) = entry.statements.last()
    else {
        return false;
    };
    let Some(false_statement) = false_statement.as_ref() else {
        return false;
    };
    if !is_leave_to_container(false_statement, container.id) {
        return false;
    }

    let IrStatement::Block(true_block) = true_statement.as_ref() else {
        return false;
    };

    let mut body_statements = entry.statements[..entry.statements.len() - 1].to_vec();
    body_statements.extend(true_block.statements.clone());

    for statement in &body_statements {
        analyze_statement(
            statement,
            container,
            None,
            Some(container.id),
            continue_target,
            function_body_id,
            label_targets,
            exit_label_targets,
        );
    }

    true
}

fn analyze_do_while_like(
    container: &IrBlockContainer,
    _break_target: Option<u32>,
    continue_target: Option<u32>,
    function_body_id: u32,
    label_targets: &mut HashSet<u32>,
    exit_label_targets: &mut HashSet<u32>,
) -> bool {
    if container.kind != IrContainerKind::DoWhile {
        return false;
    }

    let Some(entry) = container.entry_block() else {
        return false;
    };
    if container.blocks.len() != 1 || entry.statements.is_empty() {
        return false;
    }

    let Some(IrStatement::If(IrIf {
        true_statement,
        false_statement,
        ..
    })) = entry.statements.last()
    else {
        return false;
    };
    let Some(false_statement) = false_statement.as_ref() else {
        return false;
    };
    if !is_leave_to_container(false_statement, container.id) {
        return false;
    }
    if !is_branch_to_entry(
        true_statement,
        container.entry_block().map(|x| x.start_address),
    ) {
        return false;
    }

    for statement in &entry.statements[..entry.statements.len() - 1] {
        analyze_statement(
            statement,
            container,
            None,
            Some(container.id),
            continue_target,
            function_body_id,
            label_targets,
            exit_label_targets,
        );
    }

    true
}

#[allow(clippy::too_many_arguments)]
fn analyze_statement(
    statement: &IrStatement,
    current_container: &IrBlockContainer,
    next_block: Option<u32>,
    break_target: Option<u32>,
    continue_target: Option<u32>,
    function_body_id: u32,
    label_targets: &mut HashSet<u32>,
    exit_label_targets: &mut HashSet<u32>,
) {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            analyze_branch_statement(
                true_statement,
                current_container,
                break_target,
                continue_target,
                function_body_id,
                label_targets,
                exit_label_targets,
            );
            if let Some(false_statement) = false_statement.as_ref() {
                analyze_branch_statement(
                    false_statement,
                    current_container,
                    break_target,
                    continue_target,
                    function_body_id,
                    label_targets,
                    exit_label_targets,
                );
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile {
            body,
            break_target: while_break,
            continue_target: while_continue,
            ..
        }) => {
            analyze_container(
                body,
                Some(*while_break),
                Some(*while_continue),
                function_body_id,
                label_targets,
                exit_label_targets,
            );
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile {
            body,
            break_target: loop_break,
            continue_target: loop_continue,
            ..
        }) => {
            analyze_container(
                body,
                Some(*loop_break),
                Some(*loop_continue),
                function_body_id,
                label_targets,
                exit_label_targets,
            );
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for nested in &section.body.statements {
                    analyze_statement(
                        nested,
                        current_container,
                        None,
                        None,
                        None,
                        function_body_id,
                        label_targets,
                        exit_label_targets,
                    );
                }
            }
        }
        IrStatement::Jump(IrJump { target_address }) => {
            if continue_target.is_some_and(|x| x == *target_address) {
                return;
            }
            if next_block.is_some_and(|x| x == *target_address) {
                return;
            }
            if !current_container
                .blocks
                .iter()
                .any(|block| block.start_address == *target_address)
            {
                return;
            }
            label_targets.insert(*target_address);
        }
        IrStatement::Leave(IrLeave {
            target_container_id,
        }) => {
            if *target_container_id == function_body_id {
                return;
            }
            if break_target.is_some_and(|x| x == *target_container_id) {
                return;
            }
            exit_label_targets.insert(*target_container_id);
        }
        IrStatement::Return(IrReturn) => {}
        IrStatement::Block(block) => {
            for nested in &block.statements {
                analyze_statement(
                    nested,
                    current_container,
                    None,
                    break_target,
                    continue_target,
                    function_body_id,
                    label_targets,
                    exit_label_targets,
                );
            }
        }
        IrStatement::BlockContainer(container) => {
            analyze_container(
                container,
                break_target,
                continue_target,
                function_body_id,
                label_targets,
                exit_label_targets,
            );
        }
        _ => {}
    }
}

fn analyze_branch_statement(
    statement: &IrStatement,
    current_container: &IrBlockContainer,
    break_target: Option<u32>,
    continue_target: Option<u32>,
    function_body_id: u32,
    label_targets: &mut HashSet<u32>,
    exit_label_targets: &mut HashSet<u32>,
) {
    if let IrStatement::Block(block) = statement
        && current_container
            .blocks
            .iter()
            .any(|x| x.start_address == block.start_address)
    {
        label_targets.insert(block.start_address);
        return;
    }

    analyze_statement(
        statement,
        current_container,
        None,
        break_target,
        continue_target,
        function_body_id,
        label_targets,
        exit_label_targets,
    );
}

fn apply_flags(
    container: &mut IrBlockContainer,
    label_targets: &HashSet<u32>,
    exit_label_targets: &HashSet<u32>,
) {
    container.should_emit_exit_label = exit_label_targets.contains(&container.id);
    for block in &mut container.blocks {
        block.should_emit_label = label_targets.contains(&block.start_address);
        for statement in &mut block.statements {
            apply_flags_statement(statement, label_targets, exit_label_targets);
        }
    }
}

fn apply_flags_statement(
    statement: &mut IrStatement,
    label_targets: &HashSet<u32>,
    exit_label_targets: &HashSet<u32>,
) {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            apply_flags_statement(true_statement, label_targets, exit_label_targets);
            if let Some(false_statement) = false_statement.as_mut() {
                apply_flags_statement(false_statement, label_targets, exit_label_targets);
            }
        }
        IrStatement::Block(block) => {
            block.should_emit_label = label_targets.contains(&block.start_address);
            for nested in &mut block.statements {
                apply_flags_statement(nested, label_targets, exit_label_targets);
            }
        }
        IrStatement::BlockContainer(container) => {
            apply_flags(container, label_targets, exit_label_targets)
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                section.body.should_emit_label =
                    label_targets.contains(&section.body.start_address);
                for nested in &mut section.body.statements {
                    apply_flags_statement(nested, label_targets, exit_label_targets);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            apply_flags(body, label_targets, exit_label_targets)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            apply_flags(body, label_targets, exit_label_targets)
        }
        _ => {}
    }
}

fn is_leave_to_container(statement: &IrStatement, container_id: u32) -> bool {
    matches!(
        statement,
        IrStatement::Leave(IrLeave { target_container_id }) if *target_container_id == container_id
    )
}

fn is_branch_to_entry(statement: &IrStatement, entry_address: Option<u32>) -> bool {
    let Some(entry_address) = entry_address else {
        return false;
    };

    match statement {
        IrStatement::Jump(IrJump { target_address }) => *target_address == entry_address,
        IrStatement::Block(block) if block.statements.len() == 1 => matches!(
            block.statements.first(),
            Some(IrStatement::Jump(IrJump { target_address })) if *target_address == entry_address
        ),
        _ => false,
    }
}
