// rinq-stats/src/pair.rs
// QueryPair<X, Y> — two-series statistical analysis.

use crate::statistics::population_variance;
use rinq::QueryBuilder;

/// Error type for `QueryPair::try_new`.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPairError {
    pub message: String,
}

impl std::fmt::Display for QueryPairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryPairError: {}", self.message)
    }
}

impl std::error::Error for QueryPairError {}

/// A pair of equal-length data series for two-variable statistical analysis.
///
/// # Construction
///
/// ```
/// use rinq_stats::QueryPair;
///
/// // Length mismatch → truncate to shorter series (logs a warning)
/// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
/// assert_eq!(pair.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct QueryPair {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl QueryPair {
    /// Construct a `QueryPair`, truncating to the shorter series if lengths differ.
    ///
    /// When lengths differ, a `log::warn!` is emitted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// // Equal lengths — no truncation
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
    /// assert_eq!(pair.len(), 3);
    ///
    /// // Unequal lengths — truncated to 3
    /// let pair = QueryPair::new(
    ///     vec![1.0, 2.0, 3.0, 4.0, 5.0],
    ///     vec![10.0, 20.0, 30.0],
    /// );
    /// assert_eq!(pair.len(), 3);
    /// ```
    pub fn new(x: impl Into<Vec<f64>>, y: impl Into<Vec<f64>>) -> Self {
        let mut x = x.into();
        let mut y = y.into();
        if x.len() != y.len() {
            log::warn!(
                "QueryPair: length mismatch ({} vs {}), truncated to {}",
                x.len(),
                y.len(),
                x.len().min(y.len())
            );
            let n = x.len().min(y.len());
            x.truncate(n);
            y.truncate(n);
        }
        Self { x, y }
    }

    /// Construct a `QueryPair`, returning `Err` if the series have different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// assert!(QueryPair::try_new(vec![1.0, 2.0], vec![3.0, 4.0]).is_ok());
    /// assert!(QueryPair::try_new(vec![1.0, 2.0, 3.0], vec![4.0]).is_err());
    /// ```
    pub fn try_new(
        x: impl Into<Vec<f64>>,
        y: impl Into<Vec<f64>>,
    ) -> Result<Self, QueryPairError> {
        let x = x.into();
        let y = y.into();
        if x.len() != y.len() {
            return Err(QueryPairError {
                message: format!("length mismatch: {} vs {}", x.len(), y.len()),
            });
        }
        Ok(Self { x, y })
    }

    /// Construct a `QueryPair` from two `QueryBuilder` instances.
    ///
    /// Both builders are immediately consumed and materialised. Length
    /// mismatch is handled the same way as [`QueryPair::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::QueryPair;
    ///
    /// let pair = QueryPair::from_builders(
    ///     QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]),
    ///     QueryBuilder::from(vec![4.0_f64, 5.0, 6.0]),
    /// );
    /// assert_eq!(pair.len(), 3);
    /// ```
    pub fn from_builders<S1, S2>(
        x: QueryBuilder<f64, S1>,
        y: QueryBuilder<f64, S2>,
    ) -> Self
    where
        S1: 'static,
        S2: 'static,
    {
        let xv: Vec<f64> = x.collect();
        let yv: Vec<f64> = y.collect();
        Self::new(xv, yv)
    }

    /// Number of paired observations.
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Returns `true` if the pair has no observations.
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Population covariance: `(1/n) * Σ (x_i − x̄)(y_i − ȳ)`.
    ///
    /// Returns `0.0` for fewer than 2 observations.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
    /// let cov = pair.covariance();
    /// assert!((cov - 0.6666666666666666).abs() < 1e-10);
    /// ```
    pub fn covariance(&self) -> f64 {
        let n = self.x.len();
        if n < 2 {
            return 0.0;
        }
        let mx = mean(&self.x);
        let my = mean(&self.y);
        self.x
            .iter()
            .zip(self.y.iter())
            .map(|(xi, yi)| (xi - mx) * (yi - my))
            .sum::<f64>()
            / n as f64
    }

    /// Pearson product-moment correlation coefficient.
    ///
    /// Returns `0.0` when either series has zero variance or fewer than 2 observations.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// // Perfect positive correlation
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0]);
    /// assert!((pair.pearson_correlation() - 1.0).abs() < 1e-10);
    ///
    /// // Perfect negative correlation
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![6.0, 4.0, 2.0]);
    /// assert!((pair.pearson_correlation() + 1.0).abs() < 1e-10);
    /// ```
    pub fn pearson_correlation(&self) -> f64 {
        let sx = population_variance(&self.x).sqrt();
        let sy = population_variance(&self.y).sqrt();
        if sx == 0.0 || sy == 0.0 {
            return 0.0;
        }
        self.covariance() / (sx * sy)
    }

    /// Spearman rank-order correlation coefficient.
    ///
    /// Converts each series to ranks and then computes Pearson on the ranks.
    /// Returns `0.0` for fewer than 2 observations.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// // Perfectly monotone relationship
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]);
    /// assert!((pair.spearman_correlation() - 1.0).abs() < 1e-10);
    /// ```
    pub fn spearman_correlation(&self) -> f64 {
        if self.x.len() < 2 {
            return 0.0;
        }
        let rx = ranks(&self.x);
        let ry = ranks(&self.y);
        let ranked = QueryPair::new(rx, ry);
        ranked.pearson_correlation()
    }

    /// Kendall's τ-b (concordance-based rank correlation).
    ///
    /// O(n²) implementation. Returns `0.0` for fewer than 2 observations.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// // Perfect order agreement
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]);
    /// assert!((pair.kendall_tau() - 1.0).abs() < 1e-10);
    /// ```
    pub fn kendall_tau(&self) -> f64 {
        let n = self.x.len();
        if n < 2 {
            return 0.0;
        }
        let mut concordant = 0i64;
        let mut discordant = 0i64;
        let mut ties_x = 0i64;
        let mut ties_y = 0i64;
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.x[j] - self.x[i];
                let dy = self.y[j] - self.y[i];
                match (dx.partial_cmp(&0.0), dy.partial_cmp(&0.0)) {
                    (Some(sx), Some(sy)) if sx == sy && sx != std::cmp::Ordering::Equal => {
                        concordant += 1
                    }
                    (Some(sx), Some(sy))
                        if sx != std::cmp::Ordering::Equal
                            && sy != std::cmp::Ordering::Equal
                            && sx != sy =>
                    {
                        discordant += 1
                    }
                    _ => {
                        if dx.abs() < f64::EPSILON {
                            ties_x += 1;
                        }
                        if dy.abs() < f64::EPSILON {
                            ties_y += 1;
                        }
                    }
                }
            }
        }
        let n0 = (n as i64 * (n as i64 - 1)) / 2;
        let denom = ((n0 - ties_x) as f64 * (n0 - ties_y) as f64).sqrt();
        if denom == 0.0 {
            return 0.0;
        }
        (concordant - discordant) as f64 / denom
    }

    /// Ordinary least-squares linear regression: `y = slope * x + intercept`.
    ///
    /// Returns `(slope, intercept)`. Returns `(0.0, 0.0)` for fewer than 2 observations
    /// or when `x` has zero variance.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq_stats::QueryPair;
    ///
    /// // y = 2x + 1
    /// let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![3.0, 5.0, 7.0]);
    /// let (slope, intercept) = pair.linear_regression();
    /// assert!((slope - 2.0).abs() < 1e-10);
    /// assert!((intercept - 1.0).abs() < 1e-10);
    /// ```
    pub fn linear_regression(&self) -> (f64, f64) {
        let n = self.x.len();
        if n < 2 {
            return (0.0, 0.0);
        }
        let mx = mean(&self.x);
        let my = mean(&self.y);
        let var_x = population_variance(&self.x);
        if var_x == 0.0 {
            return (0.0, 0.0);
        }
        let cov = self.covariance();
        let slope = cov / var_x;
        let intercept = my - slope * mx;
        (slope, intercept)
    }
}

// ── internal helpers ──────────────────────────────────────────────────────────

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

/// Convert a slice of values to their (average) ranks (1-based).
fn ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().cloned().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut result = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < f64::EPSILON {
            j += 1;
        }
        // Average rank for tied group (1-based)
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            result[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    result
}
