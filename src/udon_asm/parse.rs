use super::text::normalize_type_name;
use super::types::{
    AsmDirectives, AsmError, AsmInstruction, BindDirective, BindTableDirective, DirectiveSection,
    EntryDirective, HeapDirective, HeapExportMark, HeapInitDirective, OpCode, OperandToken,
    ParsedAsm, Result, TypeRefDirective, Visibility,
};

pub(crate) fn parse_asm_text(text: &str) -> Result<ParsedAsm> {
    let mut parsed = ParsedAsm::default();
    let mut pending_labels = Vec::<String>::new();
    let mut section_hint = None::<DirectiveSection>;

    for (line_no, raw_line) in text.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with(';') {
            let body = trimmed.trim_start_matches(';').trim();
            if body.eq_ignore_ascii_case("entry points") {
                section_hint = Some(DirectiveSection::EntryPoints);
                continue;
            }
            if body.eq_ignore_ascii_case("heap") {
                section_hint = Some(DirectiveSection::Heap);
                continue;
            }
            if body.eq_ignore_ascii_case("binds") {
                section_hint = Some(DirectiveSection::Binds);
                continue;
            }
            parse_directive_line(&mut parsed.directives, trimmed, line_num, section_hint)?;
            continue;
        }

        let line_without_inline_comment = if let Some((head, _)) = raw_line.split_once(';') {
            head.trim()
        } else {
            trimmed
        };
        if line_without_inline_comment.is_empty() {
            continue;
        }

        if line_without_inline_comment.ends_with(':') {
            let label = line_without_inline_comment
                .trim_end_matches(':')
                .trim()
                .to_string();
            if label.is_empty() {
                return Err(AsmError::new(format!("Line {}: empty label.", line_num)));
            }
            pending_labels.push(label);
            continue;
        }

        let (opcode_text, operand_text) =
            split_opcode_operand(line_without_inline_comment, line_num)?;
        let opcode = OpCode::from_name(opcode_text)?;
        let operand = if opcode.has_operand() {
            let operand_text = operand_text.ok_or_else(|| {
                AsmError::new(format!(
                    "Line {}: opcode {} requires an operand.",
                    line_num,
                    opcode.name()
                ))
            })?;
            Some(parse_operand_token(operand_text, opcode, line_num)?)
        } else {
            if operand_text.is_some() {
                return Err(AsmError::new(format!(
                    "Line {}: opcode {} does not take an operand.",
                    line_num,
                    opcode.name()
                )));
            }
            None
        };
        parsed.instructions.push(AsmInstruction {
            labels: std::mem::take(&mut pending_labels),
            opcode,
            operand,
        });
    }

    if !pending_labels.is_empty() {
        return Err(AsmError::new(
            "Dangling label without following instruction.",
        ));
    }
    Ok(parsed)
}

fn split_opcode_operand(line: &str, line_num: usize) -> Result<(&str, Option<&str>)> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let opcode = parts.next().ok_or_else(|| {
        AsmError::new(format!(
            "Line {}: expected opcode but line was empty.",
            line_num
        ))
    })?;
    let rest = parts.next().map(|x| x.trim()).filter(|x| !x.is_empty());
    let operand = rest.map(|x| x.trim_matches(',').trim());
    Ok((opcode, operand))
}

fn parse_operand_token(token: &str, opcode: OpCode, line_num: usize) -> Result<OperandToken> {
    if opcode.is_direct_jump() {
        if token.starts_with("0x") || token.chars().all(|c| c.is_ascii_digit()) {
            return Ok(OperandToken::Number(parse_u32(token, line_num)?));
        }
        return Ok(OperandToken::Label(token.to_string()));
    }
    if opcode.is_heap_operand() {
        if let Some(symbol) = token.strip_prefix('$') {
            return Ok(OperandToken::HeapSymbol(symbol.to_string()));
        }
        if token.starts_with("0x") || token.chars().all(|c| c.is_ascii_digit()) {
            return Ok(OperandToken::Number(parse_u32(token, line_num)?));
        }
        return Ok(OperandToken::HeapSymbol(token.to_string()));
    }
    Ok(OperandToken::Number(parse_u32(token, line_num)?))
}

fn parse_u32(text: &str, line_num: usize) -> Result<u32> {
    if let Some(hex) = text.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid hex number '{}': {}",
                line_num, text, e
            ))
        })
    } else {
        text.parse::<u32>().map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid number '{}': {}",
                line_num, text, e
            ))
        })
    }
}

