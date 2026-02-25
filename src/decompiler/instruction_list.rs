use std::collections::HashMap;

use crate::udon_asm::{
    AsmInstruction, decode_bytecode_to_asm_instructions, encode_asm_instructions_to_bytecode,
};

use super::{DecompileError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstructionId(usize);

impl InstructionId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstructionList {
    instructions: Vec<AsmInstruction>,
    addresses: Vec<u32>,
    address_to_index: HashMap<u32, usize>,
}

impl InstructionList {
    pub fn from_bytecode(bytecode: &[u8]) -> Result<Self> {
        let instructions = decode_bytecode_to_asm_instructions(bytecode)
            .map_err(|e| DecompileError::new(e.to_string()))?;
        Self::from_asm_instructions(instructions)
    }

    pub fn from_asm_instructions(instructions: Vec<AsmInstruction>) -> Result<Self> {
        let mut list = Self {
            instructions,
            addresses: Vec::new(),
            address_to_index: HashMap::new(),
        };
        list.rebuild_indexes()?;
        Ok(list)
    }

    pub fn to_bytecode(&self) -> Result<Vec<u8>> {
        encode_asm_instructions_to_bytecode(&self.instructions)
            .map_err(|e| DecompileError::new(e.to_string()))
    }

    pub fn instructions(&self) -> &[AsmInstruction] {
        &self.instructions
    }

    pub fn instructions_mut(&mut self) -> &mut [AsmInstruction] {
        &mut self.instructions
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn first_id(&self) -> Option<InstructionId> {
        if self.instructions.is_empty() {
            None
        } else {
            Some(InstructionId(0))
        }
    }

    pub fn last_id(&self) -> Option<InstructionId> {
        self.instructions.len().checked_sub(1).map(InstructionId)
    }

    pub fn id_at_address(&self, address: u32) -> Option<InstructionId> {
        self.address_to_index
            .get(&address)
            .copied()
            .map(InstructionId)
    }

    pub fn address_of(&self, id: InstructionId) -> Option<u32> {
        self.addresses.get(id.0).copied()
    }

    pub fn get(&self, id: InstructionId) -> Option<&AsmInstruction> {
        self.instructions.get(id.0)
    }

    pub fn get_mut(&mut self, id: InstructionId) -> Option<&mut AsmInstruction> {
        self.instructions.get_mut(id.0)
    }

    pub fn next_of(&self, id: InstructionId) -> Option<InstructionId> {
        let next_index = id.0.checked_add(1)?;
        if next_index < self.instructions.len() {
            Some(InstructionId(next_index))
        } else {
            None
        }
    }

    pub fn prev_of(&self, id: InstructionId) -> Option<InstructionId> {
        id.0.checked_sub(1).map(InstructionId)
    }

    pub fn iter(&self) -> impl Iterator<Item = (InstructionId, u32, &AsmInstruction)> {
        self.instructions
            .iter()
            .enumerate()
            .map(move |(idx, inst)| (InstructionId(idx), self.addresses[idx], inst))
    }

    pub fn push(&mut self, instruction: AsmInstruction) -> Result<InstructionId> {
        self.instructions.push(instruction);
        self.rebuild_indexes()?;
        Ok(InstructionId(self.instructions.len() - 1))
    }

    pub fn insert(&mut self, index: usize, instruction: AsmInstruction) -> Result<InstructionId> {
        if index > self.instructions.len() {
            return Err(DecompileError::new(format!(
                "Insert index {} out of range (len={}).",
                index,
                self.instructions.len()
            )));
        }
        self.instructions.insert(index, instruction);
        self.rebuild_indexes()?;
        Ok(InstructionId(index))
    }

    pub fn remove(&mut self, id: InstructionId) -> Result<AsmInstruction> {
        if id.0 >= self.instructions.len() {
            return Err(DecompileError::new(format!(
                "Instruction id {} out of range (len={}).",
                id.0,
                self.instructions.len()
            )));
        }
        let removed = self.instructions.remove(id.0);
        self.rebuild_indexes()?;
        Ok(removed)
    }

    pub fn replace(&mut self, id: InstructionId, instruction: AsmInstruction) -> Result<()> {
        if id.0 >= self.instructions.len() {
            return Err(DecompileError::new(format!(
                "Instruction id {} out of range (len={}).",
                id.0,
                self.instructions.len()
            )));
        }
        self.instructions[id.0] = instruction;
        self.rebuild_indexes()
    }

    fn rebuild_indexes(&mut self) -> Result<()> {
        self.addresses.clear();
        self.address_to_index.clear();

        let mut pc = 0_u32;
        for (idx, inst) in self.instructions.iter().enumerate() {
            self.addresses.push(pc);
            self.address_to_index.insert(pc, idx);
            pc = pc.checked_add(inst.opcode.size()).ok_or_else(|| {
                DecompileError::new("Bytecode size overflow while rebuilding indexes.")
            })?;
        }
        Ok(())
    }
}
