// src/domain/rinq/state.rs
// Type state pattern for compile-time query validation

use std::marker::PhantomData;

/// Initial state - query just created from a collection
#[derive(Debug, Clone, Copy)]
pub struct Initial;

/// Filtered state - query has been filtered with where_()
#[derive(Debug, Clone, Copy)]
pub struct Filtered;

/// Sorted state - query has been sorted with order_by()
#[derive(Debug, Clone, Copy)]
pub struct Sorted;

/// Projected state - query has been transformed with select()
#[derive(Debug, Clone, Copy)]
pub struct Projected<U> {
    _phantom: PhantomData<U>,
}

impl<U> Projected<U> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<U> Default for Projected<U> {
    fn default() -> Self {
        Self::new()
    }
}
