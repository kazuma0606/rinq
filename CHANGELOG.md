# Changelog

All notable changes to RINQ (Rust Integrated Query) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v3.0.0] - 2026-03-25

### Breaking Changes

None. All v2.0 public API is preserved.

### Added

#### Phase A1 — Parallel execution (`parallel` feature)
- `QueryBuilder::into_parallel()` — converts any builder state into a `ParallelQueryBuilder<T, State>` backed by [rayon](https://docs.rs/rayon).
- `ParallelQueryBuilder` with: `par_where`, `par_select`, `par_flat_map`, `par_order_by`, `par_count`, `par_sum`, `par_min`, `par_max`, `par_any`, `par_all`, `par_group_by`, `collect`.
- New Cargo feature flag: `parallel`.

#### Phase A2 — Window analytics
- `running_sum()` — cumulative sum over a numeric sequence.
- `running_average()` — cumulative mean (produces `f64`).
- `moving_average(window)` — sliding-window mean; `None` until window fills (`Option<f64>` output).
- `rank_by(key)` — standard competition rank (1-based, gaps on ties).
- `dense_rank_by(key)` — dense rank (no gaps on ties).
- `lag(n)` — pair each element with its predecessor `n` positions back (`Option<T>`).
- `lead(n)` — pair each element with its successor `n` positions ahead (`Option<T>`).

#### Phase A3 — Failure-tolerant pipelines
- `TryQueryBuilder<T, E>` — wraps `Box<dyn Iterator<Item = Result<T, E>>>`.
- `QueryBuilder::try_select(f)` — fallible map; errors flow into the stream.
- `QueryBuilder::try_where_(f)` — fallible filter; `Ok(true)` keep, `Ok(false)` drop, `Err(e)` error.
- `TryQueryBuilder::collect_partitioned()` — returns `(Vec<T>, Vec<E>)` collecting all.
- `TryQueryBuilder::collect_results()` — returns `Result<Vec<T>, E>`, short-circuiting on first error.

#### Phase A4 — Serde integration (`serde` feature)
- `QueryBuilder::from_json(json: &str)` — deserialise a JSON array into a query pipeline.
- `QueryBuilder::from_json_value(json: &str)` — deserialise into `QueryBuilder<serde_json::Value, Initial>`.
- `use rinq::serde::QueryBuilder` import path for serde-oriented usage.
- New Cargo feature flag: `serde`.

#### Phase B1–B3 — `rinq-stats` companion crate
New crate `rinq-stats = "3.0"` providing statistical extensions:

- **`StatisticsExt` trait** (B1): `variance`, `std_dev`, `median`, `mode`, `percentile`, `quantile`, `skewness`, `kurtosis`, `histogram`, `frequency_table`.
- **`QueryPair`** (B2): two-series analysis. `new` (truncating), `try_new` (strict), `from_builders`. Methods: `covariance`, `pearson_correlation`, `spearman_correlation`, `kendall_tau`, `linear_regression`.
- **`SamplingExt` trait** (B3): `sample_fraction`, `sample_n`, `stratified_sample`, `bootstrap_sample`. Reservoir sampling via Vitter's Algorithm R.

#### Phase B4 — Validation (`rinq-stats`)
- **`ValidationExt` trait**: `QueryBuilder::validate(predicate, rule, message)` opens a `ValidationQueryBuilder<T>`.
- **`ValidationQueryBuilder`**: chainable `.validate()` rules; terminals: `collect_validated()` (all violations collected), `collect_valid()` (passing items only), `collect_invalid()` (failing items with errors).
- **`ValidationError`**: `{ rule: String, message: String, index: usize }` — implements `Display`, `Clone`, `PartialEq`.

#### Phase C — Documentation and publishing
- Crate-level `//!` documentation with quick start, state machine table, and feature-flag reference.
- `#![warn(missing_docs)]` enabled on `rinq`.
- `RinqError` and all public type-state markers fully documented.
- `Cargo.toml` metadata: `description`, `license`, `repository`, `keywords`, `categories`, `readme`.
- `[package.metadata.docs.rs] all-features = true` in both crates.

## [v2.0.0] - 2026-03-25

### Breaking Changes

- **`RinqError::InvalidState` removed**: This variant was dead code (never constructed). Remove any match arms for `InvalidState`.
- **`RinqError::TypeMismatch` removed**: Conceptually invalid in statically-typed Rust. Remove any match arms for `TypeMismatch`.

### Added

#### High-Priority Operators (M2)
- `flat_map(f)` — Flatten nested iterables. `Initial/Filtered → Filtered`. 遅延ストリーミング。
- `take_while(pred)` — Take elements while predicate holds. `Initial/Filtered/Sorted → Filtered`. 遅延ストリーミング。
- `skip_while(pred)` — Skip elements while predicate holds. `Initial/Filtered/Sorted → Filtered`. 遅延ストリーミング。
- `contains(&value)` — Check if a value is present (`T: PartialEq`). 即時実行。
- `first_or_default()` — First element or `T::default()` (`T: Default`). 即時実行。
- `last_or_default()` — Last element or `T::default()` (`T: Default`). 即時実行。
- `single()` — The one element or error (0 → `IteratorExhausted`, 2+ → `ExecutionError`). 即時実行。
- `single_or_default()` — Like `single` but 0 elements returns `Ok(T::default())`. 即時実行。

#### Medium-Priority Operators (M3)
- `order_by_descending(key)` — Sort descending. `Initial/Filtered → Sorted`. 遅延非ストリーミング。
- `then_by_descending(key)` — Secondary sort descending. `Sorted → Sorted`. 遅延非ストリーミング。
- `aggregate(seed, f)` — Fold with seed value. 即時実行。
- `aggregate_no_seed(f)` — Fold without seed; returns `None` if empty. 即時実行。
- `concat(other)` — Chain another iterable. `→ Filtered`. 遅延ストリーミング。
- `union(other)` — Set union, deduplicating (`T: Hash + Eq + Clone`). `→ Filtered`. 遅延非ストリーミング。
- `intersect(other)` — Set intersection. `→ Filtered`. 遅延非ストリーミング。
- `except(other)` — Set difference (self minus other). `→ Filtered`. 遅延非ストリーミング。
- `to_hashmap(key_selector)` — Collect to `HashMap<K, T>`; `Err` on duplicate keys. 即時実行。
- `to_lookup(key_selector)` — Collect to `HashMap<K, Vec<T>>`; allows duplicate keys. 即時実行。
- `element_at(index)` — Element at index or `None`. 即時実行。

#### Generation Operators (M4)
- `QueryBuilder::range(iterable)` — Build from any `IntoIterator` (e.g. `0..10`, `1..=100`).
- `QueryBuilder::repeat(value, count)` — Repeat a value N times (`T: Clone`).
- `QueryBuilder::empty()` — Empty sequence.

#### MetricsQueryBuilder (M5)
- All M2–M4 operators forwarded to `MetricsQueryBuilder` for all states.
- Immediate operators (`contains`, `single`, `aggregate`, `to_hashmap`, etc.) record metrics.
- Generation operators `range`, `repeat`, `empty` available as static constructors.

### Changed

#### Internal: Module split (no public API change)
- `src/core/builder.rs` (2619 lines) → `src/core/builder/` subdirectory:
  - `mod.rs` — `QueryBuilder<T, State>` struct + `QueryData<T>` enum
  - `iterators.rs` — `ChunkIterator`, `WindowIterator`
  - `initial.rs` — `impl QueryBuilder<T, Initial>`
  - `filtered.rs` — `impl QueryBuilder<T, Filtered>`
  - `sorted.rs` — `impl QueryBuilder<T, Sorted>`
  - `shared.rs` — `impl<T, State> QueryBuilder<T, State>` (terminal + set ops)
  - `queryable.rs` — `Queryable` trait + 7 collection impls
- `src/metrics/builder.rs` → `src/metrics/builder/` subdirectory:
  - `mod.rs` — `MetricsQueryBuilder<T, State>` struct
  - `impl_.rs` — all 4 state impl blocks

### Migration Guide

```rust
// Remove match arms for deleted variants:
match err {
    RinqError::InvalidQuery { message } => { /* ... */ }
    RinqError::IteratorExhausted => { /* ... */ }
    RinqError::ExecutionError { message } => { /* ... */ }
    // RinqError::InvalidState   ← delete this arm
    // RinqError::TypeMismatch   ← delete this arm
}
```

---

## [v1.0.0] - 2026-03-24

### Breaking Changes

- **Crate renamed**: `rusted_ca` → `rinq`. Update `Cargo.toml` to `rinq = "1.0"` and all imports to `use rinq::`.
- **Error type renamed**: `RinqDomainError` → `RinqError`. Update all match arms and type annotations.
- **Import paths changed**: `rusted_ca::domain::rinq::QueryBuilder` → `rinq::QueryBuilder`; `rusted_ca::domain::rinq::query_builder::Queryable` → `rinq::Queryable`.

### Removed

- All web-application layers (presentation, application, infrastructure, domain entities).
- gRPC / protobuf support (`build.rs`, `proto/`).
- Docker Compose configuration.
- `ApplicationError` and its conversion from `RinqError`.
- `src/shared/`, `src/state/`, `src/main.rs`.

### Added

- `rinq::core::*` — pure query engine (`QueryBuilder`, `Queryable`, `RinqError`, `RinqResult`, state types).
- `rinq::metrics::*` — metrics integration (`MetricsQueryBuilder`, `MetricsCollector`).
- Flat re-exports at crate root: `use rinq::QueryBuilder`, `use rinq::MetricsCollector`, etc.
- `versions/v1/spec.md` — formal v1.0 specification.

---

## [v0.2.0] - 2026-03-08

### Added

#### Numeric Aggregations (User Story 1 - P1 MVP)
- `sum()` - Calculate the sum of all elements
  - Works with any type implementing `std::iter::Sum`
  - Terminal operation consuming the query builder
- `average()` - Calculate the average of all elements
  - Returns `Option<f64>` (`None` for empty collections)
  - Works with any type implementing `num_traits::ToPrimitive`
- `min()` - Find the minimum element
  - Returns `Option<T>` (`None` for empty collections)
  - **Optimization**: O(1) for `Sorted` state (returns first element)
- `max()` - Find the maximum element
  - Returns `Option<T>` (`None` for empty collections)
  - **Optimization**: O(1) for `Sorted` state (returns last element)
- `min_by(key_selector)` - Find element with minimum key value
  - Supports custom key extraction for complex types
- `max_by(key_selector)` - Find element with maximum key value
  - Supports custom key extraction for complex types

#### Grouping Operations (User Story 2 - P2)
- `group_by(key_selector)` - Group elements by a key function
  - Returns `HashMap<K, Vec<T>>` (terminal operation)
  - Preserves relative order of elements within each group
- `group_by_aggregate(key_selector, aggregator)` - Group and aggregate
  - Returns `HashMap<K, R>` where `R` is the aggregation result type
  - Enables per-group analytics (sum, count, average, etc.)

#### Deduplication (User Story 3 - P3)
- `distinct()` - Remove duplicate elements
  - Preserves first occurrence of each unique element
  - Returns `QueryBuilder<T, Filtered>` (non-terminal)
  - Requires `T: Eq + Hash + Clone`
- `distinct_by(key_selector)` - Remove duplicates based on a key function
  - Preserves first occurrence per unique key
  - Only the key needs to implement `Eq + Hash`, not the entire element

#### Sequence Transformations (User Story 4 - P4)
- `reverse()` - Reverse the iteration order
  - Returns `QueryBuilder<T, Filtered>` (non-terminal)
  - Materializes into a `Vec` for reversal
- `chunk(size)` - Divide elements into fixed-size chunks
  - Returns `QueryBuilder<Vec<T>, Filtered>`
  - Last chunk may contain fewer elements
  - Panics if `size` is 0
- `window(size)` - Create sliding windows over elements
  - Returns `QueryBuilder<Vec<T>, Filtered>`
  - Creates overlapping windows of size `size`
  - Requires `T: Clone` (elements appear in multiple windows)
  - Panics if `size` is 0

#### Collection Combinations (User Story 5 - P5)
- `zip(other)` - Pair this query with another iterable
  - Returns `QueryBuilder<(T, U), Filtered>`
  - Shortest-wins semantics (stops when either iterator exhausted)
- `enumerate()` - Add indices to elements
  - Returns `QueryBuilder<(usize, T), Filtered>`
  - Indices start at 0
- `partition(predicate)` - Split into two collections based on predicate
  - Returns `(Vec<T>, Vec<T>)` (terminal operation)
  - First vec contains elements satisfying predicate, second contains the rest

### Testing
- Added **86 new tests** for v0.2 features
  - Property-based tests using `proptest` for invariant verification
  - Unit tests for edge cases
  - Integration tests for v0.1/v0.2 composition
- **Total test count**: 201+ tests (v0.1 + v0.2 combined)
- All tests passing with 100% success rate

### Performance
- Added comprehensive benchmarks comparing RINQ operations to manual implementations
- Benchmarks validate zero-cost abstraction principle:
  - Numeric aggregations: ≤5% overhead
  - Grouping operations: ~10% overhead (within acceptable range)
  - Deduplication: Equivalent to manual HashSet operations
  - Sequence transformations: Comparable to stdlib operations
  - Complex chains: ~15% overhead vs. optimized manual code
- All v0.2 methods use `#[inline]` attribute for compiler optimization

### Documentation
- Added comprehensive doc comments for all new methods
  - Usage examples for each operation
  - Trait bound documentation
  - Edge case notes (panics, empty collections)
- Updated `src/domain/rinq/README.md` with v0.2 feature showcase
- Added 25 runnable doc tests (all passing)

### Integration
- Extended `MetricsQueryBuilder` to support all v0.2 operations
  - Terminal operations record execution metrics
  - Non-terminal operations preserve metrics context
- All v0.2 methods integrate seamlessly with v0.1 operations
- Public API exports all new functionality via `rusted_ca::domain::rinq`

### Breaking Changes
**None** - v0.2 is fully backwards compatible with v0.1. All existing code continues to work without modifications.

### Migration Guide
No migration needed! v0.2 adds new optional methods. Your existing v0.1 code will continue to work exactly as before.

```rust
// v0.1 code - still works perfectly
let result: Vec<i32> = QueryBuilder::from(data)
    .where_(|x| *x > 5)
    .order_by(|x| *x)
    .collect();

// v0.2 enhancement - add aggregations, deduplication, etc.
let total: i32 = QueryBuilder::from(data)
    .where_(|x| *x > 5)
    .distinct()        // NEW in v0.2
    .sum();            // NEW in v0.2
```

---

## [v0.1.0] - 2024

### Initial Release
- Type-safe QueryBuilder with type state pattern
- Filtering (`where_`)
- Projection (`select`)
- Sorting (`order_by`, `then_by`)
- Pagination (`take`, `skip`)
- Aggregations (`count`, `first`, `last`, `any`, `all`)
- Terminal operations (`collect`)
- Debugging (`inspect`)
- Metrics integration (`MetricsQueryBuilder`)
- Property-based testing with proptest
- Zero-cost abstraction validation
