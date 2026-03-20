mod constants;
mod enum_map;
mod parse;
mod render;

pub(crate) use constants::*;
pub(crate) use enum_map::EnumRepr;
use enum_map::enum_repr;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum HeapLiteralValue {
    Null,
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    SystemType(String),
    Vector2(f32, f32),
    Vector3(f32, f32, f32),
    Quaternion(f32, f32, f32, f32),
    Color(f32, f32, f32, f32),
    SerializationResult {
        success: bool,
        byte_count: i32,
    },
    TypedArray {
        element_type: String,
        elements: Vec<HeapLiteralValue>,
    },
    OpaqueArray {
        len: usize,
    },
    U32Array(Vec<u32>),
    Unserializable,
}

impl HeapLiteralValue {
    pub(crate) fn as_u32(&self) -> Option<u32> {
        match self {
            Self::U32(v) => Some(*v),
            _ => None,
        }
    }

    pub(crate) fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    pub(crate) fn as_u32_array(&self) -> Option<&[u32]> {
        match self {
            Self::U32Array(v) => Some(v.as_slice()),
            _ => None,
        }
    }
}

pub(crate) fn type_name_head(type_name: &str) -> &str {
    type_name.split(',').next().unwrap_or(type_name).trim()
}

pub(crate) fn enum_repr_for_type(type_name: &str) -> Option<EnumRepr> {
    enum_repr(type_name_head(type_name))
}

pub(crate) use parse::{
    literal_from_typed_odin_node, parse_heap_init_directive, parse_type_ref,
    resolve_heap_literal_for_program_entry,
};
pub(crate) use render::render_heap_literal;
