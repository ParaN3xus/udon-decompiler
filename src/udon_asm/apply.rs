use std::collections::{HashMap, HashSet};

use crate::odin::{
    NodeKind, PrimitiveValue, Result as OdinResult, SymbolSection, UdonProgramBinary,
};

use super::literal::{
    EnumRepr, HeapLiteralValue, TYPE_MISSING, TYPE_SYSTEM_BOOLEAN, TYPE_SYSTEM_BYTE,
    TYPE_SYSTEM_DOUBLE, TYPE_SYSTEM_INT16, TYPE_SYSTEM_INT32, TYPE_SYSTEM_INT64, TYPE_SYSTEM_SBYTE,
    TYPE_SYSTEM_SINGLE, TYPE_SYSTEM_UINT16, TYPE_SYSTEM_UINT32, TYPE_SYSTEM_UINT64,
    TYPE_UNSERIALIZABLE, enum_repr_for_type, type_name_head,
};
use super::text::parse_generated_heap_symbol;
use super::types::{
    AsmError, BindDirective, BindTableDirective, EntryDirective, HeapDirective, HeapExportMark,
    Result, TypeRefDirective, Visibility,
};

pub(crate) fn apply_entry_directives(
    program: &mut UdonProgramBinary,
    directives: &[EntryDirective],
    label_to_addr: &HashMap<String, u32>,
) -> Result<()> {
    if directives.is_empty() {
        return Ok(());
    }

    let desired = directives
        .iter()
        .map(|d| {
            let addr = label_to_addr.get(&d.label).copied().ok_or_else(|| {
                AsmError::new(format!(
                    "Entry directive references unknown label '{}'.",
                    d.label
                ))
            })?;
            Ok((d.visibility, d.name.clone(), addr))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut len = program.symbols_len(SymbolSection::EntryPoints)?;
    while len > desired.len() {
        program.remove_symbol(SymbolSection::EntryPoints, len - 1)?;
        len -= 1;
    }
    while len < desired.len() {
        if len == 0 {
            return Err(AsmError::new(
                "Cannot create first EntryPoints symbol: no template entry exists.",
            ));
        }
        program.insert_symbol_clone(SymbolSection::EntryPoints, len, len - 1)?;
        len += 1;
    }

    for (idx, (_vis, name, address)) in desired.iter().enumerate() {
        program.set_symbol_name(SymbolSection::EntryPoints, idx, name.clone())?;
        program.set_symbol_address(SymbolSection::EntryPoints, idx, *address)?;
    }

    let desired_exported = desired
        .iter()
        .filter(|(vis, _, _)| *vis == Visibility::Public)
        .map(|(_, name, _)| name.clone())
        .collect::<Vec<_>>();

    let mut exported_len = program.exported_symbols_len(SymbolSection::EntryPoints)?;
    while exported_len > desired_exported.len() {
        program.remove_exported_symbol(SymbolSection::EntryPoints, exported_len - 1)?;
        exported_len -= 1;
    }

    if exported_len == 0 && !desired_exported.is_empty() {
        return Err(AsmError::new(
            "EntryPoints.ExportedSymbols is empty, cannot create first item without template.",
        ));
    }

    while exported_len < desired_exported.len() {
        program.insert_exported_symbol_clone(
            SymbolSection::EntryPoints,
            exported_len,
            exported_len - 1,
        )?;
        exported_len += 1;
    }

    for (idx, name) in desired_exported.iter().enumerate() {
        program.set_exported_symbol(SymbolSection::EntryPoints, idx, name.clone())?;
    }

    if desired_exported.is_empty() {
        while program.exported_symbols_len(SymbolSection::EntryPoints)? > 0 {
            let len_now = program.exported_symbols_len(SymbolSection::EntryPoints)?;
            program.remove_exported_symbol(SymbolSection::EntryPoints, len_now - 1)?;
        }
    }

    Ok(())
}

pub(crate) fn apply_heap_directives(
    program: &mut UdonProgramBinary,
    directives: &[HeapDirective],
) -> Result<()> {
    if directives.is_empty() {
        return Ok(());
    }

    for d in directives {
        let symbol_index = find_symbol_index(program, SymbolSection::SymbolTable, &d.symbol)?;
        let (symbol_index, heap_index) = if let Some(sym_idx) = symbol_index {
            let address = program
                .symbol_item(SymbolSection::SymbolTable, sym_idx)?
                .address;
            let heap_idx = find_heap_index_by_address(program, address)?.ok_or_else(|| {
                AsmError::new(format!(
                    "Symbol '{}' points to heap address {} but no heap entry exists.",
                    d.symbol, address
                ))
            })?;
            (Some(sym_idx), heap_idx)
        } else if let Some(address) = parse_generated_heap_symbol(&d.symbol) {
            let heap_idx = find_heap_index_by_address(program, address)?.ok_or_else(|| {
                AsmError::new(format!(
                    "Synthetic heap symbol '{}' points to missing heap address {}.",
                    d.symbol, address
                ))
            })?;
            (None, heap_idx)
        } else {
            let sym_len = program.symbols_len(SymbolSection::SymbolTable)?;
            if sym_len == 0 {
                return Err(AsmError::new(
                    "Cannot add first SymbolTable symbol: no template symbol exists.",
                ));
            }
            let template_index = sym_len - 1;
            program.insert_symbol_clone(SymbolSection::SymbolTable, sym_len, template_index)?;
            let new_sym_idx = sym_len;

            let heap_len = program.heap_dump_len()?;
            if heap_len == 0 {
                return Err(AsmError::new(
                    "Cannot add first heap dump entry: no template heap entry exists.",
                ));
            }
            let template_heap = heap_len - 1;
            program.insert_heap_dump_entry_clone(heap_len, template_heap)?;
            let new_heap_idx = heap_len;

            let new_address = next_free_heap_address(program)?;
            program.set_symbol_name(SymbolSection::SymbolTable, new_sym_idx, d.symbol.clone())?;
            program.set_symbol_address(SymbolSection::SymbolTable, new_sym_idx, new_address)?;
            program.set_heap_dump_address(new_heap_idx, new_address)?;
            if new_address >= program.heap_capacity()? {
                program.set_heap_capacity(new_address + 1)?;
            }
            (Some(new_sym_idx), new_heap_idx)
        };

        match d.type_ref {
            TypeRefDirective::Name(ref name) => {
                if name != TYPE_UNSERIALIZABLE && name != TYPE_MISSING {
                    let current_inline = program.heap_dump_type_name_string_inline(heap_index)?;
                    if current_inline.is_some() {
                        if current_inline.as_deref() != Some(name.as_str()) {
                            program.set_heap_dump_type_name_string(heap_index, name.clone())?;
                        }
                    } else {
                        let current_resolved = program.heap_dump_type_name_string(heap_index)?;
                        if current_resolved.as_deref() == Some(name.as_str()) {
                            // Keep original non-inline representation (for example InternalReference).
                        } else if symbol_index.is_some() {
                            // Existing non-inline type entries are currently not rewritten.
                        }
                    }
                }
            }
        }

        if let Some(sym_idx) = symbol_index {
            apply_symbol_table_export_mark(program, sym_idx, &d.symbol, d.export_mark)?;
        } else if d.export_mark != HeapExportMark::Keep {
            return Err(AsmError::new(format!(
                "Heap directive for synthetic symbol '{}' cannot set export mark.",
                d.symbol
            )));
        }

        apply_heap_init(program, heap_index, &d.init)?;
    }

    Ok(())
}

fn apply_heap_init(
    program: &mut UdonProgramBinary,
    heap_index: usize,
    init: &HeapLiteralValue,
) -> Result<()> {
    match init {
        HeapLiteralValue::Bool(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Boolean(*v))?;
        }
        HeapLiteralValue::U8(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Byte(*v))?;
        }
        HeapLiteralValue::I8(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::SByte(*v))?;
        }
        HeapLiteralValue::U16(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::UShort(*v))?;
        }
        HeapLiteralValue::I16(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Short(*v))?;
        }
        HeapLiteralValue::U32(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::UInt(*v))?;
        }
        HeapLiteralValue::I32(v) => {
            program.set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Int(*v))?;
        }
        HeapLiteralValue::U64(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::ULong(*v))?;
        }
        HeapLiteralValue::I64(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Long(*v))?;
        }
        HeapLiteralValue::F32(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Float(*v))?;
        }
        HeapLiteralValue::F64(v) => {
            program
                .set_heap_dump_strongbox_value_primitive(heap_index, PrimitiveValue::Double(*v))?;
        }
        HeapLiteralValue::String(v) => {
            program.set_heap_dump_strongbox_value_primitive(
                heap_index,
                PrimitiveValue::String(crate::odin::OdinString::utf16(v.clone())),
            )?;
        }
        HeapLiteralValue::SystemType(v) => {
            program.set_heap_dump_strongbox_system_type_name(heap_index, v.as_str())?;
        }
        HeapLiteralValue::Vector2(x, y) => {
            program.set_heap_dump_strongbox_vector2(heap_index, *x, *y)?;
        }
        HeapLiteralValue::Vector3(x, y, z) => {
            program.set_heap_dump_strongbox_vector3(heap_index, *x, *y, *z)?;
        }
        HeapLiteralValue::Quaternion(x, y, z, w) => {
            program.set_heap_dump_strongbox_quaternion(heap_index, *x, *y, *z, *w)?;
        }
        HeapLiteralValue::Color(r, g, b, a) => {
            program.set_heap_dump_strongbox_color(heap_index, *r, *g, *b, *a)?;
        }
        HeapLiteralValue::SerializationResult {
            success,
            byte_count,
        } => {
            program.set_heap_dump_strongbox_serialization_result(
                heap_index,
                *success,
                *byte_count,
            )?;
        }
        HeapLiteralValue::TypedArray {
            element_type,
            elements,
        } => {
            program.set_heap_dump_strongbox_typed_array(heap_index, element_type, elements)?;
        }
        HeapLiteralValue::Null => {
            let null_template = find_heap_entry_with_null_value(program)?.ok_or_else(|| {
                AsmError::new("Cannot apply null init: no heap entry with null value exists.")
            })?;
            program.set_heap_dump_strongbox_from_entry(heap_index, null_template)?;
        }
        HeapLiteralValue::U32Array(values) => {
            program.set_heap_dump_strongbox_u32_array(heap_index, values)?;
        }
        HeapLiteralValue::OpaqueArray { .. } | HeapLiteralValue::Unserializable => {}
    }
    Ok(())
}

