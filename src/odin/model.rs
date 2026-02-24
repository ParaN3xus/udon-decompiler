use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::Range;

pub type Result<T> = std::result::Result<T, OdinError>;
pub type NodeId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdinError {
    message: String,
}

impl OdinError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for OdinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OdinError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OdinGuid(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    SingleByte,
    Utf16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdinString {
    pub value: String,
    pub encoding: StringEncoding,
}

impl OdinString {
    pub fn utf16(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            encoding: StringEncoding::Utf16,
        }
    }

    pub fn single_byte(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            encoding: StringEncoding::SingleByte,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OdinTypeRef {
    TypeId {
        id: i32,
        resolved_name: Option<String>,
    },
    TypeName {
        id: i32,
        name: OdinString,
    },
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimitiveKind {
    SByte,
    Byte,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    Decimal,
    Char,
    String,
    Guid,
    Boolean,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveValue {
    SByte(i8),
    Byte(u8),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Long(i64),
    ULong(u64),
    Float(f32),
    Double(f64),
    Decimal([u32; 4]),
    Char(u16),
    String(OdinString),
    Guid(OdinGuid),
    Boolean(bool),
}

impl PrimitiveValue {
    pub(crate) fn kind(&self) -> PrimitiveKind {
        match self {
            PrimitiveValue::SByte(_) => PrimitiveKind::SByte,
            PrimitiveValue::Byte(_) => PrimitiveKind::Byte,
            PrimitiveValue::Short(_) => PrimitiveKind::Short,
            PrimitiveValue::UShort(_) => PrimitiveKind::UShort,
            PrimitiveValue::Int(_) => PrimitiveKind::Int,
            PrimitiveValue::UInt(_) => PrimitiveKind::UInt,
            PrimitiveValue::Long(_) => PrimitiveKind::Long,
            PrimitiveValue::ULong(_) => PrimitiveKind::ULong,
            PrimitiveValue::Float(_) => PrimitiveKind::Float,
            PrimitiveValue::Double(_) => PrimitiveKind::Double,
            PrimitiveValue::Decimal(_) => PrimitiveKind::Decimal,
            PrimitiveValue::Char(_) => PrimitiveKind::Char,
            PrimitiveValue::String(_) => PrimitiveKind::String,
            PrimitiveValue::Guid(_) => PrimitiveKind::Guid,
            PrimitiveValue::Boolean(_) => PrimitiveKind::Boolean,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    ReferenceNode {
        type_ref: OdinTypeRef,
        reference_id: i32,
    },
    StructNode {
        type_ref: OdinTypeRef,
    },
    Array {
        declared_len: i64,
    },
    PrimitiveArray {
        element_count: i32,
        bytes_per_element: i32,
    },
    Primitive(PrimitiveValue),
    InternalReference(i32),
    ExternalReferenceByIndex(i32),
    ExternalReferenceByGuid(OdinGuid),
    ExternalReferenceByString(OdinString),
    Null,
    TypeNameMetadata {
        type_id: i32,
        name: OdinString,
    },
    TypeIdMetadata {
        type_id: i32,
        resolved_name: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct OdinNode {
    pub(crate) id: NodeId,
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) name: Option<OdinString>,
    pub(crate) array_index: Option<usize>,
    pub(crate) kind: NodeKind,
    pub(crate) token_index: usize,
    pub(crate) closing_token_index: Option<usize>,
}

impl OdinNode {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    pub fn name(&self) -> Option<&OdinString> {
        self.name.as_ref()
    }

    pub fn array_index(&self) -> Option<usize> {
        self.array_index
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn token_index(&self) -> usize {
        self.token_index
    }

    pub fn closing_token_index(&self) -> Option<usize> {
        self.closing_token_index
    }
}

#[derive(Debug, Clone)]
pub struct OdinDocument {
    pub(crate) original_bytes: Vec<u8>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) nodes: Vec<OdinNode>,
    pub(crate) root_nodes: Vec<NodeId>,
}

impl OdinDocument {
    pub fn root_nodes(&self) -> &[NodeId] {
        &self.root_nodes
    }

    pub fn nodes(&self) -> &[OdinNode] {
        &self.nodes
    }

    pub fn node(&self, id: NodeId) -> Option<&OdinNode> {
        self.nodes.get(id)
    }

    pub fn reference_id_index(&self) -> HashMap<i32, NodeId> {
        let mut out = HashMap::<i32, NodeId>::new();
        for node in &self.nodes {
            if let NodeKind::ReferenceNode { reference_id, .. } = node.kind() {
                out.entry(*reference_id).or_insert(node.id());
            }
        }
        out
    }

    pub fn resolve_reference_id(&self, reference_id: i32) -> Option<NodeId> {
        self.reference_id_index().get(&reference_id).copied()
    }

    pub fn resolve_internal_reference_node_id(&self, node_id: NodeId) -> Option<NodeId> {
        let node = self.node(node_id)?;
        match node.kind() {
            NodeKind::InternalReference(reference_id) => self.resolve_reference_id(*reference_id),
            _ => None,
        }
    }

    pub fn dereference_internal_reference_chain(&self, node_id: NodeId) -> Option<NodeId> {
        let mut current = node_id;
        let mut seen = HashSet::<NodeId>::new();
        loop {
            if !seen.insert(current) {
                return None;
            }
            let node = self.node(current)?;
            let next = match node.kind() {
                NodeKind::InternalReference(reference_id) => {
                    self.resolve_reference_id(*reference_id)
                }
                _ => None,
            };
            if let Some(next) = next {
                current = next;
                continue;
            }
            return Some(current);
        }
    }

    pub fn resolve_node_payload(&self, node_id: NodeId) -> Option<NodeId> {
        let mut current = self.dereference_internal_reference_chain(node_id)?;
        let mut seen = HashSet::<NodeId>::new();
        loop {
            if !seen.insert(current) {
                return None;
            }
            let node = self.node(current)?;
            match node.kind() {
                NodeKind::ReferenceNode { .. } => {
                    let child = node.children().first().copied()?;
                    current = self
                        .dereference_internal_reference_chain(child)
                        .unwrap_or(child);
                }
                _ => return Some(current),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) entry_type: BinaryEntryType,
    pub(crate) span: Range<usize>,
    pub(crate) payload: TokenPayload,
    pub(crate) dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenPayload {
    Empty,
    StartReferenceNode {
        name: Option<OdinString>,
        type_ref: OdinTypeRef,
        reference_id: i32,
    },
    StartStructNode {
        name: Option<OdinString>,
        type_ref: OdinTypeRef,
    },
    StartArray {
        declared_len: i64,
    },
    PrimitiveArray {
        element_count: i32,
        bytes_per_element: i32,
        raw: Vec<u8>,
    },
    InternalReference {
        name: Option<OdinString>,
        value: i32,
    },
    ExternalReferenceByIndex {
        name: Option<OdinString>,
        value: i32,
    },
    ExternalReferenceByGuid {
        name: Option<OdinString>,
        value: OdinGuid,
    },
    ExternalReferenceByString {
        name: Option<OdinString>,
        value: OdinString,
    },
    Primitive {
        name: Option<OdinString>,
        value: PrimitiveValue,
    },
    Null {
        name: Option<OdinString>,
    },
    TypeName {
        type_id: i32,
        name: OdinString,
    },
    TypeId {
        type_id: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum BinaryEntryType {
    Invalid = 0x00,
    NamedStartOfReferenceNode = 0x01,
    UnnamedStartOfReferenceNode = 0x02,
    NamedStartOfStructNode = 0x03,
    UnnamedStartOfStructNode = 0x04,
    EndOfNode = 0x05,
    StartOfArray = 0x06,
    EndOfArray = 0x07,
    PrimitiveArray = 0x08,
    NamedInternalReference = 0x09,
    UnnamedInternalReference = 0x0A,
    NamedExternalReferenceByIndex = 0x0B,
    UnnamedExternalReferenceByIndex = 0x0C,
    NamedExternalReferenceByGuid = 0x0D,
    UnnamedExternalReferenceByGuid = 0x0E,
    NamedSByte = 0x0F,
    UnnamedSByte = 0x10,
    NamedByte = 0x11,
    UnnamedByte = 0x12,
    NamedShort = 0x13,
    UnnamedShort = 0x14,
    NamedUShort = 0x15,
    UnnamedUShort = 0x16,
    NamedInt = 0x17,
    UnnamedInt = 0x18,
    NamedUInt = 0x19,
    UnnamedUInt = 0x1A,
    NamedLong = 0x1B,
    UnnamedLong = 0x1C,
    NamedULong = 0x1D,
    UnnamedULong = 0x1E,
    NamedFloat = 0x1F,
    UnnamedFloat = 0x20,
    NamedDouble = 0x21,
    UnnamedDouble = 0x22,
    NamedDecimal = 0x23,
    UnnamedDecimal = 0x24,
    NamedChar = 0x25,
    UnnamedChar = 0x26,
    NamedString = 0x27,
    UnnamedString = 0x28,
    NamedGuid = 0x29,
    UnnamedGuid = 0x2A,
    NamedBoolean = 0x2B,
    UnnamedBoolean = 0x2C,
    NamedNull = 0x2D,
    UnnamedNull = 0x2E,
    TypeName = 0x2F,
    TypeID = 0x30,
    EndOfStream = 0x31,
    NamedExternalReferenceByString = 0x32,
    UnnamedExternalReferenceByString = 0x33,
}

impl BinaryEntryType {
    pub(crate) fn is_named_entry(self) -> bool {
        matches!(
            self,
            BinaryEntryType::NamedStartOfReferenceNode
                | BinaryEntryType::NamedStartOfStructNode
                | BinaryEntryType::NamedInternalReference
                | BinaryEntryType::NamedExternalReferenceByIndex
                | BinaryEntryType::NamedExternalReferenceByGuid
                | BinaryEntryType::NamedExternalReferenceByString
                | BinaryEntryType::NamedSByte
                | BinaryEntryType::NamedByte
                | BinaryEntryType::NamedShort
                | BinaryEntryType::NamedUShort
                | BinaryEntryType::NamedInt
                | BinaryEntryType::NamedUInt
                | BinaryEntryType::NamedLong
                | BinaryEntryType::NamedULong
                | BinaryEntryType::NamedFloat
                | BinaryEntryType::NamedDouble
                | BinaryEntryType::NamedDecimal
                | BinaryEntryType::NamedChar
                | BinaryEntryType::NamedString
                | BinaryEntryType::NamedGuid
                | BinaryEntryType::NamedBoolean
                | BinaryEntryType::NamedNull
        )
    }
}

impl TryFrom<u8> for BinaryEntryType {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Invalid),
            0x01 => Ok(Self::NamedStartOfReferenceNode),
            0x02 => Ok(Self::UnnamedStartOfReferenceNode),
            0x03 => Ok(Self::NamedStartOfStructNode),
            0x04 => Ok(Self::UnnamedStartOfStructNode),
            0x05 => Ok(Self::EndOfNode),
            0x06 => Ok(Self::StartOfArray),
            0x07 => Ok(Self::EndOfArray),
            0x08 => Ok(Self::PrimitiveArray),
            0x09 => Ok(Self::NamedInternalReference),
            0x0A => Ok(Self::UnnamedInternalReference),
            0x0B => Ok(Self::NamedExternalReferenceByIndex),
            0x0C => Ok(Self::UnnamedExternalReferenceByIndex),
            0x0D => Ok(Self::NamedExternalReferenceByGuid),
            0x0E => Ok(Self::UnnamedExternalReferenceByGuid),
            0x0F => Ok(Self::NamedSByte),
            0x10 => Ok(Self::UnnamedSByte),
            0x11 => Ok(Self::NamedByte),
            0x12 => Ok(Self::UnnamedByte),
            0x13 => Ok(Self::NamedShort),
            0x14 => Ok(Self::UnnamedShort),
            0x15 => Ok(Self::NamedUShort),
            0x16 => Ok(Self::UnnamedUShort),
            0x17 => Ok(Self::NamedInt),
            0x18 => Ok(Self::UnnamedInt),
            0x19 => Ok(Self::NamedUInt),
            0x1A => Ok(Self::UnnamedUInt),
            0x1B => Ok(Self::NamedLong),
            0x1C => Ok(Self::UnnamedLong),
            0x1D => Ok(Self::NamedULong),
            0x1E => Ok(Self::UnnamedULong),
            0x1F => Ok(Self::NamedFloat),
            0x20 => Ok(Self::UnnamedFloat),
            0x21 => Ok(Self::NamedDouble),
            0x22 => Ok(Self::UnnamedDouble),
            0x23 => Ok(Self::NamedDecimal),
            0x24 => Ok(Self::UnnamedDecimal),
            0x25 => Ok(Self::NamedChar),
            0x26 => Ok(Self::UnnamedChar),
            0x27 => Ok(Self::NamedString),
            0x28 => Ok(Self::UnnamedString),
            0x29 => Ok(Self::NamedGuid),
            0x2A => Ok(Self::UnnamedGuid),
            0x2B => Ok(Self::NamedBoolean),
            0x2C => Ok(Self::UnnamedBoolean),
            0x2D => Ok(Self::NamedNull),
            0x2E => Ok(Self::UnnamedNull),
            0x2F => Ok(Self::TypeName),
            0x30 => Ok(Self::TypeID),
            0x31 => Ok(Self::EndOfStream),
            0x32 => Ok(Self::NamedExternalReferenceByString),
            0x33 => Ok(Self::UnnamedExternalReferenceByString),
            _ => Err(()),
        }
    }
}

pub(crate) fn primitive_kind_for_entry_type(entry_type: BinaryEntryType) -> Option<PrimitiveKind> {
    match entry_type {
        BinaryEntryType::NamedSByte | BinaryEntryType::UnnamedSByte => Some(PrimitiveKind::SByte),
        BinaryEntryType::NamedByte | BinaryEntryType::UnnamedByte => Some(PrimitiveKind::Byte),
        BinaryEntryType::NamedShort | BinaryEntryType::UnnamedShort => Some(PrimitiveKind::Short),
        BinaryEntryType::NamedUShort | BinaryEntryType::UnnamedUShort => {
            Some(PrimitiveKind::UShort)
        }
        BinaryEntryType::NamedInt | BinaryEntryType::UnnamedInt => Some(PrimitiveKind::Int),
        BinaryEntryType::NamedUInt | BinaryEntryType::UnnamedUInt => Some(PrimitiveKind::UInt),
        BinaryEntryType::NamedLong | BinaryEntryType::UnnamedLong => Some(PrimitiveKind::Long),
        BinaryEntryType::NamedULong | BinaryEntryType::UnnamedULong => Some(PrimitiveKind::ULong),
        BinaryEntryType::NamedFloat | BinaryEntryType::UnnamedFloat => Some(PrimitiveKind::Float),
        BinaryEntryType::NamedDouble | BinaryEntryType::UnnamedDouble => {
            Some(PrimitiveKind::Double)
        }
        BinaryEntryType::NamedDecimal | BinaryEntryType::UnnamedDecimal => {
            Some(PrimitiveKind::Decimal)
        }
        BinaryEntryType::NamedChar | BinaryEntryType::UnnamedChar => Some(PrimitiveKind::Char),
        BinaryEntryType::NamedString | BinaryEntryType::UnnamedString => {
            Some(PrimitiveKind::String)
        }
        BinaryEntryType::NamedGuid | BinaryEntryType::UnnamedGuid => Some(PrimitiveKind::Guid),
        BinaryEntryType::NamedBoolean | BinaryEntryType::UnnamedBoolean => {
            Some(PrimitiveKind::Boolean)
        }
        _ => None,
    }
}
