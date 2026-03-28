---
name: Bug Report
about: Report a bug or unexpected behavior
labels: bug
---

## Description

A clear and concise description of what the bug is.

## Reproduction

```rust
// Minimal code that reproduces the issue
use rinq::QueryBuilder;

let result = QueryBuilder::from(vec![...])
    // ...
```

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened (include any error messages or panics).

## Environment

- **Rust version**: (output of `rustc --version`)
- **rinq version**: (from `Cargo.toml` or `cargo tree`)
- **OS**: (e.g. Windows 11, Ubuntu 22.04, macOS 14)
- **Features enabled**: (e.g. `parallel`, `serde`, none)

## Additional Context

Any other information that might be relevant.
