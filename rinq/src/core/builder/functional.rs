// src/core/builder/functional.rs
// Phase 4D operators: scan, chunk_by, dedup, dedup_by, zip_with, pairwise,
// unfold, unfold_bounded, intersperse, min_max

use super::iterators::{ChunkByIterator, UnfoldBoundedIter, UnfoldIter};
use super::{QueryBuilder, QueryData};
use crate::core::state::{Filtered, Initial};
use std::marker::PhantomData;

/// Scan implementation helper struct.
struct ScanIter<T, B, F> {
    inner: Box<dyn Iterator<Item = T>>,
    state: Option<B>,
    f: F,
}

impl<T, B: Clone, F: FnMut(B, T) -> B> Iterator for ScanIter<T, B, F> {
    type Item = B;

    fn next(&mut self) -> Option<B> {
        let x = self.inner.next()?;
        let acc = self.state.take()?;
        let next_acc = (self.f)(acc, x);
        self.state = Some(next_acc.clone());
        Some(next_acc)
    }
}

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Produce a running accumulator sequence (like a lazy fold).
    ///
    /// For each element `x`, `acc = f(acc, x)` and the updated `acc` is
    /// yielded.  Requires `B: Clone`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // Cumulative product: [1, 2, 6, 24, 120]
    /// let result: Vec<i64> = QueryBuilder::from(vec![1i64, 2, 3, 4, 5])
    ///     .scan(1i64, |acc, x| acc * x)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 6, 24, 120]);
    /// ```
    pub fn scan<B, F>(self, seed: B, f: F) -> QueryBuilder<B, Filtered>
    where
        B: Clone + 'static,
        F: FnMut(B, T) -> B + 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(ScanIter {
                inner: iter,
                state: Some(seed),
                f,
            })),
            _state: PhantomData,
        }
    }

    /// Group consecutive elements that share the same key into `Vec<T>` chunks.
    ///
    /// Unlike `group_by`, only *consecutive* equal-key elements are grouped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 1, 2, 2, 3, 1, 1];
    /// let chunks: Vec<Vec<i32>> = QueryBuilder::from(data)
    ///     .chunk_by(|x| *x)
    ///     .collect();
    /// assert_eq!(chunks, vec![vec![1, 1], vec![2, 2], vec![3], vec![1, 1]]);
    /// ```
    pub fn chunk_by<K, F>(self, key_fn: F) -> QueryBuilder<Vec<T>, Filtered>
    where
        K: PartialEq + 'static,
        F: FnMut(&T) -> K + 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(ChunkByIterator {
                inner: iter,
                key_fn,
                buffered: None,
                done: false,
            })),
            _state: PhantomData,
        }
    }

    /// Remove consecutive duplicate elements (based on `PartialEq`).
    ///
    /// Non-consecutive duplicates are **not** removed (use `distinct` for that).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // distinct would remove all duplicates; dedup keeps non-consecutive ones
    /// let data = vec![1, 1, 2, 3, 3, 1];
    /// let result: Vec<i32> = QueryBuilder::from(data).dedup().collect();
    /// assert_eq!(result, vec![1, 2, 3, 1]);
    /// ```
    pub fn dedup(self) -> QueryBuilder<T, Filtered>
    where
        T: PartialEq + Clone,
    {
        self.dedup_by(|x| x.clone())
    }

    /// Remove consecutive duplicate elements based on a key function.
    ///
    /// A new element is kept only when its key differs from the previous one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct Item { category: &'static str, value: i32 }
    ///
    /// let data = vec![
    ///     Item { category: "a", value: 1 },
    ///     Item { category: "a", value: 2 },
    ///     Item { category: "b", value: 3 },
    /// ];
    ///
    /// let result: Vec<Item> = QueryBuilder::from(data)
    ///     .dedup_by(|x| x.category)
    ///     .collect();
    /// assert_eq!(result.len(), 2);
    /// ```
    pub fn dedup_by<K, F>(self, mut key_fn: F) -> QueryBuilder<T, Filtered>
    where
        K: PartialEq + 'static,
        F: FnMut(&T) -> K + 'static,
        T: 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let mut last_key: Option<K> = None;
        let filtered = iter.filter(move |item| {
            let key = key_fn(item);
            if last_key.as_ref() == Some(&key) {
                false
            } else {
                last_key = Some(key);
                true
            }
        });
        QueryBuilder {
            data: QueryData::Iterator(Box::new(filtered)),
            _state: PhantomData,
        }
    }

    /// Zip with another iterable, applying a combining function to each pair.
    ///
    /// Stops at the shorter sequence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // Element-wise addition
    /// let a = vec![1, 2, 3];
    /// let b = vec![10, 20, 30];
    /// let result: Vec<i32> = QueryBuilder::from(a)
    ///     .zip_with(b, |x, y| x + y)
    ///     .collect();
    /// assert_eq!(result, vec![11, 22, 33]);
    /// ```
    pub fn zip_with<U, R, I, F>(self, other: I, f: F) -> QueryBuilder<R, Filtered>
    where
        U: 'static,
        R: 'static,
        I: IntoIterator<Item = U> + 'static,
        I::IntoIter: 'static,
        F: Fn(T, U) -> R + 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let zipped = iter.zip(other).map(move |(a, b)| f(a, b));
        QueryBuilder {
            data: QueryData::Iterator(Box::new(zipped)),
            _state: PhantomData,
        }
    }

    /// Produce consecutive overlapping pairs: `[a, b, c] → [(a,b), (b,c)]`.
    ///
    /// Returns an empty sequence for inputs of length 0 or 1.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4];
    /// let pairs: Vec<(i32, i32)> = QueryBuilder::from(data).pairwise().collect();
    /// assert_eq!(pairs, vec![(1, 2), (2, 3), (3, 4)]);
    /// ```
    pub fn pairwise(self) -> QueryBuilder<(T, T), Filtered>
    where
        T: Clone,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(PairwiseIter {
                inner: iter,
                prev: None,
            })),
            _state: PhantomData,
        }
    }

    /// Insert a separator element between every pair of consecutive elements.
    ///
    /// The separator is **not** inserted before the first element or after the
    /// last element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // Space-separated words
    /// let words = vec!["hello", "world", "foo"];
    /// let result: Vec<&str> = QueryBuilder::from(words)
    ///     .intersperse(" ")
    ///     .collect();
    /// assert_eq!(result, vec!["hello", " ", "world", " ", "foo"]);
    /// ```
    pub fn intersperse(self, separator: T) -> QueryBuilder<T, Filtered>
    where
        T: Clone,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(IntersperseIter {
                inner: iter,
                separator,
                pending: None,
                first: true,
            })),
            _state: PhantomData,
        }
    }

    /// Return both the minimum and maximum in a single pass.
    ///
    /// Returns `None` for an empty collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result = QueryBuilder::from(vec![3, 1, 4, 1, 5, 9]).min_max();
    /// assert_eq!(result, Some((1, 9)));
    ///
    /// let empty = QueryBuilder::from(Vec::<i32>::new()).min_max();
    /// assert_eq!(empty, None);
    /// ```
    pub fn min_max(self) -> Option<(T, T)>
    where
        T: Ord + Clone,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let mut iter = iter;
        let first = iter.next()?;
        let mut min = first.clone();
        let mut max = first;
        for item in iter {
            if item < min {
                min = item.clone();
            }
            if item > max {
                max = item;
            }
        }
        Some((min, max))
    }
}

