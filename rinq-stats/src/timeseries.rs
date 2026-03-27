// rinq-stats/src/timeseries.rs
// Phase 4G I1: TimeSeriesExt — EMA and Bollinger Bands.

use rinq::QueryBuilder;

/// A single Bollinger Band data point.
#[derive(Debug, Clone, PartialEq)]
pub struct BollingerPoint {
    /// The value itself.
    pub value: f64,
    /// Moving average over the preceding window.
    pub middle: f64,
    /// Upper band: middle + sigma * std_dev.
    pub upper: f64,
    /// Lower band: middle - sigma * std_dev.
    pub lower: f64,
}

/// Time-series operations for sequences of `f64` values.
pub trait TimeSeriesExt {
    /// Exponential Moving Average.
    ///
    /// EMA is defined recursively:
    ///   ema[0] = values[0]
    ///   ema[i] = alpha * values[i] + (1 - alpha) * ema[i-1]
    ///
    /// # Panics
    ///
    /// Panics if `alpha` is not in `(0.0, 1.0]`.
    ///
    /// # Returns
    ///
    /// An empty `Vec` for an empty input.
    fn exponential_moving_average(self, alpha: f64) -> Vec<f64>;

    /// Bollinger Bands over a rolling window.
    ///
    /// For each position `i` where `i >= window - 1`, computes:
    /// - `middle` = arithmetic mean of `values[i-window+1 ..= i]`
    /// - `std_dev` = population standard deviation of the same window
    /// - `upper`  = middle + sigma * std_dev
    /// - `lower`  = middle - sigma * std_dev
    ///
    /// Points with `i < window - 1` are excluded from the output.
    ///
    /// # Returns
    ///
    /// - Empty `Vec` if `window == 0` or `window > values.len()`.
    fn bollinger_bands(self, window: usize, sigma: f64) -> Vec<BollingerPoint>;
}

impl<State: 'static> TimeSeriesExt for QueryBuilder<f64, State> {
    fn exponential_moving_average(self, alpha: f64) -> Vec<f64> {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha must be in (0.0, 1.0]");
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(values.len());
        let mut ema = values[0];
        result.push(ema);
        for &v in values.iter().skip(1) {
            ema = alpha * v + (1.0 - alpha) * ema;
            result.push(ema);
        }
        result
    }

    fn bollinger_bands(self, window: usize, sigma: f64) -> Vec<BollingerPoint> {
        let values: Vec<f64> = self.collect();
        if window == 0 || window > values.len() {
            return Vec::new();
        }
        values
            .windows(window)
            .zip(values.iter().skip(window - 1))
            .map(|(w, &value)| {
                let mean = w.iter().sum::<f64>() / window as f64;
                let variance =
                    w.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window as f64;
                let std_dev = variance.sqrt();
                BollingerPoint {
                    value,
                    middle: mean,
                    upper: mean + sigma * std_dev,
                    lower: mean - sigma * std_dev,
                }
            })
            .collect()
    }
}
