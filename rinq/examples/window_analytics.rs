// rinq/examples/window_analytics.rs
// Demonstrates window analytics: running_sum, moving_average, rank_by, lag, lead.

use rinq::QueryBuilder;

fn main() {
    println!("=== RINQ Window Analytics ===\n");

    let prices = vec![10.0_f64, 12.0, 11.0, 14.0, 13.0, 15.0, 16.0];
    println!("Prices: {:?}", prices);

    // running_sum — returns a lazy QueryBuilder, so collect it
    let running: Vec<f64> = QueryBuilder::from(prices.clone()).running_sum().collect();
    println!("Running sum:  {:?}", running);

    // moving_average (window = 3) — returns QueryBuilder<Option<f64>>
    let ma: Vec<Option<f64>> = QueryBuilder::from(prices.clone())
        .moving_average(3)
        .collect();
    let ma_vals: Vec<String> = ma
        .iter()
        .map(|v| match v {
            Some(x) => format!("{x:.2}"),
            None => "—".to_owned(),
        })
        .collect();
    println!("Moving avg 3: {:?}", ma_vals);

    // rank_by — key selector (ascending rank, returns (usize, T))
    let ranked: Vec<(usize, f64)> = QueryBuilder::from(prices.clone())
        .rank_by(|x| *x as i64)
        .collect();
    println!("Rank (asc):   {:?}", ranked);

    // lag — returns (Option<T>, T) pairs
    let lag1: Vec<(Option<f64>, f64)> = QueryBuilder::from(prices.clone()).lag(1).collect();
    println!("Lag(1):       {:?}", lag1);

    // lead — returns (T, Option<T>) pairs
    let lead1: Vec<(f64, Option<f64>)> = QueryBuilder::from(prices.clone()).lead(1).collect();
    println!("Lead(1):      {:?}", lead1);
}
