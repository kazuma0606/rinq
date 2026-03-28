//! Parallel query execution via [rayon](https://docs.rs/rayon).
//!
//! Enabled with the `parallel` feature flag.
//! Convert any [`crate::QueryBuilder`] to a [`ParallelQueryBuilder`] using
//! [`crate::QueryBuilder::into_parallel`].
#![allow(missing_docs)]

mod filtered;
mod initial;
mod shared;
mod sorted;

use std::marker::PhantomData;

/// 並列クエリビルダー。`rayon` を使ってデータを並列処理する。
///
/// `QueryBuilder` と同じ型ステートパターンを採用しており、
/// コンパイル時に操作の順序を検証する。
///
/// # 基本的な使い方
///
/// ```
/// use rinq::parallel::ParallelQueryBuilder;
///
/// let result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
///     .par_where(|x| *x > 2)
///     .collect();
/// assert_eq!(result, vec![3, 4, 5]);
/// ```
///
/// # `QueryBuilder` からの変換
///
/// ```
/// use rinq::QueryBuilder;
///
/// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
///     .where_(|x| *x > 0)
///     .into_parallel()
///     .par_where(|x| *x % 2 == 0)
///     .collect();
/// assert_eq!(result, vec![2, 4]);
/// ```
pub struct ParallelQueryBuilder<T, State> {
    pub(crate) items: Vec<T>,
    pub(crate) _state: PhantomData<State>,
}
