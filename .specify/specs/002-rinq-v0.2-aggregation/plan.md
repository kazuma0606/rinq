# Implementation Plan: RINQ v0.2 - Aggregation and Transformation Extensions

**Branch**: `002-rinq-v0.2-aggregation` | **Date**: 2026-03-08 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `.specify/specs/002-rinq-v0.2-aggregation/spec.md`

## Summary

RINQ v0.2 extends the type-safe, zero-cost query engine with practical aggregation and transformation capabilities. Building on v0.1's foundation (115+ tests, validated zero-cost abstraction), v0.2 adds:

- **Numeric Aggregations**: `sum()`, `average()`, `min()`, `max()`, `min_by()`, `max_by()` for data analysis
- **Grouping**: `group_by()`, `group_by_aggregate()` for categorization and per-group analytics
- **Deduplication**: `distinct()`, `distinct_by()` for data cleaning
- **Sequence Transformations**: `reverse()`, `chunk()`, `window()` for batch processing and sliding windows
- **Collection Combinations**: `zip()`, `enumerate()`, `partition()` for data correlation and splitting

**Technical Approach**: Extend existing `QueryBuilder` with new methods that maintain type safety, lazy evaluation (where possible), and zero-cost abstraction guarantees. Use Rust's iterator adapters as foundation, wrapping in RINQ's fluent interface. Validate all performance claims with criterion benchmarks.

## Technical Context

**Language/Version**: Rust 2021 Edition, MSRV 1.70+  
**Primary Dependencies**: 
- `num-traits = "0.2"` (for generic numeric operations - NEW)
- `proptest = "1.0"` (property-based testing - existing)
- `criterion = "0.5"` (benchmarking - existing)

**Storage**: N/A (in-memory collection operations only)  
**Testing**: `cargo test` (unit + property + integration), `cargo bench` (criterion)  
**Target Platform**: Any platform supporting Rust std library  
**Project Type**: Library (domain module within rusted-ca)  
**Performance Goals**: 
- Aggregations within 5% of manual loop performance
- Grouping operations within 10% of manual HashMap construction
- Zero additional allocations beyond semantically required (e.g., distinct needs HashSet)

**Constraints**: 
- Must maintain backwards compatibility with RINQ v0.1
- Must integrate with existing type-state pattern
- Must work with `MetricsQueryBuilder` for observability
- Must compile with zero clippy warnings

**Scale/Scope**: 
- 12 new public methods
- 50+ new property tests
- 30+ new unit tests
- 15+ new benchmarks
- Documentation with examples for each method

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **I. Type Safety First**: All operations integrate with type-state pattern, preventing invalid usage at compile time  
✅ **II. Zero-Cost Abstraction**: Benchmarks required for all operations, validating ≤5% overhead  
✅ **III. Pragmatic Utility**: All features address concrete use cases (data analysis, reporting, batch processing)  
✅ **IV. API Consistency**: Methods follow v0.1 naming conventions and fluent interface pattern  
✅ **V. Test-Driven Development**: Property tests + unit tests written before implementation  
✅ **VI. Performance Validation**: Criterion benchmarks for every operation  
✅ **VII. Incremental Integration**: Extends existing `QueryBuilder`, leverages existing `MetricsQueryBuilder` and error types

**No violations.** All features align with constitution principles.

## Project Structure

### Documentation (this feature)

```text
.specify/specs/002-rinq-v0.2-aggregation/
├── spec.md              # Feature specification (created by /speckit.specify)
├── plan.md              # This file (created by /speckit.plan)
├── research.md          # Technical research and decisions
├── data-model.md        # Type system extensions and data structures
├── quickstart.md        # Quick start guide with examples
└── tasks.md             # Task breakdown (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── domain/
│   └── rinq/
│       ├── mod.rs                      # Module exports (update)
│       ├── state.rs                    # Type states (possibly extend)
│       ├── error.rs                    # Domain errors (existing)
│       ├── query_builder.rs            # Core implementation (EXTEND)
│       ├── metrics_query_builder.rs    # Metrics wrapper (EXTEND)
│       └── README.md                   # Documentation (UPDATE)
│
├── shared/
│   └── error/
│       └── application_error.rs        # Error conversion (existing)

tests/
├── rinq_property_tests.rs              # Existing v0.1 property tests
├── rinq_immutability_test.rs           # Existing immutability tests
├── rinq_integration_tests.rs           # Existing integration tests
└── rinq_v0.2_tests.rs                  # NEW: v0.2 property + unit tests

benches/
├── rinq_benchmarks.rs                  # Existing v0.1 benchmarks
└── rinq_v0.2_benchmarks.rs             # NEW: v0.2 benchmarks

Cargo.toml                              # Add num-traits dependency
```

**Structure Decision**: Extend existing `src/domain/rinq/query_builder.rs` with new methods rather than creating separate modules. This keeps all QueryBuilder functionality in one place and avoids splitting the type-state implementation. The `aggregation.rs` module will NOT be created unless the implementation becomes unwieldy (>2000 lines).

---

## Phase 0: Research & Technical Decisions

### R1: Numeric Trait Bounds Strategy

**Decision Required**: How to handle generic numeric operations across different number types (i32, f64, u32, etc.)?

**Options Researched**:

1. **Use `num-traits` crate** (RECOMMENDED)
   - Provides `Num`, `Zero`, `One`, `ToPrimitive`, `FromPrimitive` traits
   - Industry standard for numeric generics in Rust
   - Well-tested and maintained
   - Examples:
     ```rust
     use num_traits::{Zero, Num, ToPrimitive};
     
     fn sum<T: Num + Zero + Copy>(iter: impl Iterator<Item = T>) -> T {
         iter.fold(T::zero(), |acc, x| acc + x)
     }
     ```

2. **Standard library only**
   - Use `std::iter::Sum` trait for `.sum()`
   - Manual implementation for `.average()`
   - Limitations: Less flexible for custom numeric types
   
**Recommendation**: Use `num-traits` for:
- `sum()`: Can use `Iterator::sum()` which already uses `std::iter::Sum`
- `average()`: Need `ToPrimitive` to convert to `f64` safely
- Provides path for future extensions (median, standard deviation, etc.)

**Constitution Alignment**: Violates "minimize dependencies" preference, but justified by providing robust, type-safe numeric operations without reimplementing trait ecosystem. Trade-off: +1 dependency for production-grade numeric handling.

---

### R2: Lazy vs Eager Evaluation Strategy

**Decision Required**: Which operations can remain lazy (iterator-based) vs must be eager (materialize collection)?

**Analysis**:

