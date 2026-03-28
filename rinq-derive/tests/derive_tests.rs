// rinq-derive/tests/derive_tests.rs
// Integration tests for #[derive(Queryable)] and #[derive(QueryableFrom)]

use rinq::QueryBuilder;
use rinq_derive::{Queryable, QueryableFrom};

// ── Test structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Queryable)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Queryable)]
struct Product {
    id: u32,
    #[queryable(skip)]
    internal_code: String,
    #[queryable(rename = "price_jpy")]
    price: f64,
    #[queryable(key)]
    category: String,
}

#[derive(Debug, Clone, PartialEq, QueryableFrom)]
struct UserList(Vec<User>);

#[derive(Debug, Clone, PartialEq, QueryableFrom)]
struct UserRepo {
    users: Vec<User>,
}

// ── F1: basic accessor tests ──────────────────────────────────────────────────

#[test]
fn derive_basic_expansion() {
    // Verify the derive compiles and the accessor functions exist
    let users = sample_users();
    let _: Vec<_> = QueryBuilder::from(users).order_by(User::by_age).collect();
}

#[test]
fn accessor_order_by_age() {
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .order_by(User::by_age)
        .collect();
    assert_eq!(result[0].age, 17);
    assert_eq!(result[1].age, 25);
    assert_eq!(result[2].age, 30);
}

#[test]
fn accessor_order_by_name() {
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .order_by(User::by_name)
        .collect();
    assert_eq!(result[0].name, "Alice");
    assert_eq!(result[1].name, "Bob");
    assert_eq!(result[2].name, "Charlie");
}

#[test]
fn accessor_group_by_active() {
    use std::collections::HashMap;
    let groups: HashMap<bool, Vec<User>> =
        QueryBuilder::from(sample_users()).group_by(User::by_active);
    assert_eq!(groups[&true].len(), 2);
    assert_eq!(groups[&false].len(), 1);
}

// ── F1: Age predicate tests ───────────────────────────────────────────────────

#[test]
fn age_gt_predicate() {
    use user_fields::Age;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::gt(18))
        .collect();
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|u| u.age > 18));
}

#[test]
fn age_lt_predicate() {
    use user_fields::Age;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::lt(25))
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].age, 17);
}

#[test]
fn age_eq_predicate() {
    use user_fields::Age;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::eq(25))
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Alice");
}

#[test]
fn age_between_predicate() {
    use user_fields::Age;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::between(20, 29))
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].age, 25);
}

#[test]
fn age_between_inclusive_bounds() {
    use user_fields::Age;
    // between(17, 25) should include both endpoints
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::between(17, 25))
        .collect();
    assert_eq!(result.len(), 2);
}

// ── F1: Active predicate tests ────────────────────────────────────────────────

#[test]
fn active_is_true_predicate() {
    use user_fields::Active;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Active::is_true())
        .collect();
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|u| u.active));
}

#[test]
fn active_is_false_predicate() {
    use user_fields::Active;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Active::is_false())
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Bob");
}

// ── F1: Name predicate tests ──────────────────────────────────────────────────

#[test]
fn name_contains_predicate() {
    use user_fields::Name;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Name::contains("Alice"))
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Alice");
}

#[test]
fn name_eq_predicate() {
    use user_fields::Name;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Name::eq("Charlie"))
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].age, 30);
}

#[test]
fn name_starts_with_predicate() {
    use user_fields::Name;
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Name::starts_with("A"))
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Alice");
}

#[test]
fn name_is_empty_predicate() {
    use user_fields::Name;
    let mut users = sample_users();
    users.push(User {
        name: String::new(),
        age: 40,
        active: true,
    });
    let result: Vec<_> = QueryBuilder::from(users).where_(Name::is_empty()).collect();
    assert_eq!(result.len(), 1);
}

// ── F1: chained predicates ────────────────────────────────────────────────────

#[test]
fn chained_predicates() {
    use user_fields::{Active, Age};
    // Age::between(25,27) picks only Alice; Active::is_true() also passes for Alice.
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::between(25, 27))
        .where_(Active::is_true())
        .order_by(User::by_age)
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Alice");
}

// ── F1: #[queryable(skip)] tests ──────────────────────────────────────────────

