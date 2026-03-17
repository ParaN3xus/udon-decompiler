use base64::Engine as _;

use crate::odin::{
    NodeId, NodeKind, OdinDocument, OdinError, OdinString, PrimitiveValue, Result, StringEncoding,
};
use crate::str_constants::{TYPE_SYSTEM_RUNTIME_TYPE, TYPE_SYSTEM_TYPE};

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
        let resolved = self
            .doc
            .resolve_node_payload(type_node)
            .unwrap_or(type_node);
        if let Some(value) = extract_type_name_from_type_payload(&self.doc, resolved) {
            return Ok(Some(value));
        }
        if let Some(value) = extract_type_name_from_node(&self.doc, resolved)
            && !is_runtime_type_name(value.as_str())
        {
            return Ok(Some(value));
        }
        if let Some(value) = extract_type_name_from_node(&self.doc, type_node)
            && !is_runtime_type_name(value.as_str())
        {
            return Ok(Some(value));
        }
        Ok(None)
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

    pub fn heap_dump_strongbox_u32_array(&self, index: usize) -> Result<Option<Vec<u32>>> {
        let Some((bpe, raw)) = self.heap_dump_strongbox_primitive_array_raw(index)? else {
            return Ok(None);
        };
        if bpe != 4 || raw.len() % 4 != 0 {
            return Ok(None);
        }
        let mut out = Vec::<u32>::with_capacity(raw.len() / 4);
        for chunk in raw.chunks_exact(4) {
            if chunk.len() != 4 {
                return Err(OdinError::new(format!(
                    "HeapDump[{index}] has invalid u32 array element size {}.",
                    chunk.len()
                )));
            }
            out.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Ok(Some(out))
    }

    pub fn heap_dump_strongbox_primitive_array_raw(
        &self,
        index: usize,
    ) -> Result<Option<(usize, Vec<u8>)>> {
        let Some(target) = self.heap_dump_strongbox_primitive_array_node_id(index)? else {
            return Ok(None);
        };
        let (count, bpe) = primitive_array_shape(&self.doc, target)?;
        let mut raw = Vec::<u8>::with_capacity(count.saturating_mul(bpe));
        for i in 0..count {
            raw.extend_from_slice(self.doc.primitive_array_element(target, i)?);
        }
        Ok(Some((bpe, raw)))
    }

    pub fn heap_dump_strongbox_vector3(&self, index: usize) -> Result<Option<(f32, f32, f32)>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([x_node, y_node, z_node]) = find_vector3_component_node_ids(&self.doc, resolved)
        else {
            return Ok(None);
        };

        let x = read_f32_primitive(&self.doc, x_node)?;
        let y = read_f32_primitive(&self.doc, y_node)?;
        let z = read_f32_primitive(&self.doc, z_node)?;
        Ok(Some((x, y, z)))
    }

    pub fn heap_dump_strongbox_vector2(&self, index: usize) -> Result<Option<(f32, f32)>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([x_node, y_node]) = find_vector2_component_node_ids(&self.doc, resolved) else {
            return Ok(None);
        };

        let x = read_f32_primitive(&self.doc, x_node)?;
        let y = read_f32_primitive(&self.doc, y_node)?;
        Ok(Some((x, y)))
    }

    pub fn heap_dump_strongbox_quaternion(
        &self,
        index: usize,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([x_node, y_node, z_node, w_node]) =
            find_quaternion_component_node_ids(&self.doc, resolved)
        else {
            return Ok(None);
        };

        let x = read_f32_primitive(&self.doc, x_node)?;
        let y = read_f32_primitive(&self.doc, y_node)?;
        let z = read_f32_primitive(&self.doc, z_node)?;
        let w = read_f32_primitive(&self.doc, w_node)?;
        Ok(Some((x, y, z, w)))
    }

    pub fn heap_dump_strongbox_color(&self, index: usize) -> Result<Option<(f32, f32, f32, f32)>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([r_node, g_node, b_node, a_node]) =
            find_color_component_node_ids(&self.doc, resolved)
        else {
            return Ok(None);
        };

        let r = read_f32_primitive(&self.doc, r_node)?;
        let g = read_f32_primitive(&self.doc, g_node)?;
        let b = read_f32_primitive(&self.doc, b_node)?;
        let a = read_f32_primitive(&self.doc, a_node)?;
        Ok(Some((r, g, b, a)))
    }

    pub fn heap_dump_strongbox_serialization_result(
        &self,
        index: usize,
    ) -> Result<Option<(bool, i32)>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([success_node, byte_count_node]) =
            find_serialization_result_component_node_ids(&self.doc, resolved)
        else {
            return Ok(None);
        };
        let success = read_bool_primitive(&self.doc, success_node)?;
        let byte_count = read_i32_like_primitive(&self.doc, byte_count_node)?;
        Ok(Some((success, byte_count)))
    }

    pub fn heap_dump_strongbox_system_type_name(&self, index: usize) -> Result<Option<String>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        Ok(extract_system_type_name_from_node(&self.doc, resolved))
    }

    pub fn set_heap_dump_strongbox_vector3(
        &mut self,
        index: usize,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([x_node, y_node, z_node]) = find_vector3_component_node_ids(&self.doc, resolved)
        else {
            return Err(OdinError::new(format!(
                "HeapDump[{index}] cannot locate writable UnityEngine.Vector3 fields."
            )));
        };

        self.doc
            .set_primitive(x_node, PrimitiveValue::Float(x))
            .and_then(|_| self.doc.set_primitive(y_node, PrimitiveValue::Float(y)))
            .and_then(|_| self.doc.set_primitive(z_node, PrimitiveValue::Float(z)))
    }

    pub fn set_heap_dump_strongbox_vector2(&mut self, index: usize, x: f32, y: f32) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([x_node, y_node]) = find_vector2_component_node_ids(&self.doc, resolved) else {
            return Err(OdinError::new(format!(
                "HeapDump[{index}] cannot locate writable UnityEngine.Vector2 fields."
            )));
        };

        self.doc
            .set_primitive(x_node, PrimitiveValue::Float(x))
            .and_then(|_| self.doc.set_primitive(y_node, PrimitiveValue::Float(y)))
    }

    pub fn set_heap_dump_strongbox_quaternion(
        &mut self,
        index: usize,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([x_node, y_node, z_node, w_node]) =
            find_quaternion_component_node_ids(&self.doc, resolved)
        else {
            return Err(OdinError::new(format!(
                "HeapDump[{index}] cannot locate writable UnityEngine.Quaternion fields."
            )));
        };

        self.doc
            .set_primitive(x_node, PrimitiveValue::Float(x))
            .and_then(|_| self.doc.set_primitive(y_node, PrimitiveValue::Float(y)))
            .and_then(|_| self.doc.set_primitive(z_node, PrimitiveValue::Float(z)))
            .and_then(|_| self.doc.set_primitive(w_node, PrimitiveValue::Float(w)))
    }

    pub fn set_heap_dump_strongbox_color(
        &mut self,
        index: usize,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([r_node, g_node, b_node, a_node]) =
            find_color_component_node_ids(&self.doc, resolved)
        else {
            return Err(OdinError::new(format!(
                "HeapDump[{index}] cannot locate writable UnityEngine.Color fields."
            )));
        };

        self.doc
            .set_primitive(r_node, PrimitiveValue::Float(r))
            .and_then(|_| self.doc.set_primitive(g_node, PrimitiveValue::Float(g)))
            .and_then(|_| self.doc.set_primitive(b_node, PrimitiveValue::Float(b)))
            .and_then(|_| self.doc.set_primitive(a_node, PrimitiveValue::Float(a)))
    }

    pub fn set_heap_dump_strongbox_serialization_result(
        &mut self,
        index: usize,
        success: bool,
        byte_count: i32,
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some([success_node, byte_count_node]) =
            find_serialization_result_component_node_ids(&self.doc, resolved)
        else {
            return Err(OdinError::new(format!(
                "HeapDump[{index}] cannot locate writable VRC.Udon.Common.SerializationResult fields."
            )));
        };
        self.doc
            .set_primitive(success_node, PrimitiveValue::Boolean(success))
            .and_then(|_| set_i32_like_primitive(&mut self.doc, byte_count_node, byte_count))
    }

    pub fn set_heap_dump_strongbox_system_type_name(
        &mut self,
        index: usize,
        value: &str,
    ) -> Result<()> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        let Some(target) = find_system_type_storage_node(&self.doc, resolved) else {
            return Err(OdinError::new(format!(
                "HeapDump[{index}] cannot locate writable System.Type payload node."
            )));
        };

        let node_kind = self
            .doc
            .node(target)
            .ok_or_else(|| OdinError::new("Invalid System.Type payload node id."))?
            .kind()
            .clone();

        match node_kind {
            NodeKind::Primitive(PrimitiveValue::String(_)) => set_string_like_primitive(
                &mut self.doc,
                target,
                value.to_string(),
                TYPE_SYSTEM_TYPE,
            ),
            NodeKind::TypeNameMetadata { .. } => self
                .doc
                .set_type_name_metadata_name(target, OdinString::utf16(value.to_string())),
            NodeKind::TypeIdMetadata { type_id, .. } => {
                let mapped = self.doc.nodes().iter().find_map(|node| match node.kind() {
                    NodeKind::TypeNameMetadata {
                        type_id: candidate_id,
                        ..
                    } if *candidate_id == type_id => Some(node.id()),
                    _ => None,
                });
                let Some(mapped_node_id) = mapped else {
                    return Err(OdinError::new(format!(
                        "{} TypeID {} has no matching TypeName metadata entry.",
                        TYPE_SYSTEM_TYPE, type_id
                    )));
                };
                self.doc.set_type_name_metadata_name(
                    mapped_node_id,
                    OdinString::utf16(value.to_string()),
                )
            }
            _ => Err(OdinError::new(format!(
                "Unsupported System.Type payload node kind: {:?}.",
                node_kind
            ))),
        }
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

    pub fn symbol_type_name_string(
        &self,
        section: SymbolSection,
        index: usize,
    ) -> Result<Option<String>> {
        let type_node = self.symbol_type_node_id(section, index)?;
        let resolved = self
            .doc
            .resolve_node_payload(type_node)
            .unwrap_or(type_node);
        if let Some(value) = extract_type_name_from_type_payload(&self.doc, resolved) {
            return Ok(Some(value));
        }
        if let Some(value) = extract_type_name_from_node(&self.doc, resolved)
            && !is_runtime_type_name(value.as_str())
        {
            return Ok(Some(value));
        }
        if let Some(value) = extract_type_name_from_node(&self.doc, type_node)
            && !is_runtime_type_name(value.as_str())
        {
            return Ok(Some(value));
        }
        Ok(None)
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

    fn heap_dump_strongbox_primitive_array_node_id(&self, index: usize) -> Result<Option<NodeId>> {
        let value_node = self.heap_dump_strongbox_value_node_id(index)?;
        let resolved = self
            .doc
            .resolve_node_payload(value_node)
            .unwrap_or(value_node);
        if matches!(
            self.doc
                .node(resolved)
                .ok_or_else(|| OdinError::new("Invalid resolved strongbox value node id."))?
                .kind(),
            NodeKind::PrimitiveArray { .. }
        ) {
            return Ok(Some(resolved));
        }
        let Some(child) = first_child(&self.doc, resolved) else {
            return Ok(None);
        };
        if matches!(
            self.doc
                .node(child)
                .ok_or_else(|| OdinError::new("Invalid strongbox array child node id."))?
                .kind(),
            NodeKind::PrimitiveArray { .. }
        ) {
            Ok(Some(child))
        } else {
            Ok(None)
        }
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

fn read_f32_primitive(doc: &OdinDocument, node_id: NodeId) -> Result<f32> {
    let node = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
    match node.kind() {
        NodeKind::Primitive(PrimitiveValue::Float(v)) => Ok(*v),
        other => Err(OdinError::new(format!(
            "Node {} is not a Float primitive: {:?}.",
            node_id, other
        ))),
    }
}

fn read_bool_primitive(doc: &OdinDocument, node_id: NodeId) -> Result<bool> {
    let node = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
    match node.kind() {
        NodeKind::Primitive(PrimitiveValue::Boolean(v)) => Ok(*v),
        other => Err(OdinError::new(format!(
            "Node {} is not a Boolean primitive: {:?}.",
            node_id, other
        ))),
    }
}

fn read_i32_like_primitive(doc: &OdinDocument, node_id: NodeId) -> Result<i32> {
    let node = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
    let value = match node.kind() {
        NodeKind::Primitive(PrimitiveValue::SByte(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Byte(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Short(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::UShort(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Int(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::UInt(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Long(v)) => *v,
        NodeKind::Primitive(PrimitiveValue::ULong(v)) => i64::try_from(*v).map_err(|_| {
            OdinError::new(format!(
                "Node {} ULong value {} does not fit in i32.",
                node_id, v
            ))
        })?,
        other => {
            return Err(OdinError::new(format!(
                "Node {} is not an integer primitive: {:?}.",
                node_id, other
            )));
        }
    };

    i32::try_from(value).map_err(|_| {
        OdinError::new(format!(
            "Node {} integer value {} does not fit in i32.",
            node_id, value
        ))
    })
}

fn set_i32_like_primitive(doc: &mut OdinDocument, node_id: NodeId, value: i32) -> Result<()> {
    let target = doc
        .node(node_id)
        .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?
        .kind()
        .clone();
    let new_value = match target {
        NodeKind::Primitive(PrimitiveValue::SByte(_)) => PrimitiveValue::SByte(
            i8::try_from(value)
                .map_err(|_| OdinError::new(format!("Value {} does not fit in SByte.", value)))?,
        ),
        NodeKind::Primitive(PrimitiveValue::Byte(_)) => PrimitiveValue::Byte(
            u8::try_from(value)
                .map_err(|_| OdinError::new(format!("Value {} does not fit in Byte.", value)))?,
        ),
        NodeKind::Primitive(PrimitiveValue::Short(_)) => PrimitiveValue::Short(
            i16::try_from(value)
                .map_err(|_| OdinError::new(format!("Value {} does not fit in Int16.", value)))?,
        ),
        NodeKind::Primitive(PrimitiveValue::UShort(_)) => PrimitiveValue::UShort(
            u16::try_from(value)
                .map_err(|_| OdinError::new(format!("Value {} does not fit in UInt16.", value)))?,
        ),
        NodeKind::Primitive(PrimitiveValue::Int(_)) => PrimitiveValue::Int(value),
        NodeKind::Primitive(PrimitiveValue::UInt(_)) => PrimitiveValue::UInt(
            u32::try_from(value)
                .map_err(|_| OdinError::new(format!("Value {} does not fit in UInt32.", value)))?,
        ),
        NodeKind::Primitive(PrimitiveValue::Long(_)) => PrimitiveValue::Long(i64::from(value)),
        NodeKind::Primitive(PrimitiveValue::ULong(_)) => PrimitiveValue::ULong(
            u64::try_from(value)
                .map_err(|_| OdinError::new(format!("Value {} does not fit in UInt64.", value)))?,
        ),
        other => {
            return Err(OdinError::new(format!(
                "Node {} is not an integer primitive: {:?}.",
                node_id, other
            )));
        }
    };
    doc.set_primitive(node_id, new_value)
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

fn extract_system_type_name_from_node(doc: &OdinDocument, node_id: NodeId) -> Option<String> {
    let mut stack = vec![node_id];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    while let Some(current) = stack.pop() {
        let resolved = doc.resolve_node_payload(current).unwrap_or(current);
        if !seen.insert(resolved) {
            continue;
        }
        let node = doc.node(resolved)?;
        match node.kind() {
            NodeKind::TypeNameMetadata { name, .. } => return Some(name.value.clone()),
            NodeKind::TypeIdMetadata {
                resolved_name: Some(name),
                ..
            } => return Some(name.clone()),
            NodeKind::Primitive(PrimitiveValue::String(v)) => return Some(v.value.clone()),
            NodeKind::InternalReference(reference_id) => {
                if let Some(target) = doc.resolve_reference_id(*reference_id) {
                    stack.push(target);
                }
            }
            _ => {}
        }
        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    None
}

fn find_system_type_storage_node(doc: &OdinDocument, node_id: NodeId) -> Option<NodeId> {
    let mut stack = vec![node_id];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    while let Some(current) = stack.pop() {
        let resolved = doc.resolve_node_payload(current).unwrap_or(current);
        if !seen.insert(resolved) {
            continue;
        }
        let node = doc.node(resolved)?;
        match node.kind() {
            NodeKind::TypeNameMetadata { .. }
            | NodeKind::TypeIdMetadata { .. }
            | NodeKind::Primitive(PrimitiveValue::String(_)) => return Some(resolved),
            NodeKind::InternalReference(reference_id) => {
                if let Some(target) = doc.resolve_reference_id(*reference_id) {
                    stack.push(target);
                }
            }
            _ => {}
        }
        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    None
}

fn find_vector3_component_node_ids(doc: &OdinDocument, root: NodeId) -> Option<[NodeId; 3]> {
    find_float_components_by_names_or_fallback(doc, root, &["x", "y", "z"])
        .map(|ids| [ids[0], ids[1], ids[2]])
}

fn find_vector2_component_node_ids(doc: &OdinDocument, root: NodeId) -> Option<[NodeId; 2]> {
    find_float_components_by_names_or_fallback(doc, root, &["x", "y"]).map(|ids| [ids[0], ids[1]])
}

fn find_quaternion_component_node_ids(doc: &OdinDocument, root: NodeId) -> Option<[NodeId; 4]> {
    find_float_components_by_names_or_fallback(doc, root, &["x", "y", "z", "w"])
        .map(|ids| [ids[0], ids[1], ids[2], ids[3]])
}

fn find_color_component_node_ids(doc: &OdinDocument, root: NodeId) -> Option<[NodeId; 4]> {
    find_float_components_by_names_or_fallback(doc, root, &["r", "g", "b", "a"])
        .map(|ids| [ids[0], ids[1], ids[2], ids[3]])
}

fn find_serialization_result_component_node_ids(
    doc: &OdinDocument,
    root: NodeId,
) -> Option<[NodeId; 2]> {
    let success = find_named_bool_component_node(doc, root, "success").or_else(|| {
        first_node_matching(doc, root, &mut |kind| {
            matches!(kind, NodeKind::Primitive(PrimitiveValue::Boolean(_)))
        })
    })?;
    let byte_count = find_named_integer_component_node(doc, root, "byteCount")
        .or_else(|| find_named_integer_component_node(doc, root, "byte_count"))
        .or_else(|| {
            first_node_matching(doc, root, &mut |kind| {
                matches!(
                    kind,
                    NodeKind::Primitive(PrimitiveValue::SByte(_))
                        | NodeKind::Primitive(PrimitiveValue::Byte(_))
                        | NodeKind::Primitive(PrimitiveValue::Short(_))
                        | NodeKind::Primitive(PrimitiveValue::UShort(_))
                        | NodeKind::Primitive(PrimitiveValue::Int(_))
                        | NodeKind::Primitive(PrimitiveValue::UInt(_))
                        | NodeKind::Primitive(PrimitiveValue::Long(_))
                        | NodeKind::Primitive(PrimitiveValue::ULong(_))
                )
            })
        })?;
    Some([success, byte_count])
}

fn find_float_components_by_names_or_fallback(
    doc: &OdinDocument,
    root: NodeId,
    names: &[&str],
) -> Option<Vec<NodeId>> {
    let named = names
        .iter()
        .map(|name| find_named_float_component_node(doc, root, name))
        .collect::<Vec<_>>();
    if named.iter().all(Option::is_some) {
        return Some(
            named
                .into_iter()
                .map(|id| id.expect("checked Some"))
                .collect::<Vec<_>>(),
        );
    }

    let fallback = collect_float_component_nodes(doc, root, names.len());
    (fallback.len() == names.len()).then_some(fallback)
}

fn find_named_float_component_node(doc: &OdinDocument, root: NodeId, name: &str) -> Option<NodeId> {
    let mut stack = vec![root];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    while let Some(node_id) = stack.pop() {
        if !seen.insert(node_id) {
            continue;
        }
        let node = doc.node(node_id)?;
        if node.name().map(|x| x.value == name).unwrap_or(false)
            && let Some(id) = first_float_node(doc, node_id)
        {
            return Some(id);
        }

        if let NodeKind::InternalReference(reference_id) = node.kind()
            && let Some(target) = doc.resolve_reference_id(*reference_id)
        {
            stack.push(target);
        }

        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    None
}

fn find_named_bool_component_node(doc: &OdinDocument, root: NodeId, name: &str) -> Option<NodeId> {
    find_named_component_node(doc, root, name, |kind| {
        matches!(kind, NodeKind::Primitive(PrimitiveValue::Boolean(_)))
    })
}

fn find_named_integer_component_node(
    doc: &OdinDocument,
    root: NodeId,
    name: &str,
) -> Option<NodeId> {
    find_named_component_node(doc, root, name, |kind| {
        matches!(
            kind,
            NodeKind::Primitive(PrimitiveValue::SByte(_))
                | NodeKind::Primitive(PrimitiveValue::Byte(_))
                | NodeKind::Primitive(PrimitiveValue::Short(_))
                | NodeKind::Primitive(PrimitiveValue::UShort(_))
                | NodeKind::Primitive(PrimitiveValue::Int(_))
                | NodeKind::Primitive(PrimitiveValue::UInt(_))
                | NodeKind::Primitive(PrimitiveValue::Long(_))
                | NodeKind::Primitive(PrimitiveValue::ULong(_))
        )
    })
}

fn find_named_component_node<F>(
    doc: &OdinDocument,
    root: NodeId,
    name: &str,
    mut predicate: F,
) -> Option<NodeId>
where
    F: FnMut(&NodeKind) -> bool,
{
    let mut stack = vec![root];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    while let Some(node_id) = stack.pop() {
        if !seen.insert(node_id) {
            continue;
        }
        let node = doc.node(node_id)?;
        if node.name().map(|x| x.value == name).unwrap_or(false)
            && let Some(id) = first_node_matching(doc, node_id, &mut predicate)
        {
            return Some(id);
        }

        if let NodeKind::InternalReference(reference_id) = node.kind()
            && let Some(target) = doc.resolve_reference_id(*reference_id)
        {
            stack.push(target);
        }
        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    None
}

fn first_node_matching<F>(doc: &OdinDocument, root: NodeId, predicate: &mut F) -> Option<NodeId>
where
    F: FnMut(&NodeKind) -> bool,
{
    let mut stack = vec![root];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    while let Some(node_id) = stack.pop() {
        let resolved = doc.resolve_node_payload(node_id).unwrap_or(node_id);
        if !seen.insert(resolved) {
            continue;
        }
        let node = doc.node(resolved)?;
        if predicate(node.kind()) {
            return Some(resolved);
        }
        if let NodeKind::InternalReference(reference_id) = node.kind()
            && let Some(target) = doc.resolve_reference_id(*reference_id)
        {
            stack.push(target);
        }
        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    None
}

fn first_float_node(doc: &OdinDocument, root: NodeId) -> Option<NodeId> {
    let mut stack = vec![root];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    while let Some(node_id) = stack.pop() {
        let resolved = doc.resolve_node_payload(node_id).unwrap_or(node_id);
        if !seen.insert(resolved) {
            continue;
        }
        let node = doc.node(resolved)?;
        if matches!(node.kind(), NodeKind::Primitive(PrimitiveValue::Float(_))) {
            return Some(resolved);
        }
        if let NodeKind::InternalReference(reference_id) = node.kind()
            && let Some(target) = doc.resolve_reference_id(*reference_id)
        {
            stack.push(target);
        }
        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    None
}

fn collect_float_component_nodes(doc: &OdinDocument, root: NodeId, limit: usize) -> Vec<NodeId> {
    let mut out = Vec::<NodeId>::new();
    let mut stack = vec![root];
    let mut seen = std::collections::HashSet::<NodeId>::new();

    while let Some(node_id) = stack.pop() {
        let resolved = doc.resolve_node_payload(node_id).unwrap_or(node_id);
        if !seen.insert(resolved) {
            continue;
        }
        let Some(node) = doc.node(resolved) else {
            continue;
        };
        if matches!(node.kind(), NodeKind::Primitive(PrimitiveValue::Float(_))) {
            out.push(resolved);
            if out.len() >= limit {
                break;
            }
            continue;
        }
        if let NodeKind::InternalReference(reference_id) = node.kind()
            && let Some(target) = doc.resolve_reference_id(*reference_id)
        {
            stack.push(target);
        }
        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }
    out
}

fn is_runtime_type_name(type_name: &str) -> bool {
    type_name
        .split(',')
        .next()
        .map(str::trim)
        .map(|head| head == TYPE_SYSTEM_RUNTIME_TYPE)
        .unwrap_or(false)
}

fn type_name_candidate_rank(candidate: &str) -> u8 {
    if candidate.contains(',') {
        3
    } else if candidate.contains('.') || candidate.ends_with("[]") {
        2
    } else {
        1
    }
}

fn extract_type_name_from_type_payload(doc: &OdinDocument, root: NodeId) -> Option<String> {
    let mut stack = vec![root];
    let mut seen = std::collections::HashSet::<NodeId>::new();
    let mut best: Option<(u8, String)> = None;

    while let Some(node_id) = stack.pop() {
        if !seen.insert(node_id) {
            continue;
        }
        let Some(node) = doc.node(node_id) else {
            continue;
        };
        let maybe_candidate = match node.kind() {
            NodeKind::Primitive(PrimitiveValue::String(v)) => Some(v.value.as_str()),
            NodeKind::TypeNameMetadata { name, .. } => Some(name.value.as_str()),
            NodeKind::TypeIdMetadata {
                resolved_name: Some(name),
                ..
            } => Some(name.as_str()),
            NodeKind::InternalReference(reference_id) => {
                if let Some(target) = doc.resolve_reference_id(*reference_id) {
                    stack.push(target);
                }
                None
            }
            _ => None,
        };

        if let Some(candidate) = maybe_candidate {
            let rank = type_name_candidate_rank(candidate);
            match &best {
                Some((best_rank, _)) if *best_rank >= rank => {}
                _ => best = Some((rank, candidate.to_string())),
            }
        }

        for child in node.children().iter().copied().rev() {
            stack.push(child);
        }
    }

    best.map(|(_, value)| value)
}
