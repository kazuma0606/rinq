//! Metrics-instrumented query builder and collector.
#![allow(missing_docs)]

pub mod builder;
pub mod collector;

pub use builder::MetricsQueryBuilder;
pub use collector::MetricsCollector;
