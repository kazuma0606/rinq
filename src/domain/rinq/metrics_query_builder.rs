// src/domain/rinq/metrics_query_builder.rs
// Metrics-aware QueryBuilder wrapper for integration with rusted-ca

use super::query_builder::QueryBuilder;
use super::state::{Filtered, Initial, Projected, Sorted};
use crate::shared::metrics::collector::MetricsCollector;
use std::collections::HashMap;
use std::hash::Hash;
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

    /// Calculate sum with metrics
    #[inline]
    pub fn sum(self) -> T
    where
        T: std::iter::Sum,
    {
        let start = std::time::Instant::now();
        let result = self.inner.sum();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_sum", self.operation_name), duration);

        result
    }

    /// Calculate average with metrics
    #[inline]
    pub fn average(self) -> Option<f64>
    where
        T: num_traits::ToPrimitive,
    {
        let start = std::time::Instant::now();
        let result = self.inner.average();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_average", self.operation_name), duration);

        result
    }

    /// Find minimum with metrics
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.min();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_min", self.operation_name), duration);

        result
    }

    /// Find maximum with metrics
    #[inline]
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.max();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_max", self.operation_name), duration);

        result
    }

    /// Find minimum by key with metrics
    #[inline]
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.min_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_min_by", self.operation_name), duration);

        result
    }

    /// Find maximum by key with metrics
    #[inline]
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.max_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_max_by", self.operation_name), duration);

        result
    }

    /// Group by with metrics
    #[inline]
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        let start = std::time::Instant::now();
        let result = self.inner.group_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_group_by", self.operation_name), duration);

        result
    }

    /// Group by aggregate with metrics
    #[inline]
    pub fn group_by_aggregate<K, R, FK, FA>(self, key_selector: FK, aggregator: FA) -> HashMap<K, R>
    where
        FK: Fn(&T) -> K,
        FA: Fn(&[T]) -> R,
        K: Eq + Hash,
    {
        let start = std::time::Instant::now();
        let result = self.inner.group_by_aggregate(key_selector, aggregator);
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_group_by_aggregate", self.operation_name),
            duration,
        );

        result
    }

    /// Distinct with state transition
    #[inline]
    pub fn distinct(self) -> MetricsQueryBuilder<T, Filtered>
    where
        T: Eq + Hash + Clone,
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Distinct by with state transition
    #[inline]
    pub fn distinct_by<K, F>(self, key_selector: F) -> MetricsQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + Hash + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct_by(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Reverse with state transition
    #[inline]
    pub fn reverse(self) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.reverse(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Chunk with state transition
    #[inline]
    pub fn chunk(self, size: usize) -> MetricsQueryBuilder<Vec<T>, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.chunk(size),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Window with state transition
    #[inline]
    pub fn window(self, size: usize) -> MetricsQueryBuilder<Vec<T>, Filtered>
    where
        T: Clone,
    {
        MetricsQueryBuilder {
            inner: self.inner.window(size),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Zip with state transition
    #[inline]
    pub fn zip<U, I>(self, other: I) -> MetricsQueryBuilder<(T, U), Filtered>
    where
        U: 'static,
        I: IntoIterator<Item = U> + 'static,
        I::IntoIter: 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.zip(other),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Enumerate with state transition
    #[inline]
    pub fn enumerate(self) -> MetricsQueryBuilder<(usize, T), Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.enumerate(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Partition with metrics
    #[inline]
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        let start = std::time::Instant::now();
        let result = self.inner.partition(predicate);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_partition", self.operation_name), duration);

        result
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

    /// Calculate sum with metrics (Filtered state)
    #[inline]
    pub fn sum(self) -> T
    where
        T: std::iter::Sum,
    {
        let start = std::time::Instant::now();
        let result = self.inner.sum();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_sum", self.operation_name), duration);

        result
    }

    /// Calculate average with metrics (Filtered state)
    #[inline]
    pub fn average(self) -> Option<f64>
    where
        T: num_traits::ToPrimitive,
    {
        let start = std::time::Instant::now();
        let result = self.inner.average();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_average", self.operation_name), duration);

        result
    }

    /// Find minimum with metrics (Filtered state)
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.min();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_min", self.operation_name), duration);

        result
    }

    /// Find maximum with metrics (Filtered state)
    #[inline]
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.max();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_max", self.operation_name), duration);

        result
    }

    /// Find minimum by key with metrics (Filtered state)
    #[inline]
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.min_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_min_by", self.operation_name), duration);

        result
    }

    /// Find maximum by key with metrics (Filtered state)
    #[inline]
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.max_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_max_by", self.operation_name), duration);

        result
    }

    /// Group by with metrics (Filtered state)
    #[inline]
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        let start = std::time::Instant::now();
        let result = self.inner.group_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_group_by", self.operation_name), duration);

        result
    }

    /// Group by aggregate with metrics (Filtered state)
    #[inline]
    pub fn group_by_aggregate<K, R, FK, FA>(self, key_selector: FK, aggregator: FA) -> HashMap<K, R>
    where
        FK: Fn(&T) -> K,
        FA: Fn(&[T]) -> R,
        K: Eq + Hash,
    {
        let start = std::time::Instant::now();
        let result = self.inner.group_by_aggregate(key_selector, aggregator);
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_group_by_aggregate", self.operation_name),
            duration,
        );

        result
    }

    /// Distinct with state transition (Filtered state)
    #[inline]
    pub fn distinct(self) -> Self
    where
        T: Eq + Hash + Clone,
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Distinct by with state transition (Filtered state)
    #[inline]
    pub fn distinct_by<K, F>(self, key_selector: F) -> Self
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + Hash + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct_by(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Reverse with state transition (Filtered state)
    #[inline]
    pub fn reverse(self) -> Self {
        MetricsQueryBuilder {
            inner: self.inner.reverse(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Chunk with state transition (Filtered state)
    #[inline]
    pub fn chunk(self, size: usize) -> MetricsQueryBuilder<Vec<T>, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.chunk(size),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Window with state transition (Filtered state)
    #[inline]
    pub fn window(self, size: usize) -> MetricsQueryBuilder<Vec<T>, Filtered>
    where
        T: Clone,
    {
        MetricsQueryBuilder {
            inner: self.inner.window(size),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Zip with state transition (Filtered state)
    #[inline]
    pub fn zip<U, I>(self, other: I) -> MetricsQueryBuilder<(T, U), Filtered>
    where
        U: 'static,
        I: IntoIterator<Item = U> + 'static,
        I::IntoIter: 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.zip(other),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Enumerate with state transition (Filtered state)
    #[inline]
    pub fn enumerate(self) -> MetricsQueryBuilder<(usize, T), Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.enumerate(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Partition with metrics (Filtered state)
    #[inline]
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        let start = std::time::Instant::now();
        let result = self.inner.partition(predicate);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_partition", self.operation_name), duration);

        result
    }
}

impl<T: 'static> MetricsQueryBuilder<T, Sorted> {
    /// Further filter elements
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

    /// Calculate sum with metrics (Sorted state)
    #[inline]
    pub fn sum(self) -> T
    where
        T: std::iter::Sum,
    {
        let start = std::time::Instant::now();
        let result = self.inner.sum();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_sum", self.operation_name), duration);

        result
    }

    /// Calculate average with metrics (Sorted state)
    #[inline]
    pub fn average(self) -> Option<f64>
    where
        T: num_traits::ToPrimitive,
    {
        let start = std::time::Instant::now();
        let result = self.inner.average();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_average", self.operation_name), duration);

        result
    }

    /// Find minimum with metrics (Sorted state, O(1))
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.min();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_min", self.operation_name), duration);

        result
    }

    /// Find maximum with metrics (Sorted state, O(1))
    #[inline]
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.max();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_max", self.operation_name), duration);

        result
    }

    /// Find minimum by key with metrics (Sorted state)
    #[inline]
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.min_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_min_by", self.operation_name), duration);

        result
    }

    /// Find maximum by key with metrics (Sorted state)
    #[inline]
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let start = std::time::Instant::now();
        let result = self.inner.max_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_max_by", self.operation_name), duration);

        result
    }

    /// Group by with metrics (Sorted state)
    #[inline]
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        let start = std::time::Instant::now();
        let result = self.inner.group_by(key_selector);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_group_by", self.operation_name), duration);

        result
    }

    /// Group by aggregate with metrics (Sorted state)
    #[inline]
    pub fn group_by_aggregate<K, R, FK, FA>(self, key_selector: FK, aggregator: FA) -> HashMap<K, R>
    where
        FK: Fn(&T) -> K,
        FA: Fn(&[T]) -> R,
        K: Eq + Hash,
    {
        let start = std::time::Instant::now();
        let result = self.inner.group_by_aggregate(key_selector, aggregator);
        let duration = start.elapsed();

        self.metrics.record_query_execution(
            &format!("{}_group_by_aggregate", self.operation_name),
            duration,
        );

        result
    }

    /// Distinct with state transition (Sorted state)
    #[inline]
    pub fn distinct(self) -> MetricsQueryBuilder<T, Filtered>
    where
        T: Eq + Hash + Clone,
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Distinct by with state transition (Sorted state)
    #[inline]
    pub fn distinct_by<K, F>(self, key_selector: F) -> MetricsQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + Hash + 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct_by(key_selector),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Reverse with state transition (Sorted state)
    #[inline]
    pub fn reverse(self) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.reverse(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Chunk with state transition (Sorted state)
    #[inline]
    pub fn chunk(self, size: usize) -> MetricsQueryBuilder<Vec<T>, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.chunk(size),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Window with state transition (Sorted state)
    #[inline]
    pub fn window(self, size: usize) -> MetricsQueryBuilder<Vec<T>, Filtered>
    where
        T: Clone,
    {
        MetricsQueryBuilder {
            inner: self.inner.window(size),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Zip with state transition (Sorted state)
    #[inline]
    pub fn zip<U, I>(self, other: I) -> MetricsQueryBuilder<(T, U), Filtered>
    where
        U: 'static,
        I: IntoIterator<Item = U> + 'static,
        I::IntoIter: 'static,
    {
        MetricsQueryBuilder {
            inner: self.inner.zip(other),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Enumerate with state transition (Sorted state)
    #[inline]
    pub fn enumerate(self) -> MetricsQueryBuilder<(usize, T), Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.enumerate(),
            metrics: self.metrics,
            operation_name: self.operation_name,
        }
    }

    /// Partition with metrics (Sorted state)
    #[inline]
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        let start = std::time::Instant::now();
        let result = self.inner.partition(predicate);
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_partition", self.operation_name), duration);

        result
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

        self.metrics
            .record_query_execution(&self.operation_name, duration);

        result
    }

    /// Count the number of elements and record metrics
    #[inline]
    pub fn count(self) -> usize {
        let start = std::time::Instant::now();
        let result = self.inner.count();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_count", self.operation_name), duration);

        result
    }

    /// Get the first element and record metrics
    #[inline]
    pub fn first(self) -> Option<T> {
        let start = std::time::Instant::now();
        let result = self.inner.first();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_first", self.operation_name), duration);

        result
    }

    /// Get the last element and record metrics
    #[inline]
    pub fn last(self) -> Option<T> {
        let start = std::time::Instant::now();
        let result = self.inner.last();
        let duration = start.elapsed();

        self.metrics
            .record_query_execution(&format!("{}_last", self.operation_name), duration);

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

        self.metrics
            .record_query_execution(&format!("{}_any", self.operation_name), duration);

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

        self.metrics
            .record_query_execution(&format!("{}_all", self.operation_name), duration);

        result
    }
}
