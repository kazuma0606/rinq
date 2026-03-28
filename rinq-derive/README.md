# rinq-derive

[![CI](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml/badge.svg)](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rinq-derive.svg)](https://crates.io/crates/rinq-derive)
[![docs.rs](https://docs.rs/rinq-derive/badge.svg)](https://docs.rs/rinq-derive)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Derive macros for the [rinq](https://crates.io/crates/rinq) query engine.**

Auto-generate field accessors and typed predicates for your structs, so you can write expressive rinq queries without boilerplate.

## Installation

```toml
[dependencies]
rinq        = "0.1"
rinq-derive = "0.1"
```

## Quick Start

```rust
use rinq::QueryBuilder;
use rinq_derive::Queryable;

#[derive(Queryable, Clone, Debug)]
struct User {
    pub name: String,
    pub age:  u32,
    pub active: bool,
}

let users = vec![
    User { name: "Alice".into(), age: 30, active: true },
    User { name: "Bob".into(),   age: 17, active: false },
    User { name: "Carol".into(), age: 25, active: true },
];

// Use generated typed predicates from the `user_fields` module
use user_fields::{Age, Active};

let result = QueryBuilder::from(users)
    .where_(Age::gt(18))
    .where_(Active::is_true())
    .order_by(User::by_age)
    .collect_vec();

assert_eq!(result[0].name, "Carol");
assert_eq!(result[1].name, "Alice");
```

## Generated Code

`#[derive(Queryable)]` generates two things for your struct:

### 1. Field accessor functions (on `impl YourStruct`)

```rust
impl User {
    pub fn by_name(u: &User)   -> &str { &u.name }
    pub fn by_age(u: &User)    -> u32  { u.age }
    pub fn by_active(u: &User) -> bool { u.active }
}
```

Use these directly with `order_by` / `then_by`.

### 2. Typed predicate module (`your_struct_fields`)

```rust
pub mod user_fields {
    pub struct Age;
    impl Age {
        pub fn gt(n: u32)                -> impl Fn(&User) -> bool { ... }
        pub fn lt(n: u32)                -> impl Fn(&User) -> bool { ... }
        pub fn eq(n: u32)                -> impl Fn(&User) -> bool { ... }
        pub fn between(lo: u32, hi: u32) -> impl Fn(&User) -> bool { ... }
    }

    pub struct Active;
    impl Active {
        pub fn is_true()  -> impl Fn(&User) -> bool { ... }
        pub fn is_false() -> impl Fn(&User) -> bool { ... }
    }

    pub struct Name;
    impl Name {
        pub fn contains(s: &str) -> impl Fn(&User) -> bool { ... }
        pub fn eq(s: &str)       -> impl Fn(&User) -> bool { ... }
    }
}
```

### Predicate methods by field type

| Field type | Generated methods |
|---|---|
| Numeric (`u8`…`f64`) | `gt`, `lt`, `eq`, `between` |
| `bool` | `is_true`, `is_false` |
| `String` / `&str` | `contains`, `eq` |
| Other | `eq` |

## Attributes

| Attribute | Effect |
|---|---|
| `#[queryable(skip)]` | Do not generate accessors for this field |
| `#[queryable(rename = "foo")]` | Name the generated accessor `by_foo` instead |
| `#[queryable(key)]` | Mark as the default sort/group key |

```rust
#[derive(Queryable)]
struct Product {
    #[queryable(key)]
    pub id: u64,

    #[queryable(rename = "title")]
    pub name: String,

    #[queryable(skip)]
    pub internal_notes: String,
}
```

## `#[derive(QueryableFrom)]`

Implement `From<YourCollection>` for `QueryBuilder<YourItem, Initial>`:

```rust
use rinq_derive::QueryableFrom;

#[derive(QueryableFrom)]
struct UserList(Vec<User>);

// Enables:
let query = UserList(users).into_query();
```

## License

MIT — see [LICENSE](../LICENSE)
