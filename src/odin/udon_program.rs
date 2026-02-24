use base64::Engine as _;

use crate::odin::{
    NodeId, NodeKind, OdinDocument, OdinError, OdinString, PrimitiveValue, Result, StringEncoding,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolSection {
    EntryPoints,
    SymbolTable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolItem {
    pub name: String,
    pub address: u32,
    pub type_node_kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeapDumpItem {
    pub address: u32,
    pub strongbox_value_kind: NodeKind,
    pub type_kind: NodeKind,
}

#[derive(Debug, Clone)]
pub struct UdonProgramBinary {
    doc: OdinDocument,
}

impl UdonProgramBinary {
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            doc: OdinDocument::parse(bytes)?,
        })
    }

    pub fn parse_base64(text: &str) -> Result<Self> {
        let compact = text
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace())
            .collect::<String>();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(compact.as_bytes())
            .map_err(|e| OdinError::new(format!("Failed to decode base64: {e}")))?;
        Self::parse_bytes(&bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.doc.to_bytes()
    }

    pub fn into_document(self) -> OdinDocument {
        self.doc
    }

    pub fn document(&self) -> &OdinDocument {
        &self.doc
    }

    pub fn document_mut(&mut self) -> &mut OdinDocument {
        &mut self.doc
    }

    pub fn byte_code(&self) -> Result<Vec<u8>> {
        let node = self.byte_code_node_id()?;
        let (count, bpe) = primitive_array_shape(&self.doc, node)?;
        let mut out = Vec::with_capacity(count.saturating_mul(bpe));
        for index in 0..count {
            out.extend_from_slice(self.doc.primitive_array_element(node, index)?);
        }
        Ok(out)
    }

    pub fn set_byte_code(&mut self, bytes: &[u8]) -> Result<()> {
        let node = self.byte_code_node_id()?;
        self.doc.replace_primitive_array(node, bytes)
    }

    pub fn heap_capacity(&self) -> Result<u32> {
        let node = self.heap_capacity_node_id()?;
        read_u32_primitive(&self.doc, node, "HeapCapacity")
    }

    pub fn set_heap_capacity(&mut self, value: u32) -> Result<()> {
        let node = self.heap_capacity_node_id()?;
        self.doc.set_primitive(node, PrimitiveValue::UInt(value))
    }

    pub fn heap_dump_len(&self) -> Result<usize> {
        Ok(self.heap_dump_entry_node_ids()?.len())
    }

    pub fn heap_dump_item(&self, index: usize) -> Result<HeapDumpItem> {
        let tuple_node = self.heap_dump_tuple_node_id(index)?;
        let address_node = named_child(&self.doc, tuple_node, "Item1")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item1.")))?;
        let item2_node = named_child(&self.doc, tuple_node, "Item2")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item2.")))?;
        let value_node = named_child(&self.doc, item2_node, "Value")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}].Item2 missing Value.")))?;
        let type_node = named_child(&self.doc, tuple_node, "Item3")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item3.")))?;
        Ok(HeapDumpItem {
            address: read_u32_primitive(&self.doc, address_node, "HeapDump.Item1")?,
            strongbox_value_kind: self
                .doc
                .node(value_node)
                .ok_or_else(|| OdinError::new("Invalid strongbox value node id."))?
                .kind()
                .clone(),
            type_kind: self
                .doc
                .node(type_node)
                .ok_or_else(|| OdinError::new("Invalid heap type node id."))?
                .kind()
                .clone(),
        })
    }

    pub fn set_heap_dump_address(&mut self, index: usize, value: u32) -> Result<()> {
        let tuple_node = self.heap_dump_tuple_node_id(index)?;
        let address_node = named_child(&self.doc, tuple_node, "Item1")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item1.")))?;
        self.doc
            .set_primitive(address_node, PrimitiveValue::UInt(value))
    }

    pub fn set_heap_dump_strongbox_value_primitive(
        &mut self,
        index: usize,
        value: PrimitiveValue,
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        self.doc.set_primitive_resolved(value_node, value)
    }

    pub fn set_heap_dump_strongbox_u32_array_raw(
        &mut self,
        index: usize,
        raw: &[u8],
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved_value_node = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let target_node = if matches!(
            self.doc
                .node(resolved_value_node)
                .ok_or_else(|| OdinError::new("Invalid strongbox value node id."))?
                .kind(),
            NodeKind::PrimitiveArray { .. }
        ) {
            resolved_value_node
        } else {
            let child = first_child(&self.doc, resolved_value_node).ok_or_else(|| {
                OdinError::new(format!(
                    "HeapDump[{index}] value is not PrimitiveArray and has no child."
                ))
            })?;
            if !matches!(
                self.doc
                    .node(child)
                    .ok_or_else(|| OdinError::new("Invalid child node id."))?
                    .kind(),
                NodeKind::PrimitiveArray { .. }
            ) {
                return Err(OdinError::new(format!(
                    "HeapDump[{index}] value does not contain a PrimitiveArray node."
                )));
            }
            child
        };
        self.doc.replace_primitive_array(target_node, raw)
    }

    pub fn set_heap_dump_strongbox_from_entry(
        &mut self,
        target_index: usize,
        source_index: usize,
    ) -> Result<()> {
        let target_tuple = self.heap_dump_tuple_node_id(target_index)?;
        let source_tuple = self.heap_dump_tuple_node_id(source_index)?;
        let target_item2 = named_child(&self.doc, target_tuple, "Item2")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{target_index}] missing Item2.")))?;
        let source_item2 = named_child(&self.doc, source_tuple, "Item2")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{source_index}] missing Item2.")))?;
        self.doc.replace_node_with_clone(target_item2, source_item2)
    }

    pub fn set_heap_dump_type_internal_reference(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<()> {
        let type_node = self.heap_dump_type_node_id(index)?;
        self.doc.set_internal_reference(type_node, value)
    }

    pub fn set_heap_dump_type_name_string(
        &mut self,
        index: usize,
        value: impl Into<String>,
    ) -> Result<()> {
        let type_node = self.heap_dump_type_node_id(index)?;
        let value = value.into();
        if matches!(
            self.doc
                .node(type_node)
                .ok_or_else(|| OdinError::new("Invalid heap type node id."))?
                .kind(),
            NodeKind::Primitive(PrimitiveValue::String(_))
        ) {
            return set_string_like_primitive(&mut self.doc, type_node, value, "HeapDump.Item3");
        }
        let inner = first_child(&self.doc, type_node).ok_or_else(|| {
            OdinError::new(format!(
                "HeapDump[{index}].Item3 is not a reference node with inline type value."
            ))
        })?;
        set_string_like_primitive(&mut self.doc, inner, value, "HeapDump.Item3")
    }

    pub fn heap_dump_type_name_string(&self, index: usize) -> Result<Option<String>> {
        let type_node = self.heap_dump_type_node_id(index)?;
        if let Some(value) = extract_type_name_from_node(&self.doc, type_node) {
            return Ok(Some(value));
        }
        let resolved = self
            .doc
            .resolve_node_payload(type_node)
            .unwrap_or(type_node);
        Ok(extract_type_name_from_node(&self.doc, resolved))
    }

    pub(crate) fn heap_dump_type_name_string_inline(&self, index: usize) -> Result<Option<String>> {
        let type_node = self.heap_dump_type_node_id(index)?;
        Ok(extract_type_name_from_node(&self.doc, type_node))
    }

    pub fn heap_dump_strongbox_value_kind(&self, index: usize) -> Result<NodeKind> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        Ok(self
            .doc
            .node(resolved)
            .ok_or_else(|| OdinError::new("Invalid resolved strongbox value node id."))?
            .kind()
            .clone())
    }

    pub fn set_heap_dump_type_from_entry(
        &mut self,
        target_index: usize,
        source_index: usize,
    ) -> Result<()> {
        let target_type = self.heap_dump_type_node_id(target_index)?;
        let source_type = self.heap_dump_type_node_id(source_index)?;
        self.doc.replace_node_with_clone(target_type, source_type)
    }

    pub fn insert_heap_dump_entry_clone(
        &mut self,
        insert_index: usize,
        template_index: usize,
    ) -> Result<()> {
        let heap_dump_array = self.heap_dump_array_node_id()?;
        let template_node = self.heap_dump_tuple_node_id(template_index)?;
        self.doc
            .array_insert_clone(heap_dump_array, insert_index, template_node)
    }

    pub fn remove_heap_dump_entry(&mut self, index: usize) -> Result<()> {
        let heap_dump_array = self.heap_dump_array_node_id()?;
        self.doc.array_remove_at(heap_dump_array, index)
    }

    pub fn symbols_len(&self, section: SymbolSection) -> Result<usize> {
        Ok(self.symbol_item_node_ids(section)?.len())
    }

    pub fn symbol_item(&self, section: SymbolSection, index: usize) -> Result<SymbolItem> {
        let symbol_node = self.symbol_item_node_id(section, index)?;
        let payload = first_child(&self.doc, symbol_node).ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] has no payload node.",
                section_name(section)
            ))
        })?;
        let name_node = named_child(&self.doc, payload, "Name").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] missing Name.",
                section_name(section)
            ))
        })?;
        let type_node = named_child(&self.doc, payload, "Type").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] missing Type.",
                section_name(section)
            ))
        })?;
        let address_node = named_child(&self.doc, payload, "Address").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] missing Address.",
                section_name(section)
            ))
        })?;
        let name = read_string_primitive(&self.doc, name_node, "Symbol.Name")?;
        let address = read_u32_primitive(&self.doc, address_node, "Symbol.Address")?;
        let type_node_kind = self
            .doc
            .node(type_node)
            .ok_or_else(|| OdinError::new("Invalid type node id."))?
            .kind()
            .clone();
        Ok(SymbolItem {
            name,
            address,
            type_node_kind,
        })
    }

    pub fn set_symbol_name(
        &mut self,
        section: SymbolSection,
        index: usize,
        value: impl Into<String>,
    ) -> Result<()> {
        let symbol_node = self.symbol_item_node_id(section, index)?;
        let payload = first_child(&self.doc, symbol_node).ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] has no payload node.",
                section_name(section)
            ))
        })?;
        let name_node = named_child(&self.doc, payload, "Name").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] missing Name.",
                section_name(section)
            ))
        })?;
        set_string_like_primitive(&mut self.doc, name_node, value.into(), "Symbol.Name")
    }

    pub fn set_symbol_address(
        &mut self,
        section: SymbolSection,
        index: usize,
        value: u32,
    ) -> Result<()> {
        let symbol_node = self.symbol_item_node_id(section, index)?;
        let payload = first_child(&self.doc, symbol_node).ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] has no payload node.",
                section_name(section)
            ))
        })?;
        let address_node = named_child(&self.doc, payload, "Address").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] missing Address.",
                section_name(section)
            ))
        })?;
        self.doc
            .set_primitive(address_node, PrimitiveValue::UInt(value))
    }

    pub fn set_symbol_type_internal_reference(
        &mut self,
        section: SymbolSection,
        index: usize,
        value: i32,
    ) -> Result<()> {
        let type_node = self.symbol_type_node_id(section, index)?;
        self.doc.set_internal_reference(type_node, value)
    }

    pub fn set_symbol_type_from_symbol(
        &mut self,
        section: SymbolSection,
        target_index: usize,
        source_index: usize,
    ) -> Result<()> {
        let target_type = self.symbol_type_node_id(section, target_index)?;
        let source_type = self.symbol_type_node_id(section, source_index)?;
        self.doc.replace_node_with_clone(target_type, source_type)
    }

    pub fn insert_symbol_clone(
        &mut self,
        section: SymbolSection,
        insert_index: usize,
        template_index: usize,
    ) -> Result<()> {
        let symbols_array = self.symbols_array_node_id(section)?;
        let template = self.symbol_item_node_id(section, template_index)?;
        self.doc
            .array_insert_clone(symbols_array, insert_index, template)
    }

    pub fn remove_symbol(&mut self, section: SymbolSection, index: usize) -> Result<()> {
        let symbols_array = self.symbols_array_node_id(section)?;
        self.doc.array_remove_at(symbols_array, index)
    }

    pub fn exported_symbols_len(&self, section: SymbolSection) -> Result<usize> {
        Ok(self.exported_symbol_node_ids(section)?.len())
    }

    pub fn exported_symbol(&self, section: SymbolSection, index: usize) -> Result<String> {
        let node = self.exported_symbol_node_id(section, index)?;
        read_string_primitive(&self.doc, node, "ExportedSymbol")
    }

    pub fn set_exported_symbol(
        &mut self,
        section: SymbolSection,
        index: usize,
        value: impl Into<String>,
    ) -> Result<()> {
        let node = self.exported_symbol_node_id(section, index)?;
        set_string_like_primitive(&mut self.doc, node, value.into(), "ExportedSymbol")
    }

    pub fn insert_exported_symbol_clone(
        &mut self,
        section: SymbolSection,
        insert_index: usize,
        template_index: usize,
    ) -> Result<()> {
        let array = self.exported_symbols_array_node_id(section)?;
        let template = self.exported_symbol_node_id(section, template_index)?;
        self.doc.array_insert_clone(array, insert_index, template)
    }

    pub fn remove_exported_symbol(&mut self, section: SymbolSection, index: usize) -> Result<()> {
        let array = self.exported_symbols_array_node_id(section)?;
        self.doc.array_remove_at(array, index)
    }

    fn byte_code_node_id(&self) -> Result<NodeId> {
        let byte_code_field = self.program_field_node("ByteCode")?;
        first_child(&self.doc, byte_code_field)
            .ok_or_else(|| OdinError::new("ByteCode field has no payload node."))
    }

    fn heap_capacity_node_id(&self) -> Result<NodeId> {
        let heap_payload = self.heap_payload_array_node_id()?;
        named_child(&self.doc, heap_payload, "HeapCapacity")
            .ok_or_else(|| OdinError::new("HeapCapacity field not found."))
    }

    fn heap_dump_array_node_id(&self) -> Result<NodeId> {
        let heap_payload = self.heap_payload_array_node_id()?;
        let heap_dump_field = named_child(&self.doc, heap_payload, "HeapDump")
            .ok_or_else(|| OdinError::new("HeapDump field not found."))?;
        first_child(&self.doc, heap_dump_field)
            .ok_or_else(|| OdinError::new("HeapDump field has no payload array."))
    }

    fn heap_dump_tuple_node_id(&self, index: usize) -> Result<NodeId> {
        let ids = self.heap_dump_entry_node_ids()?;
        ids.get(index)
            .copied()
            .ok_or_else(|| OdinError::new(format!("HeapDump index {} is out of range.", index)))
    }

    fn heap_dump_strongbox_value_node_id(&self, index: usize) -> Result<NodeId> {
        let tuple = self.heap_dump_tuple_node_id(index)?;
        let item2 = named_child(&self.doc, tuple, "Item2")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item2.")))?;
        named_child(&self.doc, item2, "Value")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item2.Value.")))
    }

    fn heap_dump_type_node_id(&self, index: usize) -> Result<NodeId> {
        let tuple = self.heap_dump_tuple_node_id(index)?;
        named_child(&self.doc, tuple, "Item3")
            .ok_or_else(|| OdinError::new(format!("HeapDump[{index}] missing Item3.")))
    }

    fn heap_dump_entry_node_ids(&self) -> Result<Vec<NodeId>> {
        array_element_nodes(&self.doc, self.heap_dump_array_node_id()?)
    }

    fn heap_payload_array_node_id(&self) -> Result<NodeId> {
        let heap_field = self.program_field_node("Heap")?;
        first_child(&self.doc, heap_field)
            .ok_or_else(|| OdinError::new("Heap field has no payload array."))
    }

    fn symbols_array_node_id(&self, section: SymbolSection) -> Result<NodeId> {
        let payload = self.section_payload_array_node_id(section)?;
        let symbols_field = named_child(&self.doc, payload, "Symbols").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols field not found.",
                section_name(section)
            ))
        })?;
        first_child(&self.doc, symbols_field).ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols has no payload array.",
                section_name(section)
            ))
        })
    }

    fn exported_symbols_array_node_id(&self, section: SymbolSection) -> Result<NodeId> {
        let payload = self.section_payload_array_node_id(section)?;
        let field = named_child(&self.doc, payload, "ExportedSymbols").ok_or_else(|| {
            OdinError::new(format!(
                "{}.ExportedSymbols field not found.",
                section_name(section)
            ))
        })?;
        first_child(&self.doc, field).ok_or_else(|| {
            OdinError::new(format!(
                "{}.ExportedSymbols has no payload array.",
                section_name(section)
            ))
        })
    }

    fn symbol_item_node_id(&self, section: SymbolSection, index: usize) -> Result<NodeId> {
        let ids = self.symbol_item_node_ids(section)?;
        ids.get(index).copied().ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols index {} is out of range.",
                section_name(section),
                index
            ))
        })
    }

    fn symbol_type_node_id(&self, section: SymbolSection, index: usize) -> Result<NodeId> {
        let symbol = self.symbol_item_node_id(section, index)?;
        let payload = first_child(&self.doc, symbol).ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] has no payload.",
                section_name(section)
            ))
        })?;
        named_child(&self.doc, payload, "Type").ok_or_else(|| {
            OdinError::new(format!(
                "{}.Symbols[{index}] missing Type.",
                section_name(section)
            ))
        })
    }

    fn exported_symbol_node_id(&self, section: SymbolSection, index: usize) -> Result<NodeId> {
        let ids = self.exported_symbol_node_ids(section)?;
        ids.get(index).copied().ok_or_else(|| {
            OdinError::new(format!(
                "{}.ExportedSymbols index {} is out of range.",
                section_name(section),
                index
            ))
        })
    }

    fn symbol_item_node_ids(&self, section: SymbolSection) -> Result<Vec<NodeId>> {
        array_element_nodes(&self.doc, self.symbols_array_node_id(section)?)
    }

    fn exported_symbol_node_ids(&self, section: SymbolSection) -> Result<Vec<NodeId>> {
        array_element_nodes(&self.doc, self.exported_symbols_array_node_id(section)?)
    }

    fn section_payload_array_node_id(&self, section: SymbolSection) -> Result<NodeId> {
        let field = self.program_field_node(match section {
            SymbolSection::EntryPoints => "EntryPoints",
            SymbolSection::SymbolTable => "SymbolTable",
        })?;
        first_child(&self.doc, field).ok_or_else(|| {
            OdinError::new(format!("{} has no payload array.", section_name(section)))
        })
    }

    fn program_field_node(&self, field_name: &str) -> Result<NodeId> {
        let root = find_program_root(&self.doc)?;
        named_child(&self.doc, root, field_name)
            .ok_or_else(|| OdinError::new(format!("Top-level field '{}' not found.", field_name)))
    }
}

