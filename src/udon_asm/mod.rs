mod apply;
mod codec;
mod disassemble;
mod parse;
mod text;
mod types;

use base64::Engine as _;

use crate::odin::{NodeKind, UdonProgramBinary};

use apply::{
    apply_bind_directives, apply_bind_table_directives, apply_entry_directives,
    apply_heap_directives,
};
use codec::{build_heap_symbol_to_addr_map, encode_instructions};
use parse::parse_asm_text;

pub use types::{AsmError, AsmInstruction, OpCode, OperandToken};
pub type Result<T> = types::Result<T>;

pub fn disassemble_program_to_text(program: &UdonProgramBinary) -> Result<String> {
    disassemble::disassemble_program_to_text(program)
}

pub fn disassemble_program_to_text_with_indent(
    program: &UdonProgramBinary,
    indent: &str,
) -> Result<String> {
    disassemble::disassemble_program_to_text_with_indent(program, indent)
}

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

pub fn assemble_b64_with_original(original_b64: &str, asm_text: &str) -> Result<String> {
    let mut program = UdonProgramBinary::parse_base64(original_b64)?;
    assemble_text_on_program(&mut program, asm_text)?;
    let bytes = program.to_bytes()?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn decode_bytecode_to_asm_instructions(bytecode: &[u8]) -> Result<Vec<AsmInstruction>> {
    codec::decode_bytecode_to_asm_instructions(bytecode)
}

pub fn encode_asm_instructions_to_bytecode(instructions: &[AsmInstruction]) -> Result<Vec<u8>> {
    codec::encode_asm_instructions_to_bytecode(instructions)
}

pub(crate) fn render_heap_init_literal(type_name: &str, kind: &NodeKind) -> String {
    text::render_heap_init_literal(type_name, kind)
}