fn parse_directive_line(
    directives: &mut AsmDirectives,
    line: &str,
    line_num: usize,
    section_hint: Option<DirectiveSection>,
) -> Result<()> {
    let body = line.trim_start_matches(';').trim();
    if body.is_empty()
        || body.eq_ignore_ascii_case("syntax")
        || body.starts_with("heap-show ")
        || body.starts_with("heap-raw ")
        || body.starts_with("heap-example ")
        || body.starts_with("entry-example ")
        || body.starts_with("bind-example ")
        || body.starts_with("entry:")
        || body.starts_with("heap:")
        || body.starts_with("bind:")
        || body.starts_with("bind-table:")
        || body.starts_with("unsupported heap init literal:")
    {
        return Ok(());
    }
    if body.starts_with("entry ") {
        let rest = body.trim_start_matches("entry").trim();
        parse_entry_directive_payload(directives, rest, line_num)?;
        return Ok(());
    }

    if body.starts_with("heap ") {
        let rest = body.trim_start_matches("heap").trim();
        parse_heap_directive_payload(directives, rest, line_num)?;
        return Ok(());
    }

    if body.starts_with("bind-table ") {
        let rest = body.trim_start_matches("bind-table").trim();
        let (symbol, rhs) = rest.split_once("->").ok_or_else(|| {
            AsmError::new(format!(
                "Line {}: bind-table expects '<symbol> -> [label,...]'.",
                line_num
            ))
        })?;
        let rhs = rhs.trim();
        if !(rhs.starts_with('[') && rhs.ends_with(']')) {
            return Err(AsmError::new(format!(
                "Line {}: bind-table rhs must be bracket list.",
                line_num
            )));
        }
        let inside = &rhs[1..rhs.len() - 1];
        let labels = if inside.trim().is_empty() {
            Vec::new()
        } else {
            inside
                .split(',')
                .map(|x| x.trim().trim_start_matches('@').to_string())
                .collect::<Vec<_>>()
        };
        directives.bind_tables.push(BindTableDirective {
            symbol: symbol.trim().to_string(),
            labels,
        });
        return Ok(());
    }

    if body.starts_with("bind ") {
        let rest = body.trim_start_matches("bind").trim();
        let (symbol, rhs) = rest.split_once("->").ok_or_else(|| {
            AsmError::new(format!(
                "Line {}: bind expects '<symbol> -> <label>'.",
                line_num
            ))
        })?;
        directives.binds.push(BindDirective {
            symbol: symbol.trim().to_string(),
            label: rhs.trim().trim_start_matches('@').to_string(),
        });
        return Ok(());
    }

    if matches!(section_hint, Some(DirectiveSection::Heap))
        && body.contains(':')
        && body.contains('=')
    {
        parse_heap_directive_payload(directives, body, line_num)?;
        return Ok(());
    }

    if body.starts_with('+') || body.starts_with('-') || body.starts_with('~') {
        parse_shorthand_directive(directives, body, line_num, section_hint)?;
        return Ok(());
    }

    Ok(())
}

fn parse_shorthand_directive(
    directives: &mut AsmDirectives,
    body: &str,
    line_num: usize,
    section_hint: Option<DirectiveSection>,
) -> Result<()> {
    match section_hint {
        Some(DirectiveSection::EntryPoints) => {
            parse_entry_directive_payload(directives, body, line_num)
        }
        Some(DirectiveSection::Heap) => parse_heap_directive_payload(directives, body, line_num),
        Some(DirectiveSection::Binds) => Ok(()),
        None => {
            if body.contains('=') && body.contains(':') {
                parse_heap_directive_payload(directives, body, line_num)
            } else {
                parse_entry_directive_payload(directives, body, line_num)
            }
        }
    }
}

fn parse_entry_directive_payload(
    directives: &mut AsmDirectives,
    text: &str,
    line_num: usize,
) -> Result<()> {
    let mut chars = text.chars();
    let vis_ch = chars
        .next()
        .ok_or_else(|| AsmError::new(format!("Line {}: malformed entry directive.", line_num)))?;
    let visibility = match vis_ch {
        '+' => Visibility::Public,
        '-' => Visibility::Private,
        _ => {
            return Err(AsmError::new(format!(
                "Line {}: entry visibility must be '+' or '-'.",
                line_num
            )));
        }
    };
    let rest = chars.as_str().trim();
    let (name, label) = if let Some((name, label)) = rest.split_once('@') {
        (name.trim(), label.trim())
    } else if let Some((label, name)) = rest.split_once(':') {
        (name.trim(), label.trim())
    } else {
        return Err(AsmError::new(format!(
            "Line {}: entry directive expects '<name> @ <label>'.",
            line_num
        )));
    };
    if name.is_empty() || label.is_empty() {
        return Err(AsmError::new(format!(
            "Line {}: entry name/label cannot be empty.",
            line_num
        )));
    }
    if name.contains('@') || label.contains('@') {
        return Err(AsmError::new(format!(
            "Line {}: entry name and label must not contain '@'.",
            line_num
        )));
    }
    directives.entries.push(EntryDirective {
        visibility,
        label: label.to_string(),
        name: name.to_string(),
    });
    Ok(())
}

