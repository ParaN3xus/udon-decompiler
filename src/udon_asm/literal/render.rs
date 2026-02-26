use super::constants::*;
use super::enum_map::{enum_repr, enum_value_to_name};
use super::{EnumRepr, HeapLiteralValue, type_name_head};

pub(crate) fn render_heap_literal(type_name: &str, literal: &HeapLiteralValue) -> String {
    if let Some(text) = render_enum_literal(type_name, literal) {
        return text;
    }
    match literal {
        HeapLiteralValue::Null => "null".to_string(),
        HeapLiteralValue::Bool(v) => v.to_string(),
        HeapLiteralValue::I8(v) => v.to_string(),
        HeapLiteralValue::U8(v) => v.to_string(),
        HeapLiteralValue::I16(v) => v.to_string(),
        HeapLiteralValue::U16(v) => v.to_string(),
        HeapLiteralValue::I32(v) => v.to_string(),
        HeapLiteralValue::U32(v) => v.to_string(),
        HeapLiteralValue::I64(v) => v.to_string(),
        HeapLiteralValue::U64(v) => v.to_string(),
        HeapLiteralValue::F32(v) => v.to_string(),
        HeapLiteralValue::F64(v) => v.to_string(),
        HeapLiteralValue::String(v) => format!("{v:?}"),
        HeapLiteralValue::SystemType(v) => render_system_type_literal(v),
        HeapLiteralValue::Vector2(x, y) => render_vector2_literal(*x, *y),
        HeapLiteralValue::Vector3(x, y, z) => render_vector3_literal(*x, *y, *z),
        HeapLiteralValue::Quaternion(x, y, z, w) => render_quaternion_literal(*x, *y, *z, *w),
        HeapLiteralValue::Color(r, g, b, a) => render_color_literal(*r, *g, *b, *a),
        HeapLiteralValue::SerializationResult {
            success,
            byte_count,
        } => render_serialization_result_literal(*success, *byte_count),
        HeapLiteralValue::TypedArray {
            element_type,
            elements,
        } => {
            let parts = elements
                .iter()
                .map(|value| render_heap_array_element_literal(Some(element_type.as_str()), value))
                .collect::<Vec<_>>();
            format!("[{}]", parts.join(", "))
        }
        HeapLiteralValue::OpaqueArray { len } => render_unserializable_array_literal(*len),
        HeapLiteralValue::U32Array(v) => {
            let parts = v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
            format!("[{}]", parts.join(", "))
        }
        HeapLiteralValue::Unserializable => UNSERIALIZABLE_LITERAL.to_string(),
    }
}

fn render_enum_literal(type_name: &str, literal: &HeapLiteralValue) -> Option<String> {
    let head = type_name_head(type_name);
    let repr = enum_repr(head)?;
    let value = enum_value_from_literal(repr, literal)?;
    Some(enum_value_to_name(head, value).unwrap_or_else(|| value.to_string()))
}

fn enum_value_from_literal(repr: EnumRepr, literal: &HeapLiteralValue) -> Option<i64> {
    match (repr, literal) {
        (EnumRepr::U8, HeapLiteralValue::U8(v)) => Some(i64::from(*v)),
        (EnumRepr::U8, HeapLiteralValue::I32(v)) => {
            if (0..=u8::MAX as i32).contains(v) {
                Some(i64::from(*v))
            } else {
                None
            }
        }
        (EnumRepr::U8, HeapLiteralValue::U32(v)) => u8::try_from(*v).ok().map(i64::from),
        (EnumRepr::I32, HeapLiteralValue::I8(v)) => Some(i64::from(*v)),
        (EnumRepr::I32, HeapLiteralValue::U8(v)) => Some(i64::from(*v)),
        (EnumRepr::I32, HeapLiteralValue::I16(v)) => Some(i64::from(*v)),
        (EnumRepr::I32, HeapLiteralValue::U16(v)) => Some(i64::from(*v)),
        (EnumRepr::I32, HeapLiteralValue::I32(v)) => Some(i64::from(*v)),
        (EnumRepr::I32, HeapLiteralValue::U32(v)) => i32::try_from(*v).ok().map(i64::from),
        _ => None,
    }
}

