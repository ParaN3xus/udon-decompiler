use crate::odin::{NodeKind, PrimitiveValue};

pub(crate) fn sanitize_label(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "label".to_string()
    } else {
        out
    }
}

pub(crate) fn generated_heap_symbol(address: u32) -> String {
    format!("heap_0x{:08X}", address)
}

pub(crate) fn parse_generated_heap_symbol(symbol: &str) -> Option<u32> {
    if let Some(hex) = symbol.strip_prefix("heap_0x")
        && hex.len() == 8
    {
        return u32::from_str_radix(hex, 16).ok();
    }
    if let Some(hex) = symbol.strip_prefix("heap_")
        && hex.len() == 8
    {
        return u32::from_str_radix(hex, 16).ok();
    }
    None
}

pub(crate) fn normalize_type_name(type_name: &str) -> String {
    type_name
        .split(',')
        .next()
        .unwrap_or(type_name)
        .trim()
        .to_ascii_lowercase()
}

pub(crate) fn render_heap_init_literal(type_name: &str, kind: &NodeKind) -> String {
    if matches!(kind, NodeKind::Null) {
        return "null".to_string();
    }
    let normalized = normalize_type_name(type_name);
    match (normalized.as_str(), kind) {
        ("system.boolean", NodeKind::Primitive(PrimitiveValue::Boolean(v))) => v.to_string(),
        ("system.sbyte", NodeKind::Primitive(PrimitiveValue::SByte(v))) => v.to_string(),
        ("system.byte", NodeKind::Primitive(PrimitiveValue::Byte(v))) => v.to_string(),
        ("system.int16", NodeKind::Primitive(PrimitiveValue::Short(v))) => v.to_string(),
        ("system.uint16", NodeKind::Primitive(PrimitiveValue::UShort(v))) => v.to_string(),
        ("system.int32", NodeKind::Primitive(PrimitiveValue::Int(v))) => v.to_string(),
        ("system.uint32", NodeKind::Primitive(PrimitiveValue::UInt(v))) => v.to_string(),
        ("system.int64", NodeKind::Primitive(PrimitiveValue::Long(v))) => v.to_string(),
        ("system.uint64", NodeKind::Primitive(PrimitiveValue::ULong(v))) => v.to_string(),
        ("system.single", NodeKind::Primitive(PrimitiveValue::Float(v))) => v.to_string(),
        ("system.double", NodeKind::Primitive(PrimitiveValue::Double(v))) => v.to_string(),
        ("system.string", NodeKind::Primitive(PrimitiveValue::String(v))) => {
            json_escape_string(&v.value)
        }
        _ => "{ unserializable }".to_string(),
    }
}

fn json_escape_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}
