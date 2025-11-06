from dataclasses import dataclass, field
from typing import Dict, Set, Optional, List
from enum import Enum

from ..models.instruction import Instruction
from ..models.program import UdonProgramData, SymbolInfo
from .basic_block import BasicBlock
from .cfg import ControlFlowGraph
from .stack_simulator import StackFrame, StackSimulator, StackValue, StackValueType
from ..utils.logger import logger


class VariableScope(Enum):
    GLOBAL = "global"
    LOCAL = "local"
    PARAMETER = "parameter"  # function param
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
        return f"Variable({self.name}{type_str} @ 0x{self.address:08x}, scope={self.scope.value})"


class VariableIdentifier:
    def __init__(
        self,
        program: UdonProgramData,
        cfg: ControlFlowGraph,
        stack_simulator: StackSimulator
    ):
        self.program = program
        self.cfg = cfg
        self.stack_simulator = stack_simulator

        self._variables: Dict[int, Variable] = {}  # address -> Variable
        self._temp_counter = 0

    def identify(self) -> Dict[int, Variable]:
        logger.info(f"Identifying variables in {self.cfg.function_name}...")

        self._identify_global_variables()

        for block in self.cfg.graph.nodes():
            self._analyze_block_variables(block)

        self._infer_variable_scopes()

        logger.info(
            f"Identified {len(self._variables)} variables in {self.cfg.function_name}"
        )

        return self._variables

    def _identify_global_variables(self) -> None:

        for symbol_name, symbol in self.program.symbols.items():
            variable = Variable(
                address=symbol.address,
                name=symbol_name,
                type_hint=symbol.brief_type,
                scope=VariableScope.GLOBAL,
                original_symbol=symbol
            )

            self._variables[symbol.address] = variable

            logger.debug(f"Global variable: {variable}")

    def _analyze_block_variables(self, block: BasicBlock) -> None:
        for instruction in block.instructions:
            state = self.stack_simulator.get_instruction_state(
                instruction.address)
            if not state:
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
                        if target_val.value_type == StackValueType.HEAP_ADDRESS:
                            self._record_variable_write(
                                target_val.value,
                                instruction.address
                            )
                        if source_val.value_type == StackValueType.HEAP_ADDRESS:
                            self._record_variable_read(
                                source_val.value,
                                instruction.address
                            )

            elif instruction.opcode.name == "EXTERN":
                self._analyze_extern_variables(instruction, state)

    def _analyze_extern_variables(self, instruction: Instruction, state) -> None:
        if instruction.operand is None:
            return

        heap_entry = self.program.get_initial_heap_value(instruction.operand)
        if not heap_entry or not heap_entry.value.is_serializable:
            return

        signature = heap_entry.value.value
        if not isinstance(signature, str):
            return

        prev_state = self._get_previous_instruction_state(instruction)
        if not prev_state:
            return

        from ..models.module_info import UdonModuleInfo
        module_info = UdonModuleInfo()

        func_info = module_info.get_function_info(signature)
        if not func_info:
            logger.debug(
                f"EXTERN at 0x{instruction.address:08x}: "
                f"unknown function signature {signature}"
            )
            return

        param_count = func_info.parameter_count

        if len(prev_state.stack) < param_count:
            logger.warning(
                f"EXTERN at 0x{instruction.address:08x}: "
                f"stack has {len(prev_state.stack)} values but function {signature} "
                f"requires {param_count} parameters"
            )
            return

        for i in range(param_count):
            param_val = prev_state.peek(param_count - 1 - i)

            if not param_val:
                continue

            if param_val.value_type == StackValueType.HEAP_ADDRESS:
                self._record_variable_read(
                    param_val.value,
                    instruction.address
                )
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
        symbol = self.program.get_symbol_by_address(address)
        if symbol:
            variable = Variable(
                address=address,
                name=symbol.name,
                type_hint=symbol.brief_type,
                scope=VariableScope.GLOBAL,
                original_symbol=symbol
            )
        else:
            heap_entry = self.program.get_initial_heap_value(address)
            type_hint = heap_entry.brief_type if heap_entry else None

            var_name = self._generate_temp_name()

            variable = Variable(
                address=address,
                name=var_name,
                type_hint=type_hint,
                scope=VariableScope.TEMPORARY
            )

        self._variables[address] = variable
        return variable

    def _generate_temp_name(self) -> str:
        name = f"temp_{self._temp_counter}"
        self._temp_counter += 1
        return name

    def _infer_variable_scopes(self) -> None:
        for variable in self._variables.values():
            if variable.scope != VariableScope.TEMPORARY:
                # todo: fake global
                continue

            # 1w nr -> func param
            if len(variable.write_locations) == 1 and len(variable.read_locations) > 0:
                write_addr = next(iter(variable.write_locations))
                if write_addr < self.cfg.entry_block.start_address + 100:
                    variable.scope = VariableScope.PARAMETER
                    variable.name = f"param_{variable.address:x}"
                    continue

            # nw | nr -> local var
            if len(variable.write_locations) > 1 or len(variable.read_locations) > 1:
                variable.scope = VariableScope.LOCAL
                variable.name = f"local_{variable.address:x}"
                continue

            # <1 w <1 r -> temp

    def _get_previous_instruction_state(self, instruction: Instruction) -> Optional[StackFrame]:
        block = self._find_block_containing(instruction.address)
        if not block:
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
