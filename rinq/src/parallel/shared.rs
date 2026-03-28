// src/parallel/shared.rs
// impl<T, State> ParallelQueryBuilder<T, State> — 全状態共通の終端操作

use super::ParallelQueryBuilder;
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Sum;

impl<T: Send + 'static, State> ParallelQueryBuilder<T, State> {
    /// 並列に収集してコレクションを返す。
    ///
    /// 内部データはすでに `Vec<T>` として保持されているため、
    /// `Vec<T>` への収集は追加コストなし。
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3])
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    pub fn collect<B>(self) -> B
    where
        B: FromIterator<T>,
    {
        self.items.into_iter().collect()
    }

    /// 要素数を返す。
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let count = ParallelQueryBuilder::from(vec![1, 2, 3])
    ///     .par_where(|x| *x > 1)
    ///     .par_count();
    /// assert_eq!(count, 2);
    /// ```
    pub fn par_count(self) -> usize {
        self.items.len()
    }

    /// 並列に合計を計算する。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let total: i32 = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .par_sum();
    /// assert_eq!(total, 15);
    /// ```
    pub fn par_sum(self) -> T
    where
        T: Sum + Send,
    {
        self.items.into_par_iter().sum()
    }

    /// 並列に最小値を返す。空コレクションは `None`。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let min = ParallelQueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .par_min();
    /// assert_eq!(min, Some(1));
    /// ```
    pub fn par_min(self) -> Option<T>
    where
        T: Ord + Send,
    {
        self.items.into_par_iter().min()
    }

    /// 並列に最大値を返す。空コレクションは `None`。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let max = ParallelQueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .par_max();
    /// assert_eq!(max, Some(5));
    /// ```
    pub fn par_max(self) -> Option<T>
    where
        T: Ord + Send,
    {
        self.items.into_par_iter().max()
    }

    /// 並列に最小値をキーセレクタで探す。空コレクションは `None`。
    ///
    /// **実行種別**: 並列即時実行
    pub fn par_min_by<K, F>(self, key: F) -> Option<T>
    where
        K: Ord + Send,
        F: Fn(&T) -> K + Sync + Send,
    {
        self.items.into_par_iter().min_by_key(|x| key(x))
    }

    /// 並列に最大値をキーセレクタで探す。空コレクションは `None`。
    ///
    /// **実行種別**: 並列即時実行
    pub fn par_max_by<K, F>(self, key: F) -> Option<T>
    where
        K: Ord + Send,
        F: Fn(&T) -> K + Sync + Send,
    {
        self.items.into_par_iter().max_by_key(|x| key(x))
    }

    /// いずれかの要素が条件を満たすか並列に検査する。
    ///
    /// **実行種別**: 並列即時実行（条件成立次第ショートサーキット）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let found = ParallelQueryBuilder::from(vec![1, 2, 3])
    ///     .par_any(|x| *x == 2);
    /// assert!(found);
    /// ```
    pub fn par_any<F>(self, predicate: F) -> bool
    where
        T: Sync,
        F: Fn(&T) -> bool + Sync + Send,
    {
        self.items.par_iter().any(predicate)
    }

    /// すべての要素が条件を満たすか並列に検査する。
    ///
    /// **実行種別**: 並列即時実行（条件不成立次第ショートサーキット）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let all_positive = ParallelQueryBuilder::from(vec![1, 2, 3])
    ///     .par_all(|x| *x > 0);
    /// assert!(all_positive);
    /// ```
    pub fn par_all<F>(self, predicate: F) -> bool
    where
        T: Sync,
        F: Fn(&T) -> bool + Sync + Send,
    {
        self.items.par_iter().all(predicate)
    }

    /// キーセレクタで並列にグループ化する。
    ///
    /// グループ内の要素順序は実行ごとに異なる場合があるが、
    /// グループのキー集合と各グループの要素集合は逐次版と一致する。
    ///
    /// **実行種別**: 並列即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::parallel::ParallelQueryBuilder;
    ///
    /// let groups = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .par_group_by(|x| x % 2);
    /// assert_eq!(groups[&0].len(), 2); // 偶数: 2, 4
    /// assert_eq!(groups[&1].len(), 3); // 奇数: 1, 3, 5
    /// ```
    pub fn par_group_by<K, F>(self, key: F) -> HashMap<K, Vec<T>>
    where
        K: Eq + Hash + Send,
        F: Fn(&T) -> K + Sync + Send,
    {
        self.items
            .into_par_iter()
            .fold(
                || HashMap::<K, Vec<T>>::new(),
                |mut acc, item| {
                    acc.entry(key(&item)).or_default().push(item);
                    acc
                },
            )
            .reduce(
                || HashMap::new(),
                |mut a, b| {
                    for (k, v) in b {
                        a.entry(k).or_default().extend(v);
                    }
                    a
                },
            )
    }
}
