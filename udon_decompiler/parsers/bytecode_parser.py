from typing import Dict, List

from udon_decompiler.models import Instruction, OpCode, UdonProgramData
from udon_decompiler.models.program import SymbolInfo
from udon_decompiler.utils import logger


class BytecodeParser:
    def __init__(self, program: UdonProgramData):
        self.program = program
        self._instructions: List[Instruction] = []
        self._address_to_instruction: Dict[int, Instruction] = {}

    def parse(self) -> List[Instruction]:
        logger.debug("Parsing bytecode...")

        bytecode = self.program.byte_code_bytes
        instructions = []
        address = 0

        while address < len(bytecode):
            instruction = self._parse_instruction_at(bytecode, address)
            instructions.append(instruction)
            self._address_to_instruction[address] = instruction
            address = instruction.next_address

        self._instructions = instructions
        logger.info(f"Parsed {len(instructions)} instructions")

        # fix function headers
        for e in self.program.entry_points:
            inst = self.get_instruction_at(e.address)
            if inst is None:
                raise Exception(
                    "Invalid entry point! "
                    "It's address does not point to any valid instruction!"
                )
            if inst.opcode != OpCode.PUSH:
                continue
            if inst.operand is None:
                raise Exception("Invalid PUSH instruction! An operand expected!")

            sym = self.program.get_symbol_by_address(inst.operand)
            if sym.name != SymbolInfo.HALT_JUMP_ADDR_SYMBOL_NAME:
                continue
            val = self.program.get_initial_heap_value(inst.operand)
            if val is None:
                raise Exception("Invalid symbol! Initial value not found!")
            if val.value.value != Instruction.HALT_JUMP_ADDR:
                continue
            # this is a halt jump target! the next inst is the call jump addr!

            e.call_jump_target = e.address + OpCode.PUSH.size

        return instructions

    def _parse_instruction_at(self, bytecode: bytes, address: int) -> Instruction:
        if address >= len(bytecode):
            raise ValueError(f"Address {address:08x} out of bytecode range")

        # big endian
        opcode_value = int.from_bytes(
            bytecode[address : address + 4], byteorder="big", signed=False
        )

        try:
            opcode = OpCode(opcode_value)
        except ValueError:
            raise ValueError(f"Unknown opcode {opcode_value} at address {address:08x}")

        operand = None
        operand_name = None
        if opcode.has_operand:
            if address + 8 > len(bytecode):
                raise ValueError(
                    f"Incomplete instruction at address {address:08x}: "
                    f"expected operand but reached end of bytecode"
                )
            operand = int.from_bytes(
                bytecode[address + 4 : address + 8], byteorder="big", signed=False
            )

            if opcode.has_operand_name:
                if opcode == OpCode.EXTERN:
                    operand_heap_entry = self.program.get_initial_heap_value(operand)
                    if operand_heap_entry is None:
                        raise Exception(
                            "Invalid EXTERN instruction! "
                            "A heap entry corresponding to the operand expected!"
                        )
                    operand_name = operand_heap_entry.value.value
                else:
                    operand_symbol = self.program.get_symbol_by_address(operand)
                    operand_name = operand_symbol.name

        return Instruction(
            address=address,
            opcode=opcode,
            operand=operand,
            operand_name=operand_name,
        )

    def get_instruction_at(self, address: int) -> Instruction | None:
        return self._address_to_instruction.get(address)

    @property
    def instructions(self) -> List[Instruction]:
        if not self._instructions:
            self.parse()
        return self._instructions
