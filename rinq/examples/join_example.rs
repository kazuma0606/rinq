// rinq/examples/join_example.rs
// Demonstrates inner_join, left_join, and cross_join.

use rinq::QueryBuilder;

#[derive(Clone, Debug)]
struct User {
    id: u32,
    name: &'static str,
}

#[derive(Clone, Debug)]
struct Order {
    user_id: u32,
    item: &'static str,
    amount: u32,
}

fn main() {
    println!("=== RINQ JOIN Operations ===\n");

    let users = vec![
        User { id: 1, name: "Alice" },
        User { id: 2, name: "Bob" },
        User { id: 3, name: "Carol" },
    ];

    let orders = vec![
        Order { user_id: 1, item: "Book",   amount: 15 },
        Order { user_id: 1, item: "Pen",    amount: 3  },
        Order { user_id: 2, item: "Laptop", amount: 999 },
    ];

    // inner_join — only users who have orders
    println!("--- inner_join ---");
    let joined: Vec<(User, Order)> = QueryBuilder::from(users.clone())
        .inner_join(orders.clone(), |u| u.id, |o| o.user_id)
        .collect();
    for (u, o) in &joined {
        println!("  {} ordered {} (${})", u.name, o.item, o.amount);
    }

    // left_join — all users, None for those without orders
    println!("\n--- left_join ---");
    let left: Vec<(User, Option<Order>)> = QueryBuilder::from(users.clone())
        .left_join(orders.clone(), |u| u.id, |o| o.user_id)
        .collect();
    for (u, o) in &left {
        match o {
            Some(ord) => println!("  {} → {}", u.name, ord.item),
            None      => println!("  {} → (no orders)", u.name),
        }
    }

    // cross_join — cartesian product of two small sets
    println!("\n--- cross_join ---");
    let colors = vec!["red", "blue"];
    let sizes  = vec!["S", "M", "L"];
    let combos: Vec<(&str, &str)> = QueryBuilder::from(colors)
        .cross_join(sizes)
        .collect();
    for (c, s) in &combos {
        println!("  {c}-{s}");
    }
}