fn find_program_root(doc: &OdinDocument) -> Result<NodeId> {
    doc.root_nodes()
        .iter()
        .copied()
        .find(|root| named_child(doc, *root, "ByteCode").is_some())
        .ok_or_else(|| OdinError::new("Unable to locate UdonProgram root node."))
}

fn primitive_array_shape(doc: &OdinDocument, node_id: NodeId) -> Result<(usize, usize)> {
    let node = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
    match node.kind() {
        NodeKind::PrimitiveArray {
            element_count,
            bytes_per_element,
        } => {
            if *element_count < 0 || *bytes_per_element <= 0 {
                return Err(OdinError::new(format!(
                    "Invalid PrimitiveArray shape count={}, bpe={}.",
                    element_count, bytes_per_element
                )));
            }
            Ok((*element_count as usize, *bytes_per_element as usize))
        }
        _ => Err(OdinError::new(format!(
            "Node {} is not a PrimitiveArray.",
            node_id
        ))),
    }
}

fn read_u32_primitive(doc: &OdinDocument, node_id: NodeId, what: &str) -> Result<u32> {
    let node = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("{what}: node {node_id} out of range.")))?;
    match node.kind() {
        NodeKind::Primitive(PrimitiveValue::UInt(v)) => Ok(*v),
        other => Err(OdinError::new(format!(
            "{what}: expected UInt primitive, got {:?}.",
            other
        ))),
    }
}

