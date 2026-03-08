# RINQ Constitution
<!-- Rust Integrated Query - Project Governance and Development Principles -->

## Core Principles

### I. Type Safety First (型安全性第一)

**Zero runtime type errors through compile-time guarantees.**

- **Type-State Pattern**: Use Rust's type system to enforce valid query construction at compile time
- **State Transitions**: `Initial → Filtered → Sorted → Projected` states prevent invalid operations
- **Compile-Time Validation**: Invalid query chains must be rejected by the compiler, not at runtime
- **No Dynamic Dispatch**: Prefer generic/monomorphization over trait objects unless necessary
- **Explicit State**: Query builder states (`Initial`, `Filtered`, `Sorted`, `Projected`) must be explicit in type signatures

**Rationale**: RINQ v0.1 demonstrated that type-state patterns eliminate entire classes of bugs at compile time, providing confidence without runtime overhead.

### II. Zero-Cost Abstraction (ゼロコスト抽象化)

**Ergonomic APIs with no performance penalty.**

- **Iterator-Based**: Leverage Rust's `Iterator` trait for lazy evaluation
- **Inlining**: Use `#[inline]` for hot-path methods to enable cross-crate optimization
- **No Allocations**: Avoid unnecessary cloning or allocations; prefer borrowing and moves
- **Monomorphization**: Generic implementations should compile to same machine code as hand-written loops
- **Benchmark Validation**: Every new feature must include criterion benchmarks proving zero-cost claims

**Rationale**: RINQ v0.1 benchmarks showed performance parity with manual iterator chains. This must continue.

### III. Pragmatic Utility (実用性第一)

**Deliver immediate value to real-world use cases.**

- **Common Patterns**: Prioritize features developers use daily (aggregations, grouping, transformations)
- **Progressive Disclosure**: Simple cases should be simple; complex cases should be possible
- **Ergonomic API**: Methods should read like natural language (`.where_()`, `.order_by()`, `.select()`)
- **Integration Ready**: Seamlessly integrate with `rusted-ca` architecture (metrics, errors, logging)
- **No Speculation**: Only implement features with clear, concrete use cases

**Rationale**: Focus on delivering features that solve actual problems, not hypothetical ones.

### IV. API Consistency (一貫性)

**Maintain predictable patterns across all operations.**

- **Method Naming**: Continue established conventions (`where_()`, `order_by()`, `then_by()`, `select()`)
- **State Transitions**: New operations must follow existing state transition patterns
- **Error Handling**: Use `RinqDomainError` for all domain-level errors
- **Fluent Interface**: All query-building methods return `Self` or state-transitioned builder
- **Backwards Compatibility**: v0.2 additions must not break v0.1 APIs

**Rationale**: RINQ v0.1 established clear patterns. Consistency reduces cognitive load and learning curve.

### V. Test-Driven Development (テスト駆動開発 - NON-NEGOTIABLE)

**Write tests before implementation. No exceptions.**

- **Property-Based Testing**: Use `proptest` to verify invariants across all inputs
  - Immutability properties
  - Correctness properties (filtering, sorting, aggregation)
  - Composition properties (chaining operations)
- **Unit Tests**: Edge cases (empty collections, single elements, boundary conditions)
- **Integration Tests**: Verify metrics collection, error propagation, cross-module interaction
- **Benchmark Tests**: Criterion benchmarks for performance-critical paths
- **Test Coverage**: Aim for 100% coverage of public API surface

**Test-First Workflow**:
1. Write property tests defining expected behavior
2. Write unit tests for edge cases
3. Verify tests fail (Red)
4. Implement feature (Green)
5. Refactor while tests pass
6. Add benchmarks to validate zero-cost claims

**Rationale**: RINQ v0.1's 115+ passing tests gave us confidence to refactor aggressively. This discipline is mandatory.

### VI. Performance Validation (パフォーマンス検証)

**Benchmark every performance claim.**

- **Criterion Benchmarks**: Every new operation must include benchmarks comparing:
  - RINQ implementation vs manual iterator chains
  - RINQ implementation vs manual loops
  - Large datasets (10K+ elements)
- **Regression Testing**: Run benchmarks in CI to catch performance regressions
- **Memory Profiling**: Verify no unexpected allocations in hot paths
- **Performance Documentation**: Document Big-O complexity and memory characteristics

**Rationale**: "Zero-cost abstraction" is a verifiable claim, not marketing. Benchmarks provide proof.

### VII. Incremental Integration (段階的統合)

**Build on existing architecture; avoid rewrites.**

- **Domain Layer**: RINQ lives in `src/domain/rinq/` following rusted-ca's layered architecture
- **Metrics Integration**: Use `MetricsCollector` for observability
- **Error Hierarchy**: `RinqDomainError → ApplicationError` conversion path
- **Modular Design**: New features in separate modules; minimize changes to core `query_builder.rs`
- **Feature Flags**: Consider using Cargo features for optional functionality

**Rationale**: RINQ v0.1 integrated cleanly with rusted-ca's metrics and error systems. Continue this pattern.

---

## Technical Constraints

### Language and Tooling

- **Rust Edition**: 2021 or later
- **MSRV**: Minimum Supported Rust Version = 1.70 (or current stable)
- **Dependencies**: Minimize external crates; prefer `std` when possible
- **Allowed Dependencies**:
  - `proptest` (testing only)
  - `criterion` (benchmarking only)
  - Rust standard library collections (`std::collections`)
  - `num-traits` (for numeric operations, if needed)

### Code Quality

