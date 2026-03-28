//! Serde-oriented import path for [`crate::QueryBuilder`].
//!
//! Enabled with the `serde` feature flag.
//! Exposes [`crate::QueryBuilder::from_json`] for deserialising JSON arrays
//! directly into a query pipeline.

pub use crate::core::builder::QueryBuilder;