| Operation | Evaluation | Rationale |
|-----------|-----------|-----------|
| `sum()`, `average()`, `min()`, `max()` | **Eager** (Terminal) | Must consume entire iterator to produce result |
| `min_by()`, `max_by()` | **Eager** (Terminal) | Must examine all elements |
| `group_by()` | **Eager** (Terminal) | Must materialize HashMap |
| `group_by_aggregate()` | **Eager** (Terminal) | Must compute all groups |
| `distinct()` | **Eager** (Stateful) | Needs HashSet to track seen elements |
| `distinct_by()` | **Eager** (Stateful) | Needs HashSet to track seen keys |
| `reverse()` | **Eager** (Stateful) | Must collect all elements to reverse |
| `chunk()` | **Lazy** | Can use iterator adapter (but may need buffering) |
| `window()` | **Lazy** | Can use iterator adapter with sliding buffer |
| `zip()` | **Lazy** | Pure iterator adapter |
| `enumerate()` | **Lazy** | Pure iterator adapter |
| `partition()` | **Eager** (Terminal) | Must materialize both collections |

**Recommendation**:
- **Terminal Operations** (consume QueryBuilder, return result): `sum`, `average`, `min`, `max`, `min_by`, `max_by`, `group_by`, `group_by_aggregate`, `partition`
- **Stateful Operations** (return new QueryBuilder): `distinct`, `distinct_by`, `reverse`
- **Lazy Operations** (return new QueryBuilder): `chunk`, `window`, `zip`, `enumerate`

**Constitution Alignment**: Maintains lazy evaluation where semantically possible, aligning with zero-cost principle.

---

### R3: Type State Extensions

**Decision Required**: Do we need new type states for v0.2 operations?

**Current States**:
- `Initial`: Fresh QueryBuilder
- `Filtered`: After `.where_()`
- `Sorted`: After `.order_by()`
- `Projected<U>`: After `.select()`

**Analysis**:

| Operation | State Transition | New State Needed? |
|-----------|-----------------|-------------------|
| `distinct()` | `Initial/Filtered/Sorted → Filtered` | ❌ No (reuses Filtered) |
| `distinct_by()` | `Initial/Filtered/Sorted → Filtered` | ❌ No (reuses Filtered) |
| `reverse()` | `Initial/Filtered → Filtered` | ❌ No (reuses Filtered) |
| `reverse()` | `Sorted → ?` | ⚠️ Discussion: Can reverse maintain Sorted state? |
| `enumerate()` | `Any → Filtered` | ❌ No (changes element type to tuple) |
| `zip()` | `Any → Filtered` | ❌ No (changes element type to tuple) |
| `chunk()` | `Any → Filtered` | ❌ No (changes element type to Vec) |
| `window()` | `Any → Filtered` | ❌ No (changes element type to slice/Vec) |

**Recommendation**: 
- **Do NOT add new states** for v0.2. Reuse existing states:
  - Operations that maintain element type but lose sort order → transition to `Filtered`
  - Operations that change element type → transition to `Filtered` (since type changed)
- **Special case**: `reverse()` on `Sorted` state:
  - Option A: Transition to `Filtered` (conservative, maintains semantics)
  - Option B: Remain in `Sorted` with reversed comparator (complex, questionable value)
  - **Recommend Option A** for simplicity

**Constitution Alignment**: Avoids state explosion, maintains clarity of type-state pattern.

---

### R4: QueryData Enum Extensions

**Decision Required**: Does `QueryData<T>` need new variants for v0.2?

**Current Variants**:
```rust
enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec {
        items: Vec<T>,
        comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    },
}
```

**Analysis**:
- `distinct()`, `distinct_by()`: Can use `Iterator` with stateful adapter → ❌ No new variant
- `reverse()`: Can collect into `Vec` and reverse, then wrap as `Iterator` → ❌ No new variant
- `chunk()`, `window()`, `zip()`, `enumerate()`: Pure iterator adapters → ❌ No new variant
- `group_by()`: Terminal operation, returns `HashMap` directly → ❌ No new variant

**Recommendation**: **No new QueryData variants needed.** All operations can be implemented using existing `Iterator` and `SortedVec` variants.

**Rationale**: 
- Terminal operations don't need to store state in `QueryData` (they consume and return results)
- Stateful operations (distinct, reverse) can materialize, transform, and re-wrap as `Iterator`
- Lazy operations naturally fit iterator adapters

**Constitution Alignment**: Keeps internal complexity minimal, aligns with incremental integration principle.

---

### R5: Error Handling Strategy

**Decision Required**: Which operations are fallible and need error handling?

**Analysis**:

| Operation | Fallible? | Error Scenarios |
|-----------|-----------|-----------------|
| `sum()`, `average()` | ❌ No | Overflow handled by type (saturating or wrapping) |
| `min()`, `max()` | ❌ No | Returns `Option<T>` for empty case |
| `min_by()`, `max_by()` | ❌ No | Returns `Option<T>` for empty case |
| `group_by()` | ❌ No | Returns empty HashMap for empty input |
| `distinct()` | ❌ No | Returns empty collection for empty input |
| `reverse()` | ❌ No | Returns empty for empty input |
| `chunk(size)` | ⚠️ Maybe | `size == 0` is invalid |
| `window(size)` | ⚠️ Maybe | `size == 0` or `size == 1` may be invalid |
| `zip()` | ❌ No | Shortest-wins semantics |
| `enumerate()` | ❌ No | Works on empty |
| `partition()` | ❌ No | Returns two empty vecs for empty input |

**Recommendation**:
- Most operations: **No error handling needed**, use `Option` for "not found" semantics
- `chunk(0)`: **Panic** with descriptive message (invalid argument, programming error)
- `window(0)` or `window(1)`: **Panic** with descriptive message (minimum size is 2 for meaningful windows)
- Alternative: Accept invalid sizes and return empty results (more forgiving, less clear)

**Decision**: Panic on invalid sizes (following Rust conventions like `Vec::split_at` panics on out-of-bounds). Document clearly in API docs.

**Constitution Alignment**: "No silent failures" principle - invalid states must error explicitly.

---

### R6: MetricsQueryBuilder Integration

**Decision Required**: How should `MetricsQueryBuilder` handle new operations?

**Current Pattern** (from v0.1):
- `MetricsQueryBuilder` wraps `QueryBuilder`
- Terminal operations (collect, count, first, last, any, all) record timing and increment metrics
- Query-building operations (where_, order_by, select) pass through to wrapped builder

**Strategy for v0.2**:
1. **Terminal operations** (sum, average, min, max, min_by, max_by, group_by, group_by_aggregate, partition):
   - Record execution time
   - Increment operation counter
   - Delegate to wrapped `QueryBuilder`

2. **Stateful/Lazy operations** (distinct, reverse, enumerate, zip, chunk, window):
   - Pass through to wrapped builder (no metrics recorded)
   - Rationale: These don't execute until terminal operation

