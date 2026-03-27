// rinq-syntax/tests/syntax_tests.rs
// Integration tests for the `query!` macro.

use rinq_syntax::query;

// ── helpers ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct User {
    name: String,
    age: u32,
    active: bool,
    department: String,
}

fn sample_users() -> Vec<User> {
    vec![
        User { name: "Alice".into(),   age: 28, active: true,  department: "Engineering".into() },
        User { name: "Bob".into(),     age: 17, active: false, department: "Marketing".into()   },
        User { name: "Charlie".into(), age: 35, active: true,  department: "Engineering".into() },
    ]
}

// ── G1: basic from / where / collect ─────────────────────────────────────────

#[test]
fn basic_from_only() {
    let result: Vec<_> = query! {
        from user in sample_users()
    };
    assert_eq!(result.len(), 3);
}

#[test]
fn from_where_single() {
    let result: Vec<_> = query! {
        from user in sample_users()
        where user.age >= 18
    };
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|u| u.age >= 18));
}

#[test]
fn from_where_multiple() {
    let result: Vec<_> = query! {
        from user in sample_users()
        where user.age >= 18
        where user.active
    };
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|u| u.age >= 18 && u.active));
}

#[test]
fn from_take() {
    let result: Vec<_> = query! {
        from user in sample_users()
        take 2
    };
    assert_eq!(result.len(), 2);
}

#[test]
fn from_skip() {
    let result: Vec<_> = query! {
        from user in sample_users()
        skip 1
    };
    assert_eq!(result.len(), 2);
}

// ── G1: order_by ─────────────────────────────────────────────────────────────

#[test]
fn order_by_numeric_asc() {
    let result: Vec<_> = query! {
        from user in sample_users()
        order_by user.age
    };
    assert_eq!(result[0].age, 17);
    assert_eq!(result[1].age, 28);
    assert_eq!(result[2].age, 35);
}

#[test]
fn order_by_numeric_desc() {
    let result: Vec<_> = query! {
        from user in sample_users()
        order_by user.age desc
    };
    assert_eq!(result[0].age, 35);
    assert_eq!(result[1].age, 28);
    assert_eq!(result[2].age, 17);
}

#[test]
fn order_by_string() {
    let result: Vec<_> = query! {
        from user in sample_users()
        order_by user.name.clone()
    };
    assert_eq!(result[0].name, "Alice");
    assert_eq!(result[1].name, "Bob");
    assert_eq!(result[2].name, "Charlie");
}

// ── G1: select ────────────────────────────────────────────────────────────────

#[test]
fn select_projection() {
    let result: Vec<String> = query! {
        from user in sample_users()
        where user.active
        select user.name
    };
    assert_eq!(result, vec!["Alice", "Charlie"]);
}

#[test]
fn select_computed() {
    let result: Vec<u32> = query! {
        from user in sample_users()
        select user.age * 2
    };
    assert_eq!(result, vec![56, 34, 70]);
}

// ── G1: combined pipeline ─────────────────────────────────────────────────────

#[test]
fn full_pipeline() {
    let result: Vec<String> = query! {
        from user in sample_users()
        where user.age >= 18
        where user.active
        order_by user.age
        select user.name
    };
    assert_eq!(result, vec!["Alice", "Charlie"]);
}

// ── G2: multiple sort keys ────────────────────────────────────────────────────

#[test]
fn order_by_comma_then_by() {
    // order_by dept asc, age desc
    let result: Vec<_> = query! {
        from user in sample_users()
        order_by user.department.clone(), user.age desc
    };
    // Engineering: Charlie(35) before Alice(28) due to age desc
    // Marketing: Bob
    assert_eq!(result[0].name, "Charlie");
    assert_eq!(result[1].name, "Alice");
    assert_eq!(result[2].name, "Bob");
}

#[test]
fn order_by_then_by_separate_clauses() {
    let result: Vec<_> = query! {
        from user in sample_users()
        order_by user.department.clone()
        then_by user.age
    };
    // Engineering: Alice(28) before Charlie(35) because age asc
    assert_eq!(result[0].name, "Alice");
    assert_eq!(result[1].name, "Charlie");
    assert_eq!(result[2].name, "Bob");
}

// ── G1: take / skip combination ───────────────────────────────────────────────

#[test]
fn skip_then_take() {
    let result: Vec<_> = query! {
        from user in sample_users()
        skip 1
        take 1
    };
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Bob");
}

// ── G1: primitive collection ──────────────────────────────────────────────────

#[test]
fn primitive_with_deref() {
    let nums = vec![1i32, 5, 3, 2, 4];
    let result: Vec<i32> = query! {
        from x in nums
        where *x > 2
        order_by *x
        select x * 10
    };
    assert_eq!(result, vec![30, 40, 50]);
}
