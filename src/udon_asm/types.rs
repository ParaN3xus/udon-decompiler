use super::literal::HeapLiteralValue;

pub type Result<T> = std::result::Result<T, AsmError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmError {
    message: String,
}

impl AsmError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AsmError {}

impl From<crate::odin::OdinError> for AsmError {
    fn from(value: crate::odin::OdinError) -> Self {
        Self::new(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Nop = 0,
    Push = 1,
    Pop = 2,
    JumpIfFalse = 4,
    Jump = 5,
    Extern = 6,
    Annotation = 7,
    JumpIndirect = 8,
    Copy = 9,
}

impl OpCode {
    pub(crate) fn from_u32(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Self::Nop),
            1 => Ok(Self::Push),
            2 => Ok(Self::Pop),
            4 => Ok(Self::JumpIfFalse),
            5 => Ok(Self::Jump),
            6 => Ok(Self::Extern),
            7 => Ok(Self::Annotation),
            8 => Ok(Self::JumpIndirect),
            9 => Ok(Self::Copy),
            _ => Err(AsmError::new(format!("Unknown opcode value {}", value))),
        }
    }

    pub(crate) fn from_name(value: &str) -> Result<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "NOP" => Ok(Self::Nop),
            "PUSH" => Ok(Self::Push),
            "POP" => Ok(Self::Pop),
            "JUMP_IF_FALSE" => Ok(Self::JumpIfFalse),
            "JUMP" => Ok(Self::Jump),
            "EXTERN" => Ok(Self::Extern),
            "ANNOTATION" => Ok(Self::Annotation),
            "JUMP_INDIRECT" => Ok(Self::JumpIndirect),
            "COPY" => Ok(Self::Copy),
            _ => Err(AsmError::new(format!("Unknown opcode '{}'", value))),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Nop => "NOP",
            Self::Push => "PUSH",
            Self::Pop => "POP",
            Self::JumpIfFalse => "JUMP_IF_FALSE",
            Self::Jump => "JUMP",
            Self::Extern => "EXTERN",
            Self::Annotation => "ANNOTATION",
            Self::JumpIndirect => "JUMP_INDIRECT",
            Self::Copy => "COPY",
        }
    }

    pub(crate) fn has_operand(self) -> bool {
        matches!(
            self,
            Self::Push
                | Self::JumpIfFalse
                | Self::Jump
                | Self::Extern
                | Self::Annotation
                | Self::JumpIndirect
        )
    }

    pub(crate) fn is_direct_jump(self) -> bool {
        matches!(self, Self::Jump | Self::JumpIfFalse)
    }

    pub(crate) fn is_heap_operand(self) -> bool {
        matches!(
            self,
            Self::Push | Self::Extern | Self::Annotation | Self::JumpIndirect
        )
    }

    pub(crate) fn size(self) -> u32 {
        if self.has_operand() { 8 } else { 4 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandToken {
    Label(String),
    HeapSymbol(String),
    Number(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmInstruction {
    pub labels: Vec<String>,
    pub opcode: OpCode,
    pub operand: Option<OperandToken>,
}

impl AsmInstruction {
    pub fn numeric_operand(&self) -> u32 {
        // todo: labels & symbol names can also be numeric by returning their
        // addresses
        match self.operand.as_ref() {
            Some(OperandToken::Number(v)) => *v,
            _ => panic!(
                "instruction {:?} requires numeric operand but operand was {:?}",
                self.opcode, self.operand
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub(crate) struct EntryDirective {
    pub(crate) visibility: Visibility,
    pub(crate) label: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub(crate) enum TypeRefDirective {
    Name(String),
}

#[derive(Debug, Clone)]
pub(crate) struct HeapDirective {
    pub(crate) export_mark: HeapExportMark,
    pub(crate) symbol: String,
    pub(crate) type_ref: TypeRefDirective,
    pub(crate) init: HeapLiteralValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapExportMark {
    Exported,
    Private,
    Keep,
}

#[derive(Debug, Clone)]
pub(crate) struct BindDirective {
    pub(crate) symbol: String,
    pub(crate) label: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BindTableDirective {
    pub(crate) symbol: String,
    pub(crate) labels: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AsmDirectives {
    pub(crate) entries: Vec<EntryDirective>,
    pub(crate) heap: Vec<HeapDirective>,
    pub(crate) binds: Vec<BindDirective>,
    pub(crate) bind_tables: Vec<BindTableDirective>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ParsedAsm {
    pub(crate) directives: AsmDirectives,
    pub(crate) instructions: Vec<AsmInstruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectiveSection {
    EntryPoints,
    Heap,
    Binds,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DecodedInstruction {
    pub(crate) address: u32,
    pub(crate) opcode: OpCode,
    pub(crate) operand: Option<u32>,
}
