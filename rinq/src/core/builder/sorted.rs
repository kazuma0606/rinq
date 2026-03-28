// src/core/builder/sorted.rs
// impl QueryBuilder<T, Sorted>

use super::iterators::{ChunkIterator, WindowIterator};
use super::{QueryBuilder, QueryData};
use crate::core::state::{Filtered, Sorted};
use crate::core::state_diagnostics::HashEqBound;
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::iter::Sum;
use std::marker::PhantomData;

impl<T: 'static> QueryBuilder<T, Sorted> {
    /// Inspect elements without consuming the query
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// Note: This converts Sorted state to Filtered state for lazy evaluation
    #[inline]
    pub fn inspect<F>(self, f: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) + 'static,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                // Convert to iterator and apply inspect for lazy evaluation
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter().inspect(f))),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Apply a secondary sort key (stable sort preserving primary order)
    ///
    /// **実行種別**: 遅延非ストリーミング（再ソートのため全要素を収集）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct Person { name: String, age: i32 }
    ///
    /// let data = vec![
    ///     Person { name: "Alice".to_string(), age: 30 },
    ///     Person { name: "Bob".to_string(), age: 25 },
    ///     Person { name: "Charlie".to_string(), age: 30 },
    /// ];
    ///
    /// let result: Vec<_> = QueryBuilder::from(data)
    ///     .where_(|_| true)
    ///     .order_by(|p| p.age)
    ///     .then_by(|p| p.name.clone())
    ///     .collect();
    /// // Sorted first by age, then by name within same age
    /// ```
    #[inline]
    pub fn then_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
        T: 'static,
    {
        match self.data {
            QueryData::SortedVec {
                mut items,
                comparator,
            } => {
                let primary_comparator = comparator;
                let new_comparator = Box::new(move |a: &T, b: &T| match primary_comparator(a, b) {
                    Ordering::Equal => key_selector(a).cmp(&key_selector(b)),
                    other => other,
                });

                items.sort_by(|a, b| new_comparator(a, b));

                QueryBuilder {
                    data: QueryData::SortedVec {
                        items,
                        comparator: new_comparator,
                    },
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Apply a secondary sort key in descending order (stable sort preserving primary order)
    ///
    /// **実行種別**: 遅延非ストリーミング（再ソートのため全要素を収集）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct Item { category: &'static str, priority: i32 }
    ///
    /// let data = vec![
    ///     Item { category: "A", priority: 2 },
    ///     Item { category: "B", priority: 5 },
    ///     Item { category: "A", priority: 7 },
    ///     Item { category: "B", priority: 1 },
    /// ];
    ///
    /// // Sort by category ascending, then by priority descending within same category
    /// let result: Vec<_> = QueryBuilder::from(data)
    ///     .where_(|_| true)
    ///     .order_by(|item| item.category)
    ///     .then_by_descending(|item| item.priority)
    ///     .collect();
    ///
    /// assert_eq!(result[0].category, "A");
    /// assert_eq!(result[0].priority, 7); // higher priority first within "A"
    /// assert_eq!(result[2].category, "B");
    /// assert_eq!(result[2].priority, 5); // higher priority first within "B"
    /// ```
    #[inline]
    pub fn then_by_descending<K, F>(self, key_selector: F) -> QueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
        T: 'static,
    {
        match self.data {
            QueryData::SortedVec {
                mut items,
                comparator,
            } => {
                let primary_comparator = comparator;
                let new_comparator = Box::new(move |a: &T, b: &T| match primary_comparator(a, b) {
                    Ordering::Equal => key_selector(b).cmp(&key_selector(a)),
                    other => other,
                });

                items.sort_by(|a, b| new_comparator(a, b));

                QueryBuilder {
                    data: QueryData::SortedVec {
                        items,
                        comparator: new_comparator,
                    },
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Take at most n elements from sorted query
    #[inline]
    pub fn take(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::SortedVec { items, .. } => QueryBuilder {
                data: QueryData::Iterator(Box::new(items.into_iter().take(n))),
                _state: PhantomData,
            },
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Skip the first n elements from sorted query
    #[inline]
    pub fn skip(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::SortedVec { items, .. } => QueryBuilder {
                data: QueryData::Iterator(Box::new(items.into_iter().skip(n))),
                _state: PhantomData,
            },
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
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
            QueryData::SortedVec { items, .. } => QueryBuilder {
                data: QueryData::Iterator(Box::new(
                    items.into_iter().take_while(move |x| predicate(x)),
                )),
                _state: PhantomData,
            },
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
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
            QueryData::SortedVec { items, .. } => QueryBuilder {
                data: QueryData::Iterator(Box::new(
                    items.into_iter().skip_while(move |x| predicate(x)),
                )),
                _state: PhantomData,
            },
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Calculate the sum of all sorted elements
    #[inline]
    pub fn sum(self) -> T
    where
        T: Sum,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => items.into_iter().sum(),
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Calculate the average of all sorted elements
    #[inline]
    pub fn average(self) -> Option<f64>
    where
        T: ToPrimitive,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                if items.is_empty() {
                    return None;
                }
                let sum: f64 = items.iter().filter_map(|x| x.to_f64()).sum();
                Some(sum / items.len() as f64)
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Find the minimum element (O(1) for sorted data)
    ///
    /// Optimized: Returns the first element from sorted collection in O(1) time.
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::SortedVec { mut items, .. } => {
                // Optimization: First element is minimum in sorted collection
                if items.is_empty() {
                    None
                } else {
                    Some(items.remove(0))
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Find the maximum element (O(1) for sorted data)
    ///
    /// Optimized: Returns the last element from sorted collection in O(1) time.
    #[inline]
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::SortedVec { mut items, .. } => {
                // Optimization: Last element is maximum in sorted collection
                items.pop()
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
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
            QueryData::SortedVec { items, .. } => items.into_iter().min_by_key(key_selector),
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
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
            QueryData::SortedVec { items, .. } => items.into_iter().max_by_key(key_selector),
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Group sorted elements by a key function
    ///
    /// **実行種別**: 即時実行
    #[inline]
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                let mut groups: HashMap<K, Vec<T>> = HashMap::new();
                for item in items {
                    let key = key_selector(&item);
                    groups.entry(key).or_default().push(item);
                }
                groups
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Group sorted elements and apply an aggregation to each group
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

    /// Remove duplicate elements from sorted collection
    ///
    /// Converts to Filtered state with duplicates removed.
    #[inline]
    pub fn distinct(self) -> QueryBuilder<T, Filtered>
    where
        T: HashEqBound + Clone,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                let mut seen = HashSet::new();
                let filtered = items.into_iter().filter(move |item| {
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
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Remove duplicate elements based on a key selector from sorted collection
    #[inline]
    pub fn distinct_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + Hash + 'static,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                let mut seen = HashSet::new();
                let filtered = items.into_iter().filter(move |item| {
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
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Reverse the sorted iteration order
    #[inline]
    pub fn reverse(self) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::SortedVec { mut items, .. } => {
                items.reverse();

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter())),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Split sorted elements into fixed-size chunks
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    #[inline]
    pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> {
        assert!(size > 0, "chunk size must be greater than 0");

        match self.data {
            QueryData::SortedVec { items, .. } => {
                let chunk_iter = ChunkIterator {
                    inner: items.into_iter(),
                    chunk_size: size,
                };

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(chunk_iter)),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Create sliding windows over sorted elements
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
            QueryData::SortedVec { items, .. } => {
                let window_iter = WindowIterator {
                    buffer: VecDeque::new(),
                    inner: Box::new(items.into_iter()),
                    window_size: size,
                    finished: false,
                };

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(window_iter)),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Zip this sorted query with another iterable
    #[inline]
    pub fn zip<U, I>(self, other: I) -> QueryBuilder<(T, U), Filtered>
    where
        U: 'static,
        I: IntoIterator<Item = U> + 'static,
        I::IntoIter: 'static,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                let zipped = items.into_iter().zip(other);

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(zipped)),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Add indices to sorted elements
    #[inline]
    pub fn enumerate(self) -> QueryBuilder<(usize, T), Filtered> {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                let enumerated = items.into_iter().enumerate();

                QueryBuilder {
                    data: QueryData::Iterator(Box::new(enumerated)),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Partition sorted elements into two collections
    #[inline]
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                let mut left = Vec::new();
                let mut right = Vec::new();

                for item in items {
                    if predicate(&item) {
                        left.push(item);
                    } else {
                        right.push(item);
                    }
                }

                (left, right)
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }
}
