from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional, Set

from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import UdonModuleInfo
from udon_decompiler.models.program import SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class BasicBlockType(Enum):
    NORMAL = "normal"
    CONDITIONAL = "conditional"
    JUMP = "jump"
    RETURN = "return"


@dataclass
class BasicBlock:
    start_address: int
    end_address: int
    instructions: List[Instruction] = field(default_factory=list)
    predecessors: Set["BasicBlock"] = field(default_factory=set)
    successors: Set["BasicBlock"] = field(default_factory=set)
    block_type: BasicBlockType = BasicBlockType.NORMAL
    is_entry: bool = False

    # if it's a function entry
    function_name: Optional[str] = None

    def __hash__(self):
        return hash(self.start_address)

    def __eq__(self, other):
        if not isinstance(other, BasicBlock):
            return False
        return self.start_address == other.start_address

    def __lt__(self, other: "BasicBlock"):
        return self.start_address < other.start_address

    def __repr__(self) -> str:
        return (
            f"BasicBlock(0x{self.start_address:08x}-0x{self.end_address:08x}, "
            f"type={self.block_type.value}, "
            f"preds={len(self.predecessors)}, "
            f"succs={len(self.successors)})"
        )

    @property
    def last_instruction(self) -> Instruction:
        return self.instructions[-1]

    @property
    def first_instruction(self) -> Instruction:
        return self.instructions[0]

    def add_predecessor(self, block: "BasicBlock") -> None:
        self.predecessors.add(block)

    def add_successor(self, block: "BasicBlock") -> None:
        self.successors.add(block)

    def is_empty(self) -> bool:
        return len(self.instructions) == 0

    def contains_address(self, address: int) -> bool:
        return self.start_address <= address <= self.end_address


class BasicBlockIdentifier:
    switch_cases_indir_jumps: dict[int, List[int]] = {}
    return_jumps: List[int] = []

    def __init__(
        self,
        instructions: List[Instruction],
        entry_points: List[int],
        program: UdonProgramData,
    ):
        self.instructions = instructions
        self.entry_points = set(entry_points)
        self.heap_initial_values = program.heap_initial_values
        self.program = program

        self._address_to_instruction = {inst.address: inst for inst in instructions}
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

        for idx, inst in enumerate(self.instructions):
            if inst.opcode == OpCode.JUMP or inst.opcode == OpCode.JUMP_IF_FALSE:
                target = inst.get_jump_target()

                if target > self.instructions[-1].address:
                    self.return_jumps.append(inst.address)
                else:
                    block_starts.add(target)

                next_addr = inst.next_address
                if self._address_to_instruction.get(next_addr):
                    block_starts.add(next_addr)

            elif inst.opcode == OpCode.JUMP_INDIRECT:
                if inst.operand is None:
                    raise Exception(
                        "Invalid JUMP_INDIRECT instruction: missing operand!"
                    )

                operand_sym = self.program.get_symbol_by_address(inst.operand)

                if operand_sym.name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME:
                    """
                    This is ignored because a return jump only targets 0xffffffff
                    (halt) or the next line following a function call. For a halt
                    jump, no basic block needs to be created; for a real return
                    jump, the basic block was already created during the processing
                    of the call jump.
                    """
                    self.return_jumps.append(inst.address)
                    continue

                switch_targets = self._get_switch_targets(idx, operand_sym)
                if switch_targets:
                    block_starts.update(switch_targets)

        return block_starts

    def _get_switch_targets(self, jump_idx: int, operand_sym: SymbolInfo) -> List[int]:
        try:
            extern_inst = self.instructions[jump_idx - 1]
            assert extern_inst.opcode == OpCode.EXTERN and extern_inst.operand
            extern_f = self.program.get_initial_heap_value(extern_inst.operand)
            assert (
                extern_f
                and extern_f.value.value == UdonModuleInfo.UINT32ARRAY_GET_METHOD_NAME
            )

            push_addr_inst = self.instructions[jump_idx - 2]
            assert push_addr_inst.opcode == OpCode.PUSH and push_addr_inst.operand
            target_sym = self.program.get_symbol_by_address(push_addr_inst.operand)
            assert target_sym.name == operand_sym.name

            push_switch_exp_inst = self.instructions[jump_idx - 3]
            assert (
                push_switch_exp_inst.opcode == OpCode.PUSH
                and push_switch_exp_inst.operand
            )
            # switch_exp_sym = self.program.get_symbol_by_address(
            #     push_switch_exp_inst.operand
            # )
            # assert switch_exp_sym.type == UdonModuleInfo.UINT32_TYPE_NAME

            push_addr_table_inst = self.instructions[jump_idx - 4]
            assert (
                push_addr_table_inst.opcode == OpCode.PUSH
                and push_addr_table_inst.operand
            )
            addr_table_sym = self.program.get_symbol_by_address(
                push_addr_table_inst.operand
            )
            assert addr_table_sym.type == (
                UdonModuleInfo.UINT32_TYPE_NAME + UdonModuleInfo.ARRAY_TYPE_SUFFIX
            )
            addr_table_heap_entry = self.heap_initial_values.get(addr_table_sym.address)
            assert addr_table_heap_entry
            addr_table = addr_table_heap_entry.value.value
            assert isinstance(addr_table, list)

            self.switch_cases_indir_jumps[self.instructions[jump_idx].address] = (
                addr_table
            )

            return addr_table

        except Exception:
            logger.warning(
                "Unrecognized JUMP_INDIRECT encountered at %s! Ignoring...",
                self.instructions[jump_idx].address,
            )
            return []

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

            block_type = self._determine_block_type(block_instructions)

            block = BasicBlock(
                start_address=start_addr,
                end_address=end_addr,
                instructions=block_instructions,
                block_type=block_type,
                is_entry=start_addr in self.entry_points,
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

    def _determine_block_type(self, instructions: List[Instruction]) -> BasicBlockType:
        if not instructions:
            return BasicBlockType.NORMAL

        last_inst = instructions[-1]

        if last_inst.address in self.return_jumps:
            return BasicBlockType.RETURN

        if last_inst.is_conditional_jump():
            return BasicBlockType.CONDITIONAL
        elif last_inst.is_unconditional_jump():
            return BasicBlockType.JUMP
        elif last_inst.opcode == OpCode.JUMP_INDIRECT:
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
