use crate::odin::{NodeId, NodeKind, OdinDocument, PrimitiveValue, UdonProgramBinary};
use crate::udon_asm::types::{AsmError, Result, TypeRefDirective};

use super::constants::*;
use super::enum_map::{enum_name_to_value, enum_repr};
use super::render::render_heap_literal;
use super::{EnumRepr, HeapLiteralValue, type_name_head};

pub(crate) fn parse_type_ref(text: &str, line_num: usize) -> Result<TypeRefDirective> {
    let text = text.trim();
    if text.starts_with('{') && text.ends_with('}') && text.len() >= 2 {
        Ok(TypeRefDirective::Name(
            text[1..text.len() - 1].trim().to_string(),
        ))
    } else {
        Err(AsmError::new(format!(
            "Line {}: type ref must be '{{ <type-name> }}'.",
            line_num
        )))
    }
}

pub(crate) fn parse_heap_init_directive(
    text: &str,
    line_num: usize,
    type_name: &str,
) -> Result<HeapLiteralValue> {
    let trimmed = text.trim();
    if trimmed == UNSERIALIZABLE_LITERAL {
        Ok(HeapLiteralValue::Unserializable)
    } else {
        parse_typed_heap_literal(type_name, trimmed, line_num)
    }
}

/// typename + str -> HeapLiteralValue
fn parse_typed_heap_literal(
    type_name: &str,
    trimmed: &str,
    line_num: usize,
) -> Result<HeapLiteralValue> {
    if trimmed == "null" {
        return Ok(HeapLiteralValue::Null);
    }
    let head = type_name_head(type_name);

    // arr
    if let Some(element_type) = head.strip_suffix("[]")
        // u32 arr is usually addr, we process that seperately
        && head != TYPE_SYSTEM_UINT32_ARRAY
    {
        let parts = parse_array_items(trimmed, line_num)?;
        if !is_supported_typed_scalar_or_enum(element_type.trim()) {
            if parts
                .iter()
                .all(|part| is_unserializable_placeholder_token(part.as_str()))
            {
                return Ok(HeapLiteralValue::OpaqueArray { len: parts.len() });
            }
            return Err(AsmError::new(format!(
                "Line {}: unsupported array element type '{}' in '{}'. Keep placeholders like '{}' or use '{}'.",
                line_num,
                element_type,
                type_name,
                UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL,
                UNSERIALIZABLE_LITERAL
            )));
        }
        let mut elements = Vec::<HeapLiteralValue>::with_capacity(parts.len());
        for item in parts {
            elements.push(parse_typed_heap_literal(
                element_type.trim(),
                item.as_str(),
                line_num,
            )?);
        }
        return Ok(HeapLiteralValue::TypedArray {
            element_type: element_type.trim().to_string(),
            elements,
        });
    }
    // enum
    if let Some(repr) = enum_repr(head) {
        return parse_typed_enum_literal(head, repr, trimmed, line_num);
    }
    // others
    match head {
        TYPE_SYSTEM_BOOLEAN => match trimmed {
            "true" => Ok(HeapLiteralValue::Bool(true)),
            "false" => Ok(HeapLiteralValue::Bool(false)),
            _ => Err(AsmError::new(format!(
                "Line {}: bool init must be true/false, got '{}'.",
                line_num, trimmed
            ))),
        },
        TYPE_SYSTEM_SBYTE => trimmed
            .parse::<i8>()
            .map(HeapLiteralValue::I8)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i8 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_BYTE => trimmed
            .parse::<u8>()
            .map(HeapLiteralValue::U8)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u8 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_INT16 => trimmed
            .parse::<i16>()
            .map(HeapLiteralValue::I16)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i16 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_UINT16 => trimmed
            .parse::<u16>()
            .map(HeapLiteralValue::U16)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u16 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_INT32 => trimmed
            .parse::<i32>()
            .map(HeapLiteralValue::I32)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i32 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_UINT32 => parse_u32_literal(trimmed, line_num).map(HeapLiteralValue::U32),
        TYPE_SYSTEM_INT64 => trimmed
            .parse::<i64>()
            .map(HeapLiteralValue::I64)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i64 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_UINT64 => trimmed
            .parse::<u64>()
            .map(HeapLiteralValue::U64)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u64 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        TYPE_SYSTEM_SINGLE => parse_f32_literal(trimmed, line_num).map(HeapLiteralValue::F32),
        TYPE_SYSTEM_DOUBLE => parse_f64_literal(trimmed, line_num).map(HeapLiteralValue::F64),
        TYPE_SYSTEM_STRING => Ok(HeapLiteralValue::String(parse_quoted_string(
            trimmed, line_num,
        )?)),
        TYPE_SYSTEM_TYPE => Ok(HeapLiteralValue::SystemType(parse_system_type_literal(
            trimmed, line_num,
        )?)),
        TYPE_UNITY_VECTOR2 => {
            let (x, y) = parse_typed_vector2_literal(trimmed, line_num)?;
            Ok(HeapLiteralValue::Vector2(x, y))
        }
        TYPE_UNITY_VECTOR3 => {
            let (x, y, z) = parse_typed_vector3_literal(trimmed, line_num)?;
            Ok(HeapLiteralValue::Vector3(x, y, z))
        }
        TYPE_UNITY_QUATERNION => {
            let (x, y, z, w) = parse_typed_quaternion_literal(trimmed, line_num)?;
            Ok(HeapLiteralValue::Quaternion(x, y, z, w))
        }
        TYPE_UNITY_COLOR => {
            let (r, g, b, a) = parse_typed_color_literal(trimmed, line_num)?;
            Ok(HeapLiteralValue::Color(r, g, b, a))
        }
        TYPE_VRC_SERIALIZATION_RESULT => {
            let (success, byte_count) =
                parse_typed_serialization_result_literal(trimmed, line_num)?;
            Ok(HeapLiteralValue::SerializationResult {
                success,
                byte_count,
            })
        }
        TYPE_SYSTEM_UINT32_ARRAY => Ok(HeapLiteralValue::U32Array(parse_u32_array_literal(
            trimmed, line_num,
        )?)),
        _ => Err(AsmError::new(format!(
            "Line {}: unsupported typed init for '{}', use '{}'.",
            line_num, type_name, UNSERIALIZABLE_LITERAL
        ))),
    }
}

