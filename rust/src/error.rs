use thiserror::Error;

pub type Result<T> = std::result::Result<T, OmniFocusError>;

#[derive(Error, Debug)]
pub enum OmniFocusError {
    #[error("JXA execution failed: {0}")]
    JxaExecution(String),
    #[error("{0}")]
    OmniFocus(String),
    #[error("JXA command returned malformed JSON.")]
    JsonParse(#[from] serde_json::Error),
    #[error("{0}")]
    Validation(String),
    #[error("I/O error while running JXA: {0}")]
    Io(#[from] std::io::Error),
    #[error("JXA command timed out after {seconds:.0}s.")]
    Timeout { seconds: f64 },
}
