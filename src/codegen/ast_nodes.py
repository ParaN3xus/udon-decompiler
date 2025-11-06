from dataclasses import dataclass, field
from typing import List, Optional, Any
from enum import Enum


class ASTNodeType(Enum):
    PROGRAM = "program"
    FUNCTION = "function"
    BLOCK = "block"
    STATEMENT = "statement"
    EXPRESSION = "expression"
    IF = "if"
    IF_ELSE = "if_else"
    WHILE = "while"
    DO_WHILE = "do_while"
    ASSIGNMENT = "assignment"
    CALL = "call"
    RETURN = "return"
    VARIABLE_DECL = "variable_decl"
    LABEL = "label"
    GOTO = "goto"


@dataclass
class ASTNode:
    node_type: ASTNodeType
    children: List['ASTNode'] = field(default_factory=list)
    metadata: dict = field(default_factory=dict)

    def add_child(self, child: 'ASTNode') -> None:
        self.children.append(child)

    def __repr__(self) -> str:
        return f"ASTNode({self.node_type.value}, children={len(self.children)})"


@dataclass
class ProgramNode(ASTNode):
    functions: List['FunctionNode'] = field(default_factory=list)

    def __post_init__(self):
        self.node_type = ASTNodeType.PROGRAM


@dataclass
class FunctionNode(ASTNode):
    name: str = ""
    parameters: List['VariableDeclNode'] = field(default_factory=list)
    return_type: Optional[str] = None
    body: Optional['BlockNode'] = None
    node_type: ASTNodeType = field(default=ASTNodeType.FUNCTION, init=False)

    def __post_init__(self):
        self.node_type = ASTNodeType.FUNCTION


@dataclass
class BlockNode(ASTNode):
    statements: List['StatementNode'] = field(default_factory=list)
    node_type: ASTNodeType = field(default=ASTNodeType.BLOCK, init=False)

    def __post_init__(self):
        self.node_type = ASTNodeType.BLOCK

    def add_statement(self, stmt: 'StatementNode') -> None:
        self.statements.append(stmt)


@dataclass
class StatementNode(ASTNode):
    address: Optional[int] = None
    node_type: ASTNodeType = field(default=ASTNodeType.STATEMENT, init=False)

    def __post_init__(self):
        self.node_type = ASTNodeType.STATEMENT


@dataclass
class ExpressionStatementNode(StatementNode):
    expression: Optional['ExpressionNode'] = None


@dataclass
class AssignmentNode(StatementNode):
    target: str = ""
    value: Optional['ExpressionNode'] = None
    node_type: ASTNodeType = field(default=ASTNodeType.ASSIGNMENT, init=False)

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.ASSIGNMENT


@dataclass
class IfNode(StatementNode):
    condition: Optional['ExpressionNode'] = None
    then_block: Optional[BlockNode] = None
    node_type: ASTNodeType = field(default=ASTNodeType.IF, init=False)

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.IF


@dataclass
class IfElseNode(StatementNode):
    condition: Optional['ExpressionNode'] = None
    then_block: Optional[BlockNode] = None
    else_block: Optional[BlockNode] = None
    node_type: ASTNodeType = field(default=ASTNodeType.IF_ELSE, init=False)

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.IF_ELSE


@dataclass
class WhileNode(StatementNode):
    condition: Optional['ExpressionNode'] = None
    body: Optional[BlockNode] = None
    node_type: ASTNodeType = field(default=ASTNodeType.WHILE, init=False)

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.WHILE


@dataclass
class DoWhileNode(StatementNode):
    condition: Optional['ExpressionNode'] = None
    body: Optional[BlockNode] = None
    node_type: ASTNodeType = field(default=ASTNodeType.DO_WHILE, init=False)

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.DO_WHILE


@dataclass
class VariableDeclNode(StatementNode):
    var_name: str = ""
    var_type: Optional[str] = None
    initial_value: Optional['ExpressionNode'] = None
    node_type: ASTNodeType = field(
        default=ASTNodeType.VARIABLE_DECL, init=False)

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.VARIABLE_DECL


@dataclass
class LabelNode(StatementNode):
    label_name: str = ""

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.LABEL


@dataclass
class GotoNode(StatementNode):
    target_label: str = ""

    def __post_init__(self):
        super().__post_init__()
        self.node_type = ASTNodeType.GOTO


@dataclass
class ExpressionNode(ASTNode):
    expr_type: str = ""  # "literal", "variable", "call", etc.
    value: Any = None
    node_type: ASTNodeType = field(default=ASTNodeType.EXPRESSION, init=False)

    def __post_init__(self):
        self.node_type = ASTNodeType.EXPRESSION


@dataclass
class LiteralNode(ExpressionNode):
    literal_type: Optional[str] = None  # "int", "string", "bool", etc.

    def __post_init__(self):
        super().__post_init__()
        self.expr_type = "literal"


@dataclass
class VariableNode(ExpressionNode):
    var_name: str = ""
    var_type: Optional[str] = None

    def __post_init__(self):
        super().__post_init__()
        self.expr_type = "variable"


@dataclass
class CallNode(ExpressionNode):
    function_signature: str = ""
    arguments: List[ExpressionNode] = field(default_factory=list)

    def __post_init__(self):
        super().__post_init__()
        self.expr_type = "call"