fn parse_quoted_string(text: &str, line_num: usize) -> Result<String> {
    let trimmed = text.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err(AsmError::new(format!(
            "Line {}: string init must be quoted, got '{}'.",
            line_num, text
        )));
    }
    unescape_quoted_string(&trimmed[1..trimmed.len() - 1], line_num)
}

fn parse_u32_literal(text: &str, line_num: usize) -> Result<u32> {
    if let Some(hex) = text.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid u32 hex init '{}': {}",
                line_num, text, e
            ))
        })
    } else {
        text.parse::<u32>().map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid u32 init '{}': {}",
                line_num, text, e
            ))
        })
    }
}

fn parse_i32_literal(text: &str, line_num: usize) -> Result<i32> {
    if let Some(hex) = text.strip_prefix("0x") {
        let value = u32::from_str_radix(hex, 16).map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid i32 hex init '{}': {}",
                line_num, text, e
            ))
        })?;
        i32::try_from(value).map_err(|_| {
            AsmError::new(format!(
                "Line {}: i32 hex init '{}' is out of range.",
                line_num, text
            ))
        })
    } else {
        text.parse::<i32>().map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid i32 init '{}': {}",
                line_num, text, e
            ))
        })
    }
}

fn parse_f32_literal(text: &str, line_num: usize) -> Result<f32> {
    parse_f32_component(text)
        .ok_or_else(|| AsmError::new(format!("Line {}: invalid f32 init '{}'.", line_num, text)))
}

fn parse_f64_literal(text: &str, line_num: usize) -> Result<f64> {
    parse_f64_component(text)
        .ok_or_else(|| AsmError::new(format!("Line {}: invalid f64 init '{}'.", line_num, text)))
}

fn parse_typed_enum_literal(
    type_head: &str,
    repr: EnumRepr,
    trimmed: &str,
    line_num: usize,
) -> Result<HeapLiteralValue> {
    let enum_token = trimmed.rsplit('.').next().unwrap_or(trimmed).trim();
    if let Some(value) = enum_name_to_value(type_head, enum_token) {
        return match repr {
            EnumRepr::I32 => i32::try_from(value)
                .map(HeapLiteralValue::I32)
                .map_err(|_| {
                    AsmError::new(format!(
                        "Line {}: enum value '{}' out of i32 range for '{}'.",
                        line_num, trimmed, type_head
                    ))
                }),
            EnumRepr::U8 => u8::try_from(value).map(HeapLiteralValue::U8).map_err(|_| {
                AsmError::new(format!(
                    "Line {}: enum value '{}' out of u8 range for '{}'.",
                    line_num, trimmed, type_head
                ))
            }),
        };
    }

    if let Some((cast_type, cast_value)) = parse_csharp_cast_literal(trimmed)
        && normalize_type_token(cast_type) == normalize_type_token(type_head)
    {
        return match repr {
            EnumRepr::I32 => parse_i32_literal(cast_value, line_num).map(HeapLiteralValue::I32),
            EnumRepr::U8 => parse_u32_literal(cast_value, line_num)
                .and_then(|v| {
                    u8::try_from(v).map_err(|_| {
                        AsmError::new(format!(
                            "Line {}: enum init '{}' is out of u8 range for '{}'.",
                            line_num, trimmed, type_head
                        ))
                    })
                })
                .map(HeapLiteralValue::U8),
        };
    }

    match repr {
        EnumRepr::I32 => parse_i32_literal(trimmed, line_num).map(HeapLiteralValue::I32),
        EnumRepr::U8 => parse_u32_literal(trimmed, line_num)
            .and_then(|v| {
                u8::try_from(v).map_err(|_| {
                    AsmError::new(format!(
                        "Line {}: enum init '{}' is out of u8 range for '{}'.",
                        line_num, trimmed, type_head
                    ))
                })
            })
            .map(HeapLiteralValue::U8),
    }
}

