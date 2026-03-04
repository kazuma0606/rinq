// src/domain/rinq/mod.rs
// RINQ (Rust Integrated Query) v0.1
// Type-safe, zero-cost query engine for Rust

pub mod error;
pub mod query_builder;
pub mod state;

#[cfg(test)]
mod tests;

pub use error::{RinqDomainError, RinqResult};
pub use query_builder::QueryBuilder;
pub use state::{Filtered, Initial, Projected, Sorted};
