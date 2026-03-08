# Tasks: RINQ v0.2 - Aggregation and Transformation Extensions

**Input**: Design documents from `.specify/specs/002-rinq-v0.2-aggregation/`  
**Prerequisites**: ✅ plan.md, ✅ spec.md, ✅ research.md, ✅ data-model.md, ✅ quickstart.md

**Branch**: `002-rinq-v0.2-aggregation`  
**Constitution**: `.specify/memory/constitution.md`

**Tests**: Property-based tests (proptest) and unit tests are MANDATORY per Constitution Principle V.

**Organization**: Tasks grouped by user story priority (P1-P5) to enable incremental delivery.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story this task belongs to (US1=P1, US2=P2, etc.)
- File paths relative to repository root (`C:\Users\yoshi\rusted-ca\`)

---

## Phase 0: Preparation (Prerequisites)

**Purpose**: Set up dependencies and verify baseline

- [ ] **T001** Add `num-traits = "0.2"` to `[dependencies]` in `Cargo.toml`
- [ ] **T002** Add `rinq_v0.2_benchmarks` to `[[bench]]` section in `Cargo.toml`
- [ ] **T003** Run `cargo check` to verify dependency resolution
- [ ] **T004** Verify all existing v0.1 tests pass: `cargo test` (baseline: 115+ tests)

**Checkpoint**: Dependencies ready, v0.1 tests still passing (backwards compatibility baseline)

---

## Phase 1: User Story 1 - Numeric Aggregations (Priority: P1) 🎯 MVP

**Goal**: Provide `sum()`, `average()`, `min()`, `max()`, `min_by()`, `max_by()` for immediate data analysis value

**Independent Test**: Create numeric collection, call aggregation methods, verify results against known values

### Tests for User Story 1 (Write Tests FIRST, Ensure They FAIL)

#### Property Tests: sum()

- [ ] **T101** [P] [US1] Property test: `sum()` correctness vs manual sum in `tests/rinq_v0.2_tests.rs`
  - Verify: `QueryBuilder::from(data).sum() == data.iter().sum()`
  - Test on: `Vec<i32>`, `Vec<f64>`, ranges

- [ ] **T102** [P] [US1] Property test: `sum()` on filtered data in `tests/rinq_v0.2_tests.rs`
  - Verify: `.where_().sum()` equals manual filter then sum
  
- [ ] **T103** [P] [US1] Unit test: `sum()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty collection → 0 (additive identity)
  - Single element → element value
  - Negative numbers

#### Property Tests: average()

- [ ] **T104** [P] [US1] Property test: `average()` correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: matches manual sum/count calculation
  - Test on: `Vec<i32>`, `Vec<f64>`

- [ ] **T105** [P] [US1] Unit test: `average()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty collection → `None`
  - Single element → `Some(element as f64)`
  - Large numbers (precision)

#### Property Tests: min() / max()

- [ ] **T106** [P] [US1] Property test: `min()` / `max()` correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: `min()` equals `Iterator::min()`
  - Verify: `max()` equals `Iterator::max()`
  - Test on: integers, floats, custom Ord types

- [ ] **T107** [P] [US1] Unit test: `min()` / `max()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty → `None`
  - Single element → `Some(element)`
  - All elements equal

#### Property Tests: min_by() / max_by()

- [ ] **T108** [P] [US1] Property test: `min_by()` / `max_by()` with key selector in `tests/rinq_v0.2_tests.rs`
  - Verify: returns element with min/max key value
  - Test with: structs with multiple fields, tuple keys

- [ ] **T109** [P] [US1] Unit test: `min_by()` / `max_by()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty → `None`
  - Multiple elements with same min/max key (returns first)

**Checkpoint**: All US1 tests written and FAILING (expected - no implementation yet)

---

### Implementation for User Story 1

#### sum() Implementation

- [ ] **T110** [US1] Implement `sum()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Add `use std::iter::Sum;` import
  - Handle `QueryData::Iterator` case
  - Handle `QueryData::SortedVec` case (iterate and sum)

- [ ] **T111** [US1] Implement `sum()` in `impl<T> QueryBuilder<T, Filtered>` in `src/domain/rinq/query_builder.rs`
  - Reuse logic from Initial state implementation

- [ ] **T112** [US1] Implement `sum()` in `impl<T> QueryBuilder<T, Sorted>` in `src/domain/rinq/query_builder.rs`
  - Reuse logic, handle SortedVec

- [ ] **T113** [US1] Implement `sum()` in `impl<U> QueryBuilder<U, Projected<U>>` in `src/domain/rinq/query_builder.rs`
  - Reuse logic for projected type

- [ ] **T114** [US1] Verify T101-T103 tests now PASS after implementation

#### average() Implementation

- [ ] **T115** [US1] Implement `average()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Add `use num_traits::ToPrimitive;` import
  - Collect items, check if empty → `None`
  - Convert to f64, sum, divide by count

- [ ] **T116** [US1] Implement `average()` for `Filtered`, `Sorted`, `Projected` states in `src/domain/rinq/query_builder.rs`
  - Duplicate implementation across states

- [ ] **T117** [US1] Verify T104-T105 tests now PASS

#### min() / max() Implementation

- [ ] **T118** [US1] Implement `min()` and `max()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Use `Iterator::min()` and `Iterator::max()`
  - **Optimization**: For `Sorted` state, `min()` = first element, `max()` = last element

- [ ] **T119** [US1] Implement `min()` and `max()` for other states in `src/domain/rinq/query_builder.rs`
  - Apply optimization for `Sorted` state

- [ ] **T120** [US1] Verify T106-T107 tests now PASS

#### min_by() / max_by() Implementation

- [ ] **T121** [US1] Implement `min_by()` and `max_by()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Use `Iterator::min_by_key()` and `Iterator::max_by_key()`

- [ ] **T122** [US1] Implement `min_by()` and `max_by()` for other states in `src/domain/rinq/query_builder.rs`

- [ ] **T123** [US1] Verify T108-T109 tests now PASS

**Checkpoint US1**: All numeric aggregation tests pass (T101-T109), implementation complete

---

## Phase 2: User Story 2 - Grouping Operations (Priority: P2)

**Goal**: Enable `group_by()` and `group_by_aggregate()` for categorization and per-group analytics

**Independent Test**: Create heterogeneous collection, group by key function, verify HashMap correctness

### Tests for User Story 2 (Write Tests FIRST)

#### Property Tests: group_by()

- [ ] **T201** [P] [US2] Property test: `group_by()` completeness in `tests/rinq_v0.2_tests.rs`
  - Verify: All input elements appear exactly once across all groups
  - `groups.values().flatten().collect() == original (unordered equality)`

- [ ] **T202** [P] [US2] Property test: `group_by()` determinism in `tests/rinq_v0.2_tests.rs`
  - Verify: Same input produces same grouping
  - Keys are consistent

- [ ] **T203** [P] [US2] Property test: `group_by()` order preservation in `tests/rinq_v0.2_tests.rs`
  - Verify: Within each group, relative order of elements preserved

- [ ] **T204** [P] [US2] Unit test: `group_by()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty collection → empty HashMap
  - All elements same key → single-entry HashMap
  - All unique keys → HashMap with single-element vectors

#### Property Tests: group_by_aggregate()

- [ ] **T205** [P] [US2] Property test: `group_by_aggregate()` correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: Aggregation applied correctly to each group
  - Compare with manual grouping then aggregating

- [ ] **T206** [P] [US2] Unit test: `group_by_aggregate()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty collection
  - Single group
  - Multiple aggregation functions (sum, count, max)

**Checkpoint**: All US2 tests written and FAILING

---

### Implementation for User Story 2

- [ ] **T210** [US2] Implement `group_by()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Add `use std::collections::HashMap;` and `use std::hash::Hash;` imports
  - Match on `QueryData`, iterate, use `HashMap::entry().or_insert_with()`
  - Terminal operation (consumes self, returns HashMap)

- [ ] **T211** [US2] Implement `group_by()` for `Filtered`, `Sorted`, `Projected` states in `src/domain/rinq/query_builder.rs`
  - Duplicate implementation logic

- [ ] **T212** [US2] Verify T201-T204 tests now PASS

- [ ] **T213** [US2] Implement `group_by_aggregate()` in all states in `src/domain/rinq/query_builder.rs`
  - Reuse `group_by()`, then transform values with aggregator
  - `groups.into_iter().map(|(k, v)| (k, aggregator(&v))).collect()`

- [ ] **T214** [US2] Verify T205-T206 tests now PASS

**Checkpoint US2**: All grouping tests pass, grouping functionality complete

---

## Phase 3: User Story 3 - Deduplication (Priority: P3)

**Goal**: Enable `distinct()` and `distinct_by()` for data cleaning workflows

**Independent Test**: Create collection with duplicates, call distinct, verify uniqueness

### Tests for User Story 3 (Write Tests FIRST)

#### Property Tests: distinct()

- [ ] **T301** [P] [US3] Property test: `distinct()` removes all duplicates in `tests/rinq_v0.2_tests.rs`
  - Verify: Result has no duplicates (all elements unique)
  - `result.iter().collect::<HashSet>().len() == result.len()`

- [ ] **T302** [P] [US3] Property test: `distinct()` subset property in `tests/rinq_v0.2_tests.rs`
  - Verify: `distinct(xs).len() <= xs.len()`
  - Verify: All elements in result exist in original

- [ ] **T303** [P] [US3] Property test: `distinct()` preserves first occurrence in `tests/rinq_v0.2_tests.rs`
  - Verify: First occurrence of each element is kept
  - Use indexed comparison

- [ ] **T304** [P] [US3] Unit test: `distinct()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty → empty
  - All unique → unchanged
  - All duplicates `[5,5,5,5]` → `[5]`

#### Property Tests: distinct_by()

- [ ] **T305** [P] [US3] Property test: `distinct_by()` key-based deduplication in `tests/rinq_v0.2_tests.rs`
  - Verify: No duplicate keys in result
  - Test with structs and custom key functions

- [ ] **T306** [P] [US3] Unit test: `distinct_by()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty, single element, all same key, all unique keys

**Checkpoint**: All US3 tests written and FAILING

---

### Implementation for User Story 3

- [ ] **T310** [US3] Implement `distinct()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Add `use std::collections::HashSet;` import
  - For `Iterator`: Use stateful filter with `HashSet` to track seen elements
  - For `SortedVec`: Iterate, deduplicate into Vec, wrap as Iterator
  - Return `QueryBuilder<T, Filtered>`

- [ ] **T311** [US3] Implement `distinct()` for `Filtered`, `Sorted`, `Projected` states in `src/domain/rinq/query_builder.rs`
  - Note: `Sorted` → `Filtered` transition preserves sorted order but loses sort metadata

- [ ] **T312** [US3] Verify T301-T304 tests now PASS

- [ ] **T313** [US3] Implement `distinct_by()` in all states in `src/domain/rinq/query_builder.rs`
  - Similar to `distinct()` but track keys instead of full elements
  - No `Clone` bound on `T` (only key is hashed)

- [ ] **T314** [US3] Verify T305-T306 tests now PASS

**Checkpoint US3**: All deduplication tests pass, distinct functionality complete

---

## Phase 4: User Story 4 - Sequence Transformations (Priority: P4)

**Goal**: Enable `reverse()`, `chunk()`, `window()` for batch processing and time-series analysis

**Independent Test**: Create sequential collection, apply transformations, verify structure/order

### Tests for User Story 4 (Write Tests FIRST)

#### Property Tests: reverse()

- [ ] **T401** [P] [US4] Property test: `reverse()` double reversal identity in `tests/rinq_v0.2_tests.rs`
  - Verify: `reverse(reverse(xs)) == xs`

- [ ] **T402** [P] [US4] Property test: `reverse()` order inversion in `tests/rinq_v0.2_tests.rs`
  - Verify: `reverse(xs)[i] == xs[len-1-i]`

- [ ] **T403** [P] [US4] Unit test: `reverse()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty, single element

#### Property Tests: chunk()

- [ ] **T404** [P] [US4] Property test: `chunk()` completeness in `tests/rinq_v0.2_tests.rs`
  - Verify: `chunks.flat_map().collect() == original`
  - All elements accounted for

- [ ] **T405** [P] [US4] Property test: `chunk()` size correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: All chunks except last have size = n
  - Last chunk has size ≤ n

- [ ] **T406** [P] [US4] Unit test: `chunk()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty → empty result
  - Length < chunk_size → single chunk
  - `chunk(0)` → panics with descriptive message

#### Property Tests: window()

- [ ] **T407** [P] [US4] Property test: `window()` count correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: `windows.len() == max(0, xs.len() - window_size + 1)`

- [ ] **T408** [P] [US4] Property test: `window()` overlap correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: Each element appears in `window_size` windows (except edge elements)

- [ ] **T409** [P] [US4] Unit test: `window()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Length < window_size → empty result
  - `window(0)` or `window(1)` → panics

**Checkpoint**: All US4 tests written and FAILING

---

### Implementation for User Story 4

#### reverse() Implementation

- [ ] **T410** [US4] Implement `reverse()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Collect iterator into Vec, call `.reverse()`, wrap as Iterator
  - Return `QueryBuilder<T, Filtered>`

- [ ] **T411** [US4] Implement `reverse()` for `Filtered`, `Sorted`, `Projected` states in `src/domain/rinq/query_builder.rs`

- [ ] **T412** [US4] Verify T401-T403 tests now PASS

#### chunk() Implementation

- [ ] **T413** [US4] Create `ChunkIterator<T, I>` helper struct in `src/domain/rinq/query_builder.rs`
  - Fields: `iter: I`, `size: usize`
  - Implement `Iterator` trait with `Item = Vec<T>`
  - Use buffer to collect `size` elements per iteration

- [ ] **T414** [US4] Implement `chunk()` in all states in `src/domain/rinq/query_builder.rs`
  - Assert `size > 0` (panic on invalid)
  - Wrap iterator with `ChunkIterator`
  - Return `QueryBuilder<Vec<T>, Filtered>`

- [ ] **T415** [US4] Verify T404-T406 tests now PASS

#### window() Implementation

- [ ] **T416** [US4] Create `WindowIterator<T>` helper struct in `src/domain/rinq/query_builder.rs`
  - Fields: `items: Vec<T>`, `size: usize`, `position: usize`
  - Implement `Iterator` trait with `Item = Vec<T>`
  - Clone elements for overlapping windows

- [ ] **T417** [US4] Implement `window()` in all states in `src/domain/rinq/query_builder.rs`
  - Assert `size >= 2` (panic on invalid)
  - Collect into Vec, create `WindowIterator`
  - Return `QueryBuilder<Vec<T>, Filtered>`
  - Trait bound: `T: Clone`

- [ ] **T418** [US4] Verify T407-T409 tests now PASS

**Checkpoint US4**: All sequence transformation tests pass, functionality complete

---

## Phase 5: User Story 5 - Collection Combinations (Priority: P5)

**Goal**: Enable `zip()`, `enumerate()`, `partition()` for data correlation and splitting

**Independent Test**: Combine collections, verify pairing and indexing correctness

### Tests for User Story 5 (Write Tests FIRST)

#### Property Tests: zip()

- [ ] **T501** [P] [US5] Property test: `zip()` shortest-wins semantics in `tests/rinq_v0.2_tests.rs`
  - Verify: `zip(xs, ys).len() == min(xs.len(), ys.len())`

- [ ] **T502** [P] [US5] Property test: `zip()` pairing correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: `zip(xs, ys)[i] == (xs[i], ys[i])`

- [ ] **T503** [P] [US5] Unit test: `zip()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Different lengths, empty collections

#### Property Tests: enumerate()

- [ ] **T504** [P] [US5] Property test: `enumerate()` index correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: Indices are 0, 1, 2, ..., n-1
  - Verify: `enumerate().map(|(i, _)| i).collect() == (0..len).collect()`

- [ ] **T505** [P] [US5] Property test: `enumerate()` after filtering in `tests/rinq_v0.2_tests.rs`
  - Verify: Indices reflect filtered sequence, not original positions

- [ ] **T506** [P] [US5] Unit test: `enumerate()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty → empty, single element → `[(0, elem)]`

#### Property Tests: partition()

- [ ] **T507** [P] [US5] Property test: `partition()` completeness in `tests/rinq_v0.2_tests.rs`
  - Verify: `trues.len() + falses.len() == original.len()`
  - All elements accounted for

- [ ] **T508** [P] [US5] Property test: `partition()` predicate correctness in `tests/rinq_v0.2_tests.rs`
  - Verify: All elements in first Vec satisfy predicate
  - All elements in second Vec don't satisfy predicate

- [ ] **T509** [P] [US5] Unit test: `partition()` edge cases in `tests/rinq_v0.2_tests.rs`
  - Empty → `([], [])`
  - All match → `(all, [])`
  - None match → `([], all)`

**Checkpoint**: All US5 tests written and FAILING

---

### Implementation for User Story 5

#### zip() Implementation

- [ ] **T510** [US5] Implement `zip()` in `impl<T> QueryBuilder<T, Initial>` in `src/domain/rinq/query_builder.rs`
  - Use `Iterator::zip()`
  - Return `QueryBuilder<(T, U), Filtered>`

- [ ] **T511** [US5] Implement `zip()` for other states in `src/domain/rinq/query_builder.rs`

- [ ] **T512** [US5] Verify T501-T503 tests now PASS

#### enumerate() Implementation

- [ ] **T513** [US5] Implement `enumerate()` in all states in `src/domain/rinq/query_builder.rs`
  - Use `Iterator::enumerate()`
  - Return `QueryBuilder<(usize, T), Filtered>`

- [ ] **T514** [US5] Verify T504-T506 tests now PASS

#### partition() Implementation

- [ ] **T515** [US5] Implement `partition()` in all states in `src/domain/rinq/query_builder.rs`
  - Use `Iterator::partition()`
  - Return `(Vec<T>, Vec<T>)` (terminal operation)

- [ ] **T516** [US5] Verify T507-T509 tests now PASS

**Checkpoint US5**: All collection combination tests pass, functionality complete

---

## Phase 6: Integration Testing (Cross-Story Validation)

**Purpose**: Verify v0.1 and v0.2 methods compose correctly

- [ ] **T601** [P] Integration test: Filter (v0.1) + Aggregation (v0.2) in `tests/rinq_v0.2_tests.rs`
  - `.where_().sum()`, `.where_().average()`, `.where_().group_by()`

- [ ] **T602** [P] Integration test: Sort (v0.1) + Deduplication (v0.2) in `tests/rinq_v0.2_tests.rs`
  - `.order_by().distinct()` (preserves sort order)

- [ ] **T603** [P] Integration test: Projection (v0.1) + Aggregation (v0.2) in `tests/rinq_v0.2_tests.rs`
  - `.select().sum()`, `.select().average()`

- [ ] **T604** [P] Integration test: Multiple v0.2 operations chained in `tests/rinq_v0.2_tests.rs`
  - `.distinct().enumerate()`, `.reverse().chunk()`

- [ ] **T605** [P] Integration test: Pagination (v0.1) + Grouping (v0.2) in `tests/rinq_v0.2_tests.rs`
  - `.take(n).group_by()`, `.skip(n).partition()`

**Checkpoint**: All integration tests pass, v0.1/v0.2 interoperability verified

---

## Phase 7: MetricsQueryBuilder Extension (Observability)

**Purpose**: Extend `MetricsQueryBuilder` to support v0.2 operations with metrics recording

### Tests for Metrics Integration (Write Tests FIRST)

- [ ] **T701** [P] Metrics test: `sum()` records timing and count in `tests/rinq_integration_tests.rs`
  - Verify: `rinq.sum` timer recorded
  - Verify: `rinq.operations.sum` counter incremented

- [ ] **T702** [P] Metrics test: Aggregations record metrics in `tests/rinq_integration_tests.rs`
  - Test: average, min, max, min_by, max_by

- [ ] **T703** [P] Metrics test: Grouping records metrics in `tests/rinq_integration_tests.rs`
  - Test: group_by, group_by_aggregate

- [ ] **T704** [P] Metrics test: `partition()` records metrics in `tests/rinq_integration_tests.rs`

- [ ] **T705** [P] Metrics test: Non-terminal ops don't record until terminal in `tests/rinq_integration_tests.rs`
  - `.distinct().sum()` should record sum metrics, not distinct metrics

**Checkpoint**: All metrics tests written and FAILING

---

### Implementation for MetricsQueryBuilder

#### Terminal Operation Wrappers

- [ ] **T710** Implement metrics wrapper for `sum()` in `src/domain/rinq/metrics_query_builder.rs`
  - Record timing with `Instant::now()` and `elapsed()`
  - Call `self.metrics.record_query_time("rinq.sum", elapsed)`
  - Call `self.metrics.increment("rinq.operations.sum")`
  - Delegate to `self.inner.sum()`

- [ ] **T711** [P] Implement metrics wrappers for `average()`, `min()`, `max()` in `src/domain/rinq/metrics_query_builder.rs`
  - Follow same pattern as `sum()`

- [ ] **T712** [P] Implement metrics wrappers for `min_by()`, `max_by()` in `src/domain/rinq/metrics_query_builder.rs`

- [ ] **T713** [P] Implement metrics wrappers for `group_by()`, `group_by_aggregate()` in `src/domain/rinq/metrics_query_builder.rs`

- [ ] **T714** [P] Implement metrics wrapper for `partition()` in `src/domain/rinq/metrics_query_builder.rs`

- [ ] **T715** Verify T701-T704 tests now PASS

#### Non-Terminal Operation Pass-Throughs

- [ ] **T716** Implement pass-through for `distinct()`, `distinct_by()` in `src/domain/rinq/metrics_query_builder.rs`
  - Wrap returned `QueryBuilder` in `MetricsQueryBuilder`
  - Pass metrics collector through

- [ ] **T717** [P] Implement pass-throughs for `reverse()`, `enumerate()`, `zip()` in `src/domain/rinq/metrics_query_builder.rs`

- [ ] **T718** [P] Implement pass-throughs for `chunk()`, `window()` in `src/domain/rinq/metrics_query_builder.rs`

- [ ] **T719** Verify T705 test now PASSES (non-terminal ops don't record metrics)

**Checkpoint**: MetricsQueryBuilder fully supports v0.2, all metrics tests pass

---

## Phase 8: Benchmarking (Performance Validation)

**Purpose**: Validate zero-cost abstraction claims with criterion benchmarks

- [ ] **T801** Create `benches/rinq_v0.2_benchmarks.rs` with criterion setup

#### Aggregation Benchmarks

- [ ] **T802** [P] Benchmark: `sum()` vs manual loop in `benches/rinq_v0.2_benchmarks.rs`
  - RINQ: `QueryBuilder::from(data).sum()`
  - Manual: `data.iter().sum()`
  - Dataset sizes: 100, 1K, 10K, 100K elements

- [ ] **T803** [P] Benchmark: `average()` vs manual calculation in `benches/rinq_v0.2_benchmarks.rs`

- [ ] **T804** [P] Benchmark: `min()` / `max()` vs manual iteration in `benches/rinq_v0.2_benchmarks.rs`

- [ ] **T805** [P] Benchmark: `min_by()` / `max_by()` vs manual tracking in `benches/rinq_v0.2_benchmarks.rs`

#### Grouping Benchmarks

- [ ] **T806** [P] Benchmark: `group_by()` vs manual HashMap construction in `benches/rinq_v0.2_benchmarks.rs`
  - Various cardinality (few groups vs many groups)

- [ ] **T807** [P] Benchmark: `group_by_aggregate()` vs manual grouped calculation in `benches/rinq_v0.2_benchmarks.rs`

#### Transformation Benchmarks

- [ ] **T808** [P] Benchmark: `distinct()` vs manual HashSet deduplication in `benches/rinq_v0.2_benchmarks.rs`
  - Various duplication rates (10%, 50%, 90%)

- [ ] **T809** [P] Benchmark: `reverse()` vs `Vec::reverse()` in `benches/rinq_v0.2_benchmarks.rs`

- [ ] **T810** [P] Benchmark: `chunk()` vs manual chunking loop in `benches/rinq_v0.2_benchmarks.rs`

- [ ] **T811** [P] Benchmark: `window()` vs manual sliding window in `benches/rinq_v0.2_benchmarks.rs`
  - Note: Expect overhead due to Clone requirement

- [ ] **T812** [P] Benchmark: `enumerate()` vs manual index tracking in `benches/rinq_v0.2_benchmarks.rs`

- [ ] **T813** [P] Benchmark: `partition()` vs manual two-vec building in `benches/rinq_v0.2_benchmarks.rs`

#### Complex Scenario Benchmarks

- [ ] **T814** [P] Benchmark: Chained operations in `benches/rinq_v0.2_benchmarks.rs`
  - `.where_().distinct().order_by().group_by()`
  - Compare with equivalent manual implementation

- [ ] **T815** Run `cargo bench` and verify all benchmarks complete successfully

- [ ] **T816** Analyze benchmark results: Verify ≤5% overhead for aggregations, ≤10% for grouping

**Checkpoint**: All benchmarks pass, zero-cost claims validated

---

## Phase 9: Documentation (User-Facing)

**Purpose**: Provide comprehensive documentation for all v0.2 features

- [ ] **T901** [P] Add doc comments for `sum()`, `average()`, `min()`, `max()` in `src/domain/rinq/query_builder.rs`
  - Include usage examples
  - Document trait bounds
  - Note edge cases (empty collections)

- [ ] **T902** [P] Add doc comments for `min_by()`, `max_by()` in `src/domain/rinq/query_builder.rs`
  - Include struct example with key selector

- [ ] **T903** [P] Add doc comments for `group_by()`, `group_by_aggregate()` in `src/domain/rinq/query_builder.rs`
  - Include practical examples (grouping users, aggregating orders)

- [ ] **T904** [P] Add doc comments for `distinct()`, `distinct_by()` in `src/domain/rinq/query_builder.rs`
  - Document Clone requirement for `distinct()`
  - Include key-based deduplication example

- [ ] **T905** [P] Add doc comments for `reverse()`, `chunk()`, `window()` in `src/domain/rinq/query_builder.rs`
  - Document panic conditions for chunk/window
  - Document Clone requirement for `window()`
  - Include batch processing example for `chunk()`

- [ ] **T906** [P] Add doc comments for `zip()`, `enumerate()`, `partition()` in `src/domain/rinq/query_builder.rs`
  - Include correlation and indexing examples

- [ ] **T907** Update `src/domain/rinq/README.md` with v0.2 section
  - Add "v0.2 Features" heading after v0.1 features
  - Include code examples for each operation group
  - Add API reference table with all v0.2 methods
  - Add migration guide (confirm no breaking changes)

- [ ] **T908** Run `cargo doc --open` to verify documentation renders correctly

- [ ] **T909** Run `cargo test --doc` to verify all doc examples compile and pass

**Checkpoint**: Documentation complete, all doc tests pass

---

## Phase 10: Module Exports and Public API

**Purpose**: Export new functionality in module system

- [ ] **T1001** Update `src/domain/rinq/mod.rs` exports
  - Verify `pub use query_builder::QueryBuilder;` already exports all methods
  - Verify `pub use metrics_query_builder::MetricsQueryBuilder;` already exports all wrappers
  - No changes needed (methods auto-exported with types)

- [ ] **T1002** Update `src/lib.rs` if needed
  - Verify domain module exports are correct

- [ ] **T1003** Verify public API is accessible from external crates
  - Create test in `tests/rinq_v0.2_tests.rs` using public API path
  - `use rusted_ca::domain::rinq::QueryBuilder;`

**Checkpoint**: Public API correctly exposed

---

## Phase 11: Final Quality Gates (Pre-Merge Validation)

**Purpose**: Ensure all Constitution requirements met before considering complete

### Code Quality

- [ ] **T1101** Run `cargo clippy -- -D warnings` and fix any warnings
  - Zero warnings required

- [ ] **T1102** Run `cargo fmt --check` and fix formatting
  - Code must be formatted

- [ ] **T1103** Run `cargo check` and verify compilation
  - No errors

### Test Coverage

- [ ] **T1104** Run `cargo test` and verify all tests pass
  - Existing v0.1: 115+ tests (backwards compatibility)
  - New v0.2: 50+ property tests + 30+ unit tests
  - Integration: 10+ tests
  - **Target**: 195+ total passing tests

- [ ] **T1105** Verify test coverage for new code
  - Property tests cover all new methods
  - Unit tests cover edge cases
  - Integration tests cover v0.1/v0.2 composition

### Performance Validation

- [ ] **T1106** Run `cargo bench` and analyze results
  - Aggregations: ≤5% overhead vs manual
  - Grouping: ≤10% overhead vs manual
  - Document any outliers

- [ ] **T1107** Run benchmarks on large datasets (100K+ elements)
  - Verify linear scaling (no quadratic behavior)
  - Check memory usage

### Backwards Compatibility

- [ ] **T1108** Run existing v0.1 test suite independently
  - `cargo test rinq_property_tests`
  - `cargo test rinq_immutability_test`
  - `cargo test rinq_integration_tests`
  - All must pass (no regressions)

- [ ] **T1109** Verify v0.1 example code still compiles
  - Check examples in `src/domain/rinq/README.md` v0.1 section

### Documentation Validation

- [ ] **T1110** Run `cargo doc --open` and manually review generated docs
  - All new methods documented
  - Examples render correctly
  - Links work

- [ ] **T1111** Run `cargo test --doc` to verify doc examples
  - All doc code examples must compile and pass

**Checkpoint**: All quality gates passed, ready for merge

---

## Phase 12: Final Summary and Cleanup

**Purpose**: Prepare for PR and future reference

- [ ] **T1201** Create `CHANGELOG.md` entry for v0.2
  - List all new features
  - Note: No breaking changes
  - Migration guide: "v0.1 code continues to work without changes"

- [ ] **T1202** Update main project README if applicable
  - Add RINQ v0.2 to feature list

- [ ] **T1203** Review `.specify/specs/002-rinq-v0.2-aggregation/` directory
  - Ensure all spec documents are up to date
  - Mark tasks.md checkboxes complete

- [ ] **T1204** Run final validation suite
  - `cargo test --all`
  - `cargo bench --all`
  - `cargo clippy --all -- -D warnings`
  - `cargo fmt --check`

**Checkpoint**: RINQ v0.2 implementation complete, all gates passed

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 0 (Preparation)
    ↓
Phase 1 (US1: Aggregations) ────┐
    ↓                           │
Phase 2 (US2: Grouping)         │
    ↓                           │ (Independent - can run in parallel)
Phase 3 (US3: Deduplication)    │
    ↓                           │
Phase 4 (US4: Sequences)        │
    ↓                           │
Phase 5 (US5: Combinations) ────┘
    ↓
Phase 6 (Integration Testing)
    ↓
Phase 7 (MetricsQueryBuilder)
    ↓
Phase 8 (Benchmarking)
    ↓
Phase 9 (Documentation)
    ↓
Phase 10 (Module Exports)
    ↓
Phase 11 (Quality Gates)
    ↓
Phase 12 (Final Summary)
```

### User Story Independence

**After Phase 0 completes**, User Stories can be implemented:

1. **Sequential (Recommended)**: P1 → P2 → P3 → P4 → P5
   - Each story fully tested before moving to next
   - Safest approach, catches issues early

2. **Parallel (Advanced)**: Implement P1-P5 simultaneously
   - All stories add methods to same file (`query_builder.rs`)
   - Risk: Merge conflicts
   - Mitigation: Careful coordination, separate impl blocks per state

**Recommendation**: Sequential implementation (follow priority order).

---

### Within Each User Story

**Test-Driven Development Flow**:

1. ✅ Write property tests (T10X, T20X, etc.) - **MUST FAIL**
2. ✅ Write unit tests - **MUST FAIL**
3. ✅ Implement feature
4. ✅ Verify tests now PASS
5. ✅ Move to next feature

**Parallel Opportunities**:
- All property tests for a story can be written in parallel (marked [P])
- Multiple impl blocks (Initial, Filtered, Sorted, Projected) can be implemented in parallel if carefully coordinated
- Different benchmarks can be written in parallel (marked [P])

---

## Task Count Summary

| Phase | Task Count | Parallel Tasks | Description |
|-------|------------|----------------|-------------|
| Phase 0 | 4 | 0 | Preparation |
| Phase 1 (US1) | 23 | 13 | Numeric aggregations |
| Phase 2 (US2) | 15 | 8 | Grouping operations |
| Phase 3 (US3) | 15 | 8 | Deduplication |
| Phase 4 (US4) | 19 | 9 | Sequence transformations |
| Phase 5 (US5) | 17 | 9 | Collection combinations |
| Phase 6 | 5 | 5 | Integration testing |
| Phase 7 | 19 | 8 | MetricsQueryBuilder |
| Phase 8 | 16 | 12 | Benchmarking |
| Phase 9 | 11 | 9 | Documentation |
| Phase 10 | 3 | 0 | Module exports |
| Phase 11 | 11 | 0 | Quality gates |
| Phase 12 | 4 | 0 | Final summary |
| **TOTAL** | **162 tasks** | **81 can be parallel** | |

---

## Implementation Strategy

### MVP-First (Fastest to Value)

1. **Phase 0**: Preparation (T001-T004)
2. **Phase 1**: US1 - Aggregations (T101-T123) ← **DELIVERS IMMEDIATE VALUE**
   - STOP and VALIDATE: Test aggregations independently
   - Can ship this alone as v0.2-alpha
3. Continue with P2-P5 incrementally

### Complete Implementation (All Features)

1. **Phases 0-5**: All user stories (T001-T516)
2. **Phases 6-7**: Integration + Metrics (T601-T719)
3. **Phase 8**: Benchmarks (T801-T816)
4. **Phases 9-12**: Documentation + Final validation (T901-T1204)

### Parallel Team Strategy

If multiple developers available:

**After Phase 0 completes**:
- Developer A: Phase 1 (US1: Aggregations)
- Developer B: Phase 3 (US3: Deduplication)
- Developer C: Phase 5 (US5: Combinations)

**After individual phases complete**:
- All developers: Phase 2 (US2: Grouping - uses most complex patterns)
- All developers: Phase 4 (US4: Sequences - needs custom iterators)

**Final phases** (6-12): Sequential, all developers collaborate

---

## Validation Checkpoints

### After Phase 1 (US1 Complete)
```bash
cargo test rinq_v0.2_tests::aggregation
cargo bench rinq_v0.2_benchmarks::aggregation
```
**Expected**: 15+ tests pass, benchmarks show ≤5% overhead

### After Phase 2 (US2 Complete)
```bash
cargo test rinq_v0.2_tests::grouping
```
**Expected**: 8+ tests pass

### After Phase 5 (All User Stories Complete)
```bash
cargo test rinq_v0.2_tests
```
**Expected**: 80+ v0.2 tests pass

### After Phase 11 (All Quality Gates)
```bash
cargo test --all
cargo bench --all
cargo clippy --all -- -D warnings
cargo fmt --check
```
**Expected**: 195+ tests pass, zero warnings, benchmarks validate performance

---

## Risk Mitigation

### Risk: Merge Conflicts in query_builder.rs

**Mitigation**: 
- Implement user stories sequentially (not parallel)
- Each story adds methods to separate `impl` blocks (Initial, Filtered, Sorted, Projected)
- Commit after each user story completes

### Risk: Performance Regression in v0.1 Operations

**Mitigation**:
- Run v0.1 benchmark suite after each phase
- Compare results to baseline (from v0.1 completion)
- Investigate any >5% degradation immediately

### Risk: Test Count Explosion

**Mitigation**:
- Focus on property tests (fewer tests, broader coverage)
- Unit tests only for edge cases not covered by properties
- Target: 50 property tests + 30 unit tests (manageable scope)

### Risk: Trait Bound Conflicts

**Mitigation**:
- Explicit `where` clauses for all trait bounds
- Test with various types (i32, f64, custom structs)
- Integration tests verify chaining doesn't create bound conflicts

---

## Notes

- **[P] tasks**: Can run in parallel (different test functions, different benchmarks)
- **[US1]-[US5] labels**: Map task to user story priority
- **Test-first mandatory**: Per Constitution Principle V, tests must be written and fail before implementation
- **Incremental delivery**: Each user story independently testable
- **Backwards compatibility**: v0.1 tests must continue passing throughout

---

## Success Criteria (From spec.md SC-001 to SC-010)

At completion, verify:

- ✅ **SC-001**: All 12 new operations implemented and functional
- ✅ **SC-002**: At least 50 new property-based tests added, all passing
- ✅ **SC-003**: At least 30 new unit tests added, all passing
- ✅ **SC-004**: At least 15 new benchmarks added, ≤5% overhead validated
- ✅ **SC-005**: All operations chain correctly with v0.1 methods
- ✅ **SC-006**: `MetricsQueryBuilder` records metrics for all new operations
- ✅ **SC-007**: Documentation includes working examples for each operation
- ✅ **SC-008**: `cargo clippy` passes with zero warnings
- ✅ **SC-009**: `cargo test` passes with 100% success (195+ tests)
- ✅ **SC-010**: `cargo bench` shows performance parity

---

**Tasks Status**: ✅ **READY FOR IMPLEMENTATION**  
**Total Tasks**: 162  
**Parallel Opportunities**: 81 tasks can run concurrently  
**Next Command**: `/speckit.implement`
