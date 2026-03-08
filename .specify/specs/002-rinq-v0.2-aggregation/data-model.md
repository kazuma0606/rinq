# Data Model: RINQ v0.2 Type System

**Feature**: RINQ v0.2 - Aggregation and Transformation Extensions  
**Date**: 2026-03-08

## Overview

RINQ v0.2 extends the existing type-state pattern with new operations while maintaining backwards compatibility with v0.1. This document defines type signatures, trait bounds, state transitions, and internal data structures.

---

## Type State Machine

### Existing States (v0.1 - Unchanged)

```rust
/// Initial state - query just created from a collection
pub struct Initial;

/// Filtered state - query has been filtered with where_()
pub struct Filtered;

/// Sorted state - query has been sorted with order_by()
pub struct Sorted;

/// Projected state - query has been transformed with select()
pub struct Projected<U> {
    _phantom: PhantomData<U>,
}
```

### State Transition Diagram (v0.2 Extended)

```
                    ┌─────────┐
                    │ Initial │
                    └────┬────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
      where_()       order_by()      select<U>()
         │               │               │
         ▼               ▼               ▼
    ┌──────────┐   ┌─────────┐   ┌──────────────┐
    │ Filtered │   │ Sorted  │   │ Projected<U> │
    └────┬─────┘   └────┬────┘   └──────┬───────┘
         │              │                │
    order_by()     then_by()        [terminal ops]
         │              │                │
         ▼              ▼                ▼
    ┌─────────┐   ┌─────────┐      [Result]
    │ Sorted  │   │ Sorted  │
    └─────────┘   └─────────┘

    NEW v0.2 Operations:
    
    Any State ──sum()───────────→ [T]
    Any State ──average()───────→ [Option<f64>]
    Any State ──min()/max()─────→ [Option<T>]
    Any State ──group_by()──────→ [HashMap<K, Vec<T>>]
    Any State ──partition()─────→ [(Vec<T>, Vec<T>)]
    
    Any State ──distinct()──────→ Filtered
    Any State ──reverse()───────→ Filtered
    Any State ──enumerate()─────→ Filtered (type: (usize, T))
    Any State ──zip(U)──────────→ Filtered (type: (T, U))
    Any State ──chunk(n)────────→ Filtered (type: Vec<T>)
    Any State ──window(n)───────→ Filtered (type: Vec<T>)
```

**Key Principle**: All v0.2 transformations transition to `Filtered` state. Terminal operations consume the builder and return results directly.

---

## QueryBuilder Core Structure (Unchanged)

```rust
pub struct QueryBuilder<T, State> {
    data: QueryData<T>,
    _state: PhantomData<State>,
}

enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec {
        items: Vec<T>,
        comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    },
}
```

**v0.2 Note**: No new `QueryData` variants needed. All operations can be implemented using existing `Iterator` and `SortedVec` variants.

---

## Type Signatures by Operation Group

### Group 1: Numeric Aggregations (Terminal Operations)

#### sum()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn sum(self) -> T
    where
        T: std::iter::Sum,
    {
        // Implementation
    }
}
```

**Trait Bounds**: `T: Sum`  
**Return Type**: `T` (same as collection element type)  
**States**: Available on `Initial`, `Filtered`, `Sorted`, `Projected<T>`  
**Consumes**: Yes (terminal operation)

**Example**:
```rust
let total: i32 = QueryBuilder::from(vec![1, 2, 3, 4, 5]).sum();
// total = 15
```

---

#### average()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn average(self) -> Option<f64>
    where
        T: num_traits::ToPrimitive,
    {
        // Implementation
    }
}
```

**Trait Bounds**: `T: ToPrimitive` (from `num-traits` crate)  
**Return Type**: `Option<f64>`  
- `None` for empty collections
- `Some(avg)` for non-empty collections

**States**: Available on `Initial`, `Filtered`, `Sorted`, `Projected<T>`  
**Consumes**: Yes (terminal operation)

