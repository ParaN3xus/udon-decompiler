use super::constants::*;
use super::{HeapLiteralValue, type_name_head};

macro_rules! heap_literal_defaults {
    ($($type_name:path => $literal:expr),+ $(,)?) => {
        pub(crate) fn default_heap_literal_for_type(type_name: &str) -> Option<HeapLiteralValue> {
            match type_name_head(type_name) {
                $($type_name => Some($literal),)+
                _ => None,
            }
        }
    };
}

heap_literal_defaults! {
    TYPE_SYSTEM_BOOLEAN => HeapLiteralValue::Bool(false),
    TYPE_SYSTEM_SBYTE => HeapLiteralValue::I8(0),
    TYPE_SYSTEM_BYTE => HeapLiteralValue::U8(0),
    TYPE_SYSTEM_INT16 => HeapLiteralValue::I16(0),
    TYPE_SYSTEM_UINT16 => HeapLiteralValue::U16(0),
    TYPE_SYSTEM_INT32 => HeapLiteralValue::I32(0),
    TYPE_SYSTEM_UINT32 => HeapLiteralValue::U32(0),
    TYPE_SYSTEM_INT64 => HeapLiteralValue::I64(0),
    TYPE_SYSTEM_UINT64 => HeapLiteralValue::U64(0),
    TYPE_SYSTEM_SINGLE => HeapLiteralValue::F32(0.0),
    TYPE_SYSTEM_DOUBLE => HeapLiteralValue::F64(0.0),
    TYPE_SYSTEM_STRING => HeapLiteralValue::Null,
    TYPE_SYSTEM_TYPE => HeapLiteralValue::Null,
    TYPE_VRC_SDKBASE_VRCURL => HeapLiteralValue::Null,
    TYPE_UNITY_VECTOR2 => HeapLiteralValue::Vector2(0.0, 0.0),
    TYPE_UNITY_VECTOR3 => HeapLiteralValue::Vector3(0.0, 0.0, 0.0),
    TYPE_UNITY_QUATERNION => HeapLiteralValue::Quaternion(0.0, 0.0, 0.0, 0.0),
    TYPE_UNITY_COLOR => HeapLiteralValue::Color(0.0, 0.0, 0.0, 0.0),
}

pub(crate) fn is_default_heap_literal(type_name: &str, literal: &HeapLiteralValue) -> bool {
    default_heap_literal_for_type(type_name)
        .as_ref()
        .is_some_and(|default| default == literal)
}
