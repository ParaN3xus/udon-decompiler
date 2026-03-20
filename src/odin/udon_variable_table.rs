use base64::Engine as _;

use crate::odin::{NodeId, NodeKind, OdinDocument, OdinError, PrimitiveValue, Result};
use crate::str_constants::TYPE_UNSERIALIZABLE;
use crate::udon_asm::{literal_from_typed_odin_node, render_heap_literal};

#[derive(Debug, Clone, PartialEq)]
pub struct VariableItem {
    pub symbol_name: String,
    pub value_kind: NodeKind,
}

#[derive(Debug, Clone)]
pub struct UdonVariableTableBinary {
    doc: OdinDocument,
}

impl UdonVariableTableBinary {
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

    pub fn variables_len(&self) -> Result<usize> {
        Ok(self.variable_node_ids()?.len())
    }

    pub fn variable_symbols(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        for index in 0..self.variables_len()? {
            out.push(self.variable_symbol_name(index)?);
        }
        Ok(out)
    }

    pub fn find_variable_index(&self, symbol_name: &str) -> Result<Option<usize>> {
        let symbols = self.variable_symbols()?;
        Ok(symbols.iter().position(|x| x == symbol_name))
    }

    pub fn variable_item(&self, index: usize) -> Result<VariableItem> {
        let payload = self.variable_payload_array_node_id(index)?;
        let symbol_node = named_child(&self.doc, payload, "SymbolName")
            .ok_or_else(|| OdinError::new(format!("Variables[{index}] missing SymbolName.")))?;
        let value_node = named_child(&self.doc, payload, "Value")
            .ok_or_else(|| OdinError::new(format!("Variables[{index}] missing Value.")))?;
        Ok(VariableItem {
            symbol_name: read_string_primitive(&self.doc, symbol_node, "SymbolName")?,
            value_kind: self
                .doc
                .node(value_node)
                .ok_or_else(|| OdinError::new("Invalid value node id."))?
                .kind()
                .clone(),
        })
    }

    pub fn variable_symbol_name(&self, index: usize) -> Result<String> {
        let payload = self.variable_payload_array_node_id(index)?;
        let symbol_node = named_child(&self.doc, payload, "SymbolName")
            .ok_or_else(|| OdinError::new(format!("Variables[{index}] missing SymbolName.")))?;
        read_string_primitive(&self.doc, symbol_node, "SymbolName")
    }

    pub fn set_variable_symbol_name_by_index(
        &mut self,
        index: usize,
        new_symbol_name: impl Into<String>,
    ) -> Result<()> {
        let payload = self.variable_payload_array_node_id(index)?;
        let symbol_node = named_child(&self.doc, payload, "SymbolName")
            .ok_or_else(|| OdinError::new(format!("Variables[{index}] missing SymbolName.")))?;
        set_string_primitive(
            &mut self.doc,
            symbol_node,
            new_symbol_name.into(),
            "SymbolName",
        )
    }

    pub fn set_variable_symbol_name(
        &mut self,
        symbol_name: &str,
        new_symbol_name: impl Into<String>,
    ) -> Result<()> {
        let index = self.find_variable_index(symbol_name)?.ok_or_else(|| {
            OdinError::new(format!("Variable symbol '{}' was not found.", symbol_name))
        })?;
        self.set_variable_symbol_name_by_index(index, new_symbol_name)
    }

    pub fn set_variable_value_primitive_by_index(
        &mut self,
        index: usize,
        new_value: PrimitiveValue,
    ) -> Result<()> {
        let value_node = self.variable_value_node_id(index)?;
        self.doc.set_primitive(value_node, new_value)
    }

    pub fn set_variable_value_primitive(
        &mut self,
        symbol_name: &str,
        new_value: PrimitiveValue,
    ) -> Result<()> {
        let index = self.find_variable_index(symbol_name)?.ok_or_else(|| {
            OdinError::new(format!("Variable symbol '{}' was not found.", symbol_name))
        })?;
        self.set_variable_value_primitive_by_index(index, new_value)
    }

    pub fn replace_variable_value_from_index(
        &mut self,
        target_index: usize,
        source_index: usize,
    ) -> Result<()> {
        let target_value = self.variable_value_node_id(target_index)?;
        let source_value = self.variable_value_node_id(source_index)?;
        self.doc.replace_node_with_clone(target_value, source_value)
    }

    pub fn replace_variable_value_from_symbol(
        &mut self,
        target_symbol: &str,
        source_symbol: &str,
    ) -> Result<()> {
        let target_index = self.find_variable_index(target_symbol)?.ok_or_else(|| {
            OdinError::new(format!(
                "Variable symbol '{}' was not found.",
                target_symbol
            ))
        })?;
        let source_index = self.find_variable_index(source_symbol)?.ok_or_else(|| {
            OdinError::new(format!(
                "Variable symbol '{}' was not found.",
                source_symbol
            ))
        })?;
        self.replace_variable_value_from_index(target_index, source_index)
    }

