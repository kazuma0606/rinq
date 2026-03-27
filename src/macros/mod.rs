// src/macros/mod.rs
// Convenience macros exported from the `rinq` crate.

/// Time and optionally log a query terminal operation.
///
/// In debug builds (`cfg(debug_assertions)`), `rinq_explain!` wraps the
/// expression with [`std::time::Instant`] timing and prints the elapsed time
/// to stderr. In release builds the macro is a zero-cost pass-through.
///
/// # Examples
///
/// ```rust
/// use rinq::{QueryBuilder, rinq_explain};
///
/// let result: Vec<i32> = rinq_explain!(
///     QueryBuilder::from(vec![1, 2, 3, 4, 5])
///         .where_(|x| x % 2 == 0)
///         .collect::<Vec<_>>()
/// );
/// assert_eq!(result, vec![2, 4]);
/// ```
#[macro_export]
macro_rules! rinq_explain {
    ($expr:expr) => {{
        #[cfg(debug_assertions)]
        {
            let __start = ::std::time::Instant::now();
            let __result = $expr;
            let __elapsed = __start.elapsed();
            ::std::eprintln!("[rinq_explain] elapsed: {:?}", __elapsed);
            __result
        }
        #[cfg(not(debug_assertions))]
        {
            $expr
        }
    }};
}

/// Build a predicate closure from a simple expression.
///
/// `pred!` converts a boolean expression into a `|x| x.field op value`
/// closure, removing the repetition of the closure syntax.
///
/// # Single condition
///
/// ```rust
/// use rinq::{QueryBuilder, pred};
///
/// #[derive(Debug, Clone)]
/// struct User { age: u32, active: bool }
///
/// let users = vec![
///     User { age: 20, active: true },
///     User { age: 15, active: false },
///     User { age: 30, active: true },
/// ];
///
/// let result: Vec<_> = QueryBuilder::from(users)
///     .where_(pred!(age > 18))
///     .collect();
/// assert_eq!(result.len(), 2);
/// ```
///
/// # Compound condition with `&&`
///
/// ```rust
/// use rinq::{QueryBuilder, pred};
///
/// #[derive(Debug, Clone)]
/// struct User { age: u32, active: bool }
///
/// let users = vec![
///     User { age: 20, active: true },
///     User { age: 15, active: false },
///     User { age: 30, active: false },
/// ];
///
/// let result: Vec<_> = QueryBuilder::from(users)
///     .where_(pred!(age > 18 && active == true))
///     .collect();
/// assert_eq!(result.len(), 1);
/// ```
/// Internal helper for `pred!` — expands into an inline boolean expression
/// (not a closure) given a bound variable name and the condition tokens.
#[macro_export]
#[doc(hidden)]
macro_rules! pred_inner {
    // terminal: field op literal
    ($item:ident, $field:ident $op:tt $value:literal) => {
        $item.$field $op $value
    };
    // terminal: field op bool ident (true / false)
    ($item:ident, $field:ident $op:tt $value:ident) => {
        $item.$field $op $value
    };
    // compound: field op literal && rest
    ($item:ident, $field:ident $op:tt $value:literal && $($rest:tt)+) => {
        $item.$field $op $value && $crate::pred_inner!($item, $($rest)+)
    };
    // compound: field op ident && rest
    ($item:ident, $field:ident $op:tt $value:ident && $($rest:tt)+) => {
        $item.$field $op $value && $crate::pred_inner!($item, $($rest)+)
    };
}

/// Build a predicate closure from a field-expression shorthand.
///
/// `pred!(field op value)` expands to `|item: &_| item.field op value`,
/// saving repetition when used with `where_`.
/// Multiple conditions can be chained with `&&`.
///
/// # Examples
///
/// ```rust
/// use rinq::{QueryBuilder, pred};
///
/// #[derive(Debug, Clone)]
/// struct User { age: u32, active: bool }
///
/// let users = vec![
///     User { age: 20, active: true },
///     User { age: 15, active: false },
///     User { age: 30, active: true },
/// ];
///
/// // Single condition
/// let adults: Vec<_> = QueryBuilder::from(users.clone())
///     .where_(pred!(age > 18))
///     .collect();
/// assert_eq!(adults.len(), 2);
///
/// // Compound condition
/// let active_adults: Vec<_> = QueryBuilder::from(users)
///     .where_(pred!(age > 18 && active == true))
///     .collect();
/// assert_eq!(active_adults.len(), 2);
/// ```
#[macro_export]
macro_rules! pred {
    ($($tt:tt)+) => {
        |__pred_item: &_| $crate::pred_inner!(__pred_item, $($tt)+)
    };
}
