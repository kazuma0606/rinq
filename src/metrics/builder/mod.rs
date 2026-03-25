// src/metrics/builder/mod.rs
// MetricsQueryBuilder type definition

mod impl_;

use crate::core::builder::QueryBuilder;
use crate::metrics::collector::MetricsCollector;
use std::sync::Arc;

/// Wrapper around QueryBuilder that records metrics for query operations
///
/// This struct integrates RINQ with rusted-ca's metrics collection system,
/// allowing tracking of query execution times and operation counts.
pub struct MetricsQueryBuilder<T, State> {
    inner: QueryBuilder<T, State>,
    metrics: Arc<MetricsCollector>,
    operation_name: String,
}
