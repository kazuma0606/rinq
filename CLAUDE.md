# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build
cargo build --release

# Test all
cargo test

# Test a specific integration test file
cargo test --test core_tests
cargo test --test rinq_property_tests
cargo test --test rinq_v0_2_tests
cargo test --test rinq_immutability_test
cargo test --test metrics_tests

# Run a single test by name
cargo test <test_name>

# Doc tests only
cargo test --doc

# Compile benchmarks (without running)
cargo bench --no-run

# Run benchmarks
cargo bench

# Lint (zero warnings enforced)
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Architecture

This is the **`rinq`** crate — a type-safe, zero-cost query engine for Rust inspired by C# LINQ.

### Module Structure

```
src/
  lib.rs          — flat re-exports of public API
  core/
    builder.rs    — QueryBuilder<T, State> + Queryable trait + all operations + doc tests
    error.rs      — RinqError, RinqResult
    state.rs      — type-state markers: Initial, Filtered, Projected, Sorted
  metrics/
    builder.rs    — MetricsQueryBuilder<T, State> + doc tests
    collector.rs  — MetricsCollector (parking_lot::RwLock-based counter map)
```

### Public API Entry Points

```rust
use rinq::{QueryBuilder, Queryable, RinqError, RinqResult};
use rinq::{MetricsQueryBuilder, MetricsCollector};
// Also accessible via submodule paths:
use rinq::core::builder::QueryBuilder;
use rinq::metrics::MetricsCollector;
```

### Key Design Patterns

- **Type State Pattern** (`state.rs`): Compile-time enforcement of valid query operation order. States: `Initial` → `Filtered` → `Sorted` / `Projected<U>`. Methods only exist on the appropriate state type, preventing invalid chains at compile time.
- **`QueryBuilder<T, State>`**: Fluent, lazy iterator wrapper. Nothing executes until a terminal operation (`collect()`, `count()`, `first()`, `sum()`, etc.). Backed by `QueryData<T>` enum (`Iterator` or `SortedVec`).
- **`MetricsQueryBuilder<T, State>`**: Wraps `QueryBuilder`, recording per-query execution counts in `MetricsCollector` on each terminal operation. Metric key format: `query_<name>` (collect) / `query_<name>_<op>` (count/first/last/etc.).

### All Implemented Operations

| Category | Methods |
|---|---|
| Filtering | `where_`, `take`, `skip` |
| Transformation | `select`, `inspect` |
| Sorting | `order_by`, `then_by` |
| Scalar aggregation | `count`, `sum`, `average`, `min`, `max`, `min_by`, `max_by` |
| Collection aggregation | `group_by`, `partition` |
| Terminal | `first`, `last`, `any`, `all`, `collect` |
| Sequence | `distinct`, `reverse`, `chunk`, `window`, `zip` |

### Testing Notes

- Integration tests are in `tests/` and import from the `rinq` crate directly.
- Property-based tests use `proptest` — they run many random iterations and may be slow.
- Proptest regression files (`.proptest-regressions`) in `tests/` replay previously found failures; commit them when they appear.
- Unit tests and doc tests live inside `src/core/builder.rs` and `src/metrics/builder.rs`.
- Use saturating arithmetic (`saturating_mul`, `saturating_neg`) in proptest closures to avoid overflow panics on random `i32` inputs.

## Repository Layout (non-code)

| Path | Contents |
|---|---|
| `versions/v1/` | v1.0 spec (`spec.md`), implementation plan (`plan.md`), task checklist (`tasks.md`) |
| `docs/implementation.md` | Future roadmap: Phase 2 Join → Parallel → Async → WASM → Macro |
| `idea/` | Unstructured future ideas and API proposals |
| `examples/` | Runnable usage examples (`rinq_basic_usage.rs`) |
| `benches/` | Criterion benchmarks (`rinq_benchmarks.rs`, `rinq_v0_2_benchmarks.rs`) |
| `CHANGELOG.md` | Version history (v0.1 → v0.2 → v1.0) |
