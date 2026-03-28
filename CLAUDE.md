# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build --workspace
cargo build --workspace --release

# Test all crates
cargo test --workspace

# Test with all features
cargo test --workspace --all-features

# Test a specific crate
cargo test -p rinq
cargo test -p rinq-stats
cargo test -p rinq-derive
cargo test -p rinq-syntax

# Test a specific integration test file (run from workspace root)
cargo test --test core_tests
cargo test --test rinq_property_tests
cargo test --test rinq_v0_2_tests
cargo test --test rinq_immutability_test
cargo test --test metrics_tests
cargo test --test rinq_v4_tests
cargo test --test rinq_window_tests
cargo test --test rinq_parallel_tests
cargo test --test rinq_serde_tests
cargo test --test rinq_try_tests

# Run a single test by name
cargo test <test_name>

# Doc tests only
cargo test --doc

# Compile benchmarks (without running)
cargo bench --no-run --workspace

# Run benchmarks
cargo bench

# Lint (zero warnings enforced)
cargo clippy --workspace --all-features -- -D warnings

# Format
cargo fmt --all

# Format check (used in CI)
cargo fmt --all --check

# Generate docs
cargo doc --no-deps --all-features --workspace
```

## Architecture

This is a **Cargo workspace** containing four crates:

| Crate | Path | Description |
|---|---|---|
| `rinq` | `rinq/` | Core query engine |
| `rinq-stats` | `rinq-stats/` | Statistical extensions |
| `rinq-derive` | `rinq-derive/` | Derive macros (`#[derive(Queryable)]`) |
| `rinq-syntax` | `rinq-syntax/` | `query!` macro (experimental) |

---

## rinq — Core Crate

### Module Structure

```
rinq/src/
  lib.rs                    — flat re-exports of public API; type aliases
  __macro_support.rs        — stable ABI for rinq-syntax generated code
  macros/
    mod.rs                  — rinq_explain! / pred! macro_rules! definitions
  core/
    mod.rs
    builder/
      mod.rs                — QueryBuilder<T, State> struct + QueryData<T> enum
      iterators.rs          — ChunkIterator, WindowIterator, ChunkByIterator,
                              UnfoldIter, UnfoldBoundedIter (pub(crate))
      initial.rs            — impl QueryBuilder<T, Initial>
      filtered.rs           — impl QueryBuilder<T, Filtered>
      sorted.rs             — impl QueryBuilder<T, Sorted>
      shared.rs             — impl<T, State> QueryBuilder<T, State>
                              (terminal ops, set ops, lifecycle, functional ops)
      functional.rs         — scan, chunk_by, dedup, dedup_by, zip_with,
                              pairwise, unfold, intersperse, min_max
      queryable.rs          — Queryable trait + collection impls
      window.rs             — running_sum, moving_average, rank_by, lag, lead
      try_ops.rs            — TryQueryBuilder
      serde_ops.rs          — from_json (serde feature)
    error.rs                — RinqError, RinqResult
    state.rs                — type-state markers: Initial, Filtered, Projected, Sorted
    state_diagnostics.rs    — #[diagnostic::on_unimplemented] traits
  metrics/
    builder/
      mod.rs                — MetricsQueryBuilder<T, State> struct
      impl_.rs              — all state impl blocks + doc tests
    collector.rs            — MetricsCollector (parking_lot::RwLock-based counter map)
  parallel/                 — ParallelQueryBuilder (rayon feature)
  serde/                    — Serde integration (serde feature)
```

### Public API Entry Points

```rust
use rinq::{QueryBuilder, Queryable, IntoQuery, RinqError, RinqResult};
use rinq::{InitialQuery, FilteredQuery, SortedQuery, ProjectedQuery};
use rinq::{MetricsQueryBuilder, MetricsCollector};
use rinq::TryQueryBuilder;
#[cfg(feature = "parallel")]
use rinq::ParallelQueryBuilder;
```

### Key Design Patterns

- **Type State Pattern** (`state.rs`): Compile-time enforcement of valid query operation order. States: `Initial` → `Filtered` → `Sorted` / `Projected<U>`. Methods only exist on the appropriate state type, preventing invalid chains at compile time.
- **`QueryBuilder<T, State>`**: Fluent, lazy iterator wrapper. Nothing executes until a terminal operation (`collect()`, `count()`, `first()`, `sum()`, etc.). Backed by `QueryData<T>` enum (`Iterator` or `SortedVec`).
- **`MetricsQueryBuilder<T, State>`**: Wraps `QueryBuilder`, recording per-query execution counts in `MetricsCollector` on each terminal operation. Metric key format: `query_<name>` (collect) / `query_<name>_<op>` (count/first/last/etc.).
- **`Filtered` state semantics**: `Filtered` is a "chainable intermediate state", not "already-filtered". Operators like `scan`, `pairwise`, `step_by` return `Filtered` because they produce a chainable result, not because they filter.

### All Implemented Operations