    pub fn insert_variable_clone_by_index(
        &mut self,
        insert_index: usize,
        template_index: usize,
        new_symbol_name: impl Into<String>,
    ) -> Result<()> {
        let variables_array = self.variables_array_node_id()?;
        let template_node = self.variable_node_id(template_index)?;
        self.doc
            .array_insert_clone(variables_array, insert_index, template_node)?;
        self.set_variable_symbol_name_by_index(insert_index, new_symbol_name)
    }

    pub fn insert_variable_clone_by_symbol(
        &mut self,
        insert_index: usize,
        template_symbol: &str,
        new_symbol_name: impl Into<String>,
    ) -> Result<()> {
        let template_index = self.find_variable_index(template_symbol)?.ok_or_else(|| {
            OdinError::new(format!(
                "Template symbol '{}' was not found.",
                template_symbol
            ))
        })?;
        self.insert_variable_clone_by_index(insert_index, template_index, new_symbol_name)
    }

    pub fn append_variable_clone_by_symbol(
        &mut self,
        template_symbol: &str,
        new_symbol_name: impl Into<String>,
    ) -> Result<()> {
        let len = self.variables_len()?;
        self.insert_variable_clone_by_symbol(len, template_symbol, new_symbol_name)
    }

    pub fn render_public_variables_text(&self) -> Result<String> {
        let mut lines = Vec::new();
        for index in 0..self.variables_len()? {
            let symbol_name = self.variable_symbol_name(index)?;
            let (type_name, init_text) = self.variable_value_text(index)?;
            lines.push(format!("+ {symbol_name}: {{ {type_name} }} = {init_text}"));
        }
        Ok(lines.join("\n"))
    }

    fn variables_array_node_id(&self) -> Result<NodeId> {
        let root = find_variable_table_root(&self.doc)?;
        let variables_field = named_child(&self.doc, root, "Variables")
            .ok_or_else(|| OdinError::new("Variables field was not found."))?;
        first_child(&self.doc, variables_field)
            .ok_or_else(|| OdinError::new("Variables field has no payload array."))
    }

    fn variable_node_ids(&self) -> Result<Vec<NodeId>> {
        array_element_nodes(&self.doc, self.variables_array_node_id()?)
    }

    fn variable_node_id(&self, index: usize) -> Result<NodeId> {
        let ids = self.variable_node_ids()?;
        ids.get(index)
            .copied()
            .ok_or_else(|| OdinError::new(format!("Variables index {} is out of range.", index)))
    }

    fn variable_payload_array_node_id(&self, index: usize) -> Result<NodeId> {
        let variable_node = self.variable_node_id(index)?;
        first_child(&self.doc, variable_node).ok_or_else(|| {
            OdinError::new(format!(
                "Variables[{index}] has no payload array child node."
            ))
        })
    }

    fn variable_value_node_id(&self, index: usize) -> Result<NodeId> {
        let payload = self.variable_payload_array_node_id(index)?;
        named_child(&self.doc, payload, "Value")
            .ok_or_else(|| OdinError::new(format!("Variables[{index}] missing Value.")))
    }

    fn variable_declared_type_node_id(&self, index: usize) -> Result<NodeId> {
        let payload = self.variable_payload_array_node_id(index)?;
        variable_declared_type_node_id_from_layout(&self.doc, payload).ok_or_else(|| {
            OdinError::new(format!(
                "Variables[{index}] missing type metadata before Value."
            ))
        })
    }

    fn variable_value_text(&self, index: usize) -> Result<(String, String)> {
        let value_node = self.variable_value_node_id(index)?;
        let declared_type_node = self.variable_declared_type_node_id(index)?;
        let declared_type_resolved = self
            .doc
            .resolve_node_payload(declared_type_node)
            .unwrap_or(declared_type_node);
        let type_name = extract_type_name_from_node(&self.doc, declared_type_node)
            .or_else(|| extract_type_name_from_node(&self.doc, declared_type_resolved))
            .unwrap_or_else(|| TYPE_UNSERIALIZABLE.to_string());
        let literal = literal_from_typed_odin_node(&self.doc, value_node, &type_name);
        Ok((type_name.clone(), render_heap_literal(&type_name, &literal)))
    }
}

fn find_variable_table_root(doc: &OdinDocument) -> Result<NodeId> {
    doc.nodes()
        .iter()
        .map(|node| node.id())
        .find(|id| named_child(doc, *id, "Variables").is_some())
        .ok_or_else(|| OdinError::new("Unable to locate UdonVariableTable root node."))
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

fn set_string_primitive(
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
        PrimitiveValue::String(crate::odin::OdinString { value, encoding }),
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

fn variable_declared_type_node_id_from_layout(
    doc: &OdinDocument,
    payload: NodeId,
) -> Option<NodeId> {
    let value_node = named_child(doc, payload, "Value")?;
    let value_index = doc.node(value_node)?.array_index()?;
    let mut candidates = doc
        .node(payload)?
        .children()
        .iter()
        .copied()
        .filter(|id| {
            doc.node(*id)
                .and_then(|node| node.name())
                .is_some_and(|name| name.value == "type")
                && doc
                    .node(*id)
                    .and_then(|node| node.array_index())
                    .is_some_and(|idx| idx < value_index)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|id| doc.node(*id).and_then(|node| node.array_index()));
    candidates.pop()
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
