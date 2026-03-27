// rinq-syntax/src/ast.rs
// AST types for the `query!` macro.

use proc_macro2::TokenStream;

/// The parsed content of a `query! { ... }` invocation.
pub struct QueryInput {
    /// The binding name declared in `from <binding> in <source>`.
    pub binding: syn::Ident,
    /// Raw tokens for the source expression in `from <binding> in <source>`.
    pub source_tokens: TokenStream,
    /// The sequence of query clauses that follow the `from` clause.
    pub clauses: Vec<Clause>,
}

/// A single query clause (everything after `from ... in ...`).
pub enum Clause {
    /// `where <predicate>`
    Where { tokens: TokenStream },
    /// `take <n>`
    Take { tokens: TokenStream },
    /// `skip <n>`
    Skip { tokens: TokenStream },
    /// `order_by <key> [desc] [, <key> [desc]]*`
    OrderBy { keys: Vec<SortKey> },
    /// `then_by <key> [desc] [, <key> [desc]]*`
    ThenBy { keys: Vec<SortKey> },
    /// `select <projection>`
    Select { tokens: TokenStream },
}

/// A single sort key with an optional `desc` modifier.
pub struct SortKey {
    /// Token stream for the key expression.
    pub tokens: TokenStream,
    /// `true` if `desc` was specified.
    pub desc: bool,
}
