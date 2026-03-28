// rinq/examples/metrics_example.rs
// Demonstrates MetricsQueryBuilder — per-query execution counters.

use rinq::{MetricsQueryBuilder, MetricsCollector, QueryBuilder};
use std::sync::Arc;

fn main() {
    println!("=== RINQ Metrics ===\n");

    let collector = Arc::new(MetricsCollector::new());

    // Run several named queries
    let evens: Vec<i32> = MetricsQueryBuilder::new(
        QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8]),
        Arc::clone(&collector),
        "even_filter".to_owned(),
    )
    .where_(|x| x % 2 == 0)
    .collect();
    println!("Evens: {:?}", evens);

    let _count = MetricsQueryBuilder::new(
        QueryBuilder::from(vec![10, 20, 30, 40, 50]),
        Arc::clone(&collector),
        "big_numbers".to_owned(),
    )
    .where_(|x| *x > 25)
    .count();

    // Run even_filter again to see the counter increment
    let _: Vec<i32> = MetricsQueryBuilder::new(
        QueryBuilder::from(vec![1, 2, 3]),
        Arc::clone(&collector),
        "even_filter".to_owned(),
    )
    .where_(|x| x % 2 == 0)
    .collect();

    // Read specific metric keys
    println!("\nMetrics:");
    for key in ["query_even_filter", "query_big_numbers_count"] {
        let val = collector.get(key).unwrap_or(0);
        println!("  {key}: {val}");
    }
}
