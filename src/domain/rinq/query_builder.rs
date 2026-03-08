// src/domain/rinq/query_builder.rs
// Core QueryBuilder implementation with type state pattern

use super::state::{Filtered, Initial, Projected, Sorted};
use std::cmp::Ordering;
use std::marker::PhantomData;

enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec {
        items: Vec<T>,
        comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    },
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
                
                let comparator = Box::new(move |a: &T, b: &T| {
                    key_selector(a).cmp(&key_selector(b))
                });
                
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
                
                let comparator = Box::new(move |a: &T, b: &T| {
                    key_selector(b).cmp(&key_selector(a))
                });
                
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
                
                let comparator = Box::new(move |a: &T, b: &T| {
                    key_selector(a).cmp(&key_selector(b))
                });
                
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
                
                let comparator = Box::new(move |a: &T, b: &T| {
                    key_selector(b).cmp(&key_selector(a))
                });
                
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
            QueryData::SortedVec { mut items, comparator } => {
                let primary_comparator = comparator;
                let new_comparator = Box::new(move |a: &T, b: &T| {
                    match primary_comparator(a, b) {
                        Ordering::Equal => key_selector(a).cmp(&key_selector(b)),
                        other => other,
                    }
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
            QueryData::SortedVec { mut items, comparator } => {
                let primary_comparator = comparator;
                let new_comparator = Box::new(move |a: &T, b: &T| {
                    match primary_comparator(a, b) {
                        Ordering::Equal => key_selector(b).cmp(&key_selector(a)),
                        other => other,
                    }
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
            QueryData::SortedVec { items, .. } => {
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter().take(n))),
                    _state: PhantomData,
                }
            }
            QueryData::Iterator(_) => unreachable!("Sorted state must be SortedVec"),
        }
    }

    /// Skip the first n elements from sorted query
    #[inline]
    pub fn skip(self, n: usize) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter().skip(n))),
                    _state: PhantomData,
                }
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