**Example**:
```rust
let avg = QueryBuilder::from(vec![1, 2, 3, 4, 5]).average();
// avg = Some(3.0)

let empty_avg = QueryBuilder::from(Vec::<i32>::new()).average();
// empty_avg = None
```

---

#### min() / max()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        // Implementation
    }
    
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        // Implementation
    }
}
```

**Trait Bounds**: `T: Ord`  
**Return Type**: `Option<T>`  
- `None` for empty collections
- `Some(min_value)` / `Some(max_value)` for non-empty

**States**: Available on all states  
**Consumes**: Yes (terminal operation)

**Optimization**: For `Sorted` state, `min()` can return first element, `max()` last element in O(1).

**Example**:
```rust
let min = QueryBuilder::from(vec![5, 2, 8, 1, 9]).min();
// min = Some(1)

let max = QueryBuilder::from(vec![5, 2, 8, 1, 9]).max();
// max = Some(9)
```

---

#### min_by() / max_by()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn min_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        // Implementation
    }
    
    pub fn max_by<K, F>(self, key_selector: F) -> Option<T>
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        // Implementation
    }
}
```

**Trait Bounds**: 
- `F: Fn(&T) -> K` (key selector function)
- `K: Ord` (key must be orderable)

**Return Type**: `Option<T>` (full element, not just key)  
**States**: Available on all states  
**Consumes**: Yes (terminal operation)

**Example**:
```rust
struct User { name: String, age: u32 }

let youngest = QueryBuilder::from(users)
    .min_by(|u| u.age);
// Returns entire User struct with smallest age
```

---

### Group 2: Grouping Operations (Terminal Operations)

#### group_by()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
        K: Eq + std::hash::Hash,
    {
        // Implementation
    }
}
```

**Trait Bounds**:
- `F: Fn(&T) -> K` (key selector function)
- `K: Eq + Hash` (key must be hashable for HashMap)

**Return Type**: `HashMap<K, Vec<T>>`  
- Keys: Unique keys extracted by key_selector
- Values: Vectors of elements with that key

**States**: Available on all states  
**Consumes**: Yes (terminal operation)  
**Order Guarantee**: Elements within each group maintain original relative order

**Example**:
```rust
struct User { name: String, department: String }

let by_dept: HashMap<String, Vec<User>> = QueryBuilder::from(users)
    .group_by(|u| u.department.clone());
// Keys: ["Engineering", "Sales", "Marketing", ...]
// Values: Vectors of users in each department
```

---

#### group_by_aggregate()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn group_by_aggregate<K, R, FK, FA>(
        self,
        key_selector: FK,
        aggregator: FA,
    ) -> HashMap<K, R>
    where
        FK: Fn(&T) -> K,
        FA: Fn(&[T]) -> R,
        K: Eq + std::hash::Hash,
    {
        // Implementation
    }
}
```

**Trait Bounds**:
- `FK: Fn(&T) -> K` (key selector)
- `FA: Fn(&[T]) -> R` (aggregation function, takes slice of group elements)
- `K: Eq + Hash`

**Return Type**: `HashMap<K, R>` where `R` is aggregation result type  
**States**: Available on all states  
**Consumes**: Yes (terminal operation)

**Example**:
```rust
struct Order { user_id: u32, amount: f64 }

let totals: HashMap<u32, f64> = QueryBuilder::from(orders)
    .group_by_aggregate(
        |o| o.user_id,
        |group| group.iter().map(|o| o.amount).sum()
    );
// Keys: user_ids
// Values: Total amount for each user
```

---

### Group 3: Deduplication (Transformations)

#### distinct()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn distinct(self) -> QueryBuilder<T, Filtered>
    where
        T: Eq + std::hash::Hash + Clone,
    {
        // Implementation
    }
}
```

**Trait Bounds**:
- `T: Eq + Hash` (required for HashSet membership)
- `T: Clone` (required to store in HashSet while filtering)

**Return Type**: `QueryBuilder<T, Filtered>` (same element type, new state)  
**States**: Callable on all states, always returns `Filtered`  
**Consumes**: Yes (but returns new QueryBuilder for chaining)  
**Order Guarantee**: Preserves first-occurrence order

**Example**:
```rust
let unique: Vec<i32> = QueryBuilder::from(vec![1, 2, 2, 3, 3, 3])
    .distinct()
    .collect();
