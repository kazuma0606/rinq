// src/core/builder/shared.rs
// impl<T, State> QueryBuilder<T, State> — terminal operations available in all states

use super::iterators::ChunkIterator;
use super::{QueryBuilder, QueryData};
use crate::core::error::{RinqError, RinqResult};
use crate::core::state::Filtered;
use crate::core::state_diagnostics::HashEqBound;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
#[cfg(feature = "parallel")]
use crate::parallel::ParallelQueryBuilder;

// Terminal operations available in all states
impl<T: 'static, State> QueryBuilder<T, State> {
    /// Collect the results into a collection
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3];
    /// let result: Vec<_> = QueryBuilder::from(data).collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn collect<B>(self) -> B
    where
        B: FromIterator<T>,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.collect(),
            QueryData::SortedVec { items, .. } => items.into_iter().collect(),
        }
    }

    /// Materialize all items into a `Vec<T>` (internal helper).
    #[inline]
    pub(crate) fn into_vec(self) -> Vec<T> {
        match self.data {
            QueryData::Iterator(iter) => iter.collect(),
            QueryData::SortedVec { items, .. } => items,
        }
    }

    /// Count the number of elements
    ///
    /// **実行種別**: 即時実行
    #[inline]
    pub fn count(self) -> usize {
        match self.data {
            QueryData::Iterator(iter) => iter.count(),
            QueryData::SortedVec { items, .. } => items.len(),
        }
    }

    /// Get the first element, if any
    ///
    /// **実行種別**: 即時実行
    #[inline]
    pub fn first(self) -> Option<T> {
        match self.data {
            QueryData::Iterator(mut iter) => iter.next(),
            QueryData::SortedVec { mut items, .. } => {
                if items.is_empty() {
                    None
                } else {
                    Some(items.remove(0))
                }
            }
        }
    }

    /// Get the last element, if any
    ///
    /// **実行種別**: 即時実行
    #[inline]
    pub fn last(self) -> Option<T> {
        match self.data {
            QueryData::Iterator(iter) => iter.last(),
            QueryData::SortedVec { mut items, .. } => items.pop(),
        }
    }

    /// Check if any element satisfies the predicate
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// assert!(QueryBuilder::from(vec![1, 2, 3]).any(|x| *x > 2));
    /// assert!(!QueryBuilder::from(vec![1, 2, 3]).any(|x| *x > 10));
    /// assert!(!QueryBuilder::from(Vec::<i32>::new()).any(|x| *x > 0));
    /// ```
    #[inline]
    pub fn any<F>(self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.any(|item| predicate(&item)),
            QueryData::SortedVec { items, .. } => items.iter().any(predicate),
        }
    }

    /// Check if all elements satisfy the predicate
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// assert!(QueryBuilder::from(vec![2, 4, 6]).all(|x| *x % 2 == 0));
    /// assert!(!QueryBuilder::from(vec![2, 3, 6]).all(|x| *x % 2 == 0));
    /// assert!(QueryBuilder::from(Vec::<i32>::new()).all(|x| *x > 0)); // vacuously true
    /// ```
    #[inline]
    pub fn all<F>(self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.all(|item| predicate(&item)),
            QueryData::SortedVec { items, .. } => items.iter().all(predicate),
        }
    }

    /// Check if the collection contains a specific value
    ///
    /// **実行種別**: 即時実行（線形探索）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// assert!(QueryBuilder::from(data).contains(&3));
    /// ```
    #[inline]
    pub fn contains(self, value: &T) -> bool
    where
        T: PartialEq,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.any(|item| item == *value),
            QueryData::SortedVec { items, .. } => items.contains(value),
        }
    }

    /// Get the first element, or `T::default()` if empty
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data: Vec<i32> = vec![];
    /// assert_eq!(QueryBuilder::from(data).first_or_default(), 0);
    ///
    /// let data = vec![1, 2, 3];
    /// assert_eq!(QueryBuilder::from(data).first_or_default(), 1);
    /// ```
    #[inline]
    pub fn first_or_default(self) -> T
    where
        T: Default,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.next().unwrap_or_default(),
            QueryData::SortedVec { mut items, .. } => {
                if items.is_empty() {
                    T::default()
                } else {
                    items.remove(0)
                }
            }
        }
    }

    /// Get the last element, or `T::default()` if empty
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data: Vec<i32> = vec![];
    /// assert_eq!(QueryBuilder::from(data).last_or_default(), 0);
    ///
    /// let data = vec![1, 2, 3];
    /// assert_eq!(QueryBuilder::from(data).last_or_default(), 3);
    /// ```
    #[inline]
    pub fn last_or_default(self) -> T
    where
        T: Default,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.last().unwrap_or_default(),
            QueryData::SortedVec { mut items, .. } => items.pop().unwrap_or_default(),
        }
    }

    /// Return the only element in the collection
    ///
    /// **実行種別**: 即時実行
    ///
    /// - 0件 → `Err(RinqError::IteratorExhausted)`
    /// - 1件 → `Ok(element)`
    /// - 2件以上 → `Err(RinqError::ExecutionError)`
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// assert_eq!(QueryBuilder::from(vec![42]).single(), Ok(42));
    /// assert!(QueryBuilder::from(vec![1, 2]).single().is_err());
    /// assert!(QueryBuilder::from(Vec::<i32>::new()).single().is_err());
    /// ```
    #[inline]
    pub fn single(self) -> RinqResult<T> {
        match self.data {
            QueryData::Iterator(mut iter) => match iter.next() {
                None => Err(RinqError::IteratorExhausted),
                Some(first) => {
                    if iter.next().is_some() {
                        Err(RinqError::ExecutionError {
                            message: "single() called on a collection with more than one element"
                                .to_string(),
                        })
                    } else {
                        Ok(first)
                    }
                }
            },
            QueryData::SortedVec { items, .. } => match items.len() {
                0 => Err(RinqError::IteratorExhausted),
                1 => Ok(items.into_iter().next().unwrap()),
                _ => Err(RinqError::ExecutionError {
                    message: "single() called on a collection with more than one element"
                        .to_string(),
                }),
            },
        }
    }

    /// Return the only element, or `T::default()` if empty
    ///
    /// **実行種別**: 即時実行
    ///
    /// - 0件 → `Ok(T::default())`
    /// - 1件 → `Ok(element)`
    /// - 2件以上 → `Err(RinqError::ExecutionError)`
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// assert_eq!(QueryBuilder::from(Vec::<i32>::new()).single_or_default(), Ok(0));
    /// assert_eq!(QueryBuilder::from(vec![42]).single_or_default(), Ok(42));
    /// assert!(QueryBuilder::from(vec![1, 2]).single_or_default().is_err());
    /// ```
    #[inline]
    pub fn single_or_default(self) -> RinqResult<T>
    where
        T: Default,
    {
        match self.data {
            QueryData::Iterator(mut iter) => match iter.next() {
                None => Ok(T::default()),
                Some(first) => {
                    if iter.next().is_some() {
                        Err(RinqError::ExecutionError {
                            message:
                                "single_or_default() called on a collection with more than one element"
                                    .to_string(),
                        })
                    } else {
                        Ok(first)
                    }
                }
            },
            QueryData::SortedVec { items, .. } => match items.len() {
                0 => Ok(T::default()),
                1 => Ok(items.into_iter().next().unwrap()),
                _ => Err(RinqError::ExecutionError {
                    message:
                        "single_or_default() called on a collection with more than one element"
                            .to_string(),
                }),
            },
        }
    }

    /// Get the element at the specified index, or `None` if out of bounds
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![10, 20, 30, 40];
    /// assert_eq!(QueryBuilder::from(data.clone()).element_at(2), Some(30));
    /// assert_eq!(QueryBuilder::from(data).element_at(10), None);
    /// ```
    #[inline]
    pub fn element_at(self, index: usize) -> Option<T> {
        match self.data {
            QueryData::Iterator(mut iter) => iter.nth(index),
            QueryData::SortedVec { items, .. } => items.into_iter().nth(index),
        }
    }

    /// Fold all elements into an accumulator using a seed value
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let product = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .aggregate(1, |acc, x| acc * x);
    /// assert_eq!(product, 120);
    /// ```
    #[inline]
    pub fn aggregate<Acc, F>(self, seed: Acc, f: F) -> Acc
    where
        F: Fn(Acc, T) -> Acc,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.fold(seed, f),
            QueryData::SortedVec { items, .. } => items.into_iter().fold(seed, f),
        }
    }

    /// Fold all elements without a seed value, returning `None` if empty
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let max = QueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .aggregate_no_seed(|a, b| if a > b { a } else { b });
    /// assert_eq!(max, Some(5));
    ///
    /// let empty = QueryBuilder::from(Vec::<i32>::new())
    ///     .aggregate_no_seed(|a, b| a + b);
    /// assert_eq!(empty, None);
    /// ```
    #[inline]
    pub fn aggregate_no_seed<F>(self, f: F) -> Option<T>
    where
        F: Fn(T, T) -> T,
    {
        let mut iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let first = iter.next()?;
        Some(iter.fold(first, f))
    }

    /// Concatenate another sequence to the end of this query
    ///
    /// **実行種別**: 遅延ストリーミング
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .concat(vec![4, 5, 6])
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    pub fn concat(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered> {
        let chained: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => Box::new(iter.chain(other)),
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter().chain(other)),
        };
        QueryBuilder {
            data: QueryData::Iterator(chained),
            _state: PhantomData,
        }
    }

    /// Produce the set union with another sequence, removing duplicates
    ///
    /// **実行種別**: 遅延非ストリーミング（内部 `HashSet` を使用）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let mut result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .union(vec![2, 3, 4, 5])
    ///     .collect();
    /// result.sort();
    /// assert_eq!(result, vec![1, 2, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn union(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered>
    where
        T: HashEqBound + Clone,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let mut seen = HashSet::new();
        let combined = iter.chain(other).filter(move |item| {
            if seen.contains(item) {
                false
            } else {
                seen.insert(item.clone());
                true
            }
        });
        QueryBuilder {
            data: QueryData::Iterator(Box::new(combined)),
            _state: PhantomData,
        }
    }

    /// Produce the set intersection with another sequence
    ///
    /// **実行種別**: 遅延非ストリーミング（内部 `HashSet` を使用）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let mut result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4])
    ///     .intersect(vec![2, 4, 6])
    ///     .collect();
    /// result.sort();
    /// assert_eq!(result, vec![2, 4]);
    /// ```
    #[inline]
    pub fn intersect(
        self,
        other: impl IntoIterator<Item = T> + 'static,
    ) -> QueryBuilder<T, Filtered>
    where
        T: HashEqBound + Clone,
    {
        let other_set: HashSet<T> = other.into_iter().collect();
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let mut seen = HashSet::new();
        let filtered = iter.filter(move |item| {
            other_set.contains(item) && seen.insert(item.clone())
        });
        QueryBuilder {
            data: QueryData::Iterator(Box::new(filtered)),
            _state: PhantomData,
        }
    }

    /// Produce the set difference: elements in `self` but not in `other`
    ///
    /// **実行種別**: 遅延非ストリーミング（内部 `HashSet` を使用）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let mut result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .except(vec![2, 4])
    ///     .collect();
    /// result.sort();
    /// assert_eq!(result, vec![1, 3, 5]);
    /// ```
    #[inline]
    pub fn except(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered>
    where
        T: HashEqBound + Clone,
    {
        let other_set: HashSet<T> = other.into_iter().collect();
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        let mut seen = HashSet::new();
        let filtered = iter.filter(move |item| {
            !other_set.contains(item) && seen.insert(item.clone())
        });
        QueryBuilder {
            data: QueryData::Iterator(Box::new(filtered)),
            _state: PhantomData,
        }
    }

    /// Collect elements into a `HashMap` keyed by a selector
    ///
    /// Returns `Err` if duplicate keys are detected.
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![(1, "a"), (2, "b"), (3, "c")];
    /// let map = QueryBuilder::from(data)
    ///     .to_hashmap(|(k, _)| *k)
    ///     .unwrap();
    /// assert_eq!(map[&1], (1, "a"));
    /// ```
    #[inline]
    pub fn to_hashmap<K, F>(self, key_selector: F) -> RinqResult<HashMap<K, T>>
    where
        K: Hash + Eq,
        F: Fn(&T) -> K,
    {
        let mut map = HashMap::new();
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        for item in iter {
            let key = key_selector(&item);
            if map.contains_key(&key) {
                return Err(RinqError::ExecutionError {
                    message: "to_hashmap() found a duplicate key".to_string(),
                });
            }
            map.insert(key, item);
        }
        Ok(map)
    }

    /// Collect elements into a `HashMap<K, Vec<T>>`, grouping by key
    ///
    /// Unlike `to_hashmap`, duplicate keys are allowed — values are accumulated into a `Vec`.
    ///
    /// **実行種別**: 即時実行
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let data = vec![("a", 1), ("b", 2), ("a", 3)];
    /// let lookup = QueryBuilder::from(data).to_lookup(|(k, _)| *k);
    /// assert_eq!(lookup[&"a"], vec![("a", 1), ("a", 3)]);
    /// assert_eq!(lookup[&"b"], vec![("b", 2)]);
    /// ```
    #[inline]
    pub fn to_lookup<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        K: Hash + Eq,
        F: Fn(&T) -> K,
    {
        let mut map: HashMap<K, Vec<T>> = HashMap::new();
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(iter) => iter,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        for item in iter {
            let key = key_selector(&item);
            map.entry(key).or_default().push(item);
        }
        map
    }
}