fn enum_literal_from_node_kind(type_head: &str, kind: &NodeKind) -> Option<HeapLiteralValue> {
    let repr = enum_repr(type_head)?;
    let value = match kind {
        NodeKind::Primitive(PrimitiveValue::SByte(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Byte(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Short(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::UShort(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Int(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::UInt(v)) => i64::from(*v),
        NodeKind::Primitive(PrimitiveValue::Long(v)) => *v,
        NodeKind::Primitive(PrimitiveValue::ULong(v)) => i64::try_from(*v).ok()?,
        _ => return None,
    };

    match repr {
        EnumRepr::U8 => u8::try_from(value).ok().map(HeapLiteralValue::U8),
        EnumRepr::I32 => i32::try_from(value).ok().map(HeapLiteralValue::I32),
    }
}

fn parse_u32_array_literal(text: &str, line_num: usize) -> Result<Vec<u32>> {
    let body = parse_array_literal_body(text, line_num, "u32[]")?;
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::<u32>::new();
    for part in body.split(',') {
        out.push(parse_u32_literal(part.trim(), line_num)?);
    }
    Ok(out)
}

fn parse_array_items(text: &str, line_num: usize) -> Result<Vec<String>> {
    let body = parse_array_literal_body(text, line_num, "array")?;
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::<String>::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for ch in body.chars() {
        if in_string {
            current.push(ch);
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                current.push(ch);
            }
            ',' => {
                out.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if in_string {
        return Err(AsmError::new(format!(
            "Line {}: unterminated string in array literal '{}'.",
            line_num, text
        )));
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    } else if body.ends_with(',') {
        return Err(AsmError::new(format!(
            "Line {}: trailing comma in array literal '{}'.",
            line_num, text
        )));
    }
    Ok(out)
}

fn parse_array_literal_body<'a>(
    text: &'a str,
    line_num: usize,
    type_name: &str,
) -> Result<&'a str> {
    let trimmed = text.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return Ok(&trimmed[1..trimmed.len() - 1]);
    }

    if let Some(after_new) = trimmed.strip_prefix("new").map(str::trim_start) {
        let Some(open_brace) = after_new.find('{') else {
            return Err(AsmError::new(format!(
                "Line {}: {} init must be '[...]' or 'new <type>[] {{ ... }}', got '{}'.",
                line_num, type_name, text
            )));
        };
        let Some(close_brace) = after_new.rfind('}') else {
            return Err(AsmError::new(format!(
                "Line {}: invalid {} init '{}'. Missing '}}'.",
                line_num, type_name, text
            )));
        };
        if close_brace <= open_brace {
            return Err(AsmError::new(format!(
                "Line {}: invalid {} init '{}'.",
                line_num, type_name, text
            )));
        }
        if !after_new[close_brace + 1..].trim().is_empty() {
            return Err(AsmError::new(format!(
                "Line {}: invalid {} init '{}'. Trailing text after array initializer.",
                line_num, type_name, text
            )));
        }
        return Ok(after_new[open_brace + 1..close_brace].trim());
    }

    Err(AsmError::new(format!(
        "Line {}: {} init must be '[...]' or 'new <type>[] {{ ... }}', got '{}'.",
        line_num, type_name, text
    )))
}

fn parse_csharp_cast_literal(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    let rest = trimmed.strip_prefix('(')?;
    let close_idx = rest.find(')')?;
    let cast_type = rest[..close_idx].trim();
    let value = rest[close_idx + 1..].trim();
    if cast_type.is_empty() || value.is_empty() {
        return None;
    }
    Some((cast_type, value))
}

fn normalize_type_token(text: &str) -> String {
    type_name_head(text).replace('+', ".")
}

fn decode_typed_primitive_array_from_raw(
    element_type: &str,
    bytes_per_element: usize,
    raw: &[u8],
) -> Option<Vec<HeapLiteralValue>> {
    if bytes_per_element == 0 || !raw.len().is_multiple_of(bytes_per_element) {
        return None;
    }
    let head = type_name_head(element_type);
    let mut out = Vec::<HeapLiteralValue>::with_capacity(raw.len() / bytes_per_element);
    for chunk in raw.chunks_exact(bytes_per_element) {
        if let Some(repr) = enum_repr(head) {
            let value = match repr {
                EnumRepr::U8 if bytes_per_element == 1 => HeapLiteralValue::U8(chunk[0]),
                EnumRepr::I32 if bytes_per_element == 4 => {
                    HeapLiteralValue::I32(i32::from_le_bytes([
                        chunk[0], chunk[1], chunk[2], chunk[3],
                    ]))
                }
                _ => return None,
            };
            out.push(value);
            continue;
        }
        let value = match head {
            TYPE_SYSTEM_BOOLEAN if bytes_per_element == 1 => HeapLiteralValue::Bool(chunk[0] != 0),
            TYPE_SYSTEM_SBYTE if bytes_per_element == 1 => HeapLiteralValue::I8(chunk[0] as i8),
            TYPE_SYSTEM_BYTE if bytes_per_element == 1 => HeapLiteralValue::U8(chunk[0]),
            TYPE_SYSTEM_INT16 if bytes_per_element == 2 => {
                HeapLiteralValue::I16(i16::from_le_bytes([chunk[0], chunk[1]]))
            }
            TYPE_SYSTEM_UINT16 if bytes_per_element == 2 => {
                HeapLiteralValue::U16(u16::from_le_bytes([chunk[0], chunk[1]]))
            }
            TYPE_SYSTEM_INT32 if bytes_per_element == 4 => {
                HeapLiteralValue::I32(i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            }
            TYPE_SYSTEM_UINT32 if bytes_per_element == 4 => {
                HeapLiteralValue::U32(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            }
            TYPE_SYSTEM_INT64 if bytes_per_element == 8 => {
                HeapLiteralValue::I64(i64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ]))
            }
            TYPE_SYSTEM_UINT64 if bytes_per_element == 8 => {
                HeapLiteralValue::U64(u64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ]))
            }
            TYPE_SYSTEM_SINGLE if bytes_per_element == 4 => {
                HeapLiteralValue::F32(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            }
            TYPE_SYSTEM_DOUBLE if bytes_per_element == 8 => {
                HeapLiteralValue::F64(f64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ]))
            }
            _ => return None,
        };
        out.push(value);
    }
    Some(out)
}

fn is_supported_typed_scalar_or_enum(type_name: &str) -> bool {
    if enum_repr(type_name_head(type_name)).is_some() {
        return true;
    }
    matches!(
        type_name_head(type_name),
        TYPE_SYSTEM_BOOLEAN
            | TYPE_SYSTEM_SBYTE
            | TYPE_SYSTEM_BYTE
            | TYPE_SYSTEM_INT16
            | TYPE_SYSTEM_UINT16
            | TYPE_SYSTEM_INT32
            | TYPE_SYSTEM_UINT32
            | TYPE_SYSTEM_INT64
            | TYPE_SYSTEM_UINT64
            | TYPE_SYSTEM_SINGLE
            | TYPE_SYSTEM_DOUBLE
    )
}

fn is_unserializable_placeholder_token(token: &str) -> bool {
    let trimmed = token.trim();
    if trimmed == UNSERIALIZABLE_LITERAL {
        return true;
    }
    if trimmed == UNSERIALIZABLE_ARRAY_ELEMENT_LITERAL {
        return true;
    }
    if trimmed == "null" {
        return true;
    }
    if trimmed.starts_with("null") && trimmed.to_ascii_lowercase().contains("unserializable") {
        return true;
    }
    false
}

fn parse_typed_vector2_literal(text: &str, line_num: usize) -> Result<(f32, f32)> {
    parse_vector2_literal(text).ok_or_else(|| {
        AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Expected 'new {}(x, y)' or '(x, y)'.",
            line_num, TYPE_UNITY_VECTOR2, text, TYPE_UNITY_VECTOR2
        ))
    })
}