- **Clippy**: Must pass `cargo clippy -- -D warnings` (all warnings as errors)
- **Formatting**: Must pass `cargo fmt --check`
- **Documentation**: All public APIs require doc comments with examples
- **Visibility**: Default to private; only expose what's necessary
- **Safety**: Avoid `unsafe` unless absolutely required and justified with safety comments

### Performance Standards

- **Zero Allocations**: Query building should not allocate until terminal operations
- **Lazy Evaluation**: Operations must compose without intermediate collections
- **Inlining**: Hot-path methods must be `#[inline]` or `#[inline(always)]`
- **No Panics**: Public APIs must not panic; use `Result` for fallible operations

---

## Development Workflow

### Feature Development Cycle

1. **Specification**: Define what and why (user stories, requirements)
2. **Planning**: Define how (technical approach, state transitions, API design)
3. **Task Breakdown**: Create actionable task list with dependencies
4. **Test-First Implementation**:
   - Write property tests
   - Write unit tests
   - Verify tests fail
   - Implement feature
   - Verify tests pass
   - Add benchmarks
   - Update documentation
5. **Integration**: Verify metrics, errors, and cross-module behavior
6. **Validation**: Run full test suite + benchmarks + clippy + fmt

### Quality Gates

Before marking a feature complete, verify:

- ✅ All tests pass (`cargo test`)
- ✅ Benchmarks show zero-cost characteristics (`cargo bench`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Formatted correctly (`cargo fmt --check`)
- ✅ Documentation complete with examples
- ✅ Integration tests verify metrics collection
- ✅ Property tests cover invariants
- ✅ Edge cases tested (empty, single element, large datasets)

### Code Review Standards

- **API Design**: Review ergonomics, naming, consistency with existing APIs
- **Type Safety**: Verify state transitions are correct and enforce valid usage
- **Performance**: Check for unnecessary allocations, clones, or inefficiencies
- **Tests**: Ensure property tests cover invariants, not just happy paths
- **Documentation**: Verify examples compile and demonstrate common usage

---

## RINQ-Specific Principles

### Query Builder Design

- **Fluent Interface**: Methods chain naturally: `.from(data).where_(|x| x > 5).order_by(|x| x)`
- **Type State**: Invalid operations (e.g., `then_by()` before `order_by()`) must be compile errors
- **Lazy Evaluation**: No computation until terminal operation (`.collect()`, `.count()`, etc.)
- **Terminal Operations**: Consume the builder and produce results
- **Non-Destructive Inspection**: `inspect()` must not modify data or affect lazy evaluation

### Collection Support (via Queryable trait)

- **Wide Compatibility**: Support `Vec`, slices, arrays, `HashSet`, `BTreeSet`, `LinkedList`, `VecDeque`
- **Ownership**: Accept owned or borrowed collections; handle lifetimes correctly
- **Conversion**: `into_query()` should produce `QueryBuilder<T, Initial>` consistently

### Error Handling

- **Domain Errors**: Define specific error types in `RinqDomainError`
- **Descriptive Messages**: Include context (operation, expected vs actual state)
- **Conversion**: Provide `From<RinqDomainError> for ApplicationError`
- **No Silent Failures**: Invalid states must error explicitly

### Observability

- **Metrics Integration**: `MetricsQueryBuilder` wraps `QueryBuilder` for instrumentation
- **Query Timing**: Record execution time for terminal operations
- **Operation Counting**: Track query method invocations
- **Debug Support**: `inspect()` allows non-destructive observation

---

## v0.2 Evolution Principles

Building on v0.1 success, v0.2 focuses on:

### 1. Practical Aggregations (実用的な集約)

- `group_by()`: Group data by key function
- `sum()`, `average()`, `min()`, `max()`: Numeric aggregations
- `distinct()`, `distinct_by()`: Deduplication
- Must integrate with type-state pattern elegantly

### 2. Transformation Richness (豊富な変換)

- `chunk()`: Fixed-size partitioning
- `window()`: Sliding windows
- `zip()`: Combine collections
- `reverse()`: Reverse iteration order
- Must maintain lazy evaluation where possible

### 3. Compositional Complexity (組み合わせの複雑さ)

- New operations must compose with existing ones
- State transitions must remain clear and predictable
- Avoid state explosion (too many generic parameters)

---

## Governance

### Constitution Authority

- This Constitution supersedes all other development practices
- When conflicts arise, Constitution takes precedence
- Amendments require:
  1. Documentation of rationale
  2. Impact analysis on existing code
  3. Migration plan if breaking changes required

### Compliance

- All PRs must verify compliance with Core Principles
- Reviewers must check:
  - Type safety: Does it prevent misuse?
  - Zero-cost: Are benchmarks included and passing?
  - Tests: Are property tests and unit tests comprehensive?
  - Consistency: Does API follow established patterns?
  - Integration: Does it work with metrics/errors/logging?

### Justification Requirements

When introducing complexity, must justify:
- **Why is this needed?** (concrete use case)
- **Why can't simpler approaches work?** (alternatives considered)
- **What's the maintenance cost?** (testing, documentation, future changes)

### Living Document

- This Constitution is a living document
- Update it as we learn from implementation experience
- Document lessons learned in retrospectives

---

**Version**: 0.2.0  
**Ratified**: 2026-03-08  
**Last Amended**: 2026-03-08  
**Scope**: RINQ (Rust Integrated Query) v0.2 Development  
**Foundation**: Based on RINQ v0.1 implementation success (115+ tests, zero-cost validated)
