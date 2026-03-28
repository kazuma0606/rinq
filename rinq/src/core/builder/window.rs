// src/core/builder/window.rs
// Window analytics functions for all QueryBuilder states.
// All methods materialise the iterator, apply the window transform,
// and return QueryBuilder<U, Filtered> so further operations are possible.

use super::iterators::MovingAverageIterator;
use super::{QueryBuilder, QueryData};
use crate::core::state::Filtered;
use num_traits::ToPrimitive;
use std::marker::PhantomData;
use std::ops::Add;

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Compute the cumulative (prefix) sum at each position.
    ///
    /// The i-th output element equals the sum of all input elements up to and
    /// including position i.
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .running_sum()
    ///     .collect();
    /// assert_eq!(result, vec![1, 3, 6, 10, 15]);
    /// ```
    pub fn running_sum(self) -> QueryBuilder<T, Filtered>
    where
        T: Copy + Add<Output = T>,
    {
        let items = self.into_vec();
        let mut out = Vec::with_capacity(items.len());
        let mut acc: Option<T> = None;
        for item in items {
            acc = Some(match acc {
                None => item,
                Some(a) => a + item,
            });
            out.push(acc.unwrap());
        }
        QueryBuilder {
            data: QueryData::Iterator(Box::new(out.into_iter())),
            _state: PhantomData,
        }
    }

    /// Compute the cumulative average at each position.
    ///
    /// The i-th output element equals the mean of all input elements from
    /// index 0 through i (inclusive).
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<f64> = QueryBuilder::from(vec![1, 2, 3])
    ///     .running_average()
    ///     .collect();
    /// assert_eq!(result, vec![1.0, 1.5, 2.0]);
    /// ```
    pub fn running_average(self) -> QueryBuilder<f64, Filtered>
    where
        T: ToPrimitive,
    {
        let items = self.into_vec();
        let mut out = Vec::with_capacity(items.len());
        let mut sum = 0.0f64;
        for (i, item) in items.into_iter().enumerate() {
            sum += item.to_f64().unwrap_or(0.0);
            out.push(sum / (i + 1) as f64);
        }
        QueryBuilder {
            data: QueryData::Iterator(Box::new(out.into_iter())),
            _state: PhantomData,
        }
    }

    /// Compute the sliding-window average over the last `window` elements.
    ///
    /// The first `window - 1` output values are `None` because the window is
    /// not yet full. From index `window - 1` onward each value is
    /// `Some(mean of the last window elements)`.
    ///
    /// # Panics
    ///
    /// Panics if `window == 0`.
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let ma: Vec<Option<f64>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .moving_average(3)
    ///     .collect();
    /// assert_eq!(ma[0], None);
    /// assert_eq!(ma[1], None);
    /// assert_eq!(ma[2], Some(2.0));
    /// assert_eq!(ma[3], Some(3.0));
    /// assert_eq!(ma[4], Some(4.0));
    /// ```
    pub fn moving_average(self, window: usize) -> QueryBuilder<Option<f64>, Filtered>
    where
        T: ToPrimitive,
    {
        assert!(window >= 1, "moving_average: window must be >= 1");
        let floats: Vec<f64> = self
            .into_vec()
            .into_iter()
            .map(|x| x.to_f64().unwrap_or(0.0))
            .collect();
        QueryBuilder {
            data: QueryData::Iterator(Box::new(MovingAverageIterator::new(floats, window))),
            _state: PhantomData,
        }
    }

    /// Assign a 1-based rank to each element using the given key selector.
    ///
    /// Elements with equal keys receive the same rank. The next distinct rank
    /// skips over the tied positions (standard competition ranking: 1, 2, 2, 4, …).
    /// The output order matches the original input order.
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// // unique values
    /// let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![30, 10, 20])
    ///     .rank_by(|x| *x)
    ///     .collect();
    /// assert_eq!(ranked, vec![(3, 30), (1, 10), (2, 20)]);
    ///
    /// // tied values: 1, 2, 2, 4 (rank 3 is skipped)
    /// let tied: Vec<(usize, i32)> = QueryBuilder::from(vec![10, 30, 30, 20])
    ///     .rank_by(|x| *x)
    ///     .collect();
    /// assert_eq!(tied[0].0, 1); // 10 → rank 1
    /// assert_eq!(tied[1].0, 3); // 30 → rank 3
    /// assert_eq!(tied[2].0, 3); // 30 → rank 3
    /// assert_eq!(tied[3].0, 2); // 20 → rank 2
    /// ```
    pub fn rank_by<K, F>(self, key: F) -> QueryBuilder<(usize, T), Filtered>
    where
        K: Ord,
        F: Fn(&T) -> K,
    {
        let items = self.into_vec();
        let rank_map = build_rank_map(&items, &key, false);
        let result: Vec<(usize, T)> = items
            .into_iter()
            .enumerate()
            .map(|(i, x)| (rank_map[i], x))
            .collect();
        QueryBuilder {
            data: QueryData::Iterator(Box::new(result.into_iter())),
            _state: PhantomData,
        }
    }

    /// Assign a 1-based dense rank to each element using the given key selector.
    ///
    /// Elements with equal keys receive the same rank. Unlike [`rank_by`], no
    /// ranks are skipped after a tie (dense ranking: 1, 2, 2, 3, …).
    /// The output order matches the original input order.
    ///
    /// [`rank_by`]: QueryBuilder::rank_by
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let dense: Vec<(usize, i32)> = QueryBuilder::from(vec![10, 30, 30, 20])
    ///     .dense_rank_by(|x| *x)
    ///     .collect();
    /// assert_eq!(dense[0].0, 1); // 10 → rank 1
    /// assert_eq!(dense[1].0, 3); // 30 → rank 3
    /// assert_eq!(dense[2].0, 3); // 30 → rank 3
    /// assert_eq!(dense[3].0, 2); // 20 → rank 2
    /// ```
    pub fn dense_rank_by<K, F>(self, key: F) -> QueryBuilder<(usize, T), Filtered>
    where
        K: Ord,
        F: Fn(&T) -> K,
    {
        let items = self.into_vec();
        let rank_map = build_rank_map(&items, &key, true);
        let result: Vec<(usize, T)> = items
            .into_iter()
            .enumerate()
            .map(|(i, x)| (rank_map[i], x))
            .collect();
        QueryBuilder {
            data: QueryData::Iterator(Box::new(result.into_iter())),
            _state: PhantomData,
        }
    }

    /// Pair each element with the element `n` positions before it.
    ///
    /// Returns `(Option<T>, T)` tuples in input order. The first `n` elements
    /// have `None` as the lagged value.
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![10, 20, 30])
    ///     .lag(1)
    ///     .collect();
    /// assert_eq!(result, vec![(None, 10), (Some(10), 20), (Some(20), 30)]);
    /// ```
    pub fn lag(self, n: usize) -> QueryBuilder<(Option<T>, T), Filtered>
    where
        T: Clone,
    {
        let items = self.into_vec();
        let result: Vec<(Option<T>, T)> = items
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let prev = if i >= n {
                    Some(items[i - n].clone())
                } else {
                    None
                };
                (prev, x.clone())
            })
            .collect();
        QueryBuilder {
            data: QueryData::Iterator(Box::new(result.into_iter())),
            _state: PhantomData,
        }
    }

    /// Pair each element with the element `n` positions after it.
    ///
    /// Returns `(T, Option<T>)` tuples in input order. The last `n` elements
    /// have `None` as the lead value.
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<(i32, Option<i32>)> = QueryBuilder::from(vec![10, 20, 30])
    ///     .lead(1)
    ///     .collect();
    /// assert_eq!(result, vec![(10, Some(20)), (20, Some(30)), (30, None)]);
    /// ```
    pub fn lead(self, n: usize) -> QueryBuilder<(T, Option<T>), Filtered>
    where
        T: Clone,
    {
        let items = self.into_vec();
        let len = items.len();
        let result: Vec<(T, Option<T>)> = items
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let next = if i + n < len {
                    Some(items[i + n].clone())
                } else {
                    None
                };
                (x.clone(), next)
            })
            .collect();
        QueryBuilder {
            data: QueryData::Iterator(Box::new(result.into_iter())),
            _state: PhantomData,
        }
    }
}

// ── internal helpers ─────────────────────────────────────────────────────────

/// Build a rank map from original indices to 1-based ranks.
///
/// When `dense == true`, ranks are consecutive (1, 2, 2, 3, …).
/// When `dense == false`, ranks use competition style (1, 2, 2, 4, …).
fn build_rank_map<T, K, F>(items: &[T], key: &F, dense: bool) -> Vec<usize>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    if items.is_empty() {
        return vec![];
    }

    // Collect (original_index, key) pairs and sort by key.
    let mut keyed: Vec<(usize, K)> = items.iter().enumerate().map(|(i, x)| (i, key(x))).collect();
    keyed.sort_by(|a, b| a.1.cmp(&b.1));

    let mut rank_map = vec![0usize; items.len()];
    let mut rank = 1usize;
    let mut i = 0;
    while i < keyed.len() {
        // Find the end of the run of equal keys.
        let mut j = i + 1;
        while j < keyed.len() && keyed[j].1 == keyed[i].1 {
            j += 1;
        }
        // Assign the same rank to all tied elements.
        for entry in &keyed[i..j] {
            rank_map[entry.0] = rank;
        }
        rank = if dense { rank + 1 } else { rank + (j - i) };
        i = j;
    }
    rank_map
}
