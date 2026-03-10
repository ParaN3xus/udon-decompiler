use std::collections::BTreeMap;

use crate::decompiler::module_info::ExternFunctionInfo;
use crate::decompiler::variable::VariableTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Remainder,
    UnaryMinus,
    UnaryNegation,
    ConditionalAnd,
    ConditionalOr,
    ConditionalXor,
    LogicalAnd,
    LogicalOr,
    LogicalXor,
    LeftShift,
    RightShift,
    Equality,
    Inequality,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Conversion,
}

impl IrOperator {
    pub fn from_extern_signature(signature: &str) -> Option<Self> {
        let function_name = signature.split('.').nth(1).unwrap_or(signature);
        let operator_name = function_name
            .strip_prefix("__op_")
            .and_then(|name| name.split("__").next())?;
        Self::from_name(operator_name)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Addition" => Some(Self::Addition),
            "Subtraction" => Some(Self::Subtraction),
            "Multiplication" | "Multiply" => Some(Self::Multiplication),
            "Division" => Some(Self::Division),
            "Remainder" => Some(Self::Remainder),
            "UnaryMinus" => Some(Self::UnaryMinus),
            "UnaryNegation" => Some(Self::UnaryNegation),
            "ConditionalAnd" => Some(Self::ConditionalAnd),
            "ConditionalOr" => Some(Self::ConditionalOr),
            "ConditionalXor" => Some(Self::ConditionalXor),
            "LogicalAnd" => Some(Self::LogicalAnd),
            "LogicalOr" => Some(Self::LogicalOr),
            "LogicalXor" => Some(Self::LogicalXor),
            "LeftShift" => Some(Self::LeftShift),
            "RightShift" => Some(Self::RightShift),
            "Equality" => Some(Self::Equality),
            "Inequality" => Some(Self::Inequality),
            "GreaterThan" => Some(Self::GreaterThan),
            "GreaterThanOrEqual" => Some(Self::GreaterThanOrEqual),
            "LessThan" => Some(Self::LessThan),
            "LessThanOrEqual" => Some(Self::LessThanOrEqual),
            "Implicit" | "Explicit" => Some(Self::Conversion),
            _ => None,
        }
    }

    pub fn formatter(self) -> Option<&'static str> {
        match self {
            Self::Addition => Some("{} + {}"),
            Self::Subtraction => Some("{} - {}"),
            Self::Multiplication => Some("{} * {}"),
            Self::Division => Some("{} / {}"),
            Self::Remainder => Some("{} % {}"),
            Self::UnaryMinus => Some("-{}"),
            Self::UnaryNegation => Some("!{}"),
            Self::ConditionalAnd | Self::LogicalAnd => Some("{} && {}"),
            Self::ConditionalOr | Self::LogicalOr => Some("{} || {}"),
            Self::ConditionalXor | Self::LogicalXor => Some("{} ^ {}"),
            Self::LeftShift => Some("{} << {}"),
            Self::RightShift => Some("{} >> {}"),
            Self::Equality => Some("{} == {}"),
            Self::Inequality => Some("{} != {}"),
            Self::GreaterThan => Some("{} > {}"),
            Self::GreaterThanOrEqual => Some("{} >= {}"),
            Self::LessThan => Some("{} < {}"),
            Self::LessThanOrEqual => Some("{} <= {}"),
            Self::Conversion => Some("({}){}"),
        }
    }

    pub fn is_unary(self) -> bool {
        matches!(
            self,
            Self::UnaryMinus | Self::UnaryNegation | Self::Conversion
        )
    }

    pub fn is_associative(self) -> bool {
        matches!(
            self,
            Self::Addition
                | Self::Multiplication
                | Self::LogicalAnd
                | Self::LogicalOr
                | Self::LogicalXor
        )
    }

    pub fn precedence(self) -> i32 {
        if self.is_unary() {
            return 7;
        }
        match self {
            Self::Multiplication | Self::Division | Self::Remainder => 6,
            Self::Addition | Self::Subtraction => 5,
            Self::LeftShift | Self::RightShift => 4,
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => 3,
            Self::Equality | Self::Inequality => 2,
            Self::ConditionalAnd | Self::LogicalAnd => 1,
            Self::ConditionalXor | Self::LogicalXor => 0,
            Self::ConditionalOr | Self::LogicalOr => -1,
            Self::UnaryMinus | Self::UnaryNegation | Self::Conversion => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrLiteralExpression {
    pub value: String,
    pub type_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrVariableExpression {
    pub address: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrInternalCallExpression {
    pub function_name: Option<String>,
    pub entry_address: u32,
    pub call_jump_target: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrExternalCallExpression {
    pub function_info: ExternFunctionInfo,
    pub signature: String,
    pub arguments: Vec<IrExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrPropertyAccessExpression {
    pub function_info: ExternFunctionInfo,
    pub signature: String,
    pub arguments: Vec<IrExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrConstructorCallExpression {
    pub function_info: ExternFunctionInfo,
    pub signature: String,
    pub arguments: Vec<IrExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOperatorCallExpression {
    pub arguments: Vec<IrExpression>,
    pub operator: IrOperator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrExpression {
    Literal(IrLiteralExpression),
    Variable(IrVariableExpression),
    InternalCall(IrInternalCallExpression),
    ExternalCall(IrExternalCallExpression),
    PropertyAccess(IrPropertyAccessExpression),
    ConstructorCall(IrConstructorCallExpression),
    OperatorCall(IrOperatorCallExpression),
}

impl IrExpression {
    pub fn from_heap_addr(variables: &VariableTable, address: u32) -> Self {
        let variable = variables.get_by_address(address).unwrap_or_else(|| {
            panic!("IR expression requires known heap variable at 0x{address:08X}")
        });
        Self::Variable(IrVariableExpression {
            address: variable.address,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrContainerKind {
    Block,
    Loop,
    Switch,
    While,
    DoWhile,
    For,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrAssignmentStatement {
    pub target_address: u32,
    pub value: IrExpression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrExpressionStatement {
    pub expression: IrExpression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrVariableDeclarationStatement {
    pub variable_address: u32,
    pub init_value: Option<IrLiteralExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBlock {
    pub statements: Vec<IrStatement>,
    pub start_address: u32,
    pub should_emit_label: bool,
}

impl IrBlock {
    pub fn label(&self) -> String {
        format!("BB_{:08x}", self.start_address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBlockContainer {
    pub id: u32,
    pub blocks: Vec<IrBlock>,
    pub kind: IrContainerKind,
    pub should_emit_exit_label: bool,
}

impl IrBlockContainer {
    pub fn entry_block(&self) -> Option<&IrBlock> {
        self.blocks.first()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrIf {
    pub condition: IrExpression,
    pub true_statement: Box<IrStatement>,
    pub false_statement: Option<Box<IrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrJump {
    pub target_address: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrLeave {
    pub target_container_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReturn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrSwitch {
    pub index_expression: IrExpression,
    pub cases: BTreeMap<u32, u32>,
    pub default_target: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrHighLevelSwitchSection {
    pub labels: Vec<u32>,
    pub body: IrBlock,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrHighLevelSwitch {
    pub index_expression: IrExpression,
    pub sections: Vec<IrHighLevelSwitchSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrHighLevelWhile {
    pub condition: Option<IrExpression>,
    pub body: IrBlockContainer,
    pub continue_target: u32,
    pub break_target: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrHighLevelDoWhile {
    pub condition: IrExpression,
    pub body: IrBlockContainer,
    pub continue_target: u32,
    pub break_target: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrStatement {
    Assignment(IrAssignmentStatement),
    Expression(IrExpressionStatement),
    VariableDeclaration(IrVariableDeclarationStatement),
    Block(IrBlock),
    BlockContainer(IrBlockContainer),
    If(IrIf),
    Jump(IrJump),
    Leave(IrLeave),
    Return(IrReturn),
    Switch(IrSwitch),
    HighLevelSwitch(IrHighLevelSwitch),
    HighLevelWhile(IrHighLevelWhile),
    HighLevelDoWhile(IrHighLevelDoWhile),
}

impl IrStatement {
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Self::If(_) | Self::Jump(_) | Self::Leave(_) | Self::Return(_) | Self::Switch(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub function_name: String,
    pub is_function_public: bool,
    pub entry_address: u32,
    pub variable_declarations: Vec<IrVariableDeclarationStatement>,
    pub body: IrBlockContainer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrClass {
    pub class_name: String,
    pub namespace: Option<String>,
    pub variable_declarations: Vec<IrVariableDeclarationStatement>,
    pub functions: Vec<IrFunction>,
}
