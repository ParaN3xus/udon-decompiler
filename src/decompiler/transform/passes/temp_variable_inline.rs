use std::collections::{HashMap, HashSet, VecDeque};

use crate::decompiler::ir::{
    IrAssignmentStatement, IrBlock, IrBlockContainer, IrConstructorCallExpression, IrExpression,
    IrExpressionStatement, IrExternalCallExpression, IrFunction, IrHighLevelDoWhile,
    IrHighLevelSwitch, IrHighLevelWhile, IrIf, IrOperatorCallExpression,
    IrPropertyAccessExpression, IrReturn, IrStatement, IrSwitch,
};
use crate::decompiler::transform::ir_utils::{get_block_terminator, iter_block_targets};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};
use crate::decompiler::{Result, VariableRecord};

pub struct TempVariableInline;

impl ITransform for TempVariableInline {
    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let mut successor_cache = HashMap::<u32, HashMap<u32, Vec<u32>>>::new();
        let variables = context
            .program_context
            .decompile_context
            .variables
            .variables
            .iter()
            .map(|x| (x.address, x))
            .collect::<HashMap<_, _>>();
        inline_container(&mut function.body, &mut successor_cache, &variables);
        Ok(())
    }
}

fn inline_container(
    container: &mut IrBlockContainer,
    successor_cache: &mut HashMap<u32, HashMap<u32, Vec<u32>>>,
    variables: &HashMap<u32, &VariableRecord>,
) {
    let mut index = 0usize;
    while index < container.blocks.len() {
        let snapshot = container.clone();
        let Some(block) = container.blocks.get_mut(index) else {
            break;
        };
        inline_block(block, &snapshot, successor_cache, variables);
        index += 1;
    }
}

fn inline_block(
    block: &mut IrBlock,
    container: &IrBlockContainer,
    successor_cache: &mut HashMap<u32, HashMap<u32, Vec<u32>>>,
    variables: &HashMap<u32, &VariableRecord>,
) {
    let mut index = 1usize;
    while index < block.statements.len() {
        if try_inline_adjacent(block, container, index, successor_cache, variables) {
            index = index.saturating_sub(1).max(1);
            continue;
        }
        index += 1;
    }

    for statement in &mut block.statements {
        inline_nested(statement, container, successor_cache, variables);
    }
}

fn inline_nested(
    statement: &mut IrStatement,
    container: &IrBlockContainer,
    successor_cache: &mut HashMap<u32, HashMap<u32, Vec<u32>>>,
    variables: &HashMap<u32, &VariableRecord>,
) {
    match statement {
        IrStatement::Block(block) => inline_block(block, container, successor_cache, variables),
        IrStatement::BlockContainer(nested) => inline_container(nested, successor_cache, variables),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            inline_nested(true_statement, container, successor_cache, variables);
            if let Some(false_statement) = false_statement.as_mut() {
                inline_nested(false_statement, container, successor_cache, variables);
            }
        }
        _ => {}
    }
}

fn try_inline_adjacent(
    block: &mut IrBlock,
    container: &IrBlockContainer,
    index: usize,
    successor_cache: &mut HashMap<u32, HashMap<u32, Vec<u32>>>,
    variables: &HashMap<u32, &VariableRecord>,
) -> bool {
    let (target_address, replacement) = {
        let Some(prev_stmt) = block.statements.get(index - 1) else {
            return false;
        };
        let Some(curr_stmt) = block.statements.get(index) else {
            return false;
        };

        let IrStatement::Assignment(IrAssignmentStatement { target, value }) = prev_stmt else {
            return false;
        };
        let Some(target_address) = assignment_target_variable_address(target) else {
            return false;
        };
        let Some(target) = variables.get(&target_address) else {
            return false;
        };
        if !target.name.starts_with("__intnl_") {
            return false;
        }
        if writes_variable_top_level(curr_stmt, target_address) {
            return false;
        }
        if count_reads_top_level(curr_stmt, target_address) != 1 {
            return false;
        }
        if definition_flows_to_other_block_read(
            container,
            block,
            index,
            target_address,
            successor_cache,
        ) {
            return false;
        }

        (target_address, value.clone())
    };

    let Some(curr_stmt_mut) = block.statements.get_mut(index) else {
        return false;
    };
    if !replace_reads_top_level(curr_stmt_mut, target_address, &replacement) {
        return false;
    }
    block.statements.remove(index - 1);
    true
}