// unique = [1, 2, 3]
```

---

#### distinct_by()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn distinct_by<K, F>(self, key_selector: F) -> QueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> K + 'static,
        K: Eq + std::hash::Hash,
    {
        // Implementation
    }
}
```

**Trait Bounds**:
- `F: Fn(&T) -> K + 'static` (key selector, must be 'static for closure capture)
- `K: Eq + Hash` (key must be hashable)
- Note: `T` does NOT require `Clone` (only key is hashed)

**Return Type**: `QueryBuilder<T, Filtered>`  
**States**: Callable on all states  
**Consumes**: Yes (but returns new QueryBuilder)  
**Order Guarantee**: Preserves first-occurrence order

**Example**:
```rust
struct User { id: u32, name: String }

let unique_names: Vec<User> = QueryBuilder::from(users)
    .distinct_by(|u| u.name.clone())
    .collect();
// First user with each unique name
```

---

### Group 4: Sequence Transformations

#### reverse()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn reverse(self) -> QueryBuilder<T, Filtered>
    {
        // Implementation
    }
}
```

**Trait Bounds**: None (works on any type)  
**Return Type**: `QueryBuilder<T, Filtered>`  
**States**: Callable on all states, returns `Filtered`  
**Consumes**: Yes (but returns new QueryBuilder)  
**Order**: Reverses iteration order

**Example**:
```rust
let reversed: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .reverse()
    .collect();
// reversed = [5, 4, 3, 2, 1]
```

---

#### chunk()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn chunk(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
    {
        // Implementation
    }
}
```

**Trait Bounds**: None  
**Return Type**: `QueryBuilder<Vec<T>, Filtered>` (type changes from `T` to `Vec<T>`)  
**Parameters**: `size: usize` (chunk size, must be > 0)  
**States**: Callable on all states  
**Consumes**: Yes (but returns new QueryBuilder)  
**Panics**: If `size == 0`

**Example**:
```rust
let chunks: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .chunk(2)
    .collect();
// chunks = [[1, 2], [3, 4], [5]]
```

---

#### window()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn window(self, size: usize) -> QueryBuilder<Vec<T>, Filtered>
    where
        T: Clone,
    {
        // Implementation
    }
}
```

**Trait Bounds**: `T: Clone` (required for overlapping windows)  
**Return Type**: `QueryBuilder<Vec<T>, Filtered>` (type changes to `Vec<T>`)  
**Parameters**: `size: usize` (window size, must be ≥ 2)  
**States**: Callable on all states  
**Consumes**: Yes (but returns new QueryBuilder)  
**Panics**: If `size < 2`

**Example**:
```rust
let windows: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .window(3)
    .collect();
// windows = [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
```

---

### Group 5: Collection Combinations

#### zip()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn zip<U: 'static>(
        self,
        other: impl IntoIterator<Item = U> + 'static,
    ) -> QueryBuilder<(T, U), Filtered>
    where
        U: 'static,
    {
        // Implementation
    }
}
```

**Trait Bounds**: `U: 'static` (second collection element type)  
**Return Type**: `QueryBuilder<(T, U), Filtered>` (type changes to tuple)  
**States**: Callable on all states  
**Consumes**: Yes (but returns new QueryBuilder)  
**Semantics**: Shortest-wins (stops when either collection exhausted)

**Example**:
```rust
let paired: Vec<(i32, char)> = QueryBuilder::from(vec![1, 2, 3])
    .zip(vec!['a', 'b', 'c'])
    .collect();
// paired = [(1, 'a'), (2, 'b'), (3, 'c')]
```

---

