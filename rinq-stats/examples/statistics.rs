// rinq-stats/examples/statistics.rs
// Demonstrates StatisticsExt: median, std_dev, percentile, histogram,
// skewness, kurtosis, and linear_regression via QueryPair.

use rinq::QueryBuilder;
use rinq_stats::{StatisticsExt, QueryPair};

fn main() {
    println!("=== rinq-stats: Descriptive Statistics ===\n");

    let data = vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    println!("Data: {:?}", data);

    let mean = QueryBuilder::from(data.clone()).average().unwrap();
    println!("Mean:     {mean:.4}");

    let median = QueryBuilder::from(data.clone()).median().unwrap();
    println!("Median:   {median:.4}");

    let std_dev = QueryBuilder::from(data.clone()).std_dev();
    println!("Std dev:  {std_dev:.4}");

    let p90 = QueryBuilder::from(data.clone()).percentile(90.0).unwrap();
    println!("P90:      {p90:.4}");

    let skew = QueryBuilder::from(data.clone()).skewness();
    println!("Skewness: {skew:.4}");

    let kurt = QueryBuilder::from(data.clone()).kurtosis();
    println!("Kurtosis: {kurt:.4}");

    // Histogram (4 buckets) — HistogramBucket has `range` and `count`
    println!("\nHistogram (4 buckets):");
    let buckets = QueryBuilder::from(data.clone()).histogram(4);
    for b in &buckets {
        let bar = "#".repeat(b.count);
        println!(
            "  [{:.1}, {:.1}] — {bar} ({})",
            b.range.start(),
            b.range.end(),
            b.count
        );
    }

    // Linear regression via QueryPair
    let xs = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let ys = vec![2.1_f64, 3.9, 6.2, 8.0, 10.1];
    let pair = QueryPair::new(xs, ys);
    let (slope, intercept) = pair.linear_regression();
    println!("\nLinear regression y ≈ {slope:.3}x + {intercept:.3}");
}
