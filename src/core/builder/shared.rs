// src/core/builder/shared.rs
// impl<T, State> QueryBuilder<T, State> — terminal operations available in all states

use super::{QueryBuilder, QueryData};
use crate::core::error::{RinqError, RinqResult};
use crate::core::state::Filtered;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;

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
        T: Hash + Eq + Clone,
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
        T: Hash + Eq + Clone,
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
        T: Hash + Eq + Clone,
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
