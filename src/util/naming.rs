pub fn sanitize_output_stem(name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let out = out.trim_matches('_');
    if out.is_empty() {
        "output".to_string()
    } else {
        out.to_string()
    }
}
