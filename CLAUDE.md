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
```

No linting configuration is present; use `cargo clippy` and `cargo fmt` as standard.

## Architecture

This is the **`rinq`** crate — a type-safe, zero-cost query engine for Rust inspired by C# LINQ.

### Module Structure

```
src/
  lib.rs          — flat re-exports of public API
  core/
    builder.rs    — QueryBuilder<T, State> + Queryable trait
    error.rs      — RinqError, RinqResult
    state.rs      — type-state markers: Initial, Filtered, Projected, Sorted
  metrics/
    builder.rs    — MetricsQueryBuilder<T, State>
    collector.rs  — MetricsCollector (parking_lot::RwLock-based counter map)
```

### Public API Entry Points

All types are available from the crate root:

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
- **`MetricsQueryBuilder<T, State>`**: Wraps `QueryBuilder`, recording per-query execution counts in `MetricsCollector` on each terminal operation.

### v0.2 Aggregation / Collection Operations

`sum()`, `average()`, `min()`/`max()`, `min_by()`/`max_by()`, `group_by()`, `distinct()`, `reverse()`, `chunk()`, `window()`, `zip()`, `partition()`, `inspect()`.

### Testing Notes

- Integration tests are in `tests/` and import from the `rinq` crate directly.
- Property-based tests use `proptest` — they run many random iterations and may be slow.
- Proptest regression files (`.proptest-regressions`) in `tests/` replay previously found failures.
- Unit and doc tests live inside `src/core/builder.rs` and `src/metrics/builder.rs`.
- `versions/v1/` contains the v1.0 specification (`spec.md`), implementation plan (`plan.md`), and task checklist (`tasks.md`).
