use std::collections::{HashMap, HashSet};

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrFunction, IrHighLevelDoWhile, IrHighLevelSwitch, IrHighLevelWhile,
    IrIf, IrJump, IrReturn, IrStatement, IrSwitch,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct StructuredControlFlowCleanupTransform;

impl ITransform for StructuredControlFlowCleanupTransform {

    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let mut state = CleanupState::from_context(context);
        let known_blocks = collect_all_block_addresses(&function.body);
        rewrite_container(&mut function.body, &known_blocks, &mut state);
        state.commit(context);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CleanupState {
    synthetic_block_address: i64,
}

impl CleanupState {
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
    container: &mut IrBlockContainer,
    known_blocks: &HashSet<u32>,
    state: &mut CleanupState,
) -> bool {
    let mut changed = false;

    for block in &mut container.blocks {
        changed |= rewrite_statement_list(&mut block.statements, known_blocks, state);
        changed |= simplify_if_else_join_jumps(block);
    }

    changed |= hoist_shared_switch_default_tails(container);
    changed
}

fn rewrite_statement_list(
    statements: &mut Vec<IrStatement>,
    known_blocks: &HashSet<u32>,
    state: &mut CleanupState,
) -> bool {
    let mut changed = false;
    for statement in statements {
        changed |= rewrite_statement(statement, known_blocks, state);
    }
    changed
}

fn rewrite_statement(
    statement: &mut IrStatement,
    known_blocks: &HashSet<u32>,
    state: &mut CleanupState,
) -> bool {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut changed = false;
            changed |= rewrite_statement(true_statement, known_blocks, state);
            if let Some(false_statement) = false_statement.as_mut() {
                changed |= rewrite_statement(false_statement, known_blocks, state);
            }
            if false_statement
                .as_ref()
                .is_some_and(|x| is_truly_empty_branch(x.as_ref()))
            {
                *false_statement = None;
                changed = true;
            }
            changed
        }
        IrStatement::Block(block) => {
            let mut changed = false;
            changed |= rewrite_statement_list(&mut block.statements, known_blocks, state);
            changed |= simplify_if_else_join_jumps(block);
            changed |= simplify_if_missing_jump_to_else(block, known_blocks, state);
            changed
        }
        IrStatement::BlockContainer(container) => rewrite_container(container, known_blocks, state),
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            let mut changed = rewrite_container(body, known_blocks, state);
            changed |= eliminate_nested_gotos_to_next_block(body, state);
            changed |= simplify_linear_if_goto_diamonds(body);
            changed |= simplify_two_way_if_goto_diamonds(body);
            changed |= simplify_true_terminal_if_gotos(body);
            changed |= simplify_terminal_two_way_if_gotos(body);
            changed |= simplify_guarded_linear_regions(body);
            changed |= simplify_late_true_target_if_gotos(body);
            changed |= simplify_nested_if_goto_fallbacks(body);
            changed
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            let mut changed = rewrite_container(body, known_blocks, state);
            changed |= eliminate_nested_gotos_to_next_block(body, state);
            changed |= simplify_linear_if_goto_diamonds(body);
            changed |= simplify_two_way_if_goto_diamonds(body);
            changed |= simplify_true_terminal_if_gotos(body);
            changed |= simplify_terminal_two_way_if_gotos(body);
            changed |= simplify_guarded_linear_regions(body);
            changed |= simplify_late_true_target_if_gotos(body);
            changed |= simplify_nested_if_goto_fallbacks(body);
            changed
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            let mut changed = false;
            for section in sections {
                changed |=
                    rewrite_statement_list(&mut section.body.statements, known_blocks, state);
            }
            changed
        }
        _ => false,
    }
}

