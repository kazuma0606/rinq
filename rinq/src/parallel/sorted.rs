// src/parallel/sorted.rs
// impl ParallelQueryBuilder<T, Sorted>

use super::ParallelQueryBuilder;
use crate::core::state::{Filtered, Sorted};
use rayon::prelude::*;
use std::marker::PhantomData;

impl<T: Send + 'static> ParallelQueryBuilder<T, Sorted> {
    /// ソート済みコレクションに対してさらに条件フィルタを適用する。
    ///
    /// **実行種別**: 並列即時実行（`Filtered` 状態に遷移）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let result: Vec<i32> = ParallelQueryBuilder::from(vec![5, 3, 1, 4, 2])
    ///     .par_order_by(|x| *x)
    ///     .par_where(|x| *x > 2)
    ///     .collect();
    /// assert_eq!(result, vec![3, 4, 5]);
    /// ```
    pub fn par_where<F>(self, predicate: F) -> ParallelQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + Sync + Send,
    {
        let items: Vec<T> = self
            .items
            .into_par_iter()
            .filter(|x| predicate(x))
            .collect();
        ParallelQueryBuilder {
            items,
            _state: PhantomData,
        }
    }

    /// ソート済みコレクションの各要素を並列に変換する。
    ///
    /// **実行種別**: 並列即時実行（`Filtered` 状態に遷移）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let result: Vec<String> = ParallelQueryBuilder::from(vec![3, 1, 2])
    ///     .par_order_by(|x| *x)
    ///     .par_select(|x| x.to_string())
    ///     .collect();
    /// assert_eq!(result, vec!["1", "2", "3"]);
    /// ```
    pub fn par_select<U, F>(self, f: F) -> ParallelQueryBuilder<U, Filtered>
    where
        U: Send + 'static,
        F: Fn(T) -> U + Sync + Send,
    {
        let items: Vec<U> = self.items.into_par_iter().map(f).collect();
        ParallelQueryBuilder {
            items,
            _state: PhantomData,
        }
    }
}
