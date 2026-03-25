//! # rinq — Rust Integrated Query
//!
//! A type-safe, zero-cost LINQ-inspired query engine for Rust.
//!
//! `rinq` lets you compose filter → sort → aggregate pipelines over any
//! in-memory collection using a fluent builder API.  The type-state pattern
//! encodes the valid operation order at compile time, so invalid chains
//! (e.g. calling `order_by` after `select`) are rejected without runtime
//! overhead.
//!
//! ## Quick Start
//!
//! ```rust
//! use rinq::QueryBuilder;
//!
//! let total: i32 = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
//!     .where_(|x| x % 2 == 0)   // keep evens
//!     .order_by(|x| *x)          // sort ascending
//!     .sum();                    // terminal — evaluates the pipeline
//!
//! assert_eq!(total, 30);
//! ```
//!
//! ## State Machine
//!
//! Every `QueryBuilder<T, State>` carries a compile-time state that restricts
//! which methods are available:
//!
//! | State | Available transitions |
//! |---|---|
//! | [`Initial`] | `where_`, `take`, `skip`, `flat_map`, `order_by`, `group_by`, … |
//! | [`Filtered`] | same as `Initial` plus `select` |
//! | [`Sorted`] | `then_by`, `then_by_descending`, plus all terminal ops |
//! | [`Projected<U>`] | `collect()` only |
//!
//! ## Feature Flags
//!
//! | Feature | What it enables |
//! |---|---|
//! | `parallel` | [`ParallelQueryBuilder`] via [rayon](https://docs.rs/rayon) |
//! | `serde` | [`QueryBuilder::from_json`] deserialization via [serde_json](https://docs.rs/serde_json) |
//!
//! Enable features in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rinq = { version = "3", features = ["parallel", "serde"] }
//! ```
//!
//! ## Statistical Extensions
//!
//! For descriptive statistics, correlation, regression, sampling, and
//! validation, see the companion crate
//! [`rinq-stats`](https://docs.rs/rinq-stats).

#![warn(missing_docs)]

pub mod core;
pub mod metrics;
#[cfg(feature = "parallel")]
pub mod parallel;
#[cfg(feature = "serde")]
pub mod serde;

pub use core::builder::{QueryBuilder, Queryable};
pub use core::error::{RinqError, RinqResult};
pub use core::state::{Filtered, Initial, Projected, Sorted};
pub use core::try_builder::TryQueryBuilder;
pub use metrics::builder::MetricsQueryBuilder;
pub use metrics::collector::MetricsCollector;
#[cfg(feature = "parallel")]
pub use parallel::ParallelQueryBuilder;
