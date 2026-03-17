use std::collections::{HashMap, HashSet};

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrContainerKind, IrFunction, IrHighLevelSwitch,
    IrHighLevelSwitchSection, IrIf, IrJump, IrLeave, IrReturn, IrStatement, IrSwitch,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct HighLevelSwitchTransform;

impl ITransform for HighLevelSwitchTransform {
    fn run(
        &self,
        function: &mut IrFunction,
        _context: &mut TransformContext<'_, '_>,
    ) -> Result<()> {
        rewrite_container(&mut function.body);
        Ok(())
    }
}

fn rewrite_container(container: &mut IrBlockContainer) {
    let snapshot = container.clone();
    let mut consumed_blocks = HashSet::<u32>::new();

    for block in &mut container.blocks {
        for statement in &mut block.statements {
            let (rewritten, statement_consumed) =
                rewrite_statement(statement.clone(), &snapshot, block.start_address);
            consumed_blocks.extend(statement_consumed);
            *statement = rewritten;
        }
    }

    if !consumed_blocks.is_empty() {
        container
            .blocks
            .retain(|x| !consumed_blocks.contains(&x.start_address));
    }
}

fn rewrite_statement(
    statement: IrStatement,
    parent_container: &IrBlockContainer,
    parent_block_start: u32,
) -> (IrStatement, HashSet<u32>) {
    match statement {
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            let (true_statement, mut consumed) =
                rewrite_statement(*true_statement, parent_container, parent_block_start);
            let false_statement = if let Some(x) = false_statement {
                let (rewritten, false_consumed) =
                    rewrite_statement(*x, parent_container, parent_block_start);
                consumed.extend(false_consumed);
                Some(Box::new(rewritten))
            } else {
                None
            };
            (
                IrStatement::If(IrIf {
                    condition,
                    true_statement: Box::new(true_statement),
                    false_statement,
                }),
                consumed,
            )
        }
        IrStatement::Block(mut block) => {
            let mut consumed = HashSet::<u32>::new();
            let mut rewritten = Vec::<IrStatement>::with_capacity(block.statements.len());
            for nested in block.statements {
                let (statement, nested_consumed) =
                    rewrite_statement(nested, parent_container, block.start_address);
                consumed.extend(nested_consumed);
                rewritten.push(statement);
            }
            block.statements = rewritten;
            (IrStatement::Block(block), consumed)
        }
        IrStatement::BlockContainer(mut container) => {
            rewrite_container(&mut container);
            if container.kind == IrContainerKind::Switch {
                return lift_switch_container(&container, parent_container, parent_block_start);
            }
            (IrStatement::BlockContainer(container), HashSet::new())
        }
        other => (other, HashSet::new()),
    }
}

