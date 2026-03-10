use std::collections::HashSet;

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrAssignmentStatement, IrClass, IrExpression, IrExpressionStatement, IrFunction, IrIf,
    IrStatement, IrSwitch, IrVariableDeclarationStatement,
};
use crate::decompiler::transform::pass_base::{IProgramTransform, ProgramTransformContext};
use crate::decompiler::variable::VariableScope;

pub struct IrClassConstructionTransform;

impl IProgramTransform for IrClassConstructionTransform {

    fn run(
        &self,
        functions: &mut [IrFunction],
        context: &mut ProgramTransformContext,
    ) -> Result<()> {
        let class_name = context.class_name.clone();
        let mut resolved_class_name = class_name.clone();
        let mut namespace = None::<String>;
        if let Some((ns, name)) = class_name.rsplit_once('.')
            && !name.is_empty()
        {
            resolved_class_name = name.to_string();
            if !ns.is_empty() {
                namespace = Some(ns.to_string());
            }
        }

        let used_addresses = collect_used_addresses(functions);
        let mut class_variables = Vec::<IrVariableDeclarationStatement>::new();
        for variable in &context.decompile_context.variables.variables {
            if variable.scope != VariableScope::Global
                && !variable.exported
                && !variable.name.starts_with("__const_")
            {
                continue;
            }
            if !variable.exported && !used_addresses.contains(&variable.address) {
                continue;
            }

            let init_value = Some(crate::decompiler::ir::IrLiteralExpression {
                value: variable.init_value.clone(),
                type_hint: Some(variable.type_name.clone()),
            });

            class_variables.push(IrVariableDeclarationStatement {
                variable_address: variable.address,
                init_value,
            });
        }

        context.ir_class = Some(IrClass {
            class_name: resolved_class_name,
            namespace,
            variable_declarations: class_variables,
            functions: functions.to_vec(),
        });
        Ok(())
    }
}

fn collect_used_addresses(functions: &[IrFunction]) -> HashSet<u32> {
    let mut out = HashSet::<u32>::new();
    for function in functions {
        for block in &function.body.blocks {
            for statement in &block.statements {
                collect_statement_addresses(statement, &mut out);
            }
        }
    }
    out
}

fn collect_statement_addresses(statement: &IrStatement, out: &mut HashSet<u32>) {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement {
            target_address,
            value,
        }) => {
            out.insert(*target_address);
            collect_expression_addresses(value, out);
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            collect_expression_addresses(expression, out);
        }
        IrStatement::VariableDeclaration(decl) => {
            out.insert(decl.variable_address);
        }
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            collect_expression_addresses(condition, out);
            collect_statement_addresses(true_statement, out);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_statement_addresses(false_statement, out);
            }
        }
        IrStatement::Switch(IrSwitch {
            index_expression, ..
        }) => {
            collect_expression_addresses(index_expression, out);
        }
        IrStatement::Block(block) => {
            for nested in &block.statements {
                collect_statement_addresses(nested, out);
            }
        }
        IrStatement::BlockContainer(container) => {
            for block in &container.blocks {
                for nested in &block.statements {
                    collect_statement_addresses(nested, out);
                }
            }
        }
        IrStatement::HighLevelSwitch(switch_stmt) => {
            collect_expression_addresses(&switch_stmt.index_expression, out);
            for section in &switch_stmt.sections {
                for nested in &section.body.statements {
                    collect_statement_addresses(nested, out);
                }
            }
        }
        IrStatement::HighLevelWhile(while_stmt) => {
            if let Some(condition) = while_stmt.condition.as_ref() {
                collect_expression_addresses(condition, out);
            }
            for block in &while_stmt.body.blocks {
                for nested in &block.statements {
                    collect_statement_addresses(nested, out);
                }
            }
        }
        IrStatement::HighLevelDoWhile(do_while_stmt) => {
            collect_expression_addresses(&do_while_stmt.condition, out);
            for block in &do_while_stmt.body.blocks {
                for nested in &block.statements {
                    collect_statement_addresses(nested, out);
                }
            }
        }
        _ => {}
    }
}

fn collect_expression_addresses(expression: &IrExpression, out: &mut HashSet<u32>) {
    match expression {
        IrExpression::Variable(variable) => {
            out.insert(variable.address);
        }
        IrExpression::ExternalCall(call) => {
            for arg in &call.arguments {
                collect_expression_addresses(arg, out);
            }
        }
        IrExpression::PropertyAccess(call) => {
            for arg in &call.arguments {
                collect_expression_addresses(arg, out);
            }
        }
        IrExpression::ConstructorCall(call) => {
            for arg in &call.arguments {
                collect_expression_addresses(arg, out);
            }
        }
        IrExpression::OperatorCall(call) => {
            for arg in &call.arguments {
                collect_expression_addresses(arg, out);
            }
        }
        _ => {}
    }
}