// ── Phase 4C: quick-win operators ───────────────────────────────────────────

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Filter and transform elements in one step, discarding `None` results.
    ///
    /// Equivalent to `.where_(|x| f(x).is_some()).select(|x| f(x).unwrap())`
    /// but more efficient because `f` is called only once per element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let strings = vec!["1", "two", "3", "four", "5"];
    /// let numbers: Vec<i32> = QueryBuilder::from(strings)
    ///     .filter_map(|s| s.parse::<i32>().ok())
    ///     .collect();
    /// assert_eq!(numbers, vec![1, 3, 5]);
    /// ```
    pub fn filter_map<U, F>(self, f: F) -> QueryBuilder<U, Filtered>
    where
        F: Fn(T) -> Option<U> + 'static,
        U: 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(iter.filter_map(f))),
            _state: PhantomData,
        }
    }

    /// Collect all elements into a `Vec<T>`.
    ///
    /// Shorthand for `.collect::<Vec<T>>()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let v: Vec<i32> = QueryBuilder::from(vec![1, 2, 3]).collect_vec();
    /// assert_eq!(v, vec![1, 2, 3]);
    /// ```
    pub fn collect_vec(self) -> Vec<T> {
        self.collect::<Vec<T>>()
    }

    /// Return every `step`-th element, starting from the first.
    ///
    /// # Panics
    ///
    /// Panics if `step` is 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// // Downsample sensor readings — keep every 2nd value.
    /// let readings = vec![10, 20, 30, 40, 50];
    /// let sampled: Vec<i32> = QueryBuilder::from(readings)
    ///     .step_by(2)
    ///     .collect();
    /// assert_eq!(sampled, vec![10, 30, 50]);
    /// ```
    pub fn step_by(self, step: usize) -> QueryBuilder<T, Filtered> {
        assert!(step > 0, "step_by: step must be greater than 0");
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(iter.step_by(step))),
            _state: PhantomData,
        }
    }

    /// Repeat the sequence indefinitely.
    ///
    /// # Infinite loop
    ///
    /// `cycle` produces an infinite iterator.  Always pair it with `take` (or
    /// another terminating operation) to avoid an infinite loop.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .cycle()
    ///     .take(7)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3, 1, 2, 3, 1]);
    /// ```
    pub fn cycle(self) -> QueryBuilder<T, Filtered>
    where
        T: Clone,
    {
        let items: Vec<T> = match self.data {
            QueryData::Iterator(it) => it.collect(),
            QueryData::SortedVec { items, .. } => items,
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(CycleIter::new(items))),
            _state: PhantomData,
        }
    }
}

