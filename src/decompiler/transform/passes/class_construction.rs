use crate::decompiler::Result;
use crate::decompiler::ir::{IrClass, IrFunction, IrVariableDeclarationStatement};
use crate::decompiler::transform::pass_base::{IProgramTransform, ProgramTransformContext};

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

        let mut class_variables = Vec::<IrVariableDeclarationStatement>::new();
        for variable in &context.decompile_context.variables.variables {
            if !variable.exported {
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
