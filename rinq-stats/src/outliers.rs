// rinq-stats/src/outliers.rs
// Phase 4G I2: OutlierExt — z-score and IQR outlier removal.

use rinq::QueryBuilder;

/// Outlier-detection operations for sequences of `f64` values.
pub trait OutlierExt {
    /// Removes outliers using the z-score method.
    ///
    /// A value is considered an outlier if `|z-score| > threshold`.
    ///
    /// z-score = (x - mean) / std_dev (population)
    ///
    /// # Edge cases
    ///
    /// - Empty input: returns empty `Vec`.
    /// - `std_dev == 0` (all identical values): no outliers are removed
    ///   (every z-score is 0 and `0 > threshold` is false for any `threshold >= 0`).
    /// - `threshold == 0`: removes all values where `|z-score| > 0`,
    ///   i.e., everything except the mean (if std_dev > 0).
    fn remove_outliers_zscore(self, threshold: f64) -> Vec<f64>;

    /// Removes outliers using the Interquartile Range (IQR) method.
    ///
    /// A value is an outlier if it falls below `Q1 - 1.5 * IQR` or
    /// above `Q3 + 1.5 * IQR`.
    ///
    /// Quartiles are computed using the "inclusive" method:
    /// - Q1 = median of the lower half (excluding the overall median for odd-length sets).
    /// - Q3 = median of the upper half (excluding the overall median for odd-length sets).
    ///
    /// # Edge cases
    ///
    /// - Empty input: returns empty `Vec`.
    /// - Fewer than 4 elements: returns input unchanged (IQR cannot be reliably computed).
    fn remove_outliers_iqr(self) -> Vec<f64>;
}

impl<State: 'static> OutlierExt for QueryBuilder<f64, State> {
    fn remove_outliers_zscore(self, threshold: f64) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            return values;
        }
        values
            .into_iter()
            .filter(|&x| ((x - mean) / std_dev).abs() <= threshold)
            .collect()
    }

    fn remove_outliers_iqr(self) -> Vec<f64> {
        let mut values: Vec<f64> = self.collect();
        if values.len() < 4 {
            return values;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = values.len();
        let (lower_half, upper_half) = if n.is_multiple_of(2) {
            (&values[..n / 2], &values[n / 2..])
        } else {
            (&values[..n / 2], &values[n / 2 + 1..])
        };
        let q1 = median_of(lower_half);
        let q3 = median_of(upper_half);
        let iqr = q3 - q1;
        let lo = q1 - 1.5 * iqr;
        let hi = q3 + 1.5 * iqr;
        values.into_iter().filter(|&x| x >= lo && x <= hi).collect()
    }
}

fn median_of(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}
