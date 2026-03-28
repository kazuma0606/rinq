// src/parallel/initial.rs
// impl ParallelQueryBuilder<T, Initial>

use super::ParallelQueryBuilder;
use crate::core::state::{Filtered, Initial, Sorted};
use rayon::prelude::*;
use std::marker::PhantomData;

impl<T: Send + 'static> ParallelQueryBuilder<T, Initial> {
    /// `Vec<T>` から `ParallelQueryBuilder` を生成する。
    ///
    /// **実行種別**: 即時実行（Vecへの変換のみ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let qb = ParallelQueryBuilder::from(vec![1, 2, 3]);
    /// let result: Vec<i32> = qb.collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    pub fn from(data: Vec<T>) -> Self {
        Self {
            items: data,
            _state: PhantomData,
        }
    }

    /// 条件を満たす要素のみ並列に抽出する。
    ///
    /// **実行種別**: 並列即時実行（`Filtered` 状態に遷移）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .par_where(|x| *x % 2 == 0)
    ///     .collect();
    /// result.iter().for_each(|x| assert_eq!(*x % 2, 0));
    /// ```
    pub fn par_where<F>(self, predicate: F) -> ParallelQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + Sync + Send,
    {
        let items: Vec<T> = self.items.into_par_iter().filter(|x| predicate(x)).collect();
        ParallelQueryBuilder { items, _state: PhantomData }
    }

    /// ネストしたイテレータを並列に平坦化する。
    ///
    /// **実行種別**: 並列即時実行（`Filtered` 状態に遷移）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![vec![1, 2], vec![3, 4]])
    ///     .par_flat_map(|v| v)
    ///     .collect();
    /// result.sort();
    /// assert_eq!(result, vec![1, 2, 3, 4]);
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
    /// let result: Vec<i32> = ParallelQueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .par_order_by(|x| *x)
    ///     .collect();
    /// assert_eq!(result, vec![1, 1, 3, 4, 5]);
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