fn parse_typed_vector3_literal(text: &str, line_num: usize) -> Result<(f32, f32, f32)> {
    parse_vector3_literal(text).ok_or_else(|| {
        AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Expected 'new {}(x, y, z)' or '(x, y, z)'.",
            line_num, TYPE_UNITY_VECTOR3, text, TYPE_UNITY_VECTOR3
        ))
    })
}

fn parse_typed_quaternion_literal(text: &str, line_num: usize) -> Result<(f32, f32, f32, f32)> {
    parse_quaternion_literal(text).ok_or_else(|| {
        AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Expected 'new {}(x, y, z, w)' or '(x, y, z, w)'.",
            line_num, TYPE_UNITY_QUATERNION, text, TYPE_UNITY_QUATERNION
        ))
    })
}

fn parse_typed_color_literal(text: &str, line_num: usize) -> Result<(f32, f32, f32, f32)> {
    parse_color_literal(text).ok_or_else(|| {
        AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Expected 'new {}(r, g, b, a)', 'RGBA(r, g, b, a)' or '(r, g, b, a)'.",
            line_num, TYPE_UNITY_COLOR, text, TYPE_UNITY_COLOR
        ))
    })
}

fn parse_typed_serialization_result_literal(text: &str, line_num: usize) -> Result<(bool, i32)> {
    parse_serialization_result_text(text).ok_or_else(|| {
        AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Expected 'new {} {{ success = <bool>, byteCount = <int> }}'.",
            line_num, TYPE_VRC_SERIALIZATION_RESULT, text, TYPE_VRC_SERIALIZATION_RESULT
        ))
    })
}

