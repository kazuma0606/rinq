# Feature Specification: RINQ v0.2 - Aggregation and Transformation Extensions

**Feature Branch**: `002-rinq-v0.2-aggregation`  
**Created**: 2026-03-08  
**Status**: Draft  
**Input**: User description: "RINQ v0.2として集約と変換の拡張機能を実装。グループ化、数値集約、重複排除、シーケンス操作、コレクション操作を追加。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Numeric Aggregations for Data Analysis (Priority: P1)

As a Rust developer using RINQ for data analysis, I need to quickly calculate summary statistics (sum, average, min, max) on filtered datasets without writing manual loops, so I can focus on business logic rather than boilerplate iteration code.

**Why this priority**: Most common data analysis pattern. Provides immediate practical value for metrics, reporting, and analytics use cases. Independently useful without any other v0.2 features.

**Independent Test**: Can be fully tested by creating a numeric collection, applying aggregation methods (`.sum()`, `.average()`, `.min()`, `.max()`), and verifying results against known expected values. Delivers standalone value for reporting and analytics.

**Acceptance Scenarios**:

1. **Given** a collection of integers `[1, 2, 3, 4, 5]`, **When** I call `.sum()`, **Then** I receive `15`
2. **Given** a collection of integers `[1, 2, 3, 4, 5]`, **When** I call `.average()`, **Then** I receive `3.0`
3. **Given** a collection of integers `[5, 2, 8, 1, 9]`, **When** I call `.min()`, **Then** I receive `Some(1)`
4. **Given** a collection of integers `[5, 2, 8, 1, 9]`, **When** I call `.max()`, **Then** I receive `Some(9)`
5. **Given** an empty collection, **When** I call `.min()` or `.max()`, **Then** I receive `None`
6. **Given** a filtered collection where all elements are removed, **When** I call `.sum()`, **Then** I receive the additive identity (0 for integers)
7. **Given** a collection with a single element `[42]`, **When** I call `.average()`, **Then** I receive `42.0`
8. **Given** a collection of users with age field, **When** I call `.min_by(|u| u.age)`, **Then** I receive the youngest user
9. **Given** a collection of users with age field, **When** I call `.max_by(|u| u.age)`, **Then** I receive the oldest user

---

### User Story 2 - Grouping Data by Keys (Priority: P2)

As a Rust developer processing categorized data, I need to group collection elements by a key function, so I can organize data into categories and perform per-group analysis without manually building HashMaps.

**Why this priority**: Fundamental operation for categorization, reporting, and batch processing. Builds on P1 aggregations to enable grouped analytics. Can be tested independently.

**Independent Test**: Can be fully tested by creating a heterogeneous collection (e.g., users with different departments), calling `.group_by()` with a key function, and verifying the resulting HashMap has correct keys and grouped values. Delivers value for categorization tasks.

**Acceptance Scenarios**:

1. **Given** a collection of users with a `department` field, **When** I call `.group_by(|u| u.department)`, **Then** I receive a `HashMap<Department, Vec<User>>` with users grouped by department
2. **Given** a collection of strings, **When** I call `.group_by(|s| s.len())`, **Then** I receive a `HashMap<usize, Vec<String>>` grouped by string length
3. **Given** a collection of numbers, **When** I call `.group_by(|n| n % 2 == 0)` (even/odd), **Then** I receive a `HashMap<bool, Vec<i32>>` with two groups
4. **Given** an empty collection, **When** I call `.group_by()`, **Then** I receive an empty HashMap
5. **Given** a collection where all elements have the same key, **When** I call `.group_by()`, **Then** I receive a HashMap with one key and all elements in that group
6. **Given** a grouped result from `.group_by()`, **When** I iterate over the HashMap, **Then** each group maintains the original relative order of elements
7. **Given** a collection of orders with `user_id` and `amount` fields, **When** I call `.group_by_aggregate(|o| o.user_id, |group| group.iter().map(|o| o.amount).sum())`, **Then** I receive a `HashMap<UserId, f64>` with total amounts per user

---

### User Story 3 - Duplicate Removal for Data Cleaning (Priority: P3)

As a Rust developer working with potentially duplicate data, I need to efficiently remove duplicates from collections using hash-based deduplication, so I can ensure data uniqueness without manual HashSet management.

**Why this priority**: Common data cleaning pattern. Useful for sanitizing input data, removing redundant entries, and preparing datasets. Can be tested independently.

