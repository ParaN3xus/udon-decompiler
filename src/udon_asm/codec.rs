use std::collections::{BTreeMap, HashMap, HashSet};

use crate::odin::{SymbolSection, UdonProgramBinary};

use super::text::{generated_heap_symbol, parse_generated_heap_symbol, sanitize_label};
use super::types::{AsmError, AsmInstruction, DecodedInstruction, OpCode, OperandToken, Result};

pub(crate) fn decode_instructions(bytes: &[u8]) -> Result<Vec<DecodedInstruction>> {
    let mut out = Vec::<DecodedInstruction>::new();
    let mut pc = 0_usize;
    while pc < bytes.len() {
        if pc + 4 > bytes.len() {
            return Err(AsmError::new(format!(
                "Truncated opcode at byte offset {}.",
                pc
            )));
        }
        let opcode_u32 =
            u32::from_be_bytes([bytes[pc], bytes[pc + 1], bytes[pc + 2], bytes[pc + 3]]);
        let opcode = OpCode::from_u32(opcode_u32)?;
        let operand = if opcode.has_operand() {
            if pc + 8 > bytes.len() {
                return Err(AsmError::new(format!(
                    "Truncated operand at byte offset {}.",
                    pc
                )));
            }
            Some(u32::from_be_bytes([
                bytes[pc + 4],
                bytes[pc + 5],
                bytes[pc + 6],
                bytes[pc + 7],
            ]))
        } else {
            None
        };
        out.push(DecodedInstruction {
            address: pc as u32,
            opcode,
            operand,
        });
        pc += opcode.size() as usize;
    }
    Ok(out)
}

pub fn decode_bytecode_to_asm_instructions(bytes: &[u8]) -> Result<Vec<AsmInstruction>> {
    let decoded = decode_instructions(bytes)?;
    Ok(decoded
        .into_iter()
        .map(|x| AsmInstruction {
            labels: Vec::new(),
            opcode: x.opcode,
            operand: x.operand.map(OperandToken::Number),
        })
        .collect())
}

pub(crate) fn build_label_map(
    decoded: &[DecodedInstruction],
    entry_address_to_name: &HashMap<u32, String>,
) -> BTreeMap<u32, String> {
    let mut addresses = HashSet::<u32>::new();
    for inst in decoded {
        if inst.opcode.is_direct_jump()
            && let Some(op) = inst.operand
        {
            addresses.insert(op);
        }
    }
    for address in entry_address_to_name.keys() {
        addresses.insert(*address);
    }
    let mut sorted = addresses.into_iter().collect::<Vec<_>>();
    sorted.sort_unstable();
    let mut out = BTreeMap::<u32, String>::new();
    for addr in sorted {
        if let Some(name) = entry_address_to_name.get(&addr) {
            out.insert(addr, format!("entry_{}", sanitize_label(name)));
        } else {
            out.insert(addr, format!("addr_0x{:08X}", addr));
        }
    }
    out
}

pub(crate) fn build_heap_symbol_to_addr_map(
    program: &UdonProgramBinary,
) -> Result<HashMap<String, u32>> {
    let len = program.symbols_len(SymbolSection::SymbolTable)?;
    let mut map = HashMap::<String, u32>::new();
    for i in 0..len {
        let sym = program.symbol_item(SymbolSection::SymbolTable, i)?;
        map.insert(sym.name, sym.address);
    }
    let heap_len = program.heap_dump_len()?;
    for i in 0..heap_len {
        let item = program.heap_dump_item(i)?;
        map.entry(generated_heap_symbol(item.address))
            .or_insert(item.address);
    }
    Ok(map)
}

pub(crate) fn encode_instructions(
    instructions: &[AsmInstruction],
    heap_symbol_to_addr: &HashMap<String, u32>,
) -> Result<(HashMap<String, u32>, Vec<u8>)> {
    let mut label_to_addr = HashMap::<String, u32>::new();
    let mut pc = 0_u32;
    for inst in instructions {
        for label in &inst.labels {
            if label_to_addr.insert(label.clone(), pc).is_some() {
                return Err(AsmError::new(format!("Duplicate label '{}'.", label)));
            }
        }
        pc = pc
            .checked_add(inst.opcode.size())
            .ok_or_else(|| AsmError::new("Bytecode size overflow while resolving labels."))?;
    }

    let mut out = Vec::<u8>::new();
    for inst in instructions {
        out.extend_from_slice(&(inst.opcode as u32).to_be_bytes());
        if inst.opcode.has_operand() {
            let operand = inst.operand.as_ref().ok_or_else(|| {
                AsmError::new(format!("Opcode {} missing operand.", inst.opcode.name()))
            })?;
            let value = resolve_operand(inst.opcode, operand, &label_to_addr, heap_symbol_to_addr)?;
            out.extend_from_slice(&value.to_be_bytes());
        }
    }

    Ok((label_to_addr, out))
}

pub fn encode_asm_instructions_to_bytecode(
    instructions: &[AsmInstruction],
) -> Result<Vec<u8>> {
    let mut out = Vec::<u8>::new();
    for inst in instructions {
        if !inst.labels.is_empty() {
            return Err(AsmError::new(
                "Bytecode encoding does not accept labels on AsmInstruction.",
            ));
        }
        out.extend_from_slice(&(inst.opcode as u32).to_be_bytes());
        if inst.opcode.has_operand() {
            let operand = inst.operand.as_ref().ok_or_else(|| {
                AsmError::new(format!("Opcode {} missing operand.", inst.opcode.name()))
            })?;
            let value = match operand {
                OperandToken::Number(v) => *v,
                OperandToken::Label(_) | OperandToken::HeapSymbol(_) => {
                    return Err(AsmError::new(
                        "Bytecode encoding expects numeric operands only.",
                    ));
                }
            };
            out.extend_from_slice(&value.to_be_bytes());
        } else if inst.operand.is_some() {
            return Err(AsmError::new(format!(
                "Opcode {} does not take an operand.",
                inst.opcode.name()
            )));
        }
    }
    Ok(out)
}

fn resolve_operand(
    opcode: OpCode,
    operand: &OperandToken,
    label_to_addr: &HashMap<String, u32>,
    heap_symbol_to_addr: &HashMap<String, u32>,
) -> Result<u32> {
    if opcode.is_direct_jump() {
        return match operand {
            OperandToken::Label(label) => label_to_addr
                .get(label)
                .copied()
                .ok_or_else(|| AsmError::new(format!("Unknown jump label '{}'.", label))),
            OperandToken::Number(v) => Ok(*v),
            OperandToken::HeapSymbol(symbol) => Err(AsmError::new(format!(
                "Jump operand cannot be heap symbol '{}'.",
                symbol
            ))),
        };
    }

    if opcode.is_heap_operand() {
        return match operand {
            OperandToken::HeapSymbol(symbol) => heap_symbol_to_addr
                .get(symbol)
                .copied()
                .or_else(|| parse_generated_heap_symbol(symbol))
                .ok_or_else(|| AsmError::new(format!("Unknown heap symbol '{}'.", symbol))),
            OperandToken::Number(v) => Ok(*v),
            OperandToken::Label(label) => Err(AsmError::new(format!(
                "Heap operand cannot be label '{}'.",
                label
            ))),
        };
    }

    match operand {
        OperandToken::Number(v) => Ok(*v),
        _ => Err(AsmError::new(format!(
            "Opcode {} expects numeric operand.",
            opcode.name()
        ))),
    }
}