// unfold / unfold_bounded on Initial state (generation operations)
impl<T: 'static> QueryBuilder<T, Initial> {
    /// Generate a sequence by repeatedly applying a function to a seed state.
    ///
    /// The function returns `Some((yield_value, new_state))` to continue or
    /// `None` to stop.  Always pair with `take` or use `unfold_bounded` to
    /// avoid an infinite iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // First 6 Fibonacci numbers via unfold + take
    /// let fibs: Vec<u64> = QueryBuilder::unfold((0u64, 1u64), |(a, b)| Some((a, (b, a + b))))
    ///     .take(6)
    ///     .collect();
    /// assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5]);
    /// ```
    pub fn unfold<S, F>(seed: S, f: F) -> QueryBuilder<T, Initial>
    where
        S: 'static,
        F: FnMut(S) -> Option<(T, S)> + 'static,
    {
        QueryBuilder {
            data: QueryData::Iterator(Box::new(UnfoldIter {
                state: Some(seed),
                f,
            })),
            _state: PhantomData,
        }
    }

    /// Like [`unfold`](Self::unfold) but capped at `max` elements.
    ///
    /// In debug builds, a warning is printed to stderr when the cap is hit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // Bounded Fibonacci: stop after 8 elements or when sequence ends
    /// let fibs: Vec<u64> = QueryBuilder::unfold_bounded(
    ///     (0u64, 1u64),
    ///     |(a, b)| Some((a, (b, a + b))),
    ///     8,
    /// )
    /// .collect();
    /// assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13]);
    /// ```
    pub fn unfold_bounded<S, F>(seed: S, f: F, max: usize) -> QueryBuilder<T, Initial>
    where
        S: 'static,
        F: FnMut(S) -> Option<(T, S)> + 'static,
    {
        QueryBuilder {
            data: QueryData::Iterator(Box::new(UnfoldBoundedIter {
                inner: UnfoldIter {
                    state: Some(seed),
                    f,
                },
                remaining: max,
            })),
            _state: PhantomData,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper iterator structs
// ─────────────────────────────────────────────────────────────────────────────

struct PairwiseIter<T> {
    inner: Box<dyn Iterator<Item = T>>,
    prev: Option<T>,
}

impl<T: Clone> Iterator for PairwiseIter<T> {
    type Item = (T, T);

    fn next(&mut self) -> Option<(T, T)> {
        // Initialise `prev` on the first call
        if self.prev.is_none() {
            self.prev = Some(self.inner.next()?);
        }
        let next = self.inner.next()?;
        let pair = (self.prev.as_ref().unwrap().clone(), next.clone());
        self.prev = Some(next);
        Some(pair)
    }
}

struct IntersperseIter<T> {
    inner: Box<dyn Iterator<Item = T>>,
    separator: T,
    pending: Option<T>,
    first: bool,
}

impl<T: Clone> Iterator for IntersperseIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        // Emit any pending separator first
        if let Some(sep) = self.pending.take() {
            return Some(sep);
        }

        let item = self.inner.next()?;

        if self.first {
            self.first = false;
            Some(item)
        } else {
            // Queue the actual item and emit separator now
            self.pending = Some(item);
            Some(self.separator.clone())
        }
    }
}
