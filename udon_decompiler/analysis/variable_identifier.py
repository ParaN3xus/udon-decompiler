from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, Optional, Set, Tuple

from udon_decompiler.analysis.basic_block import BasicBlock
from udon_decompiler.analysis.cfg import ControlFlowGraph
from udon_decompiler.analysis.stack_simulator import (
    StackFrame,
    StackSimulator,
)
from udon_decompiler.models.instruction import Instruction
from udon_decompiler.models.program import SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class VariableScope(Enum):
    GLOBAL = "global"
    LOCAL = "local"
    TEMPORARY = "temporary"
    # RETURN = "return"


@dataclass
class Variable:
    address: int
    name: str
    type_hint: Optional[str] = None
    scope: VariableScope = VariableScope.LOCAL

    read_locations: Set[int] = field(default_factory=set)
    write_locations: Set[int] = field(default_factory=set)

    original_symbol: Optional[SymbolInfo] = None

    def __hash__(self):
        return hash(self.address)

    def __eq__(self, other):
        if not isinstance(other, Variable):
            return False
        return self.address == other.address

    def __repr__(self) -> str:
        type_str = f": {self.type_hint}" if self.type_hint else ""
        return (
            f"Variable({self.name}{type_str} @ 0x{self.address:08x}, "
            "scope={self.scope.value})"
        )


