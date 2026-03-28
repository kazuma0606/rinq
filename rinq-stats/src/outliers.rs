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

    /// Remove outliers using the **Modified Z-Score** (MAD-based) method.
    ///
    /// Modified z-score = `0.6745 * (x - median) / MAD`
    ///
    /// where MAD is the Median Absolute Deviation.
    /// A value is an outlier if `|modified z-score| > threshold` (commonly `3.5`).
    ///
    /// More robust than the standard z-score when data are non-normal.
    ///
    /// # Edge cases
    ///
    /// - Empty input: returns empty `Vec`.
    /// - `MAD == 0`: no outliers are removed (all values are identical or near-identical).
    fn remove_outliers_modified_zscore(self, threshold: f64) -> Vec<f64>;

    /// Compute an IQR-based outlier score for every element (without removing any).
    ///
    /// Score = `0.0` for values within `[Q1 - 1.5 * IQR, Q3 + 1.5 * IQR]`.
    /// Score > `0.0` for values outside that range (proportional to distance from the fence).
    ///
    /// Returns `vec![0.0; n]` if fewer than 4 elements (IQR undefined).
    /// Returns an empty `Vec` for an empty input.
    fn outlier_scores_iqr(self) -> Vec<f64>;
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

    fn remove_outliers_modified_zscore(self, threshold: f64) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let med = median_of(&sorted);
        let deviations: Vec<f64> = values.iter().map(|&x| (x - med).abs()).collect();
        let mut sorted_dev = deviations.clone();
        sorted_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = median_of(&sorted_dev);
        if mad == 0.0 {
            return values;
        }
        values
            .into_iter()
            .zip(deviations)
            .filter(|(_, dev)| (0.6745 * dev / mad) <= threshold)
            .map(|(v, _)| v)
            .collect()
    }

    fn outlier_scores_iqr(self) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        if values.len() < 4 {
            return vec![0.0; values.len()];
        }
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len();
        let (lower_half, upper_half) = if n.is_multiple_of(2) {
            (&sorted[..n / 2], &sorted[n / 2..])
        } else {
            (&sorted[..n / 2], &sorted[n / 2 + 1..])
        };
        let q1 = median_of(lower_half);
        let q3 = median_of(upper_half);
        let iqr = q3 - q1;
        let lo = q1 - 1.5 * iqr;
        let hi = q3 + 1.5 * iqr;
        values
            .into_iter()
            .map(|x| {
                if x < lo {
                    lo - x
                } else if x > hi {
                    x - hi
                } else {
                    0.0
                }
            })
            .collect()
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