#### enumerate()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn enumerate(self) -> QueryBuilder<(usize, T), Filtered>
    {
        // Implementation
    }
}
```

**Trait Bounds**: None  
**Return Type**: `QueryBuilder<(usize, T), Filtered>` (type changes to tuple with index)  
**States**: Callable on all states  
**Consumes**: Yes (but returns new QueryBuilder)  
**Indexing**: Zero-based, reflects current query state (not original indices after filtering)

**Example**:
```rust
let indexed: Vec<(usize, i32)> = QueryBuilder::from(vec![10, 20, 30])
    .enumerate()
    .collect();
// indexed = [(0, 10), (1, 20), (2, 30)]

// After filtering
let indexed_filtered: Vec<(usize, i32)> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .where_(|x| x % 2 == 0)
    .enumerate()
    .collect();
// indexed_filtered = [(0, 2), (1, 4)]  // Indices 0, 1 (not original 1, 3)
```

---

#### partition()
```rust
impl<T: 'static> QueryBuilder<T, State> {
    pub fn partition<F>(self, predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        // Implementation
    }
}
```

**Trait Bounds**: `F: Fn(&T) -> bool` (predicate function)  
**Return Type**: `(Vec<T>, Vec<T>)` (tuple of two vectors)  
- First Vec: Elements satisfying predicate
- Second Vec: Elements not satisfying predicate

**States**: Available on all states  
**Consumes**: Yes (terminal operation, returns tuple not QueryBuilder)

**Example**:
```rust
let (evens, odds) = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .partition(|x| x % 2 == 0);
// evens = [2, 4]
// odds = [1, 3, 5]
```

---

## Helper Iterator Adapters (Internal)

### ChunkIterator

```rust
struct ChunkIterator<T, I: Iterator<Item = T>> {
    iter: I,
    size: usize,
}

impl<T, I: Iterator<Item = T>> ChunkIterator<T, I> {
    fn new(iter: I, size: usize) -> Self {
        assert!(size > 0, "chunk size must be greater than 0");
        Self { iter, size }
    }
}

impl<T, I: Iterator<Item = T>> Iterator for ChunkIterator<T, I> {
    type Item = Vec<T>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            match self.iter.next() {
                Some(item) => chunk.push(item),
                None => break,
            }
        }
        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}
```

**Purpose**: Lazily partition iterator into fixed-size chunks  
**Complexity**: O(1) per chunk, O(n) overall  
**Memory**: O(chunk_size) buffer

---

### WindowIterator

```rust
struct WindowIterator<T: Clone> {
    items: Vec<T>,
    size: usize,
    position: usize,
}

impl<T: Clone> WindowIterator<T> {
    fn new(items: Vec<T>, size: usize) -> Self {
        assert!(size >= 2, "window size must be at least 2");
        Self { items, size, position: 0 }
    }
}

impl<T: Clone> Iterator for WindowIterator<T> {
    type Item = Vec<T>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.position + self.size > self.items.len() {
            return None;
        }
        
        let window: Vec<T> = self.items[self.position..self.position + self.size]
            .to_vec();  // Clone elements
        
