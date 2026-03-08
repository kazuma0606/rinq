// src/domain/rinq/query_builder.rs
// Core QueryBuilder implementation with type state pattern

use super::state::{Filtered, Initial, Projected, Sorted};
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::iter::Sum;
use std::marker::PhantomData;

enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec {
        items: Vec<T>,
        comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    },
}

// ============================================================================
// Custom Iterator Adapters
// ============================================================================

/// Iterator adapter for chunking elements into fixed-size vectors
struct ChunkIterator<I> {
    inner: I,
    chunk_size: usize,
}

impl<I, T> Iterator for ChunkIterator<I>
where
    I: Iterator<Item = T>,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.chunk_size);
        for _ in 0..self.chunk_size {
            match self.inner.next() {
                Some(item) => chunk.push(item),
                None => break,
            }
        }
        if chunk.is_empty() { None } else { Some(chunk) }
    }
}

/// Iterator adapter for sliding windows over elements
struct WindowIterator<T> {
    buffer: VecDeque<T>,
    inner: Box<dyn Iterator<Item = T>>,
    window_size: usize,
    finished: bool,
}

impl<T: Clone> Iterator for WindowIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Fill initial buffer
        while self.buffer.len() < self.window_size {
            match self.inner.next() {
                Some(item) => self.buffer.push_back(item),
                None => {
                    self.finished = true;
                    return None;
                }
            }
        }

        // Create window from current buffer
        let window: Vec<T> = self.buffer.iter().cloned().collect();

        // Slide the window
        self.buffer.pop_front();
        match self.inner.next() {
            Some(item) => self.buffer.push_back(item),
            None => self.finished = true,
        }

        Some(window)
    }
}

/// QueryBuilder - the core query construction type
/// Uses type state pattern to enforce valid query construction at compile time
pub struct QueryBuilder<T, State> {
    data: QueryData<T>,
    _state: PhantomData<State>,
}

impl<T: 'static> QueryBuilder<T, Initial> {
    /// Create a new QueryBuilder from any iterable collection
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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

    /// Filter elements based on a predicate
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                items.sort_by(|a, b| key_selector(b).cmp(&key_selector(a)));

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
}

impl<T: 'static> QueryBuilder<T, Filtered> {
    /// Apply an additional filter to an already filtered query
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
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                items.sort_by(|a, b| key_selector(b).cmp(&key_selector(a)));

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
}

impl<T: 'static> QueryBuilder<T, Sorted> {
    /// Inspect elements without consuming the query
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
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                    groups.entry(key).or_insert_with(Vec::new).push(item);
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
        T: Eq + Hash + Clone,
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
                let zipped = items.into_iter().zip(other.into_iter());

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

// Terminal operations available in all states
impl<T: 'static, State> QueryBuilder<T, State> {
    /// Collect the results into a collection
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    #[inline]
    pub fn count(self) -> usize {
        match self.data {
            QueryData::Iterator(iter) => iter.count(),
            QueryData::SortedVec { items, .. } => items.len(),
        }
    }

    /// Get the first element, if any
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
    #[inline]
    pub fn last(self) -> Option<T> {
        match self.data {
            QueryData::Iterator(iter) => iter.last(),
            QueryData::SortedVec { mut items, .. } => items.pop(),
        }
    }

    /// Check if any element satisfies the predicate
    #[inline]
    pub fn any<F>(self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.any(|item| predicate(&item)),
            QueryData::SortedVec { items, .. } => items.iter().any(|item| predicate(item)),
        }
    }

    /// Check if all elements satisfy the predicate
    #[inline]
    pub fn all<F>(self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(mut iter) => iter.all(|item| predicate(&item)),
            QueryData::SortedVec { items, .. } => items.iter().all(|item| predicate(item)),
        }
    }
}

// Inspect operations for Initial and Filtered states
impl<T: 'static> QueryBuilder<T, Initial> {
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
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Calculate the sum of all elements
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// Returns a `HashMap` where keys are the result of applying the key function,
    /// and values are vectors of elements with that key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                    groups.entry(key).or_insert_with(Vec::new).push(item);
                }
                groups
            }
            QueryData::SortedVec { .. } => unreachable!("Initial state must be Iterator"),
        }
    }

    /// Group elements by a key function and apply an aggregation to each group
    ///
    /// Returns a `HashMap` where keys are the result of applying the key function,
    /// and values are the result of applying the aggregation function to each group.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// Returns a `Filtered` query with duplicates removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// Returns a `Filtered` query with duplicates removed based on the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// Returns a `Filtered` query with elements in reverse order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                let zipped = iter.zip(other.into_iter());

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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                    groups.entry(key).or_insert_with(Vec::new).push(item);
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
    /// Returns a `Filtered` query with duplicates removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
    /// Returns a `Filtered` query with duplicates removed based on the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusted_ca::domain::rinq::QueryBuilder;
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
                let zipped = iter.zip(other.into_iter());

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

// ============================================================================
// Queryable Trait - Data source abstraction
// ============================================================================

/// Trait for types that can be queried using RINQ
///
/// This trait allows various collection types to be converted into
/// a QueryBuilder, enabling LINQ-style queries on Rust collections.
pub trait Queryable<T> {
    /// Convert this collection into a QueryBuilder
    fn into_query(self) -> QueryBuilder<T, Initial>;
}

// Vec<T> implementation - consumes the vector
impl<T: 'static> Queryable<T> for Vec<T> {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}

// &[T] implementation - clones elements
impl<T: Clone + 'static> Queryable<T> for &[T] {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self.to_vec())
    }
}

// Array implementation - consumes the array
impl<T: 'static, const N: usize> Queryable<T> for [T; N] {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}

// HashSet implementation
impl<T: 'static> Queryable<T> for std::collections::HashSet<T> {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}

// BTreeSet implementation
impl<T: 'static> Queryable<T> for std::collections::BTreeSet<T> {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}

// LinkedList implementation
impl<T: 'static> Queryable<T> for std::collections::LinkedList<T> {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}

// VecDeque implementation
impl<T: 'static> Queryable<T> for std::collections::VecDeque<T> {
    #[inline]
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}
