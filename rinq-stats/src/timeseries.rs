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

/// Output of [`TimeSeriesExt::seasonal_decompose`].
#[derive(Debug, Clone)]
pub struct SeasonalDecomposition {
    /// Trend component (centered moving average).
    pub trend: Vec<f64>,
    /// Seasonal component (average deviation from trend per period position).
    pub seasonal: Vec<f64>,
    /// Residual = original − trend − seasonal.
    pub residual: Vec<f64>,
}

/// Time-series operations for sequences of `f64` values.
pub trait TimeSeriesExt {
    /// Exponential Moving Average.
    ///
    /// EMA is defined recursively:
    ///   `ema[0] = values[0]`
    ///   `ema[i] = alpha * values[i] + (1 - alpha) * ema[i-1]`
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

    /// Simple Moving Average (SMA): arithmetic mean of each sliding window.
    ///
    /// Returns one value per position where a full window of `window` elements
    /// is available (i.e., `values.len() - window + 1` values total).
    ///
    /// Returns an empty `Vec` if `window == 0` or `window > values.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::TimeSeriesExt;
    ///
    /// let sma = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0])
    ///     .simple_moving_average(3);
    /// assert_eq!(sma.len(), 3);
    /// assert!((sma[0] - 2.0).abs() < 1e-10); // (1+2+3)/3
    /// assert!((sma[2] - 4.0).abs() < 1e-10); // (3+4+5)/3
    /// ```
    fn simple_moving_average(self, window: usize) -> Vec<f64>;

    /// Weighted Moving Average (WMA): linearly weighted mean of each window.
    ///
    /// Weight for position `i` in the window (0-indexed from oldest) is `i + 1`.
    /// The most recent element carries the highest weight.
    ///
    /// Returns an empty `Vec` if `window == 0` or `window > values.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::TimeSeriesExt;
    ///
    /// // window = 3: weights 1, 2, 3 → WMA([1,2,3]) = (1·1 + 2·2 + 3·3)/6 = 14/6
    /// let wma = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0])
    ///     .weighted_moving_average(3);
    /// assert!((wma[0] - 14.0 / 6.0).abs() < 1e-10);
    /// ```
    fn weighted_moving_average(self, window: usize) -> Vec<f64>;

    /// Percentage rate of change between elements separated by `period` steps.
    ///
    /// `roc[i] = (values[i] - values[i - period]) / values[i - period] * 100.0`
    ///
    /// The first `period` elements have no predecessor, so the output has
    /// `values.len() - period` elements.
    ///
    /// If a base value is `0.0`, the corresponding ROC is `f64::NAN`.
    /// Returns an empty `Vec` if `period == 0` or `period >= values.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::TimeSeriesExt;
    ///
    /// let roc = QueryBuilder::from(vec![100.0_f64, 110.0, 121.0])
    ///     .rate_of_change(1);
    /// assert!((roc[0] - 10.0).abs() < 1e-10); // (110-100)/100 * 100
    /// assert!((roc[1] - 10.0).abs() < 1e-10); // (121-110)/110 * 100
    /// ```
    fn rate_of_change(self, period: usize) -> Vec<f64>;

    /// Additive seasonal decomposition: trend + seasonal + residual.
    ///
    /// Uses a centered moving average of length `period` as the trend.
    /// Positions without a full window (first and last `period/2` elements)
    /// receive `f64::NAN` for trend and residual; seasonal is always computed.
    ///
    /// Returns a [`SeasonalDecomposition`] with three `Vec<f64>` of the same
    /// length as the input. Returns one with empty `Vec`s if input is empty or
    /// `period == 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::TimeSeriesExt;
    ///
    /// let data: Vec<f64> = (0..12).map(|i| (i % 4) as f64 + 10.0).collect();
    /// let decomp = QueryBuilder::from(data).seasonal_decompose(4);
    /// assert_eq!(decomp.trend.len(), 12);
    /// assert_eq!(decomp.seasonal.len(), 12);
    /// assert_eq!(decomp.residual.len(), 12);
    /// ```
    fn seasonal_decompose(self, period: usize) -> SeasonalDecomposition;
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
                let variance = w.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window as f64;
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

    fn simple_moving_average(self, window: usize) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if window == 0 || window > values.len() {
            return Vec::new();
        }
        values
            .windows(window)
            .map(|w| w.iter().sum::<f64>() / window as f64)
            .collect()
    }

    fn weighted_moving_average(self, window: usize) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if window == 0 || window > values.len() {
            return Vec::new();
        }
        let weight_sum = (1..=window).sum::<usize>() as f64;
        values
            .windows(window)
            .map(|w| {
                w.iter()
                    .enumerate()
                    .map(|(i, &v)| v * (i + 1) as f64)
                    .sum::<f64>()
                    / weight_sum
            })
            .collect()
    }

    fn rate_of_change(self, period: usize) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if period == 0 || period >= values.len() {
            return Vec::new();
        }
        values
            .iter()
            .skip(period)
            .zip(values.iter())
            .map(|(&current, &base)| {
                if base == 0.0 {
                    f64::NAN
                } else {
                    (current - base) / base * 100.0
                }
            })
            .collect()
    }

    fn seasonal_decompose(self, period: usize) -> SeasonalDecomposition {
        let values: Vec<f64> = self.collect();
        let n = values.len();
        if n == 0 || period == 0 {
            return SeasonalDecomposition {
                trend: Vec::new(),
                seasonal: Vec::new(),
                residual: Vec::new(),
            };
        }

        // Centered moving average for trend
        let half = period / 2;
        let mut trend = vec![f64::NAN; n];
        #[allow(clippy::needless_range_loop)]
        for i in half..n.saturating_sub(half) {
            let start = i - half;
            let end = (i + half + 1).min(n);
            if end - start == period {
                trend[i] = values[start..end].iter().sum::<f64>() / period as f64;
            }
        }

        // Seasonal: average (value - trend) per period position
        let mut season_sum = vec![0.0_f64; period];
        let mut season_cnt = vec![0usize; period];
        for (i, (&v, &t)) in values.iter().zip(trend.iter()).enumerate() {
            if t.is_finite() {
                let pos = i % period;
                season_sum[pos] += v - t;
                season_cnt[pos] += 1;
            }
        }
        let season_avg: Vec<f64> = season_sum
            .iter()
            .zip(season_cnt.iter())
            .map(|(&s, &c)| if c > 0 { s / c as f64 } else { 0.0 })
            .collect();

        let seasonal: Vec<f64> = (0..n).map(|i| season_avg[i % period]).collect();

        // Residual = original - trend - seasonal
        let residual: Vec<f64> = values
            .iter()
            .zip(trend.iter())
            .zip(seasonal.iter())
            .map(|((&v, &t), &s)| if t.is_finite() { v - t - s } else { f64::NAN })
            .collect();

        SeasonalDecomposition {
            trend,
            seasonal,
            residual,
        }
    }
}
