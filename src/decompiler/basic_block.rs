use std::collections::{BTreeSet, HashMap};

use super::context::DecompileContext;
use super::indir_jump_analysis::collect_switch_target_block_starts;
use super::instruction_list::InstructionList;
use super::module_info::UdonModuleInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicBlockType {
    Normal,
    Conditional,
    Jump,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchTableInfo {
    pub jump_address: u32,
    pub targets: Vec<u32>,
    pub index_operand: u32,
    pub table_operand: u32,
    pub scaffold_instruction_addresses: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub instructions: InstructionList,
    pub predecessors: Vec<usize>,
    pub successors: Vec<usize>,
    pub block_type: BasicBlockType,
    pub switch_info: Option<SwitchTableInfo>,
}

impl BasicBlock {
    pub fn start_address(&self) -> Option<u32> {
        self.instructions
            .first_id()
            .and_then(|id| self.instructions.address_of(id))
    }

    pub fn end_address(&self) -> Option<u32> {
        self.instructions
            .last_id()
            .and_then(|id| self.instructions.address_of(id))
    }

    pub fn contains_address(&self, address: u32) -> bool {
        match (self.start_address(), self.end_address()) {
            (Some(start), Some(end)) => start <= address && address <= end,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BasicBlockCollection {
    pub blocks: Vec<BasicBlock>,
    start_to_block: HashMap<u32, usize>,
}

impl BasicBlockCollection {
    pub fn identify_from_context(ctx: &DecompileContext) -> Self {
        let entry_addresses = ctx
            .entry_points
            .iter()
            .map(|x| x.address)
            .collect::<Vec<_>>();
        let module_info = UdonModuleInfo::load_default_cached().ok();
        let extra_block_starts = collect_switch_target_block_starts(ctx, module_info);
        Self::identify(&ctx.instructions, &entry_addresses, &extra_block_starts)
    }

    pub fn identify(
        instructions: &InstructionList,
        entry_points: &[u32],
        extra_block_starts: &[u32],
    ) -> Self {
        let mut starts = find_block_starts(instructions, entry_points);
        for address in extra_block_starts {
            if instructions.id_at_address(*address).is_some() {
                starts.insert(*address);
            }
        }
        if let Some((_, first_addr, _)) = instructions.iter().next() {
            starts.insert(first_addr);
        }

        let blocks = split_into_blocks(instructions, &starts);
        let mut out = Self {
            blocks,
            start_to_block: HashMap::new(),
        };
        out.rebuild_start_index();
        out.build_edges(instructions);
        out
    }

    pub fn get_block_at(&self, address: u32) -> Option<&BasicBlock> {
        if let Some(block_id) = self.start_to_block.get(&address) {
            return self.blocks.get(*block_id);
        }
        self.blocks.iter().find(|x| x.contains_address(address))
    }

    pub fn get_block_by_start(&self, start_address: u32) -> Option<&BasicBlock> {
        self.start_to_block
            .get(&start_address)
            .and_then(|idx| self.blocks.get(*idx))
    }

    pub fn get_block_id_by_start(&self, start_address: u32) -> Option<usize> {
        self.start_to_block.get(&start_address).copied()
    }

    fn rebuild_start_index(&mut self) {
        self.start_to_block.clear();
        for (idx, block) in self.blocks.iter().enumerate() {
            if let Some(start_address) = block.start_address() {
                self.start_to_block.insert(start_address, idx);
            }
        }
    }

    fn build_edges(&mut self, instructions: &InstructionList) {
        for block in &mut self.blocks {
            block.predecessors.clear();
            block.successors.clear();
        }

        for block_id in 0..self.blocks.len() {
            let Some(last_inst_id) = self.blocks[block_id].instructions.last_id() else {
                continue;
            };
            let block_instructions = &self.blocks[block_id].instructions;
            let Some(last_inst) = block_instructions.get(last_inst_id) else {
                continue;
            };
            let next_addr = block_instructions
                .address_of(last_inst_id)
                .and_then(|addr| {
                    instructions
                        .id_at_address(addr)
                        .and_then(|id| instructions.next_of(id))
                        .and_then(|next_id| instructions.address_of(next_id))
                });

            match last_inst.opcode {
                crate::udon_asm::OpCode::Jump => {
                    if let Some(target) = last_inst.numeric_operand() {
                        connect_by_start(self, block_id, target);
                    }
                }
                crate::udon_asm::OpCode::JumpIfFalse => {
                    if let Some(target) = last_inst.numeric_operand() {
                        connect_by_start(self, block_id, target);
                    }
                    if let Some(addr) = next_addr {
                        connect_by_start(self, block_id, addr);
                    }
                }
                crate::udon_asm::OpCode::JumpIndirect => {}
                _ => {
                    if let Some(addr) = next_addr {
                        connect_by_start(self, block_id, addr);
                    }
                }
            }
        }
    }
}

fn find_block_starts(instructions: &InstructionList, entry_points: &[u32]) -> BTreeSet<u32> {
    let mut starts = entry_points.iter().copied().collect::<BTreeSet<_>>();
    for (id, _addr, inst) in instructions.iter() {
        match inst.opcode {
            crate::udon_asm::OpCode::Jump | crate::udon_asm::OpCode::JumpIfFalse => {
                if let Some(target) = inst.numeric_operand()
                    && instructions.id_at_address(target).is_some()
                {
                    starts.insert(target);
                }
                if let Some(next_addr) = instructions
                    .next_of(id)
                    .and_then(|next_id| instructions.address_of(next_id))
                {
                    starts.insert(next_addr);
                }
            }
            crate::udon_asm::OpCode::JumpIndirect => {
                // return/jump-indirect often ends the block, and the next instruction
                // should start a new block if it exists.
                if let Some(next_addr) = instructions
                    .next_of(id)
                    .and_then(|next_id| instructions.address_of(next_id))
                {
                    starts.insert(next_addr);
                }
            }
            _ => {}
        }
    }
    starts
}

fn split_into_blocks(instructions: &InstructionList, starts: &BTreeSet<u32>) -> Vec<BasicBlock> {
    let sorted_starts = starts.iter().copied().collect::<Vec<_>>();
    let mut blocks = Vec::<BasicBlock>::new();

    for (i, start_addr) in sorted_starts.iter().copied().enumerate() {
        let Some(start_id) = instructions.id_at_address(start_addr) else {
            continue;
        };
        let start_index = start_id.index();
        let end_index = sorted_starts
            .get(i + 1)
            .and_then(|addr| instructions.id_at_address(*addr))
            .map(|id| id.index())
            .unwrap_or(instructions.len());
        if start_index >= end_index {
            continue;
        }
        let Ok(instruction_list) = instructions.slice_by_index_range(start_index, end_index) else {
            continue;
        };
        blocks.push(BasicBlock {
            instructions: instruction_list,
            predecessors: Vec::new(),
            successors: Vec::new(),
            block_type: BasicBlockType::Normal,
            switch_info: None,
        });
    }
    blocks
}

fn connect_by_start(blocks: &mut BasicBlockCollection, source: usize, target_start: u32) {
    let Some(target) = blocks.start_to_block.get(&target_start).copied() else {
        return;
    };
    if !blocks.blocks[source].successors.contains(&target) {
        blocks.blocks[source].successors.push(target);
    }
    if !blocks.blocks[target].predecessors.contains(&source) {
        blocks.blocks[target].predecessors.push(source);
    }
}