fn simplify_if_else_join_jumps(block: &mut IrBlock) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let mut index = 0usize;
        while index < block.statements.len() {
            let statement_snapshot = block.statements[index].clone();
            let IrStatement::If(mut if_stmt) = statement_snapshot else {
                index += 1;
                continue;
            };

            let (true_block, false_block) = match (
                if_stmt.true_statement.as_ref(),
                if_stmt.false_statement.as_ref().map(|x| x.as_ref()),
            ) {
                (IrStatement::Block(true_block), Some(IrStatement::Block(false_block))) => {
                    (true_block, false_block)
                }
                _ => {
                    index += 1;
                    continue;
                }
            };

            let false_start = false_block.start_address;
            let false_statements = false_block.statements.clone();
            let ref_count = count_jump_target_references_in_statement(
                &IrStatement::Block(true_block.clone()),
                false_start,
            );
            if ref_count != 1 {
                index += 1;
                continue;
            }

            let total_ref_count = count_jump_target_references_in_block(block, false_start);
            if total_ref_count != 1 {
                index += 1;
                continue;
            }

            let mut true_statement = IrStatement::Block(true_block.clone());
            if !remove_first_jump_target_reference(&mut true_statement, false_start) {
                index += 1;
                continue;
            }
            let IrStatement::Block(new_true_block) = true_statement else {
                index += 1;
                continue;
            };

            if_stmt.true_statement = Box::new(IrStatement::Block(new_true_block));
            if_stmt.false_statement = None;
            block.statements[index] = IrStatement::If(if_stmt);
            block
                .statements
                .splice(index + 1..index + 1, false_statements);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_if_missing_jump_to_else(
    block: &mut IrBlock,
    known_blocks: &HashSet<u32>,
    state: &mut CleanupState,
) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;

        for index in 0..block.statements.len() {
            let statement_snapshot = block.statements[index].clone();
            let IrStatement::If(mut if_stmt) = statement_snapshot else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            let IrStatement::Block(true_block) = if_stmt.true_statement.as_ref() else {
                continue;
            };
            if true_block.statements.is_empty() {
                continue;
            }

            let Some(IrStatement::Jump(IrJump { target_address })) = true_block.statements.last()
            else {
                continue;
            };
            if known_blocks.contains(target_address) {
                continue;
            }

            let suffix = block.statements[index + 1..].to_vec();
            if suffix.is_empty() || !statements_end_unreachable(&suffix) {
                continue;
            }

            let mut true_without_tail = true_block.statements.clone();
            true_without_tail.pop();
            if_stmt.true_statement = Box::new(IrStatement::Block(IrBlock {
                statements: true_without_tail,
                start_address: true_block.start_address,
                should_emit_label: true_block.should_emit_label,
            }));
            if_stmt.false_statement = Some(Box::new(IrStatement::Block(IrBlock {
                statements: suffix.clone(),
                start_address: new_block_start_address(suffix.first(), state),
                should_emit_label: false,
            })));

            block.statements[index] = IrStatement::If(if_stmt);
            block.statements.truncate(index + 1);
            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }

    changed_any
}

fn eliminate_nested_gotos_to_next_block(
    body: &mut IrBlockContainer,
    state: &mut CleanupState,
) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        for index in 0..body.blocks.len() {
            if index + 1 >= body.blocks.len() {
                continue;
            }
            let next_target = body.blocks[index + 1].start_address;
            if rewrite_nested_goto_in_statement_list(
                &mut body.blocks[index].statements,
                next_target,
                state,
            ) {
                changed = true;
                changed_any = true;
                break;
            }
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn rewrite_nested_goto_in_statement_list(
    statements: &mut Vec<IrStatement>,
    next_target: u32,
    state: &mut CleanupState,
) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;

        for index in 0..statements.len() {
            if rewrite_nested_goto_in_statement(&mut statements[index], next_target, state) {
                changed = true;
                changed_any = true;
                break;
            }

            let statement_snapshot = statements[index].clone();
            let IrStatement::If(mut if_stmt) = statement_snapshot else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }
            let IrStatement::Block(true_block) = if_stmt.true_statement.as_mut() else {
                continue;
            };
            if true_block.statements.is_empty() {
                continue;
            }
            let Some(IrStatement::Jump(IrJump { target_address })) = true_block.statements.last()
            else {
                continue;
            };
            if *target_address != next_target {
                continue;
            }

            let suffix = statements[index + 1..].to_vec();
            if suffix.is_empty()
                || !statements_end_unreachable(&suffix)
                || has_nested_control_flow_statement(&suffix)
            {
                continue;
            }

            true_block.statements.pop();
            if_stmt.false_statement = Some(Box::new(IrStatement::Block(IrBlock {
                statements: suffix,
                start_address: new_block_start_address(statements.get(index + 1), state),
                should_emit_label: false,
            })));
            statements[index] = IrStatement::If(if_stmt);
            statements.truncate(index + 1);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }

    changed_any
}

fn rewrite_nested_goto_in_statement(
    statement: &mut IrStatement,
    next_target: u32,
    state: &mut CleanupState,
) -> bool {
    match statement {
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut changed = rewrite_nested_goto_in_statement(true_statement, next_target, state);
            if let Some(false_statement) = false_statement.as_mut() {
                changed |= rewrite_nested_goto_in_statement(false_statement, next_target, state);
            }
            changed
        }
        IrStatement::Block(block) => {
            rewrite_nested_goto_in_statement_list(&mut block.statements, next_target, state)
        }
        _ => false,
    }
}

