// rinq-stats/tests/statistics_tests.rs
// Phase B1 integration tests for StatisticsExt.

use rinq::QueryBuilder;
use rinq_stats::StatisticsExt;

const EPS: f64 = 1e-9;

// ── variance ─────────────────────────────────────────────────────────────────

#[test]
fn variance_classic() {
    // Classic example: mean=5, σ²=4
    let v = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).variance();
    assert!((v - 4.0).abs() < EPS, "expected 4.0, got {v}");
}

#[test]
fn variance_single_element() {
    assert_eq!(QueryBuilder::from(vec![42.0_f64]).variance(), 0.0);
}

#[test]
fn variance_empty() {
    assert_eq!(QueryBuilder::from(vec![0.0_f64; 0]).variance(), 0.0);
}

#[test]
fn variance_identical_elements() {
    assert_eq!(QueryBuilder::from(vec![5.0_f64, 5.0, 5.0]).variance(), 0.0);
}

#[test]
fn variance_two_elements() {
    // [1, 3]: mean=2, σ²=(1+1)/2=1
    let v = QueryBuilder::from(vec![1.0_f64, 3.0]).variance();
    assert!((v - 1.0).abs() < EPS);
}

// ── std_dev ───────────────────────────────────────────────────────────────────

#[test]
fn std_dev_matches_sqrt_variance() {
    let data = vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let expected = QueryBuilder::from(data.clone()).variance().sqrt();
    let actual = QueryBuilder::from(data).std_dev();
    assert!((actual - expected).abs() < EPS);
}

#[test]
fn std_dev_classic() {
    let sd = QueryBuilder::from(vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).std_dev();
    assert!((sd - 2.0).abs() < EPS);
}

#[test]
fn std_dev_empty() {
    assert_eq!(QueryBuilder::from(vec![0.0_f64; 0]).std_dev(), 0.0);
}

// ── median ────────────────────────────────────────────────────────────────────

#[test]
fn median_odd_count() {
    let m = QueryBuilder::from(vec![3.0_f64, 1.0, 4.0, 1.0, 5.0]).median();
    assert_eq!(m, Some(3.0)); // sorted: [1,1,3,4,5] → middle = 3
}

#[test]
fn median_even_count() {
    let m = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0]).median();
    assert_eq!(m, Some(2.5)); // (2+3)/2
}

#[test]
fn median_single() {
    assert_eq!(QueryBuilder::from(vec![7.0_f64]).median(), Some(7.0));
}

#[test]
fn median_empty() {
    assert_eq!(QueryBuilder::from(vec![0.0_f64; 0]).median(), None);
}

#[test]
fn median_already_sorted() {
    assert_eq!(
        QueryBuilder::from(vec![10.0_f64, 20.0, 30.0]).median(),
        Some(20.0)
    );
}

#[test]
fn median_reverse_sorted() {
    assert_eq!(
        QueryBuilder::from(vec![30.0_f64, 20.0, 10.0]).median(),
        Some(20.0)
    );
}

// ── mode ──────────────────────────────────────────────────────────────────────

#[test]
fn mode_single_winner() {
    let m: Option<i32> = QueryBuilder::from(vec![1, 2, 2, 3, 3, 3]).mode();
    assert_eq!(m, Some(3));
}

#[test]
fn mode_all_unique() {
    // Any element may be returned — just check it's one of them
    let m: Option<i32> = QueryBuilder::from(vec![1, 2, 3]).mode();
    assert!(m.is_some());
    assert!([1, 2, 3].contains(&m.unwrap()));
}

#[test]
fn mode_empty() {
    let m: Option<i32> = QueryBuilder::from(vec![]).mode();
    assert_eq!(m, None);
}

#[test]
fn mode_single_element() {
    let m: Option<i32> = QueryBuilder::from(vec![42]).mode();
    assert_eq!(m, Some(42));
}

// ── percentile / quantile ─────────────────────────────────────────────────────

#[test]
fn percentile_50_matches_median() {
    let data = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let p50 = QueryBuilder::from(data.clone()).percentile(0.5);
    let med = QueryBuilder::from(data).median();
    assert_eq!(p50, med);
}

#[test]
fn percentile_0_is_min() {
    let p = QueryBuilder::from(vec![5.0_f64, 3.0, 1.0, 4.0, 2.0]).percentile(0.0);
    assert_eq!(p, Some(1.0));
}

#[test]
fn percentile_1_is_max() {
    let p = QueryBuilder::from(vec![5.0_f64, 3.0, 1.0, 4.0, 2.0]).percentile(1.0);
    assert_eq!(p, Some(5.0));
}

#[test]
fn percentile_empty() {
    assert_eq!(
        QueryBuilder::from(vec![0.0_f64; 0]).percentile(0.5),
        None
    );
}

