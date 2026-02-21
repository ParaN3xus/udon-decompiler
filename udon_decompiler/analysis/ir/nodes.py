from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Dict, List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.variable_identifier import Variable
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


class IRContainerKind(Enum):
    BLOCK = "block"
    LOOP = "loop"
    SWITCH = "switch"
    WHILE = "while"
    DO_WHILE = "do_while"
    FOR = "for"


@dataclass
class IRAssignmentStatement(IRStatement):
    target: Variable
    value: IRExpression


@dataclass
class IRExpressionStatement(IRStatement):
    expression: IRExpression


@dataclass
class IRVariableDeclarationStatement(IRStatement):
    variable: Variable
    init_value: Optional[IRLiteralExpression]


IRVariableDeclearationStatement = IRVariableDeclarationStatement


@dataclass(eq=False)
class IRBlock(IRStatement):
    statements: List[IRStatement]
    start_address: int = -1

    @property
    def label(self) -> str:
        return f"BB_{self.start_address:08x}"

    def __hash__(self) -> int:
        return hash(self.start_address)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, IRBlock):
            return False
        return self.start_address == other.start_address


@dataclass
class IRBlockContainer(IRStatement):
    blocks: List[IRBlock]
    kind: IRContainerKind = IRContainerKind.BLOCK

    @property
    def entry_block(self) -> Optional[IRBlock]:
        return self.blocks[0] if self.blocks else None


@dataclass
class IRIf(IRStatement):
    condition: IRExpression
    true_statement: IRStatement
    false_statement: Optional[IRStatement]


@dataclass
class IRJump(IRStatement):
    target: IRBlock


@dataclass
class IRLeave(IRStatement):
    target_container: IRBlockContainer


@dataclass
class IRReturn(IRStatement):
    pass


@dataclass
class IRSwitch(IRStatement):
    index_expression: IRExpression
    cases: Dict[int, IRBlock]
    default_target: Optional[IRBlock] = None


@dataclass
class IRFunction:
    function_name: str
    is_function_public: bool
    variable_declarations: List[IRVariableDeclarationStatement]

    body: IRBlockContainer


@dataclass
class IRClass:
    class_name: str
    namespace: Optional[str]
    program: UdonProgramData
    variable_declarations: List[IRVariableDeclarationStatement]
    functions: list[IRFunction]
