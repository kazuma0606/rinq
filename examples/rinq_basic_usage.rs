// examples/rinq_basic_usage.rs
// Basic usage example for RINQ v0.1

use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    println!("=== RINQ v0.1 Basic Usage Examples ===\n");

    // Example 1: Basic filtering
    println!("Example 1: Filtering even numbers");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let even_numbers: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("Input: {:?}", numbers);
    println!("Even numbers: {:?}\n", even_numbers);

    // Example 2: Chaining filters
    println!("Example 2: Chaining multiple filters");
    let filtered: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .where_(|x| *x > 4)
        .collect();
    println!("Even numbers > 4: {:?}\n", filtered);

    // Example 3: Projection (select)
    println!("Example 3: Projection");
    let doubled: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .select(|x| x * 2)
        .collect();
    println!("Even numbers doubled: {:?}\n", doubled);

    // Example 4: Sorting
    println!("Example 4: Sorting");
    let unsorted = vec![5, 2, 8, 1, 9, 3];
    let sorted: Vec<_> = QueryBuilder::from(unsorted.clone())
        .where_(|_| true)
        .order_by(|x| *x)
        .collect();
    println!("Unsorted: {:?}", unsorted);
    println!("Sorted: {:?}\n", sorted);

    // Example 5: Pagination
    println!("Example 5: Pagination");
    let page: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|_| true)
        .skip(3)
        .take(4)
        .collect();
    println!("Skip 3, take 4: {:?}\n", page);

    // Example 6: Aggregations
    println!("Example 6: Aggregations");
    let count = QueryBuilder::from(numbers.clone()).count();
    let first = QueryBuilder::from(numbers.clone()).first();
    let last = QueryBuilder::from(numbers.clone()).last();
    let any_gt_5 = QueryBuilder::from(numbers.clone()).any(|x| *x > 5);
    let all_positive = QueryBuilder::from(numbers.clone()).all(|x| *x > 0);
    
    println!("Count: {}", count);
    println!("First: {:?}", first);
    println!("Last: {:?}", last);
    println!("Any > 5: {}", any_gt_5);
    println!("All positive: {}\n", all_positive);

    // Example 7: Type state pattern demonstration
    println!("Example 7: Type state pattern ensures compile-time safety");
    println!("The following operations are type-safe:");
    
    // Valid: Initial -> Filtered
    let _q1 = QueryBuilder::from(numbers.clone()).where_(|x| x % 2 == 0);
    println!("✓ Initial -> Filtered (where_)");
    
    // Valid: Filtered -> Sorted
    let _q2 = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .order_by(|x| *x);
    println!("✓ Filtered -> Sorted (order_by)");
    
    // Valid: Filtered -> Projected
    let _q3 = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .select(|x| x * 2);
    println!("✓ Filtered -> Projected (select)");
    
    println!("\nThe type system prevents invalid operations at compile time!");
    println!("For example, you cannot call order_by() on Initial state.");
    println!("This ensures query correctness before runtime.");
}
