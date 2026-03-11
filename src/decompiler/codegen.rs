use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::decompiler::context::DecompileContext;
use crate::decompiler::ir::{
    IrAssignmentStatement, IrBlock, IrBlockContainer, IrClass, IrContainerKind, IrExpression,
    IrExpressionStatement, IrExternalCallExpression, IrFunction, IrHighLevelSwitch, IrIf, IrJump,
    IrLeave, IrLiteralExpression, IrOperator, IrOperatorCallExpression, IrPropertyAccessExpression,
    IrStatement, IrSwitch,
};
use crate::decompiler::{DecompileError, ParameterType, Result, VariableRecord};
use crate::str_constants::{
    CLANG_FORMAT_ASSUME_FILENAME_CS, TYPE_SYSTEM_BOOLEAN, TYPE_SYSTEM_BYTE, TYPE_SYSTEM_DOUBLE,
    TYPE_SYSTEM_INT16, TYPE_SYSTEM_INT32, TYPE_SYSTEM_INT64, TYPE_SYSTEM_OBJECT, TYPE_SYSTEM_SBYTE,
    TYPE_SYSTEM_SINGLE, TYPE_SYSTEM_STRING, TYPE_SYSTEM_UINT16, TYPE_SYSTEM_UINT32,
    TYPE_SYSTEM_UINT64, TYPE_UNITY_GAME_OBJECT, TYPE_UNITY_TRANSFORM, TYPE_UNSERIALIZABLE,
    UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL, UNSERIALIZABLE_LITERAL,
};
use crate::udon_asm::{generated_heap_symbol, parse_heap_literal, render_heap_literal};

pub fn generate_csharp(ctx: &DecompileContext, class_ir: &IrClass) -> Result<String> {
    let mut class_lines = Vec::<String>::new();
    class_lines.push(format!(
        "public class {} : UdonSharpBehaviour",
        sanitize_identifier(class_ir.class_name.as_str())
    ));
    class_lines.push("{".to_string());

    let global_declarations = class_ir.variable_declarations.iter().collect::<Vec<_>>();

    for declaration in &global_declarations {
        let variable = require_variable(ctx, declaration.variable_address);
        class_lines.push(render_variable_declaration(
            variable,
            variable.name.as_str(),
            declaration
                .init_value
                .as_ref()
                .map(|x| render_literal_for_type(x, variable.type_name.as_str()))
                .as_deref(),
            variable.exported,
        ));
    }

    if !global_declarations.is_empty() {
        class_lines.push(String::new());
    }

    for (function_index, function) in class_ir.functions.iter().enumerate() {
        append_function(&mut class_lines, function, ctx);
        if function_index + 1 != class_ir.functions.len() {
            class_lines.push(String::new());
        }
    }

    if class_ir.functions.is_empty() {
        class_lines.push("// No functions discovered yet.".to_string());
    }

    class_lines.push("}".to_string());

    let mut out = vec![
        "// Decompiled Udon Program".to_string(),
        "// This is pseudo-code and may not compile directly".to_string(),
        String::new(),
    ];

    if let Some(namespace) = class_ir.namespace.as_deref().map(str::trim)
        && !namespace.is_empty()
    {
        out.push(format!("namespace {namespace}"));
        out.push("{".to_string());
        out.extend(class_lines);
        out.push("}".to_string());
    } else {
        out.extend(class_lines);
    }

    format_csharp(out.join("\n").as_str())
}