**Implementation**:
- Add methods to `impl<T> MetricsQueryBuilder<T, Initial>`, `impl<T> MetricsQueryBuilder<T, Filtered>`, etc.
- Mirror `QueryBuilder` method signatures
- Wrap results in `MetricsQueryBuilder` to maintain fluent chain

**Constitution Alignment**: Maintains observability integration (principle VII).

---

## Phase 1: Design & Data Model

### Type System Extensions

#### Option 1: No New States (RECOMMENDED)

**Rationale**: All v0.2 operations can fit into existing states:
- `Initial`: Start state
- `Filtered`: After transformations that may lose sort order (distinct, reverse, enumerate, zip, chunk, window)
- `Sorted`: Maintained from v0.1
- `Projected<U>`: When element type changes (select)

**State Transitions**:

```rust
// Initial state methods
impl<T: 'static> QueryBuilder<T, Initial> {
    pub fn where_(...) -> QueryBuilder<T, Filtered>  // existing
    pub fn order_by(...) -> QueryBuilder<T, Sorted>   // existing
    pub fn select<U>(...) -> QueryBuilder<U, Projected<U>>  // existing
    
    // NEW: Aggregations (terminal)
    pub fn sum(self) -> T where T: Sum
    pub fn average(self) -> Option<f64> where T: ToPrimitive
    pub fn min(self) -> Option<T> where T: Ord
    pub fn max(self) -> Option<T> where T: Ord
    pub fn min_by<K, F>(self, key: F) -> Option<T> where F: Fn(&T) -> K, K: Ord
    pub fn max_by<K, F>(self, key: F) -> Option<T> where F: Fn(&T) -> K, K: Ord
    
    // NEW: Grouping (terminal)
    pub fn group_by<K, F>(self, key: F) -> HashMap<K, Vec<T>> 
        where F: Fn(&T) -> K, K: Eq + Hash
    pub fn group_by_aggregate<K, R, FK, FA>(self, key: FK, agg: FA) -> HashMap<K, R>
        where FK: Fn(&T) -> K, K: Eq + Hash, FA: Fn(&[T]) -> R
    
    // NEW: Transformations (return QueryBuilder)
    pub fn distinct(self) -> QueryBuilder<T, Filtered> where T: Eq + Hash + Clone
    pub fn distinct_by<K, F>(self, key: F) -> QueryBuilder<T, Filtered>
        where F: Fn(&T) -> K + 'static, K: Eq + Hash
    pub fn reverse(self) -> QueryBuilder<T, Filtered>
    pub fn enumerate(self) -> QueryBuilder<(usize, T), Filtered>
    pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
    pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
    pub fn zip<U>(self, other: impl IntoIterator<Item = U>) -> QueryBuilder<(T, U), Filtered>
    pub fn partition<F>(self, pred: F) -> (Vec<T>, Vec<T>) where F: Fn(&T) -> bool
}

// Filtered state methods
impl<T: 'static> QueryBuilder<T, Filtered> {
    // All existing methods (where_, order_by, select, collect, etc.)
    // Plus NEW methods (same as Initial state)
}

// Sorted state methods
impl<T: 'static> QueryBuilder<T, Sorted> {
    // Existing: then_by, select, collect, take, skip, etc.
    // NEW: Most v0.2 methods available, but may transition to Filtered
    // Example: distinct() on Sorted → Filtered (loses sort order)
}

// Projected state methods
impl<U: 'static> QueryBuilder<U, Projected<U>> {
    // Existing: collect, take, skip, count, etc.
    // NEW: Can also call aggregations on projected data
}
```

**Design Decision**: Reuse `Filtered` state for all transformations that lose sort order or change type. This keeps the state space small and predictable.

---

### Implementation Strategy by Feature Group

#### Group 1: Numeric Aggregations (P1 - Highest Priority)

**Target**: `sum()`, `average()`, `min()`, `max()`, `min_by()`, `max_by()`

**Technical Approach**:

```rust
use num_traits::{ToPrimitive, Zero};
use std::iter::Sum;

// sum() - leverage std::iter::Sum trait
impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn sum(self) -> T 
    where 
        T: Sum,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.sum(),
            QueryData::SortedVec { .. } => unreachable!(),
        }
    }
}

// average() - convert to f64, sum, divide
impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn average(self) -> Option<f64>
    where
        T: ToPrimitive,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let items: Vec<T> = iter.collect();
                if items.is_empty() {
                    return None;
                }
                let sum: f64 = items.iter()
                    .filter_map(|x| x.to_f64())
                    .sum();
                Some(sum / items.len() as f64)
            }
            QueryData::SortedVec { .. } => unreachable!(),
        }
    }
}

// min() / max() - use Iterator::min() / Iterator::max()
impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.min(),
            QueryData::SortedVec { items, .. } => items.into_iter().min(),
        }
    }
}

// min_by() / max_by() - use Iterator::min_by_key() / max_by_key()
impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        match self.data {
            QueryData::Iterator(iter) => iter.min_by_key(key_selector),
            QueryData::SortedVec { items, .. } => {
                items.into_iter().min_by_key(key_selector)
            }
        }
    }
}
```

**Implementation Notes**:
- Implement for all states (`Initial`, `Filtered`, `Sorted`, `Projected`)
- Optimizations: For `Sorted` state, `min()` could return first element, `max()` last element (O(1) vs O(n))
- Edge cases: Empty collections return `None` or identity values

**Testing**:
- Property: `sum(xs) == xs.iter().sum()`
- Property: `average(xs) == xs.iter().map(|x| x as f64).sum() / xs.len() as f64`
- Property: `min(xs) == xs.iter().min()`
- Edge cases: empty, single element, overflow handling

**Benchmarks**:
- `sum()` vs manual loop
- `average()` vs manual calculation
- `min_by()` vs manual iteration with tracking

---

#### Group 2: Grouping Operations (P2)

**Target**: `group_by()`, `group_by_aggregate()`

**Technical Approach**:

```rust
use std::collections::HashMap;
use std::hash::Hash;

impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut groups: HashMap<K, Vec<T>> = HashMap::new();
                for item in iter {
                    let key = key_selector(&item);
                    groups.entry(key).or_insert_with(Vec::new).push(item);
                }
                groups
            }
            QueryData::SortedVec { items, .. } => {
                let mut groups: HashMap<K, Vec<T>> = HashMap::new();
                for item in items {
                    let key = key_selector(&item);
                    groups.entry(key).or_insert_with(Vec::new).push(item);
                }
                groups
            }
        }
    }
    
    #[inline]
    pub fn group_by_aggregate<K, R, FK, FA>(
        self, 
        key_selector: FK, 
        aggregator: FA
    ) -> HashMap<K, R>
    where
        FK: Fn(&T) -> K,
        FA: Fn(&[T]) -> R,
        K: Eq + Hash,
    {
        let groups = self.group_by(key_selector);
        groups.into_iter()
            .map(|(k, v)| (k, aggregator(&v)))
            .collect()
    }
}
```

