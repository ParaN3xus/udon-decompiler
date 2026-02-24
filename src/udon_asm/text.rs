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

pub(crate) fn type_name_head(type_name: &str) -> &str {
    type_name.split(',').next().unwrap_or(type_name).trim()
}

pub(crate) fn render_heap_init_literal(type_name: &str, kind: &NodeKind) -> String {
    if matches!(kind, NodeKind::Null) {
        return "null".to_string();
    }
    let head = type_name_head(type_name);
    match (head, kind) {
        ("System.Boolean", NodeKind::Primitive(PrimitiveValue::Boolean(v))) => v.to_string(),
        ("System.SByte", NodeKind::Primitive(PrimitiveValue::SByte(v))) => v.to_string(),
        ("System.Byte", NodeKind::Primitive(PrimitiveValue::Byte(v))) => v.to_string(),
        ("System.Int16", NodeKind::Primitive(PrimitiveValue::Short(v))) => v.to_string(),
        ("System.UInt16", NodeKind::Primitive(PrimitiveValue::UShort(v))) => v.to_string(),
        ("System.Int32", NodeKind::Primitive(PrimitiveValue::Int(v))) => v.to_string(),
        ("System.UInt32", NodeKind::Primitive(PrimitiveValue::UInt(v))) => v.to_string(),
        ("System.Int64", NodeKind::Primitive(PrimitiveValue::Long(v))) => v.to_string(),
        ("System.UInt64", NodeKind::Primitive(PrimitiveValue::ULong(v))) => v.to_string(),
        ("System.Single", NodeKind::Primitive(PrimitiveValue::Float(v))) => v.to_string(),
        ("System.Double", NodeKind::Primitive(PrimitiveValue::Double(v))) => v.to_string(),
        ("System.String", NodeKind::Primitive(PrimitiveValue::String(v))) => {
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
