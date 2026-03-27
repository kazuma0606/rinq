# RINQ — Rust Integrated Query

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-blue?logo=rust)](https://www.rust-lang.org/)

A type-safe, zero-cost LINQ-inspired query engine for Rust.

`rinq` lets you compose filter → sort → aggregate pipelines over any in-memory
collection using a fluent builder API. The type-state pattern encodes the valid
operation order at compile time — invalid chains are rejected without runtime
overhead.

---

## Installation

```toml
[dependencies]
rinq = "4"

# Optional: parallel execution (rayon) and/or serde JSON support
rinq = { version = "4", features = ["parallel", "serde"] }

# Statistical extensions (separate crate)
rinq-stats = "4"

# Derive macros (#[derive(Queryable)], #[derive(QueryableFrom)])
rinq-derive = "4"

# LINQ-style query! macro (experimental)
rinq-syntax = "4"
```

---

## Quick Start

```rust
use rinq::QueryBuilder;

let total: i32 = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    .where_(|x| x % 2 == 0)   // keep evens: 2 4 6 8 10
    .order_by(|x| *x)          // sort ascending
    .sum();                    // terminal — evaluates the pipeline

assert_eq!(total, 30);
```

---

## Feature Flags

| Feature | What it enables | Dependency added |
|---|---|---|
| *(none)* | Core query engine | — |
| `parallel` | [`ParallelQueryBuilder`] via rayon | `rayon` |
| `serde` | [`QueryBuilder::from_json`] | `serde`, `serde_json` |

---

## Type-State Machine

Every `QueryBuilder<T, State>` carries a compile-time state. Invalid method
chains (e.g. `order_by` → `where_`) are rejected at compile time.

```
Initial
  ├─ where_() / take() / skip() / flat_map() …  →  Filtered
  │     ├─ select()                              →  Projected<U>  →  collect()
  │     └─ order_by() / order_by_descending()   →  Sorted
  │           └─ then_by() / then_by_desc()     →  Sorted
  └─ order_by()                                 →  Sorted
```

---

## All Operators

| Category | Methods |
|---|---|
| Filtering | `where_`, `take`, `skip`, `take_while`, `skip_while` |
| Transformation | `select`, `inspect`, `flat_map` |
| Sorting | `order_by`, `order_by_descending`, `then_by`, `then_by_descending` |
| Scalar aggregation | `count`, `sum`, `average`, `min`, `max`, `min_by`, `max_by` |
| Fold / reduce | `aggregate`, `aggregate_no_seed` |
| Collection aggregation | `group_by`, `group_by_aggregate`, `partition`, `to_hashmap`, `to_lookup` |
| Terminal | `first`, `last`, `first_or_default`, `last_or_default`, `any`, `all`, `contains`, `collect` |
| Single-element | `single`, `single_or_default`, `element_at` |
| Sequence | `distinct`, `distinct_by`, `reverse`, `chunk`, `window`, `zip`, `enumerate` |
| Set operations | `concat`, `union`, `intersect`, `except` |
| Generation | `QueryBuilder::range`, `QueryBuilder::repeat`, `QueryBuilder::empty` |
| Window analytics | `running_sum`, `running_average`, `moving_average`, `rank_by`, `dense_rank_by`, `lag`, `lead` |
| Failure-tolerant | `try_select`, `try_where_` → `TryQueryBuilder` |
| Parallel (`parallel`) | `into_parallel` → `ParallelQueryBuilder` |
| Serde (`serde`) | `from_json`, `from_json_value` |
| **v4 — Quick wins** | `filter_map`, `map`, `collect_vec`, `step_by`, `cycle` |
| **v4 — Functional** | `scan`, `chunk_by`, `dedup`, `dedup_by`, `zip_with`, `pairwise`, `intersperse`, `min_max` |
| **v4 — Generation** | `unfold`, `unfold_bounded` |
| **v4 — Lifecycle** | `from_arc_cloned`, `from_arc_slice_cloned`, `tap_each`, `tap_collect`, `pipe` |
| **v4 — Trait** | `IntoQuery` — `collection.into_query()` |

---

## Example: Window Analytics

```rust
use rinq::QueryBuilder;

// Running sum
let sums: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .running_sum()
    .collect();
assert_eq!(sums, vec![1, 3, 6, 10, 15]);

// Moving average (window = 3)
let avgs: Vec<Option<f64>> = QueryBuilder::from(vec![1.0, 2.0, 3.0, 4.0, 5.0])
    .moving_average(3)
    .collect();
// None, None, Some(2.0), Some(3.0), Some(4.0)
```

## Example: Parallel Execution

```rust
#[cfg(feature = "parallel")]
{
    use rinq::QueryBuilder;

    let total: i32 = QueryBuilder::from((1..=1_000_000).collect::<Vec<_>>())
        .into_parallel()
        .par_where(|x| x % 2 == 0)
        .par_sum();

    assert_eq!(total, 250_000_500_000_i64 as i32); // truncated for brevity
}
```

## Example: Serde

```rust
#[cfg(feature = "serde")]
{
    use rinq::QueryBuilder;

    let result: Vec<i32> = QueryBuilder::from_json("[1, 2, 3, 4, 5]")
        .unwrap()
        .where_(|x| *x > 2)
        .collect();

    assert_eq!(result, vec![3, 4, 5]);
}
```

---

## rinq-stats

The companion crate `rinq-stats` extends any `QueryBuilder` with:

| Module | Trait / Type | What it provides |
|---|---|---|
| B1 | `StatisticsExt` | `variance`, `std_dev`, `median`, `mode`, `percentile`, `skewness`, `kurtosis`, `histogram`, `frequency_table` |
| B2 | `QueryPair` | `covariance`, `pearson_correlation`, `spearman_correlation`, `kendall_tau`, `linear_regression` |
| B3 | `SamplingExt` | `sample_n`, `sample_fraction`, `stratified_sample`, `bootstrap_sample` |
| B4 | `ValidationExt` | Chainable `validate(pred, rule, message)` → `collect_validated` / `collect_valid` / `collect_invalid` |

```rust
use rinq::QueryBuilder;
use rinq_stats::{StatisticsExt, ValidationExt};

let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
let p95 = QueryBuilder::from(data).percentile(95);

let result = QueryBuilder::from(vec![1.0_f64, -2.0, 3.0])
    .validate(|x| *x > 0.0, "positive", "must be positive")
    .collect_validated();
// Err([ValidationError { rule: "positive", index: 1, .. }])
```

---

## Metrics Integration

```rust
use rinq::{MetricsQueryBuilder, MetricsCollector, QueryBuilder};
use std::sync::Arc;

let metrics = Arc::new(MetricsCollector::new());

let result: Vec<_> = MetricsQueryBuilder::new(
    QueryBuilder::from(vec![1, 2, 3, 4, 5]),
    metrics.clone(),
    "my_query".to_string(),
)
.where_(|x| *x > 2)
.collect();

assert_eq!(metrics.get("query_my_query"), Some(1));
```

---

## Testing

```sh
cargo test                           # all tests (rinq + doc tests)
cargo test --features parallel,serde # with optional features
cargo test -p rinq-stats             # stats crate tests
cargo bench --no-run                 # compile benchmarks
cargo clippy --all-features -- -D warnings
cargo doc --no-deps --all-features
```

---

## What's New in v4.0

### Type Aliases

```rust
use rinq::{InitialQuery, FilteredQuery, SortedQuery, ProjectedQuery};

fn get_adults(users: Vec<u32>) -> FilteredQuery<u32> {
    QueryBuilder::from(users).where_(|&age| age >= 18)
}
```

### New Operators (v4)

```rust
use rinq::{QueryBuilder, IntoQuery};

// IntoQuery — call .into_query() on any Vec
let result: Vec<i32> = vec![1, 2, 3].into_query().where_(|x| *x > 1).collect();

// scan — cumulative fold yielding each step
let running: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .scan(0, |acc, x| acc + x)
    .collect();
// [1, 3, 6, 10, 15]

// filter_map — map + filter in one pass
let nums: Vec<i32> = QueryBuilder::from(vec!["1", "two", "3"])
    .filter_map(|s| s.parse::<i32>().ok())
    .collect();
// [1, 3]

// pairwise — consecutive pairs
let pairs: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2, 3, 4])
    .pairwise()
    .collect();
// [(1,2), (2,3), (3,4)]

// pred! macro — inline predicate shorthand
use rinq::pred;
let result: Vec<_> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .where_(pred!(x > 2))
    .collect();
```

### `rinq-derive` — Derive Macros

```rust
use rinq::QueryBuilder;
use rinq_derive::{Queryable, QueryableFrom};

#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub active: bool,
    #[queryable(key)]
    pub department: String,
}

#[derive(Debug, Clone, QueryableFrom)]
pub struct UserList(Vec<User>);

// Generated accessor functions and typed predicate module:
use user_fields::{Active, Age};

let adults: Vec<User> = QueryBuilder::from(users)
    .where_(Age::gt(18))
    .where_(Active::is_true())
    .order_by(User::by_age)
    .collect();

// QueryableFrom enables .into_query():
use rinq::IntoQuery;
let list = UserList(users);
let result: Vec<_> = list.into_query().where_(Age::between(20, 40)).collect();
```

### `rinq-syntax` — LINQ-style `query!` Macro *(experimental)*

> The query syntax may change in future minor versions.

```rust
use rinq_syntax::query;

#[derive(Debug, Clone)]
struct User { name: String, age: u32, active: bool }

let adults: Vec<String> = query! {
    from user in users
    where user.age >= 18
    where user.active
    order_by user.age
    select user.name
};
```

### `rinq-stats` v4 Extensions

```rust
use rinq_stats::{TimeSeriesExt, OutlierExt, ValidationExt};

// Exponential Moving Average
let ema = QueryBuilder::from(prices).exponential_moving_average(0.2);

// Bollinger Bands
let bands = QueryBuilder::from(prices).bollinger_bands(20, 2.0);

// Remove outliers
let clean = QueryBuilder::from(data).remove_outliers_zscore(2.5);
let clean = QueryBuilder::from(data).remove_outliers_iqr();

// Conditional validation / dynamic error messages
let result = QueryBuilder::from(records)
    .validate(|r| !r.name.is_empty(), "name_required", "name is required")
    .validate_if(strict_mode, |r| r.value > 0.0, "positive", "must be positive")
    .validate_with(|r| r.value < 1000.0, "range", |r| format!("{} exceeds limit", r.value))
    .collect_validated();
```

---

## License

MIT License

## Author

Yoshimura Hisanori
