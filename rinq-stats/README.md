# rinq-stats

[![CI](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml/badge.svg)](https://github.com/kazuma0606/rinq/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/rinq-stats.svg)](https://crates.io/crates/rinq-stats)
[![docs.rs](https://docs.rs/rinq-stats/badge.svg)](https://docs.rs/rinq-stats)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Statistical extensions for the [rinq](https://crates.io/crates/rinq) query engine.**

Adds descriptive statistics, sampling, validation, time series analysis, and outlier detection directly to `QueryBuilder` pipelines.

## Installation

```toml
[dependencies]
rinq       = "0.1"
rinq-stats = "0.1"
```

## Extensions at a Glance

### StatisticsExt

Descriptive statistics on `QueryBuilder<f64, _>`:

```rust
use rinq::QueryBuilder;
use rinq_stats::StatisticsExt;

let data = QueryBuilder::from(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
println!("mean:    {}", data.clone().where_(|_| true).mean().unwrap());
println!("std_dev: {}", data.clone().where_(|_| true).std_dev().unwrap());
println!("median:  {}", data.clone().where_(|_| true).median().unwrap());
```

| Method | Description |
|---|---|
| `mean()` | Arithmetic mean |
| `variance()` | Population variance |
| `std_dev()` | Standard deviation |
| `median()` | Median value |
| `percentile(p)` | p-th percentile (0–100) |
| `skewness()` | Pearson's skewness |
| `kurtosis()` | Excess kurtosis |
| `histogram(buckets)` | Frequency histogram |
| `correlation_with(other)` | Pearson correlation coefficient |
| `linear_regression()` | Returns `(slope, intercept)` |

### SamplingExt

Random sampling on `QueryBuilder<T, _>`:

```rust
use rinq_stats::SamplingExt;

let sample = QueryBuilder::from(data)
    .where_(|_| true)
    .sample_n(100);
```

| Method | Description |
|---|---|
| `sample_n(n)` | Random sample of n elements |
| `sample_fraction(f)` | Random sample of fraction f (0.0–1.0) |
| `stratified_sample(key, n)` | Stratified sampling by key |

### ValidationExt

Rule-based validation on `QueryBuilder<T, _>`:

```rust
use rinq_stats::ValidationExt;

let (valid, errors) = QueryBuilder::from(users)
    .where_(|_| true)
    .validate(|u| u.age >= 0, "non_negative_age", "Age must be non-negative")
    .validate_if(|u| u.is_employee, |u| u.salary > 0, "positive_salary", "Salary must be positive")
    .validate_with(|u| u.name.is_empty().then(|| format!("{}: name is empty", u.id)), "non_empty_name")
    .collect();
```

| Method | Description |
|---|---|
| `validate(pred, rule, msg)` | Add a validation rule with a fixed message |
| `validate_if(cond, pred, rule, msg)` | Conditional validation rule |
| `validate_with(f, rule)` | Dynamic error message factory |

### TimeSeriesExt

Time series analysis on `QueryBuilder<f64, _>`:

```rust
use rinq_stats::TimeSeriesExt;

let ema = QueryBuilder::from(prices)
    .where_(|_| true)
    .exponential_moving_average(0.2)
    .collect_vec();

let bands = QueryBuilder::from(prices)
    .where_(|_| true)
    .bollinger_bands(20, 2.0)
    .collect_vec();
```

| Method | Description |
|---|---|
| `exponential_moving_average(alpha)` | EMA with smoothing factor α ∈ (0, 1] |
| `bollinger_bands(window, sigma)` | Returns `Vec<BollingerPoint>` with middle/upper/lower bands |

### OutlierExt

Outlier removal on `QueryBuilder<f64, _>`:

```rust
use rinq_stats::OutlierExt;

let clean = QueryBuilder::from(data)
    .where_(|_| true)
    .remove_outliers_zscore(2.5)
    .collect_vec();
```

| Method | Description |
|---|---|
| `remove_outliers_zscore(threshold)` | Remove elements with \|z-score\| > threshold |
| `remove_outliers_iqr()` | Remove elements outside [Q1 − 1.5·IQR, Q3 + 1.5·IQR] |

## License

MIT — see [LICENSE](../LICENSE)
