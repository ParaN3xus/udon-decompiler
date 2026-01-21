from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Final, List, Optional

from udon_decompiler.analysis.stack_simulator import (
    StackSimulator,
    StackValue,
    StackValueType,
)
from udon_decompiler.analysis.variable_identifier import VariableIdentifier
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import (
    ExternFunctionInfo,
    FunctionDefinitionType,
    UdonModuleInfo,
)
from udon_decompiler.models.program import EntryPointInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class ExpressionType(Enum):
    LITERAL = "literal"
    VARIABLE = "variable"  # var ref
    OPERATOR = "op"
    EXTERNAL_CALL = "ext_call"
    INTERNAL_CALL = "int_call"
    PROPERTY_ACCESS = "prop"
    CONSTRUCTOR = "ctor"
    # ARRAY_ACCESS = "array"        # arr access
    ASSIGNMENT = "assignment"
    # CAST = "cast"                 # cast


class Operator(Enum):
    formatter: str

    Addition = ("Addition", "{} + {}")
    Subtraction = ("Subtraction", "{} - {}")
    Multiplication = ("Multiplication", "{} * {}")
    Division = ("Division", "{} / {}")
    Remainder = ("Remainder", "{} % {}")
    UnaryMinus = ("UnaryMinus", "-{}")
    UnaryNegation = ("UnaryNegation", "!{}")
    LogicalAnd = ("LogicalAnd", "{} & {}")
    LogicalOr = ("LogicalOr", "{} | {}")
    LogicalXor = ("LogicalXor", "{} ^ {}")
    LeftShift = ("LeftShift", "{} << {}")
    RightShift = ("RightShift", "{} >> {}")
    Equality = ("Equality", "{} == {}")
    Inequality = ("Inequality", "{} != {}")
    GreaterThan = ("GreaterThan", "{} > {}")
    GreaterThanOrEqual = ("GreaterThanOrEqual", "{} >= {}")
    LessThan = ("LessThan", "{} < {}")
    LessThanOrEqual = ("LessThanOrEqual", "{} <= {}")
    ImplicitConversion = ("ImplicitConversion", "({}){}")

    def __new__(cls, value, formatter):
        obj = object.__new__(cls)
        obj._value_ = value
        obj.formatter = formatter
        return obj


@dataclass
class Expression:
    expr_type: ExpressionType
    value: Any = None
    type_hint: Optional[str] = None

    # for op
    operator: Optional[Operator] = None

    # for extern
    function_info: Optional[ExternFunctionInfo] = None
    arguments: List["Expression"] = field(default_factory=list)

    # for internal call
    entry_point: Optional[EntryPointInfo] = None

    source_instruction: Optional[Instruction] = None

    def __post_init__(self):
        if self.arguments is None:
            self.arguments = []

    # todo: fill all new types
    def __repr__(self) -> str:
        if self.expr_type == ExpressionType.LITERAL:
            return f"Literal({self.value})"
        elif self.expr_type == ExpressionType.VARIABLE:
            return f"Var({self.value})"
        elif self.expr_type == ExpressionType.EXTERNAL_CALL:
            return f"Call({
                self.function_info.function_name if self.function_info else '<unknown>'
            }, {len(self.arguments)} args)"
        else:
            return f"Expr({self.expr_type.value})"


