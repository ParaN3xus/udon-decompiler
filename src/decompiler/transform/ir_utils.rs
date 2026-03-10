use std::collections::{HashSet, VecDeque};

use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrFunction, IrHighLevelDoWhile, IrHighLevelSwitch, IrHighLevelWhile,
    IrIf, IrJump, IrStatement, IrSwitch,
};

pub fn get_block_terminator(block: &IrBlock) -> Option<&IrStatement> {
    let last = block.statements.last()?;
    if last.is_terminator() {
        Some(last)
    } else {
        None
    }
}

pub fn iter_block_targets(statement: &IrStatement) -> Vec<u32> {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => vec![*target_address],
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut out = iter_block_targets(true_statement);
            if let Some(false_statement) = false_statement.as_ref() {
                out.extend(iter_block_targets(false_statement));
            }
            out
        }
        IrStatement::Switch(IrSwitch {
            cases,
            default_target,
            ..
        }) => {
            let mut out = cases.values().copied().collect::<Vec<_>>();
            if let Some(default_target) = default_target {
                out.push(*default_target);
            }
            out
        }
        _ => Vec::new(),
    }
}

pub fn iter_block_containers(function: &IrFunction) -> Vec<u32> {
    let mut out = Vec::<u32>::new();
    let mut visited = HashSet::<u32>::new();
    let mut queue = VecDeque::<u32>::new();
    queue.push_back(function.body.id);

    while let Some(container_id) = queue.pop_front() {
        if !visited.insert(container_id) {
            continue;
        }
        out.push(container_id);

        let mut nested = Vec::<u32>::new();
        if function.body.id == container_id {
            collect_nested_containers_from_container(&function.body, &mut nested);
        } else {
            // caller traverses nested containers by repeatedly resolving ids;
            // this helper only knows root container directly.
            // nested ids for non-root are discovered when that container is visited.
            if let Some(container) = find_container(function, container_id) {
                collect_nested_containers_from_container(container, &mut nested);
            }
        }
        for id in nested {
            queue.push_back(id);
        }
    }

    out
}

pub fn find_container(function: &IrFunction, container_id: u32) -> Option<&IrBlockContainer> {
    find_container_in_container(&function.body, container_id)
}

pub fn find_container_mut(
    container: &mut IrBlockContainer,
    container_id: u32,
) -> Option<&mut IrBlockContainer> {
    if container.id == container_id {
        return Some(container);
    }

    for block in &mut container.blocks {
        for statement in &mut block.statements {
            if let Some(found) = find_container_in_statement_mut(statement, container_id) {
                return Some(found);
            }
        }
    }
    None
}

fn collect_nested_containers_from_container(container: &IrBlockContainer, out: &mut Vec<u32>) {
    for block in &container.blocks {
        for statement in &block.statements {
            collect_nested_containers_from_statement(statement, out);
        }
    }
}

fn collect_nested_containers_from_statement(statement: &IrStatement, out: &mut Vec<u32>) {
    match statement {
        IrStatement::BlockContainer(container) => {
            out.push(container.id);
            collect_nested_containers_from_container(container, out);
        }
        IrStatement::Block(block) => {
            for nested in &block.statements {
                collect_nested_containers_from_statement(nested, out);
            }
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            collect_nested_containers_from_statement(true_statement, out);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_nested_containers_from_statement(false_statement, out);
            }
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                for statement in &section.body.statements {
                    collect_nested_containers_from_statement(statement, out);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            out.push(body.id);
            collect_nested_containers_from_container(body, out);
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            out.push(body.id);
            collect_nested_containers_from_container(body, out);
        }
        _ => {}
    }
}

fn find_container_in_container(
    container: &IrBlockContainer,
    container_id: u32,
) -> Option<&IrBlockContainer> {
    if container.id == container_id {
        return Some(container);
    }

    for block in &container.blocks {
        for statement in &block.statements {
            if let Some(found) = find_container_in_statement(statement, container_id) {
                return Some(found);
            }
        }
    }
    None
}

fn find_container_in_statement(
    statement: &IrStatement,
    container_id: u32,
) -> Option<&IrBlockContainer> {
    match statement {
        IrStatement::BlockContainer(container) => {
            find_container_in_container(container, container_id)
        }
        IrStatement::Block(block) => {
            for nested in &block.statements {
                if let Some(found) = find_container_in_statement(nested, container_id) {
                    return Some(found);
                }
            }
            None
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            if let Some(found) = find_container_in_statement(true_statement, container_id) {
                return Some(found);
            }
            if let Some(false_statement) = false_statement.as_ref() {
                return find_container_in_statement(false_statement, container_id);
            }
            None
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            find_container_in_container(body, container_id)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            find_container_in_container(body, container_id)
        }
        _ => None,
    }
}

fn find_container_in_statement_mut(
    statement: &mut IrStatement,
    container_id: u32,
) -> Option<&mut IrBlockContainer> {
    match statement {
        IrStatement::BlockContainer(container) => find_container_mut(container, container_id),
        IrStatement::Block(block) => {
            for nested in &mut block.statements {
                if let Some(found) = find_container_in_statement_mut(nested, container_id) {
                    return Some(found);
                }
            }
            None
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            if let Some(found) = find_container_in_statement_mut(true_statement, container_id) {
                return Some(found);
            }
            if let Some(false_statement) = false_statement.as_mut()
                && let Some(found) = find_container_in_statement_mut(false_statement, container_id)
            {
                return Some(found);
            }
            None
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            find_container_mut(body, container_id)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            find_container_mut(body, container_id)
        }
        _ => None,
    }
}
