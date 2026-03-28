// rinq-syntax/src/codegen.rs
// Code generation for the `query!` macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::ast::{Clause, QueryInput};

/// Tracks the compile-time query state so the code generator can insert
/// necessary state transitions.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Initial,
    Filtered,
    Sorted,
}

/// Generates the Rust expression that implements the query.
///
/// # Shape of the generated code
///
/// ```rust,ignore
/// {
///     ::rinq::__macro_support::from(<source>)
///         .where_(|<binding>| { <predicate> })
///         .order_by(|<binding>| <key>)
///         .select(|<binding>| { <projection> })
///         .collect::<::std::vec::Vec<_>>()
/// }
/// ```
///
/// The final `.collect()` is always appended.  When a `select` clause is
/// present it projects first; otherwise the original element type is collected.
pub fn generate(input: QueryInput) -> TokenStream {
    let binding = &input.binding;
    let source = &input.source_tokens;

    // Seed the chain with the stable entry-point function.
    let mut chain = quote! {
        ::rinq::__macro_support::from(#source)
    };

    let mut state = State::Initial;

    // The pattern used in closures for the current element.
    // Starts as the single binding; becomes a tuple pattern after a join.
    let mut closure_pat: TokenStream = quote! { #binding };

    for clause in &input.clauses {
        match clause {
            Clause::Where { tokens } => {
                let pat = &closure_pat;
                chain = quote! {
                    #chain.where_(|#pat| { #tokens })
                };
                state = State::Filtered;
            }

            Clause::Take { tokens } => {
                chain = quote! {
                    #chain.take(#tokens as usize)
                };
            }

            Clause::Skip { tokens } => {
                chain = quote! {
                    #chain.skip(#tokens as usize)
                };
            }

            Clause::OrderBy { keys } => {
                let first = &keys[0];
                let key = &first.tokens;
                let pat = &closure_pat;
                chain = if first.desc {
                    quote! { #chain.order_by_descending(|#pat| #key) }
                } else {
                    quote! { #chain.order_by(|#pat| #key) }
                };
                state = State::Sorted;
                for key_spec in keys.iter().skip(1) {
                    let key = &key_spec.tokens;
                    let pat = &closure_pat;
                    chain = if key_spec.desc {
                        quote! { #chain.then_by_descending(|#pat| #key) }
                    } else {
                        quote! { #chain.then_by(|#pat| #key) }
                    };
                }
            }

            Clause::ThenBy { keys } => {
                for key_spec in keys {
                    let key = &key_spec.tokens;
                    let pat = &closure_pat;
                    chain = if key_spec.desc {
                        quote! { #chain.then_by_descending(|#pat| #key) }
                    } else {
                        quote! { #chain.then_by(|#pat| #key) }
                    };
                }
            }

            Clause::Select { tokens } => {
                if state == State::Initial {
                    chain = quote! { #chain.where_(|_| true) };
                    state = State::Filtered;
                } else if state == State::Sorted {
                    chain = quote! { #chain.inspect(|_| {}) };
                    state = State::Filtered;
                }
                let pat = &closure_pat;
                chain = quote! {
                    #chain.select(|#pat| { #tokens })
                };
            }

            Clause::Join {
                binding: join_binding,
                source_tokens,
                left_key,
                right_key,
                is_left,
            } => {
                // Transition to Filtered if needed (join is available from any state).
                if state == State::Sorted {
                    chain = quote! { #chain.inspect(|_| {}) };
                }

                let outer_pat = &closure_pat;
                let method: Ident = if *is_left {
                    syn::parse_str("left_join").unwrap()
                } else {
                    syn::parse_str("inner_join").unwrap()
                };

                chain = quote! {
                    #chain.#method(
                        #source_tokens,
                        |#outer_pat| #left_key,
                        |#join_binding| #right_key,
                    )
                };

                // After join the element type becomes a tuple; update the closure pattern.
                let outer_binding = binding;
                closure_pat = quote! { (#outer_binding, #join_binding) };
                state = State::Filtered;
            }
        }
    }

    // Always terminate with collect.
    chain = quote! {
        #chain.collect::<::std::vec::Vec<_>>()
    };

    quote! { { #chain } }
}