| Category | Methods |
|---|---|
| Filtering | `where_`, `take`, `skip`, `take_while`, `skip_while`, `step_by`, `filter_map` |
| Transformation | `select` / `map`, `flat_map`, `flatten`, `inspect`, `scan`, `zip`, `zip_with`, `enumerate`, `cycle` |
| Sorting | `order_by`, `order_by_descending`, `then_by`, `then_by_descending` |
| Dedup & Grouping | `distinct`, `distinct_by`, `dedup`, `dedup_by`, `chunk_by`, `group_by`, `group_by_aggregate`, `partition`, `frequencies` |
| Sequence | `reverse`, `chunk` / `batch`, `window`, `pairwise`, `intersperse`, `tee` |
| Set operations | `concat`, `union`, `intersect`, `except` |
| Scalar aggregation | `count`, `count_by`, `sum`, `sum_by`, `average`, `average_by`, `min`, `max`, `min_by`, `max_by`, `min_max`, `reduce`, `aggregate`, `aggregate_no_seed` |
| Terminal | `first` / `find`, `last`, `first_or_default`, `last_or_default`, `nth` / `element_at`, `any`, `all`, `none`, `all_unique`, `contains`, `single` / `exactly_one`, `single_or_default`, `collect`, `collect_vec`, `to_sorted_vec`, `to_sorted_vec_desc`, `take_last`, `skip_last`, `index_of`, `position`, `for_each` |
| Collection | `to_hashmap`, `to_lookup` |
| Window analytics | `running_sum`, `moving_average`, `rank_by`, `lag`, `lead` |
| Lifecycle | `tap_each`, `tap_collect`, `pipe`, `from_arc_cloned`, `from_arc_slice_cloned` |
| Generation | `QueryBuilder::range`, `QueryBuilder::repeat`, `QueryBuilder::empty`, `QueryBuilder::unfold`, `QueryBuilder::unfold_bounded` |
| DX macros | `rinq_explain!`, `pred!` |
| Type aliases | `InitialQuery<T>`, `FilteredQuery<T>`, `SortedQuery<T>`, `ProjectedQuery<U>` |

### Testing Notes

- Integration tests are in `rinq/tests/` and import from the `rinq` crate directly.
- Property-based tests use `proptest` — they run many random iterations and may be slow.
- Proptest regression files (`.proptest-regressions`) in `rinq/tests/` replay previously found failures; commit them when they appear.
- Unit tests and doc tests live inside `rinq/src/core/builder/*.rs` and `rinq/src/metrics/builder/impl_.rs`.
- Use saturating arithmetic (`saturating_mul`, `saturating_neg`) in proptest closures to avoid overflow panics on random `i32` inputs.
- When using `--all-features`, add type annotations to `.sum()` calls (e.g., `.sum::<i32>()`) to avoid ambiguity from serde feature.

---

## rinq-stats

Statistical extensions for `QueryBuilder`. Traits are implemented on `QueryBuilder<f64, State>` (or generic `T` where noted).

### Module Structure

```
rinq-stats/src/
  lib.rs          — re-exports
  statistics.rs   — StatisticsExt: mean, variance, std_dev, median, percentile,
                    skewness, kurtosis, histogram, correlation_with, linear_regression
  sampling.rs     — SamplingExt: sample_n, sample_fraction, stratified_sample
  validation.rs   — ValidationExt + ValidationQueryBuilder: validate, validate_if, validate_with
  timeseries.rs   — TimeSeriesExt: exponential_moving_average, bollinger_bands
  outliers.rs     — OutlierExt: remove_outliers_zscore, remove_outliers_iqr
```

### Tests

```
rinq-stats/tests/
  core_stats_tests.rs
  sampling_tests.rs
  validation_tests.rs
  timeseries_tests.rs
  outlier_tests.rs
```

---

## rinq-derive

Proc-macro crate. Generates field accessors and typed predicates via `#[derive(Queryable)]`.

### Module Structure

```
rinq-derive/src/
  lib.rs          — entry points: derive(Queryable), derive(QueryableFrom)
  queryable.rs    — #[derive(Queryable)] expansion logic
  from.rs         — #[derive(QueryableFrom)] expansion logic
```

---

## rinq-syntax

Proc-macro crate (experimental). Provides the `query!` macro.

### Module Structure

```
rinq-syntax/src/
  lib.rs          — #[proc_macro] pub fn query(input: TokenStream) -> TokenStream
  ast.rs          — QueryInput, Clause, SortKey
  parser.rs       — clause parsers (from/where/order_by/select/take/skip)
  codegen.rs      — TokenStream generation from QueryInput
```

The macro expands to a `::rinq::__macro_support::from(source)` call followed by a chain of `QueryBuilder` method calls, always terminating with `.collect::<Vec<_>>()`.

---

## Repository Layout

| Path | Contents |
|---|---|
| `rinq/` | Core crate (src, tests, benches, examples) |
| `rinq-stats/` | Statistical extensions crate |
| `rinq-derive/` | Derive macro crate |
| `rinq-syntax/` | Query macro crate (experimental) |
| `versions/v1/` | v1.0 spec, plan, tasks |
| `versions/v2/` | v2.0 spec, plan, tasks |
| `versions/v3/` | v3.0 spec, plan, tasks |
| `versions/v4/` | v4.0 spec, plan, tasks, tests, AI discussion docs |
| `versions/v5/` | v5.0 spec, plan, tasks, tests (current) |
| `docs/` | Implementation roadmap and notes |
| `idea/` | Unstructured future ideas and API proposals |
| `CHANGELOG.md` | Version history |
