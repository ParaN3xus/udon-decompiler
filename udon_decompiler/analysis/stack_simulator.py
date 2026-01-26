from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional

from udon_decompiler.analysis.basic_block import BasicBlock
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import ExternFunctionInfo, UdonModuleInfo
from udon_decompiler.models.program import SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


@dataclass
class StackValue:
    value: int
    type_hint: Optional[str] = None
    source_instruction: Optional[Instruction] = None
    literal_value: Optional[int] = None

    def __repr__(self) -> str:
        type_str = f": {self.type_hint}" if self.type_hint else ""
        literal_str = (
            f", literal={self.literal_value}" if self.literal_value is not None else ""
        )
        return f"StackValue({self.value}{type_str}{literal_str})"


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
                literal_value=sv.literal_value,
            )
            for sv in self.stack
        ]
        return new_frame

    def __repr__(self) -> str:
        return f"StackFrame(depth={len(self.stack)})"


class HeapValueKind(Enum):
    INIT = "init"
    KNOWN = "known"
    UNKNOWN = "unknown"


@dataclass
class HeapValue:
    kind: HeapValueKind
    value: Optional[object] = None

    def __repr__(self) -> str:
        if self.kind == HeapValueKind.UNKNOWN:
            return "HeapValue(unknown)"
        return f"HeapValue({self.kind.value}, {self.value!r})"


class HeapSimulator:
    def __init__(self, program: UdonProgramData):
        self.program = program
        self._values: Dict[int, HeapValue] = {}

    def get_value(self, address: int) -> HeapValue:
        if address in self._values:
            return self._values[address]

        # init
        heap_entry = self.program.get_initial_heap_value(address)
        if heap_entry and heap_entry.value.is_serializable:
            value = heap_entry.value.value
            hv = HeapValue(kind=HeapValueKind.INIT, value=value)
        else:
            hv = HeapValue(kind=HeapValueKind.UNKNOWN, value=None)

        self._values[address] = hv
        return hv

    def read_literal_int(self, address: int) -> Optional[int]:
        hv = self.get_value(address)
        if hv.kind == HeapValueKind.UNKNOWN:
            return None
        if isinstance(hv.value, int):
            return hv.value
        return None

    def set_known(self, address: int, value: Optional[object]) -> None:
        self._values[address] = HeapValue(kind=HeapValueKind.KNOWN, value=value)

    def set_unknown(self, address: int) -> None:
        self._values[address] = HeapValue(kind=HeapValueKind.UNKNOWN, value=None)

    def copy_value(self, source_address: int, target_address: int) -> None:
        source_value = self.get_value(source_address)
        if source_value.kind == HeapValueKind.UNKNOWN:
            self.set_unknown(target_address)
            return
        self.set_known(target_address, source_value.value)


class _StackSemantics:
    def __init__(self, program: UdonProgramData):
        self.program = program
        self.module_info = UdonModuleInfo()

    def _stack_value_for_push(
        self, instruction: Instruction, heap: Optional[HeapSimulator]
    ) -> Optional[StackValue]:
        operand = instruction.operand
        if operand is None:
            logger.warning(f"PUSH at 0x{instruction.address:08x} has no operand")
            return None

        heap_entry = self.program.get_initial_heap_value(operand)
        type_hint = heap_entry.type if heap_entry else None
        if type_hint is None:
            try:
                symbol = self.program.get_symbol_by_address(operand)
                type_hint = symbol.type
            except Exception:
                type_hint = None

        literal_value = None
        if heap is not None:
            literal_value = heap.read_literal_int(operand)
        else:
            if (
                heap_entry
                and heap_entry.value.is_serializable
                and isinstance(heap_entry.value.value, int)
            ):
                literal_value = heap_entry.value.value

        return StackValue(
            value=operand,
            type_hint=type_hint,
            source_instruction=instruction,
            literal_value=literal_value,
        )

    def _get_extern_info(
        self, instruction: Instruction
    ) -> Optional[tuple[ExternFunctionInfo, bool]]:
        if instruction.operand is None:
            logger.warning(f"EXTERN at 0x{instruction.address:08x} has no operand")
            return None

        heap_entry = self.program.get_initial_heap_value(instruction.operand)
        if heap_entry is None or not heap_entry.value.is_serializable:
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: "
                f"cannot resolve function signature from heap"
            )
            return None

        signature = heap_entry.value.value
        if not isinstance(signature, str):
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: heap value is not a string"
            )
            return None

        func_info = self.module_info.get_function_info(signature)
        if func_info is None:
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: unknown function {signature}"
            )
            return None

        returns_void = func_info.returns_void
        if returns_void is None:
            from ..models.module_info import FunctionDefinitionType

            if func_info.def_type == FunctionDefinitionType.OPERATOR:
                returns_void = False
            elif func_info.def_type == FunctionDefinitionType.FIELD:
                returns_void = not func_info.function_name.startswith("__get")
            elif func_info.def_type == FunctionDefinitionType.CTOR:
                returns_void = False
            else:
                returns_void = True

        return func_info, returns_void

    def _pop_extern_params(
        self, instruction: Instruction, state: StackFrame, param_count: int
    ) -> List[StackValue]:
        params: List[StackValue] = []
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
        return params


