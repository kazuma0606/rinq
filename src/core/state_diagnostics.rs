// src/core/state_diagnostics.rs
// Diagnostic traits for improved compile-time error messages when operations
// are called on incompatible query states.

use crate::core::state::{Filtered, Initial, Sorted};

/// Implemented by query states that support `select` (projection).
///
/// Only [`Filtered`] implements this trait, so calling `select` on an
/// [`Initial`] or [`Sorted`] query produces a clear error message referencing
/// this trait rather than a generic type mismatch.
pub trait SupportsSelect {}

/// Implemented by query states that support `then_by` / `then_by_descending`.
///
/// Only [`Sorted`] implements this trait.
pub trait SupportsThenBy {}

/// Implemented by query states that support `order_by` / `order_by_descending`.
///
/// Both [`Initial`] and [`Filtered`] implement this trait.
pub trait SupportsOrderBy {}

impl SupportsSelect for Filtered {}

impl SupportsThenBy for Sorted {}

impl SupportsOrderBy for Initial {}
impl SupportsOrderBy for Filtered {}

/// Bound for element types that support set operations (`distinct`, `union`,
/// `intersect`, `except`).
///
/// Any type that implements both [`std::hash::Hash`] and [`Eq`] automatically
/// implements `HashEqBound`. If your type does not implement one of these, the
/// compiler error will reference `HashEqBound` and point directly to the
/// missing derive.
pub trait HashEqBound: std::hash::Hash + Eq {}

impl<T: std::hash::Hash + Eq> HashEqBound for T {}

/// Internal macro for defining a sealed state-constraint trait with a single
/// blanket implementation.
///
/// Usage:
/// ```ignore
/// define_state_constraint!(MyConstraint, StateA, StateB);
/// ```
/// expands to a `pub trait MyConstraint {}` plus `impl MyConstraint for StateA {}`
/// and `impl MyConstraint for StateB {}`.
#[macro_export]
#[doc(hidden)]
macro_rules! define_state_constraint {
    ($trait_name:ident, $($state:ty),+) => {
        pub trait $trait_name {}
        $(impl $trait_name for $state {})+
    };
}