**Implementation Notes**:
- Use `HashMap::entry()` API for efficient grouping
- Preserve insertion order within each group (Vec maintains order)
- `group_by_aggregate` reuses `group_by` then transforms values

**Testing**:
- Property: All elements appear exactly once across all groups
- Property: Group keys are deterministic for same input
- Property: Within-group order preserved
- Edge cases: empty, all same key, all unique keys

**Benchmarks**:
- `group_by()` vs manual HashMap construction
- `group_by_aggregate()` vs manual grouped calculation

---

#### Group 3: Deduplication (P3)

**Target**: `distinct()`, `distinct_by()`

**Technical Approach**:

```rust
use std::collections::HashSet;

impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn distinct(self) -> QueryBuilder<T, Filtered>
    where
        T: Eq + Hash + Clone,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut seen = HashSet::new();
                let filtered = iter.filter(move |item| {
                    let key = item.clone();  // Clone needed to store in HashSet
                    seen.insert(key)
                });
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(filtered)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { items, .. } => {
                let mut seen = HashSet::new();
                let mut unique = Vec::new();
                for item in items {
                    if seen.insert(item.clone()) {
                        unique.push(item);
                    }
                }
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(unique.into_iter())),
                    _state: PhantomData,
                }
            }
        }
    }
    
    #[inline]
    pub fn distinct_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + Hash,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut seen = HashSet::new();
                let filtered = iter.filter(move |item| {
                    let key = key_selector(item);
                    seen.insert(key)
                });
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(filtered)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { items, .. } => {
                let mut seen = HashSet::new();
                let mut unique = Vec::new();
                for item in items {
                    let key = key_selector(&item);
                    if seen.insert(key) {
                        unique.push(item);
                    }
                }
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(unique.into_iter())),
                    _state: PhantomData,
                }
            }
        }
    }
}
```

**Implementation Notes**:
- `distinct()` requires `Clone` to store in HashSet (cost: O(1) clone per unique element)
- `distinct_by()` only clones/hashes the key, not the full element
- Both preserve first-occurrence order
- State transition: `Sorted → Filtered` (distinct may reorder relative to sort)

**Optimization Consideration**: For `Iterator` case, we're using stateful filter closure. This is safe but captures `seen: HashSet` which grows during iteration. Alternative eager approach: collect → deduplicate → re-wrap.

**Testing**:
- Property: `distinct(xs).len() <= xs.len()`
- Property: All elements in `distinct(xs)` appear in `xs`
- Property: No duplicates in result
- Property: First occurrence preserved

**Benchmarks**:
- `distinct()` vs manual HashSet deduplication
- Large datasets with varying duplication rates

---

#### Group 4: Sequence Transformations (P4)

**Target**: `reverse()`, `chunk()`, `window()`

**Technical Approach**:

```rust
impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn reverse(self) -> QueryBuilder<T, Filtered> {
        match self.data {
            QueryData::Iterator(iter) => {
                let mut items: Vec<T> = iter.collect();
                items.reverse();
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter())),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { mut items, .. } => {
                items.reverse();
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(items.into_iter())),
                    _state: PhantomData,
                }
            }
        }
    }
    
    #[inline]
    pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> {
        assert!(size > 0, "chunk size must be greater than 0");
        
        match self.data {
            QueryData::Iterator(iter) => {
                // Custom iterator adapter for chunking
                let chunks = ChunkIterator::new(iter, size);
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(chunks)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { items, .. } => {
                let chunks = ChunkIterator::new(items.into_iter(), size);
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(chunks)),
                    _state: PhantomData,
                }
            }
        }
    }
    
    #[inline]
    pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> 
    where 
        T: Clone,  // Need Clone to create overlapping windows
    {
        assert!(size >= 2, "window size must be at least 2");
        
        match self.data {
            QueryData::Iterator(iter) => {
                let windows = WindowIterator::new(iter.collect(), size);
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(windows)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { items, .. } => {
                let windows = WindowIterator::new(items, size);
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(windows)),
                    _state: PhantomData,
                }
            }
        }
    }
}

// Helper iterator adapters (implement separately)
struct ChunkIterator<T, I: Iterator<Item = T>> {
    iter: I,
    size: usize,
}

struct WindowIterator<T: Clone> {
    items: Vec<T>,
    size: usize,
    pos: usize,
}
```

**Implementation Notes**:
- `reverse()`: Eager (must collect), then re-wrap as iterator
- `chunk()`: Can be lazy with custom iterator adapter
- `window()`: Requires `Clone` for overlapping windows, can use slice windows on collected Vec
- All transition to `Filtered` state

**Alternative for chunk()**: Use Rust nightly's `Iterator::array_chunks()`, but we target stable Rust → implement custom adapter.

**Testing**:
- Property: `reverse(reverse(xs)) == xs`
- Property: `chunk(n).flat_map().collect() == original (modulo order)`
- Property: `window(n).len() == max(0, xs.len() - n + 1)`
- Edge cases: chunk size > length, window size > length

**Benchmarks**:
- `reverse()` vs manual Vec reverse
- `chunk()` vs manual chunking loop
- `window()` vs manual sliding window

---

#### Group 5: Collection Combinations (P5)

**Target**: `zip()`, `enumerate()`, `partition()`

**Technical Approach**:

```rust
impl<T: 'static> QueryBuilder<T, Initial> {
    #[inline]
    pub fn zip<U: 'static>(
        self, 
        other: impl IntoIterator<Item = U> + 'static
    ) -> QueryBuilder<(T, U), Filtered> 
    where
        U: 'static,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                let zipped = iter.zip(other);
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(zipped)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { items, .. } => {
                let zipped = items.into_iter().zip(other);
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(zipped)),
                    _state: PhantomData,
                }
            }
        }
    }
    
    #[inline]
    pub fn enumerate(self) -> QueryBuilder<(usize, T), Filtered> {
        match self.data {
            QueryData::Iterator(iter) => {
                let enumerated = iter.enumerate();
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(enumerated)),
                    _state: PhantomData,
                }
            }
            QueryData::SortedVec { items, .. } => {
                let enumerated = items.into_iter().enumerate();
                QueryBuilder {
                    data: QueryData::Iterator(Box::new(enumerated)),
                    _state: PhantomData,
                }
            }
        }
    }
    
    #[inline]
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        match self.data {
            QueryData::Iterator(iter) => {
                iter.partition(predicate)
            }
            QueryData::SortedVec { items, .. } => {
                items.into_iter().partition(predicate)
            }
        }
    }
}
```

**Implementation Notes**:
- `zip()`: Pure iterator adapter, lazy
- `enumerate()`: Pure iterator adapter, lazy
- `partition()`: Terminal operation using `Iterator::partition()`
- All are straightforward wrappers of std library functionality

