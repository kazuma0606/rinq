// src/core/try_builder.rs
// TryQueryBuilder<T, E> — failure-tolerant pipeline builder.
//
// Wraps a lazy iterator of Result<T, E> and provides two terminal operations:
//   collect_partitioned  — accumulate everything, split into (ok, err) vecs
//   collect_results      — short-circuit on the first Err

/// A lazy pipeline of fallible items produced by `try_select` or `try_where_`.
///
/// Each element in the pipeline is either `Ok(value)` or `Err(error)`.
/// Two terminal operations determine how errors are handled:
///
/// - [`collect_partitioned`]: keep going, split results into `(ok_vec, err_vec)`
/// - [`collect_results`]: stop at the first `Err`, returning `Err(e)`
///
/// [`collect_partitioned`]: TryQueryBuilder::collect_partitioned
/// [`collect_results`]: TryQueryBuilder::collect_results
pub struct TryQueryBuilder<T, E> {
    pub(crate) data: Box<dyn Iterator<Item = Result<T, E>>>,
}

impl<T, E> TryQueryBuilder<T, E> {
    /// Consume the pipeline and split all items into `(ok_values, errors)`.
    ///
    /// Processing continues even when errors are encountered — every element is
    /// examined, and errors are collected into the second vector.
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let strings = vec!["1", "two", "3", "four", "5"];
    /// let (ok, err) = QueryBuilder::from(strings)
    ///     .try_select(|s| s.parse::<i32>())
    ///     .collect_partitioned();
    ///
    /// assert_eq!(ok, vec![1, 3, 5]);
    /// assert_eq!(err.len(), 2);
    /// ```
    pub fn collect_partitioned(self) -> (Vec<T>, Vec<E>) {
        let mut ok: Vec<T> = Vec::new();
        let mut err: Vec<E> = Vec::new();
        for item in self.data {
            match item {
                Ok(v) => ok.push(v),
                Err(e) => err.push(e),
            }
        }
        (ok, err)
    }

    /// Consume the pipeline and return `Ok(vec)` if every item succeeded, or
    /// `Err(e)` on the first failure (short-circuit, remaining items are dropped).
    ///
    /// This mirrors the behaviour of `?`-propagation and
    /// `Iterator::collect::<Result<Vec<_>, _>>()`.
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// // All succeed
    /// let result = QueryBuilder::from(vec!["1", "2", "3"])
    ///     .try_select(|s| s.parse::<i32>())
    ///     .collect_results();
    /// assert_eq!(result.unwrap(), vec![1, 2, 3]);
    ///
    /// // First error short-circuits
    /// let result = QueryBuilder::from(vec!["1", "oops", "3"])
    ///     .try_select(|s| s.parse::<i32>())
    ///     .collect_results();
    /// assert!(result.is_err());
    /// ```
    pub fn collect_results(self) -> Result<Vec<T>, E> {
        let mut ok: Vec<T> = Vec::new();
        for item in self.data {
            ok.push(item?);
        }
        Ok(ok)
    }
}
