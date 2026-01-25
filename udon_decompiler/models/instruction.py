from dataclasses import dataclass
from enum import IntEnum
from typing import Final, Optional


class OpCode(IntEnum):
    NOP = 0
    PUSH = 1
    POP = 2
    JUMP_IF_FALSE = 4
    JUMP = 5
    EXTERN = 6
    ANNOTATION = 7
    JUMP_INDIRECT = 8
    COPY = 9

    def __str__(self) -> str:
        return self.name

    @property
    def has_operand(self) -> bool:
        return self in {
            OpCode.PUSH,
            OpCode.JUMP_IF_FALSE,
            OpCode.JUMP,
            OpCode.EXTERN,
            OpCode.ANNOTATION,
            OpCode.JUMP_INDIRECT,
        }

    @property
    def has_operand_name(self) -> bool:
        return self in {
            OpCode.PUSH,
            OpCode.EXTERN,
            OpCode.ANNOTATION,
            OpCode.JUMP_INDIRECT,
        }

    @property
    def size(self) -> int:
        return 8 if self.has_operand else 4


@dataclass
class Instruction:
    address: int
    opcode: "OpCode"
    operand: Optional[int] = None
    operand_name: Optional[str] = None

    HALT_JUMP_ADDR: Final[int] = 0xFFFFFFFF

    def __str__(self) -> str:
        if self.operand_name is not None:
            return f"{self.address:08x}: {self.opcode.name} {self.operand_name}"
        if self.operand is not None:
            return f"{self.address:08x}: {self.opcode.name} 0x{self.operand:08x}"
        return f"{self.address:08x}: {self.opcode.name}"

    def __repr__(self) -> str:
        return self.__str__()

    @property
    def size(self) -> int:
        return self.opcode.size

    @property
    def next_address(self) -> int:
        return self.address + self.size

    def is_jump(self) -> bool:
        return self.opcode in {OpCode.JUMP, OpCode.JUMP_IF_FALSE, OpCode.JUMP_INDIRECT}

    def is_unconditional_jump(self) -> bool:
        return self.opcode == OpCode.JUMP

    def is_conditional_jump(self) -> bool:
        return self.opcode == OpCode.JUMP_IF_FALSE

    def get_jump_target(self) -> int:
        if self.opcode not in {OpCode.JUMP, OpCode.JUMP_IF_FALSE}:
            raise Exception(
                "Trying to get jump target on a non-direct-jump instruction!"
            )
        if self.operand is None:
            raise Exception("Invalid jump instruction! An operand expected!")
        return self.operand
