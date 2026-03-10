use std::collections::{HashMap, HashSet};

use crate::str_constants::{SYMBOL_RETURN_JUMP_U32, TYPE_SYSTEM_UINT32_ARRAY};
use crate::udon_asm::OpCode;

use super::basic_block::SwitchTableInfo;
use super::context::DecompileContext;
use super::instruction_list::InstructionId;
use super::module_info::{UINT32_ARRAY_GET_METHOD_NAME, UdonModuleInfo};

pub(crate) fn collect_switch_target_block_starts(
    ctx: &DecompileContext,
    module_info: Option<&UdonModuleInfo>,
) -> Vec<u32> {
    let mut out = HashSet::<u32>::new();
    for (inst_id, _addr, inst) in ctx.instructions.iter() {
        if inst.opcode != OpCode::JumpIndirect {
            continue;
        }
        let operand = inst.numeric_operand();
        if is_return_jump_operand(operand, &ctx.symbol_name_by_address) {
            continue;
        }
        if let Some(info) = resolve_switch_info_for_jump_indirect(
            ctx,
            inst_id,
            operand,
            &ctx.symbol_name_by_address,
            &ctx.symbol_type_by_address,
            module_info,
        ) {
            for target in info.targets {
                if ctx.instructions.id_at_address(target).is_some() {
                    out.insert(target);
                }
            }
        }
    }
    let mut starts = out.into_iter().collect::<Vec<_>>();
    starts.sort_unstable();
    starts
}

pub(crate) fn is_return_jump_operand(
    operand: u32,
    symbol_name_by_address: &HashMap<u32, String>,
) -> bool {
    symbol_name_by_address
        .get(&operand)
        .is_some_and(|name| name == SYMBOL_RETURN_JUMP_U32)
}

pub(crate) fn resolve_switch_info_for_jump_indirect(
    ctx: &DecompileContext,
    jump_inst_id: InstructionId,
    jump_operand: u32,
    symbol_name_by_address: &HashMap<u32, String>,
    symbol_type_by_address: &HashMap<u32, String>,
    module_info: Option<&UdonModuleInfo>,
) -> Option<SwitchTableInfo> {
    let module_info = module_info?;
    let idx = jump_inst_id.index();
    if idx < 4 {
        return None;
    }
    let prev_extern_id = ctx.instructions.id_by_index(idx - 1)?;
    let prev_push_addr_id = ctx.instructions.id_by_index(idx - 2)?;
    let prev_push_index_id = ctx.instructions.id_by_index(idx - 3)?;
    let prev_push_table_id = ctx.instructions.id_by_index(idx - 4)?;

    let prev_extern = ctx.instructions.get(prev_extern_id)?;
    if prev_extern.opcode != OpCode::Extern {
        return None;
    }
    let extern_operand = prev_extern.numeric_operand();
    let extern_signature = ctx.heap_string_literals.get(&extern_operand)?;
    if extern_signature != UINT32_ARRAY_GET_METHOD_NAME {
        return None;
    }
    module_info.get_function_info(extern_signature)?;

    let prev_push_addr = ctx.instructions.get(prev_push_addr_id)?;
    if prev_push_addr.opcode != OpCode::Push {
        return None;
    }
    let push_addr_operand = prev_push_addr.numeric_operand();
    if push_addr_operand != jump_operand {
        return None;
    }
    if symbol_name_by_address.get(&push_addr_operand) != symbol_name_by_address.get(&jump_operand) {
        return None;
    }

    let prev_push_index = ctx.instructions.get(prev_push_index_id)?;
    if prev_push_index.opcode != OpCode::Push {
        return None;
    }
    let index_operand = prev_push_index.numeric_operand();

    let prev_push_table = ctx.instructions.get(prev_push_table_id)?;
    if prev_push_table.opcode != OpCode::Push {
        return None;
    }
    let table_operand = prev_push_table.numeric_operand();
    let is_u32_array_table = symbol_type_by_address
        .get(&table_operand)
        .map(|name| name.split(',').next().unwrap_or(name).trim())
        .is_some_and(|head| head == TYPE_SYSTEM_UINT32_ARRAY);
    if !is_u32_array_table {
        return None;
    }

    let targets = ctx.heap_u32_array_literals.get(&table_operand).cloned()?;
    Some(SwitchTableInfo {
        jump_address: ctx.instructions.address_of(jump_inst_id)?,
        targets,
        index_operand,
        table_operand,
        scaffold_instruction_addresses: vec![
            ctx.instructions.address_of(prev_push_table_id)?,
            ctx.instructions.address_of(prev_push_index_id)?,
            ctx.instructions.address_of(prev_push_addr_id)?,
            ctx.instructions.address_of(prev_extern_id)?,
        ],
    })
}