        self.position += 1;
        Some(window)
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.items.len().saturating_sub(self.position + self.size - 1);
        (remaining, Some(remaining))
    }
}
```

**Purpose**: Create sliding windows with overlap  
**Complexity**: O(window_size) per window, O(n * window_size) overall  
**Memory**: O(n) for items storage, O(window_size) per returned window  
**Clone Overhead**: Each element cloned once per window it appears in

---

## Complete Method Availability Matrix

| Method | Initial | Filtered | Sorted | Projected<U> | Return Type |
|--------|---------|----------|--------|--------------|-------------|
| **v0.1 Methods** | | | | | |
| `from()` | ✅ | - | - | - | `QueryBuilder<T, Initial>` |
| `where_()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<T, Filtered>` |
| `order_by()` | ✅ | ✅ | - | - | `QueryBuilder<T, Sorted>` |
| `then_by()` | - | - | ✅ | - | `QueryBuilder<T, Sorted>` |
| `select<U>()` | ✅ | ✅ | ✅ | - | `QueryBuilder<U, Projected<U>>` |
| `take()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<T, State>` |
| `skip()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<T, State>` |
| `collect()` | ✅ | ✅ | ✅ | ✅ | `Vec<T>` |
| `count()` | ✅ | ✅ | ✅ | ✅ | `usize` |
| `first()` | ✅ | ✅ | ✅ | ✅ | `Option<T>` |
| `last()` | ✅ | ✅ | ✅ | ✅ | `Option<T>` |
| `any()` | ✅ | ✅ | ✅ | ✅ | `bool` |
| `all()` | ✅ | ✅ | ✅ | ✅ | `bool` |
| `inspect()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<T, Filtered>` |
| **v0.2 Methods** | | | | | |
| `sum()` | ✅ | ✅ | ✅ | ✅* | `T` |
| `average()` | ✅ | ✅ | ✅ | ✅* | `Option<f64>` |
| `min()` | ✅ | ✅ | ✅ | ✅* | `Option<T>` |
| `max()` | ✅ | ✅ | ✅ | ✅* | `Option<T>` |
| `min_by()` | ✅ | ✅ | ✅ | ✅* | `Option<T>` |
| `max_by()` | ✅ | ✅ | ✅ | ✅* | `Option<T>` |
| `group_by()` | ✅ | ✅ | ✅ | ✅* | `HashMap<K, Vec<T>>` |
| `group_by_aggregate()` | ✅ | ✅ | ✅ | ✅* | `HashMap<K, R>` |
| `distinct()` | ✅ | ✅ | ✅ | ✅* | `QueryBuilder<T, Filtered>` |
| `distinct_by()` | ✅ | ✅ | ✅ | ✅* | `QueryBuilder<T, Filtered>` |
| `reverse()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<T, Filtered>` |
| `enumerate()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<(usize, T), Filtered>` |
| `zip()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<(T, U), Filtered>` |
| `chunk()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<Vec<T>, Filtered>` |
| `window()` | ✅ | ✅ | ✅ | ✅ | `QueryBuilder<Vec<T>, Filtered>` |
| `partition()` | ✅ | ✅ | ✅ | ✅* | `(Vec<T>, Vec<T>)` |

*Note: `Projected<U>` means the method is available, operating on type `U` instead of original type `T`.

---

## Trait Bound Summary

### Required Trait Bounds by Operation

| Operation | Element Type Bounds | Key Type Bounds | Notes |
|-----------|-------------------|-----------------|-------|
| `sum()` | `T: Sum` | - | Uses std::iter::Sum |
| `average()` | `T: ToPrimitive` | - | Converts to f64 |
| `min()` / `max()` | `T: Ord` | - | Total ordering required |
| `min_by()` / `max_by()` | - | `K: Ord` | Key selector returns K |
| `group_by()` | - | `K: Eq + Hash` | HashMap keys |
| `group_by_aggregate()` | - | `K: Eq + Hash` | HashMap keys |
| `distinct()` | `T: Eq + Hash + Clone` | - | HashSet membership + storage |
| `distinct_by()` | - | `K: Eq + Hash` | Only key needs Hash |
| `reverse()` | - | - | No bounds |
| `chunk()` | - | - | No bounds |
| `window()` | `T: Clone` | - | Overlapping windows need cloning |
| `zip()` | - | `U: 'static` | Second collection type |
| `enumerate()` | - | - | No bounds |
| `partition()` | - | - | No bounds (predicate is Fn) |

### External Trait Dependencies

```rust
// From std library
use std::iter::Sum;
use std::hash::Hash;
use std::cmp::Ord;

// From num-traits crate (NEW)
use num_traits::ToPrimitive;
```

---

## Memory and Performance Characteristics

### Operation Complexity

