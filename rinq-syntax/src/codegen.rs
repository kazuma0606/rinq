// rinq-syntax/src/codegen.rs
// Code generation for the `query!` macro.

use proc_macro2::TokenStream;
use quote::quote;

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

    for clause in &input.clauses {
        match clause {
            Clause::Where { tokens } => {
                chain = quote! {
                    #chain.where_(|#binding| { #tokens })
                };
                state = State::Filtered;
            }

            Clause::Take { tokens } => {
                chain = quote! {
                    #chain.take(#tokens as usize)
                };
                // take is available on Initial, Filtered, and Sorted; state unchanged
            }

            Clause::Skip { tokens } => {
                chain = quote! {
                    #chain.skip(#tokens as usize)
                };
                // skip is available on Initial, Filtered, and Sorted; state unchanged
            }

            Clause::OrderBy { keys } => {
                let first = &keys[0];
                let key = &first.tokens;
                chain = if first.desc {
                    quote! { #chain.order_by_descending(|#binding| #key) }
                } else {
                    quote! { #chain.order_by(|#binding| #key) }
                };
                state = State::Sorted;
                for key_spec in keys.iter().skip(1) {
                    let key = &key_spec.tokens;
                    chain = if key_spec.desc {
                        quote! { #chain.then_by_descending(|#binding| #key) }
                    } else {
                        quote! { #chain.then_by(|#binding| #key) }
                    };
                }
            }

            Clause::ThenBy { keys } => {
                for key_spec in keys {
                    let key = &key_spec.tokens;
                    chain = if key_spec.desc {
                        quote! { #chain.then_by_descending(|#binding| #key) }
                    } else {
                        quote! { #chain.then_by(|#binding| #key) }
                    };
                }
                // ThenBy keeps Sorted state
            }

            Clause::Select { tokens } => {
                // `select` is only available on Filtered state.
                // If we are in Initial or Sorted state, we need to collect first
                // for Sorted (which is already done via the sorted vec), but for
                // Initial we need to transition to Filtered via a pass-through filter.
                if state == State::Initial {
                    chain = quote! {
                        #chain.where_(|_| true)
                    };
                    state = State::Filtered;
                } else if state == State::Sorted {
                    // After sorting, transition to Filtered via inspect (no-op side-effect),
                    // since `where_` is not available on Sorted but `select` requires Filtered.
                    chain = quote! {
                        #chain.inspect(|_| {})
                    };
                    state = State::Filtered;
                }
                chain = quote! {
                    #chain.select(|#binding| { #tokens })
                };
            }
        }
    }

    // Always terminate with collect.
    chain = quote! {
        #chain.collect::<::std::vec::Vec<_>>()
    };

    quote! { { #chain } }
}
