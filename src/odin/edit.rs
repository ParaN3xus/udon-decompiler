use crate::odin::model::{
    NodeId, NodeKind, OdinDocument, OdinError, OdinGuid, OdinString, PrimitiveValue, Result,
    TokenPayload, primitive_kind_for_entry_type,
};

impl OdinDocument {
    pub fn primitive_array_element(&self, node_id: NodeId, element_index: usize) -> Result<&[u8]> {
        let node = self
            .nodes
            .get(node_id)
            .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
        let token_index = node.token_index;
        let (element_count, bytes_per_element) = match node.kind {
            NodeKind::PrimitiveArray {
                element_count,
                bytes_per_element,
            } => (element_count, bytes_per_element),
            _ => {
                return Err(OdinError::new(format!(
                    "Node {} is not a primitive array.",
                    node_id
                )));
            }
        };
        if bytes_per_element <= 0 || element_count <= 0 {
            return Err(OdinError::new(format!(
                "Primitive array node {} has no addressable elements.",
                node_id
            )));
        }
        let element_count_usize = element_count as usize;
        let bytes_per_element_usize = bytes_per_element as usize;
        if element_index >= element_count_usize {
            return Err(OdinError::new(format!(
                "Primitive array index {} is out of range (len={}).",
                element_index, element_count
            )));
        }
        match &self.tokens[token_index].payload {
            TokenPayload::PrimitiveArray { raw, .. } => {
                let start = element_index * bytes_per_element_usize;
                let end = start + bytes_per_element_usize;
                Ok(&raw[start..end])
            }
            _ => Err(OdinError::new(format!(
                "Token payload mismatch for primitive array node {}.",
                node_id
            ))),
        }
    }

