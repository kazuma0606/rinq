// tests/rinq_serde_tests.rs
// Phase A4 integration tests for JSON / serde integration.
// Run with: cargo test --features serde --test rinq_serde_tests

#![cfg(feature = "serde")]

use rinq::{QueryBuilder, RinqError};
use serde::Deserialize;

// ── helper types ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: u32,
    name: String,
    age: u32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
struct Product {
    name: String,
    price: f64,
    in_stock: bool,
}

// ── from_json basic ──────────────────────────────────────────────────────────

#[test]
fn from_json_parses_array() {
    let json = r#"[{"id":1,"name":"Alice","age":30},{"id":2,"name":"Bob","age":17}]"#;
    let users: Vec<User> = QueryBuilder::<User, _>::from_json(json)
        .unwrap()
        .collect();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[1].name, "Bob");
}

#[test]
fn from_json_empty_array() {
    let json = r#"[]"#;
    let users: Vec<User> = QueryBuilder::<User, _>::from_json(json)
        .unwrap()
        .collect();
    assert!(users.is_empty());
}

#[test]
fn from_json_invalid_json_returns_err() {
    let result = QueryBuilder::<User, _>::from_json("not json at all");
    assert!(matches!(result, Err(RinqError::ExecutionError { .. })));
}

#[test]
fn from_json_wrong_schema_returns_err() {
    // JSON is valid but doesn't match User struct (missing required fields)
    let json = r#"[{"foo":"bar"}]"#;
    let result = QueryBuilder::<User, _>::from_json(json);
    assert!(matches!(result, Err(RinqError::ExecutionError { .. })));
}

#[test]
fn from_json_i32_array() {
    let json = r#"[1, 2, 3, 4, 5]"#;
    let nums: Vec<i32> = QueryBuilder::<i32, _>::from_json(json)
        .unwrap()
        .collect();
    assert_eq!(nums, vec![1, 2, 3, 4, 5]);
}

#[test]
fn from_json_string_array() {
    let json = r#"["hello", "world"]"#;
    let strings: Vec<String> = QueryBuilder::<String, _>::from_json(json)
        .unwrap()
        .collect();
    assert_eq!(strings, vec!["hello", "world"]);
}

// ── from_json + query chain ───────────────────────────────────────────────────

#[test]
fn from_json_filter_adults() {
    let json = r#"[
        {"id":1,"name":"Alice","age":30},
        {"id":2,"name":"Bob","age":17},
        {"id":3,"name":"Carol","age":25}
    ]"#;
    let adults: Vec<User> = QueryBuilder::<User, _>::from_json(json)
        .unwrap()
        .where_(|u| u.age >= 18)
        .collect();
    assert_eq!(adults.len(), 2);
    assert!(adults.iter().all(|u| u.age >= 18));
}

#[test]
fn from_json_filter_and_sort() {
    let json = r#"[
        {"id":1,"name":"Alice","age":30},
        {"id":2,"name":"Bob","age":17},
        {"id":3,"name":"Carol","age":25},
        {"id":4,"name":"Dave","age":22}
    ]"#;
    // select is unavailable after order_by (Sorted state has no select).
    // Collect sorted users first, then project with std iterator.
    let sorted_adults: Vec<User> = QueryBuilder::<User, _>::from_json(json)
        .unwrap()
        .where_(|u| u.age >= 18)
        .order_by(|u| u.age)
        .collect();
    let names: Vec<String> = sorted_adults.into_iter().map(|u| u.name).collect();
    assert_eq!(names, vec!["Dave", "Carol", "Alice"]);
}

#[test]
fn from_json_count() {
    let json = r#"[{"id":1,"name":"A","age":20},{"id":2,"name":"B","age":15}]"#;
    let count = QueryBuilder::<User, _>::from_json(json)
        .unwrap()
        .where_(|u| u.age >= 18)
        .count();
    assert_eq!(count, 1);
}

#[test]
fn from_json_nested_struct() {
    let json = r#"[
        {"name":"Widget","price":9.99,"in_stock":true},
        {"name":"Gadget","price":49.99,"in_stock":false},
        {"name":"Doohickey","price":4.99,"in_stock":true}
    ]"#;
    // select unavailable after order_by (Sorted state); project after collect.
    let sorted: Vec<Product> = QueryBuilder::<Product, _>::from_json(json)
        .unwrap()
        .where_(|p| p.in_stock)
        .order_by(|p| (p.price * 100.0) as i64)
        .collect();
    let available: Vec<String> = sorted.into_iter().map(|p| p.name).collect();
    assert_eq!(available, vec!["Doohickey", "Widget"]);
}

// ── from_json_value ───────────────────────────────────────────────────────────

#[test]
fn from_json_value_basic() {
    let json = r#"[{"age":30},{"age":17},{"age":25}]"#;
    let adults: Vec<_> = QueryBuilder::from_json_value(json)
        .unwrap()
        .where_(|v| v["age"].as_u64().unwrap_or(0) >= 18)
        .collect();
    assert_eq!(adults.len(), 2);
}

#[test]
fn from_json_value_empty() {
    let result: Vec<_> = QueryBuilder::from_json_value(r#"[]"#)
        .unwrap()
        .collect();
    assert!(result.is_empty());
}

#[test]
fn from_json_value_invalid_returns_err() {
    let result = QueryBuilder::from_json_value("{not an array}");
    assert!(result.is_err());
}

#[test]
fn from_json_value_dynamic_field_access() {
    let json = r#"[{"name":"Alice","score":95},{"name":"Bob","score":60},{"name":"Carol","score":80}]"#;
    let high_scorers: Vec<String> = QueryBuilder::from_json_value(json)
        .unwrap()
        .where_(|v| v["score"].as_u64().unwrap_or(0) >= 80)
        .select(|v| v["name"].as_str().unwrap_or("").to_string())
        .collect();
    assert_eq!(high_scorers.len(), 2);
    assert!(high_scorers.contains(&"Alice".to_string()));
    assert!(high_scorers.contains(&"Carol".to_string()));
}

// ── rinq::serde module path ───────────────────────────────────────────────────

#[test]
fn serde_module_re_export_works() {
    use rinq::serde::QueryBuilder as SerdeQueryBuilder;

    let json = r#"[1, 2, 3]"#;
    let result: Vec<i32> = SerdeQueryBuilder::<i32, _>::from_json(json)
        .unwrap()
        .where_(|x| *x > 1)
        .collect();
    assert_eq!(result, vec![2, 3]);
}

// ── combined with other v3 features ──────────────────────────────────────────

#[test]
fn from_json_then_running_sum() {
    let json = r#"[1, 2, 3, 4, 5]"#;
    let cumsum: Vec<i32> = QueryBuilder::<i32, _>::from_json(json)
        .unwrap()
        .running_sum()
        .collect();
    assert_eq!(cumsum, vec![1, 3, 6, 10, 15]);
}

#[test]
fn from_json_then_try_select() {
    // JSON of string-encoded numbers
    let json = r#"["1","two","3"]"#;
    let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::<String, _>::from_json(json)
        .unwrap()
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert_eq!(ok, vec![1, 3]);
    assert_eq!(err.len(), 1);
}