fn lift_switch_container(
    switch_container: &IrBlockContainer,
    parent_container: &IrBlockContainer,
    parent_block_start: u32,
) -> (IrStatement, HashSet<u32>) {
    let Some(entry) = switch_container.entry_block() else {
        return (
            IrStatement::BlockContainer(switch_container.clone()),
            HashSet::new(),
        );
    };
    if entry.statements.len() != 1 {
        return (
            IrStatement::BlockContainer(switch_container.clone()),
            HashSet::new(),
        );
    }
    let IrStatement::Switch(switch_inst) = &entry.statements[0] else {
        return (
            IrStatement::BlockContainer(switch_container.clone()),
            HashSet::new(),
        );
    };

    let mut target_to_labels = HashMap::<u32, Vec<u32>>::new();
    let mut target_order = Vec::<u32>::new();
    for (label_value, target) in &switch_inst.cases {
        if !target_to_labels.contains_key(target) {
            target_order.push(*target);
        }
        target_to_labels
            .entry(*target)
            .or_default()
            .push(*label_value);
    }
    if target_to_labels.is_empty() {
        return (
            IrStatement::BlockContainer(switch_container.clone()),
            HashSet::new(),
        );
    }

    let explicit_default_target = switch_inst.default_target;
    let mut default_target = explicit_default_target;
    let mut largest_case_target = None::<u32>;

    if default_target.is_none() {
        let mut best_target = None::<u32>;
        let mut best_len = 0usize;
        for target in &target_order {
            let len = target_to_labels.get(target).map_or(0, Vec::len);
            if best_target.is_none() || len > best_len {
                best_target = Some(*target);
                best_len = len;
            }
        }
        if let Some(target) = best_target {
            largest_case_target = Some(target);
            default_target = Some(target);
        }
    }

    let mut unique_targets = target_to_labels.keys().copied().collect::<Vec<_>>();
    if let Some(default_target) = default_target
        && !unique_targets.contains(&default_target)
    {
        unique_targets.push(default_target);
    }

    let incoming_counts = count_incoming_edges(parent_container);
    let switch_ref_counts = count_switch_refs(switch_inst);
    let common_exit = detect_common_exit(parent_container, &unique_targets);
    let can_drop_common_exit_jump = parent_container
        .blocks
        .iter()
        .any(|x| x.start_address == parent_block_start);

    let mut sections = Vec::<IrHighLevelSwitchSection>::new();
    let mut consumed_blocks = HashSet::<u32>::new();

    let mut non_default_targets = unique_targets
        .iter()
        .copied()
        .filter(|x| Some(*x) != default_target)
        .collect::<Vec<_>>();
    non_default_targets.sort_by_key(|target| {
        target_to_labels
            .get(target)
            .and_then(|x| x.iter().min().copied())
            .unwrap_or(u32::MAX)
    });

    for target in non_default_targets {
        let labels = target_to_labels.get(&target).cloned().unwrap_or_default();
        let body = build_section_body(
            target,
            switch_container,
            parent_container,
            &incoming_counts,
            &switch_ref_counts,
            common_exit,
            can_drop_common_exit_jump,
        );
        if body.start_address == target
            && parent_container
                .blocks
                .iter()
                .any(|x| x.start_address == target)
            && is_consumable_target(
                target,
                parent_container,
                &incoming_counts,
                &switch_ref_counts,
                switch_container.id,
                common_exit,
            )
        {
            consumed_blocks.insert(target);
        }

        sections.push(IrHighLevelSwitchSection {
            labels,
            body,
            is_default: false,
        });
    }

    if let Some(default_target) = default_target {
        let mut default_labels = target_to_labels
            .get(&default_target)
            .cloned()
            .unwrap_or_default();
        if largest_case_target == Some(default_target) && explicit_default_target.is_none() {
            default_labels.clear();
        }

        let default_body = build_section_body(
            default_target,
            switch_container,
            parent_container,
            &incoming_counts,
            &switch_ref_counts,
            common_exit,
            can_drop_common_exit_jump,
        );
        if default_body.start_address == default_target
            && parent_container
                .blocks
                .iter()
                .any(|x| x.start_address == default_target)
            && is_consumable_target(
                default_target,
                parent_container,
                &incoming_counts,
                &switch_ref_counts,
                switch_container.id,
                common_exit,
            )
        {
            consumed_blocks.insert(default_target);
        }

        sections.push(IrHighLevelSwitchSection {
            labels: default_labels,
            body: default_body,
            is_default: true,
        });
    }

    (
        IrStatement::HighLevelSwitch(IrHighLevelSwitch {
            index_expression: switch_inst.index_expression.clone(),
            sections,
        }),
        consumed_blocks,
    )
}

fn build_section_body(
    target: u32,
    switch_container: &IrBlockContainer,
    parent_container: &IrBlockContainer,
    incoming_counts: &HashMap<u32, usize>,
    switch_ref_counts: &HashMap<u32, usize>,
    common_exit: Option<u32>,
    can_drop_common_exit_jump: bool,
) -> IrBlock {
    if let Some(target_block) = switch_container
        .blocks
        .iter()
        .find(|x| x.start_address == target)
    {
        let mut copied_statements = target_block.statements.clone();
        if let Some(tail) = copied_statements.last() {
            let drop_tail = match tail {
                IrStatement::Leave(IrLeave {
                    target_container_id,
                }) => *target_container_id == switch_container.id,
                IrStatement::Jump(IrJump { target_address }) => {
                    common_exit.is_some_and(|x| x == *target_address)
                        && !parent_container
                            .blocks
                            .iter()
                            .any(|b| b.start_address == *target_address)
                }
                _ => false,
            };
            if drop_tail {
                copied_statements.pop();
            }
        }
        return IrBlock {
            statements: copied_statements,
            start_address: target,
            should_emit_label: false,
        };
    }

    let consumable = is_consumable_target(
        target,
        parent_container,
        incoming_counts,
        switch_ref_counts,
        switch_container.id,
        common_exit,
    );

    if !consumable {
        return IrBlock {
            statements: vec![IrStatement::Jump(IrJump {
                target_address: target,
            })],
            start_address: target,
            should_emit_label: false,
        };
    }

    let Some(target_block) = parent_container
        .blocks
        .iter()
        .find(|x| x.start_address == target)
    else {
        return IrBlock {
            statements: vec![IrStatement::Jump(IrJump {
                target_address: target,
            })],
            start_address: target,
            should_emit_label: false,
        };
    };

    let mut copied_statements = target_block.statements.clone();
    if let Some(last) = copied_statements.last() {
        let drop_last_jump = matches!(
            last,
            IrStatement::Jump(IrJump { target_address })
                if can_drop_common_exit_jump && common_exit.is_some_and(|x| x == *target_address)
        );
        let drop_last_leave = matches!(
            last,
            IrStatement::Leave(IrLeave { target_container_id }) if *target_container_id == switch_container.id
        );
        if drop_last_jump || drop_last_leave {
            copied_statements.pop();
        }
    }

    IrBlock {
        statements: copied_statements,
        start_address: target,
        should_emit_label: false,
    }
}