#[test]
fn quantile_aliases_percentile() {
    let data = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let p = QueryBuilder::from(data.clone()).percentile(0.25);
    let q = QueryBuilder::from(data).quantile(0.25);
    assert_eq!(p, q);
}

// ── skewness ──────────────────────────────────────────────────────────────────

#[test]
fn skewness_symmetric_is_zero() {
    // Symmetric data → skewness ≈ 0
    let sk = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0]).skewness();
    assert!(sk.abs() < EPS);
}

#[test]
fn skewness_right_skewed_positive() {
    // Long right tail → positive skewness
    let sk = QueryBuilder::from(vec![1.0_f64, 1.0, 1.0, 1.0, 10.0]).skewness();
    assert!(sk > 0.0, "expected positive skewness, got {sk}");
}

#[test]
fn skewness_left_skewed_negative() {
    // Long left tail → negative skewness
    let sk = QueryBuilder::from(vec![1.0_f64, 10.0, 10.0, 10.0, 10.0]).skewness();
    assert!(sk < 0.0, "expected negative skewness, got {sk}");
}

#[test]
fn skewness_few_elements() {
    assert_eq!(QueryBuilder::from(vec![1.0_f64, 2.0]).skewness(), 0.0);
}

// ── kurtosis ──────────────────────────────────────────────────────────────────

#[test]
fn kurtosis_uniform_like_negative() {
    // Uniform-like distribution has negative excess kurtosis
    let k = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0]).kurtosis();
    assert!(k < 0.0, "expected negative excess kurtosis, got {k}");
}

#[test]
fn kurtosis_peaked_positive() {
    // Heavy-tailed / peaked distribution has positive excess kurtosis
    let data = vec![0.0_f64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10.0];
    let k = QueryBuilder::from(data).kurtosis();
    assert!(k > 0.0, "expected positive kurtosis, got {k}");
}

#[test]
fn kurtosis_few_elements() {
    assert_eq!(
        QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]).kurtosis(),
        0.0
    );
}

// ── histogram ─────────────────────────────────────────────────────────────────

#[test]
fn histogram_correct_bucket_count() {
    let hist = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0]).histogram(2);
    assert_eq!(hist.len(), 2);
}

#[test]
fn histogram_total_count_equals_input() {
    let data = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let hist = QueryBuilder::from(data.clone()).histogram(3);
    let total: usize = hist.iter().map(|b| b.count).sum();
    assert_eq!(total, data.len());
}

#[test]
fn histogram_empty_input() {
    assert!(QueryBuilder::from(vec![0.0_f64; 0]).histogram(5).is_empty());
}

#[test]
fn histogram_zero_buckets() {
    assert!(QueryBuilder::from(vec![1.0_f64, 2.0]).histogram(0).is_empty());
}

#[test]
fn histogram_single_bucket() {
    let hist = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]).histogram(1);
    assert_eq!(hist.len(), 1);
    assert_eq!(hist[0].count, 3);
}

#[test]
fn histogram_all_same_value() {
    let hist = QueryBuilder::from(vec![5.0_f64, 5.0, 5.0]).histogram(4);
    let total: usize = hist.iter().map(|b| b.count).sum();
    assert_eq!(total, 3);
}

// ── frequency_table ───────────────────────────────────────────────────────────

#[test]
fn frequency_table_basic() {
    let freq = QueryBuilder::from(vec![1, 2, 2, 3, 3, 3]).frequency_table();
    assert_eq!(freq[&1], 1);
    assert_eq!(freq[&2], 2);
    assert_eq!(freq[&3], 3);
}

#[test]
fn frequency_table_total_matches_input() {
    let data = vec![10, 20, 10, 30, 20, 10];
    let freq = QueryBuilder::from(data.clone()).frequency_table();
    let total: usize = freq.values().sum();
    assert_eq!(total, data.len());
}

#[test]
fn frequency_table_empty() {
    let freq = QueryBuilder::from(vec![0i32; 0]).frequency_table();
    assert!(freq.is_empty());
}

#[test]
fn frequency_table_all_unique() {
    let freq = QueryBuilder::from(vec![1, 2, 3, 4]).frequency_table();
    assert!(freq.values().all(|&c| c == 1));
}

// ── chaining with QueryBuilder ops ───────────────────────────────────────────

#[test]
fn variance_after_filter() {
    // filter evens from [1..=8]: [2,4,6,8] → mean=5, σ²=5
    let v = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
        .where_(|x| *x as i32 % 2 == 0)
        .variance();
    assert!((v - 5.0).abs() < EPS);
}

#[test]
fn median_after_sort() {
    let m = QueryBuilder::from(vec![5.0_f64, 3.0, 1.0, 4.0, 2.0])
        .where_(|_| true)
        .order_by(|x| (*x * 100.0) as i64)
        .median();
    assert_eq!(m, Some(3.0));
}
