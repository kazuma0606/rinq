//! Core query engine: builder, error types, type states, and try-pipeline.
#![allow(missing_docs)]

pub mod builder;
pub mod error;
pub mod state;
pub mod try_builder;

pub use builder::{QueryBuilder, Queryable};
pub use error::{RinqError, RinqResult};
pub use state::{Filtered, Initial, Projected, Sorted};
pub use try_builder::TryQueryBuilder;
