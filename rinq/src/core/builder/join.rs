// src/core/builder/join.rs
// Phase 5F: JOIN operations — inner_join, left_join, cross_join

use super::{QueryBuilder, QueryData};
use crate::core::state::Filtered;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

impl<T: Clone + 'static, State> QueryBuilder<T, State> {
    /// Join this sequence with `right` by matching keys, producing all
    /// `(left, right)` pairs where `left_key(left) == right_key(right)`.
    ///
    /// The right-hand side is **eagerly collected** into a `HashMap` before
    /// iteration begins — O(M) space where M is the size of `right`.
    /// The overall complexity is O(N + M).
    ///
    /// ⚠ Right side is eagerly collected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct User  { id: u32, name: &'static str }
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Order { user_id: u32, amount: u32 }
    ///
    /// let users  = vec![User  { id: 1, name: "Alice" }, User  { id: 2, name: "Bob" }];
    /// let orders = vec![Order { user_id: 1, amount: 100 }, Order { user_id: 1, amount: 200 }];
    ///
    /// let mut result: Vec<(&str, u32)> = QueryBuilder::from(users)
    ///     .inner_join(orders, |u| u.id, |o| o.user_id)
    ///     .select(|(u, o)| (u.name, o.amount))
    ///     .collect();
    /// result.sort_by_key(|&(_, a)| a);
    /// assert_eq!(result, vec![("Alice", 100), ("Alice", 200)]);
    /// ```
    pub fn inner_join<U, K, FK, FU>(
        self,
        right: impl IntoIterator<Item = U> + 'static,
        left_key: FK,
        right_key: FU,
    ) -> QueryBuilder<(T, U), Filtered>
    where
        FK: Fn(&T) -> K + 'static,
        FU: Fn(&U) -> K + 'static,
        K: Hash + Eq + 'static,
        U: Clone + 'static,
    {
        // Eagerly build the right-side lookup map.
        let mut right_map: HashMap<K, Vec<U>> = HashMap::new();
        for item in right {
            let key = right_key(&item);
            right_map.entry(key).or_default().push(item);
        }

        let left_iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };

        let result = left_iter.flat_map(move |left| {
            let key = left_key(&left);
            match right_map.get(&key) {
                Some(rights) => rights
                    .iter()
                    .map(|r| (left.clone(), r.clone()))
                    .collect::<Vec<_>>(),
                None => vec![],
            }
        });

        QueryBuilder {
            data: QueryData::Iterator(Box::new(result)),
            _state: PhantomData,
        }
    }

    /// Left join: for each left element, emit `(left, Some(right))` for every
    /// match, or `(left, None)` if no right element matched.
    ///
    /// ⚠ Right side is eagerly collected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// #[derive(Clone, Debug)]
    /// struct User  { id: u32, name: &'static str }
    /// #[derive(Clone, Debug)]
    /// struct Order { user_id: u32, amount: u32 }
    ///
    /// let users  = vec![User  { id: 1, name: "Alice" }, User  { id: 2, name: "Bob" }];
    /// let orders = vec![Order { user_id: 1, amount: 50 }];
    ///
    /// let result: Vec<(&str, Option<u32>)> = QueryBuilder::from(users)
    ///     .left_join(orders, |u| u.id, |o| o.user_id)
    ///     .select(|(u, o)| (u.name, o.map(|x| x.amount)))
    ///     .collect();
    ///
    /// assert_eq!(result[0], ("Alice", Some(50)));
    /// assert_eq!(result[1], ("Bob",   None));
    /// ```
    pub fn left_join<U, K, FK, FU>(
        self,
        right: impl IntoIterator<Item = U> + 'static,
        left_key: FK,
        right_key: FU,
    ) -> QueryBuilder<(T, Option<U>), Filtered>
    where
        FK: Fn(&T) -> K + 'static,
        FU: Fn(&U) -> K + 'static,
        K: Hash + Eq + 'static,
        U: Clone + 'static,
    {
        let mut right_map: HashMap<K, Vec<U>> = HashMap::new();
        for item in right {
            let key = right_key(&item);
            right_map.entry(key).or_default().push(item);
        }

        let left_iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };

        let result = left_iter.flat_map(move |left| {
            let key = left_key(&left);
            match right_map.get(&key) {
                Some(rights) => rights
                    .iter()
                    .map(|r| (left.clone(), Some(r.clone())))
                    .collect::<Vec<_>>(),
                None => vec![(left, None)],
            }
        });

        QueryBuilder {
            data: QueryData::Iterator(Box::new(result)),
            _state: PhantomData,
        }
    }

    /// Cross join (Cartesian product): emit every `(left, right)` combination.
    ///
    /// ⚠ O(N×M) — use with caution on large collections.
    /// Right side is eagerly collected into a `Vec`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rinq::QueryBuilder;
    ///
    /// let mut result: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2])
    ///     .cross_join(vec![10, 20, 30])
    ///     .collect();
    /// assert_eq!(result.len(), 6);
    /// assert_eq!(result[0], (1, 10));
    /// assert_eq!(result[5], (2, 30));
    /// ```
    pub fn cross_join<U>(
        self,
        right: impl IntoIterator<Item = U> + 'static,
    ) -> QueryBuilder<(T, U), Filtered>
    where
        U: Clone + 'static,
    {
        let right_vec: Vec<U> = right.into_iter().collect();

        let left_iter: Box<dyn Iterator<Item = T>> = match self.data {
            QueryData::Iterator(it) => it,
            QueryData::SortedVec { items, .. } => Box::new(items.into_iter()),
        };

        let result = left_iter.flat_map(move |left| {
            right_vec
                .iter()
                .map(|r| (left.clone(), r.clone()))
                .collect::<Vec<_>>()
        });

        QueryBuilder {
            data: QueryData::Iterator(Box::new(result)),
            _state: PhantomData,
        }
    }
}