fn simplify_linear_if_goto_diamonds(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;

        for block_index in 0..body.blocks.len() {
            if block_index >= body.blocks.len() {
                break;
            }
            let block = body.blocks[block_index].clone();
            if block.statements.len() < 2 {
                continue;
            }

            let if_stmt = block.statements[block.statements.len() - 2].clone();
            let after_if = block.statements[block.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: false_target,
                }),
            ) = (if_stmt, after_if)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            let Some(true_target) = as_jump_target(if_stmt.true_statement.as_ref()) else {
                continue;
            };

            if block_index + 2 >= body.blocks.len() {
                continue;
            }
            if body.blocks[block_index + 1].start_address != true_target {
                continue;
            }
            if body.blocks[block_index + 2].start_address != false_target {
                continue;
            }

            let true_block = body.blocks[block_index + 1].clone();
            let mut true_statements = true_block.statements.clone();
            if let Some(last_true) = true_statements.last() {
                match last_true {
                    IrStatement::Jump(IrJump { target_address }) => {
                        if *target_address != false_target {
                            continue;
                        }
                        true_statements.pop();
                    }
                    IrStatement::If(_) | IrStatement::Leave(_) => continue,
                    _ => {}
                }
            }

            if_stmt.true_statement = Box::new(IrStatement::Block(IrBlock {
                statements: true_statements,
                start_address: true_block.start_address,
                should_emit_label: true_block.should_emit_label,
            }));

            if let Some(target_block) = body.blocks.get_mut(block_index)
                && target_block.statements.len() >= 2
            {
                let len = target_block.statements.len();
                target_block.statements[len - 2] = IrStatement::If(if_stmt);
                target_block.statements.pop();
            }

            body.blocks.remove(block_index + 1);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_two_way_if_goto_diamonds(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let incoming = compute_incoming_counts(body);

        for block_index in 0..body.blocks.len() {
            let block = body.blocks[block_index].clone();
            if block.statements.len() < 2 {
                continue;
            }

            let if_stmt = block.statements[block.statements.len() - 2].clone();
            let fallback_jump = block.statements[block.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: false_target,
                }),
            ) = (if_stmt, fallback_jump)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            let Some(true_target) = as_jump_target(if_stmt.true_statement.as_ref()) else {
                continue;
            };
            if true_target == false_target {
                continue;
            }

            let Some(true_idx) = find_block_index_by_start(body, true_target) else {
                continue;
            };
            let Some(false_idx) = find_block_index_by_start(body, false_target) else {
                continue;
            };
            if incoming.get(&true_target).copied().unwrap_or(0) != 1
                || incoming.get(&false_target).copied().unwrap_or(0) != 1
            {
                continue;
            }

            let true_block = body.blocks[true_idx].clone();
            let false_block = body.blocks[false_idx].clone();
            let Some(IrStatement::Jump(IrJump {
                target_address: true_join,
            })) = true_block.statements.last()
            else {
                continue;
            };
            let Some(IrStatement::Jump(IrJump {
                target_address: false_join,
            })) = false_block.statements.last()
            else {
                continue;
            };
            if true_join != false_join {
                continue;
            }
            if find_block_index_by_start(body, *true_join).is_none() {
                continue;
            }

            if_stmt.true_statement = Box::new(IrStatement::Block(IrBlock {
                statements: true_block
                    .statements
                    .iter()
                    .take(true_block.statements.len().saturating_sub(1))
                    .cloned()
                    .collect(),
                start_address: true_block.start_address,
                should_emit_label: true_block.should_emit_label,
            }));
            if_stmt.false_statement = Some(Box::new(IrStatement::Block(IrBlock {
                statements: false_block
                    .statements
                    .iter()
                    .take(false_block.statements.len().saturating_sub(1))
                    .cloned()
                    .collect(),
                start_address: false_block.start_address,
                should_emit_label: false_block.should_emit_label,
            })));

            if let Some(target_block) = body.blocks.get_mut(block_index)
                && target_block.statements.len() >= 2
            {
                let len = target_block.statements.len();
                target_block.statements[len - 2] = IrStatement::If(if_stmt);
                target_block.statements.pop();
            }

            remove_block_by_start(body, true_target);
            remove_block_by_start(body, false_target);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_true_terminal_if_gotos(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let incoming = compute_incoming_counts(body);

        for block_index in 0..body.blocks.len() {
            let block = body.blocks[block_index].clone();
            if block.statements.len() < 2 {
                continue;
            }

            let if_stmt = block.statements[block.statements.len() - 2].clone();
            let fallback_jump = block.statements[block.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: false_target,
                }),
            ) = (if_stmt, fallback_jump)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            let Some(true_target) = as_jump_target(if_stmt.true_statement.as_ref()) else {
                continue;
            };
            let Some(true_idx) = find_block_index_by_start(body, true_target) else {
                continue;
            };
            let Some(false_idx) = find_block_index_by_start(body, false_target) else {
                continue;
            };
            if incoming.get(&true_target).copied().unwrap_or(0) != 1 {
                continue;
            }
            if block_index + 2 >= body.blocks.len() {
                continue;
            }
            if body.blocks[block_index + 1].start_address != true_target
                || body.blocks[block_index + 2].start_address != false_target
            {
                continue;
            }

            let true_block = body.blocks[true_idx].clone();
            if !statements_end_unreachable(&true_block.statements) {
                continue;
            }

            if_stmt.true_statement = Box::new(IrStatement::Block(true_block.clone()));
            if let Some(target_block) = body.blocks.get_mut(block_index)
                && target_block.statements.len() >= 2
            {
                let len = target_block.statements.len();
                target_block.statements[len - 2] = IrStatement::If(if_stmt);
                target_block.statements.pop();
            }
            let _ = false_idx;
            remove_block_by_start(body, true_target);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_terminal_two_way_if_gotos(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let incoming = compute_incoming_counts(body);

        for block_index in 0..body.blocks.len() {
            let block = body.blocks[block_index].clone();
            if block.statements.len() < 2 {
                continue;
            }

            let if_stmt = block.statements[block.statements.len() - 2].clone();
            let fallback_jump = block.statements[block.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: false_target,
                }),
            ) = (if_stmt, fallback_jump)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            let Some(true_target) = as_jump_target(if_stmt.true_statement.as_ref()) else {
                continue;
            };
            if true_target == false_target {
                continue;
            }
            let Some(true_idx) = find_block_index_by_start(body, true_target) else {
                continue;
            };
            let Some(false_idx) = find_block_index_by_start(body, false_target) else {
                continue;
            };
            if incoming.get(&true_target).copied().unwrap_or(0) != 1
                || incoming.get(&false_target).copied().unwrap_or(0) != 1
            {
                continue;
            }

            let true_block = body.blocks[true_idx].clone();
            let false_block = body.blocks[false_idx].clone();
            if !statements_end_unreachable(&true_block.statements)
                || !statements_end_unreachable(&false_block.statements)
            {
                continue;
            }

            if_stmt.true_statement = Box::new(IrStatement::Block(true_block.clone()));
            if_stmt.false_statement = Some(Box::new(IrStatement::Block(false_block.clone())));

            if let Some(target_block) = body.blocks.get_mut(block_index)
                && target_block.statements.len() >= 2
            {
                let len = target_block.statements.len();
                target_block.statements[len - 2] = IrStatement::If(if_stmt);
                target_block.statements.pop();
            }

            remove_block_by_start(body, true_target);
            remove_block_by_start(body, false_target);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_guarded_linear_regions(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let incoming = compute_incoming_counts(body);

        for block_index in 0..body.blocks.len() {
            let block = body.blocks[block_index].clone();
            if block.statements.len() < 2 {
                continue;
            }

            let if_stmt = block.statements[block.statements.len() - 2].clone();
            let fallback_jump = block.statements[block.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: region_end,
                }),
            ) = (if_stmt, fallback_jump)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            let Some(region_start) = as_jump_target(if_stmt.true_statement.as_ref()) else {
                continue;
            };
            let Some(start_index) = find_block_index_by_start(body, region_start) else {
                continue;
            };
            let Some(end_index) = find_block_index_by_start(body, region_end) else {
                continue;
            };
            if start_index != block_index + 1 || end_index <= start_index {
                continue;
            }

            let region_blocks = body.blocks[start_index..end_index].to_vec();
            if region_blocks.is_empty() {
                continue;
            }
            if incoming
                .get(&region_blocks[0].start_address)
                .copied()
                .unwrap_or(0)
                != 1
            {
                continue;
            }
            if !region_has_no_extra_explicit_entries(&region_blocks, &incoming)
                || !region_is_mergeable(&region_blocks, region_end)
            {
                continue;
            }

            let mut merged = Vec::<IrStatement>::new();
            for region_block in &region_blocks {
                merged.extend(region_block.statements.clone());
            }
            if let Some(IrStatement::Jump(IrJump { target_address })) = merged.last()
                && *target_address == region_end
            {
                merged.pop();
            }

            if_stmt.true_statement = Box::new(IrStatement::Block(IrBlock {
                statements: merged,
                start_address: region_blocks[0].start_address,
                should_emit_label: region_blocks[0].should_emit_label,
            }));

            if let Some(target_block) = body.blocks.get_mut(block_index)
                && target_block.statements.len() >= 2
            {
                let len = target_block.statements.len();
                target_block.statements[len - 2] = IrStatement::If(if_stmt);
                target_block.statements.pop();
            }

            let region_addresses = region_blocks
                .iter()
                .map(|x| x.start_address)
                .collect::<Vec<_>>();
            body.blocks
                .retain(|x| !region_addresses.contains(&x.start_address));

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_late_true_target_if_gotos(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let incoming = compute_incoming_counts(body);

        for block_index in 0..body.blocks.len() {
            if block_index + 1 >= body.blocks.len() {
                continue;
            }
            let block = body.blocks[block_index].clone();
            if block.statements.len() < 2 {
                continue;
            }

            let if_stmt = block.statements[block.statements.len() - 2].clone();
            let fallback_jump = block.statements[block.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: false_target,
                }),
            ) = (if_stmt, fallback_jump)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }

            if body.blocks[block_index + 1].start_address != false_target {
                continue;
            }

            let Some(true_target) = as_jump_target(if_stmt.true_statement.as_ref()) else {
                continue;
            };
            let Some(true_index) = find_block_index_by_start(body, true_target) else {
                continue;
            };
            if true_target == false_target
                || incoming.get(&true_target).copied().unwrap_or(0) != 1
                || true_index <= block_index + 1
            {
                continue;
            }

            let true_block = body.blocks[true_index].clone();
            let mut inlined = true_block.statements.clone();
            if !statements_end_unreachable(&inlined) {
                if true_index + 1 >= body.blocks.len() {
                    continue;
                }
                let successor = body.blocks[true_index + 1].start_address;
                inlined.push(IrStatement::Jump(IrJump {
                    target_address: successor,
                }));
            }

            if_stmt.true_statement = Box::new(IrStatement::Block(IrBlock {
                statements: inlined,
                start_address: true_block.start_address,
                should_emit_label: true_block.should_emit_label,
            }));

            if let Some(target_block) = body.blocks.get_mut(block_index)
                && target_block.statements.len() >= 2
            {
                let len = target_block.statements.len();
                target_block.statements[len - 2] = IrStatement::If(if_stmt);
                target_block.statements.pop();
            }

            remove_block_by_start(body, true_target);

            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }
    changed_any
}