| Operation | Time Complexity | Space Complexity | Allocations |
|-----------|----------------|------------------|-------------|
| `sum()` | O(n) | O(1) | 0 |
| `average()` | O(n) | O(n) | 1 Vec allocation |
| `min()` / `max()` | O(n) | O(1) | 0 |
| `min_by()` / `max_by()` | O(n) | O(1) | 0 |
| `group_by()` | O(n) | O(n) | 1 HashMap + k Vecs |
| `group_by_aggregate()` | O(n) | O(k) | 1 HashMap |
| `distinct()` | O(n) | O(k) | 1 HashSet + 1 Vec |
| `distinct_by()` | O(n) | O(k) | 1 HashSet + 1 Vec |
| `reverse()` | O(n) | O(n) | 1 Vec allocation |
| `chunk()` | O(n) | O(chunk_size) | Multiple small Vecs |
| `window()` | O(n * window_size) | O(n + window_size) | Multiple cloned windows |
| `zip()` | O(min(n, m)) | O(1) | 0 (lazy) |
| `enumerate()` | O(n) | O(1) | 0 (lazy) |
| `partition()` | O(n) | O(n) | 2 Vec allocations |

**Legend**:
- `n`: Number of elements in collection
- `k`: Number of unique elements/keys (k ≤ n)
- `m`: Size of second collection (for zip)

---

## Type Safety Guarantees

### Compile-Time Prevented Errors

| Invalid Operation | Prevented By | Error Message |
|------------------|--------------|---------------|
| `.then_by()` before `.order_by()` | State: `then_by()` only on `Sorted` | "method `then_by` not found for type `QueryBuilder<T, Initial>`" |
| `.sum()` on non-numeric | Trait bound `T: Sum` | "the trait `Sum` is not implemented for `SomeType`" |
| `.distinct()` on non-hashable | Trait bound `T: Hash` | "the trait `Hash` is not implemented for `SomeType`" |
| `.window()` on non-clonable | Trait bound `T: Clone` | "the trait `Clone` is not implemented for `SomeType`" |
| `.collect()` after `.sum()` | Terminal op consumes builder | "use of moved value: `query_builder`" |

**Key Insight**: Rust's type system enforces correct usage at compile time. No runtime checks needed.

---

## Example: Complex Query Type Evolution

```rust
// Start with integers
let data: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Type: QueryBuilder<i32, Initial>
let query = QueryBuilder::from(data);

// Type: QueryBuilder<i32, Filtered>
let query = query.where_(|x| x % 2 == 0);  // [2, 4, 6, 8, 10]

// Type: QueryBuilder<(usize, i32), Filtered>
let query = query.enumerate();  // [(0,2), (1,4), (2,6), (3,8), (4,10)]

// Type: QueryBuilder<Vec<(usize, i32)>, Filtered>
let query = query.chunk(2);  // [[(0,2), (1,4)], [(2,6), (3,8)], [(4,10)]]

// Type: Vec<Vec<(usize, i32)>>
let result = query.collect();
```

**Observation**: Type changes are tracked at each step, ensuring compile-time type safety throughout the transformation pipeline.

---

## Backwards Compatibility Analysis

### v0.1 Code Compatibility

All existing v0.1 code must continue to compile and work without changes.

**Test Case 1**: Basic filtering and collection
```rust
// v0.1 code (must still work)
let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .where_(|x| x % 2 == 0)
    .collect();
```
✅ **Status**: No breaking changes (no v0.2 methods added to this chain)

**Test Case 2**: Sorting with secondary sort
```rust
// v0.1 code (must still work)
let result = QueryBuilder::from(users)
    .order_by(|u| u.department.clone())
    .then_by(|u| u.age)
    .collect();
```
✅ **Status**: No breaking changes

**Test Case 3**: Projection with pagination
```rust
// v0.1 code (must still work)
let result = QueryBuilder::from(orders)
    .select(|o| o.amount)
    .take(10)
    .collect();
```
✅ **Status**: No breaking changes