fn read_string_primitive(doc: &OdinDocument, node_id: NodeId, what: &str) -> Result<String> {
    let node = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("{what}: node {node_id} out of range.")))?;
    match node.kind() {
        NodeKind::Primitive(PrimitiveValue::String(v)) => Ok(v.value.clone()),
        other => Err(OdinError::new(format!(
            "{what}: expected String primitive, got {:?}.",
            other
        ))),
    }
}

fn set_string_like_primitive(
    doc: &mut OdinDocument,
    node_id: NodeId,
    value: String,
    what: &str,
) -> Result<()> {
    let encoding = {
        let node = doc
            .node(node_id)
            .ok_or_else(|| OdinError::new(format!("{what}: node {node_id} out of range.")))?;
        match node.kind() {
            NodeKind::Primitive(PrimitiveValue::String(v)) => v.encoding,
            other => {
                return Err(OdinError::new(format!(
                    "{what}: expected String primitive, got {:?}.",
                    other
                )));
            }
        }
    };
    doc.set_primitive(
        node_id,
        PrimitiveValue::String(OdinString {
            value,
            encoding: match encoding {
                StringEncoding::SingleByte => StringEncoding::SingleByte,
                StringEncoding::Utf16 => StringEncoding::Utf16,
            },
        }),
    )
}

