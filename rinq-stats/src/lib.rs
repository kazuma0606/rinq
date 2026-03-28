// rinq-stats/src/lib.rs

pub mod statistics;
pub mod types;
pub mod pair;
pub mod sampling;
pub mod validation;
pub mod timeseries;
pub mod outliers;
pub mod transform;

pub use statistics::StatisticsExt;
pub use types::HistogramBucket;
pub use pair::{QueryPair, QueryPairError};
pub use sampling::SamplingExt;
pub use validation::{ValidationExt, ValidationError, ValidationQueryBuilder};
pub use timeseries::{TimeSeriesExt, BollingerPoint, SeasonalDecomposition};
pub use outliers::OutlierExt;
pub use transform::NormalizeExt;
