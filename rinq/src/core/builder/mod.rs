// src/core/builder/mod.rs
// Module declarations + QueryData<T> enum + QueryBuilder<T, State> struct

mod filtered;
mod functional;
mod initial;
pub mod iterators;
mod join;
mod queryable;
mod serde_ops;
mod shared;
mod sorted;
mod try_ops;
mod window;

pub use queryable::Queryable;
pub use shared::IntoQuery;

use std::cmp::Ordering;
use std::marker::PhantomData;

#[allow(clippy::type_complexity)]
pub(crate) enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec {
        items: Vec<T>,
        comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    },
}

/// QueryBuilder - the core query construction type
/// Uses type state pattern to enforce valid query construction at compile time
pub struct QueryBuilder<T, State> {
    pub(crate) data: QueryData<T>,
    pub(crate) _state: PhantomData<State>,
}