fn render_heap_array_element_literal(type_name: Option<&str>, value: &HeapLiteralValue) -> String {
    if let Some(type_name) = type_name
        && let Some(text) = render_enum_literal(type_name, value)
    {
        return text;
    }
    match value {
        HeapLiteralValue::Null => "null".to_string(),
        HeapLiteralValue::Bool(v) => v.to_string(),
        HeapLiteralValue::I8(v) => v.to_string(),
        HeapLiteralValue::U8(v) => v.to_string(),
        HeapLiteralValue::I16(v) => v.to_string(),
        HeapLiteralValue::U16(v) => v.to_string(),
        HeapLiteralValue::I32(v) => v.to_string(),
        HeapLiteralValue::U32(v) => v.to_string(),
        HeapLiteralValue::I64(v) => v.to_string(),
        HeapLiteralValue::U64(v) => v.to_string(),
        HeapLiteralValue::F32(v) => v.to_string(),
        HeapLiteralValue::F64(v) => v.to_string(),
        HeapLiteralValue::String(v) => format!("{v:?}"),
        HeapLiteralValue::SystemType(v) => render_system_type_literal(v),
        HeapLiteralValue::Vector2(x, y) => render_vector2_literal(*x, *y),
        HeapLiteralValue::Vector3(x, y, z) => render_vector3_literal(*x, *y, *z),
        HeapLiteralValue::Quaternion(x, y, z, w) => render_quaternion_literal(*x, *y, *z, *w),
        HeapLiteralValue::Color(r, g, b, a) => render_color_literal(*r, *g, *b, *a),
        HeapLiteralValue::SerializationResult {
            success,
            byte_count,
        } => render_serialization_result_literal(*success, *byte_count),
        HeapLiteralValue::TypedArray {
            element_type,
            elements,
        } => {
            let parts = elements
                .iter()
                .map(|item| render_heap_array_element_literal(Some(element_type.as_str()), item))
                .collect::<Vec<_>>();
            format!("[{}]", parts.join(", "))
        }
        HeapLiteralValue::OpaqueArray { .. } => UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL.to_string(),
        HeapLiteralValue::U32Array(v) => {
            let parts = v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
            format!("[{}]", parts.join(", "))
        }
        HeapLiteralValue::Unserializable => UNSERIALIZABLE_LITERAL.to_string(),
    }
}

fn render_unserializable_array_literal(len: usize) -> String {
    if len == 0 {
        return "[]".to_string();
    }
    let parts = std::iter::repeat_n(UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL, len).collect::<Vec<_>>();
    format!("[{}]", parts.join(", "))
}

fn render_vector3_literal(x: f32, y: f32, z: f32) -> String {
    format!(
        "new {}({}, {}, {})",
        TYPE_UNITY_VECTOR3,
        render_f32_component(x),
        render_f32_component(y),
        render_f32_component(z)
    )
}

fn render_vector2_literal(x: f32, y: f32) -> String {
    format!(
        "new {}({}, {})",
        TYPE_UNITY_VECTOR2,
        render_f32_component(x),
        render_f32_component(y)
    )
}

fn render_quaternion_literal(x: f32, y: f32, z: f32, w: f32) -> String {
    format!(
        "new {}({}, {}, {}, {})",
        TYPE_UNITY_QUATERNION,
        render_f32_component(x),
        render_f32_component(y),
        render_f32_component(z),
        render_f32_component(w)
    )
}

fn render_color_literal(r: f32, g: f32, b: f32, a: f32) -> String {
    format!(
        "new {}({}, {}, {}, {})",
        TYPE_UNITY_COLOR,
        render_f32_component(r),
        render_f32_component(g),
        render_f32_component(b),
        render_f32_component(a)
    )
}

fn render_serialization_result_literal(success: bool, byte_count: i32) -> String {
    format!(
        "new {} {{ success = {}, byteCount = {} }}",
        TYPE_VRC_SERIALIZATION_RESULT,
        if success { "true" } else { "false" },
        byte_count
    )
}

fn render_system_type_literal(text: &str) -> String {
    let value = text.trim();
    if value.is_empty() {
        return "\"\"".to_string();
    }
    let (type_name, assembly_name) = split_assembly_qualified_type(value);
    format!("typeof({type_name}) /* {assembly_name} */")
}

fn split_assembly_qualified_type(value: &str) -> (&str, &str) {
    let mut bracket_depth = 0usize;
    for (idx, ch) in value.char_indices() {
        match ch {
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if bracket_depth == 0 => {
                let type_name = value[..idx].trim();
                let assembly_name = value[idx + 1..].trim();
                return (type_name, assembly_name);
            }
            _ => {}
        }
    }
    panic!("Invalid type {}", value);
}

fn render_f32_component(value: f32) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}f")
    } else {
        format!("{value}f")
    }
}