fn simplify_nested_if_goto_fallbacks(body: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;
        let incoming = compute_incoming_counts(body);

        let nested_blocks = collect_nested_ir_blocks(body);
        for nested_start in nested_blocks {
            let Some(nested_snapshot) = find_nested_block(body, nested_start).cloned() else {
                continue;
            };
            if nested_snapshot.statements.len() < 2 {
                continue;
            }

            let if_stmt = nested_snapshot.statements[nested_snapshot.statements.len() - 2].clone();
            let fallback_jump =
                nested_snapshot.statements[nested_snapshot.statements.len() - 1].clone();
            let (
                IrStatement::If(mut if_stmt),
                IrStatement::Jump(IrJump {
                    target_address: fallback_target,
                }),
            ) = (if_stmt, fallback_jump)
            else {
                continue;
            };
            if if_stmt.false_statement.is_some() {
                continue;
            }
            // Python IR jump targets use block identity, while Rust currently
            // matches by start address only. Keep synthetic loop sentinels out
            // of this rewrite to avoid collapsing loop bodies.
            if fallback_target == u32::MAX {
                continue;
            }
            let Some(fallback_index) = find_block_index_by_start(body, fallback_target) else {
                continue;
            };
            if incoming.get(&fallback_target).copied().unwrap_or(0) != 1 {
                continue;
            }

            let fallback_block = body.blocks[fallback_index].clone();
            let mut inlined = fallback_block.statements.clone();
            if !statements_end_unreachable(&inlined) {
                if fallback_index + 1 >= body.blocks.len() {
                    continue;
                }
                inlined.push(IrStatement::Jump(IrJump {
                    target_address: body.blocks[fallback_index + 1].start_address,
                }));
            }

            if_stmt.false_statement = Some(Box::new(IrStatement::Block(IrBlock {
                statements: inlined,
                start_address: fallback_block.start_address,
                should_emit_label: fallback_block.should_emit_label,
            })));
            let Some(nested_mut) = find_nested_block_mut(body, nested_start) else {
                continue;
            };
            let len = nested_mut.statements.len();
            nested_mut.statements[len - 2] = IrStatement::If(if_stmt);
            nested_mut.statements.pop();

            remove_block_by_start(body, fallback_target);
            changed = true;
            changed_any = true;
            break;
        }

        if !changed {
            break;
        }
    }

    changed_any
}

