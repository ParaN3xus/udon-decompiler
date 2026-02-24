from dataclasses import dataclass, field
from typing import Dict, List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.variable_identifier import Variable
from udon_decompiler.models.instruction import Instruction
from udon_decompiler.models.module_info import ExternFunctionInfo
from udon_decompiler.models.program import EntryPointInfo, UdonProgramData

# region IRExpression


class IRExpression:
    pass


@dataclass
class IRLiteralExpression(IRExpression):
    value: object
    type_hint: Optional[str]


@dataclass
class IRVariableExpression(IRExpression):
    variable: Variable


@dataclass
class IRInternalCallExpression(IRExpression):
    entry_point: EntryPointInfo


@dataclass
class IRExternalCallExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]


@dataclass
class IRPropertyAccessExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]


@dataclass
class IRConstructorCallExpression(IRExpression):
    function_info: ExternFunctionInfo
    signature: str
    arguments: List[IRExpression]


@dataclass
class IROperatorCallExpression(IRExpression):
    arguments: List[IRExpression]
    operator: Operator


# endregion


class IRStatement:
    pass


@dataclass
class IRAssignmentStatement(IRStatement):
    target: Variable
    value: IRExpression


@dataclass
class IRExpressionStatement(IRStatement):
    expression: IRExpression


@dataclass
class IRVariableDeclearationStatement(IRStatement):
    variable: Variable
    init_value: Optional[IRLiteralExpression]


@dataclass
class IRBlock(IRStatement):
    statements: List[IRStatement]


@dataclass
class IRBlockContainer(IRStatement):
    blocks: List[IRBlock]


@dataclass
class IRIf(IRStatement):
    condition: IRExpression
    true_statement: IRStatement
    false_statement: Optional[IRStatement]


@dataclass
class IRJump(IRStatement):
    target: IRBlock


@dataclass
class IRFunction:
    function_name: str
    is_function_public: bool
    variable_declarations: List[IRVariableDeclearationStatement]

    body: IRBlockContainer


@dataclass
class IRClass:
    class_name: str
    namespace: Optional[str]
    program: UdonProgramData
    variable_declarations: List[IRVariableDeclearationStatement]
    functions: list[IRFunction]