**Independent Test**: Can be fully tested by creating a collection with known duplicates, calling `.distinct()` or `.distinct_by()`, and verifying the result contains only unique elements. Delivers value for data cleaning workflows.

**Acceptance Scenarios**:

1. **Given** a collection `[1, 2, 2, 3, 3, 3]`, **When** I call `.distinct()`, **Then** I receive `[1, 2, 3]`
2. **Given** a collection of users with duplicate names, **When** I call `.distinct_by(|u| u.name.clone())`, **Then** I receive only users with unique names
3. **Given** an empty collection, **When** I call `.distinct()`, **Then** I receive an empty collection
4. **Given** a collection with all unique elements, **When** I call `.distinct()`, **Then** I receive all elements unchanged
5. **Given** a collection with all duplicate elements `[5, 5, 5, 5]`, **When** I call `.distinct()`, **Then** I receive `[5]`
6. **Given** a filtered collection, **When** I chain `.distinct()`, **Then** duplicates are removed from the filtered result
7. **Given** distinct elements, **When** I call `.collect()`, **Then** the order of first occurrence is preserved (insertion order)

---

### User Story 4 - Sequence Transformations for Data Restructuring (Priority: P4)

As a Rust developer processing sequential data, I need to reverse iteration order, partition data into fixed-size chunks, or create sliding windows, so I can implement pagination, batch processing, and moving averages without manual index management.

**Why this priority**: Supports specific use cases like pagination (chunking), time-series analysis (windowing), and reverse chronological display. Useful but not as universally needed as aggregation or grouping.

**Independent Test**: Can be fully tested by creating a sequential collection, applying transformations (`.reverse()`, `.chunk(n)`, `.window(n)`), and verifying the structure and order of results. Delivers value for batch processing and analytics.

**Acceptance Scenarios**:

1. **Given** a collection `[1, 2, 3, 4, 5]`, **When** I call `.reverse()`, **Then** I receive `[5, 4, 3, 2, 1]`
2. **Given** a collection `[1, 2, 3, 4, 5]`, **When** I call `.chunk(2)`, **Then** I receive `[[1, 2], [3, 4], [5]]`
3. **Given** a collection `[1, 2, 3, 4, 5]`, **When** I call `.window(3)`, **Then** I receive `[[1, 2, 3], [2, 3, 4], [3, 4, 5]]`
4. **Given** an empty collection, **When** I call `.chunk(2)`, **Then** I receive an empty result
5. **Given** a collection smaller than chunk size, **When** I call `.chunk(5)` on `[1, 2]`, **Then** I receive `[[1, 2]]`
6. **Given** a collection smaller than window size, **When** I call `.window(5)` on `[1, 2]`, **Then** I receive an empty result (not enough elements)
7. **Given** a filtered and sorted collection, **When** I call `.chunk(3)`, **Then** chunks respect the sorted order

---

### User Story 5 - Collection Combining and Partitioning (Priority: P5)

As a Rust developer working with multiple data sources, I need to combine collections element-wise (zip), add indices (enumerate), or split collections based on predicates (partition), so I can correlate data, track positions, and categorize results efficiently.

**Why this priority**: Supports advanced scenarios like data correlation, index tracking, and binary classification. Less frequently needed than core aggregations but valuable for specific workflows.

**Independent Test**: Can be fully tested by creating two collections for zipping, calling `.zip()`, `.enumerate()`, or `.partition()`, and verifying paired results, indexed elements, or split collections. Delivers value for data correlation and categorization.

**Acceptance Scenarios**:

1. **Given** two collections `[1, 2, 3]` and `['a', 'b', 'c']`, **When** I call `.zip()`, **Then** I receive `[(1, 'a'), (2, 'b'), (3, 'c')]`
2. **Given** two collections of different lengths `[1, 2]` and `['a', 'b', 'c']`, **When** I call `.zip()`, **Then** I receive `[(1, 'a'), (2, 'b')]` (stops at shortest)
3. **Given** a collection `[10, 20, 30]`, **When** I call `.enumerate()`, **Then** I receive `[(0, 10), (1, 20), (2, 30)]`
4. **Given** a filtered collection, **When** I call `.enumerate()`, **Then** indices reflect the filtered sequence (0, 1, 2...), not original positions
5. **Given** a collection `[1, 2, 3, 4, 5]`, **When** I call `.partition(|x| x % 2 == 0)`, **Then** I receive two collections: evens `[2, 4]` and odds `[1, 3, 5]`
6. **Given** a collection where no elements match partition predicate, **When** I call `.partition()`, **Then** I receive an empty matching collection and all elements in non-matching collection
7. **Given** a sorted collection, **When** I call `.partition()`, **Then** both resulting collections maintain sorted order

