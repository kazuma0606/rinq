// src/core/builder/try_ops.rs
// Fallible pipeline entry points for all QueryBuilder states.
//
// try_select  — maps T → Result<U, E>, producing TryQueryBuilder<U, E>
// try_where_  — filters with a predicate that may fail, producing TryQueryBuilder<T, E>

use super::{QueryBuilder, QueryData};
use crate::core::try_builder::TryQueryBuilder;

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Apply a fallible transformation to each element.
    ///
    /// Returns a [`TryQueryBuilder`] that lazily holds `Result<U, E>` values.
    /// Use [`collect_partitioned`] to accumulate all results, or
    /// [`collect_results`] to short-circuit on the first error.
    ///
    /// This is the `Result`-aware counterpart of `select`.
    ///
    /// **実行種別**: 遅延（ターミナル操作で実行）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec!["1", "x", "3"])
    ///     .try_select(|s| s.parse::<i32>())
    ///     .collect_partitioned();
    /// assert_eq!(ok, vec![1, 3]);
    /// assert_eq!(err.len(), 1);
    /// ```
    ///
    /// [`collect_partitioned`]: crate::TryQueryBuilder::collect_partitioned
    /// [`collect_results`]: crate::TryQueryBuilder::collect_results
    pub fn try_select<U, E, F>(self, f: F) -> TryQueryBuilder<U, E>
    where
        U: 'static,
        E: 'static,
        F: Fn(T) -> Result<U, E> + 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        TryQueryBuilder {
            data: Box::new(iter.map(f)),
        }
    }

    /// Apply a fallible predicate to each element.
    ///
    /// - `Ok(true)` — the element is kept.
    /// - `Ok(false)` — the element is silently dropped.
    /// - `Err(e)` — the element is replaced by `Err(e)` in the output stream.
    ///
    /// Returns a [`TryQueryBuilder`] that can be consumed with
    /// [`collect_partitioned`] or [`collect_results`].
    ///
    /// **実行種別**: 遅延（ターミナル操作で実行）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// // Predicate always succeeds: behaves like a normal filter
    /// let result = QueryBuilder::from(vec![1, 2, 3, 4])
    ///     .try_where_(|x| Ok::<bool, String>(*x % 2 == 0))
    ///     .collect_results()
    ///     .unwrap();
    /// assert_eq!(result, vec![2, 4]);
    /// ```
    ///
    /// [`collect_partitioned`]: crate::TryQueryBuilder::collect_partitioned
    /// [`collect_results`]: crate::TryQueryBuilder::collect_results
    pub fn try_where_<E, F>(self, f: F) -> TryQueryBuilder<T, E>
    where
        E: 'static,
        F: Fn(&T) -> Result<bool, E> + 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        TryQueryBuilder {
            data: Box::new(iter.filter_map(move |x| match f(&x) {
                Ok(true) => Some(Ok(x)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            })),
        }
    }
}