fn parse_heap_directive_payload(
    directives: &mut AsmDirectives,
    text: &str,
    line_num: usize,
) -> Result<()> {
    let text = text.trim();
    let (export_mark, rest) = if let Some(rest) = text.strip_prefix('+') {
        (HeapExportMark::Exported, rest.trim())
    } else if let Some(rest) = text.strip_prefix('-') {
        (HeapExportMark::Private, rest.trim())
    } else if let Some(rest) = text.strip_prefix('~') {
        (HeapExportMark::Keep, rest.trim())
    } else {
        (HeapExportMark::Keep, text)
    };
    let (symbol, type_text, init_text) = if rest.contains('|') {
        let parts = rest.split('|').map(|x| x.trim()).collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(AsmError::new(format!(
                "Line {}: heap directive expects '<symbol> | <type> | <init>'.",
                line_num
            )));
        }
        (parts[0], parts[1], parts[2])
    } else {
        let (symbol, rhs) = rest.split_once(':').ok_or_else(|| {
            AsmError::new(format!(
                "Line {}: heap shorthand expects '<symbol>: <type> = <init>'.",
                line_num
            ))
        })?;
        let (type_text, init_text) = rhs.split_once('=').ok_or_else(|| {
            AsmError::new(format!(
                "Line {}: heap shorthand expects '<symbol>: <type> = <init>'.",
                line_num
            ))
        })?;
        (symbol.trim(), type_text.trim(), init_text.trim())
    };

    let type_ref = parse_type_ref(type_text, line_num)?;
    let init = parse_heap_init_with_type(
        init_text,
        line_num,
        match &type_ref {
            TypeRefDirective::Name(name) => Some(name.as_str()),
            TypeRefDirective::InternalRef(_) => None,
        },
    )?;
    directives.heap.push(HeapDirective {
        export_mark,
        symbol: symbol.to_string(),
        type_ref,
        init,
    });
    Ok(())
}

fn parse_type_ref(text: &str, line_num: usize) -> Result<TypeRefDirective> {
    let text = text.trim();
    if let Some(v) = text.strip_prefix("ref:") {
        let parsed = v.trim().parse::<i32>().map_err(|e| {
            AsmError::new(format!(
                "Line {}: invalid internal ref '{}': {}",
                line_num, text, e
            ))
        })?;
        Ok(TypeRefDirective::InternalRef(parsed))
    } else if let Some(name) = text.strip_prefix("name:") {
        Ok(TypeRefDirective::Name(name.trim().to_string()))
    } else if text.starts_with('{') && text.ends_with('}') && text.len() >= 2 {
        Ok(TypeRefDirective::Name(
            text[1..text.len() - 1].trim().to_string(),
        ))
    } else {
        Err(AsmError::new(format!(
            "Line {}: type ref must be 'ref:<i32>', 'name:<string>' or '{{<type-name>}}'.",
            line_num
        )))
    }
}

fn parse_heap_init_with_type(
    text: &str,
    line_num: usize,
    type_name: Option<&str>,
) -> Result<HeapInitDirective> {
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("{ unserializable }") {
        return Ok(HeapInitDirective::Ignore);
    }
    if let Some(type_name) = type_name {
        return parse_typed_heap_init(type_name, trimmed, line_num);
    }
    parse_legacy_heap_init(trimmed, line_num)
}

