# rinq

[![CI](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml/badge.svg)](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rinq.svg)](https://crates.io/crates/rinq)
[![docs.rs](https://docs.rs/rinq/badge.svg)](https://docs.rs/rinq)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**LINQ-style query engine for Rust** — type-safe, zero-cost, chainable pipelines over any collection.

## Quick Start

```toml
[dependencies]
rinq = "0.1"
```

```rust
use rinq::QueryBuilder;

let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    .where_(|x| x % 2 == 0)
    .select(|x| x * x)
    .order_by_descending(|x| *x)
    .take(3)
    .collect();

assert_eq!(result, vec![100, 64, 36]);
```

## Feature Flags

| Flag       | Description                                   | Activates                  |
|------------|-----------------------------------------------|----------------------------|
| `parallel` | Parallel execution via [rayon]                | `ParallelQueryBuilder`     |
| `serde`    | JSON deserialization via [serde_json]         | `QueryBuilder::from_json`  |

```toml
[dependencies]
rinq = { version = "0.1", features = ["parallel", "serde"] }
```

## Type-State Machine

RINQ enforces valid operation order at compile time:

| State       | Allowed next operations                                |
|-------------|--------------------------------------------------------|
| `Initial`   | filter, transform, sort, set ops, terminal ops         |
| `Filtered`  | filter, transform, sort, terminal ops                  |
| `Sorted`    | `then_by`, `then_by_descending`, terminal ops          |
| `Projected` | terminal ops only                                      |

Invalid chains (e.g. `order_by` after `select`) produce a clear compile error.

## Operator Reference

### Filtering
| Method | Description |
|---|---|
| `where_(pred)` | Keep elements matching predicate |
| `take(n)` | Keep first N elements |
| `skip(n)` | Drop first N elements |
| `take_while(pred)` | Keep elements while predicate holds |
| `skip_while(pred)` | Drop elements while predicate holds |
| `step_by(n)` | Keep every N-th element |
| `filter_map(f)` | Map + filter in one pass |

### Transformation
| Method | Description |
|---|---|
| `select(f)` / `map(f)` | Project each element |
| `flat_map(f)` | Map then flatten |
| `flatten()` | Flatten nested iterables |
| `scan(seed, f)` | Running accumulation |
| `zip(other)` / `zip_with(other, f)` | Combine two sequences |
| `enumerate()` | Pair each element with its index |
| `cycle()` | Repeat infinitely (use with `take`) |
| `inspect(f)` | Side-effect without consuming |

### Sorting
| Method | Description |
|---|---|
| `order_by(key)` | Sort ascending by key |
| `order_by_descending(key)` | Sort descending by key |
| `then_by(key)` | Secondary ascending sort |
| `then_by_descending(key)` | Secondary descending sort |

### Grouping & Dedup
| Method | Description |
|---|---|
| `distinct()` / `distinct_by(key)` | Remove duplicates |
| `dedup()` / `dedup_by(key)` | Remove consecutive duplicates |
| `group_by(key)` | Group into `HashMap<K, Vec<T>>` |
| `group_by_aggregate(key, agg)` | Group + aggregate |
| `chunk_by(key)` | Group consecutive equal-key runs |
| `partition(pred)` | Split into two `Vec`s |
| `frequencies()` | Count occurrences → `HashMap<T, usize>` |

### Sequence
| Method | Description |
|---|---|
| `reverse()` | Reverse order |
| `chunk(n)` / `batch(n)` | Split into fixed-size chunks |
| `window(n)` | Sliding window |
| `pairwise()` | Consecutive `(T, T)` pairs |
| `intersperse(sep)` | Insert separator between elements |
| `tee()` | Clone into two identical `Vec`s |

### Set Operations
| Method | Description |
|---|---|
| `concat(other)` | Append another sequence |
| `union(other)` | Distinct union |
| `intersect(other)` | Common elements |
| `except(other)` | Elements not in other |

### JOIN Operations
| Method | Description |
|---|---|
| `inner_join(right, lk, rk)` | Equi-join → `(T, U)` |
| `left_join(right, lk, rk)` | Left outer join → `(T, Option<U>)` |
| `cross_join(right)` | Cartesian product → `(T, U)` |

### Aggregation
| Method | Description |
|---|---|
| `count()` / `count_by(pred)` | Count elements |
| `sum()` / `sum_by(f)` | Sum values or projected field |
| `average()` / `average_by(f)` | Mean |
| `min()` / `max()` | Min / max |
| `min_by(key)` / `max_by(key)` | Min / max by key |
| `min_max()` | Both min and max in one pass |
| `reduce(f)` / `aggregate(seed, f)` | Fold |

### Window Analytics
| Method | Description |
|---|---|
| `running_sum()` | Cumulative sum |
| `moving_average(n)` | Rolling mean |
| `rank_by(key)` | Rank each element (ascending) |
| `lag(n)` | Previous-N-element pairs |
| `lead(n)` | Next-N-element pairs |

### Terminal
| Method | Description |
|---|---|
| `first()` / `find(pred)` | First element or by predicate |
| `last()` | Last element |
| `nth(n)` / `element_at(n)` | Element at index |
| `any(pred)` / `all(pred)` / `none(pred)` | Boolean checks |
| `all_unique()` | All elements distinct? |
| `contains(val)` | Element present? |
| `single()` / `exactly_one()` | Exactly one element |
| `position(pred)` / `index_of(val)` | Find index |
| `collect()` / `collect_vec()` | Collect to `Vec` |
| `take_last(n)` / `skip_last(n)` | Last N / drop last N |
| `for_each(f)` | Consume with side effect |
| `to_sorted_vec(key)` / `to_sorted_vec_desc(key)` | Collect sorted |
| `to_hashmap(key, val)` | Collect to `HashMap` |
| `to_lookup(key)` | Collect to `HashMap<K, Vec<T>>` |

### DX Macros
| Macro | Description |
|---|---|
| `pred!(expr)` | Inline predicate shorthand |
| `rinq_explain!(query)` | Debug-mode timing wrapper |

## Sub-crates

| Crate | Description |
|---|---|
| [`rinq-stats`](https://crates.io/crates/rinq-stats) | Descriptive statistics, correlation, sampling, time-series, outlier detection, validation |
| [`rinq-derive`](https://crates.io/crates/rinq-derive) | `#[derive(Queryable)]` and `#[derive(QueryableFrom)]` macros |
| [`rinq-syntax`](https://crates.io/crates/rinq-syntax) | `query! { from x in data where ... select ... }` macro (experimental) |

## License

MIT — see [LICENSE](../LICENSE).
