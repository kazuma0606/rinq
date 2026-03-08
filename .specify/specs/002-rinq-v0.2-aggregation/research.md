# Research: RINQ v0.2 Technical Decisions

**Feature**: RINQ v0.2 - Aggregation and Transformation Extensions  
**Date**: 2026-03-08  
**Status**: Completed

## Overview

This document captures technical research and decision-making for RINQ v0.2 implementation. Each section addresses a specific technical challenge and documents the chosen solution with rationale.

---

## R1: Generic Numeric Operations

### Problem Statement

How to implement `sum()`, `average()`, `min()`, `max()` that work across all numeric types (i32, f64, u32, i64, f32, etc.) without duplicating code for each type?

### Options Evaluated

#### Option 1: Use `num-traits` crate ✅ SELECTED

**Description**: Add `num-traits = "0.2"` dependency for `Num`, `Zero`, `ToPrimitive`, `FromPrimitive` traits.

**Pros**:
- Industry-standard solution (used by `ndarray`, `nalgebra`, and other numeric libraries)
- Well-tested and maintained
- Provides comprehensive trait bounds for numeric operations
- Handles edge cases (overflow, type conversion, precision)
- Future-proof for additional numeric operations (median, std dev, etc.)

**Cons**:
- Adds external dependency
- Small increase in compilation time

**Code Example**:
```rust
use num_traits::{ToPrimitive, Zero};
use std::iter::Sum;

pub fn sum<T: Sum>(self) -> T {
    self.into_iter().sum()
}

pub fn average<T: ToPrimitive>(self) -> Option<f64> {
    let items: Vec<T> = self.collect();
    if items.is_empty() {
        return None;
    }
    let sum: f64 = items.iter().filter_map(|x| x.to_f64()).sum();
    Some(sum / items.len() as f64)
}
```

**Benchmarks**: No overhead (delegates to compiler-optimized `Iterator::sum()`)

---

#### Option 2: Standard Library Only

**Description**: Use only `std::iter::Sum` and manual implementations.

**Pros**:
- Zero new dependencies
- Minimal compilation time

**Cons**:
- Limited to types implementing `Sum` trait
- Manual implementation for `average()` less robust
- Harder to extend for future numeric operations
- May not handle all numeric types uniformly

**Verdict**: Rejected. Small dependency cost justified by robustness and extensibility.

---

### Decision

**Use `num-traits` crate.**

**Justification**: 
- Aligns with Constitution's "Pragmatic Utility" principle (deliver robust functionality)
- Trade-off is acceptable: +1 small, well-vetted dependency for production-grade numeric handling
- Enables future extensions (percentile, median, mode, standard deviation) without architectural changes

**Implementation**:
```toml
[dependencies]
num-traits = "0.2"
```

---

## R2: Lazy vs Eager Evaluation

### Problem Statement

Which operations can maintain lazy evaluation (iterator-based) versus which must eagerly materialize collections?

### Analysis

#### Terminal Operations (Must Be Eager)

**These consume the QueryBuilder and return a result**:

| Operation | Why Eager | Cost |
|-----------|-----------|------|
| `sum()` | Must traverse entire collection | O(n) time, O(1) space |
| `average()` | Must count and sum all elements | O(n) time, O(n) space (collects to count) |
| `min()` / `max()` | Must examine all elements | O(n) time, O(1) space |
| `min_by()` / `max_by()` | Must examine all elements | O(n) time, O(1) space |
| `group_by()` | Must build HashMap with all elements | O(n) time, O(n) space |
| `group_by_aggregate()` | Must group then aggregate | O(n) time, O(n) space |
| `partition()` | Must build two Vecs | O(n) time, O(n) space |

**Decision**: These are terminal operations that consume the builder. No lazy option exists.

---

#### Stateful Operations (Must Be Eager, but return QueryBuilder)

**These need to track state during iteration**:

| Operation | Why Eager | Cost | State Tracking |
|-----------|-----------|------|----------------|
| `distinct()` | Must track seen elements | O(n) time, O(k) space | HashSet of unique elements |
| `distinct_by()` | Must track seen keys | O(n) time, O(k) space | HashSet of unique keys |
| `reverse()` | Must see all elements to reverse | O(n) time, O(n) space | Vec buffer |