class ExpressionBuilder:
    EXTERN_OP_PREFIX: Final[str] = "__op_"

    def __init__(
        self,
        program: UdonProgramData,
        module_info: UdonModuleInfo,
        stack_simulator: StackSimulator,
        variable_identifier: VariableIdentifier,
    ):
        self.program = program
        self.module_info = module_info
        self.stack_simulator = stack_simulator
        self.variable_identifier = variable_identifier

    def build_expression_from_instruction(
        self, instruction: Instruction
    ) -> Optional[Expression]:
        opcode = instruction.opcode

        match opcode:
            case OpCode.COPY:
                return self._build_copy_expression(instruction)
            case OpCode.EXTERN:
                return self._build_extern_expression(instruction)
            case OpCode.JUMP:
                return self._build_jump_expression(instruction)
            case OpCode.JUMP_INDIRECT:
                # todo: return
                pass
            case (
                OpCode.JUMP_IF_FALSE
                | OpCode.NOP
                | OpCode.ANNOTATION
                | OpCode.POP
                | OpCode.PUSH
            ):
                return None

        return None

    def _build_copy_expression(self, instruction: Instruction) -> Optional[Expression]:
        prev_state = self._get_previous_state(instruction)
        if prev_state is None or len(prev_state.stack) < 2:
            return None

        target_val = prev_state.peek(0)
        source_val = prev_state.peek(1)

        if source_val is None or not target_val:
            return None

        target_var = self.variable_identifier.get_variable_name(target_val.value)
        source_expr = self._stack_value_to_expression(source_val)

        return Expression(
            expr_type=ExpressionType.ASSIGNMENT,
            value=target_var,
            arguments=[source_expr] if source_expr else [],
            source_instruction=instruction,
        )

    def _build_extern_expression(
        self, instruction: Instruction
    ) -> Optional[Expression]:
        if instruction.operand is None:
            return None

        heap_entry = self.program.get_initial_heap_value(instruction.operand)
        if heap_entry is None or not heap_entry.value.is_serializable:
            return None

        signature = heap_entry.value.value
        if not isinstance(signature, str):
            return None

        func_info = self.module_info.get_function_info(signature)
        if func_info is None:
            logger.warning(f"Unknown function: {signature}")
            return Expression(
                expr_type=ExpressionType.EXTERNAL_CALL,
                function_info=None,
                source_instruction=instruction,
            )

        prev_state = self._get_previous_state(instruction)
        arguments = []

        if prev_state:
            for i in range(func_info.parameter_count):
                param_val = prev_state.peek(func_info.parameter_count - 1 - i)
                if param_val:
                    arg_expr = self._stack_value_to_expression(param_val)
                    if arg_expr:
                        arguments.append(arg_expr)

        match func_info.def_type:
            case FunctionDefinitionType.FIELD:
                return Expression(
                    expr_type=ExpressionType.PROPERTY_ACCESS,
                    function_info=func_info,
                    arguments=arguments,
                    source_instruction=instruction,
                )
            case FunctionDefinitionType.CTOR:
                return Expression(
                    expr_type=ExpressionType.CONSTRUCTOR,
                    function_info=func_info,
                    arguments=arguments,
                    source_instruction=instruction,
                )
            case FunctionDefinitionType.OPERATOR:
                return Expression(
                    expr_type=ExpressionType.OPERATOR,
                    function_info=func_info,
                    operator=self._build_op(
                        func_info,
                    ),
                    arguments=arguments,
                    source_instruction=instruction,
                )
            case FunctionDefinitionType.METHOD:
                return Expression(
                    expr_type=ExpressionType.EXTERNAL_CALL,
                    function_info=func_info,
                    arguments=arguments,
                    source_instruction=instruction,
                )
            case FunctionDefinitionType.SPECIAL | FunctionDefinitionType.UNKNOWN:
                # todo, this is a fallback
                return Expression(
                    expr_type=ExpressionType.EXTERNAL_CALL,
                    function_info=func_info,
                    arguments=arguments,
                    source_instruction=instruction,
                )
            case (
                FunctionDefinitionType.CONST
                | FunctionDefinitionType.TYPE
                | FunctionDefinitionType.VARIABLE
                | FunctionDefinitionType.EVENT
            ):
                # these shouldn't happen
                raise Exception("Unexpected FunctionDefinitionType encountered!")

    def _build_jump_expression(self, instruction: Instruction) -> Optional[Expression]:
        target = instruction.get_jump_target()

        target_ep = next(
            (ep for ep in self.program.entry_points if ep.call_jump_target == target),
            None,
        )
        if target_ep is None:
            return None

        return Expression(
            expr_type=ExpressionType.INTERNAL_CALL,
            entry_point=target_ep,
            source_instruction=instruction,
        )

    def _build_op(
        self,
        func_info: ExternFunctionInfo,
    ) -> Operator:
        if not func_info.function_name.startswith(self.EXTERN_OP_PREFIX):
            raise Exception("Invalid operator")

        op = Operator(
            func_info.function_name[len(self.EXTERN_OP_PREFIX) :].split(
                "__", maxsplit=2
            )[0]
        )

        return op

    def _stack_value_to_expression(
        self, stack_value: StackValue
    ) -> Optional[Expression]:
        if stack_value.value_type == StackValueType.HEAP_ADDRESS:
            variable = self.variable_identifier.get_variable(stack_value.value)
            if variable:
                return Expression(
                    expr_type=ExpressionType.VARIABLE,
                    value=variable.name,
                    type_hint=variable.type_hint,
                )

            # in heap
            heap_entry = self.program.get_initial_heap_value(stack_value.value)
            if heap_entry and heap_entry.value.is_serializable:
                return Expression(
                    expr_type=ExpressionType.LITERAL,
                    value=heap_entry.value.value,
                    type_hint=heap_entry.type,
                )

        elif stack_value.value_type == StackValueType.IMMEDIATE:
            return Expression(
                expr_type=ExpressionType.LITERAL,
                value=stack_value.value,
                type_hint=stack_value.type_hint,
            )

        return Expression(
            expr_type=ExpressionType.LITERAL,
            value=f"0x{stack_value.value:08x}",
            type_hint="unknown",
        )

    def _get_previous_state(self, instruction: Instruction):
        # todo: use cfg to find previous instruction and block
        for block in self.stack_simulator._block_entry_states.keys():
            for i, inst in enumerate(block.instructions):
                if inst.address == instruction.address:
                    if i > 0:
                        prev_inst = block.instructions[i - 1]
                        return self.stack_simulator.get_instruction_state(
                            prev_inst.address
                        )
                    else:
                        return self.stack_simulator.get_block_entry_state(block)

        return None