fn is_consumable_target(
    target: u32,
    parent_container: &IrBlockContainer,
    incoming_counts: &HashMap<u32, usize>,
    switch_ref_counts: &HashMap<u32, usize>,
    switch_container_id: u32,
    common_exit: Option<u32>,
) -> bool {
    let Some(target_block) = parent_container
        .blocks
        .iter()
        .find(|x| x.start_address == target)
    else {
        return false;
    };

    if incoming_counts.get(&target).copied().unwrap_or(0)
        != switch_ref_counts.get(&target).copied().unwrap_or(0)
    {
        return false;
    }

    let Some(last) = target_block.statements.last() else {
        return true;
    };

    match last {
        IrStatement::Jump(IrJump { target_address }) => {
            common_exit.is_some_and(|x| x == *target_address)
        }
        IrStatement::Leave(IrLeave {
            target_container_id,
        }) => *target_container_id == switch_container_id,
        IrStatement::Return(IrReturn) => true,
        _ => false,
    }
}

fn detect_common_exit(parent_container: &IrBlockContainer, targets: &[u32]) -> Option<u32> {
    let target_set = targets.iter().copied().collect::<HashSet<_>>();
    let mut counts = HashMap::<u32, usize>::new();

    for target in targets {
        let Some(target_block) = parent_container
            .blocks
            .iter()
            .find(|x| x.start_address == *target)
        else {
            continue;
        };
        let Some(last) = target_block.statements.last() else {
            continue;
        };
        if let IrStatement::Jump(IrJump { target_address }) = last {
            if target_set.contains(target_address) {
                continue;
            }
            *counts.entry(*target_address).or_insert(0) += 1;
        }
    }

    let (best_target, best_count) = counts.into_iter().max_by_key(|(_, count)| *count)?;
    if best_count < 2 {
        return None;
    }
    Some(best_target)
}

fn count_switch_refs(switch_inst: &IrSwitch) -> HashMap<u32, usize> {
    let mut out = HashMap::<u32, usize>::new();
    for target in switch_inst.cases.values() {
        *out.entry(*target).or_insert(0) += 1;
    }
    if let Some(default_target) = switch_inst.default_target {
        *out.entry(default_target).or_insert(0) += 1;
    }
    out
}

fn count_incoming_edges(container: &IrBlockContainer) -> HashMap<u32, usize> {
    let mut out = HashMap::<u32, usize>::new();
    for block in &container.blocks {
        for statement in &block.statements {
            count_targets_in_statement(statement, &mut out);
        }
    }
    out
}

fn count_targets_in_statement(statement: &IrStatement, counts: &mut HashMap<u32, usize>) {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => {
            *counts.entry(*target_address).or_insert(0) += 1;
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            count_targets_in_statement(true_statement, counts);
            if let Some(false_statement) = false_statement.as_ref() {
                count_targets_in_statement(false_statement, counts);
            }
        }
        IrStatement::Switch(IrSwitch {
            cases,
            default_target,
            ..
        }) => {
            for target in cases.values() {
                *counts.entry(*target).or_insert(0) += 1;
            }
            if let Some(default_target) = default_target {
                *counts.entry(*default_target).or_insert(0) += 1;
            }
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for nested in &section.body.statements {
                    count_targets_in_statement(nested, counts);
                }
            }
        }
        IrStatement::Block(block) => {
            for nested in &block.statements {
                count_targets_in_statement(nested, counts);
            }
        }
        IrStatement::BlockContainer(container) => {
            for block in &container.blocks {
                for nested in &block.statements {
                    count_targets_in_statement(nested, counts);
                }
            }
        }
        _ => {}
    }
}
