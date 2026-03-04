// src/domain/rinq/error.rs
// Error types for RINQ domain layer

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RinqDomainError {
    #[error("Invalid query construction: {message}")]
    InvalidQuery { message: String },

    #[error("Iterator exhausted")]
    IteratorExhausted,

    #[error("Query execution failed: {message}")]
    ExecutionError { message: String },

    #[error("Invalid query state: {message}")]
    InvalidState { message: String },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}

/// Result type for RINQ operations
pub type RinqResult<T> = Result<T, RinqDomainError>;