---

### Edge Cases

- **Empty Collections**: All operations must handle empty inputs gracefully
  - `sum()` → additive identity (0)
  - `average()` → `None` or `0.0` (to be decided in planning)
  - `min()` / `max()` → `None`
  - `group_by()` → empty HashMap
  - `distinct()` → empty collection
  - `chunk()` / `window()` → empty result
  - `partition()` → two empty collections

- **Single Element**: Operations on single-element collections must work correctly
  - `average([x])` → `x as f64`
  - `distinct([x])` → `[x]`
  - `chunk([x], 5)` → `[[x]]`
  - `window([x], 3)` → `[]` (not enough elements)

- **All Duplicates**: `distinct([5, 5, 5, 5])` → `[5]`

- **Numeric Overflow**: 
  - `sum()` on large integers → consider saturation or explicit overflow handling
  - `average()` precision → use `f64` for floating-point results

- **Window Size Edge Cases**:
  - `window(n)` where `n > collection.len()` → empty result
  - `window(0)` or `window(1)` → edge case behavior (to be defined)

- **Chunk Size Edge Cases**:
  - `chunk(0)` → invalid, should return error or panic (to be defined)
  - `chunk(n)` where `n > collection.len()` → single chunk with all elements

- **Type State Transitions**:
  - Can `.group_by()` be called after `.order_by()`? (Discuss in planning)
  - Does `.distinct()` preserve or discard sort order?
  - Can operations be chained arbitrarily?

## Requirements *(mandatory)*

### Functional Requirements

#### Aggregation Operations

- **FR-001**: System MUST provide `.sum()` method that calculates the sum of numeric collection elements
- **FR-002**: System MUST provide `.average()` method that calculates the arithmetic mean of numeric collection elements, returning `Option<f64>` or `f64`
- **FR-003**: System MUST provide `.min()` method that returns the minimum element as `Option<T>` where `T: Ord`
- **FR-004**: System MUST provide `.max()` method that returns the maximum element as `Option<T>` where `T: Ord`
- **FR-005**: System MUST provide `.min_by()` method that accepts a key selector function and returns the element with the minimum key value
- **FR-006**: System MUST provide `.max_by()` method that accepts a key selector function and returns the element with the maximum key value
- **FR-007**: All aggregation operations MUST be terminal operations that consume the `QueryBuilder`

#### Grouping Operations

- **FR-008**: System MUST provide `.group_by()` method that accepts a key function and returns `HashMap<K, Vec<T>>` where `K: Eq + Hash`
- **FR-009**: System MUST provide `.group_by_aggregate()` method that accepts a key function and an aggregation function, returning `HashMap<K, R>` where `R` is the aggregation result type
- **FR-010**: Grouped results MUST preserve the relative order of elements within each group (insertion order)
- **FR-011**: `.group_by()` MUST handle duplicate keys by accumulating elements into the same Vec

#### Deduplication Operations

- **FR-012**: System MUST provide `.distinct()` method that removes duplicate elements using hash-based comparison (requires `T: Eq + Hash`)
- **FR-013**: System MUST provide `.distinct_by()` method that accepts a key function and removes elements with duplicate keys
- **FR-014**: `.distinct()` MUST preserve the order of first occurrence (insertion order)
- **FR-015**: `.distinct()` MUST be compatible with chaining (can be called on `Filtered` or `Sorted` states)

#### Sequence Transformation Operations

- **FR-016**: System MUST provide `.reverse()` method that reverses the iteration order of elements
- **FR-017**: System MUST provide `.chunk(size: usize)` method that partitions elements into fixed-size chunks, returning `Vec<Vec<T>>`
- **FR-018**: The last chunk from `.chunk()` MAY contain fewer elements than the specified size if the collection length is not evenly divisible
- **FR-019**: System MUST provide `.window(size: usize)` method that creates sliding windows, returning an iterator of overlapping slices
- **FR-020**: `.window(n)` MUST return `n - 1` fewer windows than the collection length (for a collection of length `L`, return `L - n + 1` windows)

#### Collection Combination Operations