fn parse_system_type_literal(text: &str, line_num: usize) -> Result<String> {
    let trimmed = text.trim();
    let Some(rest) = trimmed.strip_prefix("typeof") else {
        return Err(AsmError::new(format!(
            "Line {}: {} init must use typeof(...), got '{}'.",
            line_num, TYPE_SYSTEM_TYPE, text
        )));
    };
    let rest = rest.trim_start();
    if !rest.starts_with('(') {
        return Err(AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Expected typeof(<type>) optionally with /* <assembly> */.",
            line_num, TYPE_SYSTEM_TYPE, text
        )));
    }

    let mut depth = 0usize;
    let mut close_index = None::<usize>;
    for (idx, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Err(AsmError::new(format!(
                        "Line {}: invalid {} init '{}'.",
                        line_num, TYPE_SYSTEM_TYPE, text
                    )));
                }
                depth -= 1;
                if depth == 0 {
                    close_index = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close_index) = close_index else {
        return Err(AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Missing ')'.",
            line_num, TYPE_SYSTEM_TYPE, text
        )));
    };

    let type_name = rest[1..close_index].trim();
    if type_name.is_empty() {
        return Err(AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Empty type inside typeof(...).",
            line_num, TYPE_SYSTEM_TYPE, text
        )));
    }

    let tail = rest[close_index + 1..].trim();
    if tail.is_empty() {
        return Ok(type_name.to_string());
    }

    if !(tail.starts_with("/*") && tail.ends_with("*/") && tail.len() >= 4) {
        return Err(AsmError::new(format!(
            "Line {}: invalid {} init '{}'. Trailing text must be block comment /* <assembly> */.",
            line_num, TYPE_SYSTEM_TYPE, text
        )));
    }
    let assembly = tail[2..tail.len() - 2].trim();
    if assembly.is_empty() {
        return Ok(type_name.to_string());
    }
    Ok(format!("{type_name}, {assembly}"))
}

fn parse_vector2_literal(text: &str) -> Option<(f32, f32)> {
    parse_ctor_or_tuple_literal(text, &[TYPE_UNITY_VECTOR2], 2).map(|v| (v[0], v[1]))
}

fn parse_vector3_literal(text: &str) -> Option<(f32, f32, f32)> {
    parse_ctor_or_tuple_literal(text, &[TYPE_UNITY_VECTOR3], 3).map(|v| (v[0], v[1], v[2]))
}

fn parse_quaternion_literal(text: &str) -> Option<(f32, f32, f32, f32)> {
    parse_ctor_or_tuple_literal(text, &[TYPE_UNITY_QUATERNION], 4).map(|v| (v[0], v[1], v[2], v[3]))
}

fn parse_color_literal(text: &str) -> Option<(f32, f32, f32, f32)> {
    if let Some(inner) = text.trim().strip_prefix("RGBA(")
        && let Some(inner) = inner.strip_suffix(')')
    {
        let parts = inner.split(',').map(str::trim).collect::<Vec<_>>();
        if parts.len() == 4 {
            return Some((
                parse_f32_component(parts[0])?,
                parse_f32_component(parts[1])?,
                parse_f32_component(parts[2])?,
                parse_f32_component(parts[3])?,
            ));
        }
    }
    parse_ctor_or_tuple_literal(text, &[TYPE_UNITY_COLOR], 4).map(|v| (v[0], v[1], v[2], v[3]))
}

fn parse_ctor_or_tuple_literal(
    text: &str,
    ctor_names: &[&str],
    component_count: usize,
) -> Option<Vec<f32>> {
    let mut s = text.trim();
    if let Some(stripped) = s.strip_prefix("new") {
        s = stripped.trim_start();
        let mut matched_ctor = false;
        for ctor in ctor_names {
            if let Some(rest) = s.strip_prefix(ctor) {
                s = rest.trim_start();
                matched_ctor = true;
                break;
            }
        }
        if !matched_ctor {
            return None;
        }
    }

    if !(s.starts_with('(') && s.ends_with(')')) {
        return None;
    }
    let body = &s[1..s.len() - 1];
    let parts = body.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != component_count {
        return None;
    }
    let mut out = Vec::<f32>::with_capacity(component_count);
    for part in parts {
        out.push(parse_f32_component(part)?);
    }
    Some(out)
}

fn parse_serialization_result_text(text: &str) -> Option<(bool, i32)> {
    let trimmed = text.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return parse_serialization_result_json_like(trimmed);
    }
    parse_serialization_result_object_initializer(trimmed)
}