/// Iterator adapter that cycles through a `Vec<T>` indefinitely.
struct CycleIter<T> {
    items: Vec<T>,
    index: usize,
}

impl<T> CycleIter<T> {
    fn new(items: Vec<T>) -> Self {
        Self { items, index: 0 }
    }
}

impl<T: Clone> Iterator for CycleIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }
        let item = self.items[self.index].clone();
        self.index = (self.index + 1) % self.items.len();
        Some(item)
    }
}

// `map` alias on Filtered state
impl<T: 'static> QueryBuilder<T, crate::core::state::Filtered> {
    /// Alias for [`select`](QueryBuilder::select).
    ///
    /// Provided for users familiar with iterator-style APIs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let doubled: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .where_(|_| true)
    ///     .map(|x| x * 2)
    ///     .collect();
    /// assert_eq!(doubled, vec![2, 4, 6]);
    /// ```
    #[inline]
    pub fn map<U, F>(self, projection: F) -> QueryBuilder<U, crate::core::state::Projected<U>>
    where
        F: Fn(T) -> U + 'static,
        U: 'static,
    {
        self.select(projection)
    }
}

// `IntoQuery` trait

/// Conversion trait for building a [`QueryBuilder`] directly from a collection.
///
/// Implement this trait (or use the provided blanket impl for `Vec<T>`) to
/// write `collection.into_query()` instead of `QueryBuilder::from(collection)`.
///
/// # Examples
///
/// ```rust
/// use rinq::IntoQuery;
///
/// let result: Vec<i32> = vec![1, 2, 3, 4, 5]
///     .into_query()
///     .where_(|x| x % 2 == 0)
///     .collect();
/// assert_eq!(result, vec![2, 4]);
/// ```
pub trait IntoQuery: Sized {
    /// The element type of the resulting query.
    type Item: 'static;

