use std::collections::{BTreeSet, HashMap};

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrAssignmentStatement, IrExpression, IrExpressionStatement, IrFunction, IrIf, IrStatement,
    IrSwitch, IrVariableDeclarationStatement,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};
use crate::str_constants::{
    SYMBOL_PREFIX_THIS, SYMBOL_THIS, SYMBOL_THIS_GAME_OBJECT, SYMBOL_THIS_TRANSFORM,
};

pub struct CollectVariables;

impl ITransform for CollectVariables {
    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let mut used = BTreeSet::<u32>::new();
        for block in &function.body.blocks {
            for statement in &block.statements {
                collect_statement_variables(statement, &mut used);
            }
        }

        let all_vars = context
            .program_context
            .decompile_context
            .variables
            .variables
            .iter()
            .map(|x| (x.address, x))
            .collect::<HashMap<_, _>>();

        let mut declarations = Vec::<IrVariableDeclarationStatement>::new();
        for address in used {
            let Some(variable) = all_vars.get(&address) else {
                continue;
            };

            if variable.exported || is_this_like(variable.name.as_str()) {
                continue;
            }

            let init_value = Some(crate::decompiler::ir::IrLiteralExpression {
                value: variable.init_value.clone(),
                type_hint: variable.type_name.clone(),
            });

            declarations.push(IrVariableDeclarationStatement {
                variable_address: variable.address,
                init_value,
            });
        }

        function.variable_declarations = declarations;
        Ok(())
    }
}

fn collect_statement_variables(statement: &IrStatement, used: &mut BTreeSet<u32>) {
    match statement {
        IrStatement::Assignment(IrAssignmentStatement { target, value }) => {
            collect_assignment_target_variables(target, used);
            collect_expression_variables(value, used);
        }
        IrStatement::Expression(IrExpressionStatement { expression }) => {
            collect_expression_variables(expression, used);
        }
        IrStatement::If(IrIf {
            condition,
            true_statement,
            false_statement,
        }) => {
            collect_expression_variables(condition, used);
            collect_statement_variables(true_statement, used);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_statement_variables(false_statement, used);
            }
        }
        IrStatement::Switch(IrSwitch {
            index_expression, ..
        }) => {
            collect_expression_variables(index_expression, used);
        }
        IrStatement::Block(block) => {
            for statement in &block.statements {
                collect_statement_variables(statement, used);
            }
        }
        IrStatement::BlockContainer(container) => {
            for block in &container.blocks {
                for statement in &block.statements {
                    collect_statement_variables(statement, used);
                }
            }
        }
        IrStatement::HighLevelSwitch(switch_stmt) => {
            collect_expression_variables(&switch_stmt.index_expression, used);
            for section in &switch_stmt.sections {
                for statement in &section.body.statements {
                    collect_statement_variables(statement, used);
                }
            }
        }
        IrStatement::HighLevelWhile(while_stmt) => {
            if let Some(condition) = while_stmt.condition.as_ref() {
                collect_expression_variables(condition, used);
            }
            for block in &while_stmt.body.blocks {
                for statement in &block.statements {
                    collect_statement_variables(statement, used);
                }
            }
        }
        IrStatement::HighLevelDoWhile(do_while_stmt) => {
            collect_expression_variables(&do_while_stmt.condition, used);
            for block in &do_while_stmt.body.blocks {
                for statement in &block.statements {
                    collect_statement_variables(statement, used);
                }
            }
        }
        _ => {}
    }
}

fn collect_assignment_target_variables(target: &IrExpression, used: &mut BTreeSet<u32>) {
    match target {
        IrExpression::Variable(variable_expression) => {
            used.insert(variable_expression.address);
        }
        IrExpression::PropertyAccess(property_access) => {
            for arg in &property_access.arguments {
                collect_expression_variables(arg, used);
            }
        }
        IrExpression::ArrayAccess(array_access) => {
            for arg in &array_access.arguments {
                collect_expression_variables(arg, used);
            }
        }
        _ => {}
    }
}

fn collect_expression_variables(expression: &IrExpression, used: &mut BTreeSet<u32>) {
    match expression {
        IrExpression::Variable(variable_expression) => {
            used.insert(variable_expression.address);
        }
        IrExpression::ExternalCall(call) => {
            for arg in &call.arguments {
                collect_expression_variables(arg, used);
            }
        }
        IrExpression::PropertyAccess(call) => {
            for arg in &call.arguments {
                collect_expression_variables(arg, used);
            }
        }
        IrExpression::ArrayAccess(call) => {
            for arg in &call.arguments {
                collect_expression_variables(arg, used);
            }
        }
        IrExpression::ConstructorCall(call) => {
            for arg in &call.arguments {
                collect_expression_variables(arg, used);
            }
        }
        IrExpression::OperatorCall(call) => {
            for arg in &call.arguments {
                collect_expression_variables(arg, used);
            }
        }
        _ => {}
    }
}

fn is_this_like(name: &str) -> bool {
    name == SYMBOL_THIS
        || name == SYMBOL_THIS_TRANSFORM
        || name == SYMBOL_THIS_GAME_OBJECT
        || name.starts_with(SYMBOL_PREFIX_THIS)
}