fn definition_flows_to_other_block_read(
    container: &IrBlockContainer,
    block: &IrBlock,
    index: usize,
    variable_address: u32,
    successor_cache: &mut HashMap<u32, HashMap<u32, Vec<u32>>>,
) -> bool {
    for statement in block.statements.iter().skip(index + 1) {
        if count_reads_in_statement(statement, variable_address) > 0 {
            return true;
        }
        if count_writes_in_statement(statement, variable_address) > 0 {
            return false;
        }
    }

    let successors = get_successor_map(container, successor_cache);
    let mut worklist = VecDeque::<u32>::new();
    if let Some(nexts) = successors.get(&block.start_address) {
        for addr in nexts {
            worklist.push_back(*addr);
        }
    }

    let mut visited = HashSet::<u32>::new();
    while let Some(current_address) = worklist.pop_front() {
        if !visited.insert(current_address) {
            continue;
        }

        let Some(current_block) = container
            .blocks
            .iter()
            .find(|x| x.start_address == current_address)
        else {
            continue;
        };

        let mut killed = false;
        for statement in &current_block.statements {
            if count_reads_in_statement(statement, variable_address) > 0 {
                return true;
            }
            if count_writes_in_statement(statement, variable_address) > 0 {
                killed = true;
                break;
            }
        }
        if !killed && let Some(nexts) = successors.get(&current_address) {
            for addr in nexts {
                worklist.push_back(*addr);
            }
        }
    }

    false
}

fn get_successor_map<'a>(
    container: &'a IrBlockContainer,
    successor_cache: &'a mut HashMap<u32, HashMap<u32, Vec<u32>>>,
) -> &'a HashMap<u32, Vec<u32>> {
    successor_cache
        .entry(container.id)
        .or_insert_with(|| build_successor_map(container))
}

fn build_successor_map(container: &IrBlockContainer) -> HashMap<u32, Vec<u32>> {
    let block_set = container
        .blocks
        .iter()
        .map(|x| x.start_address)
        .collect::<HashSet<_>>();

    let mut out = HashMap::<u32, Vec<u32>>::new();
    for (index, block) in container.blocks.iter().enumerate() {
        let next_block = container.blocks.get(index + 1).map(|x| x.start_address);
        let mut successors = Vec::<u32>::new();
        let Some(terminator) = get_block_terminator(block) else {
            if let Some(next_address) = next_block {
                successors.push(next_address);
            }
            out.insert(block.start_address, successors);
            continue;
        };

        for target in iter_block_targets(terminator) {
            if block_set.contains(&target) && !successors.contains(&target) {
                successors.push(target);
            }
        }
        if matches!(
            terminator,
            IrStatement::If(IrIf {
                false_statement: None,
                ..
            })
        ) && let Some(next_address) = next_block
            && !successors.contains(&next_address)
        {
            successors.push(next_address);
        }
        if matches!(
            terminator,
            IrStatement::Switch(IrSwitch {
                default_target: None,
                ..
            })
        ) && let Some(next_address) = next_block
            && !successors.contains(&next_address)
        {
            successors.push(next_address);
        }
        out.insert(block.start_address, successors);
    }

    out
}

fn writes_variable_top_level(statement: &IrStatement, address: u32) -> bool {
    matches!(statement, IrStatement::Assignment(IrAssignmentStatement { target, .. })
        if assignment_target_variable_address(target).is_some_and(|target_address| target_address == address))
}

fn count_reads_top_level(statement: &IrStatement, address: u32) -> usize {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, value }) => {
            count_assignment_target_reads(target, address) + count_expr_reads(value, address)
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            count_expr_reads(expression, address)
        }
        IrStatement::If(IrIf { condition, .. }) => count_expr_reads(condition, address),
        IrStatement::Switch(IrSwitch {
            index_expression, ..
        }) => count_expr_reads(index_expression, address),
        _ => 0,
    }
}

fn count_reads_in_statement(statement: &IrStatement, address: u32) -> usize {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, value }) => {
            count_assignment_target_reads(target, address) + count_expr_reads(value, address)
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            count_expr_reads(expression, address)
        }
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            let mut count = count_expr_reads(condition, address);
            count += count_reads_in_statement(true_statement, address);
            if let Some(false_statement) = false_statement.as_ref() {
                count += count_reads_in_statement(false_statement, address);
            }
            count
        }
        IrStatement::Switch(IrSwitch {
            index_expression, ..
        }) => count_expr_reads(index_expression, address),
        IrStatement::Block(block) => block
            .statements
            .iter()
            .map(|x| count_reads_in_statement(x, address))
            .sum(),
        IrStatement::BlockContainer(container) => container
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .map(|x| count_reads_in_statement(x, address))
            .sum(),
        IrStatement::HighLevelSwitch(IrHighLevelSwitch {
            index_expression,
            sections,
        }) => {
            let mut count = count_expr_reads(index_expression, address);
            count += sections
                .iter()
                .flat_map(|s| s.body.statements.iter())
                .map(|x| count_reads_in_statement(x, address))
                .sum::<usize>();
            count
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => body
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .map(|x| count_reads_in_statement(x, address))
            .sum(),
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => body
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .map(|x| count_reads_in_statement(x, address))
            .sum(),
        IrStatement::Jump(_) | IrStatement::Leave(_) | IrStatement::Return(IrReturn) => 0,
        _ => 0,
    }
}

