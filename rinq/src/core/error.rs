// src/core/error.rs
// Error types for RINQ

use thiserror::Error;

/// Errors that can be returned by rinq query operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RinqError {
    /// Query construction failed due to an invalid argument.
    #[error("Invalid query construction: {message}")]
    InvalidQuery {
        /// Description of the invalid argument or construction error.
        message: String,
    },

    /// The iterator was exhausted before the requested element was found.
    /// Returned by `first()`, `last()`, `single()`, `element_at()` on empty inputs.
    #[error("Iterator exhausted")]
    IteratorExhausted,

    /// A runtime error occurred during query execution (duplicate key, wrong element count, etc.).
    #[error("Query execution failed: {message}")]
    ExecutionError {
        /// Description of the execution failure.
        message: String,
    },
}

/// Result type alias for all rinq operations that can fail.
pub type RinqResult<T> = Result<T, RinqError>;
