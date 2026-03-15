use std::collections::HashMap;

use crate::str_constants::{
    SYMBOL_PREFIX_CONST, SYMBOL_PREFIX_GINTNL, SYMBOL_PREFIX_GLOBAL, SYMBOL_PREFIX_INTNL,
    SYMBOL_PREFIX_LCL, SYMBOL_PREFIX_THIS, SYMBOL_THIS, SYMBOL_THIS_GAME_OBJECT,
    SYMBOL_THIS_TRANSFORM,
};
use crate::udon_asm::generated_heap_symbol;

use super::context::{DecompileHeapEntry, DecompileSymbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableScope {
    Global,
    Local,
    Temporary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableRecord {
    pub address: u32,
    pub name: String,
    pub type_name: String,
    pub init_value: String,
    pub exported: bool,
    pub scope: VariableScope,
}

#[derive(Debug, Clone, Default)]
pub struct VariableTable {
    pub variables: Vec<VariableRecord>,
    by_address: HashMap<u32, usize>,
}

impl VariableTable {
    pub fn identify_from_heap(
        heap_entries: &[DecompileHeapEntry],
        symbols: &[DecompileSymbol],
    ) -> Self {
        let symbol_by_address = symbols
            .iter()
            .map(|x| (x.address, x))
            .collect::<HashMap<_, _>>();

        let mut out = Self::default();
        for heap in heap_entries {
            let (name, exported, scope) = if let Some(sym) = symbol_by_address.get(&heap.address) {
                let resolved_name =
                    resolve_this_symbol(sym.name.as_str()).unwrap_or_else(|| sym.name.clone());
                (
                    resolved_name,
                    sym.exported,
                    classify_symbol_scope(sym.name.as_str()),
                )
            } else {
                (
                    generated_heap_symbol(heap.address),
                    false,
                    VariableScope::Temporary,
                )
            };

            let record = VariableRecord {
                address: heap.address,
                name,
                type_name: heap.type_name.clone(),
                init_value: heap.init_value.clone(),
                exported,
                scope,
            };
            out.by_address.insert(heap.address, out.variables.len());
            out.variables.push(record);
        }
        out
    }

    pub fn get_by_address(&self, address: u32) -> Option<&VariableRecord> {
        self.by_address
            .get(&address)
            .and_then(|idx| self.variables.get(*idx))
    }

    pub fn get_mut_by_address(&mut self, address: u32) -> Option<&mut VariableRecord> {
        self.by_address
            .get(&address)
            .copied()
            .and_then(|idx| self.variables.get_mut(idx))
    }

    pub fn symbol_name_by_address_map(&self) -> HashMap<u32, String> {
        self.variables
            .iter()
            .map(|x| (x.address, x.name.clone()))
            .collect()
    }

    pub fn symbol_type_by_address_map(&self) -> HashMap<u32, String> {
        self.variables
            .iter()
            .map(|x| (x.address, x.type_name.clone()))
            .collect()
    }
}

fn resolve_this_symbol(name: &str) -> Option<String> {
    let body = name.strip_prefix(SYMBOL_PREFIX_THIS)?;
    let kind = body.rsplit_once('_').map_or(body, |(x, _)| x);
    match kind {
        "VRCUdonUdonBehaviour" => Some(SYMBOL_THIS.to_string()),
        "UnityEngineTransform" => Some(SYMBOL_THIS_TRANSFORM.to_string()),
        "UnityEngineGameObject" => Some(SYMBOL_THIS_GAME_OBJECT.to_string()),
        _ => None,
    }
}

fn classify_symbol_scope(name: &str) -> VariableScope {
    if name.starts_with(SYMBOL_PREFIX_CONST)
        || name.starts_with(SYMBOL_PREFIX_GLOBAL)
        || name.starts_with(SYMBOL_PREFIX_GINTNL)
        || name.starts_with(SYMBOL_PREFIX_THIS)
        || name == SYMBOL_THIS
        || name == SYMBOL_THIS_TRANSFORM
        || name == SYMBOL_THIS_GAME_OBJECT
    {
        return VariableScope::Global;
    }
    if name.starts_with(SYMBOL_PREFIX_LCL) {
        return VariableScope::Local;
    }
    if name.starts_with(SYMBOL_PREFIX_INTNL) {
        return VariableScope::Temporary;
    }
    VariableScope::Global
}
