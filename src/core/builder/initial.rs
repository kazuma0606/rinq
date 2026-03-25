// src/core/builder/initial.rs
// impl QueryBuilder<T, Initial> — both blocks

use super::{QueryBuilder, QueryData};
use super::iterators::{ChunkIterator, WindowIterator};
use crate::core::state::{Filtered, Initial, Sorted};
use num_traits::ToPrimitive;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::iter::Sum;
use std::marker::PhantomData;

impl<T: 'static> QueryBuilder<T, Initial> {
    /// Create a new QueryBuilder from any iterable collection
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let query = QueryBuilder::from(data);
    /// ```
    #[inline]
    pub fn from<I>(source: I) -> Self
    where
        I: IntoIterator<Item = T> + 'static,
        I::IntoIter: 'static,
    {
        Self {
            data: QueryData::Iterator(Box::new(source.into_iter())),
            _state: PhantomData,
        }
    }

    /// Create a QueryBuilder from a range or any iterable that generates a sequence
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// Accepts any type implementing `IntoIterator`, including standard Rust ranges
    /// (`0..10`, `1..=100`, etc.).
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::range(1..=5).collect();
    /// assert_eq!(result, vec![1, 2, 3, 4, 5]);
    ///
    /// let squares: Vec<i32> = QueryBuilder::range(1..=5)
    ///     .where_(|_| true)
    ///     .select(|x| x * x)
    ///     .collect();
    /// assert_eq!(squares, vec![1, 4, 9, 16, 25]);
    /// ```
    #[inline]
    pub fn range<R>(range: R) -> QueryBuilder<T, Initial>
    where
        R: IntoIterator<Item = T> + 'static,
        R::IntoIter: 'static,
    {
        QueryBuilder::from(range)
    }

    /// Create a QueryBuilder that yields `value` repeated `count` times
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::repeat(0, 4).collect();
    /// assert_eq!(result, vec![0, 0, 0, 0]);
    /// ```
    #[inline]
    pub fn repeat(value: T, count: usize) -> QueryBuilder<T, Initial>
    where
        T: Clone,
    {
        QueryBuilder::from(std::iter::repeat_n(value, count))
    }

    /// Create a QueryBuilder that yields no elements
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::empty().collect();
    /// assert!(result.is_empty());
    /// ```
    #[inline]
    pub fn empty() -> QueryBuilder<T, Initial> {
        QueryBuilder::from(std::iter::empty::<T>())
    }

    /// Filter elements based on a predicate
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let result: Vec<_> = QueryBuilder::from(data)
    ///     .where_(|x| x % 2 == 0)
    ///     .collect();
    /// assert_eq!(result, vec![2, 4]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Sort elements in ascending order by a key
    ///
    /// **実行種別**: 遅延非ストリーミング（ソートのため全要素を収集）
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
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Sort elements in descending order by a key
    ///
    /// **実行種別**: 遅延非ストリーミング（ソートのため全要素を収集）
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
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Take at most n elements
    ///
    /// **実行種別**: 遅延ストリーミング
    #[inline]
    pub fn take(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.take(n))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Skip the first n elements
    ///
    /// **実行種別**: 遅延ストリーミング
    #[inline]
    pub fn skip(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::Iterator(iter) => QueryBuilder {
                data: QueryData::Iterator(Box::new(iter.skip(n))),
                _state: PhantomData,
            },
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Flatten nested iterables into a single sequence
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![vec![1, 2], vec![3, 4], vec![5]];
    /// let result: Vec<i32> = QueryBuilder::from(data)
    ///     .flat_map(|v| v)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3, 4, 5]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Take elements while the predicate holds, stopping at the first failure
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 1];
    /// let result: Vec<i32> = QueryBuilder::from(data)
    ///     .take_while(|x| *x < 4)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }

    /// Skip elements while the predicate holds, yielding from the first failure onward
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let result: Vec<i32> = QueryBuilder::from(data)
    ///     .skip_while(|x| *x < 3)
    ///     .collect();
    /// assert_eq!(result, vec![3, 4, 5]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state cannot be SortedVec"),
        }
    }
}