fn format_csharp(code: &str) -> Result<String> {
    let style = "{BasedOnStyle: Google, Language: CSharp, IndentWidth: 4, ColumnLimit: 160, BreakBeforeBraces: Allman}";

    let mut child = Command::new("clang-format")
        .arg(format!(
            "--assume-filename={}",
            CLANG_FORMAT_ASSUME_FILENAME_CS
        ))
        .arg(format!("-style={style}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => {
                DecompileError::new("clang-format not found on PATH. Install it or adjust PATH.")
            }
            _ => DecompileError::new(format!("failed to start clang-format: {error}")),
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(code.as_bytes()).map_err(|error| {
            DecompileError::new(format!("failed to write clang-format stdin: {error}"))
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        DecompileError::new(format!("failed to wait for clang-format: {error}"))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            "unknown error"
        } else {
            stderr.as_str()
        };
        return Err(DecompileError::new(format!(
            "clang-format failed: {detail}"
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| DecompileError::new(format!("clang-format output is not utf-8: {error}")))
}

fn append_function(
    out: &mut Vec<String>,
    function: &IrFunction,
    ctx: &DecompileContext,
) {
    out.push(format!(
        "{}void {}()",
        if function.is_function_public {
            "public "
        } else {
            ""
        },
        sanitize_identifier(function.function_name.as_str())
    ));
    out.push("{".to_string());

    let mut body_lines = Vec::<String>::new();
    append_function_body(&mut body_lines, function, ctx);
    let body_text = body_lines.join("\n");

    let local_declarations = function
        .variable_declarations
        .iter()
        .filter_map(|declaration| {
            let variable = require_variable(ctx, declaration.variable_address);
            let name = variable.name.as_str();
            variable_name_used_in_body(name, body_text.as_str())
                .then(|| render_local_variable_declaration(variable, name, declaration.init_value.as_ref()))
        })
        .collect::<Vec<_>>();

    for declaration in &local_declarations {
        out.push(declaration.clone());
    }
    if !local_declarations.is_empty() && !body_lines.is_empty() {
        out.push(String::new());
    }

    out.extend(body_lines);
    out.push("}".to_string());
}

fn append_function_body(
    out: &mut Vec<String>,
    function: &IrFunction,
    ctx: &DecompileContext,
) {
    let emit_indices = function
        .body
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(idx, block)| {
            (!block.statements.is_empty() || block.should_emit_label).then_some(idx)
        })
        .collect::<Vec<_>>();

    for (emit_idx, block_index) in emit_indices.iter().enumerate() {
        let block = &function.body.blocks[*block_index];
        if block.should_emit_label && !is_synthetic_block_address(block.start_address) {
            out.push(format!("label_bb_{:08x}:", block.start_address));
        }
        let next_block = emit_indices
            .get(emit_idx + 1)
            .and_then(|next_index| function.body.blocks.get(*next_index))
            .map(|next| next.start_address);
        append_block_statements(out, block, ctx, next_block);
    }
}

fn append_block_statements(
    out: &mut Vec<String>,
    block: &IrBlock,
    ctx: &DecompileContext,
    next_block: Option<u32>,
) {
    if block.statements.is_empty() {
        out.push(";".to_string());
        return;
    }

    let mut index = 0usize;
    while index < block.statements.len() {
        if let Some(consumed) = try_render_exclusive_if_chain_as_nested_else(
            out, block, index, ctx, None, None,
        ) {
            index += consumed;
            continue;
        }
        if let Some(consumed) =
            try_render_if_return_chain_as_else_if(out, block, index, ctx, None, None)
        {
            index += consumed;
            continue;
        }
        if let Some(consumed) =
            try_render_if_suffix_as_else(out, block, index, ctx, None, None, false, false)
        {
            index += consumed;
            continue;
        }
        let statement = &block.statements[index];
        let statement_next = if index + 1 == block.statements.len() {
            next_block
        } else {
            None
        };
        let next_statement_is_return_like = block
            .statements
            .get(index + 1)
            .is_some_and(is_return_like_statement);
        append_statement_with_context(
            out,
            statement,
            ctx,
            FlowRenderContext {
                next_block: statement_next,
                break_target_container: None,
                continue_target: None,
                loop_break_target: None,
                has_following_statement: index + 1 < block.statements.len(),
                next_statement_is_return_like,
            },
        );
        index += 1;
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct FlowRenderContext {
    next_block: Option<u32>,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
    loop_break_target: Option<u32>,
    has_following_statement: bool,
    next_statement_is_return_like: bool,
}

fn append_statement_with_context(
    out: &mut Vec<String>,
    statement: &IrStatement,
    ctx: &DecompileContext,
    context: FlowRenderContext,
) {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement {
            target_address,
            value,
        }) => {
            let variable = require_variable(ctx, *target_address);
            let name = variable.name.as_str();
            out.push(format!("{name} = {};", render_expression(value, ctx)));
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            out.push(format!("{};", render_expression(expression, ctx)));
        }
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            let cond_expr = condition.clone();
            let mut true_stmt = true_statement.as_ref();
            let mut false_stmt = false_statement.as_deref();

            // Collapse redundant nested guards: if (A && B) { if (B) { ... } }
            if false_stmt.is_none()
                && let IrStatement::Block(block) = true_stmt
                && block.statements.len() == 1
                && let Some(IrStatement::If(nested_if)) = block.statements.first()
                && nested_if.false_statement.is_none()
                && condition_contains(&cond_expr, &nested_if.condition)
            {
                true_stmt = nested_if.true_statement.as_ref();
            }
            let owned_true_stmt: Option<IrStatement>;
            let owned_false_stmt: Option<IrStatement>;
            let should_strip_branch_returns = false_stmt.is_some()
                && (context.next_statement_is_return_like
                    || (context.has_following_statement
                        && statement_ends_unreachable(true_stmt)
                        && false_stmt
                            .as_ref()
                            .is_some_and(|stmt| statement_ends_unreachable(stmt))));
            if should_strip_branch_returns {
                owned_true_stmt = Some(strip_terminal_return_like(true_stmt));
                true_stmt = owned_true_stmt.as_ref().expect("owned true present");
                if let Some(false_statement) = false_stmt {
                    owned_false_stmt = Some(strip_terminal_return_like(false_statement));
                    false_stmt = owned_false_stmt.as_ref();
                }
            }
            out.push(format!("if ({})", render_expression(&cond_expr, ctx)));
            out.push("{".to_string());
            append_statement_with_context(
                out,
                true_stmt,
                ctx,
                FlowRenderContext {
                    next_block: None,
                    break_target_container: context.break_target_container,
                    continue_target: context.continue_target,
                    loop_break_target: context.loop_break_target,
                    has_following_statement: context.has_following_statement,
                    next_statement_is_return_like: context.next_statement_is_return_like,
                },
            );
            out.push("}".to_string());
            if let Some(false_statement) = false_stmt {
                out.push("else".to_string());
                out.push("{".to_string());
                append_statement_with_context(
                    out,
                    false_statement,
                    ctx,
                    FlowRenderContext {
                        next_block: None,
                        break_target_container: context.break_target_container,
                        continue_target: context.continue_target,
                        loop_break_target: context.loop_break_target,
                        has_following_statement: context.has_following_statement,
                        next_statement_is_return_like: context.next_statement_is_return_like,
                    },
                );
                out.push("}".to_string());
            }
        }
        IrStatement::Jump(IrJump { target_address }) => {
            if context
                .continue_target
                .is_some_and(|target| target == *target_address)
            {
                out.push("continue;".to_string());
            } else if context
                .loop_break_target
                .is_some_and(|target| target == *target_address)
            {
                out.push("break;".to_string());
            } else if context
                .next_block
                .is_some_and(|next| next == *target_address)
            {
            } else {
                out.push(format!("goto label_bb_{target_address:08x};"));
            }
        }
        IrStatement::Leave(IrLeave {
            target_container_id,
        }) => {
            if context
                .break_target_container
                .is_some_and(|target| target == *target_container_id)
            {
                out.push("break;".to_string());
            } else {
                out.push("return;".to_string());
            }
        }
        IrStatement::Return(_) => {
            out.push("return;".to_string());
        }
        IrStatement::Switch(IrSwitch {
            index_expression,
            cases,
            default_target,
        }) => {
            out.push(format!(
                "switch ({})",
                render_expression(index_expression, ctx)
            ));
            out.push("{".to_string());
            for (case_value, target) in cases {
                out.push(format!("case {case_value}: goto label_bb_{target:08x};"));
            }
            if let Some(default_target) = default_target {
                out.push(format!("default: goto label_bb_{default_target:08x};"));
            } else {
                out.push("default: break;".to_string());
            }
            out.push("}".to_string());
        }
        IrStatement::Block(nested_block) => {
            let mut idx = 0usize;
            while idx < nested_block.statements.len() {
                if let Some(consumed) = try_render_exclusive_if_chain_as_nested_else(
                    out,
                    nested_block,
                    idx,
                    ctx,
                    context.break_target_container,
                    context.continue_target,
                ) {
                    idx += consumed;
                    continue;
                }
                if let Some(consumed) = try_render_if_return_chain_as_else_if(
                    out,
                    nested_block,
                    idx,
                    ctx,
                    context.break_target_container,
                    context.continue_target,
                ) {
                    idx += consumed;
                    continue;
                }
                if let Some(consumed) = try_render_if_suffix_as_else(
                    out,
                    nested_block,
                    idx,
                    ctx,
                    context.break_target_container,
                    context.continue_target,
                    context.has_following_statement,
                    context.next_statement_is_return_like,
                ) {
                    idx += consumed;
                    continue;
                }
                let nested = &nested_block.statements[idx];
                let next_statement_is_return_like = nested_block
                    .statements
                    .get(idx + 1)
                    .is_some_and(is_return_like_statement);
                append_statement_with_context(
                    out,
                    nested,
                    ctx,
                    FlowRenderContext {
                        next_block: None,
                        break_target_container: context.break_target_container,
                        continue_target: context.continue_target,
                        loop_break_target: context.loop_break_target,
                        has_following_statement: idx + 1 < nested_block.statements.len()
                            || context.has_following_statement,
                        next_statement_is_return_like,
                    },
                );
                idx += 1;
            }
        }
        IrStatement::BlockContainer(container) => {
            append_container_with_context(
                out,
                container,
                ctx,
                context.break_target_container,
                context.continue_target,
            );
        }
        IrStatement::VariableDeclaration(declaration) => {
            let variable = require_variable(ctx, declaration.variable_address);
            out.push(render_local_variable_declaration(
                variable,
                variable.name.as_str(),
                declaration.init_value.as_ref(),
            ));
        }
        IrStatement::HighLevelSwitch(switch_stmt) => {
            append_high_level_switch(out, switch_stmt, ctx, context);
        }
        IrStatement::HighLevelWhile(while_stmt) => {
            if try_render_guarded_infinite_while(out, while_stmt, ctx, context) {
                return;
            }
            let condition = while_stmt
                .condition
                .as_ref()
                .map(|expr| render_expression(expr, ctx))
                .unwrap_or_else(|| "true".to_string());
            out.push(format!("while ({condition})"));
            out.push("{".to_string());
            append_container_with_context(
                out,
                &while_stmt.body,
                ctx,
                Some(while_stmt.break_target),
                Some(while_stmt.continue_target),
            );
            out.push("}".to_string());
        }
        IrStatement::HighLevelDoWhile(do_while_stmt) => {
            if do_while_contains_break_target(&do_while_stmt.body, do_while_stmt.break_target) {
                out.push("while (true)".to_string());
                out.push("{".to_string());
                append_container_with_context(
                    out,
                    &do_while_stmt.body,
                    ctx,
                    Some(do_while_stmt.break_target),
                    Some(do_while_stmt.continue_target),
                );
                let cond = render_expression(&do_while_stmt.condition, ctx);
                out.push(format!("if ({cond})"));
                out.push("{".to_string());
                out.push("continue;".to_string());
                out.push("}".to_string());
                out.push("break;".to_string());
                out.push("}".to_string());
                return;
            }
            out.push("do".to_string());
            out.push("{".to_string());
            append_container_with_context(
                out,
                &do_while_stmt.body,
                ctx,
                Some(do_while_stmt.break_target),
                Some(do_while_stmt.continue_target),
            );
            out.push(format!(
                "}} while ({});",
                render_expression(&do_while_stmt.condition, ctx)
            ));
        }
    }
}

fn append_container_with_context(
    out: &mut Vec<String>,
    container: &IrBlockContainer,
    ctx: &DecompileContext,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
) {
    let jump_target_counts = collect_jump_target_counts_in_container(container);
    let _block_by_address = container
        .blocks
        .iter()
        .map(|x| (x.start_address, x))
        .collect::<HashMap<_, _>>();
    let mut sorted_block_addrs = container
        .blocks
        .iter()
        .map(|x| x.start_address)
        .collect::<Vec<_>>();
    sorted_block_addrs.sort_unstable();
    let skipped_block_addresses = std::collections::HashSet::<u32>::new();
    let mut suppressed_label_addresses = std::collections::HashSet::<u32>::new();

    for (index, block) in container.blocks.iter().enumerate() {
        if skipped_block_addresses.contains(&block.start_address) {
            continue;
        }
        let block_next = container
            .blocks
            .get(index + 1)
            .map(|next| next.start_address);
        if try_render_self_loop_while(
            out,
            block,
            ctx,
            block_next,
            break_target_container,
            continue_target,
            &mut suppressed_label_addresses,
            &jump_target_counts,
        ) {
            continue;
        }
        if block.should_emit_label
            && !suppressed_label_addresses.contains(&block.start_address)
            && !is_synthetic_block_address(block.start_address)
        {
            out.push(format!("label_bb_{:08x}:", block.start_address));
        }

        let _flow_context = FlowRenderContext {
            next_block: container
                .blocks
                .get(index + 1)
                .map(|next| next.start_address),
            break_target_container,
            continue_target,
            loop_break_target: None,
            has_following_statement: false,
            next_statement_is_return_like: false,
        };
        let mut statement_index = 0usize;
        while statement_index < block.statements.len() {
            if let Some(consumed) = try_render_exclusive_if_chain_as_nested_else(
                out,
                block,
                statement_index,
                ctx,
                break_target_container,
                continue_target,
            ) {
                statement_index += consumed;
                continue;
            }
            if let Some(consumed) = try_render_if_return_chain_as_else_if(
                out,
                block,
                statement_index,
                ctx,
                break_target_container,
                continue_target,
            ) {
                statement_index += consumed;
                continue;
            }
            if let Some(consumed) = try_render_if_suffix_as_else(
                out,
                block,
                statement_index,
                ctx,
                break_target_container,
                continue_target,
                false,
                false,
            ) {
                statement_index += consumed;
                continue;
            }
            let statement = &block.statements[statement_index];
            let next_block = (statement_index + 1 == block.statements.len())
                .then_some(index + 1)
                .and_then(|next_index| container.blocks.get(next_index))
                .map(|next| next.start_address);
            let next_statement_is_return_like = block
                .statements
                .get(statement_index + 1)
                .is_some_and(is_return_like_statement);
            append_statement_with_context(
                out,
                statement,
                ctx,
                FlowRenderContext {
                    next_block,
                    break_target_container,
                    continue_target,
                    loop_break_target: None,
                    has_following_statement: statement_index + 1 < block.statements.len(),
                    next_statement_is_return_like,
                },
            );
            statement_index += 1;
        }
    }
}

fn try_render_exclusive_if_chain_as_nested_else(
    out: &mut Vec<String>,
    block: &IrBlock,
    start_index: usize,
    ctx: &DecompileContext,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
) -> Option<usize> {
    let Some(first) = block.statements.get(start_index) else {
        return None;
    };
    let IrStatement::If(IrIf {
        condition: first_condition,
        false_statement: first_false,
        ..
    }) = first
    else {
        return None;
    };
    if first_false.is_some() {
        return None;
    }
    let (lhs_addr, first_literal) = extract_var_equals_literal_key(first_condition)?;

    let mut entries = vec![(
        first_condition,
        match first {
            IrStatement::If(IrIf { true_statement, .. }) => true_statement.as_ref(),
            _ => unreachable!(),
        },
    )];
    let mut seen_literals = HashSet::<String>::new();
    seen_literals.insert(first_literal);

    let mut idx = start_index + 1;
    while let Some(IrStatement::If(IrIf {
        condition,
        true_statement,
        false_statement,
    })) = block.statements.get(idx)
    {
        if false_statement.is_some() {
            break;
        }
        let Some((addr, literal)) = extract_var_equals_literal_key(condition) else {
            break;
        };
        if addr != lhs_addr || !seen_literals.insert(literal) {
            break;
        }
        entries.push((condition, true_statement));
        idx += 1;
    }

    if entries.len() < 2 {
        return None;
    }
    let tail_return_like = block
        .statements
        .get(start_index + entries.len())
        .is_some_and(is_return_like_statement);

    let mut else_depth = 0usize;
    for (entry_index, (condition, true_statement)) in entries.iter().enumerate() {
        if entry_index == 0 {
            out.push(format!("if ({})", render_expression(condition, ctx)));
        } else {
            out.push("else".to_string());
            out.push("{".to_string());
            else_depth += 1;
            out.push(format!("if ({})", render_expression(condition, ctx)));
        }

        out.push("{".to_string());
        let owned_true_statement =
            tail_return_like.then(|| strip_terminal_return_like(true_statement));
        let emitted_true_statement = owned_true_statement
            .as_ref()
            .map_or(*true_statement, |statement| statement);
        append_statement_with_context(
            out,
            emitted_true_statement,
            ctx,
            FlowRenderContext {
                next_block: None,
                break_target_container,
                continue_target,
                loop_break_target: None,
                has_following_statement: tail_return_like,
                next_statement_is_return_like: tail_return_like,
            },
        );
        out.push("}".to_string());
    }
    for _ in (1..=else_depth).rev() {
        out.push("}".to_string());
    }

    Some(entries.len())
}

fn extract_var_equals_literal_key(condition: &IrExpression) -> Option<(u32, String)> {
    let IrExpression::OperatorCall(IrOperatorCallExpression {
        arguments,
        operator,
        ..
    }) = condition
    else {
        return None;
    };
    if *operator != IrOperator::Equality || arguments.len() != 2 {
        return None;
    }
    match (&arguments[0], &arguments[1]) {
        (IrExpression::Variable(variable_expr), IrExpression::Literal(literal_expr)) => {
            Some((variable_expr.address, literal_expr.value.clone()))
        }
        (IrExpression::Literal(literal_expr), IrExpression::Variable(variable_expr)) => {
            Some((variable_expr.address, literal_expr.value.clone()))
        }
        _ => None,
    }
}

fn try_render_if_return_chain_as_else_if(
    out: &mut Vec<String>,
    block: &IrBlock,
    start_index: usize,
    ctx: &DecompileContext,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
) -> Option<usize> {
    let mut entries = Vec::<(&IrExpression, &IrStatement)>::new();
    let mut idx = start_index;
    while idx < block.statements.len() {
        let IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) = &block.statements[idx]
        else {
            break;
        };
        if false_statement.is_some() {
            break;
        }
        if !statement_ends_unreachable(true_statement) {
            break;
        }
        entries.push((condition, true_statement));
        idx += 1;
    }

    if entries.len() < 2 {
        return None;
    }
    let Some(tail) = block.statements.get(idx) else {
        return None;
    };
    if !is_return_like_statement(tail) {
        return None;
    }

    for (entry_index, (condition, true_statement)) in entries.iter().enumerate() {
        if entry_index == 0 {
            out.push(format!("if ({})", render_expression(condition, ctx)));
        } else {
            out.push(format!("else if ({})", render_expression(condition, ctx)));
        }
        out.push("{".to_string());
        append_statement_with_context(
            out,
            true_statement,
            ctx,
            FlowRenderContext {
                next_block: None,
                break_target_container,
                continue_target,
                loop_break_target: None,
                has_following_statement: false,
                next_statement_is_return_like: false,
            },
        );
        out.push("}".to_string());
    }

    Some(entries.len())
}

fn try_render_if_suffix_as_else(
    out: &mut Vec<String>,
    block: &IrBlock,
    start_index: usize,
    ctx: &DecompileContext,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
    parent_has_following_statement: bool,
    parent_next_statement_is_return_like: bool,
) -> Option<usize> {
    if continue_target.is_some() {
        return None;
    }
    if start_index + 1 >= block.statements.len() {
        return None;
    }

    let IrStatement::If(IrIf {
        condition,
        true_statement,
        false_statement,
    }) = &block.statements[start_index]
    else {
        return None;
    };
    if false_statement.is_some() {
        return None;
    }
    let mut normalized_true_statement = true_statement.as_ref();
    if let IrStatement::Block(true_block) = normalized_true_statement
        && true_block.statements.len() == 1
        && let Some(IrStatement::If(nested_if)) = true_block.statements.first()
        && nested_if.false_statement.is_none()
        && condition_contains(condition, &nested_if.condition)
    {
        normalized_true_statement = nested_if.true_statement.as_ref();
    }
    let true_exits_via_while = while_block_exits_outer_scope(normalized_true_statement);
    let true_branch_is_unreachable =
        statement_ends_unreachable(normalized_true_statement) || true_exits_via_while;
    if !true_branch_is_unreachable {
        return None;
    }
    if statement_ends_with_jump(normalized_true_statement) && !true_exits_via_while {
        return None;
    }

    let suffix = &block.statements[start_index + 1..];
    if suffix.len() < 2 {
        return None;
    }
    if !true_exits_via_while && !statements_end_unreachable(suffix) {
        return None;
    }
    if has_nested_control_flow_statement(suffix)
        && !true_exits_via_while
        && !allow_nested_suffix_as_else(normalized_true_statement, suffix)
    {
        return None;
    }

    let keep_tail_return = suffix.last().is_some_and(is_return_like_statement);
    let emitted_suffix = if keep_tail_return {
        &suffix[..suffix.len() - 1]
    } else {
        suffix
    };
    if emitted_suffix.is_empty() {
        return None;
    }

    let consumed = 1 + emitted_suffix.len();
    let has_post_statement_in_block = start_index + consumed < block.statements.len();
    let has_following_after_transformed =
        has_post_statement_in_block || parent_has_following_statement;
    let next_statement_after_transformed_is_return_like = if has_post_statement_in_block {
        block
            .statements
            .get(start_index + consumed)
            .is_some_and(is_return_like_statement)
    } else {
        parent_next_statement_is_return_like
    };
    let should_strip_branch_returns = next_statement_after_transformed_is_return_like
        || (has_following_after_transformed
            && statement_ends_unreachable(normalized_true_statement)
            && statements_end_unreachable(emitted_suffix));
    let owned_true_statement: Option<IrStatement>;
    let rendered_true_statement = if should_strip_branch_returns {
        owned_true_statement = Some(strip_terminal_return_like(normalized_true_statement));
        owned_true_statement
            .as_ref()
            .expect("owned true statement should exist")
    } else {
        normalized_true_statement
    };

    out.push(format!("if ({})", render_expression(condition, ctx)));
    out.push("{".to_string());
    append_statement_with_context(
        out,
        rendered_true_statement,
        ctx,
        FlowRenderContext {
            next_block: None,
            break_target_container,
            continue_target,
            loop_break_target: None,
            has_following_statement: has_following_after_transformed,
            next_statement_is_return_like: next_statement_after_transformed_is_return_like,
        },
    );
    out.push("}".to_string());
    out.push("else".to_string());
    out.push("{".to_string());
    for (suffix_index, suffix_statement) in emitted_suffix.iter().enumerate() {
        let owned_suffix_statement: Option<IrStatement>;
        let rendered_suffix_statement =
            if should_strip_branch_returns && suffix_index + 1 == emitted_suffix.len() {
                owned_suffix_statement = Some(strip_terminal_return_like(suffix_statement));
                owned_suffix_statement
                    .as_ref()
                    .expect("owned suffix statement should exist")
            } else {
                suffix_statement
            };
        let has_next_in_suffix = suffix_index + 1 < emitted_suffix.len();
        let next_statement_is_return_like = if has_next_in_suffix {
            emitted_suffix
                .get(suffix_index + 1)
                .is_some_and(is_return_like_statement)
        } else {
            next_statement_after_transformed_is_return_like
        };
        let has_following_statement = has_next_in_suffix || has_following_after_transformed;
        append_statement_with_context(
            out,
            rendered_suffix_statement,
            ctx,
            FlowRenderContext {
                next_block: None,
                break_target_container,
                continue_target,
                loop_break_target: None,
                has_following_statement,
                next_statement_is_return_like,
            },
        );
    }
    out.push("}".to_string());
    Some(consumed)
}

fn append_high_level_switch(
    out: &mut Vec<String>,
    switch_stmt: &IrHighLevelSwitch,
    ctx: &DecompileContext,
    _context: FlowRenderContext,
) {
    out.push(format!(
        "switch ({})",
        render_expression(&switch_stmt.index_expression, ctx)
    ));
    out.push("{".to_string());
    for section in &switch_stmt.sections {
        for label in &section.labels {
            out.push(format!("case {label}:"));
        }
        if section.is_default {
            out.push("default:".to_string());
        }
        let drop_terminal_return = section
            .body
            .statements
            .last()
            .is_some_and(is_return_like_statement);
        let body_len = section.body.statements.len();
        let emit_len = if drop_terminal_return && body_len > 0 {
            body_len - 1
        } else {
            body_len
        };
        for statement in &section.body.statements[..emit_len] {
            append_statement_with_context(
                out,
                statement,
                ctx,
                FlowRenderContext {
                    next_block: None,
                    break_target_container: None,
                    continue_target: None,
                    loop_break_target: None,
                    has_following_statement: false,
                    next_statement_is_return_like: false,
                },
            );
        }
        if switch_section_needs_break_slice(&section.body.statements[..emit_len]) {
            out.push("break;".to_string());
        }
    }
    out.push("}".to_string());
}

fn switch_section_needs_break_slice(statements: &[IrStatement]) -> bool {
    let Some(last) = statements.last() else {
        return true;
    };
    !matches!(
        last,
        IrStatement::Jump(_) | IrStatement::Leave(_) | IrStatement::Return(_)
    )
}

fn strip_terminal_return_like(statement: &IrStatement) -> IrStatement {
    let IrStatement::Block(block) = statement else {
        return statement.clone();
    };
    let mut stripped = block.clone();
    if stripped
        .statements
        .last()
        .is_some_and(is_return_like_statement)
    {
        stripped.statements.pop();
    }
    IrStatement::Block(stripped)
}

fn is_return_like_statement(statement: &IrStatement) -> bool {
    matches!(statement, IrStatement::Return(_) | IrStatement::Leave(_))
}

fn statements_end_unreachable(statements: &[IrStatement]) -> bool {
    statements.last().is_some_and(statement_ends_unreachable)
}

fn statement_ends_with_jump(statement: &IrStatement) -> bool {
    match statement {
        IrStatement::Jump(_) => true,
        IrStatement::Block(block) => block
            .statements
            .last()
            .is_some_and(statement_ends_with_jump),
        _ => false,
    }
}

fn statement_ends_unreachable(statement: &IrStatement) -> bool {
    match statement {
        IrStatement::Jump(_) | IrStatement::Leave(_) | IrStatement::Return(_) => true,
        IrStatement::Block(block) => statements_end_unreachable(&block.statements),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => false_statement.as_ref().is_some_and(|x| {
            statement_ends_unreachable(true_statement) && statement_ends_unreachable(x)
        }),
        IrStatement::HighLevelSwitch(switch_stmt) => {
            !switch_stmt.sections.is_empty()
                && switch_stmt
                    .sections
                    .iter()
                    .all(|section| statements_end_unreachable(&section.body.statements))
        }
        _ => false,
    }
}

fn has_nested_control_flow_statement(statements: &[IrStatement]) -> bool {
    for statement in statements {
        match statement {
            IrStatement::If(_)
            | IrStatement::BlockContainer(_)
            | IrStatement::HighLevelSwitch(_)
            | IrStatement::HighLevelWhile(_)
            | IrStatement::HighLevelDoWhile(_)
            | IrStatement::Switch(_) => return true,
            IrStatement::Block(block) => {
                if has_nested_control_flow_statement(&block.statements) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn while_block_exits_outer_scope(statement: &IrStatement) -> bool {
    let IrStatement::Block(block) = statement else {
        return false;
    };
    if block.statements.is_empty() {
        return false;
    }
    let Some(IrStatement::HighLevelWhile(while_stmt)) = block.statements.last() else {
        return false;
    };
    if !block.statements[..block.statements.len() - 1]
        .iter()
        .all(|x| {
            matches!(
                x,
                IrStatement::Assignment(_)
                    | IrStatement::Expression(_)
                    | IrStatement::VariableDeclaration(_)
            )
        })
    {
        return false;
    }
    if while_stmt.condition.is_some() {
        return false;
    }
    let Some(entry_block) = while_stmt.body.blocks.first() else {
        return false;
    };
    let Some(last) = entry_block.statements.last() else {
        return false;
    };
    match last {
        IrStatement::Return(_) => true,
        IrStatement::Leave(IrLeave {
            target_container_id,
        }) => *target_container_id != while_stmt.break_target,
        _ => false,
    }
}

fn allow_nested_suffix_as_else(true_statement: &IrStatement, suffix: &[IrStatement]) -> bool {
    let IrStatement::Block(true_block) = true_statement else {
        return false;
    };
    if true_block.statements.is_empty() {
        return false;
    }
    let Some(IrStatement::HighLevelWhile(while_stmt)) = true_block.statements.last() else {
        return false;
    };
    if while_stmt.condition.is_some() {
        return false;
    }
    if !true_block.statements[..true_block.statements.len() - 1]
        .iter()
        .all(|statement| {
            matches!(
                statement,
                IrStatement::Assignment(_)
                    | IrStatement::Expression(_)
                    | IrStatement::VariableDeclaration(_)
            )
        })
    {
        return false;
    }
    let Some(last) = suffix.last() else {
        return false;
    };
    if !is_return_like_statement(last) {
        return false;
    }

    // Allow a narrow shape: if (cond) { while (...) ... } <linear suffix> return;
    // This reconstructs Python-style else blocks for event-dispatch patterns.
    let check_slice = &suffix[..suffix.len() - 1];
    check_slice.iter().all(|statement| {
        matches!(
            statement,
            IrStatement::Assignment(_)
                | IrStatement::Expression(_)
                | IrStatement::VariableDeclaration(_)
                | IrStatement::HighLevelWhile(_)
                | IrStatement::HighLevelDoWhile(_)
        )
    })
}

fn do_while_contains_break_target(body: &IrBlockContainer, break_target: u32) -> bool {
    for block in &body.blocks {
        for statement in &block.statements {
            if statement_contains_break_target(statement, break_target) {
                return true;
            }
        }
    }
    false
}

fn statement_contains_break_target(statement: &IrStatement, break_target: u32) -> bool {
    match statement {
        IrStatement::Leave(IrLeave {
            target_container_id,
        }) => *target_container_id == break_target,
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            statement_contains_break_target(true_statement, break_target)
                || false_statement
                    .as_ref()
                    .is_some_and(|x| statement_contains_break_target(x, break_target))
        }
        IrStatement::Block(block) => block
            .statements
            .iter()
            .any(|x| statement_contains_break_target(x, break_target)),
        IrStatement::BlockContainer(container) => container.blocks.iter().any(|block| {
            block
                .statements
                .iter()
                .any(|x| statement_contains_break_target(x, break_target))
        }),
        _ => false,
    }
}

fn try_render_guarded_infinite_while(
    out: &mut Vec<String>,
    while_stmt: &crate::decompiler::ir::IrHighLevelWhile,
    ctx: &DecompileContext,
    context: FlowRenderContext,
) -> bool {
    if while_stmt.condition.is_some() {
        return false;
    }
    if while_stmt.body.blocks.len() != 1 {
        return false;
    }

    let guard_block = &while_stmt.body.blocks[0];
    if guard_block.start_address != while_stmt.continue_target {
        return false;
    }
    if guard_block.statements.len() != 2 {
        return false;
    }

    let Some(IrStatement::If(guard_if)) = guard_block.statements.first() else {
        return false;
    };
    if guard_if.false_statement.is_some() {
        return false;
    }

    let IrStatement::Block(true_block) = guard_if.true_statement.as_ref() else {
        return false;
    };
    let mut loop_body_statements = true_block.statements.clone();
    if let Some(IrStatement::Jump(IrJump { target_address })) = loop_body_statements.last()
        && *target_address == while_stmt.continue_target
    {
        let _ = loop_body_statements.pop();
    }

    let exit_statement = guard_block.statements[1].clone();
    let skip_exit = matches!(
        exit_statement,
        IrStatement::Leave(IrLeave { target_container_id }) if target_container_id == while_stmt.break_target
    ) || context.has_following_statement;

    out.push(format!(
        "while ({})",
        render_expression(&guard_if.condition, ctx)
    ));
    out.push("{".to_string());
    let synthetic_body = IrBlockContainer {
        id: while_stmt.body.id,
        blocks: vec![IrBlock {
            statements: loop_body_statements,
            start_address: guard_block.start_address,
            should_emit_label: false,
        }],
        kind: IrContainerKind::Block,
        should_emit_exit_label: false,
    };
    append_container_with_context(
        out,
        &synthetic_body,
        ctx,
        Some(while_stmt.break_target),
        Some(while_stmt.continue_target),
    );
    out.push("}".to_string());

    if !skip_exit {
        append_statement_with_context(out, &exit_statement, ctx, context);
    }

    true
}

fn try_render_self_loop_while(
    out: &mut Vec<String>,
    block: &IrBlock,
    ctx: &DecompileContext,
    next_block: Option<u32>,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
    suppressed_label_addresses: &mut HashSet<u32>,
    _jump_target_counts: &HashMap<u32, usize>,
) -> bool {
    if block.statements.is_empty() {
        return false;
    }

    let Some(IrStatement::If(IrIf {
        condition,
        true_statement,
        false_statement,
    })) = block.statements.first()
    else {
        return false;
    };
    if false_statement.is_some() {
        return false;
    }
    let IrStatement::Block(true_block) = true_statement.as_ref() else {
        return false;
    };
    if true_block.statements.is_empty() {
        return false;
    }
    let Some(IrStatement::Jump(IrJump { target_address })) = true_block.statements.last() else {
        return false;
    };
    if *target_address != block.start_address {
        return false;
    }

    out.push(format!(
        "while ({})",
        render_expression(condition, ctx)
    ));
    out.push("{".to_string());
    let loop_break_target = block.statements.get(1).and_then(|stmt| match stmt {
        IrStatement::Jump(IrJump { target_address }) => Some(*target_address),
        _ => None,
    });
    if let Some(loop_break_target) = loop_break_target {
        suppressed_label_addresses.insert(loop_break_target);
    }
    for (index, statement) in true_block
        .statements
        .iter()
        .take(true_block.statements.len().saturating_sub(1))
        .enumerate()
    {
        let next_statement_is_return_like = true_block
            .statements
            .get(index + 1)
            .is_some_and(is_return_like_statement);
        append_statement_with_context(
            out,
            statement,
            ctx,
            FlowRenderContext {
                next_block: None,
                break_target_container,
                continue_target: Some(block.start_address).or(continue_target),
                loop_break_target,
                has_following_statement: index + 1 < true_block.statements.len(),
                next_statement_is_return_like,
            },
        );
    }
    out.push("}".to_string());

    let trailing = &block.statements[1..];
    for (index, statement) in trailing.iter().enumerate() {
        let statement_next = if index + 1 == trailing.len() {
            next_block
        } else {
            None
        };
        let next_statement_is_return_like = trailing
            .get(index + 1)
            .is_some_and(is_return_like_statement);
        append_statement_with_context(
            out,
            statement,
            ctx,
            FlowRenderContext {
                next_block: statement_next,
                break_target_container,
                continue_target,
                loop_break_target: None,
                has_following_statement: index + 1 < trailing.len(),
                next_statement_is_return_like,
            },
        );
    }
    true
}

fn collect_jump_target_counts_in_container(container: &IrBlockContainer) -> HashMap<u32, usize> {
    let mut counts = HashMap::<u32, usize>::new();
    for block in &container.blocks {
        for statement in &block.statements {
            collect_statement_jump_target_counts(statement, &mut counts);
        }
    }
    counts
}

fn collect_statement_jump_target_counts(statement: &IrStatement, counts: &mut HashMap<u32, usize>) {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => {
            *counts.entry(*target_address).or_insert(0) += 1;
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            collect_statement_jump_target_counts(true_statement, counts);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_statement_jump_target_counts(false_statement, counts);
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
                collect_statement_jump_target_counts(nested, counts);
            }
        }
        IrStatement::BlockContainer(container) => {
            for block in &container.blocks {
                for nested in &block.statements {
                    collect_statement_jump_target_counts(nested, counts);
                }
            }
        }
        IrStatement::HighLevelSwitch(switch_stmt) => {
            for section in &switch_stmt.sections {
                for nested in &section.body.statements {
                    collect_statement_jump_target_counts(nested, counts);
                }
            }
        }
        IrStatement::HighLevelWhile(while_stmt) => {
            for block in &while_stmt.body.blocks {
                for nested in &block.statements {
                    collect_statement_jump_target_counts(nested, counts);
                }
            }
        }
        IrStatement::HighLevelDoWhile(do_while_stmt) => {
            for block in &do_while_stmt.body.blocks {
                for nested in &block.statements {
                    collect_statement_jump_target_counts(nested, counts);
                }
            }
        }
        _ => {}
    }
}

fn condition_contains(expression: &IrExpression, needle: &IrExpression) -> bool {
    if expression == needle {
        return true;
    }

    if let IrExpression::OperatorCall(IrOperatorCallExpression {
        arguments,
        operator,
        ..
    }) = expression
        && *operator == IrOperator::LogicalAnd
    {
        return arguments
            .iter()
            .any(|argument| condition_contains(argument, needle));
    }

    false
}

fn render_expression(expression: &IrExpression, ctx: &DecompileContext) -> String {
    match expression {
        IrExpression::Literal(literal) => render_literal(literal),
        IrExpression::Variable(variable) => render_variable_expression(variable.address, ctx),
        IrExpression::InternalCall(call) => {
            let name = call
                .function_name
                .clone()
                .unwrap_or_else(|| format!("func_0x{:08X}", call.entry_address));
            format!("{}()", sanitize_identifier(name.as_str()))
        }
        IrExpression::ExternalCall(call) => render_external_call_expression(call, ctx),
        IrExpression::PropertyAccess(call) => render_property_access_expression(call, ctx),
        IrExpression::ConstructorCall(call) => {
            let args = call
                .arguments
                .iter()
                .map(|x| render_expression(x, ctx))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "new {}({args})",
                render_type_name(call.function_info.type_name.as_str())
            )
        }
        IrExpression::OperatorCall(IrOperatorCallExpression {
            arguments,
            operator,
        }) => render_operator_call(*operator, arguments, ctx),
    }
}

fn is_synthetic_block_address(address: u32) -> bool {
    (address & 0x8000_0000) != 0
}

fn render_literal(literal: &IrLiteralExpression) -> String {
    if literal.value.eq_ignore_ascii_case(UNSERIALIZABLE_LITERAL)
        || literal.value.eq_ignore_ascii_case(TYPE_UNSERIALIZABLE)
    {
        return UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL.to_string();
    }

    let Some(type_hint) = literal.type_hint.as_deref() else {
        return literal.value.clone();
    };
    render_literal_value_with_type(type_hint, literal.value.as_str())
}

fn render_literal_for_type(literal: &IrLiteralExpression, explicit_type_hint: &str) -> String {
    if literal.value.eq_ignore_ascii_case(UNSERIALIZABLE_LITERAL)
        || literal.value.eq_ignore_ascii_case(TYPE_UNSERIALIZABLE)
    {
        return UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL.to_string();
    }
    render_literal_value_with_type(explicit_type_hint, literal.value.as_str())
}

fn render_literal_value_with_type(type_hint: &str, value: &str) -> String {
    if let Ok(parsed) = parse_heap_literal(type_hint, value) {
        return render_heap_literal(type_hint, &parsed);
    }
    if value.trim().eq_ignore_ascii_case("null") {
        return "null".to_string();
    }
    UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL.to_string()
}

fn render_variable_declaration(
    variable: &VariableRecord,
    name: &str,
    init_value: Option<&str>,
    _exported: bool,
) -> String {
    let type_name = render_type_name(variable.type_name.as_str());
    if let Some(init_value) = init_value {
        format!("{type_name} {name} = {init_value};")
    } else {
        format!("{type_name} {name};")
    }
}

fn render_type_name(type_name: &str) -> String {
    let head = type_name.split(',').next().unwrap_or(type_name).trim();
    if head.eq_ignore_ascii_case(TYPE_UNSERIALIZABLE) {
        TYPE_SYSTEM_OBJECT.to_string()
    } else {
        normalize_csharp_type_name(head)
    }
}

fn render_local_variable_declaration(
    variable: &VariableRecord,
    name: &str,
    init_value: Option<&IrLiteralExpression>,
) -> String {
    let type_name = render_type_name(variable.type_name.as_str());
    let init = init_value
        .map(|x| render_literal_for_type(x, variable.type_name.as_str()))
        .or_else(|| default_local_initializer(variable.type_name.as_str()))
        .unwrap_or_else(|| "null".to_string());
    format!("{type_name} {name} = {init};")
}

fn default_local_initializer(type_name: &str) -> Option<String> {
    let head = type_name.split(',').next().unwrap_or(type_name).trim();
    match head {
        TYPE_SYSTEM_BOOLEAN => Some("false".to_string()),
        TYPE_SYSTEM_SBYTE | TYPE_SYSTEM_BYTE | TYPE_SYSTEM_INT16 | TYPE_SYSTEM_UINT16
        | TYPE_SYSTEM_INT32 | TYPE_SYSTEM_UINT32 | TYPE_SYSTEM_INT64 | TYPE_SYSTEM_UINT64 => {
            Some("0".to_string())
        }
        TYPE_SYSTEM_SINGLE => Some("0.0f".to_string()),
        TYPE_SYSTEM_DOUBLE => Some("0.0".to_string()),
        TYPE_SYSTEM_STRING => Some("null".to_string()),
        _ => None,
    }
}

fn render_external_call_expression(
    call: &IrExternalCallExpression,
    ctx: &DecompileContext,
) -> String {
    let args = call
        .arguments
        .iter()
        .map(|x| render_expression(x, ctx))
        .collect::<Vec<_>>();
    let method = render_external_member_name(
        call.function_info.function_name.as_str(),
        call.function_info.original_name.as_deref(),
    );

    if method.to_ascii_lowercase().contains("implicit") && args.len() == 1 {
        return args[0].clone();
    }

    if call.function_info.is_static || args.is_empty() {
        let rendered_args = render_call_args(call, &args, 0);
        let callable = format!(
            "{}.{}",
            render_type_name(call.function_info.type_name.as_str()),
            method
        );
        return format!("{callable}({rendered_args})");
    }

    let receiver = map_this_receiver(args[0].clone(), call.function_info.type_name.as_str());
    let rendered_args = render_call_args(call, &args[1..], 1);
    format!("{receiver}.{method}({rendered_args})")
}

fn render_property_access_expression(
    call: &IrPropertyAccessExpression,
    ctx: &DecompileContext,
) -> String {
    let args = call
        .arguments
        .iter()
        .map(|x| render_expression(x, ctx))
        .collect::<Vec<_>>();
    let property = render_property_name(
        call.function_info.function_name.as_str(),
        call.function_info.original_name.as_deref(),
    );

    let function_name = call.function_info.function_name.as_str();
    let is_setter = function_name.starts_with("set_") || function_name.starts_with("__set");

    if call.function_info.is_static || args.is_empty() {
        let owner = render_type_name(call.function_info.type_name.as_str());
        if is_setter && !args.is_empty() {
            return format!("{owner}.{property} = {}", args[0]);
        }
        return format!("{owner}.{property}");
    }

    let receiver = map_this_receiver(args[0].clone(), call.function_info.type_name.as_str());
    if is_setter {
        if args.len() >= 2 {
            return format!("{receiver}.{property} = {}", args[1]);
        }
        return format!("{receiver}.{property}");
    }
    format!("{receiver}.{property}")
}

fn render_external_member_name(function_name: &str, original_name: Option<&str>) -> String {
    let raw = original_name.unwrap_or(function_name);
    sanitize_identifier(raw)
}

fn render_property_name(function_name: &str, original_name: Option<&str>) -> String {
    let raw = original_name.unwrap_or(function_name);
    let stripped = raw
        .strip_prefix("get_")
        .or_else(|| raw.strip_prefix("set_"))
        .or_else(|| raw.strip_prefix("__get_"))
        .or_else(|| raw.strip_prefix("__set_"))
        .unwrap_or(raw);
    sanitize_identifier(stripped)
}

fn render_call_args(
    call: &IrExternalCallExpression,
    args: &[String],
    param_offset: usize,
) -> String {
    let mut out = Vec::<String>::with_capacity(args.len());
    for (idx, arg) in args.iter().enumerate() {
        let rendered = match call.function_info.parameters.get(param_offset + idx) {
            Some(ParameterType::Out) => format!("out {arg}"),
            Some(ParameterType::InOut) => format!("ref {arg}"),
            _ => arg.clone(),
        };
        out.push(rendered);
    }
    out.join(", ")
}

fn map_this_receiver(receiver: String, declaring_type: &str) -> String {
    if receiver != "this" {
        return receiver;
    }
    let head = declaring_type
        .split(',')
        .next()
        .unwrap_or(declaring_type)
        .trim();
    match head {
        TYPE_UNITY_GAME_OBJECT => "this.gameObject".to_string(),
        TYPE_UNITY_TRANSFORM => "this.transform".to_string(),
        _ => receiver,
    }
}

fn render_operator_call(
    operator: IrOperator,
    arguments: &[IrExpression],
    ctx: &DecompileContext,
) -> String {
    if let Some(rendered) = render_known_operator(operator, arguments, ctx) {
        return rendered;
    }
    panic!("unsupported operator rendering: {:?}", operator)
}

fn render_known_operator(
    operator: IrOperator,
    arguments: &[IrExpression],
    ctx: &DecompileContext,
) -> Option<String> {
    if operator == IrOperator::Conversion {
        if arguments.len() != 1 {
            return None;
        }
        return Some(render_operator_operand(
            operator,
            &arguments[0],
            0,
            ctx,
        ));
    }

    if operator.is_unary() {
        if arguments.len() != 1 {
            return None;
        }
        let value = render_operator_operand(operator, &arguments[0], 0, ctx);
        let prefix = match operator {
            IrOperator::UnaryMinus => "-",
            IrOperator::UnaryNegation => "!",
            _ => return None,
        };
        return Some(format!("{prefix}{value}"));
    }

    if arguments.len() != 2 {
        return None;
    }

    let lhs = render_operator_operand(operator, &arguments[0], 0, ctx);
    let rhs = render_operator_operand(operator, &arguments[1], 1, ctx);
    let symbol = binary_operator_symbol(operator)?;
    Some(format!("{lhs} {symbol} {rhs}"))
}

fn binary_operator_symbol(operator: IrOperator) -> Option<&'static str> {
    match operator {
        IrOperator::Addition => Some("+"),
        IrOperator::Subtraction => Some("-"),
        IrOperator::Multiplication => Some("*"),
        IrOperator::Division => Some("/"),
        IrOperator::Remainder => Some("%"),
        IrOperator::ConditionalAnd | IrOperator::LogicalAnd => Some("&&"),
        IrOperator::ConditionalOr | IrOperator::LogicalOr => Some("||"),
        IrOperator::ConditionalXor | IrOperator::LogicalXor => Some("^"),
        IrOperator::LeftShift => Some("<<"),
        IrOperator::RightShift => Some(">>"),
        IrOperator::Equality => Some("=="),
        IrOperator::Inequality => Some("!="),
        IrOperator::GreaterThan => Some(">"),
        IrOperator::GreaterThanOrEqual => Some(">="),
        IrOperator::LessThan => Some("<"),
        IrOperator::LessThanOrEqual => Some("<="),
        IrOperator::UnaryMinus | IrOperator::UnaryNegation | IrOperator::Conversion => None,
    }
}