pub(crate) fn apply_bind_directives(
    program: &mut UdonProgramBinary,
    directives: &[BindDirective],
    label_to_addr: &HashMap<String, u32>,
) -> Result<()> {
    for bind in directives {
        let address = label_to_addr.get(&bind.label).copied().ok_or_else(|| {
            AsmError::new(format!(
                "bind directive references unknown label '{}'.",
                bind.label
            ))
        })?;
        program.set_heap_value_u32_by_symbol(&bind.symbol, address)?;
    }
    Ok(())
}

pub(crate) fn apply_bind_table_directives(
    program: &mut UdonProgramBinary,
    directives: &[BindTableDirective],
    label_to_addr: &HashMap<String, u32>,
) -> Result<()> {
    for bind in directives {
        let mut values = Vec::<u32>::with_capacity(bind.labels.len());
        for label in &bind.labels {
            values.push(
                label_to_addr.get(label).copied().ok_or_else(|| {
                    AsmError::new(format!("Unknown bind-table label '{}'.", label))
                })?,
            );
        }
        program.set_heap_value_u32_array_by_symbol(&bind.symbol, &values)?;
    }
    Ok(())
}

fn find_symbol_index(
    program: &UdonProgramBinary,
    section: SymbolSection,
    name: &str,
) -> Result<Option<usize>> {
    let len = program.symbols_len(section)?;
    for i in 0..len {
        if program.symbol_item(section, i)?.name == name {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

fn find_exported_symbol_index(
    program: &UdonProgramBinary,
    section: SymbolSection,
    name: &str,
) -> Result<Option<usize>> {
    let len = program.exported_symbols_len(section)?;
    for i in 0..len {
        if program.exported_symbol(section, i)? == name {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

fn apply_symbol_table_export_mark(
    program: &mut UdonProgramBinary,
    _symbol_index: usize,
    symbol_name: &str,
    mark: HeapExportMark,
) -> Result<()> {
    match mark {
        HeapExportMark::Keep => Ok(()),
        HeapExportMark::Exported => {
            if find_exported_symbol_index(program, SymbolSection::SymbolTable, symbol_name)?
                .is_none()
            {
                let len = program.exported_symbols_len(SymbolSection::SymbolTable)?;
                if len == 0 {
                    return Err(AsmError::new(
                        "SymbolTable.ExportedSymbols is empty, cannot create first item without template.",
                    ));
                }
                program.insert_exported_symbol_clone(SymbolSection::SymbolTable, len, len - 1)?;
                program.set_exported_symbol(
                    SymbolSection::SymbolTable,
                    len,
                    symbol_name.to_string(),
                )?;
            }
            Ok(())
        }
        HeapExportMark::Private => {
            if let Some(idx) =
                find_exported_symbol_index(program, SymbolSection::SymbolTable, symbol_name)?
            {
                program.remove_exported_symbol(SymbolSection::SymbolTable, idx)?;
            }
            Ok(())
        }
    }
}

fn find_heap_index_by_address(program: &UdonProgramBinary, address: u32) -> Result<Option<usize>> {
    let len = program.heap_dump_len()?;
    for i in 0..len {
        if program.heap_dump_item(i)?.address == address {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

fn next_free_heap_address(program: &UdonProgramBinary) -> Result<u32> {
    let mut used = HashSet::<u32>::new();
    let len = program.heap_dump_len()?;
    for i in 0..len {
        used.insert(program.heap_dump_item(i)?.address);
    }
    let mut candidate = 0_u32;
    loop {
        if !used.contains(&candidate) {
            return Ok(candidate);
        }
        candidate = candidate
            .checked_add(1)
            .ok_or_else(|| AsmError::new("Heap address overflow."))?;
    }
}

fn find_heap_entry_with_null_value(program: &UdonProgramBinary) -> Result<Option<usize>> {
    let len = program.heap_dump_len()?;
    for i in 0..len {
        let item = program.heap_dump_item(i)?;
        if matches!(item.strongbox_value_kind, NodeKind::Null) {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

trait ProgramExt {
    fn set_heap_dump_strongbox_u32_array(
        &mut self,
        heap_index: usize,
        values: &[u32],
    ) -> OdinResult<()>;
    fn set_heap_value_u32_by_symbol(&mut self, symbol: &str, value: u32) -> OdinResult<()>;
    fn set_heap_value_u32_array_by_symbol(
        &mut self,
        symbol: &str,
        values: &[u32],
    ) -> OdinResult<()>;
    fn set_heap_dump_strongbox_typed_array(
        &mut self,
        heap_index: usize,
        element_type: &str,
        elements: &[HeapLiteralValue],
    ) -> OdinResult<()>;
}

impl ProgramExt for UdonProgramBinary {
    fn set_heap_dump_strongbox_u32_array(
        &mut self,
        heap_index: usize,
        values: &[u32],
    ) -> OdinResult<()> {
        let raw = values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        self.set_heap_dump_strongbox_u32_array_raw(heap_index, &raw)
    }

    fn set_heap_value_u32_by_symbol(&mut self, symbol: &str, value: u32) -> OdinResult<()> {
        let symbol_count = self.symbols_len(SymbolSection::SymbolTable)?;
        let mut address = None;
        for i in 0..symbol_count {
            let item = self.symbol_item(SymbolSection::SymbolTable, i)?;
            if item.name == symbol {
                address = Some(item.address);
                break;
            }
        }
        if address.is_none() {
            address = parse_generated_heap_symbol(symbol);
        }
        let address = address
            .ok_or_else(|| crate::odin::OdinError::new(format!("Unknown symbol '{}'.", symbol)))?;
        let heap_len = self.heap_dump_len()?;
        for i in 0..heap_len {
            if self.heap_dump_item(i)?.address == address {
                return self
                    .set_heap_dump_strongbox_value_primitive(i, PrimitiveValue::UInt(value));
            }
        }
        Err(crate::odin::OdinError::new(format!(
            "Symbol '{}' points to heap address {} but no heap entry exists.",
            symbol, address
        )))
    }

    fn set_heap_value_u32_array_by_symbol(
        &mut self,
        symbol: &str,
        values: &[u32],
    ) -> OdinResult<()> {
        let symbol_count = self.symbols_len(SymbolSection::SymbolTable)?;
        let mut address = None;
        for i in 0..symbol_count {
            let item = self.symbol_item(SymbolSection::SymbolTable, i)?;
            if item.name == symbol {
                address = Some(item.address);
                break;
            }
        }
        if address.is_none() {
            address = parse_generated_heap_symbol(symbol);
        }
        let address = address
            .ok_or_else(|| crate::odin::OdinError::new(format!("Unknown symbol '{}'.", symbol)))?;
        let heap_len = self.heap_dump_len()?;
        for i in 0..heap_len {
            if self.heap_dump_item(i)?.address == address {
                return self.set_heap_dump_strongbox_u32_array(i, values);
            }
        }
        Err(crate::odin::OdinError::new(format!(
            "Symbol '{}' points to heap address {} but no heap entry exists.",
            symbol, address
        )))
    }

    fn set_heap_dump_strongbox_typed_array(
        &mut self,
        heap_index: usize,
        element_type: &str,
        elements: &[HeapLiteralValue],
    ) -> OdinResult<()> {
        let mut raw = Vec::<u8>::new();
        for element in elements {
            append_typed_array_element_raw(&mut raw, element_type, element)?;
        }
        self.set_heap_dump_strongbox_u32_array_raw(heap_index, raw.as_slice())
    }
}

fn append_typed_array_element_raw(
    out: &mut Vec<u8>,
    element_type: &str,
    element: &HeapLiteralValue,
) -> OdinResult<()> {
    let head = type_name_head(element_type);
    if let Some(repr) = enum_repr_for_type(head) {
        match (repr, element) {
            (EnumRepr::U8, HeapLiteralValue::U8(v)) => out.push(*v),
            (EnumRepr::U8, HeapLiteralValue::I32(v)) if (0..=u8::MAX as i32).contains(v) => {
                out.push(*v as u8);
            }
            (EnumRepr::I32, HeapLiteralValue::I32(v)) => out.extend_from_slice(&v.to_le_bytes()),
            (EnumRepr::I32, HeapLiteralValue::U32(v)) if *v <= i32::MAX as u32 => {
                out.extend_from_slice(&(*v as i32).to_le_bytes());
            }
            _ => {
                return Err(crate::odin::OdinError::new(format!(
                    "Unsupported enum array element for '{}': {:?}.",
                    element_type, element
                )));
            }
        }
        return Ok(());
    }

    match (head, element) {
        (TYPE_SYSTEM_BOOLEAN, HeapLiteralValue::Bool(v)) => out.push(if *v { 1 } else { 0 }),
        (TYPE_SYSTEM_SBYTE, HeapLiteralValue::I8(v)) => out.push(*v as u8),
        (TYPE_SYSTEM_BYTE, HeapLiteralValue::U8(v)) => out.push(*v),
        (TYPE_SYSTEM_INT16, HeapLiteralValue::I16(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_UINT16, HeapLiteralValue::U16(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_INT32, HeapLiteralValue::I32(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_UINT32, HeapLiteralValue::U32(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_INT64, HeapLiteralValue::I64(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_UINT64, HeapLiteralValue::U64(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_SINGLE, HeapLiteralValue::F32(v)) => out.extend_from_slice(&v.to_le_bytes()),
        (TYPE_SYSTEM_DOUBLE, HeapLiteralValue::F64(v)) => out.extend_from_slice(&v.to_le_bytes()),
        _ => {
            return Err(crate::odin::OdinError::new(format!(
                "Unsupported typed array element for '{}': {:?}.",
                element_type, element
            )));
        }
    }
    Ok(())
}
