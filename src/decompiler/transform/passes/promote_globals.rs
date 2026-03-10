use std::collections::{HashMap, HashSet};

use crate::decompiler::Result;
use crate::decompiler::ir::{IrFunction, IrVariableDeclarationStatement};
use crate::decompiler::transform::pass_base::{IProgramTransform, ProgramTransformContext};
use crate::decompiler::variable::VariableScope;

pub struct PromoteGlobals;

impl IProgramTransform for PromoteGlobals {

    fn run(
        &self,
        functions: &mut [IrFunction],
        context: &mut ProgramTransformContext,
    ) -> Result<()> {
        let Some(class_ir) = context.ir_class.as_mut() else {
            return Ok(());
        };

        let mut class_address_to_declaration = class_ir
            .variable_declarations
            .iter()
            .map(|x| (x.variable_address, x.clone()))
            .collect::<HashMap<_, _>>();

        let mut address_to_functions = HashMap::<u32, HashSet<usize>>::new();
        let mut address_to_declaration = HashMap::<u32, IrVariableDeclarationStatement>::new();
        let mut forced_global_addresses = HashSet::<u32>::new();
        let variables_by_address = context
            .decompile_context
            .variables
            .variables
            .iter()
            .map(|x| (x.address, x))
            .collect::<HashMap<_, _>>();

        for (function_index, function) in functions.iter().enumerate() {
            for declaration in &function.variable_declarations {
                let address = declaration.variable_address;
                address_to_functions
                    .entry(address)
                    .or_default()
                    .insert(function_index);
                if variables_by_address
                    .get(&address)
                    .is_some_and(|variable| variable.scope == VariableScope::Global)
                {
                    forced_global_addresses.insert(address);
                }
                let existing = address_to_declaration.get(&address);
                if existing.is_none()
                    || (existing.is_some_and(|x| x.init_value.is_none())
                        && declaration.init_value.is_some())
                {
                    address_to_declaration.insert(address, declaration.clone());
                }
            }
        }

        let mut promoted_addresses = address_to_functions
            .iter()
            .filter_map(|(address, owners)| (owners.len() >= 2).then_some(*address))
            .collect::<HashSet<_>>();
        promoted_addresses.extend(forced_global_addresses);

        for function in functions.iter_mut() {
            function
                .variable_declarations
                .retain(|x| !promoted_addresses.contains(&x.variable_address));
        }

        let mut promoted_sorted = promoted_addresses.into_iter().collect::<Vec<_>>();
        promoted_sorted.sort_unstable();

        for address in promoted_sorted {
            if let Some(candidate) = address_to_declaration.get(&address).cloned() {
                let keep_existing = class_address_to_declaration
                    .get(&address)
                    .is_some_and(|x| x.init_value.is_some() && candidate.init_value.is_none());
                if !keep_existing {
                    class_address_to_declaration.insert(address, candidate);
                }
            }
        }

        let mut merged_addresses = class_address_to_declaration
            .keys()
            .copied()
            .collect::<Vec<_>>();
        merged_addresses.sort_unstable();
        class_ir.variable_declarations = merged_addresses
            .into_iter()
            .filter_map(|x| class_address_to_declaration.get(&x).cloned())
            .collect();

        Ok(())
    }
}
