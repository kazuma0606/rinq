// src/parallel/filtered.rs
// impl ParallelQueryBuilder<T, Filtered>

use super::ParallelQueryBuilder;
use crate::core::state::{Filtered, Sorted};
use rayon::prelude::*;
use std::marker::PhantomData;

impl<T: Send + 'static> ParallelQueryBuilder<T, Filtered> {
    /// 条件を満たす要素のみ並列に抽出する。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .par_where(|x| *x > 1)
    ///     .par_where(|x| *x < 5)
    ///     .collect();
    /// result.iter().for_each(|x| assert!(*x > 1 && *x < 5));
    /// ```
    pub fn par_where<F>(self, predicate: F) -> ParallelQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + Sync + Send,
    {
        let items: Vec<T> = self.items.into_par_iter().filter(|x| predicate(x)).collect();
        ParallelQueryBuilder { items, _state: PhantomData }
    }

    /// 各要素を並列に変換する。
    ///
    /// `QueryBuilder::select` と異なり、戻り値は `Filtered` 状態のままであるため
    /// 変換後もさらに `par_where` 等の操作を連鎖できる。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .par_where(|x| *x % 2 == 0)
    ///     .par_select(|x| x * 10)
    ///     .collect();
    /// result.sort();
    /// assert_eq!(result, vec![20, 40]);
    /// ```
    pub fn par_select<U, F>(self, f: F) -> ParallelQueryBuilder<U, Filtered>
    where
        U: Send + 'static,
        F: Fn(T) -> U + Sync + Send,
    {
        let items: Vec<U> = self.items.into_par_iter().map(f).collect();
        ParallelQueryBuilder { items, _state: PhantomData }
    }

    /// ネストしたイテレータを並列に平坦化する。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![vec![1, 2], vec![3]])
    ///     .par_where(|v| !v.is_empty())
    ///     .par_flat_map(|v| v)
    ///     .collect();
    /// result.sort();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    pub fn par_flat_map<U, I, F>(self, f: F) -> ParallelQueryBuilder<U, Filtered>
    where
        U: Send + 'static,
        I: IntoIterator<Item = U>,
        F: Fn(T) -> I + Sync + Send,
    {
        let items: Vec<U> = self.items.into_par_iter().flat_map_iter(f).collect();
        ParallelQueryBuilder { items, _state: PhantomData }
    }

    /// キーセレクタによる並列昇順ソートを行う。
    ///
    /// **実行種別**: 並列即時実行（`Sorted` 状態に遷移）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let result: Vec<i32> = ParallelQueryBuilder::from(vec![3, 1, 2])
    ///     .par_where(|x| *x > 0)
    ///     .par_order_by(|x| *x)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    pub fn par_order_by<K, F>(self, key: F) -> ParallelQueryBuilder<T, Sorted>
    where
        K: Ord,
        F: Fn(&T) -> K + Sync,
        T: Send,
    {
        let mut items = self.items;
        items.par_sort_by_key(|x| key(x));
        ParallelQueryBuilder { items, _state: PhantomData }
    }
}
