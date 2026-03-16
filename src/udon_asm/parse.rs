use super::literal::{parse_heap_init_directive, parse_type_ref};
use super::types::{
    AsmDirectives, AsmError, AsmInstruction, BindDirective, BindTableDirective, DirectiveSection,
    EntryDirective, HeapDirective, HeapExportMark, OpCode, OperandToken, ParsedAsm, Result,
    Visibility,
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
            if body == "entry points" {
                section_hint = Some(DirectiveSection::EntryPoints);
                continue;
            }
            if body == "heap" {
                section_hint = Some(DirectiveSection::Heap);
                continue;
            }
            if body == "binds" {
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
        || body == "syntax"
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
    let init = parse_heap_init_directive(
        init_text,
        line_num,
        match &type_ref {
            super::types::TypeRefDirective::Name(name) => name.as_str(),
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