fn count_writes_in_statement(statement: &IrStatement, address: u32) -> usize {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, .. }) => usize::from(
            assignment_target_variable_address(target)
                .is_some_and(|target_address| target_address == address),
        ),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut count = count_writes_in_statement(true_statement, address);
            if let Some(false_statement) = false_statement.as_ref() {
                count += count_writes_in_statement(false_statement, address);
            }
            count
        }
        IrStatement::Block(block) => block
            .statements
            .iter()
            .map(|x| count_writes_in_statement(x, address))
            .sum(),
        IrStatement::BlockContainer(container) => container
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .map(|x| count_writes_in_statement(x, address))
            .sum(),
        IrStatement::HighLevelSwitch(switch_stmt) => switch_stmt
            .sections
            .iter()
            .flat_map(|s| s.body.statements.iter())
            .map(|x| count_writes_in_statement(x, address))
            .sum(),
        IrStatement::HighLevelWhile(while_stmt) => while_stmt
            .body
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .map(|x| count_writes_in_statement(x, address))
            .sum(),
        IrStatement::HighLevelDoWhile(do_while_stmt) => do_while_stmt
            .body
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .map(|x| count_writes_in_statement(x, address))
            .sum(),
        _ => 0,
    }
}

fn count_expr_reads(expression: &IrExpression, address: u32) -> usize {
    match expression {
        IrExpression::Variable(variable_expression) => {
            usize::from(variable_expression.address == address)
        }
        IrExpression::OperatorCall(IrOperatorCallExpression { arguments, .. }) => {
            arguments.iter().map(|x| count_expr_reads(x, address)).sum()
        }
        IrExpression::ExternalCall(IrExternalCallExpression { arguments, .. }) => {
            arguments.iter().map(|x| count_expr_reads(x, address)).sum()
        }
        IrExpression::PropertyAccess(IrPropertyAccessExpression { arguments, .. }) => {
            arguments.iter().map(|x| count_expr_reads(x, address)).sum()
        }
        IrExpression::ConstructorCall(IrConstructorCallExpression { arguments, .. }) => {
            arguments.iter().map(|x| count_expr_reads(x, address)).sum()
        }
        _ => 0,
    }
}

fn replace_reads_top_level(
    statement: &mut IrStatement,
    address: u32,
    replacement: &IrExpression,
) -> bool {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, value }) => {
            replace_assignment_target_reads(target, address, replacement)
                + replace_expr_reads(value, address, replacement)
                == 1
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            replace_expr_reads(expression, address, replacement) == 1
        }
        IrStatement::If(IrIf { condition, .. }) => {
            replace_expr_reads(condition, address, replacement) == 1
        }
        IrStatement::Switch(IrSwitch {
            index_expression, ..
        }) => replace_expr_reads(index_expression, address, replacement) == 1,
        _ => false,
    }
}

fn replace_expr_reads(
    expression: &mut IrExpression,
    address: u32,
    replacement: &IrExpression,
) -> usize {
    match expression {
        IrExpression::Variable(variable_expression) => {
            if variable_expression.address == address {
                *expression = replacement.clone();
                1
            } else {
                0
            }
        }
        IrExpression::OperatorCall(IrOperatorCallExpression { arguments, .. }) => arguments
            .iter_mut()
            .map(|x| replace_expr_reads(x, address, replacement))
            .sum(),
        IrExpression::ExternalCall(IrExternalCallExpression { arguments, .. }) => arguments
            .iter_mut()
            .map(|x| replace_expr_reads(x, address, replacement))
            .sum(),
        IrExpression::PropertyAccess(IrPropertyAccessExpression { arguments, .. }) => arguments
            .iter_mut()
            .map(|x| replace_expr_reads(x, address, replacement))
            .sum(),
        IrExpression::ConstructorCall(IrConstructorCallExpression { arguments, .. }) => arguments
            .iter_mut()
            .map(|x| replace_expr_reads(x, address, replacement))
            .sum(),
        _ => 0,
    }
}

fn assignment_target_variable_address(target: &IrExpression) -> Option<u32> {
    match target {
        IrExpression::Variable(variable) => Some(variable.address),
        _ => None,
    }
}

fn count_assignment_target_reads(target: &IrExpression, address: u32) -> usize {
    match target {
        IrExpression::Variable(_) => 0,
        IrExpression::PropertyAccess(_) => count_expr_reads(target, address),
        _ => 0,
    }
}

fn replace_assignment_target_reads(
    target: &mut IrExpression,
    address: u32,
    replacement: &IrExpression,
) -> usize {
    match target {
        IrExpression::Variable(_) => 0,
        IrExpression::PropertyAccess(_) => replace_expr_reads(target, address, replacement),
        _ => 0,
    }
}
