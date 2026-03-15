use std::collections::HashMap;

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrAssignmentStatement, IrExpression, IrExpressionStatement, IrFunction, IrIf,
    IrLiteralExpression, IrStatement, IrSwitch, IrVariableExpression,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};
use crate::decompiler::VariableRecord;
use crate::str_constants::{
    TYPE_SYSTEM_BOOLEAN, TYPE_SYSTEM_DOUBLE, TYPE_SYSTEM_INT32, TYPE_SYSTEM_INT64,
    TYPE_SYSTEM_SINGLE, TYPE_SYSTEM_STRING, TYPE_SYSTEM_UINT32, TYPE_SYSTEM_UINT64,
    UNSERIALIZABLE_LITERAL,
};

pub struct ConstToLiteral;

impl ITransform for ConstToLiteral {

    fn run(&self, function: &mut IrFunction, _context: &mut TransformContext<'_, '_>) -> Result<()> {
        let variables_by_address = _context
            .program_context
            .decompile_context
            .variables
            .variables
            .iter()
            .map(|variable| (variable.address, variable))
            .collect::<HashMap<_, _>>();
        for block in &mut function.body.blocks {
            for statement in &mut block.statements {
                rewrite_statement(statement, &variables_by_address);
            }
        }
        Ok(())
    }
}

fn rewrite_statement(
    statement: &mut IrStatement,
    variables_by_address: &HashMap<u32, &VariableRecord>,
) {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, value }) => {
            rewrite_assignment_target(target, variables_by_address);
            rewrite_expression(value, variables_by_address);
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            rewrite_expression(expression, variables_by_address);
        }
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            rewrite_expression(condition, variables_by_address);
            rewrite_statement(true_statement, variables_by_address);
            if let Some(false_statement) = false_statement.as_mut() {
                rewrite_statement(false_statement, variables_by_address);
            }
        }
        IrStatement::Switch(IrSwitch {
            index_expression, ..
        }) => {
            rewrite_expression(index_expression, variables_by_address);
        }
        IrStatement::Block(block) => {
            for statement in &mut block.statements {
                rewrite_statement(statement, variables_by_address);
            }
        }
        IrStatement::BlockContainer(container) => {
            for block in &mut container.blocks {
                for statement in &mut block.statements {
                    rewrite_statement(statement, variables_by_address);
                }
            }
        }
        IrStatement::HighLevelSwitch(switch_stmt) => {
            rewrite_expression(&mut switch_stmt.index_expression, variables_by_address);
            for section in &mut switch_stmt.sections {
                for statement in &mut section.body.statements {
                    rewrite_statement(statement, variables_by_address);
                }
            }
        }
        IrStatement::HighLevelWhile(while_stmt) => {
            if let Some(condition) = while_stmt.condition.as_mut() {
                rewrite_expression(condition, variables_by_address);
            }
            for block in &mut while_stmt.body.blocks {
                for statement in &mut block.statements {
                    rewrite_statement(statement, variables_by_address);
                }
            }
        }
        IrStatement::HighLevelDoWhile(do_while_stmt) => {
            rewrite_expression(&mut do_while_stmt.condition, variables_by_address);
            for block in &mut do_while_stmt.body.blocks {
                for statement in &mut block.statements {
                    rewrite_statement(statement, variables_by_address);
                }
            }
        }
        _ => {}
    }
}

fn rewrite_assignment_target(
    target: &mut IrExpression,
    variables_by_address: &HashMap<u32, &VariableRecord>,
) {
    if let IrExpression::PropertyAccess(call) = target {
        for arg in &mut call.arguments {
            rewrite_expression(arg, variables_by_address);
        }
    }
}

fn rewrite_expression(
    expression: &mut IrExpression,
    variables_by_address: &HashMap<u32, &VariableRecord>,
) {
    match expression {
        IrExpression::Variable(IrVariableExpression { address }) => {
            let Some(variable) = variables_by_address.get(address) else {
                return;
            };
            if !variable.name.starts_with("__const_") {
                return;
            }
            if !variable.is_init_serializable {
                return;
            }
            if let Some(literal) =
                parse_literal(variable.type_name.as_str(), variable.init_value.as_str())
            {
                *expression = IrExpression::Literal(literal);
            }
        }
        IrExpression::ExternalCall(call) => {
            for arg in &mut call.arguments {
                rewrite_expression(arg, variables_by_address);
            }
        }
        IrExpression::PropertyAccess(call) => {
            for arg in &mut call.arguments {
                rewrite_expression(arg, variables_by_address);
            }
        }
        IrExpression::ConstructorCall(call) => {
            for arg in &mut call.arguments {
                rewrite_expression(arg, variables_by_address);
            }
        }
        IrExpression::OperatorCall(call) => {
            for arg in &mut call.arguments {
                rewrite_expression(arg, variables_by_address);
            }
        }
        _ => {}
    }
}

fn parse_literal(type_name: &str, init_value: &str) -> Option<IrLiteralExpression> {
    if init_value.eq_ignore_ascii_case(UNSERIALIZABLE_LITERAL) {
        return None;
    }
    let head = type_name.split(',').next().unwrap_or(type_name).trim();

    match head {
        TYPE_SYSTEM_BOOLEAN => match init_value {
            "true" | "false" => Some(IrLiteralExpression {
                value: init_value.to_string(),
                type_hint: type_name.to_string(),
            }),
            _ => None,
        },
        TYPE_SYSTEM_INT32 | TYPE_SYSTEM_UINT32 | TYPE_SYSTEM_INT64 | TYPE_SYSTEM_UINT64
        | TYPE_SYSTEM_SINGLE | TYPE_SYSTEM_DOUBLE => {
            if init_value.trim().is_empty() {
                return None;
            }
            Some(IrLiteralExpression {
                value: init_value.to_string(),
                type_hint: type_name.to_string(),
            })
        }
        TYPE_SYSTEM_STRING => Some(IrLiteralExpression {
            value: init_value.to_string(),
            type_hint: type_name.to_string(),
        }),
        _ => {
            if init_value.trim().is_empty() {
                return None;
            }
            Some(IrLiteralExpression {
                value: init_value.to_string(),
                type_hint: type_name.to_string(),
            })
        }
    }
}