// Second impl block for Initial state: inspect, sum, average, min, max, min_by, max_by,
// group_by, group_by_aggregate, distinct, distinct_by, reverse, chunk, window, zip, enumerate, partition
impl<T: 'static> QueryBuilder<T, Initial> {
    /// Inspect elements without consuming the query
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<_> = QueryBuilder::from(vec![1, 2, 3])
    ///     .inspect(|x| { let _ = x; })  // no-op side effect for illustration
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Calculate the sum of all elements
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let total: i32 = QueryBuilder::from(data).sum();
    /// assert_eq!(total, 15);
    /// ```
    #[inline]
    pub fn sum(self) -> T
    where
        T: Sum,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.sum(),
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Calculate the average of all elements
    ///
    /// Returns `None` for empty collections, `Some(average)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let avg = QueryBuilder::from(data).average().unwrap();
    /// assert!((avg - 3.0).abs() < 1e-10);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Find the minimum element
    ///
    /// Returns `None` for empty collections, `Some(min_element)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![5, 2, 8, 1, 9];
    /// let min = QueryBuilder::from(data).min();
    /// assert_eq!(min, Some(1));
    /// ```
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.min(),
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Find the maximum element
    ///
    /// Returns `None` for empty collections, `Some(max_element)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![5, 2, 8, 1, 9];
    /// let max = QueryBuilder::from(data).max();
    /// assert_eq!(max, Some(9));
    /// ```
    #[inline]
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.max(),
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Find the element with the minimum key value
    ///
    /// Returns `None` for empty collections, `Some(element)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct User { name: String, age: u32 }
    ///
    /// let users = vec![
    ///     User { name: "Alice".into(), age: 30 },
    ///     User { name: "Bob".into(), age: 25 },
    ///     User { name: "Charlie".into(), age: 35 },
    /// ];
    ///
    /// let youngest = QueryBuilder::from(users).min_by(|u| u.age).unwrap();
    /// assert_eq!(youngest.name, "Bob");
    /// ```
    #[inline]
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.min_by_key(key_selector),
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Find the element with the maximum key value
    ///
    /// Returns `None` for empty collections, `Some(element)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct User { name: String, age: u32 }
    ///
    /// let users = vec![
    ///     User { name: "Alice".into(), age: 30 },
    ///     User { name: "Bob".into(), age: 25 },
    ///     User { name: "Charlie".into(), age: 35 },
    /// ];
    ///
    /// let oldest = QueryBuilder::from(users).max_by(|u| u.age).unwrap();
    /// assert_eq!(oldest.name, "Charlie");
    /// ```
    #[inline]
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.max_by_key(key_selector),
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Group elements by a key function
    ///
    /// **実行種別**: 即時実行
    ///
    /// Returns a `HashMap` where keys are the result of applying the key function,
    /// and values are vectors of elements with that key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use std::collections::HashMap;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6];
    /// let groups: HashMap<i32, Vec<i32>> = QueryBuilder::from(data)
    ///     .group_by(|x| x % 2);
    ///
    /// assert_eq!(groups.get(&0).unwrap(), &vec![2, 4, 6]);
    /// assert_eq!(groups.get(&1).unwrap(), &vec![1, 3, 5]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Group elements by a key function and apply an aggregation to each group
    ///
    /// **実行種別**: 即時実行
    ///
    /// Returns a `HashMap` where keys are the result of applying the key function,
    /// and values are the result of applying the aggregation function to each group.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use std::collections::HashMap;
    ///
    /// #[derive(Clone)]
    /// struct Order { user_id: u32, amount: f64 }
    ///
    /// let orders = vec![
    ///     Order { user_id: 1, amount: 100.0 },
    ///     Order { user_id: 2, amount: 50.0 },
    ///     Order { user_id: 1, amount: 75.0 },
    /// ];
    ///
    /// let totals: HashMap<u32, f64> = QueryBuilder::from(orders)
    ///     .group_by_aggregate(
    ///         |o| o.user_id,
    ///         |group| group.iter().map(|o| o.amount).sum()
    ///     );
    ///
    /// assert_eq!(totals.get(&1), Some(&175.0));
    /// ```
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
        T: Eq + Hash + Clone,
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
    /// **実行種別**: 遅延非ストリーミング
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
    ///
    /// **実行種別**: 遅延非ストリーミング（全要素収集）
    ///
    /// Returns a `Filtered` query with elements in reverse order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let reversed: Vec<i32> = QueryBuilder::from(data)
    ///     .reverse()
    ///     .collect();
    ///
    /// assert_eq!(reversed, vec![5, 4, 3, 2, 1]);
    /// ```
    #[inline]
    pub fn reverse(self) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut items: Vec<T> = iter.collect();
                items.reverse();

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter())),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Split elements into fixed-size chunks
    ///
    /// Returns a `Filtered` query of `Vec<T>` chunks.
    /// The last chunk may contain fewer elements if the collection size
    /// is not evenly divisible.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let chunks: Vec<Vec<i32>> = QueryBuilder::from(data)
    ///     .chunk(2)
    ///     .collect();
    ///
    /// assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Create sliding windows of fixed size
    ///
    /// Returns a `Filtered` query of `Vec<T>` windows.
    /// Each window overlaps with the previous and next windows.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let windows: Vec<Vec<i32>> = QueryBuilder::from(data)
    ///     .window(3)
    ///     .collect();
    ///
    /// assert_eq!(windows, vec![
    ///     vec![1, 2, 3],
    ///     vec![2, 3, 4],
    ///     vec![3, 4, 5]
    /// ]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Zip this query with another iterable, creating pairs
    ///
    /// Returns a `Filtered` query of `(T, U)` tuples.
    /// Stops when either iterator is exhausted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let numbers = vec![1, 2, 3];
    /// let letters = vec!["a", "b", "c"];
    /// let pairs: Vec<(i32, &str)> = QueryBuilder::from(numbers)
    ///     .zip(letters)
    ///     .collect();
    ///
    /// assert_eq!(pairs, vec![(1, "a"), (2, "b"), (3, "c")]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Add indices to elements, creating (index, element) pairs
    ///
    /// Returns a `Filtered` query of `(usize, T)` tuples.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec!["a", "b", "c"];
    /// let indexed: Vec<(usize, &str)> = QueryBuilder::from(data)
    ///     .enumerate()
    ///     .collect();
    ///
    /// assert_eq!(indexed, vec![(0, "a"), (1, "b"), (2, "c")]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Partition elements into two collections based on a predicate
    ///
    /// Returns a tuple of `(Vec<T>, Vec<T>)` where the first contains
    /// elements satisfying the predicate, and the second contains the rest.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6];
    /// let (evens, odds) = QueryBuilder::from(data)
    ///     .partition(|x| *x % 2 == 0);
    ///
    /// assert_eq!(evens, vec![2, 4, 6]);
    /// assert_eq!(odds, vec![1, 3, 5]);
    /// ```
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }
}