fn hoist_shared_switch_default_tails(container: &mut IrBlockContainer) -> bool {
    let mut changed_any = false;
    loop {
        let mut changed = false;

        for block_index in 0..container.blocks.len() {
            let block_snapshot = container.blocks[block_index].clone();

            for stmt_index in 0..block_snapshot.statements.len() {
                let IrStatement::If(if_stmt) = &block_snapshot.statements[stmt_index] else {
                    continue;
                };
                if if_stmt.false_statement.is_some() {
                    continue;
                }

                let Some(mut switch_stmt) =
                    extract_single_switch(if_stmt.true_statement.as_ref()).cloned()
                else {
                    continue;
                };
                let Some(default_section_index) =
                    switch_stmt.sections.iter().position(|x| x.is_default)
                else {
                    continue;
                };

                let tail = block_snapshot.statements[stmt_index + 1..].to_vec();
                if tail.is_empty() {
                    continue;
                }

                let default_body = switch_stmt.sections[default_section_index].body.clone();
                if default_body.statements != tail {
                    continue;
                }

                let existing = find_block_by_start_address(container, default_body.start_address);
                if existing.is_some_and(|existing_index| existing_index != block_index) {
                    continue;
                }

                let shared_block = IrBlock {
                    statements: tail,
                    start_address: default_body.start_address,
                    should_emit_label: false,
                };

                let mut new_if = if_stmt.clone();
                switch_stmt.sections[default_section_index].body = IrBlock {
                    statements: vec![IrStatement::Jump(IrJump {
                        target_address: shared_block.start_address,
                    })],
                    start_address: default_body.start_address,
                    should_emit_label: default_body.should_emit_label,
                };
                new_if.true_statement = Box::new(IrStatement::Block(IrBlock {
                    statements: vec![IrStatement::HighLevelSwitch(switch_stmt)],
                    start_address: statement_start_address(if_stmt.true_statement.as_ref())
                        .unwrap_or(default_body.start_address),
                    should_emit_label: false,
                }));

                if let Some(block) = container.blocks.get_mut(block_index) {
                    block.statements.truncate(stmt_index + 1);
                    block.statements[stmt_index] = IrStatement::If(new_if);
                }

                container.blocks.insert(block_index + 1, shared_block);

                changed = true;
                changed_any = true;
                break;
            }

            if changed {
                break;
            }
        }

        if !changed {
            break;
        }
    }

    changed_any
}