- **FR-021**: System MUST provide `.zip(other: impl IntoIterator<Item = U>)` method that pairs elements from two collections element-wise
- **FR-022**: `.zip()` MUST stop at the length of the shorter collection (shortest-wins semantics)
- **FR-023**: System MUST provide `.enumerate()` method that pairs each element with its zero-based index
- **FR-024**: `.enumerate()` indices MUST reflect the position in the current query state (not original collection indices after filtering)
- **FR-025**: System MUST provide `.partition(predicate)` method that splits a collection into two based on a predicate, returning `(Vec<T>, Vec<T>)`

#### Type Safety and State Management

- **FR-026**: All new methods MUST integrate with the existing type-state pattern (`Initial`, `Filtered`, `Sorted`, `Projected`)
- **FR-027**: Invalid operation sequences MUST be compile-time errors (e.g., cannot call `.then_by()` before `.order_by()`)
- **FR-028**: State transitions MUST be explicit and documented in type signatures
- **FR-029**: Operations that break lazy evaluation (e.g., `.reverse()`, `.distinct()`) MUST transition to appropriate states

#### Performance Requirements

- **FR-030**: All operations MUST maintain zero-cost abstraction guarantees (verified by benchmarks)
- **FR-031**: Operations MUST use lazy evaluation where semantically possible (e.g., `.enumerate()`, `.zip()`)
- **FR-032**: Operations requiring materialization (e.g., `.reverse()`, `.distinct()`, `.group_by()`) MUST minimize allocations
- **FR-033**: Aggregation operations (`.sum()`, `.average()`, `.min()`, `.max()`) MUST compile to performance equivalent to hand-written loops

#### Testing and Quality

- **FR-034**: Each new operation MUST have at least 3 property-based tests using `proptest`
- **FR-035**: Each new operation MUST have unit tests covering edge cases (empty, single element, boundary conditions)
- **FR-036**: Each new operation MUST have at least one benchmark comparing against manual implementation
- **FR-037**: All operations MUST have integration tests verifying compatibility with existing RINQ features (filtering, sorting, projection)

#### Integration with rusted-ca

- **FR-038**: All new operations MUST work correctly with `MetricsQueryBuilder` for observability
- **FR-039**: All fallible operations MUST use `RinqDomainError` for error reporting
- **FR-040**: Operations MUST integrate seamlessly with existing RINQ v0.1 methods (chainable without breaking type state)

### Key Entities *(include if feature involves data)*

- **QueryBuilder State Extensions**:
  - `Grouped`: Represents state after `.group_by()` operation (may or may not be needed depending on design)
  - `Reversed`: Represents state after `.reverse()` operation (if state tracking is required)
  - State transitions need to be carefully designed to maintain type safety

- **QueryData Extensions** (internal):
  - May need new variants for `Grouped(HashMap<K, Vec<T>>)` or `Reversed(VecDeque<T>)`
  - Design should minimize internal complexity while enabling required functionality

- **Numeric Trait Bounds**:
  - Operations like `.sum()`, `.average()` require `T: Add + Div + From<u8>` or similar numeric trait bounds
  - Consider using `num-traits` crate for generic numeric operations if standard library bounds are insufficient

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 12 new operations (sum, average, min, max, min_by, max_by, group_by, group_by_aggregate, distinct, distinct_by, reverse, chunk, window, zip, enumerate, partition) are implemented and fully functional
- **SC-002**: At least 50 new property-based tests are added, all passing
- **SC-003**: At least 30 new unit tests are added covering edge cases, all passing
- **SC-004**: At least 15 new benchmarks are added, demonstrating zero-cost abstraction (≤5% overhead vs manual code)
- **SC-005**: All operations chain correctly with existing RINQ v0.1 methods (filtering, sorting, projection, pagination)
- **SC-006**: `MetricsQueryBuilder` correctly records metrics for all new operations
- **SC-007**: Documentation includes working code examples for each new operation
- **SC-008**: `cargo clippy` passes with zero warnings
- **SC-009**: `cargo test` passes with 100% success rate (existing 115+ tests plus new tests)
- **SC-010**: `cargo bench` shows performance parity (within 5%) with hand-written loops for all aggregation operations

### User Experience Goals

