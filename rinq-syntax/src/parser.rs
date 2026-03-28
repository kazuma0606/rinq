// rinq-syntax/src/parser.rs
// Parser for the `query!` macro DSL.

use proc_macro2::{TokenStream, TokenTree};
use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
};

use crate::ast::{Clause, QueryInput, SortKey};

// ── custom keywords ───────────────────────────────────────────────────────────

mod kw {
    syn::custom_keyword!(from);
    syn::custom_keyword!(select);
    syn::custom_keyword!(order_by);
    syn::custom_keyword!(then_by);
    syn::custom_keyword!(take);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(desc);
    syn::custom_keyword!(asc);
    syn::custom_keyword!(join);
    syn::custom_keyword!(on);
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Returns `true` if the next token starts a new query clause.
fn is_clause_keyword(input: ParseStream) -> bool {
    input.peek(kw::from)
        || input.peek(Token![where])
        || input.peek(kw::order_by)
        || input.peek(kw::then_by)
        || input.peek(kw::select)
        || input.peek(kw::take)
        || input.peek(kw::skip)
        || input.peek(kw::join)
        || peek_left_join(input)
}

/// Returns `true` if the next two tokens are the ident `left` followed by `join`.
fn peek_left_join(input: ParseStream) -> bool {
    let fork = input.fork();
    if let Ok(id) = fork.parse::<syn::Ident>() {
        return id == "left" && fork.peek(kw::join);
    }
    false
}

/// Collects tokens until the next clause keyword OR `on` keyword (for join source).
fn parse_until_clause_keyword_or_on(input: ParseStream) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    while !input.is_empty() && !is_clause_keyword(input) && !input.peek(kw::on) {
        let tt: TokenTree = input.parse()?;
        tokens.extend(std::iter::once(tt));
    }
    Ok(tokens)
}

/// Collects tokens until `==` or a clause keyword (for join left_key).
fn parse_until_eq_eq(input: ParseStream) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    while !input.is_empty() && !input.peek(Token![==]) && !is_clause_keyword(input) {
        let tt: TokenTree = input.parse()?;
        tokens.extend(std::iter::once(tt));
    }
    Ok(tokens)
}

/// Collects tokens until the next clause keyword (or end of input).
///
/// The collected tokens are returned as a raw `TokenStream` suitable for
/// splicing into generated code.
fn parse_until_clause_keyword(input: ParseStream) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    while !input.is_empty() && !is_clause_keyword(input) {
        let tt: TokenTree = input.parse()?;
        tokens.extend(std::iter::once(tt));
    }
    Ok(tokens)
}

/// Parses one or more comma-separated sort keys, each optionally followed by
/// `asc` or `desc`.
fn parse_sort_keys(input: ParseStream) -> Result<Vec<SortKey>> {
    let mut keys = Vec::new();
    loop {
        let mut key_tokens = TokenStream::new();
        while !input.is_empty()
            && !input.peek(Token![,])
            && !input.peek(kw::desc)
            && !input.peek(kw::asc)
            && !is_clause_keyword(input)
        {
            let tt: TokenTree = input.parse()?;
            key_tokens.extend(std::iter::once(tt));
        }

        if key_tokens.is_empty() {
            return Err(input.error("expected sort-key expression"));
        }

        let desc = if input.peek(kw::desc) {
            input.parse::<kw::desc>()?;
            true
        } else if input.peek(kw::asc) {
            input.parse::<kw::asc>()?;
            false
        } else {
            false
        };

        keys.push(SortKey { tokens: key_tokens, desc });

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        } else {
            break;
        }
    }
    Ok(keys)
}

// ── Parse impl ────────────────────────────────────────────────────────────────