#[test]
fn skip_attr_does_not_affect_other_fields() {
    // internal_code is skipped; by_id and by_price_jpy (renamed) should still work
    let products = sample_products();
    let result: Vec<_> = QueryBuilder::from(products)
        .order_by(Product::by_id)
        .collect();
    assert_eq!(result[0].id, 1);
}

// ── F1: #[queryable(rename = "...")] tests ────────────────────────────────────

#[test]
fn rename_attr_changes_accessor_name() {
    // price is renamed to price_jpy — verify the accessor is generated with the new name
    // and returns the correct value.  (f64 doesn't implement Ord, so we call it directly.)
    let product = Product {
        id: 1,
        internal_code: "A01".into(),
        price: 800.0,
        category: "A".into(),
    };
    assert_eq!(Product::by_price_jpy(&product), 800.0);
}

// ── F1: #[queryable(key)] tests ───────────────────────────────────────────────

#[test]
fn key_attr_generates_default_key() {
    let products = sample_products();
    let result: Vec<_> = QueryBuilder::from(products)
        .order_by(Product::default_key)
        .collect();
    // Sorted by category alphabetically
    assert!(result[0].category <= result[1].category);
}

// ── F1: variable name conflict / hygiene ─────────────────────────────────────

#[test]
fn no_conflict_with_user_variable_named_item() {
    // Variables named 'user', '__rinq_item', etc. must not conflict with
    // identifiers generated by the proc-macro.
    use user_fields::Age;
    let user = "this is a local variable, not the struct";
    let __rinq_item = 999;

    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Age::gt(18))
        .collect();

    // Suppress "unused variable" warnings
    let _ = user;
    let _ = __rinq_item;

    assert_eq!(result.len(), 2);
}

#[test]
fn no_conflict_with_variable_named_s() {
    // The string predicate closures internally use `__s`; user's `s` must be fine.
    use user_fields::Name;
    let s = "shadowed outer variable";
    let result: Vec<_> = QueryBuilder::from(sample_users())
        .where_(Name::contains("Alice"))
        .collect();
    let _ = s;
    assert_eq!(result.len(), 1);
}

// ── F2: QueryableFrom (tuple struct) tests ────────────────────────────────────

#[test]
fn queryable_from_tuple_struct_into_query() {
    use rinq::IntoQuery;
    let list = UserList(sample_users());
    let result: Vec<_> = list.into_query().where_(|u| u.age > 18).collect();
    assert_eq!(result.len(), 2);
}

#[test]
fn queryable_from_tuple_struct_from_impl() {
    let list = UserList(sample_users());
    let qb: QueryBuilder<User, rinq::Initial> = list.into();
    let result: Vec<_> = qb.collect();
    assert_eq!(result.len(), 3);
}

#[test]
fn queryable_from_named_struct_into_query() {
    use rinq::IntoQuery;
    let repo = UserRepo {
        users: sample_users(),
    };
    let result: Vec<_> = repo.into_query().where_(|u| u.active).collect();
    assert_eq!(result.len(), 2);
}

#[test]
fn queryable_from_empty_collection() {
    use rinq::IntoQuery;
    let list = UserList(vec![]);
    let result: Vec<_> = list.into_query().collect();
    assert!(result.is_empty());
}

#[test]
fn queryable_from_chained_with_predicates() {
    use rinq::IntoQuery;
    use user_fields::{Active, Age};
    // Age::between(25,27) selects only Alice; she is also active.
    let list = UserList(sample_users());
    let result: Vec<_> = list
        .into_query()
        .where_(Age::between(25, 27))
        .where_(Active::is_true())
        .order_by(User::by_name)
        .collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Alice");
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn sample_users() -> Vec<User> {
    vec![
        User {
            name: "Alice".into(),
            age: 25,
            active: true,
        },
        User {
            name: "Bob".into(),
            age: 17,
            active: false,
        },
        User {
            name: "Charlie".into(),
            age: 30,
            active: true,
        },
    ]
}

fn sample_products() -> Vec<Product> {
    vec![
        Product {
            id: 2,
            internal_code: "X99".into(),
            price: 1500.0,
            category: "B".into(),
        },
        Product {
            id: 1,
            internal_code: "A01".into(),
            price: 800.0,
            category: "A".into(),
        },
    ]
}