fn parse_serialization_result_object_initializer(text: &str) -> Option<(bool, i32)> {
    let prefix = format!("new {TYPE_VRC_SERIALIZATION_RESULT}");
    let rest = text.strip_prefix(prefix.as_str())?.trim_start();
    let body = if let Some(after_brace) = rest.strip_prefix('{') {
        after_brace.strip_suffix('}')?.trim()
    } else {
        return None;
    };
    parse_serialization_result_key_values(body, '=')
}

fn parse_serialization_result_json_like(text: &str) -> Option<(bool, i32)> {
    let body = text.strip_prefix('{')?.strip_suffix('}')?.trim();
    parse_serialization_result_key_values(body, ':')
}

fn parse_serialization_result_key_values(body: &str, sep: char) -> Option<(bool, i32)> {
    let mut success: Option<bool> = None;
    let mut byte_count: Option<i32> = None;

    for pair in body.split(',') {
        let (key_raw, value_raw) = pair.split_once(sep)?;
        let key = key_raw.trim().trim_matches('"');
        let value = value_raw.trim();
        if key == "success" {
            success = match value {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        } else if key == "byteCount" || key == "byte_count" {
            byte_count = value.parse::<i32>().ok();
        }
    }
    Some((success?, byte_count?))
}

fn parse_f32_component(text: &str) -> Option<f32> {
    let trimmed = text.trim();
    let number = if let Some(v) = trimmed.strip_suffix('f') {
        v.trim_end()
    } else if let Some(v) = trimmed.strip_suffix('F') {
        v.trim_end()
    } else {
        trimmed
    };
    number.parse::<f32>().ok()
}

fn parse_f64_component(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let number = if let Some(v) = trimmed.strip_suffix('d') {
        v.trim_end()
    } else if let Some(v) = trimmed.strip_suffix('D') {
        v.trim_end()
    } else {
        trimmed
    };
    number.parse::<f64>().ok()
}

fn unescape_quoted_string(input: &str, line_num: usize) -> Result<String> {
    serde_json::from_str::<String>(&format!("\"{input}\"")).map_err(|e| {
        AsmError::new(format!(
            "Line {}: invalid escaped string literal '{}': {}",
            line_num, input, e
        ))
    })
}

pub(crate) fn literal_from_typed_odin_node(
    doc: &OdinDocument,
    node_id: NodeId,
    type_name: &str,
) -> HeapLiteralValue {
    let resolved = doc.resolve_node_payload(node_id).unwrap_or(node_id);
    let Some(node) = doc.node(resolved) else {
        return HeapLiteralValue::Unserializable;
    };

    let head = type_name_head(type_name);
    if head.strip_suffix("[]").is_some() {
        if let NodeKind::PrimitiveArray {
            element_count,
            bytes_per_element,
        } = node.kind()
        {
            let count = usize::try_from(*element_count).ok();
            let bpe = usize::try_from(*bytes_per_element).ok();
            if let (Some(count), Some(bpe)) = (count, bpe) {
                let mut raw = Vec::with_capacity(count.saturating_mul(bpe));
                let mut ok = true;
                for i in 0..count {
                    match doc.primitive_array_element(resolved, i) {
                        Ok(chunk) => raw.extend_from_slice(chunk),
                        Err(_) => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok
                    && let Some(literal) =
                        literal_from_typed_primitive_array_raw(type_name, bpe, &raw)
                {
                    return parse_roundtrip_literal(type_name, literal);
                }
                return HeapLiteralValue::OpaqueArray { len: count };
            }
        }

        if let NodeKind::Array { declared_len } = node.kind() {
            let len = usize::try_from((*declared_len).max(0)).unwrap_or(0);
            return HeapLiteralValue::OpaqueArray { len };
        }
    }

    if head == TYPE_SYSTEM_TYPE
        && let Some(name) = extract_system_type_name_from_node(doc, resolved)
    {
        return parse_roundtrip_literal(type_name, HeapLiteralValue::SystemType(name));
    }
    if head == TYPE_UNITY_VECTOR3
        && let Some((x, y, z)) = extract_vector3(doc, resolved)
    {
        return parse_roundtrip_literal(type_name, HeapLiteralValue::Vector3(x, y, z));
    }
    if head == TYPE_UNITY_VECTOR2
        && let Some((x, y)) = extract_vector2(doc, resolved)
    {
        return parse_roundtrip_literal(type_name, HeapLiteralValue::Vector2(x, y));
    }
    if head == TYPE_UNITY_QUATERNION
        && let Some((x, y, z, w)) = extract_quaternion(doc, resolved)
    {
        return parse_roundtrip_literal(type_name, HeapLiteralValue::Quaternion(x, y, z, w));
    }
    if head == TYPE_UNITY_COLOR
        && let Some((r, g, b, a)) = extract_color(doc, resolved)
    {
        return parse_roundtrip_literal(type_name, HeapLiteralValue::Color(r, g, b, a));
    }
    if head == TYPE_VRC_SERIALIZATION_RESULT
        && let Some((success, byte_count)) = extract_serialization_result(doc, resolved)
    {
        return parse_roundtrip_literal(
            type_name,
            HeapLiteralValue::SerializationResult {
                success,
                byte_count,
            },
        );
    }

    let literal = heap_literal_from_node_kind(type_name, node.kind());
    if matches!(literal, HeapLiteralValue::Unserializable) {
        HeapLiteralValue::Unserializable
    } else {
        parse_roundtrip_literal(type_name, literal)
    }
}

fn parse_roundtrip_literal(type_name: &str, literal: HeapLiteralValue) -> HeapLiteralValue {
    let text = render_heap_literal(type_name, &literal);
    parse_typed_heap_literal(type_name, text.as_str(), 0).unwrap_or(literal)
}

fn literal_from_typed_primitive_array_raw(
    type_name: &str,
    bytes_per_element: usize,
    raw: &[u8],
) -> Option<HeapLiteralValue> {
    let head = type_name_head(type_name);
    let element_type = head.strip_suffix("[]")?.trim();
    let elements = decode_typed_primitive_array_from_raw(element_type, bytes_per_element, raw)?;
    if head == TYPE_SYSTEM_UINT32_ARRAY {
        let values = elements
            .iter()
            .map(|x| x.as_u32())
            .collect::<Option<Vec<_>>>()?;
        return Some(HeapLiteralValue::U32Array(values));
    }
    Some(HeapLiteralValue::TypedArray {
        element_type: element_type.to_string(),
        elements,
    })
}

pub(crate) fn resolve_heap_literal_for_program_entry(
    program: &UdonProgramBinary,
    index: usize,
    type_name: &str,
    _resolved_kind: &NodeKind,
) -> crate::odin::Result<HeapLiteralValue> {
    let value_node = program.heap_dump_strongbox_value_node_id(index)?;
    Ok(literal_from_typed_odin_node(
        program.document(),
        value_node,
        type_name,
    ))
}

pub(crate) fn heap_literal_from_node_kind(type_name: &str, kind: &NodeKind) -> HeapLiteralValue {
    if matches!(kind, NodeKind::Null) {
        return HeapLiteralValue::Null;
    }
    let head = type_name_head(type_name);
    if let Some(value) = enum_literal_from_node_kind(head, kind) {
        return value;
    }
    match (head, kind) {
        (TYPE_SYSTEM_BOOLEAN, NodeKind::Primitive(PrimitiveValue::Boolean(v))) => {
            HeapLiteralValue::Bool(*v)
        }
        (TYPE_SYSTEM_SBYTE, NodeKind::Primitive(PrimitiveValue::SByte(v))) => {
            HeapLiteralValue::I8(*v)
        }
        (TYPE_SYSTEM_BYTE, NodeKind::Primitive(PrimitiveValue::Byte(v))) => {
            HeapLiteralValue::U8(*v)
        }
        (TYPE_SYSTEM_INT16, NodeKind::Primitive(PrimitiveValue::Short(v))) => {
            HeapLiteralValue::I16(*v)
        }
        (TYPE_SYSTEM_UINT16, NodeKind::Primitive(PrimitiveValue::UShort(v))) => {
            HeapLiteralValue::U16(*v)
        }
        (TYPE_SYSTEM_INT32, NodeKind::Primitive(PrimitiveValue::Int(v))) => {
            HeapLiteralValue::I32(*v)
        }
        (TYPE_SYSTEM_UINT32, NodeKind::Primitive(PrimitiveValue::UInt(v))) => {
            HeapLiteralValue::U32(*v)
        }
        (TYPE_SYSTEM_INT64, NodeKind::Primitive(PrimitiveValue::Long(v))) => {
            HeapLiteralValue::I64(*v)
        }
        (TYPE_SYSTEM_UINT64, NodeKind::Primitive(PrimitiveValue::ULong(v))) => {
            HeapLiteralValue::U64(*v)
        }
        (TYPE_SYSTEM_SINGLE, NodeKind::Primitive(PrimitiveValue::Float(v))) => {
            HeapLiteralValue::F32(*v)
        }
        (TYPE_SYSTEM_DOUBLE, NodeKind::Primitive(PrimitiveValue::Double(v))) => {
            HeapLiteralValue::F64(*v)
        }
        (TYPE_SYSTEM_STRING, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            HeapLiteralValue::String(v.value.clone())
        }
        (TYPE_SYSTEM_TYPE, NodeKind::TypeNameMetadata { name, .. }) => {
            HeapLiteralValue::SystemType(name.value.clone())
        }
        (TYPE_SYSTEM_TYPE, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            HeapLiteralValue::SystemType(v.value.clone())
        }
        (
            TYPE_SYSTEM_TYPE,
            NodeKind::TypeIdMetadata {
                resolved_name: Some(name),
                ..
            },
        ) => HeapLiteralValue::SystemType(name.clone()),
        (TYPE_UNITY_VECTOR2, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            if let Some((x, y)) = parse_vector2_literal(v.value.as_str()) {
                HeapLiteralValue::Vector2(x, y)
            } else {
                HeapLiteralValue::Unserializable
            }
        }
        (TYPE_UNITY_VECTOR3, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            if let Some((x, y, z)) = parse_vector3_literal(v.value.as_str()) {
                HeapLiteralValue::Vector3(x, y, z)
            } else {
                HeapLiteralValue::Unserializable
            }
        }
        (TYPE_UNITY_QUATERNION, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            if let Some((x, y, z, w)) = parse_quaternion_literal(v.value.as_str()) {
                HeapLiteralValue::Quaternion(x, y, z, w)
            } else {
                HeapLiteralValue::Unserializable
            }
        }
        (TYPE_UNITY_COLOR, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            if let Some((r, g, b, a)) = parse_color_literal(v.value.as_str()) {
                HeapLiteralValue::Color(r, g, b, a)
            } else {
                HeapLiteralValue::Unserializable
            }
        }
        (TYPE_VRC_SERIALIZATION_RESULT, NodeKind::Primitive(PrimitiveValue::String(v))) => {
            if let Some((success, byte_count)) = parse_serialization_result_text(v.value.as_str()) {
                HeapLiteralValue::SerializationResult {
                    success,
                    byte_count,
                }
            } else {
                HeapLiteralValue::Unserializable
            }
        }
        _ => HeapLiteralValue::Unserializable,
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

fn extract_vector3(doc: &OdinDocument, root: NodeId) -> Option<(f32, f32, f32)> {
    let ids = find_float_components_by_names_or_fallback(doc, root, &["x", "y", "z"])?;
    Some((
        read_f32_primitive(doc, ids[0])?,
        read_f32_primitive(doc, ids[1])?,
        read_f32_primitive(doc, ids[2])?,
    ))
}

fn extract_vector2(doc: &OdinDocument, root: NodeId) -> Option<(f32, f32)> {
    let ids = find_float_components_by_names_or_fallback(doc, root, &["x", "y"])?;
    Some((
        read_f32_primitive(doc, ids[0])?,
        read_f32_primitive(doc, ids[1])?,
    ))
}

fn extract_quaternion(doc: &OdinDocument, root: NodeId) -> Option<(f32, f32, f32, f32)> {
    let ids = find_float_components_by_names_or_fallback(doc, root, &["x", "y", "z", "w"])?;
    Some((
        read_f32_primitive(doc, ids[0])?,
        read_f32_primitive(doc, ids[1])?,
        read_f32_primitive(doc, ids[2])?,
        read_f32_primitive(doc, ids[3])?,
    ))
}

fn extract_color(doc: &OdinDocument, root: NodeId) -> Option<(f32, f32, f32, f32)> {
    let ids = find_float_components_by_names_or_fallback(doc, root, &["r", "g", "b", "a"])?;
    Some((
        read_f32_primitive(doc, ids[0])?,
        read_f32_primitive(doc, ids[1])?,
        read_f32_primitive(doc, ids[2])?,
        read_f32_primitive(doc, ids[3])?,
    ))
}

fn extract_serialization_result(doc: &OdinDocument, root: NodeId) -> Option<(bool, i32)> {
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
    Some((
        read_bool_primitive(doc, success)?,
        read_i32_like_primitive(doc, byte_count)?,
    ))
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
                .collect(),
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

fn read_f32_primitive(doc: &OdinDocument, node_id: NodeId) -> Option<f32> {
    match doc.node(node_id)?.kind() {
        NodeKind::Primitive(PrimitiveValue::Float(v)) => Some(*v),
        _ => None,
    }
}

fn read_bool_primitive(doc: &OdinDocument, node_id: NodeId) -> Option<bool> {
    match doc.node(node_id)?.kind() {
        NodeKind::Primitive(PrimitiveValue::Boolean(v)) => Some(*v),
        _ => None,
    }
}

fn read_i32_like_primitive(doc: &OdinDocument, node_id: NodeId) -> Option<i32> {
    match doc.node(node_id)?.kind() {
        NodeKind::Primitive(PrimitiveValue::SByte(v)) => Some(i32::from(*v)),
        NodeKind::Primitive(PrimitiveValue::Byte(v)) => Some(i32::from(*v)),
        NodeKind::Primitive(PrimitiveValue::Short(v)) => Some(i32::from(*v)),
        NodeKind::Primitive(PrimitiveValue::UShort(v)) => Some(i32::from(*v)),
        NodeKind::Primitive(PrimitiveValue::Int(v)) => Some(*v),
        NodeKind::Primitive(PrimitiveValue::UInt(v)) => i32::try_from(*v).ok(),
        NodeKind::Primitive(PrimitiveValue::Long(v)) => i32::try_from(*v).ok(),
        NodeKind::Primitive(PrimitiveValue::ULong(v)) => i32::try_from(*v).ok(),
        _ => None,
    }
}