fn is_truly_empty_branch(statement: &IrStatement) -> bool {
    matches!(statement, IrStatement::Block(block) if block.statements.is_empty())
}

fn as_jump_target(statement: &IrStatement) -> Option<u32> {
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

fn extract_single_switch(statement: &IrStatement) -> Option<&IrHighLevelSwitch> {
    let IrStatement::Block(block) = statement else {
        return None;
    };
    if block.statements.len() != 1 {
        return None;
    }
    match &block.statements[0] {
        IrStatement::HighLevelSwitch(switch_stmt) => Some(switch_stmt),
        _ => None,
    }
}

fn has_nested_control_flow_statement(statements: &[IrStatement]) -> bool {
    statements.iter().any(|statement| match statement {
        IrStatement::If(_)
        | IrStatement::BlockContainer(_)
        | IrStatement::HighLevelSwitch(_)
        | IrStatement::HighLevelWhile(_)
        | IrStatement::HighLevelDoWhile(_)
        | IrStatement::Switch(_) => true,
        IrStatement::Block(block) => has_nested_control_flow_statement(&block.statements),
        _ => false,
    })
}

fn statements_end_unreachable(statements: &[IrStatement]) -> bool {
    let Some(last) = statements.last() else {
        return false;
    };
    statement_is_unreachable_endpoint(last)
}

fn statement_is_unreachable_endpoint(statement: &IrStatement) -> bool {
    match statement {
        IrStatement::Jump(_) | IrStatement::Leave(_) | IrStatement::Return(IrReturn) => true,
        IrStatement::Switch(IrSwitch { .. }) => true,
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => false_statement.as_ref().is_some_and(|false_statement| {
            statement_is_unreachable_endpoint(true_statement)
                && statement_is_unreachable_endpoint(false_statement)
        }),
        IrStatement::Block(block) => block
            .statements
            .last()
            .is_some_and(statement_is_unreachable_endpoint),
        _ => false,
    }
}

fn count_jump_target_references_in_block(block: &IrBlock, target: u32) -> usize {
    block
        .statements
        .iter()
        .map(|statement| count_jump_target_references_in_statement(statement, target))
        .sum()
}

fn count_jump_target_references_in_statement(statement: &IrStatement, target: u32) -> usize {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => usize::from(*target_address == target),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut count = count_jump_target_references_in_statement(true_statement, target);
            if let Some(false_statement) = false_statement.as_ref() {
                count += count_jump_target_references_in_statement(false_statement, target);
            }
            count
        }
        IrStatement::Block(block) => block
            .statements
            .iter()
            .map(|nested| count_jump_target_references_in_statement(nested, target))
            .sum(),
        IrStatement::BlockContainer(container) => container
            .blocks
            .iter()
            .flat_map(|nested_block| nested_block.statements.iter())
            .map(|nested| count_jump_target_references_in_statement(nested, target))
            .sum(),
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => sections
            .iter()
            .map(|section| {
                count_jump_target_references_in_statement(
                    &IrStatement::Block(section.body.clone()),
                    target,
                )
            })
            .sum(),
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. })
        | IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => body
            .blocks
            .iter()
            .flat_map(|nested_block| nested_block.statements.iter())
            .map(|nested| count_jump_target_references_in_statement(nested, target))
            .sum(),
        _ => 0,
    }
}

