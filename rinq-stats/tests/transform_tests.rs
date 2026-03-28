// rinq-stats/tests/transform_tests.rs
// Phase 5G: NormalizeExt tests.

use rinq::QueryBuilder;
use rinq_stats::NormalizeExt;

// ── normalize ──────────────────────────────────────────────────────────────

#[test]
fn normalize_maps_min_to_zero_max_to_one() {
    let v = QueryBuilder::from(vec![0.0_f64, 5.0, 10.0]).normalize();
    assert!((v[0] - 0.0).abs() < 1e-10);
    assert!((v[1] - 0.5).abs() < 1e-10);
    assert!((v[2] - 1.0).abs() < 1e-10);
}

#[test]
fn normalize_empty_returns_empty() {
    let v = QueryBuilder::from(vec![]).normalize();
    assert!(v.is_empty());
}

#[test]
fn normalize_all_identical_returns_zeros() {
    let v = QueryBuilder::from(vec![3.0_f64, 3.0, 3.0]).normalize();
    assert!(v.iter().all(|&x| x == 0.0));
}

#[test]
fn normalize_negative_values() {
    let v = QueryBuilder::from(vec![-10.0_f64, 0.0, 10.0]).normalize();
    assert!((v[0] - 0.0).abs() < 1e-10);
    assert!((v[1] - 0.5).abs() < 1e-10);
    assert!((v[2] - 1.0).abs() < 1e-10);
}

// ── standardize ───────────────────────────────────────────────────────────

#[test]
fn standardize_mean_zero_std_one() {
    let v = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
        .standardize();
    let mean: f64 = v.iter().sum::<f64>() / v.len() as f64;
    assert!(mean.abs() < 1e-10);
    // value 9: z = (9 - 5) / 2 = 2.0
    assert!((v[7] - 2.0).abs() < 1e-10);
}

#[test]
fn standardize_empty_returns_empty() {
    let v: Vec<f64> = QueryBuilder::from(vec![]).standardize();
    assert!(v.is_empty());
}

#[test]
fn standardize_all_identical_returns_zeros() {
    let v = QueryBuilder::from(vec![5.0_f64, 5.0, 5.0]).standardize();
    assert!(v.iter().all(|&x| x == 0.0));
}

// ── weighted_average ──────────────────────────────────────────────────────

#[test]
fn weighted_average_basic() {
    // weights = values themselves: (1*1 + 2*2 + 3*3) / 6 = 14/6
    let avg = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0])
        .weighted_average(|x| *x);
    assert!((avg.unwrap() - 14.0 / 6.0).abs() < 1e-10);
}

#[test]
fn weighted_average_empty_returns_none() {
    let avg = QueryBuilder::from(vec![]).weighted_average(|x| *x);
    assert!(avg.is_none());
}

#[test]
fn weighted_average_zero_weight_returns_none() {
    let avg = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0])
        .weighted_average(|_| 0.0);
    assert!(avg.is_none());
}

#[test]
fn weighted_average_uniform_weights_equals_mean() {
    let avg = QueryBuilder::from(vec![2.0_f64, 4.0, 6.0])
        .weighted_average(|_| 1.0);
    assert!((avg.unwrap() - 4.0).abs() < 1e-10);
}

// ── outlier_scores_zscore ─────────────────────────────────────────────────

#[test]
fn outlier_scores_zscore_value_at_mean_is_zero() {
    let scores = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]).outlier_scores_zscore();
    // mean = 2, value 2 → z = 0
    assert!((scores[1] - 0.0).abs() < 1e-10);
}

#[test]
fn outlier_scores_zscore_empty_returns_empty() {
    let scores: Vec<f64> = QueryBuilder::from(vec![]).outlier_scores_zscore();
    assert!(scores.is_empty());
}

#[test]
fn outlier_scores_zscore_all_identical_returns_zeros() {
    let scores = QueryBuilder::from(vec![5.0_f64, 5.0, 5.0]).outlier_scores_zscore();
    assert!(scores.iter().all(|&x| x == 0.0));
}

#[test]
fn outlier_scores_zscore_extreme_value() {
    // value 9: mean=5, std_dev=2 → z=2.0
    let scores = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
        .outlier_scores_zscore();
    assert!((scores[7] - 2.0).abs() < 1e-10);
}

// ── percentile_filter ─────────────────────────────────────────────────────

#[test]
fn percentile_filter_basic() {
    let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let filtered = QueryBuilder::from(data).percentile_filter(0.1, 0.9);
    assert!(filtered.iter().all(|&x| x >= 10.0 && x <= 90.0));
}

#[test]
fn percentile_filter_empty_returns_empty() {
    let filtered: Vec<f64> = QueryBuilder::from(vec![]).percentile_filter(0.1, 0.9);
    assert!(filtered.is_empty());
}

#[test]
fn percentile_filter_full_range_keeps_all() {
    let data = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let n = data.len();
    let filtered = QueryBuilder::from(data).percentile_filter(0.0, 1.0);
    assert_eq!(filtered.len(), n);
}

// ── cumulative_distribution ───────────────────────────────────────────────

#[test]
fn cumulative_distribution_basic() {
    let cdf = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0]).cumulative_distribution();
    assert!((cdf[0] - 0.25).abs() < 1e-10);
    assert!((cdf[1] - 0.50).abs() < 1e-10);
    assert!((cdf[2] - 0.75).abs() < 1e-10);
    assert!((cdf[3] - 1.00).abs() < 1e-10);
}

#[test]
fn cumulative_distribution_empty_returns_empty() {
    let cdf: Vec<f64> = QueryBuilder::from(vec![]).cumulative_distribution();
    assert!(cdf.is_empty());
}

#[test]
fn cumulative_distribution_single_element() {
    let cdf = QueryBuilder::from(vec![42.0_f64]).cumulative_distribution();
    assert_eq!(cdf.len(), 1);
    assert!((cdf[0] - 1.0).abs() < 1e-10);
}

#[test]
fn cumulative_distribution_last_is_one() {
    let cdf = QueryBuilder::from(vec![5.0_f64, 3.0, 1.0, 4.0, 2.0]).cumulative_distribution();
    assert!((cdf[cdf.len() - 1] - 1.0).abs() < 1e-10);
}
