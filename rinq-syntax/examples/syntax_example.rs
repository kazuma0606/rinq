// rinq-syntax/examples/syntax_example.rs
// Demonstrates the query! macro for SQL-like query syntax.

use rinq_syntax::query;

fn main() {
    println!("=== rinq-syntax: query! macro ===\n");

    // Basic filter + projection
    let numbers = vec![1_i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let evens_doubled: Vec<i32> = query! {
        from x in numbers.clone()
        where *x % 2 == 0
        select x * 2
    };
    println!("Even numbers doubled: {:?}", evens_doubled);

    // Filter + sort descending + take (use `desc` keyword)
    let top3: Vec<i32> = query! {
        from x in numbers.clone()
        where *x > 3
        order_by *x desc
        take 3
    };
    println!("Top 3 (>3, desc):     {:?}", top3);

    // Projection to new type
    let labels: Vec<String> = query! {
        from x in vec![1_i32, 2, 3]
        select format!("item-{x}")
    };
    println!("Labels:               {:?}", labels);

    // Skip + take (pagination)
    let page2: Vec<i32> = query! {
        from x in (1..=10).collect::<Vec<_>>()
        skip 3
        take 3
    };
    println!("Page 2 (skip 3, take 3): {:?}", page2);
}
