// rinq-stats/src/lib.rs

pub mod statistics;
pub mod types;
pub mod pair;
pub mod sampling;
pub mod validation;
pub mod timeseries;
pub mod outliers;

pub use statistics::StatisticsExt;
pub use types::HistogramBucket;
pub use pair::{QueryPair, QueryPairError};
pub use sampling::SamplingExt;
pub use validation::{ValidationExt, ValidationError, ValidationQueryBuilder};
pub use timeseries::{TimeSeriesExt, BollingerPoint};
pub use outliers::OutlierExt;