fn named_child(doc: &OdinDocument, parent_id: NodeId, name: &str) -> Option<NodeId> {
    doc.node(parent_id)?.children().iter().copied().find(|id| {
        doc.node(*id)
            .and_then(|node| node.name())
            .map(|value| value.value.as_str() == name)
            .unwrap_or(false)
    })
}

fn first_child(doc: &OdinDocument, parent_id: NodeId) -> Option<NodeId> {
    doc.node(parent_id)?.children().first().copied()
}

fn array_element_nodes(doc: &OdinDocument, array_node_id: NodeId) -> Result<Vec<NodeId>> {
    let array_node = doc
        .node(array_node_id)
        .ok_or_else(|| OdinError::new(format!("Array node {} is out of range.", array_node_id)))?;
    if !matches!(array_node.kind(), NodeKind::Array { .. }) {
        return Err(OdinError::new(format!(
            "Node {} is not a normal Array node.",
            array_node_id
        )));
    }
    let mut ids = array_node
        .children()
        .iter()
        .copied()
        .filter(|id| doc.node(*id).and_then(|x| x.array_index()).is_some())
        .collect::<Vec<_>>();
    ids.sort_by_key(|id| {
        doc.node(*id)
            .and_then(|x| x.array_index())
            .unwrap_or(usize::MAX)
    });
    Ok(ids)
}

fn section_name(section: SymbolSection) -> &'static str {
    match section {
        SymbolSection::EntryPoints => "EntryPoints",
        SymbolSection::SymbolTable => "SymbolTable",
    }
}

fn extract_type_name_from_node(doc: &OdinDocument, node_id: NodeId) -> Option<String> {
    let node = doc.node(node_id)?;
    match node.kind() {
        NodeKind::Primitive(PrimitiveValue::String(value)) => Some(value.value.clone()),
        NodeKind::TypeNameMetadata { name, .. } => Some(name.value.clone()),
        NodeKind::TypeIdMetadata {
            resolved_name: Some(name),
            ..
        } => Some(name.clone()),
        NodeKind::ReferenceNode { type_ref, .. } | NodeKind::StructNode { type_ref } => {
            match type_ref {
                crate::odin::OdinTypeRef::TypeName { name, .. } => Some(name.value.clone()),
                crate::odin::OdinTypeRef::TypeId {
                    resolved_name: Some(name),
                    ..
                } => Some(name.clone()),
                _ => None,
            }
        }
        _ => first_child(doc, node_id).and_then(|child| extract_type_name_from_node(doc, child)),
    }
}