**Testing**:
- Property: `zip(xs, ys).len() == min(xs.len(), ys.len())`
- Property: `enumerate(xs).map(|(i, _)| i).collect() == (0..xs.len()).collect()`
- Property: `partition(xs, p) => (trues, falses)` where all `trues` satisfy `p`, all `falses` don't
- Edge cases: zip with different lengths, enumerate empty, partition where all match/none match

**Benchmarks**:
- `enumerate()` vs manual index tracking
- `partition()` vs manual two-vec building

---

### Critical Design Decisions

#### D1: Handling Type Changes in Operations

**Problem**: Operations like `enumerate()`, `zip()`, `chunk()` change element type (`T → (usize, T)`, `T → (T, U)`, `T → Vec<T>`).

**Solution**: 
- Return `QueryBuilder<NewType, Filtered>` where `NewType` is the transformed type
- This allows continued chaining: `.enumerate().where_(|(i, x)| i < 10).collect()`
- Type state pattern naturally enforces correct usage

**Example**:
```rust
let result = QueryBuilder::from(vec![10, 20, 30])
    .enumerate()                    // QueryBuilder<(usize, i32), Filtered>
    .where_(|(i, _)| *i > 0)        // Still Filtered
    .collect();                     // Vec<(usize, i32)>
```

---

#### D2: State Duplication vs Macro

**Problem**: Many methods need to be implemented for multiple states (Initial, Filtered, Sorted, Projected).

**Options**:

1. **Manual duplication** (current v0.1 approach)
   - Copy implementations across `impl` blocks
   - Pro: Explicit, no macro magic, easy to customize per state
   - Con: Maintenance burden, risk of divergence

2. **Macro-based generation**
   ```rust
   macro_rules! impl_aggregations {
       ($state:ty) => {
           impl<T: 'static> QueryBuilder<T, $state> {
               // ... aggregation methods
           }
       };
   }
   ```
   - Pro: DRY, guaranteed consistency
   - Con: Less readable, harder to debug

**Recommendation**: **Manual duplication for v0.2.** 
- Rationale: Only 12 new methods, duplication is manageable
- Defer macro approach to future version if maintenance becomes burden
- Aligns with constitution: "Start simple, YAGNI principles"

---

#### D3: Clone Requirement for window()

**Problem**: `window()` creates overlapping views, requiring either:
- Option A: Return slices (`&[T]`) - but ownership/lifetime issues with QueryBuilder pattern
- Option B: Clone elements - cost is O(window_size * num_windows)

**Recommendation**: **Option B - Require `Clone` bound** for `window()`.

```rust
pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> 
where 
    T: Clone
```

