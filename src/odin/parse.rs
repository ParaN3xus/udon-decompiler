use std::collections::HashMap;

use crate::odin::io::ByteReader;
use crate::odin::model::{
    BinaryEntryType, NodeKind, OdinDocument, OdinError, OdinNode, OdinTypeRef, PrimitiveValue,
    Result, Token, TokenPayload,
};

#[derive(Debug, Clone)]
struct PendingNode {
    name: Option<crate::odin::model::OdinString>,
    consume_array_slot: bool,
    kind: NodeKind,
}

#[derive(Debug, Clone)]
struct ContainerFrame {
    node_id: usize,
    is_array: bool,
    next_array_index: usize,
}

impl OdinDocument {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(OdinError::new("Odin binary payload is empty."));
        }

        let mut reader = ByteReader::new(bytes);
        let mut type_id_to_name = HashMap::<i32, String>::new();
        let mut container_stack = Vec::<ContainerFrame>::new();
        let mut tokens = Vec::<Token>::new();
        let mut nodes = Vec::<OdinNode>::new();
        let mut root_nodes = Vec::<usize>::new();
        let mut saw_end_of_stream = false;

        while reader.has_remaining() {
            let token_start = reader.offset();
            let raw_type = reader.read_u8()?;
            let entry_type = BinaryEntryType::try_from(raw_type).map_err(|_| {
                OdinError::new(format!(
                    "Unknown entry type byte 0x{raw_type:02X} at offset {}.",
                    token_start
                ))
            })?;

            let token_index = tokens.len();
            let mut pending_node: Option<PendingNode> = None;
            let mut push_container: Option<bool> = None;
            let mut close_container = false;

            let payload = match entry_type {
                BinaryEntryType::NamedStartOfReferenceNode => {
                    let name = reader.read_string_value()?;
                    let type_ref = read_type_ref(&mut reader, &mut type_id_to_name)?;
                    let reference_id = reader.read_i32()?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::ReferenceNode {
                            type_ref: type_ref.clone(),
                            reference_id,
                        },
                    });
                    push_container = Some(false);
                    TokenPayload::StartReferenceNode {
                        name: Some(name),
                        type_ref,
                        reference_id,
                    }
                }
                BinaryEntryType::UnnamedStartOfReferenceNode => {
                    let type_ref = read_type_ref(&mut reader, &mut type_id_to_name)?;
                    let reference_id = reader.read_i32()?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::ReferenceNode {
                            type_ref: type_ref.clone(),
                            reference_id,
                        },
                    });
                    push_container = Some(false);
                    TokenPayload::StartReferenceNode {
                        name: None,
                        type_ref,
                        reference_id,
                    }
                }
                BinaryEntryType::NamedStartOfStructNode => {
                    let name = reader.read_string_value()?;
                    let type_ref = read_type_ref(&mut reader, &mut type_id_to_name)?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::StructNode {
                            type_ref: type_ref.clone(),
                        },
                    });
                    push_container = Some(false);
                    TokenPayload::StartStructNode {
                        name: Some(name),
                        type_ref,
                    }
                }
                BinaryEntryType::UnnamedStartOfStructNode => {
                    let type_ref = read_type_ref(&mut reader, &mut type_id_to_name)?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::StructNode {
                            type_ref: type_ref.clone(),
                        },
                    });
                    push_container = Some(false);
                    TokenPayload::StartStructNode {
                        name: None,
                        type_ref,
                    }
                }
                BinaryEntryType::EndOfNode => {
                    close_container = true;
                    TokenPayload::Empty
                }
                BinaryEntryType::StartOfArray => {
                    let declared_len = reader.read_i64()?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::Array { declared_len },
                    });
                    push_container = Some(true);
                    TokenPayload::StartArray { declared_len }
                }
                BinaryEntryType::EndOfArray => {
                    close_container = true;
                    TokenPayload::Empty
                }
                BinaryEntryType::PrimitiveArray => {
                    let element_count = reader.read_i32()?;
                    let bytes_per_element = reader.read_i32()?;
                    if element_count < 0 || bytes_per_element < 0 {
                        return Err(OdinError::new(format!(
                            "Primitive array has negative shape at offset {}.",
                            token_start
                        )));
                    }
                    let byte_count = (element_count as usize)
                        .checked_mul(bytes_per_element as usize)
                        .ok_or_else(|| {
                            OdinError::new(format!(
                                "Primitive array byte count overflow at offset {}.",
                                token_start
                            ))
                        })?;
                    let raw = reader.read_exact(byte_count)?.to_vec();
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::PrimitiveArray {
                            element_count,
                            bytes_per_element,
                        },
                    });
                    TokenPayload::PrimitiveArray {
                        element_count,
                        bytes_per_element,
                        raw,
                    }
                }
                BinaryEntryType::NamedInternalReference => {
                    let name = reader.read_string_value()?;
                    let value = reader.read_i32()?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::InternalReference(value),
                    });
                    TokenPayload::InternalReference {
                        name: Some(name),
                        value,
                    }
                }
                BinaryEntryType::UnnamedInternalReference => {
                    let value = reader.read_i32()?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::InternalReference(value),
                    });
                    TokenPayload::InternalReference { name: None, value }
                }
                BinaryEntryType::NamedExternalReferenceByIndex => {
                    let name = reader.read_string_value()?;
                    let value = reader.read_i32()?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::ExternalReferenceByIndex(value),
                    });
                    TokenPayload::ExternalReferenceByIndex {
                        name: Some(name),
                        value,
                    }
                }
                BinaryEntryType::UnnamedExternalReferenceByIndex => {
                    let value = reader.read_i32()?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::ExternalReferenceByIndex(value),
                    });
                    TokenPayload::ExternalReferenceByIndex { name: None, value }
                }
                BinaryEntryType::NamedExternalReferenceByGuid => {
                    let name = reader.read_string_value()?;
                    let value = reader.read_guid()?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::ExternalReferenceByGuid(value),
                    });
                    TokenPayload::ExternalReferenceByGuid {
                        name: Some(name),
                        value,
                    }
                }
                BinaryEntryType::UnnamedExternalReferenceByGuid => {
                    let value = reader.read_guid()?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::ExternalReferenceByGuid(value),
                    });
                    TokenPayload::ExternalReferenceByGuid { name: None, value }
                }
                BinaryEntryType::NamedExternalReferenceByString => {
                    let name = reader.read_string_value()?;
                    let value = reader.read_string_value()?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::ExternalReferenceByString(value.clone()),
                    });
                    TokenPayload::ExternalReferenceByString {
                        name: Some(name),
                        value,
                    }
                }
                BinaryEntryType::UnnamedExternalReferenceByString => {
                    let value = reader.read_string_value()?;
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::ExternalReferenceByString(value.clone()),
                    });
                    TokenPayload::ExternalReferenceByString { name: None, value }
                }
                BinaryEntryType::NamedSByte | BinaryEntryType::UnnamedSByte => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::SByte(reader.read_i8()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedByte | BinaryEntryType::UnnamedByte => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Byte(reader.read_u8()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedShort | BinaryEntryType::UnnamedShort => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Short(reader.read_i16()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedUShort | BinaryEntryType::UnnamedUShort => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::UShort(reader.read_u16()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedInt | BinaryEntryType::UnnamedInt => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Int(reader.read_i32()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedUInt | BinaryEntryType::UnnamedUInt => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::UInt(reader.read_u32()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedLong | BinaryEntryType::UnnamedLong => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Long(reader.read_i64()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedULong | BinaryEntryType::UnnamedULong => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::ULong(reader.read_u64()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedFloat | BinaryEntryType::UnnamedFloat => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Float(reader.read_f32()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedDouble | BinaryEntryType::UnnamedDouble => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Double(reader.read_f64()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedDecimal | BinaryEntryType::UnnamedDecimal => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Decimal(reader.read_decimal_bits()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedChar | BinaryEntryType::UnnamedChar => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Char(reader.read_u16()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedString | BinaryEntryType::UnnamedString => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::String(reader.read_string_value()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedGuid | BinaryEntryType::UnnamedGuid => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Guid(reader.read_guid()?);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedBoolean | BinaryEntryType::UnnamedBoolean => {
                    let name = read_optional_name(&mut reader, entry_type)?;
                    let value = PrimitiveValue::Boolean(reader.read_u8()? != 0);
                    pending_node = Some(PendingNode {
                        name: name.clone(),
                        consume_array_slot: true,
                        kind: NodeKind::Primitive(value.clone()),
                    });
                    TokenPayload::Primitive { name, value }
                }
                BinaryEntryType::NamedNull => {
                    let name = reader.read_string_value()?;
                    pending_node = Some(PendingNode {
                        name: Some(name.clone()),
                        consume_array_slot: true,
                        kind: NodeKind::Null,
                    });
                    TokenPayload::Null { name: Some(name) }
                }
                BinaryEntryType::UnnamedNull => {
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: true,
                        kind: NodeKind::Null,
                    });
                    TokenPayload::Null { name: None }
                }
                BinaryEntryType::TypeName => {
                    let type_id = reader.read_i32()?;
                    let name = reader.read_string_value()?;
                    type_id_to_name.insert(type_id, name.value.clone());
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: false,
                        kind: NodeKind::TypeNameMetadata {
                            type_id,
                            name: name.clone(),
                        },
                    });
                    TokenPayload::TypeName { type_id, name }
                }
                BinaryEntryType::TypeID => {
                    let type_id = reader.read_i32()?;
                    let resolved_name = type_id_to_name.get(&type_id).cloned();
                    pending_node = Some(PendingNode {
                        name: None,
                        consume_array_slot: false,
                        kind: NodeKind::TypeIdMetadata {
                            type_id,
                            resolved_name,
                        },
                    });
                    TokenPayload::TypeId { type_id }
                }
                BinaryEntryType::EndOfStream => {
                    saw_end_of_stream = true;
                    TokenPayload::Empty
                }
                BinaryEntryType::Invalid => {
                    return Err(OdinError::new(format!(
                        "Invalid entry type byte 0x00 at offset {}.",
                        token_start
                    )));
                }
            };

            let token_end = reader.offset();
            tokens.push(Token {
                entry_type,
                span: token_start..token_end,
                payload,
                dirty: false,
            });

            if let Some(pending) = pending_node {
                let parent = container_stack.last().map(|frame| frame.node_id);
                let array_index = if pending.consume_array_slot {
                    take_array_slot(&mut container_stack)
                } else {
                    None
                };
                let node_id = nodes.len();
                if let Some(parent_id) = parent {
                    nodes[parent_id].children.push(node_id);
                } else {
                    root_nodes.push(node_id);
                }
                nodes.push(OdinNode {
                    id: node_id,
                    parent,
                    children: Vec::new(),
                    name: pending.name,
                    array_index,
                    kind: pending.kind,
                    token_index,
                    closing_token_index: None,
                });
                if let Some(is_array) = push_container {
                    container_stack.push(ContainerFrame {
                        node_id,
                        is_array,
                        next_array_index: 0,
                    });
                }
            }

            if close_container {
                let closed = container_stack.pop().ok_or_else(|| {
                    OdinError::new(format!(
                        "Unexpected {:?} at offset {} with empty container stack.",
                        entry_type, token_start
                    ))
                })?;

                if matches!(entry_type, BinaryEntryType::EndOfArray) && !closed.is_array {
                    return Err(OdinError::new(format!(
                        "EndOfArray at offset {} closed a non-array container.",
                        token_start
                    )));
                }
                if matches!(entry_type, BinaryEntryType::EndOfNode) && closed.is_array {
                    return Err(OdinError::new(format!(
                        "EndOfNode at offset {} closed an array container.",
                        token_start
                    )));
                }
                nodes[closed.node_id].closing_token_index = Some(token_index);
            }

            if entry_type == BinaryEntryType::EndOfStream {
                break;
            }
        }

        if !container_stack.is_empty() {
            return Err(OdinError::new(
                "Odin stream ended but container stack is not empty.",
            ));
        }
        if saw_end_of_stream && reader.has_remaining() {
            return Err(OdinError::new(format!(
                "Trailing bytes found after EndOfStream at offset {}.",
                reader.offset()
            )));
        }

        Ok(Self {
            original_bytes: bytes.to_vec(),
            tokens,
            nodes,
            root_nodes,
        })
    }
}

fn take_array_slot(container_stack: &mut [ContainerFrame]) -> Option<usize> {
    if let Some(frame) = container_stack.last_mut()
        && frame.is_array
    {
        let index = frame.next_array_index;
        frame.next_array_index += 1;
        return Some(index);
    }
    None
}

fn read_optional_name(
    reader: &mut ByteReader<'_>,
    entry_type: BinaryEntryType,
) -> Result<Option<crate::odin::model::OdinString>> {
    if entry_type.is_named_entry() {
        Ok(Some(reader.read_string_value()?))
    } else {
        Ok(None)
    }
}

fn read_type_ref(
    reader: &mut ByteReader<'_>,
    map: &mut HashMap<i32, String>,
) -> Result<OdinTypeRef> {
    let offset = reader.offset();
    let raw = reader.read_u8()?;
    let entry = BinaryEntryType::try_from(raw).map_err(|_| {
        OdinError::new(format!(
            "Invalid type-ref entry 0x{raw:02X} at offset {}.",
            offset
        ))
    })?;
    match entry {
        BinaryEntryType::TypeID => {
            let id = reader.read_i32()?;
            Ok(OdinTypeRef::TypeId {
                id,
                resolved_name: map.get(&id).cloned(),
            })
        }
        BinaryEntryType::TypeName => {
            let id = reader.read_i32()?;
            let name = reader.read_string_value()?;
            map.insert(id, name.value.clone());
            Ok(OdinTypeRef::TypeName { id, name })
        }
        BinaryEntryType::UnnamedNull => Ok(OdinTypeRef::Null),
        _ => Err(OdinError::new(format!(
            "Expected TypeID/TypeName/UnnamedNull in type-ref but found {:?} at offset {}.",
            entry, offset
        ))),
    }
}
