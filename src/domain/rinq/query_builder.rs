// src/domain/rinq/query_builder.rs
// Core QueryBuilder implementation with type state pattern

use super::state::{Filtered, Initial, Projected, Sorted};
use std::marker::PhantomData;

/// QueryBuilder - the core query construction type
/// Uses type state pattern to enforce valid query construction at compile time
pub struct QueryBuilder<T, State> {
    source: Box<dyn Iterator<Item = T>>,
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
            source: Box::new(source.into_iter()),
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
        QueryBuilder {
            source: Box::new(self.source.filter(predicate)),
            _state: PhantomData,
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
        QueryBuilder {
            source: Box::new(self.source.filter(predicate)),
            _state: PhantomData,
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
        let mut items: Vec<T> = self.source.collect();
        items.sort_by_key(key_selector);
        QueryBuilder {
            source: Box::new(items.into_iter()),
            _state: PhantomData,
        }
    }

    /// Transform elements to a different type
    #[inline]
    pub fn select<U, F>(self, projection: F) -> QueryBuilder<U, Projected<U>>
    where
        F: Fn(T) -> U + 'static,
        U: 'static,
    {
        QueryBuilder {
            source: Box::new(self.source.map(projection)),
            _state: PhantomData,
        }
    }

    /// Take at most n elements
    #[inline]
    pub fn take(self, n: usize) -> QueryBuilder<T, Filtered> {
        QueryBuilder {
            source: Box::new(self.source.take(n)),
            _state: PhantomData,
        }
    }

    /// Skip the first n elements
    #[inline]
    pub fn skip(self, n: usize) -> QueryBuilder<T, Filtered> {
        QueryBuilder {
            source: Box::new(self.source.skip(n)),
            _state: PhantomData,
        }
    }
}

impl<T: 'static> QueryBuilder<T, Sorted> {
    /// Apply a secondary sort key
    #[inline]
    pub fn then_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Sorted>
    where
        F: Fn(&T) -> K + 'static,
        K: Ord + 'static,
        T: 'static,
    {
        let mut items: Vec<T> = self.source.collect();
        items.sort_by_key(key_selector);
        QueryBuilder {
            source: Box::new(items.into_iter()),
            _state: PhantomData,
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
        self.source.collect()
    }

    /// Count the number of elements
    #[inline]
    pub fn count(self) -> usize {
        self.source.count()
    }

    /// Get the first element, if any
    #[inline]
    pub fn first(mut self) -> Option<T> {
        self.source.next()
    }

    /// Get the last element, if any
    #[inline]
    pub fn last(self) -> Option<T> {
        self.source.last()
    }

    /// Check if any element satisfies the predicate
    #[inline]
    pub fn any<F>(mut self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.source.any(|item| predicate(&item))
    }

    /// Check if all elements satisfy the predicate
    #[inline]
    pub fn all<F>(mut self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.source.all(|item| predicate(&item))
    }

    /// Inspect elements without consuming the query
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: Fn(&T) + 'static,
    {
        Self {
            source: Box::new(self.source.inspect(f)),
            _state: PhantomData,
        }
    }
}