**Test Case 4**: MetricsQueryBuilder
```rust
// v0.1 code (must still work)
let metrics = MetricsCollector::new();
let result = MetricsQueryBuilder::from(data, metrics.clone())
    .where_(|x| x > 5)
    .count();
```
✅ **Status**: No breaking changes (MetricsQueryBuilder extended additively)

---

### Breaking Change Analysis

**Potential Breaking Changes**: None identified.

**Additive Changes Only**:
- New methods added to existing `impl` blocks
- New helper structs (ChunkIterator, WindowIterator) are internal
- No changes to existing method signatures
- No changes to type state definitions
- No changes to QueryData enum (can be extended without breaking)

**Verification Strategy**: Run full v0.1 test suite (115+ tests) after v0.2 implementation to confirm no regressions.

---

## MetricsQueryBuilder Extension

### New Method Wrappers

```rust
impl<T: 'static> MetricsQueryBuilder<T, Initial> {
    // Terminal operations (record metrics)
    pub fn sum(self) -> T where T: Sum {
        let start = Instant::now();
        let result = self.inner.sum();
        self.metrics.record_query_time("rinq.sum", start.elapsed());
        self.metrics.increment("rinq.operations.sum");
        result
    }
    
    // Similar for: average, min, max, min_by, max_by, group_by, group_by_aggregate, partition
    
    // Non-terminal operations (pass through)
    pub fn distinct(self) -> MetricsQueryBuilder<T, Filtered> 
    where T: Eq + Hash + Clone 
    {
        MetricsQueryBuilder {
            inner: self.inner.distinct(),
            metrics: self.metrics,
            _state: PhantomData,
        }
    }
    
    // Similar for: distinct_by, reverse, enumerate, zip, chunk, window
}
```

### Metrics Schema

**New Metrics Recorded**:

| Metric Name | Type | Description |
|-------------|------|-------------|
| `rinq.sum` | Timer | Execution time for sum() |
| `rinq.average` | Timer | Execution time for average() |
| `rinq.min` | Timer | Execution time for min() |
| `rinq.max` | Timer | Execution time for max() |
| `rinq.min_by` | Timer | Execution time for min_by() |
| `rinq.max_by` | Timer | Execution time for max_by() |
| `rinq.group_by` | Timer | Execution time for group_by() |
| `rinq.group_by_aggregate` | Timer | Execution time for group_by_aggregate() |
| `rinq.partition` | Timer | Execution time for partition() |
| `rinq.operations.sum` | Counter | Number of sum() calls |
| `rinq.operations.average` | Counter | Number of average() calls |
| `rinq.operations.min` | Counter | Number of min() calls |
| `rinq.operations.max` | Counter | Number of max() calls |
| `rinq.operations.min_by` | Counter | Number of min_by() calls |
| `rinq.operations.max_by` | Counter | Number of max_by() calls |
| `rinq.operations.group_by` | Counter | Number of group_by() calls |
| `rinq.operations.group_by_aggregate` | Counter | Number of group_by_aggregate() calls |
| `rinq.operations.partition` | Counter | Number of partition() calls |

**Note**: Non-terminal operations (distinct, reverse, etc.) do not record metrics until a terminal operation is called.

---

## Data Model Summary

### Public API Extensions

**12 New Public Methods**:
- 6 Aggregations: sum, average, min, max, min_by, max_by
- 2 Grouping: group_by, group_by_aggregate
- 2 Deduplication: distinct, distinct_by
- 1 Reversal: reverse
- 3 Sequence: chunk, window, enumerate
- 2 Combination: zip, partition

**0 New Type States**: Reuse existing states (Initial, Filtered, Sorted, Projected)

**0 New QueryData Variants**: Reuse existing variants (Iterator, SortedVec)

**2 New Helper Iterators** (internal): ChunkIterator, WindowIterator

**1 New Dependency**: `num-traits = "0.2"`

---

**Data Model Status**: ✅ **COMPLETE**  
**Next Phase**: Task Breakdown (`/speckit.tasks`)
