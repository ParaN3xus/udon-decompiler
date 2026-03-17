use std::collections::{HashMap, HashSet};

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrFunction, IrHighLevelDoWhile, IrHighLevelSwitch, IrHighLevelWhile,
    IrIf, IrJump, IrLeave, IrReturn, IrStatement, IrSwitch,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct ControlFlowSimplification;

impl ITransform for ControlFlowSimplification {
    fn run(
        &self,
        function: &mut IrFunction,
        _context: &mut TransformContext<'_, '_>,
    ) -> Result<()> {
        for _ in 0..32 {
            let mut changed = false;
            if simplify_branch_chains(function) {
                changed = true;
            }
            if cleanup_empty_blocks(function) {
                changed = true;
            }
            if !changed {
                break;
            }
        }
        Ok(())
    }
}

fn simplify_branch_chains(function: &mut IrFunction) -> bool {
    let mut address_to_block = HashMap::<u32, IrBlock>::new();
    collect_blocks_map(&function.body, &mut address_to_block);
    rewrite_container_statements(&mut function.body, &address_to_block)
}

fn rewrite_container_statements(
    container: &mut IrBlockContainer,
    address_to_block: &HashMap<u32, IrBlock>,
) -> bool {
    let mut changed = false;
    for block in &mut container.blocks {
        for statement in &mut block.statements {
            if rewrite_statement(statement, address_to_block) {
                changed = true;
            }
        }
    }
    changed
}

fn rewrite_statement(
    statement: &mut IrStatement,
    address_to_block: &HashMap<u32, IrBlock>,
) -> bool {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => {
            let original = *target_address;
            let (resolved, chain_changed) = resolve_branch_target(original, address_to_block);

            if let Some(target_block) = address_to_block.get(&resolved)
                && target_block.statements.len() == 1
                && let Some(IrStatement::Leave(IrLeave {
                    target_container_id,
                })) = target_block.statements.first()
            {
                *statement = IrStatement::Leave(IrLeave {
                    target_container_id: *target_container_id,
                });
                return true;
            }

            if chain_changed {
                *target_address = resolved;
                return true;
            }

            false
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut changed = rewrite_statement(true_statement, address_to_block);
            if let Some(false_statement) = false_statement.as_mut()
                && rewrite_statement(false_statement, address_to_block)
            {
                changed = true;
            }
            changed
        }
        IrStatement::Switch(IrSwitch {
            cases,
            default_target,
            ..
        }) => {
            let mut changed = false;
            for target in cases.values_mut() {
                let (resolved, target_changed) = resolve_branch_target(*target, address_to_block);
                if target_changed {
                    *target = resolved;
                    changed = true;
                }
            }
            if let Some(default_target) = default_target.as_mut() {
                let (resolved, default_changed) =
                    resolve_branch_target(*default_target, address_to_block);
                if default_changed {
                    *default_target = resolved;
                    changed = true;
                }
            }
            changed
        }
        IrStatement::Block(block) => {
            let mut changed = false;
            for nested in &mut block.statements {
                if rewrite_statement(nested, address_to_block) {
                    changed = true;
                }
            }
            changed
        }
        IrStatement::BlockContainer(container) => {
            rewrite_container_statements(container, address_to_block)
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            let mut changed = false;
            for section in sections {
                for nested in &mut section.body.statements {
                    if rewrite_statement(nested, address_to_block) {
                        changed = true;
                    }
                }
            }
            changed
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            rewrite_container_statements(body, address_to_block)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            rewrite_container_statements(body, address_to_block)
        }
        _ => false,
    }
}

fn resolve_branch_target(
    target_address: u32,
    address_to_block: &HashMap<u32, IrBlock>,
) -> (u32, bool) {
    let mut current = target_address;
    let mut changed = false;
    let mut visited = HashSet::<u32>::new();

    while visited.insert(current) {
        let Some(block) = address_to_block.get(&current) else {
            break;
        };
        if block.statements.len() != 1 {
            break;
        }
        let Some(IrStatement::Jump(IrJump { target_address })) = block.statements.first() else {
            break;
        };
        current = *target_address;
        changed = true;
    }

    (current, changed)
}

