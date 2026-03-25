// rinq-stats/tests/pair_tests.rs
// Phase B2 integration tests for QueryPair.

use rinq::QueryBuilder;
use rinq_stats::{QueryPair, QueryPairError};

const EPS: f64 = 1e-9;
const EPS_LOOSE: f64 = 1e-6;

// ── construction ──────────────────────────────────────────────────────────────

#[test]
fn new_equal_lengths() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
    assert_eq!(pair.len(), 3);
}

#[test]
fn new_truncates_longer_x() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![10.0, 20.0, 30.0]);
    assert_eq!(pair.len(), 3);
}

#[test]
fn new_truncates_longer_y() {
    let pair = QueryPair::new(vec![1.0, 2.0], vec![10.0, 20.0, 30.0]);
    assert_eq!(pair.len(), 2);
}

#[test]
fn new_empty() {
    let pair = QueryPair::new(vec![], vec![]);
    assert!(pair.is_empty());
}

#[test]
fn try_new_equal_lengths_ok() {
    assert!(QueryPair::try_new(vec![1.0, 2.0], vec![3.0, 4.0]).is_ok());
}

#[test]
fn try_new_mismatch_returns_err() {
    let result = QueryPair::try_new(vec![1.0, 2.0, 3.0], vec![4.0]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("mismatch"));
}

#[test]
fn from_builders() {
    let pair = QueryPair::from_builders(
        QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]),
        QueryBuilder::from(vec![4.0_f64, 5.0, 6.0]),
    );
    assert_eq!(pair.len(), 3);
}

#[test]
fn from_builders_with_filter() {
    let pair = QueryPair::from_builders(
        QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0])
            .where_(|x| *x <= 3.0),
        QueryBuilder::from(vec![10.0_f64, 20.0, 30.0]),
    );
    assert_eq!(pair.len(), 3);
}

// ── covariance ────────────────────────────────────────────────────────────────

#[test]
fn covariance_basic() {
    // [1,2,3] vs [4,5,6]: cov = 2/3
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
    assert!((pair.covariance() - 2.0 / 3.0).abs() < EPS);
}

#[test]
fn covariance_negative() {
    // x increases, y decreases → negative covariance
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![6.0, 4.0, 2.0]);
    assert!(pair.covariance() < 0.0);
}

#[test]
fn covariance_zero_for_independent() {
    // y constant → cov = 0
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![5.0, 5.0, 5.0]);
    assert!((pair.covariance()).abs() < EPS);
}

#[test]
fn covariance_single_observation() {
    let pair = QueryPair::new(vec![1.0], vec![2.0]);
    assert_eq!(pair.covariance(), 0.0);
}

// ── pearson_correlation ───────────────────────────────────────────────────────

#[test]
fn pearson_perfect_positive() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0]);
    assert!((pair.pearson_correlation() - 1.0).abs() < EPS);
}

#[test]
fn pearson_perfect_negative() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![6.0, 4.0, 2.0]);
    assert!((pair.pearson_correlation() + 1.0).abs() < EPS);
}

#[test]
fn pearson_zero_variance_y() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![5.0, 5.0, 5.0]);
    assert_eq!(pair.pearson_correlation(), 0.0);
}

#[test]
fn pearson_near_zero_for_uncorrelated() {
    // [1,2,3,4,5] vs [3,1,4,1,5] — not perfectly uncorrelated but weak
    let pair = QueryPair::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![3.0, 1.0, 4.0, 1.0, 5.0],
    );
    let r = pair.pearson_correlation().abs();
    assert!(r < 1.0); // not perfect
}

#[test]
fn pearson_within_bounds() {
    let pair = QueryPair::new(
        vec![10.0, 50.0, 30.0, 20.0, 40.0],
        vec![100.0, 200.0, 150.0, 80.0, 175.0],
    );
    let r = pair.pearson_correlation();
    assert!(r >= -1.0 && r <= 1.0);
}

// ── spearman_correlation ──────────────────────────────────────────────────────

#[test]
fn spearman_perfect_monotone() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]);
    assert!((pair.spearman_correlation() - 1.0).abs() < EPS_LOOSE);
}

#[test]
fn spearman_perfect_inverse() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![30.0, 20.0, 10.0]);
    assert!((pair.spearman_correlation() + 1.0).abs() < EPS_LOOSE);
}

#[test]
fn spearman_within_bounds() {
    let pair = QueryPair::new(
        vec![1.0, 5.0, 3.0, 2.0, 4.0],
        vec![4.0, 2.0, 5.0, 1.0, 3.0],
    );
    let r = pair.spearman_correlation();
    assert!(r >= -1.0 && r <= 1.0);
}

// ── kendall_tau ───────────────────────────────────────────────────────────────

#[test]
fn kendall_perfect_agreement() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]);
    assert!((pair.kendall_tau() - 1.0).abs() < EPS_LOOSE);
}

#[test]
fn kendall_perfect_disagreement() {
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![30.0, 20.0, 10.0]);
    assert!((pair.kendall_tau() + 1.0).abs() < EPS_LOOSE);
}

#[test]
fn kendall_within_bounds() {
    let pair = QueryPair::new(
        vec![12.0, 2.0, 1.0, 12.0, 2.0],
        vec![1.0, 4.0, 7.0, 1.0, 0.0],
    );
    let tau = pair.kendall_tau();
    assert!(tau >= -1.0 && tau <= 1.0);
}

// ── linear_regression ─────────────────────────────────────────────────────────

#[test]
fn linear_regression_exact() {
    // y = 2x + 1
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![3.0, 5.0, 7.0]);
    let (slope, intercept) = pair.linear_regression();
    assert!((slope - 2.0).abs() < EPS, "slope={slope}");
    assert!((intercept - 1.0).abs() < EPS, "intercept={intercept}");
}

#[test]
fn linear_regression_horizontal() {
    // y = constant → slope = 0
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0, 4.0], vec![5.0, 5.0, 5.0, 5.0]);
    let (slope, _) = pair.linear_regression();
    assert!(slope.abs() < EPS);
}

#[test]
fn linear_regression_single_obs() {
    let pair = QueryPair::new(vec![1.0], vec![2.0]);
    let (slope, intercept) = pair.linear_regression();
    assert_eq!(slope, 0.0);
    assert_eq!(intercept, 0.0);
}

#[test]
fn linear_regression_negative_slope() {
    // y = -3x + 10
    let pair = QueryPair::new(vec![1.0, 2.0, 3.0], vec![7.0, 4.0, 1.0]);
    let (slope, intercept) = pair.linear_regression();
    assert!((slope + 3.0).abs() < EPS_LOOSE, "slope={slope}");
    assert!((intercept - 10.0).abs() < EPS_LOOSE, "intercept={intercept}");
}

// ── integration with rinq operations ─────────────────────────────────────────

#[test]
fn pair_from_filtered_builders() {
    // Build pair from two filtered query builders
    let xs: Vec<f64> = (1..=10).map(|x| x as f64).collect();
    let ys: Vec<f64> = (1..=10).map(|x| (x * x) as f64).collect();

    let pair = QueryPair::from_builders(
        QueryBuilder::from(xs).where_(|x| *x <= 5.0),
        QueryBuilder::from(ys).where_(|y| *y <= 25.0), // 1,4,9,16,25
    );
    assert_eq!(pair.len(), 5);
    let r = pair.pearson_correlation();
    // x=[1..5], y=[1,4,9,16,25]: non-linear but positively correlated
    assert!(r > 0.0);
}
