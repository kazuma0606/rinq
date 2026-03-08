// src/domain/rinq/metrics_query_builder.rs
// Metrics-aware QueryBuilder wrapper for integration with rusted-ca

use super::query_builder::QueryBuilder;
use super::state::{Filtered, Initial, Projected, Sorted};
use crate::shared::metrics::collector::MetricsCollector;
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

impl<T: 'static> MetricsQueryBuilder<T, Initial> {
    /// Create a new MetricsQueryBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use rusted_ca::domain::rinq::MetricsQueryBuilder;
    /// use rusted_ca::domain::rinq::QueryBuilder;
    /// use rusted_ca::shared::metrics::collector::MetricsCollector;
    ///
    /// let metrics = Arc::new(MetricsCollector::new());
    /// let data = vec![1, 2, 3, 4, 5];
    ///
    /// let result: Vec<_> = MetricsQueryBuilder::new(
    ///     QueryBuilder::from(data),
    ///     metrics,
    ///     "my_query".to_string(),
    /// )
    /// .where_(|x| *x > 2)
    /// .collect();
    ///
    /// assert_eq!(result, vec![3, 4, 5]);
    /// ```
    pub fn new(
        inner: QueryBuilder<T, Initial>,
        metrics: Arc<MetricsCollector>,
        operation_name: String,
    ) -> Self {
        Self {
            inner,
            metrics,
            operation_name,
        }
    }

    /// Filter elements based on a predicate
    #[inline]
    pub fn where_<F>(self, predicate: F) -> MetricsQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.where_(predicate),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Sort elements by a key in ascending order
    #[inline]
    pub fn order_by<K, F>(self, key_selector: F) -> MetricsQueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.order_by(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Sort elements by a key in descending order
    #[inline]
    pub fn order_by_descending<K, F>(self, key_selector: F) -> MetricsQueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.order_by_descending(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Take the first n elements
    #[inline]
    pub fn take(self, n: usize) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.take(n),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Skip the first n elements
    #[inline]
    pub fn skip(self, n: usize) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.skip(n),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }
}

impl<T: 'static> MetricsQueryBuilder<T, Filtered> {
    /// Further filter elements
    #[inline]
    pub fn where_<F>(self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            inner: self.inner.where_(predicate),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Sort elements by a key in ascending order
    #[inline]
    pub fn order_by<K, F>(self, key_selector: F) -> MetricsQueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.order_by(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Sort elements by a key in descending order
    #[inline]
    pub fn order_by_descending<K, F>(self, key_selector: F) -> MetricsQueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.order_by_descending(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Take the first n elements
    #[inline]
    pub fn take(self, n: usize) -> Self {
        Self {
            inner: self.inner.take(n),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Skip the first n elements
    #[inline]
    pub fn skip(self, n: usize) -> Self {
        Self {
            inner: self.inner.skip(n),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Project elements to a different type
    #[inline]
    pub fn select<U, F>(self, projection: F) -> MetricsQueryBuilder<U, Projected<U>>
    where
        F: Fn(T) -> U + 'static,
        U: 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.select(projection),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }
}

impl<T: 'static> MetricsQueryBuilder<T, Sorted> {
    /// Apply a secondary sort key
    #[inline]
    pub fn then_by<K, F>(self, key_selector: F) -> Self
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
    {
        Self {
            inner: self.inner.then_by(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Apply a secondary sort key in descending order
    #[inline]
    pub fn then_by_descending<K, F>(self, key_selector: F) -> Self
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
    {
        Self {
            inner: self.inner.then_by_descending(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Take the first n elements
    #[inline]
    pub fn take(self, n: usize) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.take(n),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Skip the first n elements
    #[inline]
    pub fn skip(self, n: usize) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.skip(n),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }
}

// Terminal operations with metrics recording
impl<T: 'static, State> MetricsQueryBuilder<T, State> {
    /// Collect the results into a collection and record metrics
    #[inline]
    pub fn collect<B>(self) -> B
    where
        B: FromIterator<T>,
    {
        let start = std::time::Instant::now();
        let result = self.inner.collect();
        let duration = start.elapsed();

        self.metrics.record_query_execution(&self.operation_name, duration);

        result
    }

    /// Count the number of elements and record metrics
    #[inline]
    pub fn count(self) -> usize {
        let start = std::time::Instant::now();
        let result = self.inner.count();
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_count", self.operation_name),
            duration,
        );

        result
    }

    /// Get the first element and record metrics
    #[inline]
    pub fn first(self) -> Option<T> {
        let start = std::time::Instant::now();
        let result = self.inner.first();
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_first", self.operation_name),
            duration,
        );

        result
    }

    /// Get the last element and record metrics
    #[inline]
    pub fn last(self) -> Option<T> {
        let start = std::time::Instant::now();
        let result = self.inner.last();
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_last", self.operation_name),
            duration,
        );

        result
    }

    /// Check if any element satisfies the predicate and record metrics
    #[inline]
    pub fn any<F>(self, predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        let start = std::time::Instant::now();
        let result = self.inner.any(predicate);
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_any", self.operation_name),
            duration,
        );

        result
    }

    /// Check if all elements satisfy the predicate and record metrics
    #[inline]
    pub fn all<F>(self, predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        let start = std::time::Instant::now();
        let result = self.inner.all(predicate);
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_all", self.operation_name),
            duration,
        );

        result
    }
}
