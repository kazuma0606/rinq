// src/core/builder/queryable.rs
// Queryable trait and impls

use super::QueryBuilder;
use crate::core::state::Initial;

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
