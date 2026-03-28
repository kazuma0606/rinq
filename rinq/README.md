# rinq

[![CI](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml/badge.svg)](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rinq.svg)](https://crates.io/crates/rinq)
[![docs.rs](https://docs.rs/rinq/badge.svg)](https://docs.rs/rinq)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Type-safe, zero-cost LINQ-inspired query engine for Rust.**

rinq lets you compose filter → sort → aggregate pipelines over any in-memory collection using a fluent builder API. The type-state pattern encodes the valid operation order at compile time, so invalid chains are rejected without runtime overhead.

## Quick Start

```toml
[dependencies]
rinq = "0.1"
```

```rust
use rinq::QueryBuilder;

let total: i32 = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    .where_(|x| x % 2 == 0)   // keep evens
    .order_by(|x| *x)          // sort ascending
    .sum();                    // terminal — evaluates the pipeline

assert_eq!(total, 30);
```

## Feature Flags

| Feature | What it enables |
|---|---|
| `parallel` | [`ParallelQueryBuilder`] via [rayon](https://docs.rs/rayon) |
| `serde` | [`QueryBuilder::from_json`] via [serde_json](https://docs.rs/serde_json) |

```toml
rinq = { version = "0.1", features = ["parallel", "serde"] }
```

## State Machine

Every `QueryBuilder<T, State>` carries a compile-time state that restricts which methods are available:

| State | Available transitions |
|---|---|
| `Initial` | `where_`, `take`, `skip`, `flat_map`, `order_by`, `group_by`, … |
| `Filtered` | same as `Initial` plus `select` / `map` |
| `Sorted` | `then_by`, `then_by_descending`, plus all terminal ops |
| `Projected<U>` | `collect()` only |

## Operator Reference

### Filtering
`where_` · `take` · `skip` · `take_while` · `skip_while` · `step_by` · `filter_map`

### Transformation
`select` / `map` · `flat_map` · `flatten` · `inspect` · `scan` · `zip` · `zip_with` · `enumerate` · `cycle`

### Sorting
`order_by` · `order_by_descending` · `then_by` · `then_by_descending`

### Deduplication & Grouping
`distinct` · `distinct_by` · `dedup` · `dedup_by` · `chunk_by` · `group_by` · `group_by_aggregate` · `partition` · `frequencies`

### Sequence
`reverse` · `chunk` · `window` · `pairwise` · `intersperse` · `tee`

### Set Operations
`concat` · `union` · `intersect` · `except`

### Scalar Aggregation
`count` · `count_by` · `sum` · `sum_by` · `average` · `average_by` · `min` · `max` · `min_by` · `max_by` · `min_max` · `reduce` · `aggregate` · `aggregate_no_seed`

### Terminal
`first` · `find` · `last` · `first_or_default` · `last_or_default` · `nth` · `element_at` · `any` · `all` · `none` · `all_unique` · `contains` · `single` · `exactly_one` · `single_or_default` · `collect` · `collect_vec` · `to_sorted_vec` · `to_sorted_vec_desc` · `take_last` · `skip_last` · `index_of` · `position` · `for_each`

### Lifecycle & Utilities
`tap_each` · `tap_collect` · `pipe` · `from_arc_cloned` · `from_arc_slice_cloned`

### Generation
`QueryBuilder::range` · `QueryBuilder::repeat` · `QueryBuilder::empty` · `QueryBuilder::unfold` · `QueryBuilder::unfold_bounded`

### Collection
`to_hashmap` · `to_lookup`

## Sub-crates

| Crate | Description |
|---|---|
| [`rinq-stats`](https://crates.io/crates/rinq-stats) | Descriptive statistics, sampling, validation, time series, outlier detection |
| [`rinq-derive`](https://crates.io/crates/rinq-derive) | `#[derive(Queryable)]` — auto-generate field accessors and typed predicates |
| [`rinq-syntax`](https://crates.io/crates/rinq-syntax) | `query!` macro — LINQ-style query syntax (experimental) |

## Development Process

This crate is developed with AI-assisted design. Internal planning documents
(`versions/`) are written in Japanese — the development log of how rinq
grew from v1 to v5.

## License

MIT — see [LICENSE](../LICENSE)
