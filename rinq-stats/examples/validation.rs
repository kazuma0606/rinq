// rinq-stats/examples/validation.rs
// Demonstrates ValidationExt: validate, validate_if, validate_with,
// validate_range, validate_unique, validate_non_empty, report.

use rinq::QueryBuilder;
use rinq_stats::ValidationExt;

#[derive(Clone, Debug)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

fn main() {
    println!("=== rinq-stats: Validation ===\n");

    let products = vec![
        Product {
            id: 1,
            name: "Widget".to_owned(),
            price: 9.99,
        },
        Product {
            id: 2,
            name: "".to_owned(),
            price: -5.0,
        },
        Product {
            id: 3,
            name: "Gadget".to_owned(),
            price: 250.0,
        },
        Product {
            id: 2,
            name: "Duplicate".to_owned(),
            price: 1.0,
        }, // duplicate id
    ];

    println!("Products: {:#?}\n", products);

    // Chain multiple rules
    let report = QueryBuilder::from(products.clone())
        .validate(|_| true, "always_ok", "dummy")
        .validate_non_empty(|p| p.name.as_str(), "name_required")
        .validate_range(|p| p.price, 0.0, 200.0, "price_range")
        .validate_unique(|p| p.id, "unique_id")
        .report();

    if report.is_empty() {
        println!("All products are valid.");
    } else {
        println!("Validation errors:");
        for msg in &report {
            println!("  {msg}");
        }
    }

    // collect_valid — keep only the passing items
    println!("\nValid products (all rules pass):");
    let valid = QueryBuilder::from(products.clone())
        .validate(|_| true, "always_ok", "dummy")
        .validate_non_empty(|p| p.name.as_str(), "name_required")
        .validate_range(|p| p.price, 0.0, 200.0, "price_range")
        .validate_unique(|p| p.id, "unique_id")
        .collect_valid();
    for p in &valid {
        println!("  {:?}", p);
    }
}
