// src/core/error.rs
// Error types for RINQ

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RinqError {
    /// クエリの構築が不正な場合（無効な引数等）
    #[error("Invalid query construction: {message}")]
    InvalidQuery { message: String },

    /// イテレータが枯渇した場合（空コレクションへの要素アクセス等）
    #[error("Iterator exhausted")]
    IteratorExhausted,

    /// クエリ実行中のランタイムエラー（重複キー・要素数の期待値違反等）
    #[error("Query execution failed: {message}")]
    ExecutionError { message: String },
}

/// Result type for RINQ operations
pub type RinqResult<T> = Result<T, RinqError>;
