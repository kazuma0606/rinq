// src/domain/rinq/state.rs
// Type state pattern for compile-time query validation

use std::marker::PhantomData;

/// Initial state - query just created from a collection
#[derive(Debug, Clone, Copy)]
pub struct Initial;

/// Chainable intermediate state.
///
/// Despite the name, `Filtered` does **not** mean "the data has been filtered".
/// It is the general-purpose intermediate state that most operations produce and
/// accept.  Any operation that yields a lazily-chainable result (filtering,
/// scanning, deduplication, zip, etc.) transitions to `Filtered` so that the
/// next operation in the pipeline can be `select`, `order_by`, or another
/// chainable method.
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
    /// Create a new `Projected` state marker.
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
