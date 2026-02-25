use std::sync::OnceLock;

use tracing_subscriber::EnvFilter;

pub type Result<T> = std::result::Result<T, LoggingError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggingError {
    message: String,
}

impl LoggingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for LoggingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LoggingError {}

static LOGGING_INIT: OnceLock<()> = OnceLock::new();

pub fn init_logging(level: &str) -> Result<()> {
    if LOGGING_INIT.get().is_some() {
        return Ok(());
    }

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .map_err(|e| LoggingError::new(format!("invalid log filter '{}': {}", level, e)))?;

    let result = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_ansi(true)
        .compact()
        .try_init();

    result.map_err(|e| LoggingError::new(format!("failed to initialize logging: {}", e)))?;
    let _ = LOGGING_INIT.set(());
    Ok(())
}
