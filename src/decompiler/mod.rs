mod context;
mod instruction_list;

pub use context::{DecompileContext, DecompileHeapEntry, DecompileSymbol};
pub use instruction_list::{InstructionId, InstructionList};

pub type Result<T> = std::result::Result<T, DecompileError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompileError {
    message: String,
}

impl DecompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for DecompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecompileError {}

impl From<crate::odin::OdinError> for DecompileError {
    fn from(value: crate::odin::OdinError) -> Self {
        Self::new(value.to_string())
    }
}

impl From<std::io::Error> for DecompileError {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}
