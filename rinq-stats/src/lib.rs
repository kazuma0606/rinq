// rinq-stats/src/lib.rs

pub mod outliers;
pub mod pair;
pub mod sampling;
pub mod statistics;
pub mod timeseries;
pub mod transform;
pub mod types;
pub mod validation;

pub use outliers::OutlierExt;
pub use pair::{QueryPair, QueryPairError};
pub use sampling::SamplingExt;
pub use statistics::StatisticsExt;
pub use timeseries::{BollingerPoint, SeasonalDecomposition, TimeSeriesExt};
pub use transform::NormalizeExt;
pub use types::HistogramBucket;
pub use validation::{ValidationError, ValidationExt, ValidationQueryBuilder};