    /// Convert `self` into a [`QueryBuilder`] in the `Initial` state.
    fn into_query(self) -> QueryBuilder<Self::Item, crate::core::state::Initial>;
}

impl<T: 'static> IntoQuery for Vec<T> {
    type Item = T;

    fn into_query(self) -> QueryBuilder<T, crate::core::state::Initial> {
        QueryBuilder::from(self)
    }
}

// ── Phase 4B: lifecycle helpers ─────────────────────────────────────────────

impl<T: Clone + 'static> QueryBuilder<T, crate::core::state::Initial> {
    /// Create a query by cloning every element out of an `Arc<Vec<T>>`.
    ///
    /// This copies all N elements eagerly (O(N)), so that the resulting query
    /// owns its data independently from the `Arc`.  The `Arc` itself is not
    /// held by the returned query.
    ///
    /// # O(N) copy
    ///
    /// Every call clones all elements.  If you need multiple independent
    /// queries over the same data, prefer calling this once and reusing the
    /// resulting `Vec`, or use rayon's parallel iterator directly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::{QueryBuilder, FilteredQuery};
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct User { age: u32 }
    ///
    /// let shared: Arc<Vec<User>> = Arc::new(vec![
    ///     User { age: 10 },
    ///     User { age: 25 },
    ///     User { age: 30 },
    /// ]);
    ///
    /// let adults: FilteredQuery<User> = QueryBuilder::from_arc_cloned(&shared)
    ///     .where_(|u| u.age >= 18);
    ///
    /// let result: Vec<User> = adults.collect();
    /// assert_eq!(result.len(), 2);
    /// ```
    pub fn from_arc_cloned(arc: &std::sync::Arc<Vec<T>>) -> Self {
        let cloned: Vec<T> = arc.as_ref().clone();
        Self {
            data: QueryData::Iterator(Box::new(cloned.into_iter())),
            _state: PhantomData,
        }
    }

    /// Create a query by cloning every element out of an `Arc<[T]>`.
    ///
    /// Behaves identically to [`from_arc_cloned`](Self::from_arc_cloned) but
    /// accepts a slice-based Arc, which avoids the double indirection of
    /// `Arc<Vec<T>>`.
    ///
    /// # O(N) copy
    ///
    /// Every call clones all elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    /// use std::sync::Arc;
    ///
    /// let data: Arc<[i32]> = Arc::from(vec![1, 2, 3, 4, 5]);
    /// let result: Vec<i32> = QueryBuilder::from_arc_slice_cloned(&data)
    ///     .where_(|x| x % 2 == 0)
    ///     .collect();
    /// assert_eq!(result, vec![2, 4]);
    /// ```
    pub fn from_arc_slice_cloned(arc: &std::sync::Arc<[T]>) -> Self {
        let cloned: Vec<T> = arc.iter().cloned().collect();
        Self {
            data: QueryData::Iterator(Box::new(cloned.into_iter())),
            _state: PhantomData,
        }
    }
}

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Inspect each element without consuming the query (lazy).
    ///
    /// `tap_each` is a thin wrapper around `inspect` that makes the intent
    /// explicit: *observe* elements (e.g. for logging) without transforming
    /// them.  The closure is **not** called until a terminal operation drives
    /// the pipeline.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .tap_each(|x| { /* e.g. log::debug!("{}", x) */ let _ = x; })
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    pub fn tap_each<F>(self, f: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) + 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(iter.inspect(f))),
            _state: PhantomData,
        }
    }

    /// Eagerly collect all elements, call a side-effect closure, then re-wrap
    /// as a new `FilteredQuery`.
    ///
    /// ⚠ Eagerly collects all elements.  Unlike `tap_each`, the side-effect
    /// runs *before* any further chaining; the full collection is materialised
    /// at the point of the call.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use std::sync::Arc;
    ///
    /// let counter = Arc::new(AtomicUsize::new(0));
    /// let c2 = counter.clone();
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .tap_collect(move |items| {
    ///         c2.store(items.len(), Ordering::SeqCst);
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(counter.load(Ordering::SeqCst), 3);
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    pub fn tap_collect<F>(self, f: F) -> QueryBuilder<T, Filtered>
    where
        F: FnOnce(&[T]),
    {
        let items: Vec<T> = match self.data {
            QueryData::Iterator(it) => it.collect(),
            QueryData::SortedVec { items, .. } => items,
        };
        f(&items);
        QueryBuilder {
            data: QueryData::Iterator(Box::new(items.into_iter())),
            _state: PhantomData,
        }
    }

    /// Pass `self` through a transformation function, enabling dynamic
    /// pipeline construction.
    ///
    /// `pipe` lets you apply a function `FnOnce(Self) -> QueryBuilder<T2, S2>`
    /// inline, which is useful for:
    ///
    /// * Conditional filters (`if only_active { q.pipe(...) } else { q.pipe(...) }`)
    /// * Dynamic sort keys chosen at runtime
    /// * Delegating to an external `apply_tenant_filter` helper
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::{QueryBuilder, FilteredQuery};
    ///
    /// fn apply_min_age(q: QueryBuilder<u32, rinq::Initial>, min: u32) -> FilteredQuery<u32> {
    ///     q.where_(move |&x| x >= min)
    /// }
    ///
    /// let result: Vec<u32> = QueryBuilder::from(vec![5u32, 10, 15, 20])
    ///     .pipe(|q| apply_min_age(q, 12))
    ///     .collect();
    /// assert_eq!(result, vec![15, 20]);
    /// ```
    pub fn pipe<T2, S2, F>(self, f: F) -> QueryBuilder<T2, S2>
    where
        F: FnOnce(Self) -> QueryBuilder<T2, S2>,
        T2: 'static,
    {
        f(self)
    }
}

// ── Phase 5D: terminal operator enhancements ────────────────────────────────

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Apply a side-effect closure to every element, consuming the query.
    ///
    /// Unlike `tap_each`, `for_each` is a **terminal** operation — it drives the
    /// pipeline to completion and does not return a new `QueryBuilder`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let mut sum = 0;
    /// QueryBuilder::from(vec![1, 2, 3]).for_each(|x| sum += x);
    /// assert_eq!(sum, 6);
    /// ```
    pub fn for_each<F>(self, f: F)
    where
        F: FnMut(T),
    {
        match self.data {
            QueryData::Iterator(iter) => iter.for_each(f),
            QueryData::SortedVec { items, .. } => items.into_iter().for_each(f),
        }
    }

    /// Sort all elements by `key_selector` (ascending) and collect into a `Vec<T>`.
    ///
    /// Shorthand for `.order_by(key).collect::<Vec<_>>()`.
    ///
    /// ⚠ Eagerly collects all elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result = QueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .to_sorted_vec(|x| *x);
    /// assert_eq!(result, vec![1, 1, 3, 4, 5]);
    /// ```
    pub fn to_sorted_vec<K, F>(self, key_selector: F) -> Vec<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let mut items = self.into_vec();
        items.sort_by_key(key_selector);
        items
    }

    /// Sort all elements by `key_selector` (descending) and collect into a `Vec<T>`.
    ///
    /// Shorthand for `.order_by_descending(key).collect::<Vec<_>>()`.
    ///
    /// ⚠ Eagerly collects all elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result = QueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .to_sorted_vec_desc(|x| *x);
    /// assert_eq!(result, vec![5, 4, 3, 1, 1]);
    /// ```
    pub fn to_sorted_vec_desc<K, F>(self, key_selector: F) -> Vec<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let mut items = self.into_vec();
        items.sort_by_key(|x| std::cmp::Reverse(key_selector(x)));
        items
    }

    /// Return the last `n` elements as a `Vec<T>`.
    ///
    /// If the collection has fewer than `n` elements, all elements are returned.
    /// If `n == 0`, an empty `Vec` is returned.
    ///
    /// ⚠ Eagerly collects all elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result = QueryBuilder::from(vec![1, 2, 3, 4, 5]).take_last(3);
    /// assert_eq!(result, vec![3, 4, 5]);
    ///
    /// // Fewer elements than n
    /// let result = QueryBuilder::from(vec![1, 2]).take_last(5);
    /// assert_eq!(result, vec![1, 2]);
    /// ```
    pub fn take_last(self, n: usize) -> Vec<T> {
        let mut items = self.into_vec();
        if n >= items.len() {
            items
        } else {
            items.split_off(items.len() - n)
        }
    }

    /// Return all elements except the last `n`, as a `Vec<T>`.
    ///
    /// If the collection has `n` or fewer elements, an empty `Vec` is returned.
    /// If `n == 0`, all elements are returned.
    ///
    /// ⚠ Eagerly collects all elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let result = QueryBuilder::from(vec![1_i32, 2, 3, 4, 5]).skip_last(2);
    /// assert_eq!(result, vec![1, 2, 3]);
    ///
    /// // More elements skipped than exist
    /// let result = QueryBuilder::from(vec![1_i32, 2]).skip_last(5);
    /// assert_eq!(result, Vec::<i32>::new());
    /// ```
    pub fn skip_last(self, n: usize) -> Vec<T> {
        let mut items = self.into_vec();
        if n >= items.len() {
            items.clear();
        } else {
            items.truncate(items.len() - n);
        }
        items
    }

    /// Count elements matching a predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let count = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .count_by(|x| x % 2 == 0);
    /// assert_eq!(count, 2);
    /// ```
    pub fn count_by<F>(self, predicate: F) -> usize
    where
        F: Fn(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.filter(|x| predicate(x)).count(),
            QueryData::SortedVec { items, .. } => items.iter().filter(|x| predicate(x)).count(),
        }
    }

    /// Sum a numeric field extracted from each element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Clone)]
    /// struct Order { amount: i32 }
    ///
    /// let total = QueryBuilder::from(vec![
    ///     Order { amount: 10 },
    ///     Order { amount: 25 },
    ///     Order { amount: 15 },
    /// ])
    /// .sum_by(|o| o.amount);
    /// assert_eq!(total, 50_i32);
    /// ```
    pub fn sum_by<N, F>(self, key: F) -> N
    where
        F: Fn(T) -> N,
        N: Default + std::ops::Add<Output = N>,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.map(key).fold(N::default(), |a, b| a + b),
            QueryData::SortedVec { items, .. } => {
                items.into_iter().map(key).fold(N::default(), |a, b| a + b)
            }
        }
    }

    /// Compute the arithmetic mean of a `f64` field extracted from each element.
    ///
    /// Returns `None` for an empty collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let avg = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]).average_by(|x| *x);
    /// assert_eq!(avg, Some(2.0));
    ///
    /// let empty = QueryBuilder::from(Vec::<f64>::new()).average_by(|x| *x);
    /// assert_eq!(empty, None);
    /// ```
    pub fn average_by<F>(self, key: F) -> Option<f64>
    where
        F: Fn(&T) -> f64,
    {
        let mut sum = 0.0_f64;
        let mut count = 0usize;
        match self.data {
            QueryData::Iterator(iter) => {
                for item in iter {
                    sum += key(&item);
                    count += 1;
                }
            }
            QueryData::SortedVec { items, .. } => {
                for item in &items {
                    sum += key(item);
                    count += 1;
                }
            }
        }
        if count == 0 { None } else { Some(sum / count as f64) }
    }

    /// Alias for [`aggregate_no_seed`](QueryBuilder::aggregate_no_seed).
    ///
    /// Fold all elements without a seed value, returning `None` if empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let max = QueryBuilder::from(vec![3, 1, 4, 1, 5])
    ///     .reduce(|a, b| if a > b { a } else { b });
    /// assert_eq!(max, Some(5));
    /// ```
    pub fn reduce<F>(self, f: F) -> Option<T>
    where
        F: Fn(T, T) -> T,
    {
        self.aggregate_no_seed(f)
    }

    /// Check whether all elements are distinct (`T: Hash + Eq`).
    ///
    /// Returns `true` for an empty collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// assert!(QueryBuilder::from(vec![1, 2, 3]).all_unique());
    /// assert!(!QueryBuilder::from(vec![1, 2, 2, 3]).all_unique());
    /// assert!(QueryBuilder::from(Vec::<i32>::new()).all_unique());
    /// ```
    pub fn all_unique(self) -> bool
    where
        T: Hash + Eq,
    {
        let mut seen = HashSet::new();
        match self.data {
            QueryData::Iterator(iter) => {
                for item in iter {
                    if !seen.insert(item) {
                        return false;
                    }
                }
            }
            QueryData::SortedVec { items, .. } => {
                for item in items {
                    if !seen.insert(item) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check whether **no** element satisfies the predicate.
    ///
    /// Equivalent to `!any(pred)`. Returns `true` for an empty collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// assert!(QueryBuilder::from(vec![1, 2, 3]).none(|x| *x > 10));
    /// assert!(!QueryBuilder::from(vec![1, 2, 3]).none(|x| *x > 2));
    /// assert!(QueryBuilder::from(Vec::<i32>::new()).none(|x| *x > 0));
    /// ```
    pub fn none<F>(self, predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        !match self.data {
            QueryData::Iterator(mut iter) => iter.any(|item| predicate(&item)),
            QueryData::SortedVec { items, .. } => items.iter().any(|item| predicate(item as &T)),
        }
    }
}

// ── Phase 5E: query enrichment ───────────────────────────────────────────────

impl<T: 'static, State> QueryBuilder<T, State> {
    /// Count how many times each distinct value appears.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let freq = QueryBuilder::from(vec!["a", "b", "a", "c", "b", "a"])
    ///     .frequencies();
    /// assert_eq!(freq[&"a"], 3);
    /// assert_eq!(freq[&"b"], 2);
    /// assert_eq!(freq[&"c"], 1);
    /// ```
    pub fn frequencies(self) -> HashMap<T, usize>
    where
        T: Hash + Eq,
    {
        let mut map: HashMap<T, usize> = HashMap::new();
        match self.data {
            QueryData::Iterator(iter) => {
                for item in iter {
                    *map.entry(item).or_insert(0) += 1;
                }
            }
            QueryData::SortedVec { items, .. } => {
                for item in items {
                    *map.entry(item).or_insert(0) += 1;
                }
            }
        }
        map
    }

    /// Flatten one level of nesting from a sequence of iterables.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
    /// let flat: Vec<i32> = QueryBuilder::from(nested).flatten().collect();
    /// assert_eq!(flat, vec![1, 2, 3, 4, 5]);
    /// ```
    pub fn flatten<U>(self) -> QueryBuilder<U, Filtered>
    where
        T: IntoIterator<Item = U>,
        U: 'static,
    {
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(iter.flatten())),
            _state: PhantomData,
        }
    }

    /// Return the zero-based index of the first element matching the predicate,
    /// or `None` if no match is found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let pos = QueryBuilder::from(vec![10, 20, 30, 40])
    ///     .position(|x| *x == 30);
    /// assert_eq!(pos, Some(2));
    ///
    /// let none = QueryBuilder::from(vec![10, 20, 30])
    ///     .position(|x| *x == 99);
    /// assert_eq!(none, None);
    /// ```
    pub fn position<F>(self, mut predicate: F) -> Option<usize>
    where
        F: FnMut(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(iter) => iter
                .enumerate()
                .find_map(|(i, item)| if predicate(&item) { Some(i) } else { None }),
            QueryData::SortedVec { items, .. } => {
                items.iter().position(|item| predicate(item as &T))
            }
        }
    }

    /// Return the first element matching a predicate, or `None` if absent.
    ///
    /// Alias for `.where_(pred).first()` applied efficiently in a single pass.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let found = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .find(|x| *x > 3);
    /// assert_eq!(found, Some(4));
    ///
    /// let not_found = QueryBuilder::from(vec![1, 2, 3])
    ///     .find(|x| *x > 10);
    /// assert_eq!(not_found, None);
    /// ```
    pub fn find<F>(self, mut predicate: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.find(|item| predicate(item)),
            QueryData::SortedVec { items, .. } => {
                items.into_iter().find(|item| predicate(item))
            }
        }
    }

    /// Return the zero-based index of the first occurrence of `value`,
    /// or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let idx = QueryBuilder::from(vec![10, 20, 30, 20])
    ///     .index_of(&20);
    /// assert_eq!(idx, Some(1));
    ///
    /// let none = QueryBuilder::from(vec![1, 2, 3])
    ///     .index_of(&99);
    /// assert_eq!(none, None);
    /// ```
    pub fn index_of(self, value: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        match self.data {
            QueryData::Iterator(iter) => iter
                .enumerate()
                .find_map(|(i, item)| if item == *value { Some(i) } else { None }),
            QueryData::SortedVec { items, .. } => {
                items.iter().position(|item| item == value)
            }
        }
    }

    /// Alias for [`element_at`](QueryBuilder::element_at).
    ///
    /// Return the element at zero-based `index`, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// assert_eq!(QueryBuilder::from(vec![10, 20, 30]).nth(1), Some(20));
    /// assert_eq!(QueryBuilder::from(vec![10, 20, 30]).nth(5), None);
    /// ```
    pub fn nth(self, index: usize) -> Option<T> {
        self.element_at(index)
    }

    /// Alias for `chunk` available from any query state.
    ///
    /// Split the sequence into fixed-size `Vec<T>` batches.
    /// The last batch may be smaller than `size` if the total count is not
    /// evenly divisible.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let batches: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .batch(2)
    ///     .collect();
    /// assert_eq!(batches, vec![vec![1, 2], vec![3, 4], vec![5]]);
    /// ```
    pub fn batch(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> {
        assert!(size > 0, "batch size must be greater than 0");
        let iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };
        QueryBuilder {
            data: QueryData::Iterator(Box::new(ChunkIterator {
                inner: iter,
                chunk_size: size,
            })),
            _state: PhantomData,
        }
    }

    /// Alias for [`single`](QueryBuilder::single).
    ///
    /// Return `Ok(element)` if exactly one element exists, or `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// assert_eq!(QueryBuilder::from(vec![42]).exactly_one(), Ok(42));
    /// assert!(QueryBuilder::from(vec![1, 2]).exactly_one().is_err());
    /// assert!(QueryBuilder::from(Vec::<i32>::new()).exactly_one().is_err());
    /// ```
    pub fn exactly_one(self) -> RinqResult<T> {
        self.single()
    }

    /// Produce two independent `Vec<T>` clones of the sequence.
    ///
    /// Useful when you need to apply two different terminal operations on the
    /// same data without rebuilding the source.
    ///
    /// ⚠ Clones all elements. Both `Vec` values are fully independent.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let (a, b) = QueryBuilder::from(vec![1, 2, 3]).tee();
    /// assert_eq!(a, vec![1, 2, 3]);
    /// assert_eq!(b, vec![1, 2, 3]);
    /// ```
    pub fn tee(self) -> (Vec<T>, Vec<T>)
    where
        T: Clone,
    {
        let items = self.into_vec();
        let clone = items.clone();
        (items, clone)
    }
}

// ── parallel feature ────────────────────────────────────────────────────────

#[cfg(feature = "parallel")]
impl<T: Send + 'static, State> QueryBuilder<T, State> {
    /// `QueryBuilder` を `ParallelQueryBuilder` に変換する。
    ///
    /// 内部データを `Vec<T>` にマテリアライズしてから `ParallelQueryBuilder` に渡す。
    /// 遅延評価のメリットはこの時点で失われるが、以降の操作は rayon で並列実行される。
    ///
    /// **実行種別**: 即時実行（マテリアライズ）
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    ///
    /// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .where_(|x| *x > 2)
    ///     .into_parallel()
    ///     .par_where(|x| *x % 2 != 0)
    ///     .collect();
    /// assert_eq!(result, vec![3, 5]);
    /// ```
    pub fn into_parallel(self) -> ParallelQueryBuilder<T, State> {
        ParallelQueryBuilder {
            items: self.into_vec(),
            _state: PhantomData,
        }
    }
}
