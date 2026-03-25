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
rinq = "3"

# Optional: parallel execution (rayon) and/or serde JSON support
rinq = { version = "3", features = ["parallel", "serde"] }

# Statistical extensions (separate crate)
rinq-stats = "3"
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

## License

MIT License

## Author

Yoshimura Hisanori