fn remove_first_jump_target_reference(statement: &mut IrStatement, target: u32) -> bool {
    match statement {
        IrStatement::Block(block) => {
            let mut index = 0usize;
            while index < block.statements.len() {
                if let IrStatement::Jump(IrJump { target_address }) = &block.statements[index]
                    && *target_address == target
                {
                    block.statements.remove(index);
                    return true;
                }
                if remove_first_jump_target_reference(&mut block.statements[index], target) {
                    return true;
                }
                index += 1;
            }
            false
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            if remove_first_jump_target_reference(true_statement, target) {
                return true;
            }
            false_statement.as_mut().is_some_and(|false_statement| {
                remove_first_jump_target_reference(false_statement, target)
            })
        }
        IrStatement::BlockContainer(container) => {
            for nested_block in &mut container.blocks {
                for nested in &mut nested_block.statements {
                    if remove_first_jump_target_reference(nested, target) {
                        return true;
                    }
                }
            }
            false
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                let mut section_stmt = IrStatement::Block(section.body.clone());
                if remove_first_jump_target_reference(&mut section_stmt, target) {
                    if let IrStatement::Block(updated) = section_stmt {
                        section.body = updated;
                    }
                    return true;
                }
            }
            false
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. })
        | IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            for nested_block in &mut body.blocks {
                for nested in &mut nested_block.statements {
                    if remove_first_jump_target_reference(nested, target) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn collect_all_block_addresses(container: &IrBlockContainer) -> HashSet<u32> {
    let mut blocks = HashSet::<u32>::new();

    fn visit_statement(statement: &IrStatement, blocks: &mut HashSet<u32>) {
        match statement {
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                visit_statement(true_statement, blocks);
                if let Some(false_statement) = false_statement.as_ref() {
                    visit_statement(false_statement, blocks);
                }
            }
            IrStatement::Block(block) => {
                blocks.insert(block.start_address);
                for nested in &block.statements {
                    visit_statement(nested, blocks);
                }
            }
            IrStatement::BlockContainer(container) => visit_container(container, blocks),
            IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. })
            | IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
                visit_container(body, blocks)
            }
            IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
                for section in sections {
                    visit_statement(&IrStatement::Block(section.body.clone()), blocks);
                }
            }
            _ => {}
        }
    }

    fn visit_container(container: &IrBlockContainer, blocks: &mut HashSet<u32>) {
        for block in &container.blocks {
            blocks.insert(block.start_address);
            for statement in &block.statements {
                visit_statement(statement, blocks);
            }
        }
    }

    visit_container(container, &mut blocks);
    blocks
}

fn collect_nested_ir_blocks(body: &IrBlockContainer) -> Vec<u32> {
    let mut nested = Vec::<u32>::new();

    fn visit_statement(statement: &IrStatement, nested: &mut Vec<u32>) {
        match statement {
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                visit_statement(true_statement, nested);
                if let Some(false_statement) = false_statement.as_ref() {
                    visit_statement(false_statement, nested);
                }
            }
            IrStatement::Block(block) => {
                nested.push(block.start_address);
                for child in &block.statements {
                    visit_statement(child, nested);
                }
            }
            IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. })
            | IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
                for child_block in &body.blocks {
                    for child in &child_block.statements {
                        visit_statement(child, nested);
                    }
                }
            }
            IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
                for section in sections {
                    visit_statement(&IrStatement::Block(section.body.clone()), nested);
                }
            }
            IrStatement::BlockContainer(container) => {
                for child_block in &container.blocks {
                    for child in &child_block.statements {
                        visit_statement(child, nested);
                    }
                }
            }
            _ => {}
        }
    }

    for block in &body.blocks {
        for statement in &block.statements {
            visit_statement(statement, &mut nested);
        }
    }

    nested
}

fn compute_incoming_counts(body: &IrBlockContainer) -> HashMap<u32, usize> {
    let mut counts = body
        .blocks
        .iter()
        .map(|block| (block.start_address, 0usize))
        .collect::<HashMap<_, _>>();

    for block in &body.blocks {
        for statement in &block.statements {
            count_targets_in_statement(statement, &mut counts);
        }
    }

    counts
}

fn count_targets_in_statement(statement: &IrStatement, counts: &mut HashMap<u32, usize>) {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => {
            if let Some(count) = counts.get_mut(target_address) {
                *count += 1;
            }
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
        IrStatement::Block(block) => {
            for nested in &block.statements {
                count_targets_in_statement(nested, counts);
            }
        }
        IrStatement::BlockContainer(container) => {
            for nested_block in &container.blocks {
                for nested in &nested_block.statements {
                    count_targets_in_statement(nested, counts);
                }
            }
        }
        _ => {}
    }
}

