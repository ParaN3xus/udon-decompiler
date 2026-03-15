use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, info};

use crate::odin::{SymbolSection, UdonProgramBinary};
use crate::str_constants::{
    TYPE_UNSERIALIZABLE, UNSERIALIZABLE_LITERAL,
};
use crate::udon_asm::{
    HeapLiteralValue, render_heap_literal, resolve_heap_literal_for_program_entry,
};
use crate::util::sanitize_output_stem;

use super::basic_block::BasicBlockCollection;
use super::cfg::{FunctionCfg, StackSimulationResult};
use super::pipeline::DecompilePipelineOutput;
use super::variable::VariableTable;
use super::{InstructionList, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct DecompileSymbol {
    pub name: String,
    pub address: u32,
    pub exported: bool,
    pub type_name: String,
}

impl DecompileSymbol {
    pub fn entry_call_jump_target(&self, ctx: &DecompileContext) -> u32 {
        let Some(inst_id) = ctx.instructions.id_at_address(self.address) else {
            return self.address;
        };
        let Some(inst) = ctx.instructions.get(inst_id) else {
            return self.address;
        };
        if inst.opcode != crate::udon_asm::OpCode::Push {
            return self.address;
        }
        let push_operand = inst.numeric_operand();

        let is_halt_literal = ctx
            .heap_u32_literals
            .get(&push_operand)
            .copied()
            .is_some_and(|x| ctx.is_out_of_program_counter_range(x));
        if !is_halt_literal {
            return self.address;
        }

        // pushing halt jump addr
        ctx.instructions
            .next_of(inst_id)
            .and_then(|next_id| ctx.instructions.address_of(next_id))
            .unwrap_or(self.address)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecompileHeapEntry {
    pub index: usize,
    pub address: u32,
    pub type_name: String,
    pub init_value: String,
}

#[derive(Debug, Clone)]
pub struct DecompileContext {
    pub input_path: Option<PathBuf>,
    pub input_file_name: Option<String>,
    pub bytecode: Vec<u8>,
    pub instructions: InstructionList,
    pub heap_capacity: u32,
    pub heap_entries: Vec<DecompileHeapEntry>,
    pub entry_points: Vec<DecompileSymbol>,
    pub symbols: Vec<DecompileSymbol>,
}

impl DecompileContext {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        info!(path = %path.display(), "loading decompile context from b64 file");
        let raw = fs::read_to_string(path)?;
        let program = UdonProgramBinary::parse_base64(&raw)?;
        let mut ctx = Self::from_program(&program)?;
        ctx.input_path = Some(path.to_path_buf());
        ctx.input_file_name = path
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string());
        Ok(ctx)
    }

    pub fn from_base64_text(text: &str, input_file_name: Option<String>) -> Result<Self> {
        debug!(
            input_file_name = ?input_file_name,
            input_len = text.len(),
            "loading decompile context from base64 text"
        );
        let program = UdonProgramBinary::parse_base64(text)?;
        let mut ctx = Self::from_program(&program)?;
        ctx.input_file_name = input_file_name;
        Ok(ctx)
    }

    pub fn from_program(program: &UdonProgramBinary) -> Result<Self> {
        let bytecode = program.byte_code()?;
        let instructions = InstructionList::from_bytecode(&bytecode)?;
        let heap_capacity = program.heap_capacity()?;
        let heap_entries = load_heap_entries(program)?;
        let entry_points = load_symbols(program, SymbolSection::EntryPoints)?;
        let symbols = load_symbols(program, SymbolSection::SymbolTable)?;

        let ctx = Self {
            input_path: None,
            input_file_name: None,
            bytecode,
            instructions,
            heap_capacity,
            heap_entries,
            entry_points,
            symbols,
        };
        info!(
            bytecode_len = ctx.bytecode.len(),
            instruction_count = ctx.instructions.len(),
            entry_points = ctx.entry_points.len(),
            symbols = ctx.symbols.len(),
            heap_entries = ctx.heap_entries.len(),
            "decompile context loaded"
        );
        Ok(ctx)
    }

    pub fn entry_point_index_by_name(&self, name: &str) -> Option<usize> {
        self.entry_points.iter().position(|x| x.name == name)
    }

    pub fn entry_point_index_by_address(&self, address: u32) -> Option<usize> {
        self.entry_points.iter().position(|x| x.address == address)
    }

    pub fn symbol_index_by_name(&self, name: &str) -> Option<usize> {
        self.symbols.iter().position(|x| x.name == name)
    }

    pub fn symbol_index_by_address(&self, address: u32) -> Option<usize> {
        self.symbols.iter().position(|x| x.address == address)
    }

    pub fn heap_index_by_address(&self, address: u32) -> Option<usize> {
        self.heap_entries.iter().position(|x| x.address == address)
    }

    pub fn sync_bytecode_from_instructions(&mut self) -> Result<()> {
        debug!(
            instruction_count = self.instructions.len(),
            "syncing bytecode from instruction list"
        );
        self.bytecode = self.instructions.to_bytecode()?;
        Ok(())
    }

    pub fn reload_instructions_from_bytecode(&mut self) -> Result<()> {
        debug!(
            bytecode_len = self.bytecode.len(),
            "reloading instruction list from bytecode"
        );
        self.instructions = InstructionList::from_bytecode(&self.bytecode)?;
        Ok(())
    }
}

fn load_symbols(
    program: &UdonProgramBinary,
    section: SymbolSection,
) -> Result<Vec<DecompileSymbol>> {
    let exported = exported_name_set(program, section)?;
    let len = program.symbols_len(section)?;
    let mut out = Vec::<DecompileSymbol>::with_capacity(len);
    for i in 0..len {
        let item = program.symbol_item(section, i)?;
        let type_name = program
            .symbol_type_name_string(section, i)?
            .unwrap_or_else(|| TYPE_UNSERIALIZABLE.to_string());
        out.push(DecompileSymbol {
            exported: exported.contains(&item.name),
            name: item.name,
            address: item.address,
            type_name,
        });
    }
    Ok(out)
}

fn exported_name_set(
    program: &UdonProgramBinary,
    section: SymbolSection,
) -> Result<HashSet<String>> {
    let len = program.exported_symbols_len(section)?;
    let mut out = HashSet::<String>::with_capacity(len);
    for i in 0..len {
        out.insert(program.exported_symbol(section, i)?);
    }
    Ok(out)
}

struct LoadedHeapData {
    entries: Vec<DecompileHeapEntry>,
    u32_literals: HashMap<u32, u32>,
    string_literals: HashMap<u32, String>,
    u32_array_literals: HashMap<u32, Vec<u32>>,
}

fn load_heap_data(program: &UdonProgramBinary) -> Result<LoadedHeapData> {
    let len = program.heap_dump_len()?;
    let mut out = Vec::<DecompileHeapEntry>::with_capacity(len);
    let mut u32_literals = HashMap::<u32, u32>::new();
    let mut string_literals = HashMap::<u32, String>::new();
    let mut u32_array_literals = HashMap::<u32, Vec<u32>>::new();
    for i in 0..len {
        let item = program.heap_dump_item(i)?;
        let maybe_type_name = program.heap_dump_type_name_string(i)?;
        let value_kind = program.heap_dump_strongbox_value_kind(i)?;
        let (type_name, init_value, literal) = if let Some(type_name) = maybe_type_name {
            let literal =
                resolve_heap_literal_for_program_entry(program, i, &type_name, &value_kind)?;
            (
                type_name.clone(),
                render_heap_literal(&type_name, &literal),
                literal,
            )
        } else {
            (
                TYPE_UNSERIALIZABLE.to_string(),
                UNSERIALIZABLE_LITERAL.to_string(),
                HeapLiteralValue::Unserializable,
            )
        };
        if let Some(v) = literal.as_u32() {
            u32_literals.insert(item.address, v);
        }
        if let Some(v) = literal.as_string() {
            string_literals.insert(item.address, v.to_string());
        }
        if let Some(v) = literal.as_u32_array() {
            u32_array_literals.insert(item.address, v.to_vec());
        }
        out.push(DecompileHeapEntry {
            address: item.address,
            type_name,
            init_value,
        });
    }
    Ok(LoadedHeapData {
        entries: out,
        u32_literals,
        string_literals,
        u32_array_literals,
    })
}
