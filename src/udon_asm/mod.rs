mod analysis;
mod apply;
mod codec;
mod disassemble;
mod literal;
mod parse;
mod text;
mod types;

use crate::odin::UdonProgramBinary;
use crate::util::{decode_compressed_hex_text, encode_compressed_hex_bytes};

use apply::{
    apply_bind_directives, apply_bind_table_directives, apply_entry_directives,
    apply_heap_directives,
};
use codec::{build_heap_symbol_to_addr_map, encode_instructions};
pub use disassemble::disassemble_program_to_text;
use parse::parse_asm_text;

pub use analysis::{AsmBindAnalysis, collect_asm_bind_analysis, collect_asm_instruction_comments};
pub use codec::{decode_bytecode_to_asm_instructions, encode_asm_instructions_to_bytecode};
pub use types::{
    AsmBindDirective, AsmBindTableDirective, AsmError, AsmInstruction, AsmInstructionComment,
    OpCode, OperandToken,
};
pub type Result<T> = types::Result<T>;
pub(crate) use literal::{
    HeapLiteralValue, render_heap_literal, resolve_heap_literal_for_program_entry,
};
pub(crate) use text::generated_heap_symbol;

pub fn assemble_text_on_program(program: &mut UdonProgramBinary, asm_text: &str) -> Result<()> {
    let parsed = parse_asm_text(asm_text)?;
    if parsed.instructions.is_empty() {
        return Err(AsmError::new("ASM has no instructions."));
    }

    let (label_to_addr, encoded_bytecode) = encode_instructions(
        &parsed.instructions,
        &build_heap_symbol_to_addr_map(program)?,
    )?;
    program.set_byte_code(&encoded_bytecode)?;

    apply_entry_directives(program, &parsed.directives.entries, &label_to_addr)?;
    apply_heap_directives(program, &parsed.directives.heap)?;
    apply_bind_directives(program, &parsed.directives.binds, &label_to_addr)?;
    apply_bind_table_directives(program, &parsed.directives.bind_tables, &label_to_addr)?;

    Ok(())
}

pub fn assemble_hex_with_original(original_hex: &str, asm_text: &str) -> Result<String> {
    let bytes =
        decode_compressed_hex_text(original_hex).map_err(|e| AsmError::new(e.to_string()))?;
    let mut program = UdonProgramBinary::parse_bytes(&bytes)?;
    assemble_text_on_program(&mut program, asm_text)?;
    let bytes = program.to_bytes()?;
    encode_compressed_hex_bytes(&bytes).map_err(|e| AsmError::new(e.to_string()))
}
