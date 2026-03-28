// src/core/builder/filtered.rs
// impl QueryBuilder<T, Filtered> — both blocks

use super::iterators::{ChunkIterator, WindowIterator};
use super::{QueryBuilder, QueryData};
use crate::core::state::{Filtered, Projected, Sorted};
use crate::core::state_diagnostics::HashEqBound;
use num_traits::ToPrimitive;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::iter::Sum;
use std::marker::PhantomData;

impl<T: 'static> QueryBuilder<T, Filtered> {
    /// Apply an additional filter to an already filtered query
    ///
    /// **実行種別**: 遅延ストリーミング
    #[inline]
    pub fn where_<F>(self, predicate: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.filter(predicate))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Sort elements in ascending order by a key
    ///
    /// **実行種別**: 遅延非ストリーミング（ソートのため全要素を収集）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![3, 1, 4, 1, 5];
    /// let result: Vec<_> = QueryBuilder::from(data)
    ///     .where_(|x| *x > 0)
    ///     .order_by(|x| *x)
    ///     .collect();
    /// assert_eq!(result, vec![1, 1, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn order_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
        T: 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut items: Vec<T> = iter.collect();
                items.sort_by_key(&key_selector);

                let comparator =
                    Box::new(move |a: &T, b: &T| key_selector(a).cmp(&key_selector(b)));

                QueryBuilder {
                    data: QueryData::SortedVec { items, comparator },
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Sort elements in descending order by a key
    ///
    /// **実行種別**: 遅延非ストリーミング（ソートのため全要素を収集）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![3, 1, 4, 1, 5];
    /// let result: Vec<_> = QueryBuilder::from(data)
    ///     .where_(|x| *x > 0)
    ///     .order_by_descending(|x| *x)
    ///     .collect();
    /// assert_eq!(result, vec![5, 4, 3, 1, 1]);
    /// ```
    #[inline]
    pub fn order_by_descending<K, F>(self, key_selector: F) -> QueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
        T: 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut items: Vec<T> = iter.collect();
                items.sort_by_key(|b| std::cmp::Reverse(key_selector(b)));

                let comparator =
                    Box::new(move |a: &T, b: &T| key_selector(b).cmp(&key_selector(a)));

                QueryBuilder {
                    data: QueryData::SortedVec { items, comparator },
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Transform elements to a different type
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .where_(|x| *x > 0)
    ///     .select(|x| x * 2)
    ///     .collect();
    /// assert_eq!(result, vec![2, 4, 6]);
    /// ```
    #[inline]
    pub fn select<U, F>(self, projection: F) -> QueryBuilder<U, Projected<U>>
    where
        F: Fn(T) -> U + 'static,
        U: 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.map(projection))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Take at most n elements
    #[inline]
    pub fn take(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.take(n))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Skip the first n elements
    #[inline]
    pub fn skip(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.skip(n))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Flatten nested iterables into a single sequence
    ///
    /// **実行種別**: 遅延ストリーミング
    #[inline]
    pub fn flat_map<U, I, F>(self, f: F) -> QueryBuilder<U, Filtered>
    where
        F: Fn(T) -> I + 'static,
        I: IntoIterator<Item = U> + 'static,
        U: 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.flat_map(f))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Take elements while the predicate holds
    ///
    /// **実行種別**: 遅延ストリーミング
    #[inline]
    pub fn take_while<F>(self, predicate: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.take_while(move |x| predicate(x)))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }

    /// Skip elements while the predicate holds
    ///
    /// **実行種別**: 遅延ストリーミング
    #[inline]
    pub fn skip_while<F>(self, predicate: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.skip_while(move |x| predicate(x)))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state cannot be SortedVec"),
        }
    }
}