class BlockStackSimulator(_StackSemantics):
    def __init__(self, program: UdonProgramData):
        super().__init__(program)

    def step(
        self, instruction: Instruction, state: StackFrame, heap: HeapSimulator
    ) -> StackFrame:
        opcode = instruction.opcode

        match opcode:
            case OpCode.NOP | OpCode.ANNOTATION | OpCode.JUMP | OpCode.JUMP_INDIRECT:
                pass
            case OpCode.PUSH:
                stack_value = self._stack_value_for_push(instruction, heap)
                if stack_value is not None:
                    state.push(stack_value)
            case OpCode.POP | OpCode.JUMP_IF_FALSE:
                state.pop()
            case OpCode.EXTERN:
                self._simulate_extern_call(instruction, state, heap)
            case OpCode.COPY:
                source = state.pop()
                target = state.pop()
                if source and target:
                    heap.copy_value(source.value, target.value)
        return state

    def simulate_block(
        self,
        block: BasicBlock,
        entry_state: Optional[StackFrame] = None,
        heap: Optional[HeapSimulator] = None,
    ) -> StackFrame:
        if entry_state is None:
            entry_state = StackFrame()
        if heap is None:
            heap = HeapSimulator(self.program)

        current_state = entry_state
        for instruction in block.instructions:
            current_state = self.step(instruction, current_state, heap)
        return current_state

    def _simulate_extern_call(
        self, instruction: Instruction, state: StackFrame, heap: HeapSimulator
    ) -> None:
        info = self._get_extern_info(instruction)
        if info is None:
            return
        func_info, returns_void = info

        params = self._pop_extern_params(instruction, state, func_info.parameter_count)
        if not returns_void and params:
            heap.set_unknown(params[-1].value)


class StackSimulator(_StackSemantics):
    def __init__(self, program: UdonProgramData):
        super().__init__(program)

        self._block_entry_states: Dict[BasicBlock, StackFrame] = {}
        self._block_exit_states: Dict[BasicBlock, StackFrame] = {}
        # address -> state before the instruection at the address executes
        self._instruction_states: Dict[int, StackFrame] = {}

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
            self._instruction_states[instruction.address] = current_state.copy()
            halt, current_state = self._simulate_instruction(instruction, current_state)
            if halt:
                break

        self._block_exit_states[block] = current_state.copy()

        logger.debug(
            f"Block 0x{block.start_address:08x} exit, "
            f"stack depth={len(current_state.stack)}"
        )

        return current_state

    def _simulate_instruction(
        self, instruction: Instruction, state: StackFrame
    ) -> tuple[bool, StackFrame]:
        """
        :return: `is_halted`, `state`
        :rtype: tuple[bool, StackFrame]
        """
        opcode = instruction.opcode

        match opcode:
            case OpCode.NOP | OpCode.ANNOTATION:
                pass
            case OpCode.PUSH:
                stack_value = self._stack_value_for_push(instruction, None)
                if stack_value is not None:
                    state.push(stack_value)
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
                if instruction.operand is None:
                    raise Exception(
                        "Invalid JUMP_INDIRECT instruction: missing operand!"
                    )
                operand_sym = self.program.get_symbol_by_address(instruction.operand)
                if operand_sym.name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME:
                    return True, state
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
                else:
                    raise Exception(
                        "Failed to simulate instruction! "
                        "More entries in the stack expected!"
                    )
        return False, state

    def _simulate_extern_call(
        self, instruction: Instruction, state: StackFrame
    ) -> None:
        info = self._get_extern_info(instruction)
        if info is None:
            return
        func_info, _ = info

        self._pop_extern_params(instruction, state, func_info.parameter_count)

    def get_block_entry_state(self, block: BasicBlock) -> Optional[StackFrame]:
        return self._block_entry_states.get(block)

    def get_block_exit_state(self, block: BasicBlock) -> Optional[StackFrame]:
        return self._block_exit_states.get(block)

    def get_instruction_state(self, address: int) -> Optional[StackFrame]:
        """
        Returns the StackFrame **before** the instruction at the given address executes.
        """
        return self._instruction_states.get(address)
