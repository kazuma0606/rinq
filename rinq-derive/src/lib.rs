//! Derive macros for the [`rinq`](https://docs.rs/rinq) query engine.
//!
//! # `#[derive(Queryable)]`
//!
//! Generates per-field accessor functions and a typed predicate module for
//! any named-field struct.
//!
//! ```ignore
//! use rinq::QueryBuilder;
//! use rinq_derive::Queryable;
//!
//! #[derive(Debug, Clone, Queryable)]
//! pub struct User {
//!     pub name: String,
//!     pub age: u32,
//!     pub active: bool,
//! }
//!
//! use user_fields::*;
//!
//! let adults: Vec<User> = QueryBuilder::from(users)
//!     .where_(Age::gt(18))
//!     .where_(Active::is_true())
//!     .order_by(User::by_age)
//!     .collect();
//! ```
//!
//! ## Generated accessors
//!
//! For each non-skipped field `foo: T`, a method `by_foo(item: &Struct) -> T`
//! is added to the struct's `impl` block.  Use these directly with `order_by`
//! and `group_by` without writing a lambda.
//!
//! ## Generated predicate module
//!
//! A module named `{snake_case_struct}_fields` is generated containing one
//! zero-sized predicate builder struct per field.  The available methods
//! depend on the field type:
//!
//! | Field type | Methods |
//! |---|---|
//! | `bool` | `is_true()`, `is_false()` |
//! | `String` / `&str` | `eq(&str)`, `contains(&str)`, `starts_with(&str)`, `is_empty()` |
//! | Numeric | `gt(n)`, `lt(n)`, `eq(n)`, `between(lo, hi)` |
//! | Other | `eq(val)` |
//!
//! ## Attributes
//!
//! | Attribute | Effect |
//! |---|---|
//! | `#[queryable(skip)]` | No accessor or predicate is generated for this field |
//! | `#[queryable(rename = "name")]` | Accessor is named `by_name` instead of `by_{field}` |
//! | `#[queryable(key)]` | Also generates a `default_key` accessor pointing to this field |
//!
//! # `#[derive(QueryableFrom)]`
//!
//! Implements `From<Wrapper>` and `IntoQuery` for a newtype struct wrapping `Vec<T>`.
//!
//! ```ignore
//! use rinq_derive::QueryableFrom;
//!
//! #[derive(QueryableFrom)]
//! pub struct UserList(Vec<User>);
//!
//! let result: Vec<User> = UserList(users).into_query()
//!     .where_(|u| u.age > 18)
//!     .collect();
//! ```

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod from;
mod queryable;

/// Derive accessor functions and a typed predicate module for a struct.
///
/// See the [crate-level documentation](self) for full details.
#[proc_macro_derive(Queryable, attributes(queryable))]
pub fn derive_queryable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    queryable::derive_queryable(input).into()
}

/// Implement `From<Wrapper>` and `IntoQuery` for a newtype wrapping `Vec<T>`.
///
/// See the [crate-level documentation](self) for full details.
#[proc_macro_derive(QueryableFrom)]
pub fn derive_queryable_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    from::derive_queryable_from(input).into()
}
