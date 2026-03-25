// rinq-stats/src/types.rs
// Shared data types used across the rinq-stats crate.

use std::ops::RangeInclusive;

/// A single bucket in a histogram, with the value range and count of elements.
///
/// Produced by [`StatisticsExt::histogram`].
///
/// [`StatisticsExt::histogram`]: crate::StatisticsExt::histogram
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBucket {
    /// The half-open interval `[low, high]` covered by this bucket.
    pub range: RangeInclusive<f64>,
    /// Number of elements whose value falls within `range`.
    pub count: usize,
}