fn cleanup_empty_blocks(function: &mut IrFunction) -> bool {
    let mut changed = false;

    loop {
        let incoming = compute_global_incoming_counts(function);
        if !combine_once_in_container(&mut function.body, &incoming) {
            break;
        }
        changed = true;
    }

    let incoming = compute_global_incoming_counts(function);
    if remove_dead_blocks_in_container(&mut function.body, &incoming) {
        changed = true;
    }

    changed
}

fn combine_once_in_container(
    container: &mut IrBlockContainer,
    incoming: &HashMap<u32, usize>,
) -> bool {
    for idx in 0..container.blocks.len() {
        let Some(block) = container.blocks.get(idx) else {
            continue;
        };
        if block.statements.is_empty() {
            continue;
        }
        if block.statements.len() > 1
            && block
                .statements
                .get(block.statements.len() - 2)
                .is_some_and(statement_may_branch)
        {
            continue;
        }

        let Some(IrStatement::Jump(IrJump { target_address })) = block.statements.last() else {
            continue;
        };
        let Some(target_idx) = container
            .blocks
            .iter()
            .position(|x| x.start_address == *target_address)
        else {
            continue;
        };
        if target_idx == idx {
            continue;
        }
        if incoming.get(target_address).copied().unwrap_or(0) != 1 {
            continue;
        }

        let mut merged = container.blocks[idx].statements.clone();
        let _ = merged.pop();
        merged.extend(container.blocks[target_idx].statements.clone());
        container.blocks[idx].statements = merged;
        container.blocks[target_idx].statements.clear();
        return true;
    }

    for block in &mut container.blocks {
        for statement in &mut block.statements {
            if combine_once_in_statement(statement, incoming) {
                return true;
            }
        }
    }

    false
}

fn combine_once_in_statement(statement: &mut IrStatement, incoming: &HashMap<u32, usize>) -> bool {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            if combine_once_in_statement(true_statement, incoming) {
                return true;
            }
            if let Some(false_statement) = false_statement.as_mut()
                && combine_once_in_statement(false_statement, incoming)
            {
                return true;
            }
        }
        IrStatement::Block(block) => {
            for nested in &mut block.statements {
                if combine_once_in_statement(nested, incoming) {
                    return true;
                }
            }
        }
        IrStatement::BlockContainer(container) => {
            if combine_once_in_container(container, incoming) {
                return true;
            }
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for nested in &mut section.body.statements {
                    if combine_once_in_statement(nested, incoming) {
                        return true;
                    }
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            if combine_once_in_container(body, incoming) {
                return true;
            }
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            if combine_once_in_container(body, incoming) {
                return true;
            }
        }
        _ => {}
    }
    false
}

fn remove_dead_blocks_in_container(
    container: &mut IrBlockContainer,
    incoming: &HashMap<u32, usize>,
) -> bool {
    let mut changed = false;
    let mut kept = Vec::<IrBlock>::new();
    for (index, block) in container.blocks.iter().cloned().enumerate() {
        if index == 0 || incoming.get(&block.start_address).copied().unwrap_or(0) > 0 {
            kept.push(block);
        } else {
            changed = true;
        }
    }
    container.blocks = kept;

    for block in &mut container.blocks {
        for statement in &mut block.statements {
            if remove_dead_blocks_in_statement(statement, incoming) {
                changed = true;
            }
        }
    }

    changed
}

