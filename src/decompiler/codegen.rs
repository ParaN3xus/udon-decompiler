use tracing::{debug, info};

use crate::decompiler::clang_format::format_csharp;
use crate::decompiler::context::DecompileContext;
use crate::decompiler::ir::{
    IrAssignmentStatement, IrBlock, IrBlockContainer, IrClass, IrExpression, IrExpressionStatement,
    IrExternalCallExpression, IrFunction, IrHighLevelSwitch, IrIf, IrJump, IrLeave,
    IrLiteralExpression, IrOperator, IrOperatorCallExpression, IrPropertyAccessExpression,
    IrStatement, IrSwitch,
};
use crate::decompiler::{ParameterType, Result, VariableRecord};
use crate::str_constants::{
    TYPE_SYSTEM_BOOLEAN, TYPE_SYSTEM_BYTE, TYPE_SYSTEM_DOUBLE, TYPE_SYSTEM_INT16,
    TYPE_SYSTEM_INT32, TYPE_SYSTEM_INT64, TYPE_SYSTEM_OBJECT, TYPE_SYSTEM_SBYTE,
    TYPE_SYSTEM_SINGLE, TYPE_SYSTEM_STRING, TYPE_SYSTEM_UINT16, TYPE_SYSTEM_UINT32,
    TYPE_SYSTEM_UINT64, TYPE_UNSERIALIZABLE, UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL,
    UNSERIALIZABLE_LITERAL,
};
use crate::udon_asm::generated_heap_symbol;

