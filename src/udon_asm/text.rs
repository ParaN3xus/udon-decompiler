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
