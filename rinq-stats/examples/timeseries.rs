// rinq-stats/examples/timeseries.rs
// Demonstrates TimeSeriesExt: EMA, SMA, WMA, Bollinger Bands, ROC, seasonal decompose.

use rinq::QueryBuilder;
use rinq_stats::TimeSeriesExt;

fn main() {
    println!("=== rinq-stats: Time Series ===\n");

    let prices = vec![
        100.0_f64, 102.0, 101.0, 105.0, 107.0, 106.0, 110.0, 112.0, 111.0, 115.0,
    ];
    println!("Prices: {:?}", prices);

    // EMA
    let ema = QueryBuilder::from(prices.clone()).exponential_moving_average(0.3);
    println!(
        "\nEMA(α=0.3): {:?}",
        ema.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>()
    );

    // SMA (window=3)
    let sma = QueryBuilder::from(prices.clone()).simple_moving_average(3);
    println!(
        "SMA(3):     {:?}",
        sma.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>()
    );

    // WMA (window=3)
    let wma = QueryBuilder::from(prices.clone()).weighted_moving_average(3);
    println!(
        "WMA(3):     {:?}",
        wma.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>()
    );

    // Rate of Change
    let roc = QueryBuilder::from(prices.clone()).rate_of_change(1);
    println!(
        "ROC(1)%:    {:?}",
        roc.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>()
    );

    // Bollinger Bands (window=5, sigma=2)
    println!("\nBollinger Bands (window=5, σ=2):");
    let bands = QueryBuilder::from(prices.clone()).bollinger_bands(5, 2.0);
    for (i, b) in bands.iter().enumerate() {
        println!(
            "  [{}] val={:.1}  mid={:.2}  lo={:.2}  hi={:.2}",
            i + 4,
            b.value,
            b.middle,
            b.lower,
            b.upper
        );
    }

    // Seasonal decomposition (period=4)
    let seasonal_data: Vec<f64> = (0..16).map(|i| (i % 4) as f64 * 2.0 + 10.0).collect();
    println!(
        "\nSeasonal decompose (period=4, {} points):",
        seasonal_data.len()
    );
    let decomp = QueryBuilder::from(seasonal_data).seasonal_decompose(4);
    let mid = decomp.trend.len() / 2;
    println!(
        "  trend[{}]={:.2}  seasonal[{}]={:.2}  residual[{}]={:.2}",
        mid, decomp.trend[mid], mid, decomp.seasonal[mid], mid, decomp.residual[mid]
    );
}