- **UX-001**: Developers can perform common data analysis tasks (sum, average, grouping) in a single fluent chain without leaving the RINQ API
- **UX-002**: API feels consistent with RINQ v0.1 patterns (naming, chaining, error handling)
- **UX-003**: Compiler errors for invalid operation sequences are clear and actionable (due to type-state pattern)
- **UX-004**: Operations integrate seamlessly with existing features (e.g., `.where_().group_by()`, `.distinct().order_by()`)

### Technical Quality Goals

- **TQ-001**: Zero runtime type errors (enforced by type-state pattern)
- **TQ-002**: No performance regressions to existing RINQ v0.1 functionality
- **TQ-003**: Memory usage scales linearly with data size (no unexpected quadratic behavior)
- **TQ-004**: All public APIs have comprehensive documentation with examples
- **TQ-005**: Code coverage for new functionality exceeds 95%

## Constraints

### Technical Constraints

- **MUST maintain backwards compatibility with RINQ v0.1 APIs**: Existing code using v0.1 features must continue to compile and work without changes
- **MUST NOT introduce runtime overhead**: Benchmarks must validate zero-cost abstraction claims
- **MUST follow rusted-ca architecture**: Domain layer, metrics integration, error propagation
- **MUST use property-based testing**: All new operations require `proptest` coverage
- **SHOULD avoid new external dependencies**: Prefer `std` library; only add dependencies if absolutely necessary (e.g., `num-traits` for numeric operations)

### Design Constraints

- **Type-state pattern**: New operations must integrate with `Initial`, `Filtered`, `Sorted`, `Projected` states
- **Lazy evaluation**: Prefer iterator-based implementations; materialize only when necessary
- **Fluent interface**: All query-building methods must be chainable
- **Explicit terminal operations**: Methods that consume the builder and return results must be clearly documented

### Quality Constraints

- **Clippy**: Zero warnings with `-- -D warnings`
- **Formatting**: Must pass `cargo fmt --check`
- **Documentation**: All public methods require doc comments with examples
- **Testing**: Test-first development mandatory (tests before implementation)

## Non-Goals (Out of Scope for v0.2)

- **Database Integration**: ORM features, SQL generation (existing Rust ORMs handle this)
- **Async Operations**: Async iterators, futures integration (deferred to Phase 4: RINQ Async)
- **Parallel Processing**: Rayon integration, parallel iterators (deferred to Phase 3: RINQ Parallel)
- **Join Operations**: Multi-collection joins (deferred to Phase 2: RINQ Join)
- **Compile-time Optimization**: Procedural macros, const evaluation (deferred to Phase 7: RINQ Compile)
- **WASM/Browser Integration**: (deferred to Phase 6: RINQ WASM)
- **Interactive Documentation Server**: (deferred to Phase 5: RINQ Docs)

## Dependencies

### On RINQ v0.1

- All v0.2 features build on top of v0.1's foundation
- v0.1 APIs must remain stable and unchanged
- v0.2 tests must verify compatibility with v0.1 operations

### On rusted-ca Infrastructure

- `MetricsCollector` for observability
- `ApplicationError` for error propagation
- Domain layer architecture

## Review & Acceptance Checklist

Before considering this specification complete, verify:

- [ ] **User Stories are Independent**: Each story can be implemented, tested, and deployed independently
- [ ] **Priorities are Clear**: P1-P5 priorities reflect business and user value
- [ ] **Acceptance Scenarios are Testable**: Each scenario maps to a concrete test case
- [ ] **Requirements are Specific**: All FR-* items are concrete and measurable
- [ ] **Edge Cases are Identified**: Boundary conditions, empty collections, error scenarios documented
- [ ] **Success Criteria are Measurable**: SC-* items have clear, quantifiable metrics
- [ ] **Constraints are Explicit**: Technical, design, and quality constraints are documented
- [ ] **Non-Goals are Clear**: Out-of-scope items prevent scope creep
- [ ] **No Technology Decisions**: Specification focuses on "what" and "why", not "how" (that's for planning phase)
- [ ] **Backwards Compatibility**: v0.1 APIs remain unchanged and functional

## Clarifications

*This section will be populated during the clarification phase (`/speckit.clarify`)*

## Notes

- This specification extends RINQ v0.1 with practical aggregation and transformation features
- Focus on immediate utility for data analysis, reporting, and batch processing scenarios
- All features must maintain RINQ's core principles: type safety, zero-cost abstraction, lazy evaluation
- Implementation will follow test-driven development with property tests, unit tests, and benchmarks
- Refer to `docs/implementation.md` Phase 1 for additional technical context and code examples