fn region_has_no_extra_explicit_entries(
    region_blocks: &[IrBlock],
    incoming: &HashMap<u32, usize>,
) -> bool {
    for region_block in region_blocks.iter().skip(1) {
        if incoming
            .get(&region_block.start_address)
            .copied()
            .unwrap_or(0)
            != 0
        {
            return false;
        }
    }
    true
}

fn region_is_mergeable(region_blocks: &[IrBlock], region_end: u32) -> bool {
    for (index, region_block) in region_blocks.iter().enumerate() {
        let Some(tail) = region_block.statements.last() else {
            continue;
        };
        let is_last = index + 1 == region_blocks.len();
        if is_last
            && let IrStatement::Jump(IrJump { target_address }) = tail
            && *target_address == region_end
        {
            continue;
        }
        if statement_is_unreachable_endpoint(tail) {
            return false;
        }
    }
    true
}

fn find_block_index_by_start(container: &IrBlockContainer, start: u32) -> Option<usize> {
    container
        .blocks
        .iter()
        .position(|block| block.start_address == start)
}

fn find_block_by_start_address(container: &IrBlockContainer, start: u32) -> Option<usize> {
    container
        .blocks
        .iter()
        .position(|block| block.start_address == start)
}

fn remove_block_by_start(container: &mut IrBlockContainer, start: u32) {
    if let Some(index) = find_block_index_by_start(container, start) {
        container.blocks.remove(index);
    }
}

fn find_nested_block(body: &IrBlockContainer, nested_start: u32) -> Option<&IrBlock> {
    fn visit_statement(statement: &IrStatement, nested_start: u32) -> Option<&IrBlock> {
        match statement {
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                if let Some(found) = visit_statement(true_statement, nested_start) {
                    return Some(found);
                }
                if let Some(false_statement) = false_statement.as_ref()
                    && let Some(found) = visit_statement(false_statement, nested_start)
                {
                    return Some(found);
                }
                None
            }
            IrStatement::Block(block) => {
                if block.start_address == nested_start {
                    return Some(block);
                }
                for child in &block.statements {
                    if let Some(found) = visit_statement(child, nested_start) {
                        return Some(found);
                    }
                }
                None
            }
            IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. })
            | IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
                for child_block in &body.blocks {
                    for child in &child_block.statements {
                        if let Some(found) = visit_statement(child, nested_start) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
                for section in sections {
                    for child in &section.body.statements {
                        if let Some(found) = visit_statement(child, nested_start) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            IrStatement::BlockContainer(container) => {
                for child_block in &container.blocks {
                    for child in &child_block.statements {
                        if let Some(found) = visit_statement(child, nested_start) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    for block in &body.blocks {
        for statement in &block.statements {
            if let Some(found) = visit_statement(statement, nested_start) {
                return Some(found);
            }
        }
    }
    None
}

fn find_nested_block_mut(body: &mut IrBlockContainer, nested_start: u32) -> Option<&mut IrBlock> {
    fn visit_statement(statement: &mut IrStatement, nested_start: u32) -> Option<&mut IrBlock> {
        match statement {
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                if let Some(found) = visit_statement(true_statement, nested_start) {
                    return Some(found);
                }
                if let Some(false_statement) = false_statement.as_mut()
                    && let Some(found) = visit_statement(false_statement, nested_start)
                {
                    return Some(found);
                }
                None
            }
            IrStatement::Block(block) => {
                if block.start_address == nested_start {
                    return Some(block);
                }
                for child in &mut block.statements {
                    if let Some(found) = visit_statement(child, nested_start) {
                        return Some(found);
                    }
                }
                None
            }
            IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. })
            | IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
                for child_block in &mut body.blocks {
                    for child in &mut child_block.statements {
                        if let Some(found) = visit_statement(child, nested_start) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
                for section in sections {
                    for child in &mut section.body.statements {
                        if let Some(found) = visit_statement(child, nested_start) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            IrStatement::BlockContainer(container) => {
                for child_block in &mut container.blocks {
                    for child in &mut child_block.statements {
                        if let Some(found) = visit_statement(child, nested_start) {
                            return Some(found);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    for block in &mut body.blocks {
        for statement in &mut block.statements {
            if let Some(found) = visit_statement(statement, nested_start) {
                return Some(found);
            }
        }
    }
    None
}

fn new_block_start_address(first_statement: Option<&IrStatement>, state: &mut CleanupState) -> u32 {
    first_statement
        .and_then(statement_start_address)
        .unwrap_or_else(|| state.next_synthetic_block_address())
}

fn statement_start_address(statement: &IrStatement) -> Option<u32> {
    if let IrStatement::Block(block) = statement {
        return Some(block.start_address);
    }
    None
}
