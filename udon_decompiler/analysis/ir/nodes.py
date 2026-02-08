from dataclasses import dataclass, field
from typing import Dict, List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.variable_identifier import Variable
from udon_decompiler.models.instruction import Instruction
from udon_decompiler.models.module_info import ExternFunctionInfo
from udon_decompiler.models.program import EntryPointInfo, UdonProgramData


class IRExpression:
    pass


@dataclass
class LiteralExpression(IRExpression):
    value: object
    type_hint: Optional[str]


@dataclass
class VariableExpression(IRExpression):
    variable: Variable


@dataclass
class InternalCallExpression(IRExpression):
    entry_point: EntryPointInfo


@dataclass
class ExternalCallExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]


@dataclass
class PropertyAccessExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]


@dataclass
class ConstructorCallExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]


@dataclass
class OperatorCallExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]
    operator: Operator


class IRStatement:
    pass


@dataclass
class AssignmentStatement(IRStatement):
    instruction_address: int
    instruction: Instruction
    target: Variable
    value: IRExpression


@dataclass
class ExpressionStatement(IRStatement):
    instruction_address: int
    instruction: Instruction
    expression: IRExpression


class IRTerminator:
    address: int


@dataclass
class GotoTerminator(IRTerminator):
    address: int
    target: int


@dataclass
class ConditionalTerminator(IRTerminator):
    address: int
    condition: IRExpression
    true_target: int
    false_target: int


@dataclass
class SwitchTerminator(IRTerminator):
    address: int
    switch_index: IRExpression
    switch_targets: List[int]


@dataclass
class ReturnTerminator(IRTerminator):
    address: int


@dataclass
class EndTerminator(IRTerminator):
    address: int


@dataclass
class BlockIR:
    start_address: int
    end_address: int
    statements: List[IRStatement]
    terminator: IRTerminator


@dataclass
class FunctionIR:
    function_name: str
    is_function_public: bool
    entry_block_start: int
    variable_declarations: List[Variable]
    block_order: List[int]
    blocks: Dict[int, BlockIR] = field(default_factory=dict)


@dataclass
class GlobalVariableIR:
    variable: Variable
    initial_value: object


@dataclass
class ClassIR:
    class_name: str
    namespace: Optional[str]
    program: UdonProgramData
    global_variables: list[GlobalVariableIR]
    functions: list[FunctionIR]
