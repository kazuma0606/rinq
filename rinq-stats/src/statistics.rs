// rinq-stats/src/statistics.rs
// StatisticsExt trait — single-source statistical operations on QueryBuilder.

use crate::types::HistogramBucket;
use rinq::QueryBuilder;
use std::collections::HashMap;
use std::hash::Hash;

/// Statistical operations that can be applied to any `QueryBuilder`.
///
/// Import this trait to add statistical methods to `QueryBuilder`:
///
/// ```
/// use rinq::QueryBuilder;
/// use rinq_stats::StatisticsExt;
///
/// let variance = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
///     .variance();
/// assert!((variance - 4.0).abs() < 1e-10);
/// ```
pub trait StatisticsExt<T> {
    /// Population variance: `(1/n) * Σ (x_i - μ)²`.
    ///
    /// Returns `0.0` for an empty or single-element collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// // Classic example: mean=5, variance=4
    /// let v = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
    ///     .variance();
    /// assert!((v - 4.0).abs() < 1e-10);
    /// ```
    fn variance(self) -> f64;

    /// Population standard deviation: `√variance`.
    ///
    /// Returns `0.0` for an empty or single-element collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// let sd = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
    ///     .std_dev();
    /// assert!((sd - 2.0).abs() < 1e-10);
    /// ```
    fn std_dev(self) -> f64;

    /// Median (middle value after sorting).
    ///
    /// Returns `None` for an empty collection.
    /// For even-length collections, returns the mean of the two middle values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// assert_eq!(
    ///     QueryBuilder::from(vec![3.0_f64, 1.0, 4.0, 1.0, 5.0]).median(),
    ///     Some(3.0)
    /// );
    /// assert_eq!(
    ///     QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0]).median(),
    ///     Some(2.5)
    /// );
    /// ```
    fn median(self) -> Option<f64>;

    /// Value that appears most frequently.
    ///
    /// Returns `None` for an empty collection.
    /// When multiple values tie, one of them is returned (unspecified which).
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// let m = QueryBuilder::from(vec![1, 2, 2, 3, 3, 3]).mode();
    /// assert_eq!(m, Some(3));
    /// ```
    fn mode(self) -> Option<T>
    where
        T: Eq + Hash + Clone;

    /// p-th percentile (0.0 – 1.0), using the "nearest rank" method.
    ///
    /// Returns `None` for an empty collection. `p` is clamped to `[0.0, 1.0]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// let p95 = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0])
    ///     .percentile(0.95);
    /// assert!(p95.is_some());
    /// ```
    fn percentile(self, p: f64) -> Option<f64>;

    /// Alias for [`percentile`] — the p-th quantile (0.0 – 1.0).
    ///
    /// [`percentile`]: StatisticsExt::percentile
    fn quantile(self, p: f64) -> Option<f64>;

    /// Fisher–Pearson standardised moment coefficient (third moment / σ³).
    ///
    /// Returns `0.0` for fewer than 3 elements or zero standard deviation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// // Symmetric data has ~0 skewness
    /// let sk = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0]).skewness();
    /// assert!(sk.abs() < 1e-10);
    /// ```
    fn skewness(self) -> f64;

    /// Excess kurtosis (fourth moment / σ⁴ − 3).
    ///
    /// Returns `0.0` for fewer than 4 elements or zero standard deviation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// let k = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0]).kurtosis();
    /// // Uniform-like data has negative excess kurtosis
    /// assert!(k < 0.0);
    /// ```
    fn kurtosis(self) -> f64;

    /// Divide the value range into `buckets` equal-width intervals and count elements.
    ///
    /// Returns an empty `Vec` if the collection is empty or `buckets == 0`.
    /// If all values are equal, every element falls into bucket 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// let hist = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0])
    ///     .histogram(2);
    /// assert_eq!(hist.len(), 2);
    /// let total: usize = hist.iter().map(|b| b.count).sum();
    /// assert_eq!(total, 5);
    /// ```
    fn histogram(self, buckets: usize) -> Vec<HistogramBucket>;

    /// Count the frequency of each distinct value.
    ///
    /// Returns a `HashMap<T, usize>` mapping each value to its occurrence count.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::StatisticsExt;
    ///
    /// let freq = QueryBuilder::from(vec![1, 2, 2, 3, 3, 3]).frequency_table();
    /// assert_eq!(freq[&1], 1);
    /// assert_eq!(freq[&2], 2);
    /// assert_eq!(freq[&3], 3);
    /// ```
    fn frequency_table(self) -> HashMap<T, usize>
    where
        T: Eq + Hash;
}