    pub fn set_primitive(&mut self, node_id: NodeId, new_value: PrimitiveValue) -> Result<()> {
        let (token_index, expected_kind) = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match &node.kind {
                NodeKind::Primitive(current) => (node.token_index, current.kind()),
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not a primitive node.",
                        node_id
                    )));
                }
            }
        };
        if expected_kind != new_value.kind() {
            return Err(OdinError::new(format!(
                "Primitive kind mismatch on node {}: cannot replace {:?} with {:?}.",
                node_id,
                expected_kind,
                new_value.kind()
            )));
        }
        if primitive_kind_for_entry_type(self.tokens[token_index].entry_type) != Some(expected_kind)
        {
            return Err(OdinError::new(format!(
                "Token kind mismatch on node {}.",
                node_id
            )));
        }
        match &mut self.tokens[token_index].payload {
            TokenPayload::Primitive { value, .. } => *value = new_value.clone(),
            _ => {
                return Err(OdinError::new(format!(
                    "Token payload mismatch on primitive node {}.",
                    node_id
                )));
            }
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::Primitive(new_value);
        Ok(())
    }

    pub fn set_array_length(&mut self, node_id: NodeId, declared_len: i64) -> Result<()> {
        let token_index = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::Array { .. } => node.token_index,
                _ => return Err(OdinError::new(format!("Node {} is not an array.", node_id))),
            }
        };
        match &mut self.tokens[token_index].payload {
            TokenPayload::StartArray { declared_len: v } => *v = declared_len,
            _ => return Err(OdinError::new("Token payload mismatch for array node.")),
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::Array { declared_len };
        Ok(())
    }

    pub fn set_reference_node_id(&mut self, node_id: NodeId, reference_id: i32) -> Result<()> {
        let token_index = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::ReferenceNode { .. } => node.token_index,
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not a reference node.",
                        node_id
                    )));
                }
            }
        };
        match &mut self.tokens[token_index].payload {
            TokenPayload::StartReferenceNode {
                reference_id: id, ..
            } => *id = reference_id,
            _ => return Err(OdinError::new("Token payload mismatch for reference node.")),
        }
        self.tokens[token_index].dirty = true;
        if let NodeKind::ReferenceNode {
            reference_id: id, ..
        } = &mut self.nodes[node_id].kind
        {
            *id = reference_id;
            Ok(())
        } else {
            Err(OdinError::new("Node kind mismatch for reference node."))
        }
    }

    pub fn set_internal_reference(&mut self, node_id: NodeId, value: i32) -> Result<()> {
        let token_index = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::InternalReference(_) => node.token_index,
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not an internal reference node.",
                        node_id
                    )));
                }
            }
        };
        match &mut self.tokens[token_index].payload {
            TokenPayload::InternalReference { value: v, .. } => *v = value,
            _ => {
                return Err(OdinError::new(
                    "Token payload mismatch for internal reference node.",
                ));
            }
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::InternalReference(value);
        Ok(())
    }

    pub fn set_external_reference_by_index(&mut self, node_id: NodeId, value: i32) -> Result<()> {
        let token_index = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::ExternalReferenceByIndex(_) => node.token_index,
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not an external-index reference node.",
                        node_id
                    )));
                }
            }
        };
        match &mut self.tokens[token_index].payload {
            TokenPayload::ExternalReferenceByIndex { value: v, .. } => *v = value,
            _ => {
                return Err(OdinError::new(
                    "Token payload mismatch for external-index reference node.",
                ));
            }
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::ExternalReferenceByIndex(value);
        Ok(())
    }

    pub fn set_external_reference_by_guid(
        &mut self,
        node_id: NodeId,
        value: OdinGuid,
    ) -> Result<()> {
        let token_index = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::ExternalReferenceByGuid(_) => node.token_index,
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not an external-guid reference node.",
                        node_id
                    )));
                }
            }
        };
        match &mut self.tokens[token_index].payload {
            TokenPayload::ExternalReferenceByGuid { value: v, .. } => *v = value,
            _ => {
                return Err(OdinError::new(
                    "Token payload mismatch for external-guid reference node.",
                ));
            }
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::ExternalReferenceByGuid(value);
        Ok(())
    }

    pub fn set_external_reference_by_string(
        &mut self,
        node_id: NodeId,
        value: OdinString,
    ) -> Result<()> {
        let token_index = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::ExternalReferenceByString(_) => node.token_index,
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not an external-string reference node.",
                        node_id
                    )));
                }
            }
        };
        match &mut self.tokens[token_index].payload {
            TokenPayload::ExternalReferenceByString { value: v, .. } => *v = value.clone(),
            _ => {
                return Err(OdinError::new(
                    "Token payload mismatch for external-string reference node.",
                ));
            }
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::ExternalReferenceByString(value);
        Ok(())
    }

    pub fn set_primitive_array_element(
        &mut self,
        node_id: NodeId,
        element_index: usize,
        new_bytes: &[u8],
    ) -> Result<()> {
        let (token_index, element_count, bytes_per_element) = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::PrimitiveArray {
                    element_count,
                    bytes_per_element,
                } => (node.token_index, element_count, bytes_per_element),
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not a primitive array.",
                        node_id
                    )));
                }
            }
        };
        if bytes_per_element <= 0 || element_count <= 0 {
            return Err(OdinError::new(format!(
                "Primitive array node {} has no addressable elements.",
                node_id
            )));
        }
        let element_count_usize = element_count as usize;
        let bytes_per_element_usize = bytes_per_element as usize;
        if element_index >= element_count_usize {
            return Err(OdinError::new(format!(
                "Primitive array index {} is out of range (len={}).",
                element_index, element_count
            )));
        }
        if new_bytes.len() != bytes_per_element_usize {
            return Err(OdinError::new(format!(
                "Primitive array element expects {} bytes but got {}.",
                bytes_per_element,
                new_bytes.len()
            )));
        }
        let start = element_index * bytes_per_element_usize;
        let end = start + bytes_per_element_usize;
        match &mut self.tokens[token_index].payload {
            TokenPayload::PrimitiveArray { raw, .. } => raw[start..end].copy_from_slice(new_bytes),
            _ => {
                return Err(OdinError::new(
                    "Token payload mismatch for primitive array node.",
                ));
            }
        }
        self.tokens[token_index].dirty = true;
        Ok(())
    }

    pub fn replace_primitive_array(&mut self, node_id: NodeId, new_raw: &[u8]) -> Result<()> {
        let (token_index, bytes_per_element) = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
            match node.kind {
                NodeKind::PrimitiveArray {
                    bytes_per_element, ..
                } => (node.token_index, bytes_per_element),
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not a primitive array.",
                        node_id
                    )));
                }
            }
        };
        if bytes_per_element <= 0 {
            return Err(OdinError::new(format!(
                "Primitive array node {} has non-positive bytes_per_element={}.",
                node_id, bytes_per_element
            )));
        }
        let bytes_per_element_usize = bytes_per_element as usize;
        if new_raw.len() % bytes_per_element_usize != 0 {
            return Err(OdinError::new(format!(
                "Raw byte length {} is not divisible by bytes_per_element={}.",
                new_raw.len(),
                bytes_per_element
            )));
        }
        let element_count = i32::try_from(new_raw.len() / bytes_per_element_usize)
            .map_err(|_| OdinError::new("Primitive array element count exceeds i32."))?;
        match &mut self.tokens[token_index].payload {
            TokenPayload::PrimitiveArray {
                element_count: c,
                raw,
                ..
            } => {
                *c = element_count;
                raw.clear();
                raw.extend_from_slice(new_raw);
            }
            _ => {
                return Err(OdinError::new(
                    "Token payload mismatch for primitive array node.",
                ));
            }
        }
        self.tokens[token_index].dirty = true;
        self.nodes[node_id].kind = NodeKind::PrimitiveArray {
            element_count,
            bytes_per_element,
        };
        Ok(())
    }

    pub fn array_remove_at(&mut self, array_node_id: NodeId, element_index: usize) -> Result<()> {
        let element_node_ids = self.array_element_node_ids(array_node_id)?;
        if element_index >= element_node_ids.len() {
            return Err(OdinError::new(format!(
                "Array element index {} is out of range (len={}).",
                element_index,
                element_node_ids.len()
            )));
        }
        let target_node_id = element_node_ids[element_index];
        let (remove_start, remove_end) = self.node_token_range(target_node_id)?;
        self.tokens.drain(remove_start..remove_end);
        self.bump_array_declared_len(array_node_id, -1)?;
        self.reparse_after_structural_edit()
    }

    pub fn array_insert_clone(
        &mut self,
        array_node_id: NodeId,
        insert_index: usize,
        source_node_id: NodeId,
    ) -> Result<()> {
        let element_node_ids = self.array_element_node_ids(array_node_id)?;
        if insert_index > element_node_ids.len() {
            return Err(OdinError::new(format!(
                "Array insert index {} is out of range (len={}).",
                insert_index,
                element_node_ids.len()
            )));
        }
        let insert_token_index = if insert_index == element_node_ids.len() {
            self.array_end_token_index(array_node_id)?
        } else {
            let target_node = element_node_ids[insert_index];
            self.nodes[target_node].token_index
        };
        let (source_start, source_end) = self.node_token_range(source_node_id)?;
        let mut cloned_tokens = self.tokens[source_start..source_end].to_vec();
        for token in &mut cloned_tokens {
            token.dirty = true;
            token.span = 0..0;
        }
        self.tokens
            .splice(insert_token_index..insert_token_index, cloned_tokens);
        self.bump_array_declared_len(array_node_id, 1)?;
        self.reparse_after_structural_edit()
    }

    pub fn replace_node_with_clone(
        &mut self,
        target_node_id: NodeId,
        source_node_id: NodeId,
    ) -> Result<()> {
        let (target_start, target_end) = self.node_token_range(target_node_id)?;
        let (source_start, source_end) = self.node_token_range(source_node_id)?;
        let mut cloned_tokens = self.tokens[source_start..source_end].to_vec();
        for token in &mut cloned_tokens {
            token.dirty = true;
            token.span = 0..0;
        }
        self.tokens.splice(target_start..target_end, cloned_tokens);
        self.reparse_after_structural_edit()
    }

    fn reparse_after_structural_edit(&mut self) -> Result<()> {
        let rebuilt = self.to_bytes()?;
        *self = OdinDocument::parse(&rebuilt)?;
        Ok(())
    }

    fn array_element_node_ids(&self, array_node_id: NodeId) -> Result<Vec<NodeId>> {
        let array_node = self
            .nodes
            .get(array_node_id)
            .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", array_node_id)))?;
        if !matches!(array_node.kind, NodeKind::Array { .. }) {
            return Err(OdinError::new(format!(
                "Node {} is not a normal array.",
                array_node_id
            )));
        }
        let mut ids = array_node
            .children
            .iter()
            .copied()
            .filter(|id| self.nodes[*id].array_index.is_some())
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| self.nodes[*id].array_index.unwrap_or(usize::MAX));
        Ok(ids)
    }

    fn array_end_token_index(&self, array_node_id: NodeId) -> Result<usize> {
        let array_node = self
            .nodes
            .get(array_node_id)
            .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", array_node_id)))?;
        if !matches!(array_node.kind, NodeKind::Array { .. }) {
            return Err(OdinError::new(format!(
                "Node {} is not a normal array.",
                array_node_id
            )));
        }
        array_node.closing_token_index.ok_or_else(|| {
            OdinError::new(format!(
                "Array node {} has no EndOfArray token.",
                array_node_id
            ))
        })
    }

    fn node_token_range(&self, node_id: NodeId) -> Result<(usize, usize)> {
        let node = self
            .nodes
            .get(node_id)
            .ok_or_else(|| OdinError::new(format!("Node {} is out of range.", node_id)))?;
        let start = node.token_index;
        let end = node
            .closing_token_index
            .map_or(start + 1, |close| close + 1);
        Ok((start, end))
    }

    fn bump_array_declared_len(&mut self, array_node_id: NodeId, delta: i64) -> Result<()> {
        let (token_index, current_len) = {
            let node = self.nodes.get(array_node_id).ok_or_else(|| {
                OdinError::new(format!("Node {} is out of range.", array_node_id))
            })?;
            match node.kind {
                NodeKind::Array { declared_len } => (node.token_index, declared_len),
                _ => {
                    return Err(OdinError::new(format!(
                        "Node {} is not a normal array.",
                        array_node_id
                    )));
                }
            }
        };
        let new_len = current_len
            .checked_add(delta)
            .ok_or_else(|| OdinError::new("Array declared length overflow."))?;
        if new_len < 0 {
            return Err(OdinError::new("Array declared length cannot be negative."));
        }
        match &mut self.tokens[token_index].payload {
            TokenPayload::StartArray { declared_len } => *declared_len = new_len,
            _ => return Err(OdinError::new("Token payload mismatch for array node.")),
        }
        self.tokens[token_index].dirty = true;
        self.nodes[array_node_id].kind = NodeKind::Array {
            declared_len: new_len,
        };
        Ok(())
    }
}
