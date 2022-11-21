/*
 * module: error
 */

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenizeError {
    #[error("InvalidTokenizer: {0}")]
    InvalidTokenizerError(String),

    #[error("Artifact: {0}")]
    ArtifactError(String),

    #[error("AcquireToker: {0}")]
    AcquireTokerError(String),

    #[error("MissingSeparator")]
    MissingSeparatorError,

    #[error("Tokenizing: {0}")]
    TokenizingError(String),

    #[error("Serde: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("IO: {0}")]
    IOError(#[from] std::io::Error),
}