pub fn generate_csharp(ctx: &DecompileContext, class_ir: &IrClass) -> Result<String> {
    debug!("generating C# code for {}...", class_ir.class_name);

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
                .map(render_literal)
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

    info!("c# code for {} generated!", class_ir.class_name);

    format_csharp(
        out.join("\n").as_str(),
        ctx.clang_format_override.as_deref(),
    )
}
fn append_function(out: &mut Vec<String>, function: &IrFunction, ctx: &DecompileContext) {
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

    let local_declarations = function
        .variable_declarations
        .iter()
        .map(|declaration| {
            let variable = require_variable(ctx, declaration.variable_address);
            let name = variable.name.as_str();
            render_local_variable_declaration(variable, name, declaration.init_value.as_ref())
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

fn append_function_body(out: &mut Vec<String>, function: &IrFunction, ctx: &DecompileContext) {
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
        if block.should_emit_label {
            out.push(format!("{}:", render_block_label(block.start_address)));
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

    for (index, statement) in block.statements.iter().enumerate() {
        let statement_next = if index + 1 == block.statements.len() {
            next_block
        } else {
            None
        };
        append_statement_with_context(
            out,
            statement,
            ctx,
            FlowRenderContext {
                next_block: statement_next,
                break_target_container: None,
                continue_target: None,
            },
        );
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct FlowRenderContext {
    next_block: Option<u32>,
    break_target_container: Option<u32>,
    continue_target: Option<u32>,
}

fn append_statement_with_context(
    out: &mut Vec<String>,
    statement: &IrStatement,
    ctx: &DecompileContext,
    context: FlowRenderContext,
) {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, value }) => {
            out.push(format!(
                "{} = {};",
                render_assignment_target(target, ctx),
                render_expression(value, ctx)
            ));
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            out.push(format!("{};", render_expression(expression, ctx)));
        }
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            out.push(format!("if ({})", render_expression(condition, ctx)));
            out.push("{".to_string());
            append_statement_with_context(
                out,
                true_statement,
                ctx,
                FlowRenderContext {
                    next_block: None,
                    break_target_container: context.break_target_container,
                    continue_target: context.continue_target,
                },
            );
            out.push("}".to_string());
            if let Some(false_statement) = false_statement {
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
                .next_block
                .is_some_and(|next| next == *target_address)
            {
            } else {
                out.push(format!("goto {};", render_block_label(*target_address)));
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
                out.push(format!(
                    "case {case_value}: goto {};",
                    render_block_label(*target)
                ));
            }
            if let Some(default_target) = default_target {
                out.push(format!(
                    "default: goto {};",
                    render_block_label(*default_target)
                ));
            } else {
                out.push("default: break;".to_string());
            }
            out.push("}".to_string());
        }
        IrStatement::Block(nested_block) => {
            for nested in &nested_block.statements {
                append_statement_with_context(
                    out,
                    nested,
                    ctx,
                    FlowRenderContext {
                        next_block: None,
                        break_target_container: context.break_target_container,
                        continue_target: context.continue_target,
                    },
                );
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
    for (index, block) in container.blocks.iter().enumerate() {
        if block.should_emit_label {
            out.push(format!("{}:", render_block_label(block.start_address)));
        }

        for (statement_index, statement) in block.statements.iter().enumerate() {
            let next_block = (statement_index + 1 == block.statements.len())
                .then_some(index + 1)
                .and_then(|next_index| container.blocks.get(next_index))
                .map(|next| next.start_address);
            append_statement_with_context(
                out,
                statement,
                ctx,
                FlowRenderContext {
                    next_block,
                    break_target_container,
                    continue_target,
                },
            );
        }
    }
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
        for statement in &section.body.statements {
            append_statement_with_context(
                out,
                statement,
                ctx,
                FlowRenderContext {
                    next_block: None,
                    break_target_container: None,
                    continue_target: None,
                },
            );
        }
        if switch_section_needs_break_slice(&section.body.statements) {
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

pub(crate) fn render_expression(expression: &IrExpression, ctx: &DecompileContext) -> String {
    match expression {
        IrExpression::Literal(literal) => render_literal(literal),
        IrExpression::Variable(variable) => render_variable_expression(variable.address, ctx),
        IrExpression::Raw(raw) => raw.value.clone(),
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
        IrExpression::OperatorCall(call) => render_operator_call(call, ctx),
    }
}

fn render_block_label(address: u32) -> String {
    format!("bb_{address:08x}")
}

fn render_literal(literal: &IrLiteralExpression) -> String {
    if literal.value == UNSERIALIZABLE_LITERAL {
        return UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL.to_string();
    }
    literal.value.clone()
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
    if head == TYPE_UNSERIALIZABLE {
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
        .map(render_literal)
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
    let method = call.function_info.original_name.clone().unwrap_or_else(|| {
        panic!(
            "Invalid function {}! originalName expected!",
            call.function_info.signature
        )
    });

    if call.function_info.is_static {
        let rendered_args = render_call_args(call, &args, 0);
        let callable = format!(
            "{}.{}",
            render_type_name(call.function_info.type_name.as_str()),
            method
        );
        return format!("{callable}({rendered_args})");
    }

    let receiver = args[0].clone();
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
    let property = call.function_info.original_name.clone().unwrap_or_else(|| {
        panic!(
            "Invalid function {}! originalName expected!",
            call.function_info.signature
        )
    });
    if call.function_info.is_static || args.is_empty() {
        let owner = render_type_name(call.function_info.type_name.as_str());
        return format!("{owner}.{property}");
    }

    let receiver = args[0].clone();
    format!("{receiver}.{property}")
}

fn render_assignment_target(target: &IrExpression, ctx: &DecompileContext) -> String {
    match target {
        IrExpression::Variable(variable) => render_variable_expression(variable.address, ctx),
        IrExpression::PropertyAccess(call) => render_property_access_expression(call, ctx),
        _ => panic!("unsupported assignment target: {target:?}"),
    }
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

fn render_operator_call(call: &IrOperatorCallExpression, ctx: &DecompileContext) -> String {
    if let Some(rendered) = render_known_operator(call, ctx) {
        return rendered;
    }
    panic!("unsupported operator rendering: {:?}", call.operator)
}

fn render_known_operator(
    call: &IrOperatorCallExpression,
    ctx: &DecompileContext,
) -> Option<String> {
    let expected_arity = match call.operator {
        IrOperator::ExplicitConversion => 2,
        _ if call.operator.is_unary() => 1,
        _ => 2,
    };
    if call.arguments.len() != expected_arity {
        return None;
    }

    let mut rendered = call.operator.formatter()?.to_string();
    for (index, argument) in call.arguments.iter().enumerate() {
        let operand = render_operator_operand(call.operator, argument, index, ctx);
        rendered = rendered.replacen("{}", operand.as_str(), 1);
    }

    Some(rendered)
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

pub(crate) fn render_variable_expression(address: u32, ctx: &DecompileContext) -> String {
    ctx.variables
        .get_by_address(address)
        .map(|variable| variable.name.clone())
        .unwrap_or_else(|| generated_heap_symbol(address))
}
