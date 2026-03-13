/// LRGP error hierarchy.
#[derive(Debug, thiserror::Error)]
pub enum LrgpError {
    #[error("envelope too large: {0} bytes (max {1})")]
    EnvelopeTooLarge(usize, usize),

    #[error("invalid envelope: {0}")]
    InvalidEnvelope(String),

    #[error("illegal transition: cannot apply '{command}' to session in '{status}' state")]
    IllegalTransition { command: String, status: String },

    #[error("unknown game: {0}")]
    UnknownApp(String),

    #[error("validation error [{code}]: {message}")]
    Validation { code: String, message: String },

    #[error("store error: {0}")]
    Store(String),
}
