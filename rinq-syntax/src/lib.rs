//! LINQ-style `query!` macro for the [`rinq`](https://docs.rs/rinq) query engine.
//!
//! > **Experimental** — the query syntax may change in future minor versions.
//!
//! # Syntax
//!
//! ```text
//! query! {
//!     from <binding> in <source>
//!     [where <predicate>]*
//!     [take <n>]
//!     [skip <n>]
//!     [order_by <key> [asc|desc] [, <key> [asc|desc]]*]
//!     [then_by  <key> [asc|desc] [, <key> [asc|desc]]*]*
//!     [select <projection>]
//! }
//! ```
//!
//! The macro always evaluates eagerly and returns `Vec<T>` (or `Vec<U>` when a
//! `select` clause is present).
//!
//! ## Binding type
//!
//! In `where`, `order_by`, and `then_by` clauses the binding is a **shared
//! reference** (`&T`).  Field access on structs auto-derefs correctly:
//! `user.age`.  For primitive collections, prefix with `*` to obtain the value:
//! `*x > 18`.
//!
//! In `select` the binding is an **owned value** (`T`), so fields can be moved
//! out directly: `select user.name`.
//!
//! # Examples
//!
//! ```rust,ignore
//! use rinq_syntax::query;
//!
//! #[derive(Debug, Clone)]
//! struct User { name: String, age: u32, active: bool }
//!
//! let users = vec![
//!     User { name: "Alice".into(), age: 28, active: true  },
//!     User { name: "Bob".into(),   age: 17, active: false },
//! ];
//!
//! let adults: Vec<_> = query! {
//!     from user in users
//!     where user.age >= 18
//!     where user.active
//!     order_by user.age
//!     select user.name
//! };
//! assert_eq!(adults, vec!["Alice"]);
//! ```

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod ast;
mod codegen;
mod parser;

/// LINQ-style query macro.
///
/// See the [crate-level documentation](self) for the full syntax reference.
#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let query_input = parse_macro_input!(input as ast::QueryInput);
    codegen::generate(query_input).into()
}
