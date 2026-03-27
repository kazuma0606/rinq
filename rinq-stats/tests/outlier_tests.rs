use rinq::QueryBuilder;
use rinq_stats::OutlierExt;

// ── Z-score ───────────────────────────────────────────────────────────────────

#[test]
fn zscore_empty_returns_empty() {
    let result = QueryBuilder::from(Vec::<f64>::new())
        .remove_outliers_zscore(2.0);
    assert!(result.is_empty());
}

#[test]
fn zscore_no_outliers() {
    let values = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let result = QueryBuilder::from(values.clone())
        .remove_outliers_zscore(3.0);
    assert_eq!(result, values);
}

#[test]
fn zscore_removes_high_outlier() {
    // 100.0 is clearly an outlier among [1..5]
    let values = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0, 100.0];
    let result = QueryBuilder::from(values).remove_outliers_zscore(2.0);
    assert!(!result.contains(&100.0));
    assert_eq!(result.len(), 5);
}

#[test]
fn zscore_all_identical_no_removal() {
    // std_dev == 0, so nothing is removed
    let values = vec![5.0_f64; 10];
    let result = QueryBuilder::from(values.clone()).remove_outliers_zscore(2.0);
    assert_eq!(result, values);
}

#[test]
fn zscore_threshold_zero_removes_non_mean() {
    // threshold=0: only keep values where z-score == 0
    // [1,2,3]: mean=2, std_dev>0; only 2 has z=0
    let result = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0])
        .remove_outliers_zscore(0.0);
    assert_eq!(result, vec![2.0]);
}

// ── IQR ───────────────────────────────────────────────────────────────────────

#[test]
fn iqr_empty_returns_empty() {
    let result = QueryBuilder::from(Vec::<f64>::new()).remove_outliers_iqr();
    assert!(result.is_empty());
}

#[test]
fn iqr_fewer_than_4_unchanged() {
    let values = vec![1.0_f64, 2.0, 3.0];
    let result = QueryBuilder::from(values.clone()).remove_outliers_iqr();
    // Returned sorted (implementation sorts internally)
    assert_eq!(result.len(), 3);
}

#[test]
fn iqr_no_outliers() {
    let values = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let result = QueryBuilder::from(values).remove_outliers_iqr();
    assert_eq!(result.len(), 8);
}

#[test]
fn iqr_removes_high_outlier() {
    let mut values = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 100.0];
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let result = QueryBuilder::from(values).remove_outliers_iqr();
    assert!(!result.contains(&100.0));
}

#[test]
fn iqr_symmetric_distribution() {
    // Symmetric: [1,2,3,4,5,6,7,8,9,10] — no outliers expected
    let values: Vec<f64> = (1..=10).map(|x| x as f64).collect();
    let result = QueryBuilder::from(values).remove_outliers_iqr();
    assert_eq!(result.len(), 10);
}