fn render_operator_operand(
    parent_op: IrOperator,
    operand: &IrExpression,
    operand_index: usize,
    ctx: &DecompileContext,
) -> String {
    let operand_str = render_expression(operand, ctx);
    if operator_needs_parentheses(parent_op, operand, operand_index) {
        return format!("({operand_str})");
    }
    operand_str
}

fn operator_needs_parentheses(
    parent_op: IrOperator,
    child_expression: &IrExpression,
    operand_index: usize,
) -> bool {
    let IrExpression::OperatorCall(IrOperatorCallExpression { operator, .. }) = child_expression
    else {
        return false;
    };
    let child_op = *operator;

    let parent_prec = parent_op.precedence();
    let child_prec = child_op.precedence();

    if child_prec < parent_prec {
        return true;
    }
    if child_prec > parent_prec {
        return false;
    }
    if parent_op.is_unary() {
        return true;
    }
    if operand_index == 1 {
        if child_op != parent_op {
            return true;
        }
        return !parent_op.is_associative();
    }
    false
}

fn normalize_csharp_type_name(type_name: &str) -> String {
    type_name.replace('+', ".")
}

fn require_variable(ctx: &DecompileContext, address: u32) -> &VariableRecord {
    ctx.variables
        .get_by_address(address)
        .unwrap_or_else(|| panic!("missing variable at address 0x{address:08X} during codegen"))
}

fn sanitize_identifier(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        let valid = ch.is_ascii_alphanumeric() || ch == '_';
        if index == 0 {
            if valid && !ch.is_ascii_digit() {
                out.push(ch);
            } else if valid {
                out.push('_');
                out.push(ch);
            } else {
                out.push('_');
            }
        } else if valid {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    out
}

fn variable_name_used_in_body(name: &str, body_text: &str) -> bool {
    if name.is_empty() || body_text.is_empty() {
        return false;
    }

    for line in body_text.lines() {
        let mut token = String::new();
        for ch in line.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                token.push(ch);
            } else if !token.is_empty() {
                if token == name {
                    return true;
                }
                token.clear();
            }
        }
        if token == name {
            return true;
        }
    }

    false
}

fn render_variable_expression(address: u32, ctx: &DecompileContext) -> String {
    ctx.variables
        .get_by_address(address)
        .map(|variable| variable.name.clone())
        .unwrap_or_else(|| generated_heap_symbol(address))
}