fn remove_dead_blocks_in_statement(
    statement: &mut IrStatement,
    incoming: &HashMap<u32, usize>,
) -> bool {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut changed = remove_dead_blocks_in_statement(true_statement, incoming);
            if let Some(false_statement) = false_statement.as_mut()
                && remove_dead_blocks_in_statement(false_statement, incoming)
            {
                changed = true;
            }
            changed
        }
        IrStatement::Block(block) => {
            let mut changed = false;
            for nested in &mut block.statements {
                if remove_dead_blocks_in_statement(nested, incoming) {
                    changed = true;
                }
            }
            changed
        }
        IrStatement::BlockContainer(container) => {
            remove_dead_blocks_in_container(container, incoming)
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            let mut changed = false;
            for section in sections {
                for nested in &mut section.body.statements {
                    if remove_dead_blocks_in_statement(nested, incoming) {
                        changed = true;
                    }
                }
            }
            changed
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            remove_dead_blocks_in_container(body, incoming)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            remove_dead_blocks_in_container(body, incoming)
        }
        _ => false,
    }
}

fn statement_may_branch(statement: &IrStatement) -> bool {
    matches!(
        statement,
        IrStatement::If(_)
            | IrStatement::Jump(_)
            | IrStatement::Leave(_)
            | IrStatement::Return(IrReturn)
            | IrStatement::Switch(_)
            | IrStatement::HighLevelSwitch(_)
            | IrStatement::BlockContainer(_)
    )
}

fn compute_global_incoming_counts(function: &IrFunction) -> HashMap<u32, usize> {
    let mut counts = HashMap::<u32, usize>::new();
    collect_block_addresses(&function.body, &mut counts);
    count_targets_in_container(&function.body, &mut counts);
    counts
}

fn collect_block_addresses(container: &IrBlockContainer, counts: &mut HashMap<u32, usize>) {
    for block in &container.blocks {
        counts.entry(block.start_address).or_insert(0);
        for statement in &block.statements {
            collect_block_addresses_from_statement(statement, counts);
        }
    }
}

fn collect_block_addresses_from_statement(
    statement: &IrStatement,
    counts: &mut HashMap<u32, usize>,
) {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            collect_block_addresses_from_statement(true_statement, counts);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_block_addresses_from_statement(false_statement, counts);
            }
        }
        IrStatement::Block(block) => {
            for nested in &block.statements {
                collect_block_addresses_from_statement(nested, counts);
            }
        }
        IrStatement::BlockContainer(container) => collect_block_addresses(container, counts),
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for nested in &section.body.statements {
                    collect_block_addresses_from_statement(nested, counts);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            collect_block_addresses(body, counts);
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            collect_block_addresses(body, counts);
        }
        _ => {}
    }
}

fn count_targets_in_container(container: &IrBlockContainer, counts: &mut HashMap<u32, usize>) {
    for block in &container.blocks {
        for statement in &block.statements {
            count_targets_in_statement(statement, counts);
        }
    }
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
        IrStatement::Block(block) => {
            for nested in &block.statements {
                count_targets_in_statement(nested, counts);
            }
        }
        IrStatement::BlockContainer(container) => {
            count_targets_in_container(container, counts);
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for nested in &section.body.statements {
                    count_targets_in_statement(nested, counts);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            count_targets_in_container(body, counts);
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            count_targets_in_container(body, counts);
        }
        _ => {}
    }
}

fn collect_blocks_map(container: &IrBlockContainer, out: &mut HashMap<u32, IrBlock>) {
    for block in &container.blocks {
        out.insert(block.start_address, block.clone());
        for statement in &block.statements {
            collect_blocks_map_from_statement(statement, out);
        }
    }
}

fn collect_blocks_map_from_statement(statement: &IrStatement, out: &mut HashMap<u32, IrBlock>) {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            collect_blocks_map_from_statement(true_statement, out);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_blocks_map_from_statement(false_statement, out);
            }
        }
        IrStatement::Block(block) => {
            for nested in &block.statements {
                collect_blocks_map_from_statement(nested, out);
            }
        }
        IrStatement::BlockContainer(container) => collect_blocks_map(container, out),
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for nested in &section.body.statements {
                    collect_blocks_map_from_statement(nested, out);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            collect_blocks_map(body, out);
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            collect_blocks_map(body, out);
        }
        _ => {}
    }
}
