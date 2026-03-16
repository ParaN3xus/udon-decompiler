use std::collections::{HashMap, HashSet};

use crate::odin::{SymbolSection, UdonProgramBinary};

use super::codec::{build_label_map, decode_instructions};
use super::literal::{TYPE_UNSERIALIZABLE, UNSERIALIZABLE_LITERAL};
use super::text::generated_heap_symbol;
use super::types::Result;
use super::{AsmBindDirective, AsmBindTableDirective, AsmInstructionComment};
use super::{render_heap_literal, resolve_heap_literal_for_program_entry};

pub fn disassemble_program_to_text(
    program: &UdonProgramBinary,
    binds: &[AsmBindDirective],
    bind_tables: &[AsmBindTableDirective],
    instruction_comments: &[AsmInstructionComment],
) -> Result<String> {
    let indent = "  ";
    let bytecode = program.byte_code()?;
    let decoded = decode_instructions(&bytecode)?;

    let symbol_count = program.symbols_len(SymbolSection::SymbolTable)?;
    let mut heap_addr_to_symbol = HashMap::<u32, String>::new();
    for i in 0..symbol_count {
        let item = program.symbol_item(SymbolSection::SymbolTable, i)?;
        heap_addr_to_symbol.insert(item.address, item.name.clone());
    }
    let heap_len = program.heap_dump_len()?;
    for i in 0..heap_len {
        let item = program.heap_dump_item(i)?;
        heap_addr_to_symbol
            .entry(item.address)
            .or_insert_with(|| generated_heap_symbol(item.address));
    }

    let entry_count = program.symbols_len(SymbolSection::EntryPoints)?;
    let mut entry_address_to_name = HashMap::<u32, String>::new();
    for i in 0..entry_count {
        let entry = program.symbol_item(SymbolSection::EntryPoints, i)?;
        entry_address_to_name.insert(entry.address, entry.name);
    }
    let exported_entry_count = program.exported_symbols_len(SymbolSection::EntryPoints)?;
    let mut exported_entry_names = HashSet::<String>::new();
    for i in 0..exported_entry_count {
        exported_entry_names.insert(program.exported_symbol(SymbolSection::EntryPoints, i)?);
    }

    let extra_label_addresses = binds
        .iter()
        .map(|x| x.address)
        .chain(bind_tables.iter().flat_map(|x| x.addresses.iter().copied()))
        .collect::<Vec<_>>();
    let label_map = build_label_map(&decoded, &entry_address_to_name, &extra_label_addresses);
    let instruction_addresses = decoded.iter().map(|x| x.address).collect::<HashSet<_>>();
    let comment_by_address = instruction_comments
        .iter()
        .map(|x| (x.address, x.text.as_str()))
        .collect::<HashMap<_, _>>();

    let mut lines = Vec::<String>::new();
    lines.push("; syntax".to_string());
    lines.push(format!("; {}entry: [+/-] <entry_name> @ <label>", indent));
    lines.push(format!(
        "; {}heap:  [+/-/~] <symbol>: {{ <type_name> }} = <init_literal>",
        indent
    ));
    lines.push(format!("; {}bind:  bind <symbol> -> <label>", indent));
    lines.push(format!(
        "; {}bind-table: bind-table <symbol> -> [<label>, ...]",
        indent
    ));
    lines.push(String::new());

    lines.push("; entry points".to_string());
    for i in 0..entry_count {
        let entry = program.symbol_item(SymbolSection::EntryPoints, i)?;
        let label = label_map
            .get(&entry.address)
            .cloned()
            .unwrap_or_else(|| format!("addr_0x{:08X}", entry.address));
        let vis = if exported_entry_names.contains(&entry.name) {
            '+'
        } else {
            '-'
        };
        lines.push(format!("; {}{} {} @ {}", indent, vis, entry.name, label));
    }

    let exported_symbol_count = program.exported_symbols_len(SymbolSection::SymbolTable)?;
    let mut exported_symbol_names = HashSet::<String>::new();
    for i in 0..exported_symbol_count {
        exported_symbol_names.insert(program.exported_symbol(SymbolSection::SymbolTable, i)?);
    }
    let mut symbol_table_names = HashSet::<String>::new();
    for i in 0..symbol_count {
        symbol_table_names.insert(program.symbol_item(SymbolSection::SymbolTable, i)?.name);
    }

    lines.push("; heap".to_string());
    let mut heap_lines = Vec::<(u32, String, String, String)>::new();
    for i in 0..heap_len {
        let item = program.heap_dump_item(i)?;
        let symbol = heap_addr_to_symbol
            .get(&item.address)
            .cloned()
            .unwrap_or_else(|| generated_heap_symbol(item.address));
        let resolved_value_kind = program.heap_dump_strongbox_value_kind(i)?;
        if let Some(type_name) = program.heap_dump_type_name_string(i)? {
            let literal = resolve_heap_literal_for_program_entry(
                program,
                i,
                &type_name,
                &resolved_value_kind,
            )
            .map_err(|e| super::types::AsmError::new(e.to_string()))?;
            let init_text = render_heap_literal(&type_name, &literal);
            let mark = if symbol_table_names.contains(&symbol) {
                if exported_symbol_names.contains(&symbol) {
                    '+'
                } else {
                    '-'
                }
            } else {
                '~'
            };
            heap_lines.push((
                item.address,
                format!("{mark} {symbol}"),
                type_name,
                init_text,
            ));
            continue;
        }
        heap_lines.push((
            item.address,
            format!("~ {symbol}"),
            TYPE_UNSERIALIZABLE.to_string(),
            UNSERIALIZABLE_LITERAL.to_string(),
        ));
    }
    heap_lines.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    for (_address, symbol, type_name, init_text) in heap_lines {
        lines.push(format!(
            "; {}{}: {{ {} }} = {}",
            indent, symbol, type_name, init_text
        ));
    }

    lines.push("; binds".to_string());
    for bind in binds {
        let label = label_map
            .get(&bind.address)
            .cloned()
            .unwrap_or_else(|| format!("addr_0x{:08X}", bind.address));
        lines.push(format!("; {}bind {} -> {}", indent, bind.symbol, label));
    }
    for bind in bind_tables {
        let labels = bind
            .addresses
            .iter()
            .map(|address| {
                label_map
                    .get(address)
                    .cloned()
                    .unwrap_or_else(|| format!("addr_0x{:08X}", address))
            })
            .collect::<Vec<_>>();
        lines.push(format!(
            "; {}bind-table {} -> [{}]",
            indent,
            bind.symbol,
            labels.join(", ")
        ));
    }
    lines.push(String::new());

    for inst in &decoded {
        if let Some(label) = label_map.get(&inst.address) {
            lines.push(format!("{}: ; @ 0x{:08X}", label, inst.address));
        }
        let op_name = inst.opcode.name();
        let trailing_comment = comment_by_address
            .get(&inst.address)
            .map(|text| format!(" ; {}", text))
            .unwrap_or_default();
        if let Some(operand) = inst.operand {
            if inst.opcode.is_direct_jump() {
                if instruction_addresses.contains(&operand) {
                    let operand_label = label_map
                        .get(&operand)
                        .cloned()
                        .unwrap_or_else(|| format!("addr_0x{:08X}", operand));
                    lines.push(format!(
                        "{}{} {}{}",
                        indent, op_name, operand_label, trailing_comment
                    ));
                } else {
                    lines.push(format!(
                        "{}{} 0x{:08X}{}",
                        indent, op_name, operand, trailing_comment
                    ));
                }
            } else if inst.opcode.is_heap_operand() {
                let rendered = if let Some(symbol) = heap_addr_to_symbol.get(&operand) {
                    symbol.clone()
                } else {
                    generated_heap_symbol(operand)
                };
                lines.push(format!(
                    "{}{} {}{}",
                    indent, op_name, rendered, trailing_comment
                ));
            } else {
                lines.push(format!(
                    "{}{} 0x{:08X}{}",
                    indent, op_name, operand, trailing_comment
                ));
            }
        } else {
            lines.push(format!("{}{}{}", indent, op_name, trailing_comment));
        }
    }

    Ok(lines.join("\n"))
}
