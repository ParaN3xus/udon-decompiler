from typing import Final, List, Optional

from udon_decompiler.analysis.basic_block import (
    BasicBlock,
    BasicBlockType,
    SwitchTableInfo,
)
from udon_decompiler.analysis.cfg import ControlFlowGraph
from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.ir.nodes import (
    AssignmentStatement,
    BlockIR,
    ConditionalTerminator,
    ConstructorCallExpression,
    EndTerminator,
    ExpressionStatement,
    ExternalCallExpression,
    FunctionIR,
    GotoTerminator,
    InternalCallExpression,
    IRExpression,
    IRStatement,
    IRTerminator,
    LiteralExpression,
    OperatorCallExpression,
    PropertyAccessExpression,
    ReturnTerminator,
    SwitchTerminator,
    VariableExpression,
)
from udon_decompiler.analysis.stack_simulator import (
    StackFrame,
    StackSimulator,
    StackValue,
)
from udon_decompiler.analysis.variable_identifier import (
    Variable,
    VariableIdentifier,
    VariableScope,
)
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import (
    ExternFunctionInfo,
    FunctionDefinitionType,
    UdonModuleInfo,
)
from udon_decompiler.models.program import EntryPointInfo, SymbolInfo, UdonProgramData


class IRBuilder:
    EXTERN_OP_PREFIX: Final[str] = "__op_"

    def __init__(
        self,
        program: UdonProgramData,
        cfg: ControlFlowGraph,
        stack_simulator: StackSimulator,
        variable_identifier: VariableIdentifier,
    ):
        self.program = program
        self.cfg = cfg
        self.stack_simulator = stack_simulator
        self.variable_identifier = variable_identifier
        self.module_info = UdonModuleInfo()

    def build(self) -> FunctionIR:
        blocks_sorted = sorted(self.cfg.graph.nodes(), key=lambda b: b.start_address)
        blocks = {
            block.start_address: self._build_block(block) for block in blocks_sorted
        }

        function_name = self.cfg.function_name
        if function_name is None:
            function_name = f"func_0x{self.cfg.entry_block.start_address:08x}"

        return FunctionIR(
            function_name=function_name,
            is_function_public=self.cfg.is_function_public,
            entry_block_start=self.cfg.entry_block.start_address,
            variable_declarations=self._collect_variable_declarations(),
            block_order=[block.start_address for block in blocks_sorted],
            blocks=blocks,
        )

    def _build_block(self, block: BasicBlock) -> BlockIR:
        return BlockIR(
            start_address=block.start_address,
            end_address=block.end_address,
            statements=self._build_statements(block),
            terminator=self._build_terminator(block),
        )

    def _build_statements(self, block: BasicBlock) -> List[IRStatement]:
        statements: List[IRStatement] = []
        for instruction in block.instructions:
            statements.extend(self._build_statements_from_instruction(instruction))
        return statements

    def _build_statements_from_instruction(
        self, instruction: Instruction
    ) -> List[IRStatement]:
        match instruction.opcode:
            case OpCode.COPY:
                return self._build_copy_statements(instruction)
            case OpCode.EXTERN:
                return self._build_extern_statements(instruction)
            case OpCode.JUMP:
                return self._build_internal_call_statements(instruction)
            case (
                OpCode.JUMP_IF_FALSE
                | OpCode.JUMP_INDIRECT
                | OpCode.NOP
                | OpCode.ANNOTATION
                | OpCode.POP
                | OpCode.PUSH
            ):
                return []

    def _build_copy_statements(self, instruction: Instruction) -> List[IRStatement]:
        state = self._require_instruction_state(instruction.address)

        target_value = state.peek(0)
        source_value = state.peek(1)
        if target_value is None or source_value is None:
            raise Exception(
                f"COPY at 0x{instruction.address:08x} has incomplete stack operands"
            )

        target_variable = self._require_variable(target_value.value)
        if target_variable.name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME:
            return []

        value_expression = self._stack_value_to_expression(source_value)
        return [
            AssignmentStatement(
                instruction_address=instruction.address,
                instruction=instruction,
                target=target_variable,
                value=value_expression,
            )
        ]

    def _build_extern_statements(self, instruction: Instruction) -> List[IRStatement]:
        if instruction.operand is None:
            raise Exception(f"EXTERN at 0x{instruction.address:08x} missing operand")

        heap_entry = self.program.get_initial_heap_value(instruction.operand)
        if heap_entry is None or not heap_entry.value.is_serializable:
            raise Exception(
                f"EXTERN at 0x{instruction.address:08x} missing serializable signature"
            )

        signature = heap_entry.value.value
        if not isinstance(signature, str):
            raise Exception(
                f"EXTERN at 0x{instruction.address:08x} signature is not string"
            )

        function_info = self.module_info.get_function_info(signature)
        if function_info is None:
            raise Exception(
                f"Unknown extern signature at 0x{instruction.address:08x}: {signature}"
            )

        arguments = self._build_call_arguments(
            instruction, function_info.parameter_count
        )
        if function_info.returns_void:
            call_expression = self._build_extern_expression(
                function_info,
                signature,
                arguments,
            )
            return [
                ExpressionStatement(
                    instruction_address=instruction.address,
                    instruction=instruction,
                    expression=call_expression,
                )
            ]

        if not arguments:
            raise Exception(
                f"Non-void EXTERN at 0x{instruction.address:08x} has no return slot"
            )

        return_slot = arguments[-1]
        if not isinstance(return_slot, VariableExpression):
            raise Exception(
                f"Non-void EXTERN at 0x{instruction.address:08x} return slot "
                "is not a variable"
            )

        call_expression = self._build_extern_expression(
            function_info,
            signature,
            arguments[:-1],
        )
        return [
            AssignmentStatement(
                instruction_address=instruction.address,
                instruction=instruction,
                target=return_slot.variable,
                value=call_expression,
            )
        ]

    def _build_internal_call_statements(
        self, instruction: Instruction
    ) -> List[IRStatement]:
        target_entry = self._resolve_internal_call_entry(instruction)
        if target_entry is None:
            return []
        return [
            ExpressionStatement(
                instruction_address=instruction.address,
                instruction=instruction,
                expression=InternalCallExpression(entry_point=target_entry),
            )
        ]

    def _build_extern_expression(
        self,
        function_info: ExternFunctionInfo,
        signature: str,
        arguments: List[IRExpression],
    ) -> IRExpression:
        match function_info.def_type:
            case FunctionDefinitionType.FIELD:
                return PropertyAccessExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                )
            case FunctionDefinitionType.CTOR:
                return ConstructorCallExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                )
            case FunctionDefinitionType.OPERATOR:
                return OperatorCallExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                    operator=self._resolve_operator(function_info),
                )
            case FunctionDefinitionType.METHOD:
                return ExternalCallExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                )

    def _build_terminator(self, block: BasicBlock) -> IRTerminator:
        last_inst = block.last_instruction

        if block.block_type == BasicBlockType.RETURN:
            return ReturnTerminator(address=last_inst.address)

        if last_inst.opcode == OpCode.JUMP_IF_FALSE:
            false_target = last_inst.get_jump_target()
            true_target = self._resolve_true_target(block, false_target)
            condition = self._build_condition_expression(last_inst)
            return ConditionalTerminator(
                address=last_inst.address,
                condition=condition,
                true_target=true_target,
                false_target=false_target,
            )

        if last_inst.opcode == OpCode.JUMP:
            target_entry = self._resolve_internal_call_entry(last_inst)
            if target_entry is not None:
                successor = self._single_successor_start(block)
                if successor is None:
                    return EndTerminator(address=last_inst.address)
                return GotoTerminator(address=last_inst.address, target=successor)

            return GotoTerminator(
                address=last_inst.address,
                target=last_inst.get_jump_target(),
            )

        if last_inst.opcode == OpCode.JUMP_INDIRECT:
            if block.switch_info is not None:
                return self._build_switch_terminator(last_inst, block.switch_info)
            return ReturnTerminator(address=last_inst.address)

        successor = self._single_successor_start(block)
        if successor is None:
            return EndTerminator(address=last_inst.address)
        return GotoTerminator(address=last_inst.address, target=successor)

    def _build_switch_terminator(
        self, instruction: Instruction, switch_info: SwitchTableInfo
    ) -> SwitchTerminator:
        return SwitchTerminator(
            address=instruction.address,
            switch_index=self._operand_to_expression(switch_info.index_operand),
            switch_targets=list(switch_info.targets),
        )

    def _build_condition_expression(self, instruction: Instruction) -> IRExpression:
        state = self._require_instruction_state(instruction.address)
        condition_value = state.peek(0)
        if condition_value is None:
            raise Exception(
                f"JUMP_IF_FALSE at 0x{instruction.address:08x} missing condition value"
            )
        return self._stack_value_to_expression(condition_value)

    def _stack_value_to_expression(self, stack_value: StackValue) -> IRExpression:
        variable = self.variable_identifier.get_variable(stack_value.value)
        if variable is not None:
            return VariableExpression(variable=variable)

        raise Exception("Unknown stack address 0x%08x in IR build" % stack_value.value)

    def _operand_to_expression(self, operand: int) -> IRExpression:
        variable = self.variable_identifier.get_variable(operand)
        if variable is not None:
            return VariableExpression(variable=variable)

        raise Exception(f"Unknown operand 0x{operand:08x} for switch index")

    def _build_call_arguments(
        self, instruction: Instruction, parameter_count: int
    ) -> List[IRExpression]:
        state = self._require_instruction_state(instruction.address)
        if len(state.stack) < parameter_count:
            raise Exception(
                f"EXTERN at 0x{instruction.address:08x} requires {parameter_count} "
                f"stack args, got {len(state.stack)}"
            )

        arguments: List[IRExpression] = []
        for index in range(parameter_count):
            stack_value = state.peek(parameter_count - 1 - index)
            if stack_value is None:
                raise Exception(
                    f"EXTERN at 0x{instruction.address:08x} missing stack arg {index}"
                )
            arguments.append(self._stack_value_to_expression(stack_value))
        return arguments

    def _resolve_operator(self, function_info: ExternFunctionInfo) -> Operator:
        if not function_info.function_name.startswith(self.EXTERN_OP_PREFIX):
            raise Exception(
                f"Extern operator does not start with {self.EXTERN_OP_PREFIX}: "
                f"{function_info.function_name}"
            )

        raw_name = function_info.function_name[len(self.EXTERN_OP_PREFIX) :].split(
            "__", maxsplit=2
        )[0]
        return Operator.from_name(raw_name)

    def _resolve_internal_call_entry(
        self, instruction: Instruction
    ) -> Optional[EntryPointInfo]:
        target = instruction.get_jump_target()
        return next(
            (ep for ep in self.program.entry_points if ep.call_jump_target == target),
            None,
        )

    def _resolve_true_target(self, block: BasicBlock, false_target: int) -> int:
        direct_block = self.cfg.get_block_at(block.last_instruction.next_address)
        if direct_block is not None and direct_block.start_address != false_target:
            return direct_block.start_address

        for successor in self.cfg.get_successors(block):
            if successor.start_address != false_target:
                return successor.start_address

        raise Exception(
            f"Cannot resolve true branch target for block 0x{block.start_address:08x}"
        )

    def _single_successor_start(self, block: BasicBlock) -> Optional[int]:
        successors = list(self.cfg.get_successors(block))
        if not successors:
            return None
        if len(successors) > 1:
            raise Exception(
                f"Unexpected multi-successor block at 0x{block.start_address:08x}"
            )
        return successors[0].start_address

    def _require_instruction_state(self, address: int) -> StackFrame:
        state = self.stack_simulator.get_instruction_state(address)
        if state is None:
            raise Exception(f"Missing stack state at instruction 0x{address:08x}")
        return state

    def _require_variable(self, address: int) -> Variable:
        variable = self.variable_identifier.get_variable(address)
        if variable is None:
            raise Exception(f"No variable registered at address 0x{address:08x}")
        return variable

    def _collect_variable_declarations(self) -> List[Variable]:
        declarations: List[Variable] = []
        for variable in self.variable_identifier.variables.values():
            if variable.scope not in {VariableScope.LOCAL, VariableScope.TEMPORARY}:
                continue
            if not variable.read_locations and not variable.write_locations:
                continue
            if variable.name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME:
                continue

            symbol_name = (
                variable.original_symbol.name
                if variable.original_symbol
                else variable.name
            )
            if symbol_name.startswith(SymbolInfo.CONST_SYMBOL_PREFIX):
                continue

            declarations.append(variable)

        declarations.sort(key=lambda variable: variable.address)
        return declarations