// Second impl block for Filtered state: inspect, sum, average, min, max, min_by, max_by,
// group_by, group_by_aggregate, distinct, distinct_by, reverse, chunk, window, zip, enumerate, partition
impl<T: 'static> QueryBuilder<T, Filtered> {
    /// Inspect elements without consuming the query
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: Fn(&T) + 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => Self {
                data: QueryData::Iterator(Box::new(iter.inspect(f))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Calculate the sum of all filtered elements
    #[inline]
    pub fn sum(self) -> T
    where
        T: Sum,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.sum(),
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Calculate the average of all filtered elements
    #[inline]
    pub fn average(self) -> Option<f64>
    where
        T: ToPrimitive,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let items: Vec<T> = iter.collect();
                if items.is_empty() {
                    return None;
                }
                let sum: f64 = items.iter().filter_map(|x| x.to_f64()).sum();
                Some(sum / items.len() as f64)
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Find the minimum element
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.min(),
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Find the maximum element
    #[inline]
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.max(),
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Find the element with the minimum key value
    #[inline]
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.min_by_key(key_selector),
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Find the element with the maximum key value
    #[inline]
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.max_by_key(key_selector),
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Group elements by a key function
    ///
    /// **実行種別**: 即時実行
    #[inline]
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut groups: HashMap<K, Vec<T>> = HashMap::new();
                for item in iter {
                    let key = key_selector(&item);
                    groups.entry(key).or_default().push(item);
                }
                groups
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Group elements and apply an aggregation to each group
    #[inline]
    pub fn group_by_aggregate<K, R, FK, FA>(self, key_selector: FK, aggregator: FA) -> HashMap<K, R>
    where
        FK: Fn(&T) -> K,
        FA: Fn(&[T]) -> R,
        K: Eq + Hash,
    {
        let groups = self.group_by(key_selector);
        groups
            .into_iter()
            .map(|(k, v)| (k, aggregator(&v)))
            .collect()
    }

    /// Remove duplicate elements, preserving first occurrence
    ///
    /// **実行種別**: 遅延非ストリーミング
    ///
    /// Returns a `Filtered` query with duplicates removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 2, 3, 1, 4];
    /// let unique: Vec<i32> = QueryBuilder::from(data)
    ///     .distinct()
    ///     .collect();
    ///
    /// // Results in [1, 2, 3, 4] (first occurrence preserved)
    /// ```
    #[inline]
    pub fn distinct(self) -> QueryBuilder<T, Filtered>
    where
        T: HashEqBound + Clone,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut seen = HashSet::new();
                let filtered = iter.filter(move |item| {
                    if seen.contains(item) {
                        false
                    } else {
                        seen.insert(item.clone());
                        true
                    }
                });

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(filtered)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Remove duplicate elements based on a key selector
    ///
    /// Returns a `Filtered` query with duplicates removed based on the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Clone, Debug)]
    /// struct User { id: u32, name: String }
    ///
    /// let users = vec![
    ///     User { id: 1, name: "Alice".into() },
    ///     User { id: 2, name: "Bob".into() },
    ///     User { id: 3, name: "Alice".into() }, // Duplicate name
    /// ];
    ///
    /// let unique: Vec<User> = QueryBuilder::from(users)
    ///     .distinct_by(|u| u.name.clone())
    ///     .collect();
    ///
    /// assert_eq!(unique.len(), 2); // First Alice and Bob
    /// ```
    #[inline]
    pub fn distinct_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + Hash + 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut seen = HashSet::new();
                let filtered = iter.filter(move |item| {
                    let key = key_selector(item);
                    if seen.contains(&key) {
                        false
                    } else {
                        seen.insert(key);
                        true
                    }
                });

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(filtered)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Reverse the iteration order
    #[inline]
    pub fn reverse(self) -> Self {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut items: Vec<T> = iter.collect();
                items.reverse();

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter())),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Split elements into fixed-size chunks
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    #[inline]
    pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> {
        assert!(size > 0, "chunk size must be greater than 0");

        match self.data {
            QueryData::Iterator(iter) => {
                let chunk_iter = ChunkIterator {
                    inner: iter,
                    chunk_size: size,
                };

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(chunk_iter)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Create sliding windows of fixed size
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    #[inline]
    pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
    where
        T: Clone,
    {
        assert!(size > 0, "window size must be greater than 0");

        match self.data {
            QueryData::Iterator(iter) => {
                let window_iter = WindowIterator {
                    buffer: VecDeque::new(),
                    inner: iter,
                    window_size: size,
                    finished: false,
                };

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(window_iter)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Zip this filtered query with another iterable
    #[inline]
    pub fn zip<U, I>(self, other: I) -> QueryBuilder<(T, U), Filtered>
    where
        U: 'static,
        I: IntoIterator<Item = U> + 'static,
        I::IntoIter: 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let zipped = iter.zip(other);

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(zipped)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Add indices to filtered elements
    #[inline]
    pub fn enumerate(self) -> QueryBuilder<(usize, T), Filtered> {
        match self.data {
            QueryData::Iterator(iter) => {
                let enumerated = iter.enumerate();

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(enumerated)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }

    /// Partition filtered elements into two collections
    #[inline]
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut left = Vec::new();
                let mut right = Vec::new();

                for item in iter {
                    if predicate(&item) {
                        left.push(item);
                    } else {
                        right.push(item);
                    }
                }

                (left, right)
            }
            QueryData::SortedVec { .. } => unreachable!("Filtered state must be Iterator"),
        }
    }
}