**Rationale**:
- Maintains ownership model (QueryBuilder owns data, consumes it)
- Explicit cost in type signature (Clone bound signals copying)
- Users aware of performance implications
- Consistent with Rust idioms (e.g., `Vec::windows()` returns slices but we can't do that with owned data)

**Constitution Alignment**: Explicit about costs (Clone requirement), no hidden allocations.

---

### State Transition Matrix

Complete state transition table for all v0.2 operations:

| Method | Initial → | Filtered → | Sorted → | Projected<U> → |
|--------|-----------|------------|----------|----------------|
| `sum()` | Terminal | Terminal | Terminal | Terminal |
| `average()` | Terminal | Terminal | Terminal | Terminal |
| `min()` | Terminal | Terminal | Terminal | Terminal |
| `max()` | Terminal | Terminal | Terminal | Terminal |
| `min_by()` | Terminal | Terminal | Terminal | Terminal |
| `max_by()` | Terminal | Terminal | Terminal | Terminal |
| `group_by()` | Terminal | Terminal | Terminal | Terminal |
| `group_by_aggregate()` | Terminal | Terminal | Terminal | Terminal |
| `partition()` | Terminal | Terminal | Terminal | Terminal |
| `distinct()` | Filtered | Filtered | Filtered | N/A (type mismatch) |
| `distinct_by()` | Filtered | Filtered | Filtered | N/A |
| `reverse()` | Filtered | Filtered | Filtered | Filtered |
| `enumerate()` | Filtered* | Filtered* | Filtered* | Filtered* |
| `zip()` | Filtered* | Filtered* | Filtered* | Filtered* |
| `chunk()` | Filtered* | Filtered* | Filtered* | Filtered* |
| `window()` | Filtered* | Filtered* | Filtered* | Filtered* |

*Note: These change the element type, so technically return `QueryBuilder<NewType, Filtered>`

---

## Dependencies

### New Dependency: num-traits

**Crate**: `num-traits = "0.2"`  
**Purpose**: Generic numeric operations (ToPrimitive for average(), Zero for sum())  
**Justification**: Standard crate for numeric trait bounds in Rust ecosystem, well-tested, minimal overhead  
**Alternative Rejected**: Implementing custom traits increases maintenance burden and reinvents well-established patterns

### Existing Dependencies (no changes)

- `proptest = "1.0"` (dev-dependencies)
- `criterion = "0.5"` (dev-dependencies)

---

## Implementation Phases

### Phase 0: Preparation
- [ ] Add `num-traits = "0.2"` to `Cargo.toml` dependencies
- [ ] Verify `cargo check` passes with new dependency
- [ ] Review `docs/implementation.md` Phase 1 for detailed examples

### Phase 1: Numeric Aggregations (P1)

**Files**:
- `src/domain/rinq/query_builder.rs`: Add aggregation methods
- `tests/rinq_v0.2_tests.rs`: Create with aggregation tests
- `benches/rinq_v0.2_benchmarks.rs`: Create with aggregation benchmarks

**Implementation Order**:
1. Implement `sum()` for `Initial` state
2. Add property tests for `sum()`
3. Implement `sum()` for `Filtered`, `Sorted`, `Projected` states
4. Implement `min()` / `max()`
5. Add property tests for `min()` / `max()`
6. Implement `average()`
7. Add property tests for `average()`
8. Implement `min_by()` / `max_by()`
9. Add property tests for key-based min/max
10. Add benchmarks comparing all aggregations to manual implementations
11. Verify zero-cost claims (≤5% overhead)

**Checkpoint**: All aggregation tests pass, benchmarks validate zero-cost

---

### Phase 2: Grouping Operations (P2)

**Files**:
- `src/domain/rinq/query_builder.rs`: Add grouping methods
- `tests/rinq_v0.2_tests.rs`: Add grouping tests

**Implementation Order**:
1. Implement `group_by()` for `Initial` state
2. Add property tests for `group_by()` (all elements accounted, groups correct, order preserved)
3. Implement `group_by()` for other states
4. Implement `group_by_aggregate()`
5. Add property tests for aggregated grouping
6. Add integration tests: `.where_().group_by()`, `.group_by()` then iterate and filter groups
7. Add benchmarks vs manual HashMap construction

**Checkpoint**: Grouping tests pass, benchmarks show acceptable overhead (≤10%)

---

### Phase 3: Deduplication (P3)

**Files**:
- `src/domain/rinq/query_builder.rs`: Add distinct methods
- `tests/rinq_v0.2_tests.rs`: Add deduplication tests

**Implementation Order**:
1. Implement `distinct()` for `Initial` state
2. Add property tests (no duplicates, first occurrence, length constraint)
3. Implement `distinct()` for other states
4. Implement `distinct_by()`
5. Add property tests for key-based distinct
6. Add integration tests: `.where_().distinct()`, `.distinct().order_by()`
7. Add benchmarks vs manual HashSet deduplication

**Checkpoint**: Distinct tests pass, benchmarks validate performance

---

### Phase 4: Sequence Transformations (P4)

**Files**:
- `src/domain/rinq/query_builder.rs`: Add sequence methods
- May need helper iterators: `ChunkIterator`, `WindowIterator`
- `tests/rinq_v0.2_tests.rs`: Add sequence tests

**Implementation Order**:
1. Implement `reverse()` for all states
2. Add property tests (`reverse(reverse(x)) == x`, order preservation)
3. Implement `ChunkIterator` helper
4. Implement `chunk()` using `ChunkIterator`
5. Add property tests for chunking
6. Implement `WindowIterator` helper
7. Implement `window()` using `WindowIterator`
8. Add property tests for windowing
9. Add benchmarks for all three operations

**Checkpoint**: Sequence transformation tests pass

---

### Phase 5: Collection Combinations (P5)

**Files**:
- `src/domain/rinq/query_builder.rs`: Add combination methods
- `tests/rinq_v0.2_tests.rs`: Add combination tests

**Implementation Order**:
1. Implement `enumerate()` for all states
2. Add property tests (indices correct, count matches)
3. Implement `zip()` for all states
4. Add property tests (shortest-wins, pairing correct)
5. Implement `partition()` for all states
6. Add property tests (all elements accounted, predicate correct)
7. Add benchmarks

**Checkpoint**: Combination tests pass

---

### Phase 6: MetricsQueryBuilder Integration

**Files**:
- `src/domain/rinq/metrics_query_builder.rs`: Extend with v0.2 methods
- `tests/rinq_integration_tests.rs`: Add metrics recording tests

**Implementation Order**:
1. Add terminal operation wrappers (sum, average, min, max, min_by, max_by, group_by, group_by_aggregate, partition)
2. Add pass-through wrappers for non-terminal operations
3. Add tests verifying metrics are recorded correctly
4. Verify metrics collection for chained operations

**Checkpoint**: Metrics integration tests pass

---

### Phase 7: Documentation & Polish

**Files**:
- `src/domain/rinq/README.md`: Add v0.2 API documentation
- `src/domain/rinq/query_builder.rs`: Add doc comments with examples for all new methods

**Implementation Order**:
1. Write doc comments for each new method (with examples)
2. Update README with v0.2 features section
3. Add usage examples for common patterns
4. Add performance characteristics documentation
5. Run `cargo doc --open` to verify documentation renders correctly
6. Run `cargo test --doc` to verify doc examples compile and run

**Checkpoint**: Documentation complete, all doc tests pass

---

### Phase 8: Final Validation

**Quality Gates**:
- [ ] All tests pass: `cargo test` (existing 115+ plus 50+ new tests)
- [ ] All benchmarks pass: `cargo bench`
- [ ] Zero clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted correctly: `cargo fmt --check`
- [ ] Documentation complete with examples
- [ ] Backwards compatibility: v0.1 code still compiles and works
- [ ] Performance validated: Benchmarks show ≤5% overhead for aggregations, ≤10% for grouping

---

## File-by-File Implementation Details

### File 1: `Cargo.toml`

**Changes**:
```toml
[dependencies]
# ... existing dependencies ...
num-traits = "0.2"  # ADD for generic numeric operations

[dev-dependencies]
# ... existing (no changes) ...

[[bench]]
name = "rinq_v0.2_benchmarks"  # ADD new benchmark suite
harness = false
```

**Testing**: Run `cargo check` after adding dependency

---

### File 2: `src/domain/rinq/query_builder.rs`

**Changes**: Add new methods to existing `impl` blocks

**Structure**:
```rust
// Existing imports
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Sum;
use num_traits::ToPrimitive;

// Existing QueryData, QueryBuilder struct (no changes)

// Extend: impl<T: 'static> QueryBuilder<T, Initial>
impl<T: 'static> QueryBuilder<T, Initial> {
    // ... existing methods (from, where_, order_by, etc.) ...
    
    // ADD: Aggregation methods (terminal)
    pub fn sum(self) -> T where T: Sum { /* ... */ }
    pub fn average(self) -> Option<f64> where T: ToPrimitive { /* ... */ }
    pub fn min(self) -> Option<T> where T: Ord { /* ... */ }
    pub fn max(self) -> Option<T> where T: Ord { /* ... */ }
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T> { /* ... */ }
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T> { /* ... */ }
    
    // ADD: Grouping methods (terminal)
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>> { /* ... */ }
    pub fn group_by_aggregate<K, R, FK, FA>(self, key_selector: FK, aggregator: FA) -> HashMap<K, R> { /* ... */ }
    
    // ADD: Transformation methods (return QueryBuilder)
    pub fn distinct(self) -> QueryBuilder<T, Filtered> where T: Eq + Hash + Clone { /* ... */ }
    pub fn distinct_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Filtered> { /* ... */ }
    pub fn reverse(self) -> QueryBuilder<T, Filtered> { /* ... */ }
    pub fn enumerate(self) -> QueryBuilder<(usize, T), Filtered> { /* ... */ }
    pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> { /* ... */ }
    pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> where T: Clone { /* ... */ }
    pub fn zip<U>(self, other: impl IntoIterator<Item = U>) -> QueryBuilder<(T, U), Filtered> { /* ... */ }
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>) { /* ... */ }
}

// Repeat for: impl<T: 'static> QueryBuilder<T, Filtered>
// Repeat for: impl<T: 'static> QueryBuilder<T, Sorted>
// Repeat for: impl<U: 'static> QueryBuilder<U, Projected<U>>

// ADD: Helper iterator adapters (at end of file)
struct ChunkIterator<T, I: Iterator<Item = T>> {
    iter: I,
    size: usize,
    buffer: Vec<T>,
}

impl<T, I: Iterator<Item = T>> Iterator for ChunkIterator<T, I> {
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> { /* ... */ }
}

struct WindowIterator<T: Clone> {
    items: Vec<T>,
    size: usize,
    position: usize,
}

impl<T: Clone> Iterator for WindowIterator<T> {
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> { /* ... */ }
}
```

**Estimated LOC**: +500-700 lines (including duplications across states)

**Testing Strategy**:
- Each method has dedicated unit tests
- Property tests verify invariants
- Integration tests verify chaining with v0.1 methods

---

### File 3: `src/domain/rinq/metrics_query_builder.rs`

**Changes**: Mirror new methods from `QueryBuilder`

**Structure**:
```rust
// Existing imports and struct (no changes)

// Extend each impl block
impl<T: 'static> MetricsQueryBuilder<T, Initial> {
    // ... existing methods ...
    
    // ADD: Terminal operations with metrics recording
    pub fn sum(self) -> T where T: Sum {
        let start = std::time::Instant::now();
        let result = self.inner.sum();
        let elapsed = start.elapsed();
        self.metrics.record_query_time("rinq.sum", elapsed);
        self.metrics.increment("rinq.operations.sum");
        result
    }
    
    // Similar for: average, min, max, min_by, max_by, group_by, group_by_aggregate, partition
    
    // ADD: Pass-through operations (no metrics)
    pub fn distinct(self) -> MetricsQueryBuilder<T, Filtered> 
    where T: Eq + Hash + Clone 
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct(),
            metrics: self.metrics,
            _state: PhantomData,
        }
    }
    
    // Similar for: distinct_by, reverse, enumerate, chunk, window, zip
}

// Repeat for other states
```

**Estimated LOC**: +300-400 lines

**Testing Strategy**:
- Integration tests verify metrics are recorded for terminal operations
- Verify pass-through operations don't record metrics until terminal operation

---

### File 4: `tests/rinq_v0.2_tests.rs` (NEW)

**Structure**:
```rust
use proptest::prelude::*;
use rusted_ca::domain::rinq::QueryBuilder;

// Property Tests

#[cfg(test)]
mod aggregation_properties {
    // Property 20: sum() correctness
    proptest! {
        #[test]
        fn prop_sum_equals_manual_sum(data: Vec<i32>) {
            let rinq_sum = QueryBuilder::from(data.clone()).sum();
            let manual_sum: i32 = data.iter().sum();
            prop_assert_eq!(rinq_sum, manual_sum);
        }
    }
    
    // Property 21: average() correctness
    // Property 22: min() correctness
    // Property 23: max() correctness
    // Property 24: min_by() correctness
    // Property 25: max_by() correctness
}

#[cfg(test)]
mod grouping_properties {
    // Property 26: group_by() completeness (all elements accounted)
    // Property 27: group_by() determinism
    // Property 28: group_by_aggregate() correctness
}

#[cfg(test)]
mod deduplication_properties {
    // Property 29: distinct() removes all duplicates
    // Property 30: distinct() preserves first occurrence
    // Property 31: distinct_by() key-based deduplication
}

#[cfg(test)]
mod sequence_properties {
    // Property 32: reverse() double reversal identity
    // Property 33: chunk() completeness
    // Property 34: window() count and overlap
}

#[cfg(test)]
mod combination_properties {
    // Property 35: zip() shortest-wins
    // Property 36: enumerate() index correctness
    // Property 37: partition() completeness
}

// Unit Tests

#[cfg(test)]
mod aggregation_unit_tests {
    // Empty collection tests
    // Single element tests
    // Boundary condition tests
}

#[cfg(test)]
mod integration_tests {
    // Chaining v0.1 and v0.2 methods
    // .where_().group_by()
    // .distinct().order_by()
    // .enumerate().where_()
}
```

**Estimated LOC**: 800-1000 lines

**Coverage Goal**: 100% of new public API surface

---

### File 5: `benches/rinq_v0.2_benchmarks.rs` (NEW)

**Structure**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_ca::domain::rinq::QueryBuilder;

fn benchmark_sum(c: &mut Criterion) {
    let data: Vec<i32> = (0..10_000).collect();
    
    c.bench_function("rinq_sum", |b| {
        b.iter(|| {
            QueryBuilder::from(data.clone()).sum()
        })
    });
    
    c.bench_function("manual_sum", |b| {
        b.iter(|| {
            data.iter().sum::<i32>()
        })
    });
}

// Similar benchmarks for:
// - average vs manual calculation
// - min/max vs manual iteration
// - group_by vs manual HashMap construction
// - distinct vs manual HashSet deduplication
// - reverse vs Vec::reverse
// - chunk vs manual chunking
// - window vs manual sliding window
// - enumerate vs manual index tracking
// - partition vs manual two-vec building

criterion_group!(
    benches,
    benchmark_sum,
    benchmark_average,
    benchmark_min_max,
    benchmark_group_by,
    benchmark_distinct,
    benchmark_reverse,
    benchmark_chunk,
    benchmark_window,
    benchmark_enumerate,
    benchmark_partition
);
criterion_main!(benches);
```

**Estimated LOC**: 500-600 lines

**Success Criteria**: All RINQ operations within 5-10% of manual implementations

---

### File 6: `src/domain/rinq/README.md`

**Changes**: Add v0.2 section after v0.1 documentation

**Structure**:
```markdown
# RINQ (Rust Integrated Query)

## Features

### v0.1 Features
[Existing documentation...]

### v0.2 Features (NEW)

#### Numeric Aggregations
[Examples for sum, average, min, max, min_by, max_by]

#### Grouping and Categorization
[Examples for group_by, group_by_aggregate]

#### Deduplication
[Examples for distinct, distinct_by]

#### Sequence Transformations
[Examples for reverse, chunk, window]

#### Collection Combinations
[Examples for zip, enumerate, partition]

## API Reference

### v0.2 Methods

| Method | State(s) | Returns | Trait Bounds |
|--------|----------|---------|--------------|
| `sum()` | All | `T` | `T: Sum` |
| `average()` | All | `Option<f64>` | `T: ToPrimitive` |
| ... (complete table) ... |

## Migration Guide: v0.1 → v0.2

- All v0.1 APIs remain unchanged
- New methods are additive only
- No breaking changes
```

**Estimated LOC**: +300-400 lines of documentation

---

## Testing Strategy

### Property-Based Tests (proptest)

**Test Categories**:

1. **Correctness Properties** (verify operations produce correct results)
   - `sum(xs) == manual_sum(xs)`
   - `average(xs) == manual_average(xs)`
   - `min(xs) == xs.iter().min()`
   - `group_by(xs, f).values().flatten().collect() == xs` (all elements accounted)
   - `distinct(xs).len() <= xs.len()` (no element addition)

2. **Invariant Properties** (verify mathematical properties)
   - `reverse(reverse(xs)) == xs` (involution)
   - `partition(xs, p) => (trues, falses)` where `trues.all(p) && falses.all(!p)`
   - `enumerate(xs).map(|(i, _)| i) == 0..xs.len()`

3. **Composition Properties** (verify chaining works correctly)
   - `.where_().sum()` filters then sums
   - `.distinct().group_by()` groups unique elements
   - `.order_by().reverse()` produces descending order

4. **Edge Case Properties**
   - Empty collections
   - Single element collections
   - All elements same value (for distinct, grouping)
   - Large collections (10K+ elements)

**Test Data Strategies**:
- `any::<Vec<i32>>()` for numeric tests
- `any::<Vec<String>>()` for string-based grouping/distinct
- Custom structs with multiple fields for key-based operations
- `prop::collection::vec(any::<i32>(), 0..1000)` for varying sizes

---

### Unit Tests

**Coverage Areas**:

1. **Empty Collections**
   - `sum([])` → identity value (0)
   - `average([])` → `None`
   - `min([])` → `None`
   - `group_by([])` → empty HashMap
   - `distinct([])` → `[]`
   - `chunk([])` → `[]`

2. **Single Element**
   - `sum([x])` → `x`
   - `average([x])` → `Some(x as f64)`
   - `min([x])` → `Some(x)`
   - `group_by([x])` → single-entry HashMap

3. **Boundary Conditions**
   - `chunk([1,2], 5)` → `[[1, 2]]` (chunk size > length)
   - `window([1,2], 5)` → `[]` (window size > length)
   - `zip(long, short)` → stops at short length

4. **Special Cases**
   - `distinct([5, 5, 5, 5])` → `[5]` (all duplicates)
   - `group_by(all_same_key)` → single group
   - `partition([], pred)` → `([], [])`

---

### Integration Tests

**Test Scenarios**:

1. **v0.1 + v0.2 Chaining**
   ```rust
   // Filter → Group → Aggregate
   QueryBuilder::from(orders)
       .where_(|o| o.amount > 100.0)
       .group_by_aggregate(
           |o| o.user_id,
           |group| group.iter().map(|o| o.amount).sum::<f64>()
       );
   ```

2. **Metrics Recording**
   ```rust
   // Verify MetricsQueryBuilder records metrics for new operations
   let metrics = MetricsCollector::new();
   let _result = MetricsQueryBuilder::from(data, metrics.clone())
       .where_(|x| x > 5)
       .sum();
   assert_eq!(metrics.get_count("rinq.operations.sum"), 1);
   ```

3. **Error Propagation**
   - Verify panic behavior for invalid chunk/window sizes
   - Document panic conditions in API docs

---

### Benchmarks

**Benchmark Suite**:

| Benchmark | RINQ Operation | Manual Baseline |
|-----------|----------------|-----------------|
| `sum_comparison` | `.sum()` | `iter().sum()` |
| `average_comparison` | `.average()` | manual sum/count |
| `min_max_comparison` | `.min()` / `.max()` | `iter().min()` |
| `group_by_comparison` | `.group_by()` | manual HashMap build |
| `distinct_comparison` | `.distinct()` | manual HashSet dedup |
| `reverse_comparison` | `.reverse()` | `Vec::reverse()` |
| `chunk_comparison` | `.chunk(n)` | manual chunking loop |
| `window_comparison` | `.window(n)` | manual sliding window |
| `enumerate_comparison` | `.enumerate()` | manual index tracking |
| `partition_comparison` | `.partition()` | manual two-vec build |

**Dataset Sizes**:
- Small: 100 elements
- Medium: 1,000 elements
- Large: 10,000 elements
- Extra-large: 100,000 elements (for operations that should scale linearly)

**Success Criteria**:
- Aggregations: ≤5% overhead vs manual
- Grouping: ≤10% overhead vs manual HashMap
- Transformations: ≤5% overhead vs manual

---

## Complexity Tracking

> **All decisions align with Constitution - no violations to justify.**

| Decision | Rationale | Alternative Considered |
|----------|-----------|------------------------|
| Add `num-traits` dependency | Provides robust, type-safe numeric operations without reinventing trait ecosystem | Custom traits - rejected due to maintenance burden |
| Manual method duplication across states | Only 12 methods, explicit and clear | Macro generation - deferred to future (YAGNI) |
| No new type states | All operations fit existing states naturally | New `Grouped`, `Chunked` states - rejected as unnecessary complexity |
| No new QueryData variants | Terminal ops return directly, others use Iterator | New variants - rejected as premature |

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Performance regression in v0.1 operations | Low | High | Comprehensive benchmark suite runs on all operations |
| Trait bound conflicts | Medium | Medium | Careful design, explicit `where` clauses, integration testing |
| State transition bugs | Low | High | Property tests verify state transitions, type system prevents invalid states |
| Clone overhead in `window()` | Medium | Low | Document explicitly, benchmark to quantify, users can choose not to use if cost is concern |

### Process Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep (adding more features) | Medium | Medium | Strict adherence to spec, mark additional ideas as v0.3 candidates |
| Test writing takes longer than implementation | Low | Low | Parallel test writing (property tests while implementing) |

---

## Timeline Estimation

**Not providing specific timeline per constitution guidance.** Implementation phases ordered for logical dependency flow:

1. Preparation (dependency addition)
2. P1: Numeric aggregations (most impactful, simplest)
3. P2: Grouping (builds on aggregations)
4. P3: Deduplication (independent)
5. P4: Sequence transformations (moderate complexity)
6. P5: Collection combinations (simplest, but least impactful)
7. Metrics integration (wraps all above)
8. Documentation and polish

**Phases 3-5 can potentially run in parallel** if multiple developers or if implementation is straightforward.

---

## Success Validation Checklist

Before considering v0.2 complete:

- [ ] All 50+ new property tests pass
- [ ] All 30+ new unit tests pass
- [ ] All 15+ new benchmarks show ≤5-10% overhead
- [ ] All existing 115+ v0.1 tests still pass (backwards compatibility)
- [ ] `cargo clippy -- -D warnings` produces zero warnings
- [ ] `cargo fmt --check` passes
- [ ] Documentation includes examples for every new method
- [ ] `cargo doc --open` renders correctly
- [ ] Integration tests verify metrics recording
- [ ] README.md updated with v0.2 features
- [ ] Migration guide confirms no breaking changes

---

## Next Steps

After this plan is approved:

1. Run `/speckit.tasks` to generate detailed task breakdown from this plan
2. Review and prioritize tasks
3. Run `/speckit.implement` to execute implementation phase
4. Iterate on test failures and performance tuning

---

**Plan Status**: ✅ Ready for Task Breakdown  
**Estimated Implementation Scope**: 
- +500-700 LOC in `query_builder.rs`
- +300-400 LOC in `metrics_query_builder.rs`
- +800-1000 LOC in `tests/rinq_v0.2_tests.rs`
- +500-600 LOC in `benches/rinq_v0.2_benchmarks.rs`
- +300-400 LOC in documentation

**Total**: ~2400-3100 new lines of code
