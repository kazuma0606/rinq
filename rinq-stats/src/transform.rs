// rinq-stats/src/transform.rs
// Phase 5G-1: NormalizeExt — normalization, standardization, and derived statistics.

use rinq::QueryBuilder;

/// Normalization and transformation operations for sequences of `f64` values.
pub trait NormalizeExt {
    /// Min-max normalization: scale all values to the range `[0.0, 1.0]`.
    ///
    /// If all values are equal (range = 0), every element is mapped to `0.0`.
    /// Returns an empty `Vec` for an empty input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::NormalizeExt;
    ///
    /// let v = QueryBuilder::from(vec![0.0_f64, 5.0, 10.0]).normalize();
    /// assert!((v[0] - 0.0).abs() < 1e-10);
    /// assert!((v[1] - 0.5).abs() < 1e-10);
    /// assert!((v[2] - 1.0).abs() < 1e-10);
    /// ```
    fn normalize(self) -> Vec<f64>;

    /// Z-score standardization: `(x - mean) / std_dev`.
    ///
    /// If `std_dev == 0` (all identical values), every element is mapped to `0.0`.
    /// Returns an empty `Vec` for an empty input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::NormalizeExt;
    ///
    /// let v = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
    ///     .standardize();
    /// // mean = 5, std_dev = 2 → value 7 has z-score 1.0
    /// assert!((v[6] - 1.0).abs() < 1e-10);
    /// ```
    fn standardize(self) -> Vec<f64>;

    /// Weighted average using per-element weights returned by `weight_fn`.
    ///
    /// Returns `None` for an empty input or when the total weight is `0.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::NormalizeExt;
    ///
    /// // Weights: 1, 2, 3 → weighted avg = (1*1 + 2*2 + 3*3) / (1+2+3) = 14/6
    /// let avg = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0])
    ///     .weighted_average(|x| *x);
    /// assert!((avg.unwrap() - 14.0 / 6.0).abs() < 1e-10);
    /// ```
    fn weighted_average<F>(self, weight_fn: F) -> Option<f64>
    where
        F: Fn(&f64) -> f64;

    /// Compute a z-score for every element (without removing any).
    ///
    /// Returns `vec![0.0; n]` when `std_dev == 0`.
    /// Returns an empty `Vec` for an empty input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::NormalizeExt;
    ///
    /// let scores = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
    ///     .outlier_scores_zscore();
    /// // value 9: z = (9 - 5) / 2 = 2.0
    /// assert!((scores[7] - 2.0).abs() < 1e-10);
    /// ```
    fn outlier_scores_zscore(self) -> Vec<f64>;

    /// Retain only elements whose value falls within the `[lo_pct, hi_pct]` percentile range.
    ///
    /// `lo_pct` and `hi_pct` must be in `[0.0, 1.0]`. Values are sorted internally.
    /// Returns an empty `Vec` for an empty input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::NormalizeExt;
    ///
    /// let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    /// let filtered = QueryBuilder::from(data).percentile_filter(0.1, 0.9);
    /// // values in the 10th–90th percentile range
    /// assert!(filtered.iter().all(|&x| x >= 10.0 && x <= 90.0));
    /// ```
    fn percentile_filter(self, lo_pct: f64, hi_pct: f64) -> Vec<f64>;

    /// Empirical Cumulative Distribution Function (eCDF) values.
    ///
    /// Returns a `Vec<f64>` of the same length as the input where each element
    /// is the proportion of values ≤ that element (after sorting).
    ///
    /// Returns an empty `Vec` for an empty input.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::NormalizeExt;
    ///
    /// let cdf = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0]).cumulative_distribution();
    /// assert!((cdf[0] - 0.25).abs() < 1e-10);
    /// assert!((cdf[3] - 1.00).abs() < 1e-10);
    /// ```
    fn cumulative_distribution(self) -> Vec<f64>;
}

impl<State: 'static> NormalizeExt for QueryBuilder<f64, State> {
    fn normalize(self) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;
        if range == 0.0 {
            return vec![0.0; values.len()];
        }
        values.into_iter().map(|x| (x - min) / range).collect()
    }

    fn standardize(self) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            return vec![0.0; values.len()];
        }
        values.into_iter().map(|x| (x - mean) / std_dev).collect()
    }

    fn weighted_average<F>(self, weight_fn: F) -> Option<f64>
    where
        F: Fn(&f64) -> f64,
    {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return None;
        }
        let mut weighted_sum = 0.0_f64;
        let mut total_weight = 0.0_f64;
        for v in &values {
            let w = weight_fn(v);
            weighted_sum += v * w;
            total_weight += w;
        }
        if total_weight == 0.0 {
            None
        } else {
            Some(weighted_sum / total_weight)
        }
    }

    fn outlier_scores_zscore(self) -> Vec<f64> {
        let values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            return vec![0.0; values.len()];
        }
        values
            .into_iter()
            .map(|x| ((x - mean) / std_dev).abs())
            .collect()
    }

    fn percentile_filter(self, lo_pct: f64, hi_pct: f64) -> Vec<f64> {
        let mut values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = values.len();
        let lo_idx = (lo_pct.clamp(0.0, 1.0) * n as f64).floor() as usize;
        let hi_idx = (hi_pct.clamp(0.0, 1.0) * n as f64).ceil() as usize;
        let lo_val = values[lo_idx.min(n - 1)];
        let hi_val = values[(hi_idx.saturating_sub(1)).min(n - 1)];
        values
            .into_iter()
            .filter(|&x| x >= lo_val && x <= hi_val)
            .collect()
    }

    fn cumulative_distribution(self) -> Vec<f64> {
        let mut values: Vec<f64> = self.collect();
        if values.is_empty() {
            return Vec::new();
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = values.len() as f64;
        values
            .iter()
            .enumerate()
            .map(|(i, _)| (i + 1) as f64 / n)
            .collect()
    }
}
