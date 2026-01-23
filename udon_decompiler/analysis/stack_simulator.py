from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional

from udon_decompiler.analysis.basic_block import BasicBlock
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import UdonModuleInfo
from udon_decompiler.models.program import UdonProgramData
from udon_decompiler.utils.logger import logger


@dataclass
class StackValue:
    value: int
    type_hint: Optional[str] = None
    source_instruction: Optional[Instruction] = None

    def __repr__(self) -> str:
        type_str = f": {self.type_hint}" if self.type_hint else ""
        return f"StackValue({self.value}{type_str})"


@dataclass
class StackFrame:
    stack: List[StackValue] = field(default_factory=list)

    def push(self, value: StackValue) -> None:
        self.stack.append(value)

    def pop(self) -> Optional[StackValue]:
        return self.stack.pop() if self.stack else None

    def peek(self, depth: int = 0) -> Optional[StackValue]:
        if depth < len(self.stack):
            return self.stack[-(depth + 1)]
        return None

    def copy(self) -> "StackFrame":
        new_frame = StackFrame()
        new_frame.stack = [
            StackValue(
                value=sv.value,
                type_hint=sv.type_hint,
                source_instruction=sv.source_instruction,
            )
            for sv in self.stack
        ]
        return new_frame

    def __repr__(self) -> str:
        return f"StackFrame(depth={len(self.stack)})"


class StackSimulator:
    def __init__(self, program: UdonProgramData, module_info: UdonModuleInfo):
        self.program = program
        self.module_info = module_info

        self._block_entry_states: Dict[BasicBlock, StackFrame] = {}
        self._block_exit_states: Dict[BasicBlock, StackFrame] = {}
        self._instruction_states: Dict[int, StackFrame] = {}  # address -> state

    def simulate_block(
        self, block: BasicBlock, entry_state: Optional[StackFrame] = None
    ) -> StackFrame:
        if entry_state is None:
            entry_state = StackFrame()

        current_state = entry_state.copy()
        self._block_entry_states[block] = entry_state.copy()

        logger.debug(
            f"Simulating block 0x{block.start_address:08x}, "
            f"entry stack depth={len(current_state.stack)}"
        )

        for instruction in block.instructions:
            current_state = self._simulate_instruction(instruction, current_state)
            self._instruction_states[instruction.address] = current_state.copy()

        self._block_exit_states[block] = current_state.copy()

        logger.debug(
            f"Block 0x{block.start_address:08x} exit, "
            f"stack depth={len(current_state.stack)}"
        )

        return current_state

    def _simulate_instruction(
        self, instruction: Instruction, state: StackFrame
    ) -> StackFrame:
        opcode = instruction.opcode

        match opcode:
            case OpCode.NOP | OpCode.ANNOTATION:
                pass
            case OpCode.PUSH:
                operand = instruction.operand
                if operand is None:
                    logger.warning(
                        f"PUSH at 0x{instruction.address:08x} has no operand"
                    )
                    return state

                heap_entry = self.program.get_initial_heap_value(operand)
                if heap_entry:
                    state.push(
                        StackValue(
                            value=operand,
                            type_hint=heap_entry.type,
                            source_instruction=instruction,
                        )
                    )
                    return state

                symbol = self.program.get_symbol_by_address(operand)
                if symbol:
                    state.push(
                        StackValue(
                            value=operand,
                            type_hint=symbol.type,
                            source_instruction=instruction,
                        )
                    )
                    return state
                raise Exception("Unknown stack value!")
            case OpCode.POP:
                state.pop()
            case OpCode.JUMP_IF_FALSE:
                cond_value = state.pop()
                if cond_value and cond_value.type_hint != "System.Boolean":
                    logger.debug(
                        f"JUMP_IF_FALSE at 0x{instruction.address:08x} "
                        f"popped non-boolean: {cond_value}"
                    )
            case OpCode.JUMP:
                target = instruction.get_jump_target()
                is_call_jump = any(
                    ep.call_jump_target == target for ep in self.program.entry_points
                )
                if is_call_jump:
                    state.pop()
            case OpCode.JUMP_INDIRECT:
                pass
            case OpCode.EXTERN:
                self._simulate_extern_call(instruction, state)
            case OpCode.COPY:
                source = state.pop()
                target = state.pop()
                if source and target:
                    logger.debug(
                        f"COPY at 0x{instruction.address:08x}: "
                        f"heap[0x{target.value:08x}] = heap[0x{source.value:08x}]"
                    )
        return state

    def _simulate_extern_call(
        self, instruction: Instruction, state: StackFrame
    ) -> None:
        if instruction.operand is None:
            logger.warning(f"EXTERN at 0x{instruction.address:08x} has no operand")
            return

        heap_entry = self.program.get_initial_heap_value(instruction.operand)
        if heap_entry is None or not heap_entry.value.is_serializable:
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: "
                f"cannot resolve function signature from heap"
            )
            return

        signature = heap_entry.value.value
        if not isinstance(signature, str):
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: heap value is not a string"
            )
            return

        func_info = self.module_info.get_function_info(signature)
        if func_info is None:
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: unknown function {signature}"
            )
            return

        param_count = func_info.parameter_count

        params = []
        for i in range(param_count):
            param = state.pop()
            if param:
                params.append(param)
            else:
                logger.warning(
                    f"EXTERN at 0x{instruction.address:08x}: "
                    f"stack underflow when popping parameter {i}"
                )

        params.reverse()

        logger.debug(
            f"EXTERN at 0x{instruction.address:08x}: "
            f"{signature} with {param_count} params"
        )

    def get_block_entry_state(self, block: BasicBlock) -> Optional[StackFrame]:
        return self._block_entry_states.get(block)

    def get_block_exit_state(self, block: BasicBlock) -> Optional[StackFrame]:
        return self._block_exit_states.get(block)

    def get_instruction_state(self, address: int) -> Optional[StackFrame]:
        return self._instruction_states.get(address)
