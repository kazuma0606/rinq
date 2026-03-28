//! Demonstrates `#[derive(Queryable)]` and `#[derive(QueryableFrom)]`
//! from the `rinq-derive` crate.
//!
//! Run with:
//! ```shell
//! cargo run --example rinq_derive_example
//! ```

use rinq::QueryBuilder;
use rinq_derive::{Queryable, QueryableFrom};

// ── Domain structs ────────────────────────────────────────────────────────────

/// A user record.  `#[derive(Queryable)]` generates:
/// - `User::by_name`, `User::by_age`, `User::by_department` — accessor fns
/// - module `user_fields` with typed predicate builders
#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub active: bool,
    #[queryable(key)]
    pub department: String,
}

/// A newtype wrapper.  `#[derive(QueryableFrom)]` generates `IntoQuery` so
/// that `UserList(vec![...]).into_query()` works directly.
#[derive(Debug, Clone, QueryableFrom)]
pub struct UserList(Vec<User>);

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let users = vec![
        User {
            name: "Alice".into(),
            age: 28,
            active: true,
            department: "Engineering".into(),
        },
        User {
            name: "Bob".into(),
            age: 22,
            active: false,
            department: "Marketing".into(),
        },
        User {
            name: "Charlie".into(),
            age: 35,
            active: true,
            department: "Engineering".into(),
        },
        User {
            name: "Diana".into(),
            age: 17,
            active: true,
            department: "Intern".into(),
        },
        User {
            name: "Eve".into(),
            age: 42,
            active: true,
            department: "Engineering".into(),
        },
    ];

    println!("=== Using typed predicates ===");
    {
        use user_fields::{Active, Age};

        let result: Vec<_> = QueryBuilder::from(users.clone())
            .where_(Age::between(20, 40))
            .where_(Active::is_true())
            .order_by(User::by_age)
            .collect();
        println!("Active users aged 20-40, sorted by age:");
        for u in &result {
            println!("  {:?}", u);
        }
    }

    println!("\n=== Using accessor functions directly ===");
    {
        let groups = QueryBuilder::from(users.clone()).group_by(User::by_department);
        for (dept, members) in &groups {
            let names: Vec<_> = members.iter().map(|u| u.name.as_str()).collect();
            println!("  {}: {:?}", dept, names);
        }
    }

    println!("\n=== Using QueryableFrom / IntoQuery ===");
    {
        use rinq::IntoQuery;
        use user_fields::Age;

        let list = UserList(users.clone());
        let adults: Vec<_> = list.into_query().where_(Age::gt(18)).collect();
        println!("Adults ({}):", adults.len());
        for u in &adults {
            println!("  {} ({})", u.name, u.age);
        }
    }

    println!("\n=== default_key (marked with #[queryable(key)]) ===");
    {
        let result: Vec<_> = QueryBuilder::from(users)
            .order_by(User::default_key) // sorts by department
            .collect();
        println!("Sorted by department:");
        for u in &result {
            println!("  {} — {}", u.name, u.department);
        }
    }
}