// ── blanket impl for QueryBuilder<T, State> ──────────────────────────────────

impl<T, State> StatisticsExt<T> for QueryBuilder<T, State>
where
    T: Into<f64> + Clone + 'static,
    State: 'static,
{
    fn variance(self) -> f64 {
        let values: Vec<f64> = self.collect::<Vec<T>>().into_iter().map(|x| x.into()).collect();
        population_variance(&values)
    }

    fn std_dev(self) -> f64 {
        self.variance().sqrt()
    }

    fn median(self) -> Option<f64> {
        let mut values: Vec<f64> = self.collect::<Vec<T>>().into_iter().map(|x| x.into()).collect();
        if values.is_empty() {
            return None;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = values.len();
        if n % 2 == 1 {
            Some(values[n / 2])
        } else {
            Some((values[n / 2 - 1] + values[n / 2]) / 2.0)
        }
    }

    fn mode(self) -> Option<T>
    where
        T: Eq + Hash + Clone,
    {
        let items: Vec<T> = self.collect();
        if items.is_empty() {
            return None;
        }
        let mut counts: HashMap<&T, usize> = HashMap::new();
        for x in &items {
            *counts.entry(x).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(k, _)| k.clone())
    }

    fn percentile(self, p: f64) -> Option<f64> {
        let mut values: Vec<f64> = self.collect::<Vec<T>>().into_iter().map(|x| x.into()).collect();
        if values.is_empty() {
            return None;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p = p.clamp(0.0, 1.0);
        let idx = ((p * (values.len() - 1) as f64).round() as usize).min(values.len() - 1);
        Some(values[idx])
    }

    fn quantile(self, p: f64) -> Option<f64> {
        self.percentile(p)
    }

    fn skewness(self) -> f64 {
        let values: Vec<f64> = self.collect::<Vec<T>>().into_iter().map(|x| x.into()).collect();
        let n = values.len();
        if n < 3 {
            return 0.0;
        }
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = population_variance(&values);
        let sd = variance.sqrt();
        if sd == 0.0 {
            return 0.0;
        }
        let third_moment = values.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / n as f64;
        third_moment / sd.powi(3)
    }

    fn kurtosis(self) -> f64 {
        let values: Vec<f64> = self.collect::<Vec<T>>().into_iter().map(|x| x.into()).collect();
        let n = values.len();
        if n < 4 {
            return 0.0;
        }
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = population_variance(&values);
        let sd = variance.sqrt();
        if sd == 0.0 {
            return 0.0;
        }
        let fourth_moment = values.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n as f64;
        fourth_moment / sd.powi(4) - 3.0
    }

    fn histogram(self, buckets: usize) -> Vec<HistogramBucket> {
        if buckets == 0 {
            return vec![];
        }
        let values: Vec<f64> = self.collect::<Vec<T>>().into_iter().map(|x| x.into()).collect();
        if values.is_empty() {
            return vec![];
        }
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let width = if (max - min).abs() < f64::EPSILON {
            1.0
        } else {
            (max - min) / buckets as f64
        };

        let mut counts = vec![0usize; buckets];
        for &v in &values {
            let idx = if (v - min).abs() < f64::EPSILON && (max - min).abs() < f64::EPSILON {
                0
            } else {
                let i = ((v - min) / width) as usize;
                i.min(buckets - 1)
            };
            counts[idx] += 1;
        }

        (0..buckets)
            .map(|i| {
                let lo = min + i as f64 * width;
                let hi = min + (i + 1) as f64 * width;
                HistogramBucket {
                    range: lo..=hi,
                    count: counts[i],
                }
            })
            .collect()
    }

    fn frequency_table(self) -> HashMap<T, usize>
    where
        T: Eq + Hash,
    {
        let mut map: HashMap<T, usize> = HashMap::new();
        for x in self.collect::<Vec<T>>() {
            *map.entry(x).or_insert(0) += 1;
        }
        map
    }
}

// ── internal helpers ──────────────────────────────────────────────────────────

pub(crate) fn population_variance(values: &[f64]) -> f64 {
    let n = values.len();
    if n <= 1 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64
}