class VariableIdentifier:
    def __init__(
        self,
        program: UdonProgramData,
        cfg: ControlFlowGraph,
        stack_simulator: StackSimulator,
    ):
        self.program = program
        self.cfg = cfg
        self.stack_simulator = stack_simulator

        self._variables: Dict[int, Variable] = {}  # address -> Variable
        self._temp_counter = 0

    def identify(self) -> Dict[int, Variable]:
        logger.info(f"Identifying variables in {self.cfg.function_name}...")

        self._initialize_variables_from_symbols()

        for block in self.cfg.graph.nodes():
            self._analyze_block_variables(block)

        logger.info(
            f"Identified {len(self._variables)} variables in {self.cfg.function_name}"
        )

        return self._variables

    def _initialize_variables_from_symbols(self) -> None:
        for symbol_name, symbol in self.program.symbols.items():
            scope, resolved_name = self._classify_symbol(symbol_name)
            variable = Variable(
                address=symbol.address,
                name=resolved_name,
                type_hint=symbol.type,
                scope=scope,
                original_symbol=symbol,
            )

            self._variables[symbol.address] = variable

            logger.debug(f"Variable: {variable}")

    def _analyze_block_variables(self, block: BasicBlock) -> None:
        for instruction in block.instructions:
            state = self.stack_simulator.get_instruction_state(instruction.address)
            if state is None:
                continue

            if instruction.opcode.name == "PUSH" and instruction.operand is not None:
                address = instruction.operand

                if self.program.get_initial_heap_value(address) is not None:
                    self._record_variable_read(address, instruction.address)

            elif instruction.opcode.name == "COPY":
                prev_state = self._get_previous_instruction_state(instruction)
                if prev_state and len(prev_state.stack) >= 2:
                    source_val = prev_state.peek(0)
                    target_val = prev_state.peek(1)

                    if source_val and target_val:
                        self._record_variable_write(
                            target_val.value, instruction.address
                        )
                        self._record_variable_read(
                            source_val.value, instruction.address
                        )

            elif instruction.opcode.name == "EXTERN":
                self._analyze_extern_variables(instruction, state)

    def _analyze_extern_variables(self, instruction: Instruction, state) -> None:
        if instruction.operand is None:
            return

        heap_entry = self.program.get_initial_heap_value(instruction.operand)
        if heap_entry is None or not heap_entry.value.is_serializable:
            return

        signature = heap_entry.value.value
        if not isinstance(signature, str):
            return

        frame = state or self._get_previous_instruction_state(instruction)
        if frame is None:
            return

        from ..models.module_info import UdonModuleInfo

        module_info = UdonModuleInfo()

        func_info = module_info.get_function_info(signature)
        if func_info is None:
            logger.debug(
                f"EXTERN at 0x{instruction.address:08x}: "
                f"unknown function signature {signature}"
            )
            return

        param_count = func_info.parameter_count
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

        if len(frame.stack) < param_count:
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: "
                f"stack has {len(frame.stack)} values but function {signature} "
                f"requires {param_count} parameters"
            )
            return

        for i in range(param_count):
            param_val = frame.peek(param_count - 1 - i)

            if param_val is None:
                continue

            is_output_param = not returns_void and i == param_count - 1
            if is_output_param:
                self._record_variable_write(param_val.value, instruction.address)
                logger.debug(
                    f"EXTERN at 0x{instruction.address:08x}: "
                    f"parameter {i} writes variable at 0x{param_val.value:08x}"
                )
            else:
                self._record_variable_read(param_val.value, instruction.address)
                logger.debug(
                    f"EXTERN at 0x{instruction.address:08x}: "
                    f"parameter {i} reads variable at 0x{param_val.value:08x}"
                )

    def _record_variable_read(self, address: int, instruction_address: int) -> None:
        if address not in self._variables:
            self._create_variable(address)

        self._variables[address].read_locations.add(instruction_address)

    def _record_variable_write(self, address: int, instruction_address: int) -> None:
        if address not in self._variables:
            self._create_variable(address)

        self._variables[address].write_locations.add(instruction_address)

    def _create_variable(self, address: int) -> Variable:
        try:
            symbol = self.program.get_symbol_by_address(address)
        except Exception:
            symbol = None

        if symbol is None:
            heap_entry = self.program.get_initial_heap_value(address)
            type_hint = heap_entry.type if heap_entry else None
            var_name = self._generate_temp_name()
            variable = Variable(
                address=address,
                name=var_name,
                type_hint=type_hint,
                scope=VariableScope.TEMPORARY,
            )
            self._variables[address] = variable
            return variable

        scope, resolved_name = self._classify_symbol(symbol.name)
        variable = Variable(
            address=address,
            name=resolved_name,
            type_hint=symbol.type,
            scope=scope,
            original_symbol=symbol,
        )

        self._variables[address] = variable
        return variable

    def _generate_temp_name(self) -> str:
        name = f"temp_{self._temp_counter}"
        self._temp_counter += 1
        return name

    def _classify_symbol(self, symbol_name: str) -> Tuple[VariableScope, str]:
        if symbol_name.startswith(SymbolInfo.CONST_SYMBOL_PREFIX):
            return VariableScope.GLOBAL, symbol_name

        if symbol_name.startswith(SymbolInfo.INTERNAL_SYMBOL_PREFIX):
            return VariableScope.TEMPORARY, symbol_name

        if symbol_name.startswith(SymbolInfo.GLOBAL_INTERNAL_SYMBOL_PREFIX):
            return VariableScope.GLOBAL, symbol_name

        if symbol_name.startswith(SymbolInfo.LOCAL_SYMBOL_PREFIX):
            return VariableScope.LOCAL, symbol_name

        if symbol_name.startswith(SymbolInfo.THIS_SYMBOL_PREFIX):
            return VariableScope.GLOBAL, self._resolve_this_symbol(symbol_name)

        # params or fields - treat as global
        return VariableScope.GLOBAL, symbol_name

    def _resolve_this_symbol(self, symbol_name: str) -> str:
        if "VRCUdonUdonBehaviour" in symbol_name:
            return "this"
        if "UnityEngineTransform" in symbol_name:
            return "this.transform"
        if "UnityEngineGameObject" in symbol_name:
            return "this.gameObject"
        return "this"

    def _get_previous_instruction_state(
        self, instruction: Instruction
    ) -> Optional[StackFrame]:
        block = self._find_block_containing(instruction.address)
        if block is None:
            return None

        inst_index = None
        for i, inst in enumerate(block.instructions):
            if inst.address == instruction.address:
                inst_index = i
                break

        if inst_index is None:
            return None

        if inst_index > 0:
            # common inst
            prev_inst = block.instructions[inst_index - 1]
            return self.stack_simulator.get_instruction_state(prev_inst.address)
        else:
            # block entry
            return self.stack_simulator.get_block_entry_state(block)

    def _find_block_containing(self, address: int) -> Optional[BasicBlock]:
        for block in self.cfg.graph.nodes():
            block: BasicBlock
            if block.contains_address(address):
                return block
        return None

    def get_variable(self, address: int) -> Optional[Variable]:
        return self._variables.get(address)

    def get_variable_name(self, address: int) -> str:
        variable = self.get_variable(address)
        return variable.name if variable else f"var_0x{address:08x}"

    @property
    def variables(self) -> Dict[int, Variable]:
        return self._variables
