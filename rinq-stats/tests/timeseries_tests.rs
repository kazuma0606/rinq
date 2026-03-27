use rinq::QueryBuilder;
use rinq_stats::TimeSeriesExt;

// ── EMA ───────────────────────────────────────────────────────────────────────

#[test]
fn ema_empty_returns_empty() {
    let result = QueryBuilder::from(Vec::<f64>::new())
        .exponential_moving_average(0.5);
    assert!(result.is_empty());
}

#[test]
fn ema_single_element() {
    let result = QueryBuilder::from(vec![5.0_f64])
        .exponential_moving_average(0.5);
    assert_eq!(result, vec![5.0]);
}

#[test]
fn ema_alpha_one_equals_input() {
    // alpha=1.0 means EMA = current value
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = QueryBuilder::from(input.clone())
        .exponential_moving_average(1.0);
    assert_eq!(result, input);
}

#[test]
fn ema_alpha_half_basic() {
    // ema[0]=10, ema[1]=0.5*20+0.5*10=15, ema[2]=0.5*30+0.5*15=22.5
    let result = QueryBuilder::from(vec![10.0_f64, 20.0, 30.0])
        .exponential_moving_average(0.5);
    assert!((result[0] - 10.0).abs() < 1e-10);
    assert!((result[1] - 15.0).abs() < 1e-10);
    assert!((result[2] - 22.5).abs() < 1e-10);
}

#[test]
#[should_panic(expected = "alpha must be in (0.0, 1.0]")]
fn ema_alpha_zero_panics() {
    let _ = QueryBuilder::from(vec![1.0_f64]).exponential_moving_average(0.0);
}

#[test]
#[should_panic(expected = "alpha must be in (0.0, 1.0]")]
fn ema_alpha_gt_one_panics() {
    let _ = QueryBuilder::from(vec![1.0_f64]).exponential_moving_average(1.1);
}

// ── Bollinger Bands ──────────────────────────────────────────────────────────

#[test]
fn bollinger_empty_window_returns_empty() {
    let result = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0])
        .bollinger_bands(0, 2.0);
    assert!(result.is_empty());
}

#[test]
fn bollinger_window_greater_than_len_returns_empty() {
    let result = QueryBuilder::from(vec![1.0_f64, 2.0])
        .bollinger_bands(5, 2.0);
    assert!(result.is_empty());
}

#[test]
fn bollinger_empty_input_returns_empty() {
    let result = QueryBuilder::from(Vec::<f64>::new())
        .bollinger_bands(3, 2.0);
    assert!(result.is_empty());
}

#[test]
fn bollinger_window_one_std_dev_zero() {
    // window=1: each point is its own window, std_dev=0, upper=lower=middle=value
    let result = QueryBuilder::from(vec![5.0_f64, 10.0, 15.0])
        .bollinger_bands(1, 2.0);
    assert_eq!(result.len(), 3);
    for (p, &v) in result.iter().zip([5.0, 10.0, 15.0].iter()) {
        assert!((p.value - v).abs() < 1e-10);
        assert!((p.middle - v).abs() < 1e-10);
        assert!((p.upper - v).abs() < 1e-10);
        assert!((p.lower - v).abs() < 1e-10);
    }
}

#[test]
fn bollinger_window_equals_len_single_point() {
    // [2, 4, 4, 4, 5, 5, 7, 9]: mean=5, pop std_dev=2
    let values = vec![2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let result = QueryBuilder::from(values.clone())
        .bollinger_bands(values.len(), 1.0);
    assert_eq!(result.len(), 1);
    assert!((result[0].middle - 5.0).abs() < 1e-10);
    assert!((result[0].upper - 7.0).abs() < 1e-10);
    assert!((result[0].lower - 3.0).abs() < 1e-10);
}

#[test]
fn bollinger_middle_equals_rolling_mean() {
    let values = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let result = QueryBuilder::from(values.clone()).bollinger_bands(3, 2.0);
    // 3 points: windows [1,2,3],[2,3,4],[3,4,5] → means 2,3,4
    assert_eq!(result.len(), 3);
    assert!((result[0].middle - 2.0).abs() < 1e-10);
    assert!((result[1].middle - 3.0).abs() < 1e-10);
    assert!((result[2].middle - 4.0).abs() < 1e-10);
}
