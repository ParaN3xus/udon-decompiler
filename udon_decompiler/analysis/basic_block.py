from dataclasses import dataclass, field
from typing import Dict, List, Set, Optional
from enum import Enum

from udon_decompiler.models.program import HeapEntry
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.utils.logger import logger


class BasicBlockType(Enum):
    NORMAL = "normal"
    ENTRY = "entry"
    CONDITIONAL = "conditional"
    JUMP = "jump"


@dataclass
class BasicBlock:
    start_address: int
    end_address: int
    instructions: List[Instruction] = field(default_factory=list)
    predecessors: Set['BasicBlock'] = field(default_factory=set)
    successors: Set['BasicBlock'] = field(default_factory=set)
    block_type: BasicBlockType = BasicBlockType.NORMAL

    # if it's an function entry
    function_name: Optional[str] = None

    def __hash__(self):
        return hash(self.start_address)

    def __eq__(self, other):
        if not isinstance(other, BasicBlock):
            return False
        return self.start_address == other.start_address

    def __repr__(self) -> str:
        return (
            f"BasicBlock(0x{self.start_address:08x}-0x{self.end_address:08x}, "
            f"type={self.block_type.value}, "
            f"preds={len(self.predecessors)}, "
            f"succs={len(self.successors)})"
        )

    @property
    def last_instruction(self) -> Optional[Instruction]:
        return self.instructions[-1] if self.instructions else None

    @property
    def first_instruction(self) -> Optional[Instruction]:
        return self.instructions[0] if self.instructions else None

    def add_predecessor(self, block: 'BasicBlock') -> None:
        self.predecessors.add(block)

    def add_successor(self, block: 'BasicBlock') -> None:
        self.successors.add(block)

    def is_empty(self) -> bool:
        return len(self.instructions) == 0

    def contains_address(self, address: int) -> bool:
        return self.start_address <= address <= self.end_address


class BasicBlockIdentifier:
    def __init__(self, instructions: List[Instruction], entry_points: List[int], heap_initial_values: Dict[int, HeapEntry]):
        self.instructions = instructions
        self.entry_points = set(entry_points)
        self.heap_initial_values = heap_initial_values

        self._address_to_instruction = {
            inst.address: inst for inst in instructions
        }
        self._basic_blocks: List[BasicBlock] = []
        self._address_to_block: dict[int, BasicBlock] = {}

    def identify(self) -> List[BasicBlock]:
        logger.info("Identifying basic blocks...")

        block_starts = self._find_block_starts()
        logger.info(f"Found {len(block_starts)} block start addresses")

        self._basic_blocks = self._split_into_blocks(block_starts)
        logger.info(f"Created {len(self._basic_blocks)} basic blocks")

        self._build_address_mapping()

        return self._basic_blocks

    def _find_block_starts(self) -> Set[int]:
        block_starts = set()

        block_starts.update(self.entry_points)

        if self.instructions:
            block_starts.add(self.instructions[0].address)

        for inst in self.instructions:
            if inst.opcode == OpCode.JUMP or inst.opcode == OpCode.JUMP_IF_FALSE:
                target = inst.get_jump_target()
                if target is not None:
                    block_starts.add(target)

                next_addr = inst.next_address
                if self._address_to_instruction.get(next_addr):
                    block_starts.add(next_addr)

            elif inst.opcode == OpCode.JUMP_INDIRECT:
                if inst.operand is not None:
                    heap_entry = self.heap_initial_values.get(inst.operand)
                    if heap_entry and heap_entry.value.is_serializable:
                        target = heap_entry.value.value
                        if isinstance(target, int):
                            block_starts.add(target)
                            logger.debug(
                                f"Assuming indirect jump target {target} as block start from heap's initial value in addr {inst.operand}"
                            )

        return block_starts

    def _split_into_blocks(self, block_starts: Set[int]) -> List[BasicBlock]:
        sorted_starts = sorted(block_starts)
        blocks = []

        for i, start_addr in enumerate(sorted_starts):
            if i + 1 < len(sorted_starts):
                next_start = sorted_starts[i + 1]
                block_instructions = self._get_instructions_in_range(
                    start_addr, next_start
                )
            else:
                block_instructions = self._get_instructions_from(start_addr)

            if not block_instructions:
                continue

            end_addr = block_instructions[-1].address

            block_type = self._determine_block_type(
                start_addr, block_instructions
            )

            block = BasicBlock(
                start_address=start_addr,
                end_address=end_addr,
                instructions=block_instructions,
                block_type=block_type
            )

            blocks.append(block)

        return blocks

    def _get_instructions_in_range(
        self, start_addr: int, end_addr: int
    ) -> List[Instruction]:
        instructions = []
        current_addr = start_addr

        while current_addr < end_addr:
            inst = self._address_to_instruction.get(current_addr)
            if inst is None:
                break
            instructions.append(inst)
            current_addr = inst.next_address

        return instructions

    def _get_instructions_from(self, start_addr: int) -> List[Instruction]:
        instructions = []
        current_addr = start_addr

        while current_addr in self._address_to_instruction:
            inst = self._address_to_instruction[current_addr]
            instructions.append(inst)
            current_addr = inst.next_address

        return instructions

    def _determine_block_type(
        self, start_addr: int, instructions: List[Instruction]
    ) -> BasicBlockType:
        if start_addr in self.entry_points:
            return BasicBlockType.ENTRY

        if not instructions:
            return BasicBlockType.NORMAL

        last_inst = instructions[-1]

        if last_inst.is_conditional_jump():
            return BasicBlockType.CONDITIONAL
        elif last_inst.is_unconditional_jump():
            return BasicBlockType.JUMP

        return BasicBlockType.NORMAL

    def _build_address_mapping(self) -> None:
        for block in self._basic_blocks:
            self._address_to_block[block.start_address] = block

    def get_block_at(self, address: int) -> Optional[BasicBlock]:
        if address in self._address_to_block:
            return self._address_to_block[address]

        for block in self._basic_blocks:
            if block.contains_address(address):
                return block

        return None

    @property
    def basic_blocks(self) -> List[BasicBlock]:
        return self._basic_blocks