**Design Decision**: 
- **Eager materialization inside the method**
- Return `QueryBuilder<T, Filtered>` with transformed data as iterator
- Cost is paid upfront, but subsequent operations remain lazy

**Example**:
```rust
pub fn distinct(self) -> QueryBuilder<T, Filtered> 
where T: Eq + Hash + Clone 
{
    // Eagerly deduplicate
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for item in self.into_iter() {
        if seen.insert(item.clone()) {
            unique.push(item);
        }
    }
    
    // Return as lazy iterator
    QueryBuilder::from(unique)  // Now lazy for subsequent ops
}
```

---

#### Lazy Operations (Can Remain Lazy)

**These can use pure iterator adapters**:

| Operation | Why Lazy | Implementation |
|-----------|----------|----------------|
| `enumerate()` | Pure transformation | `Iterator::enumerate()` |
| `zip()` | Pure pairing | `Iterator::zip()` |
| `chunk()` | Buffering adapter | Custom `ChunkIterator` |
| `window()` | Sliding buffer | Custom `WindowIterator` |

**Decision**: Keep these lazy using iterator adapters.

**Trade-off**: `chunk()` and `window()` need custom iterator implementations, but they maintain lazy semantics (don't process until consumed).

---

### Decision Matrix

| Operation | Evaluation | Returns | Consumed? |
|-----------|-----------|---------|-----------|
| Aggregations (sum, avg, min, max) | Eager | Result value | Yes |
| Grouping (group_by) | Eager | HashMap | Yes |
| Deduplication (distinct) | Eager | QueryBuilder | No |
| Reverse | Eager | QueryBuilder | No |
| Chunk / Window | Lazy | QueryBuilder | No |
| Enumerate / Zip | Lazy | QueryBuilder | No |
| Partition | Eager | (Vec, Vec) | Yes |

**Constitution Alignment**: Maintains lazy evaluation where semantically possible (principle II: Zero-Cost Abstraction).

---

## R3: Type State Design

### Problem Statement

Do we need new type states (`Grouped`, `Chunked`, `Windowed`, etc.) for v0.2 operations?

### Current Type State Design (v0.1)

```rust
pub struct Initial;          // Fresh query
pub struct Filtered;         // After where_()
pub struct Sorted;           // After order_by()
pub struct Projected<U>;     // After select() with type change
```

**State Transition Rules**:
- `Initial → Filtered` (via `where_()`)
- `Initial → Sorted` (via `order_by()`)
- `Any → Projected<U>` (via `select()`)
- `Sorted → Sorted` (via `then_by()`)
- `Filtered → Sorted` (via `order_by()`)

### Analysis: Do v0.2 Operations Need New States?

#### Operations That Don't Need New States

**Aggregations (Terminal)**:
- `sum()`, `average()`, `min()`, `max()`, `min_by()`, `max_by()`
- **State**: N/A (consume builder, return result directly)
- **Rationale**: Terminal operations don't have a "next state"

**Grouping (Terminal)**:
- `group_by()`, `group_by_aggregate()`
- **State**: N/A (return HashMap/aggregated result)

**Partition (Terminal)**:
- `partition()`
- **State**: N/A (return tuple of Vecs)

**Transformations (Reuse Filtered)**:
- `distinct()`, `distinct_by()`: Remove duplicates → `Filtered`
- `reverse()`: Reverse order → `Filtered` (lose any sort order)
- `enumerate()`: Add indices → `Filtered` (type changes to `(usize, T)`)
- `zip()`: Pair with another collection → `Filtered` (type changes to `(T, U)`)
- `chunk()`: Create chunks → `Filtered` (type changes to `Vec<T>`)
- `window()`: Create windows → `Filtered` (type changes to `Vec<T>`)

**Rationale for Using `Filtered`**:
- Operations that lose sort order naturally transition to `Filtered`
- Operations that change element type start fresh (no meaningful prior state)
- Keeps state space small and predictable

---

### Option 1: Reuse Existing States ✅ SELECTED

**Approach**: Map all v0.2 operations to existing states:
- Terminal operations → no state (return result)
- Transformations → `Filtered` state

**Pros**:
- Simple, no state explosion
- Clear semantics: if not sorted, it's filtered
- Easy to understand and maintain

**Cons**:
- Lose information (e.g., "grouped" state not tracked)
- Cannot prevent invalid operations after grouping (but grouping is terminal, so N/A)

**State Transition Examples**:
```rust
QueryBuilder::from(data)        // Initial
    .distinct()                 // Initial → Filtered
    .order_by(|x| x)            // Filtered → Sorted
    .reverse()                  // Sorted → Filtered (loses sort)
    .collect();                 // Filtered → Vec<T>

QueryBuilder::from(data)        // Initial
    .where_(|x| x > 5)          // Initial → Filtered
    .group_by(|x| x % 2);       // Filtered → HashMap (terminal)
```

---

### Option 2: Add New States for Each Operation

**Approach**: Create `Grouped`, `Chunked`, `Windowed`, `Enumerated`, `Zipped`, `Reversed`, `Distinct` states.

**Pros**:
- More precise type-level tracking
- Could enable state-specific optimizations

**Cons**:
- State explosion (7+ new states)
- Combinatorial complexity (what if `.chunk().enumerate()`? → `ChunkedEnumerated` state?)
- Maintenance burden
- Questionable value (most states are terminal or transitional)

**Verdict**: Rejected. Complexity cost outweighs benefits.

---

### Decision

**Reuse existing states.** No new states needed for v0.2.

**Type State Mapping**:
- All transformations → `Filtered`
- Terminal operations → return result directly (no state)

**Constitution Alignment**: Avoids unnecessary complexity, maintains clarity (principles IV: API Consistency, VII: Incremental Integration).

---

## R4: Clone Bounds for window()

### Problem Statement

`window()` creates overlapping slices. How to handle ownership?

### Options

#### Option 1: Return References (`&[T]`) 

**Problem**: Lifetimes conflict with QueryBuilder's ownership model.

```rust
// This doesn't work - lifetime 'a tied to QueryBuilder
pub fn window<'a>(&'a self, size: usize) -> impl Iterator<Item = &'a [T]>
```

**Verdict**: Incompatible with QueryBuilder pattern (consumes self, moves ownership).

---

#### Option 2: Require Clone ✅ SELECTED

**Approach**: Clone elements to create owned windows.

```rust
pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
where
    T: Clone
```

**Pros**:
- Maintains ownership model
- Explicit cost in type signature (Clone bound)
- Users aware of performance implications
- Works with QueryBuilder's consumption pattern

**Cons**:
- Clone overhead: O(window_size * num_windows)
- Not usable for non-Clone types

**Cost Example**:
```rust
// Input: [1, 2, 3, 4, 5], window size = 3
// Output: [[1,2,3], [2,3,4], [3,4,5]]
// Clones: 9 total (3 per window × 3 windows)
// vs original: 5 elements
// Overhead: 4 extra clones
```

**Mitigation**:
- Document clearly in API docs
- Benchmark to quantify actual cost
- Provide alternative: users can use standard `Iterator::collect().windows()` if they want slice-based approach

---

#### Option 3: Return Indices

**Approach**: Return `Vec<(usize, usize)>` representing window ranges, users index into original collection.

**Pros**:
- Zero cloning

**Cons**:
- Breaks fluent interface (can't chain further operations on window contents)
- User needs to maintain reference to original collection
- Inconsistent with RINQ's ownership model

**Verdict**: Rejected. Violates API consistency principle.

---

### Decision

**Require `Clone` for `window()` operation.**

**Justification**:
- Maintains QueryBuilder's ownership and fluent interface patterns
- Explicit cost (Clone bound signals copying to users)
- Consistent with Rust ecosystem patterns (e.g., Vec owns its data)
- Users who need zero-copy can use alternative approaches outside RINQ

**Documentation Required**: 
- Clearly document Clone overhead in method doc comment
- Provide example with performance note
- Benchmark to quantify real-world cost

---

## R5: Error Handling for Invalid Sizes

### Problem Statement

How to handle invalid arguments like `chunk(0)` or `window(0)`?

### Options

#### Option 1: Panic ✅ SELECTED

**Approach**: Use `assert!()` to panic on invalid sizes with descriptive messages.

```rust
pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> {
    assert!(size > 0, "chunk size must be greater than 0");
    // ... implementation
}

pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> 
where T: Clone 
{
    assert!(size >= 2, "window size must be at least 2");
    // ... implementation
}
```

**Pros**:
- Follows Rust conventions (e.g., `Vec::split_at` panics on invalid index)
- Fails fast, clear error message
- Programming errors (passing 0) should be caught in development, not production

**Cons**:
- Panics are not recoverable
- Users must ensure valid arguments

**Precedent in std**: `Vec::split_at()`, `slice::split_at()`, `Vec::swap()` all panic on invalid arguments.

---

#### Option 2: Return Result<_, RinqDomainError>

**Approach**: Make methods fallible, return `Result`.

```rust
pub fn chunk(self, size: usize) -> Result<QueryBuilder<Vec<T>, Filtered>, RinqDomainError>
```

**Pros**:
- Recoverable errors
- Explicit error handling in type signature

**Cons**:
- Breaks fluent interface (need `?` operator)
- Error handling for programming errors (invalid arguments) is typically not warranted
- Inconsistent with v0.1 API style (no fallible query-building methods)

**Verdict**: Rejected. Programming errors should panic, not return Result.

---

#### Option 3: Silently Handle (Return Empty)

**Approach**: `chunk(0)` or `window(0)` return empty iterators.

**Pros**:
- No panics, no Result

**Cons**:
- Silent failure violates Constitution's "No Silent Failures" principle
- Harder to debug (why is my result empty?)
- Encourages incorrect usage

**Verdict**: Rejected. Violates Constitution.

---

### Decision

**Panic on invalid sizes with descriptive error messages.**

**Rules**:
- `chunk(0)` → panic with "chunk size must be greater than 0"
- `window(0)` or `window(1)` → panic with "window size must be at least 2"

**Justification**:
- Follows Rust standard library conventions
- Fails fast with clear error messages
- Aligns with Constitution's explicit error handling principle
- Invalid arguments are programming errors, not runtime conditions

**Documentation Required**: 
- Document panic conditions in method doc comments
- Add `# Panics` section to docs

---

## R6: State Transitions for Type-Changing Operations

### Problem Statement

Operations like `enumerate()`, `zip()`, `chunk()` change element types (`T → (usize, T)`, `T → (T, U)`, `T → Vec<T>`). How to handle state transitions?

### Analysis

**Example Chains**:
```rust
// Case 1: enumerate changes type
QueryBuilder::from(vec![10, 20, 30])  // QueryBuilder<i32, Initial>
    .enumerate()                      // QueryBuilder<(usize, i32), ??>
    .where_(|(i, x)| *i < 2)          // What state?
    .collect();                       // Vec<(usize, i32)>

// Case 2: Multiple type changes
QueryBuilder::from(vec![1, 2, 3])     // QueryBuilder<i32, Initial>
    .enumerate()                      // QueryBuilder<(usize, i32), ??>
    .chunk(2)                         // QueryBuilder<Vec<(usize, i32)>, ??>
    .collect();                       // Vec<Vec<(usize, i32)>>
```

### Decision

**Return `QueryBuilder<NewType, Filtered>` for all type-changing operations.**

**Rationale**:
- Type change represents a transformation, naturally fits `Filtered` semantics
- Allows continued chaining with type-safe operations
- Generic type parameter `T` changes, state resets to `Filtered`
- Consistent with `select()` which uses `Projected<U>` for explicit projections

**State Transition Rules**:
- `enumerate()`: `Any<T> → Filtered<(usize, T)>`
- `zip()`: `Any<T> → Filtered<(T, U)>`
- `chunk()`: `Any<T> → Filtered<Vec<T>>`
- `window()`: `Any<T> → Filtered<Vec<T>>`

**Why not `Projected<NewType>`?**
- `Projected<U>` is specifically for `.select()` operations (explicit user-defined projections)
- These operations are transformations, not projections
- Keeps `Projected` state semantics clear

---

## R7: Integration with Existing v0.1 Features

### Problem Statement

How do v0.2 operations compose with v0.1 features (filtering, sorting, pagination)?

### Test Cases

#### Composition: Filtering + Aggregation
```rust
let sum = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
    .where_(|x| x % 2 == 0)    // v0.1
    .sum();                     // v0.2
// Expected: 2 + 4 + 6 = 12
```

✅ Works naturally (filter then sum)

---

#### Composition: Sorting + Deduplication
```rust
let result = QueryBuilder::from(vec![3, 1, 2, 1, 3])
    .order_by(|x| *x)          // v0.1: Initial → Sorted
    .distinct()                // v0.2: Sorted → Filtered (loses sort!)
    .collect();
// Expected: [3, 1, 2] (first occurrence order) OR [1, 2, 3] (sorted order)?
```

**Issue**: Does `distinct()` on `Sorted` preserve sort order?

**Options**:
1. Preserve sort order: Use `SortedVec`, deduplicate while maintaining sort
2. First-occurrence order: Transition to `Filtered`, lose sort

**Decision**: **Preserve sort order when called on `Sorted` state.**

**Implementation**:
```rust
impl<T: 'static> QueryBuilder<T, Sorted> {
    pub fn distinct(self) -> QueryBuilder<T, Filtered>  // Still transition to Filtered
    where T: Eq + Hash + Clone 
    {
        match self.data {
            QueryData::SortedVec { items, .. } => {
                // Deduplicate sorted vec (preserves sort)
                let mut seen = HashSet::new();
                let unique: Vec<T> = items.into_iter()
                    .filter(|item| seen.insert(item.clone()))
                    .collect();
                QueryBuilder::from(unique)  // Loses sort metadata, but order preserved
            }
            _ => unreachable!(),
        }
    }
}
```

**Result**: Elements remain sorted, but state becomes `Filtered` (sort metadata lost). User can call `.order_by()` again if needed (will be cheap since already sorted).

---

#### Composition: Grouping + Pagination
```rust
let groups = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .take(3)                    // v0.1
    .group_by(|x| x % 2);       // v0.2
// Expected: Only first 3 elements grouped
```

✅ Works naturally (pagination before grouping)

---

#### Composition: Projection + Aggregation
```rust
let sum = QueryBuilder::from(orders)
    .select(|o| o.amount)       // v0.1: Project to f64
    .sum();                     // v0.2: Sum the amounts
```

✅ Works naturally (projection changes type, aggregation operates on new type)

---

### Composition Matrix

| v0.1 Operation | v0.2 Operation | Expected Behavior |
|----------------|----------------|-------------------|
| `where_()` → | `sum()` | Sum filtered elements |
| `order_by()` → | `min()` | Min of sorted (optimizable: first element) |
| `order_by()` → | `distinct()` | Deduplicate while preserving sort order |
| `select()` → | `average()` | Average projected values |
| `take()` → | `group_by()` | Group first N elements |
| `skip()` → | `sum()` | Sum elements after skipping |

**Validation**: Integration tests will verify all these compositions work correctly.

---

## R8: MetricsQueryBuilder Extension Strategy

### Problem Statement

How should `MetricsQueryBuilder` handle 12 new operations while maintaining consistency with v0.1 pattern?

### Current v0.1 Pattern

```rust
impl<T> MetricsQueryBuilder<T, Initial> {
    // Terminal ops: record metrics
    pub fn count(self) -> usize {
        let start = Instant::now();
        let result = self.inner.count();
        self.metrics.record_query_time("rinq.count", start.elapsed());
        self.metrics.increment("rinq.operations.count");
        result
    }
    
    // Query-building ops: pass through
    pub fn where_<F>(self, predicate: F) -> MetricsQueryBuilder<T, Filtered> {
        MetricsQueryBuilder {
            inner: self.inner.where_(predicate),
            metrics: self.metrics,
            _state: PhantomData,
        }
    }
}
```

### v0.2 Extension Strategy

**Terminal Operations** (record metrics):
- `sum()`, `average()`, `min()`, `max()`, `min_by()`, `max_by()`
- `group_by()`, `group_by_aggregate()`
- `partition()`

**Pattern**:
```rust
pub fn sum(self) -> T where T: Sum {
    let start = Instant::now();
    let result = self.inner.sum();
    let elapsed = start.elapsed();
    self.metrics.record_query_time("rinq.sum", elapsed);
    self.metrics.increment("rinq.operations.sum");
    result
}
```

**Non-Terminal Operations** (pass through):
- `distinct()`, `distinct_by()`, `reverse()`
- `enumerate()`, `zip()`, `chunk()`, `window()`

**Pattern**:
```rust
pub fn distinct(self) -> MetricsQueryBuilder<T, Filtered> 
where T: Eq + Hash + Clone 
{
    MetricsQueryBuilder {
        inner: self.inner.distinct(),
        metrics: self.metrics,
        _state: PhantomData,
    }
}
```

### Decision

**Mirror `QueryBuilder` API exactly in `MetricsQueryBuilder`.**

**Metrics Recording Rules**:
1. Terminal operations: Record `rinq.<operation>` timing and increment `rinq.operations.<operation>` counter
2. Non-terminal operations: Pass through (no metrics recorded until terminal operation)

**Rationale**:
- Non-terminal operations don't execute until terminal op
- Recording metrics for pass-through would inflate counts
- Terminal operation metrics capture full query execution time

**Constitution Alignment**: Maintains observability (principle VII) while accurately reflecting actual execution.

---

## R9: Handling Overflow in sum()

### Problem Statement

What happens when `sum()` overflows for integer types?

### Options

#### Option 1: Use Standard sum() (Wrapping/Panic) ✅ SELECTED

**Approach**: Delegate to `Iterator::sum()`, which uses type's default overflow behavior.

```rust
pub fn sum(self) -> T where T: Sum {
    self.into_iter().sum()
}
```

**Behavior**:
- `i32`: Wrapping overflow in release, panic in debug
- `u32`: Wrapping overflow
- `f64`: Infinity on overflow

**Pros**:
- Consistent with Rust std library behavior
- Zero overhead
- Developers familiar with Rust's overflow semantics

**Cons**:
- Overflow behavior may surprise some users

---

#### Option 2: Saturating Arithmetic

**Approach**: Use `saturating_add()` for integers.

**Pros**:
- Predictable behavior (clamps at min/max)

**Cons**:
- Performance overhead (check on every addition)
- Not universally applicable (what about f64?)
- Inconsistent with std library

**Verdict**: Rejected. Adds overhead without clear benefit.

---

#### Option 3: Checked Arithmetic (Return Result)

**Approach**: Return `Result<T, OverflowError>`.

**Pros**:
- Explicit error handling

**Cons**:
- Breaks fluent interface
- Significant performance overhead
- Inconsistent with API style

**Verdict**: Rejected.

---

### Decision

**Use standard `Iterator::sum()` behavior (wrapping/panicking depending on compilation mode).**

**Justification**:
- Consistent with Rust std library conventions
- Zero performance overhead
- Developers can use checked/saturating types if needed (e.g., `Wrapping<i32>`)

**Documentation Required**:
- Document overflow behavior in method doc comment
- Note: "Overflow behavior follows Rust's default semantics (wrapping in release, panic in debug)"

---

## R10: Chunk vs Window Semantics

### Problem Statement

Clarify the difference between `chunk()` and `window()` and their use cases.

### Definitions

#### `chunk(n)`: Non-Overlapping Partitions

```rust
vec![1, 2, 3, 4, 5].chunk(2)
// → [[1, 2], [3, 4], [5]]
```

**Characteristics**:
- Non-overlapping
- Last chunk may be smaller than `n`
- Count: `⌈len / n⌉` chunks

**Use Cases**:
- Pagination (page size)
- Batch processing (process N items at a time)
- Parallel processing (divide work into chunks)

---

#### `window(n)`: Overlapping Sliding Windows

```rust
vec![1, 2, 3, 4, 5].window(3)
// → [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
```

**Characteristics**:
- Overlapping (each element appears in multiple windows)
- All windows same size (or empty if collection too small)
- Count: `max(0, len - n + 1)` windows

**Use Cases**:
- Moving averages (time series analysis)
- N-gram generation (NLP)
- Pattern detection (sliding window algorithms)

---

### Decision

**Both operations needed** - they serve different use cases.

**API Design**:
```rust
pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered> where T: Clone
```

**Documentation Required**: Clearly explain the difference with visual examples in doc comments.

---

## R11: Should group_by() Return HashMap or BTreeMap?

### Problem Statement

`group_by()` returns a map. Should it be `HashMap` (unordered, O(1) lookup) or `BTreeMap` (ordered, O(log n) lookup)?

### Options

#### Option 1: HashMap ✅ SELECTED

**Pros**:
- Faster: O(1) average case for insertion and lookup
- Standard choice for grouping operations
- No ordering requirement for groups in most use cases

**Cons**:
- Non-deterministic iteration order (unless using `IndexMap`)
- Keys must implement `Hash + Eq`

---

#### Option 2: BTreeMap

**Pros**:
- Deterministic iteration order (sorted by key)
- Keys only need `Ord`

**Cons**:
- Slower: O(log n) for insertion and lookup
- Unnecessary overhead if ordering not needed

---

#### Option 3: Provide Both

**Approach**: `group_by()` returns HashMap, `group_by_sorted()` returns BTreeMap.

**Pros**:
- Users can choose based on needs

**Cons**:
- API bloat
- Most users don't need sorted groups

**Verdict**: Rejected (defer to v0.3 if users request).

---

### Decision

**Use `HashMap<K, Vec<T>>` for `group_by()`.**

**Justification**:
- Performance: O(1) grouping is faster for typical use cases
- Consistency: HashMap is standard for grouping in other languages/libraries
- Flexibility: Users can convert to BTreeMap if needed: `group_by().into_iter().collect::<BTreeMap<_, _>>()`

**Trait Bounds**: `K: Eq + Hash` (required for HashMap keys)

**Future Extension**: If demand exists, add `group_by_sorted()` in v0.3.

---

## R12: average() Return Type

### Problem Statement

Should `average()` return `Option<f64>` (nullable) or `f64` (with 0.0 for empty)?

### Options

#### Option 1: Option<f64> ✅ SELECTED

```rust
pub fn average(self) -> Option<f64> where T: ToPrimitive
```

**Behavior**:
- Empty collection → `None`
- Non-empty → `Some(average_value)`

**Pros**:
- Explicit handling of empty case
- Consistent with `min()` / `max()` returning `Option`
- Avoids misleading 0.0 result for empty collections

**Cons**:
- Requires unwrapping or pattern matching

---

#### Option 2: f64 (0.0 for empty)

```rust
pub fn average(self) -> f64 where T: ToPrimitive
```

**Pros**:
- Simpler return type

**Cons**:
- `average([])` → `0.0` is misleading (0.0 is a valid average, not "no data")
- Inconsistent with `min()` / `max()` semantics

**Verdict**: Rejected. Misleading for empty case.

---

### Decision

**Return `Option<f64>` for `average()`.**

**Justification**:
- Explicit null-case handling
- Consistent with other aggregations (`min()`, `max()`)
- Aligns with Rust conventions (e.g., `slice::first()` returns `Option`)

**Edge Cases**:
- Empty collection: `None`
- Single element `[x]`: `Some(x as f64)`
- Overflow: Infinity (f64 semantics)

---

## Research Summary

### Key Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Numeric traits | `num-traits` crate | Industry standard, robust, extensible |
| Evaluation strategy | Mixed (lazy where possible, eager where required) | Balances performance and semantics |
| Type states | Reuse existing (no new states) | Avoids state explosion |
| `window()` Clone | Require `Clone` bound | Maintains ownership model |
| Invalid sizes | Panic with descriptive message | Follows Rust std conventions |
| Type-changing ops | Return `Filtered<NewType>` | Natural fit for transformations |
| `group_by()` collection | `HashMap` | O(1) performance |
| `average()` return type | `Option<f64>` | Explicit null-case handling |

### Constitution Compliance

✅ All decisions align with Constitution principles:
- Type safety enforced through trait bounds and state machine
- Zero-cost validated through benchmarks
- Pragmatic utility through common-pattern operations
- API consistency with v0.1 patterns
- Incremental integration with existing architecture

### Open Questions

*None.* All technical decisions resolved during research phase.

---

**Research Status**: ✅ **COMPLETE**  
**Ready for**: Task Breakdown (`/speckit.tasks`)