fn parse_typed_heap_init(
    type_name: &str,
    trimmed: &str,
    line_num: usize,
) -> Result<HeapInitDirective> {
    if trimmed.eq_ignore_ascii_case("null") {
        return Ok(HeapInitDirective::Null);
    }
    let normalized = normalize_type_name(type_name);
    match normalized.as_str() {
        "system.boolean" => match trimmed {
            "true" => Ok(HeapInitDirective::Bool(true)),
            "false" => Ok(HeapInitDirective::Bool(false)),
            _ => Err(AsmError::new(format!(
                "Line {}: bool init must be true/false, got '{}'.",
                line_num, trimmed
            ))),
        },
        "system.sbyte" => trimmed
            .parse::<i8>()
            .map(HeapInitDirective::I8)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i8 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.byte" => trimmed
            .parse::<u8>()
            .map(HeapInitDirective::U8)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u8 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.int16" => trimmed
            .parse::<i16>()
            .map(HeapInitDirective::I16)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i16 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.uint16" => trimmed
            .parse::<u16>()
            .map(HeapInitDirective::U16)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u16 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.int32" => trimmed
            .parse::<i32>()
            .map(HeapInitDirective::I32)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i32 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.uint32" => trimmed
            .parse::<u32>()
            .map(HeapInitDirective::U32)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u32 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.int64" => trimmed
            .parse::<i64>()
            .map(HeapInitDirective::I64)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid i64 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.uint64" => trimmed
            .parse::<u64>()
            .map(HeapInitDirective::U64)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid u64 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.single" => trimmed
            .parse::<f32>()
            .map(HeapInitDirective::F32)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid f32 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.double" => trimmed
            .parse::<f64>()
            .map(HeapInitDirective::F64)
            .map_err(|e| {
                AsmError::new(format!(
                    "Line {}: invalid f64 init '{}': {}",
                    line_num, trimmed, e
                ))
            }),
        "system.string" => Ok(HeapInitDirective::String(parse_quoted_string(
            trimmed, line_num,
        )?)),
        _ => Err(AsmError::new(format!(
            "Line {}: unsupported typed init for '{}', use '{{ unserializable }}'.",
            line_num, type_name
        ))),
    }
}

fn parse_legacy_heap_init(trimmed: &str, line_num: usize) -> Result<HeapInitDirective> {
    if trimmed.eq_ignore_ascii_case("{\"null\":true}") {
        return Ok(HeapInitDirective::Null);
    }
    if let Some(v) = extract_json_bool(trimmed, "bool") {
        return Ok(HeapInitDirective::Bool(v));
    }
    if let Some(v) = extract_json_u32(trimmed, "u32") {
        return Ok(HeapInitDirective::U32(v));
    }
    if let Some(v) = extract_json_i32(trimmed, "i32") {
        return Ok(HeapInitDirective::I32(v));
    }
    if let Some(v) = extract_json_f32(trimmed, "f32") {
        return Ok(HeapInitDirective::F32(v));
    }
    if let Some(v) = extract_json_string(trimmed, "string") {
        return Ok(HeapInitDirective::String(v));
    }
    if let Some(v) = extract_json_u32_array(trimmed, "u32_array") {
        return Ok(HeapInitDirective::U32Array(v));
    }
    Err(AsmError::new(format!(
        "Line {}: unsupported heap init '{}'.",
        line_num, trimmed
    )))
}

fn parse_quoted_string(text: &str, line_num: usize) -> Result<String> {
    let trimmed = text.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err(AsmError::new(format!(
            "Line {}: string init must be quoted, got '{}'.",
            line_num, text
        )));
    }
    Ok(trimmed[1..trimmed.len() - 1]
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t"))
}

fn extract_json_bool(text: &str, key: &str) -> Option<bool> {
    let prefix = format!("{{\"{}\":", key);
    if !(text.starts_with(&prefix) && text.ends_with('}')) {
        return None;
    }
    let body = &text[prefix.len()..text.len() - 1];
    match body.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn extract_json_u32(text: &str, key: &str) -> Option<u32> {
    let prefix = format!("{{\"{}\":", key);
    if !(text.starts_with(&prefix) && text.ends_with('}')) {
        return None;
    }
    let body = &text[prefix.len()..text.len() - 1];
    body.trim().parse::<u32>().ok()
}

fn extract_json_i32(text: &str, key: &str) -> Option<i32> {
    let prefix = format!("{{\"{}\":", key);
    if !(text.starts_with(&prefix) && text.ends_with('}')) {
        return None;
    }
    let body = &text[prefix.len()..text.len() - 1];
    body.trim().parse::<i32>().ok()
}

fn extract_json_f32(text: &str, key: &str) -> Option<f32> {
    let prefix = format!("{{\"{}\":", key);
    if !(text.starts_with(&prefix) && text.ends_with('}')) {
        return None;
    }
    let body = &text[prefix.len()..text.len() - 1];
    body.trim().parse::<f32>().ok()
}

fn extract_json_string(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{{\"{}\":", key);
    if !(text.starts_with(&prefix) && text.ends_with('}')) {
        return None;
    }
    let body = text[prefix.len()..text.len() - 1].trim();
    if !(body.starts_with('"') && body.ends_with('"') && body.len() >= 2) {
        return None;
    }
    Some(
        body[1..body.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t"),
    )
}

fn extract_json_u32_array(text: &str, key: &str) -> Option<Vec<u32>> {
    let prefix = format!("{{\"{}\":[", key);
    if !(text.starts_with(&prefix) && text.ends_with("]}")) {
        return None;
    }
    let inside = &text[prefix.len()..text.len() - 2];
    if inside.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in inside.split(',') {
        out.push(part.trim().parse::<u32>().ok()?);
    }
    Some(out)
}