impl Parse for QueryInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // ── from <binding> in <source> ────────────────────────────────────────
        if !input.peek(kw::from) {
            return Err(input.error("query! must start with `from <binding> in <source>`"));
        }
        input.parse::<kw::from>()?;

        let binding: syn::Ident = input.parse()?;
        input.parse::<Token![in]>()?;

        let source_tokens = parse_until_clause_keyword(input)?;
        if source_tokens.is_empty() {
            return Err(input.error("expected source expression after `in`"));
        }

        // ── subsequent clauses ────────────────────────────────────────────────
        let mut clauses: Vec<Clause> = Vec::new();
        let mut seen_order_by = false;

        while !input.is_empty() {
            if input.peek(kw::from) {
                return Err(syn::Error::new(
                    input.span(),
                    "multiple `from` clauses are not supported; use `join` instead",
                ));
            } else if input.peek(Token![where]) {
                if seen_order_by {
                    return Err(syn::Error::new(
                        input.span(),
                        "`where` cannot appear after `order_by`; \
                         move all `where` clauses before `order_by`",
                    ));
                }
                input.parse::<Token![where]>()?;
                let tokens = parse_until_clause_keyword(input)?;
                if tokens.is_empty() {
                    return Err(input.error("expected predicate expression after `where`"));
                }
                clauses.push(Clause::Where { tokens });
            } else if input.peek(kw::take) {
                input.parse::<kw::take>()?;
                let tokens = parse_until_clause_keyword(input)?;
                if tokens.is_empty() {
                    return Err(input.error("expected count expression after `take`"));
                }
                clauses.push(Clause::Take { tokens });
            } else if input.peek(kw::skip) {
                input.parse::<kw::skip>()?;
                let tokens = parse_until_clause_keyword(input)?;
                if tokens.is_empty() {
                    return Err(input.error("expected count expression after `skip`"));
                }
                clauses.push(Clause::Skip { tokens });
            } else if input.peek(kw::order_by) {
                input.parse::<kw::order_by>()?;
                let keys = parse_sort_keys(input)?;
                clauses.push(Clause::OrderBy { keys });
                seen_order_by = true;
            } else if input.peek(kw::then_by) {
                if !seen_order_by {
                    return Err(syn::Error::new(
                        input.span(),
                        "`then_by` must appear after `order_by`",
                    ));
                }
                input.parse::<kw::then_by>()?;
                let keys = parse_sort_keys(input)?;
                clauses.push(Clause::ThenBy { keys });
            } else if input.peek(kw::select) {
                input.parse::<kw::select>()?;
                let tokens = parse_until_clause_keyword(input)?;
                if tokens.is_empty() {
                    return Err(input.error("expected projection expression after `select`"));
                }
                clauses.push(Clause::Select { tokens });
            } else if input.peek(kw::join) || peek_left_join(input) {
                // Parse: [left] join <binding> in <source> on <left_key> == <right_key>
                let is_left = if peek_left_join(input) {
                    input.parse::<syn::Ident>()?; // consume "left"
                    true
                } else {
                    false
                };
                input.parse::<kw::join>()?;

                let join_binding: syn::Ident = input.parse()?;
                input.parse::<Token![in]>()?;
                let source_tokens = parse_until_clause_keyword_or_on(input)?;
                if source_tokens.is_empty() {
                    return Err(input.error("expected source expression after `in`"));
                }

                input.parse::<kw::on>()?;

                // Parse left_key up to `==`
                let left_key = parse_until_eq_eq(input)?;
                if left_key.is_empty() {
                    return Err(input.error("expected key expression before `==` in `on` clause"));
                }
                input.parse::<Token![==]>()?;

                // Parse right_key until next clause keyword
                let right_key = parse_until_clause_keyword(input)?;
                if right_key.is_empty() {
                    return Err(input.error("expected key expression after `==` in `on` clause"));
                }

                clauses.push(Clause::Join {
                    binding: join_binding,
                    source_tokens,
                    left_key,
                    right_key,
                    is_left,
                });
                seen_order_by = false; // join resets sort state
            } else {
                return Err(input.error(
                    "unexpected token; expected a query clause keyword \
                     (`where`, `order_by`, `then_by`, `select`, `take`, `skip`, `join`)",
                ));
            }
        }

        Ok(QueryInput { binding, source_tokens, clauses })
    }
}
