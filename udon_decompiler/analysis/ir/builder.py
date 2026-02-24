from typing import Dict, Final, List, Optional, cast

from udon_decompiler.analysis.basic_block import (
    BasicBlock,
    BasicBlockType,
    SwitchTableInfo,
)
from udon_decompiler.analysis.cfg import ControlFlowGraph
from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.ir.nodes import (
    IRAssignmentStatement,
    IRBlock,
    IRBlockContainer,
    IRConstructorCallExpression,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRFunction,
    IRIf,
    IRInternalCallExpression,
    IRJump,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRStatement,
    IRVariableExpression,
)
from udon_decompiler.analysis.stack_simulator import (
    StackFrame,
    StackSimulator,
    StackValue,
)
from udon_decompiler.analysis.variable_identifier import (
    Variable,
    VariableIdentifier,
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
    _SYNTHETIC_DECLARATION_INSTRUCTION: Final[Instruction] = Instruction(
        address=-1,
        opcode=OpCode.NOP,
    )

    def __init__(
        self,
        program: UdonProgramData,
        function_name: str,
        is_function_public: bool,
        raw_blocks: List[BasicBlock],
        stack_simulator: StackSimulator,
        variable_identifier: VariableIdentifier,
    ):
        self.program = program
        self.is_function_public = is_function_public
        self.function_name = function_name
        self.raw_blocks = raw_blocks
        self.stack_simulator = stack_simulator
        self.variable_identifier = variable_identifier
        self.module_info = UdonModuleInfo()

        self._block_map: Dict[int, IRBlock] = {}

    def build(self) -> IRFunction:
        for raw_block in self.raw_blocks:
            ir_block = IRBlock(statements=[])
            self._block_map[raw_block.start_address] = ir_block

        body = IRBlockContainer(blocks=[])
        for raw_block in self.raw_blocks:
            ir_block = self._block_map[raw_block.start_address]
            ir_block.statements = self._build_statements(raw_block)
            body.blocks.append(ir_block)

        return IRFunction(
            function_name=self.function_name,
            is_function_public=self.is_function_public,
            variable_declarations=[],
            body=body,
        )

    # region _build_statements

    def _build_statements(self, block: BasicBlock) -> List[IRStatement]:
        statements: List[IRStatement] = []
        for instruction in block.instructions:
            statements.extend(
                self._build_statements_from_instruction(block, instruction)
            )
        return statements

    def _build_statements_from_instruction(
        self, block: BasicBlock, instruction: Instruction
    ) -> List[IRStatement]:
        match instruction.opcode:
            case OpCode.COPY:
                return self._build_copy_statements(instruction)
            case OpCode.EXTERN:
                return self._build_extern_statements(instruction)
            case OpCode.JUMP:
                return self._build_jump_statements(instruction)
            case OpCode.JUMP_IF_FALSE:
                return self._build_jump_if_false_statements(instruction)
            case OpCode.JUMP_INDIRECT:
                return self._build_jump_indirect_statements(block, instruction)
            case OpCode.NOP | OpCode.ANNOTATION | OpCode.POP | OpCode.PUSH:
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
            IRAssignmentStatement(
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
                cast(
                    IRStatement,
                    IRExpressionStatement(expression=call_expression),
                )
            ]

        if not arguments:
            raise Exception(
                f"Non-void EXTERN at 0x{instruction.address:08x} has no return slot"
            )

        return_slot = arguments[-1]
        if not isinstance(return_slot, IRVariableExpression):
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
            IRAssignmentStatement(
                target=return_slot.variable,
                value=call_expression,
            )
        ]

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

    def _build_extern_expression(
        self,
        function_info: ExternFunctionInfo,
        signature: str,
        arguments: List[IRExpression],
    ) -> IRExpression:
        match function_info.def_type:
            case FunctionDefinitionType.FIELD:
                return IRPropertyAccessExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                )
            case FunctionDefinitionType.CTOR:
                return IRConstructorCallExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                )
            case FunctionDefinitionType.OPERATOR:
                return IROperatorCallExpression(
                    arguments=arguments,
                    operator=self._resolve_operator(function_info),
                )
            case FunctionDefinitionType.METHOD:
                return IRExternalCallExpression(
                    function_info=function_info,
                    signature=signature,
                    arguments=arguments,
                )

    def _build_jump_statements(self, instruction: Instruction) -> List[IRStatement]:
        internal_call_entry = self._resolve_internal_call_entry(instruction)
        if internal_call_entry is None:
            return self._build_internal_call_statements(instruction)

        target_addr = instruction.get_jump_target()
        return [IRJump(target=self._get_block_ref(target_addr))]

    def _build_internal_call_statements(
        self, instruction: Instruction
    ) -> List[IRStatement]:
        target_entry = self._resolve_internal_call_entry(instruction)
        if target_entry is None:
            return []
        return [
            IRExpressionStatement(
                expression=IRInternalCallExpression(entry_point=target_entry),
            )
        ]

    def _build_jump_if_false_statements(
        self, instruction: Instruction
    ) -> List[IRStatement]:
        false_addr = instruction.get_jump_target()
        false_block = self._get_block_ref(false_addr)
        jump_statement: IRStatement = IRJump(target=false_block)

        condition = self._build_condition_expression(instruction)

        return [
            cast(
                IRStatement,
                IRIf(
                    condition=IROperatorCallExpression(
                        arguments=[condition], operator=Operator.UnaryNegation
                    ),
                    true_statement=jump_statement,
                    false_statement=None,
                ),
            )
        ]

    def _build_jump_indirect_statements(
        self, block: BasicBlock, instruction: Instruction
    ) -> List[IRStatement]:
        switch_info = block.switch_info
        if switch_info is None:
            raise Exception("JUMP_INDIRECT without switch_info detected!")
        index_expr = self._operand_to_expression(switch_info.index_operand)

        cases: Dict[int, IRBlock] = {}
        for case_val, target_addr in enumerate(switch_info.targets):
            cases[case_val] = self._get_block_ref(target_addr)

        # todo!

    # endregion

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
            return IRVariableExpression(variable=variable)

        raise Exception("Unknown stack address 0x%08x in IR build" % stack_value.value)

    def _operand_to_expression(self, operand: int) -> IRExpression:
        variable = self.variable_identifier.get_variable(operand)
        if variable is not None:
            return IRVariableExpression(variable=variable)

        raise Exception(f"Unknown operand 0x{operand:08x} for switch index")

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

    def _get_block_ref(self, address: int) -> IRBlock:
        if address not in self._block_map:
            raise Exception(f"Target address 0x{address:08x} not found in block map")
        return self._block_map[address]

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
