# rinq-syntax

[![CI](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml/badge.svg)](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rinq-syntax.svg)](https://crates.io/crates/rinq-syntax)
[![docs.rs](https://docs.rs/rinq-syntax/badge.svg)](https://docs.rs/rinq-syntax)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> ⚠ **Experimental** — API may change between minor versions.

**LINQ-style `query!` macro for the [rinq](https://crates.io/crates/rinq) query engine.**

Write queries in a declarative, SQL-like syntax that expands at compile time into a fully typed rinq pipeline.

## Installation

```toml
[dependencies]
rinq        = "0.1"
rinq-syntax = "0.1"
```

## Quick Start

```rust
use rinq_syntax::query;

let users = vec![
    User { name: "Alice".into(), age: 30, active: true },
    User { name: "Bob".into(),   age: 17, active: false },
    User { name: "Carol".into(), age: 25, active: true },
];

let result = query! {
    from user in users
    where user.active
    where user.age >= 18
    order_by user.age
    select user.name.clone()
};
// result: Vec<String> = ["Carol", "Alice"]
```

## Syntax Reference

### Clauses

| Clause | Example | Expands to |
|---|---|---|
| `from x in source` | `from user in users` | `::rinq::__macro_support::from(users)` |
| `where expr` | `where user.age > 18` | `.where_(\|user\| { user.age > 18 })` |
| `order_by key` | `order_by user.age` | `.order_by(\|user\| user.age)` |
| `order_by key desc` | `order_by user.age desc` | `.order_by_descending(\|user\| user.age)` |
| `then_by key` | `then_by user.name` | `.then_by(\|user\| user.name.clone())` |
| `take n` | `take 10` | `.take(10)` |
| `skip n` | `skip 5` | `.skip(5)` |
| `select expr` | `select user.name.clone()` | `.select(\|user\| { user.name.clone() }).collect::<Vec<_>>()` |

Multiple `where` clauses are chained in order:

```rust
query! {
    from x in numbers
    where x > 0
    where x % 2 == 0
    select x * 10
}
// expands to:
// __macro_support::from(numbers)
//     .where_(|x| { x > 0 })
//     .where_(|x| { x % 2 == 0 })
//     .select(|x| { x * 10 })
//     .collect::<Vec<_>>()
```

Multiple sort keys with `order_by key1, key2`:

```rust
query! {
    from u in users
    order_by u.department, u.age desc
    select u.name.clone()
}
```

### Binding Semantics

The binding variable (`x`, `user`, etc.) has type `&T` in `where` and `order_by` clauses, and owned `T` in `select`:

```rust
// Primitive — dereference with * in where/order_by
query! {
    from x in vec![1, 2, 3]
    where *x > 1
    select *x * 2
}

// Struct — field access auto-derefs
query! {
    from user in users
    where user.age > 18      // user: &User, auto-deref works
    select user.name.clone() // user: User (owned), clone needed for String
}
```

## Known Limitations

- `from` can only appear once (no JOIN syntax yet — planned for a future version)
- `let` bindings inside the query block are not supported
- Expressions containing the `from` keyword may be misidentified as a new clause
- `group_by` is not yet supported in `query!`

## License

MIT — see [LICENSE](../LICENSE)
