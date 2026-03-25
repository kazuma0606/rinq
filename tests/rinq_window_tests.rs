// tests/rinq_window_tests.rs
// Phase A2 integration tests for window analytics functions.
// Run with: cargo test --test rinq_window_tests

use rinq::QueryBuilder;

// ── running_sum ──────────────────────────────────────────────────────────────

#[test]
fn running_sum_basic() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .running_sum()
        .collect();
    assert_eq!(result, vec![1, 3, 6, 10, 15]);
}

#[test]
fn running_sum_single_element() {
    let result: Vec<i32> = QueryBuilder::from(vec![42]).running_sum().collect();
    assert_eq!(result, vec![42]);
}

#[test]
fn running_sum_empty() {
    let result: Vec<i32> = QueryBuilder::from(vec![]).running_sum().collect();
    assert!(result.is_empty());
}

#[test]
fn running_sum_floats() {
    let result: Vec<f64> = QueryBuilder::from(vec![1.0, 2.0, 3.0])
        .running_sum()
        .collect();
    assert_eq!(result, vec![1.0, 3.0, 6.0]);
}

#[test]
fn running_sum_after_filter() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|x| *x % 2 != 0) // [1, 3, 5]
        .running_sum()
        .collect();
    assert_eq!(result, vec![1, 4, 9]);
}

#[test]
fn running_sum_after_sort() {
    let result: Vec<i32> = QueryBuilder::from(vec![3, 1, 2])
        .where_(|_| true)
        .order_by(|x| *x)
        .running_sum()
        .collect();
    assert_eq!(result, vec![1, 3, 6]);
}

// ── running_average ──────────────────────────────────────────────────────────

#[test]
fn running_average_basic() {
    let result: Vec<f64> = QueryBuilder::from(vec![1, 2, 3]).running_average().collect();
    assert_eq!(result, vec![1.0, 1.5, 2.0]);
}

#[test]
fn running_average_single() {
    let result: Vec<f64> = QueryBuilder::from(vec![7]).running_average().collect();
    assert_eq!(result, vec![7.0]);
}

#[test]
fn running_average_empty() {
    let result: Vec<f64> = QueryBuilder::from(vec![0i32; 0])
        .running_average()
        .collect();
    assert!(result.is_empty());
}

#[test]
fn running_average_precision() {
    let result: Vec<f64> = QueryBuilder::from(vec![1, 2, 3, 4])
        .running_average()
        .collect();
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], 1.5);
    assert_eq!(result[2], 2.0);
    assert_eq!(result[3], 2.5);
}

#[test]
fn running_average_after_filter() {
    // even numbers: [2, 4, 6] → avg: [2.0, 3.0, 4.0]
    let result: Vec<f64> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
        .where_(|x| *x % 2 == 0)
        .running_average()
        .collect();
    assert_eq!(result, vec![2.0, 3.0, 4.0]);
}

// ── moving_average ───────────────────────────────────────────────────────────

#[test]
fn moving_average_basic() {
    let ma: Vec<Option<f64>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .moving_average(3)
        .collect();
    assert_eq!(ma.len(), 5);
    assert_eq!(ma[0], None);
    assert_eq!(ma[1], None);
    assert_eq!(ma[2], Some(2.0)); // avg(1,2,3)
    assert_eq!(ma[3], Some(3.0)); // avg(2,3,4)
    assert_eq!(ma[4], Some(4.0)); // avg(3,4,5)
}

#[test]
fn moving_average_window_1() {
    // window=1 → every element is Some(itself)
    let ma: Vec<Option<f64>> = QueryBuilder::from(vec![10, 20, 30])
        .moving_average(1)
        .collect();
    assert_eq!(ma, vec![Some(10.0), Some(20.0), Some(30.0)]);
}

#[test]
fn moving_average_window_equals_len() {
    let ma: Vec<Option<f64>> = QueryBuilder::from(vec![1, 2, 3])
        .moving_average(3)
        .collect();
    assert_eq!(ma[0], None);
    assert_eq!(ma[1], None);
    assert_eq!(ma[2], Some(2.0));
}

#[test]
fn moving_average_window_larger_than_len() {
    // window > len → all None
    let ma: Vec<Option<f64>> = QueryBuilder::from(vec![1, 2])
        .moving_average(5)
        .collect();
    assert_eq!(ma, vec![None, None]);
}

#[test]
fn moving_average_empty() {
    let ma: Vec<Option<f64>> = QueryBuilder::from(vec![0i32; 0])
        .moving_average(3)
        .collect();
    assert!(ma.is_empty());
}

#[test]
#[should_panic(expected = "window must be >= 1")]
fn moving_average_window_zero_panics() {
    let _: Vec<Option<f64>> = QueryBuilder::from(vec![1, 2, 3]).moving_average(0).collect();
}

#[test]
fn moving_average_none_filtering() {
    // Spec pattern: filter out None and unwrap
    let values: Vec<f64> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .moving_average(3)
        .where_(|v| v.is_some())
        .select(|v| v.unwrap())
        .collect();
    assert_eq!(values, vec![2.0, 3.0, 4.0]);
}

#[test]
fn moving_average_after_filter() {
    // filter evens first: [2,4,6,8] → ma(2): [None, Some(3.0), Some(5.0), Some(7.0)]
    let ma: Vec<Option<f64>> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8])
        .where_(|x| *x % 2 == 0)
        .moving_average(2)
        .collect();
    assert_eq!(ma[0], None);
    assert_eq!(ma[1], Some(3.0)); // avg(2,4)
    assert_eq!(ma[2], Some(5.0)); // avg(4,6)
    assert_eq!(ma[3], Some(7.0)); // avg(6,8)
}

// ── rank_by ──────────────────────────────────────────────────────────────────

#[test]
fn rank_by_unique_values() {
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![30, 10, 20])
        .rank_by(|x| *x)
        .collect();
    // sorted: 10(rank1), 20(rank2), 30(rank3)
    assert_eq!(ranked, vec![(3, 30), (1, 10), (2, 20)]);
}

#[test]
fn rank_by_all_same() {
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![5, 5, 5])
        .rank_by(|x| *x)
        .collect();
    // all rank 1 (tie), next would be 4 (skipped 2,3)
    assert!(ranked.iter().all(|(r, _)| *r == 1));
}

#[test]
fn rank_by_with_ties_skips() {
    // [10, 30, 30, 20]: sorted 10,20,30,30 → rank 1,2,3,3
    // 10→1, 20→2, 30→3, 30→3
    // in original order: 10→1, 30→3, 30→3, 20→2
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![10, 30, 30, 20])
        .rank_by(|x| *x)
        .collect();
    assert_eq!(ranked[0], (1, 10));
    assert_eq!(ranked[1], (3, 30));
    assert_eq!(ranked[2], (3, 30));
    assert_eq!(ranked[3], (2, 20));
}

#[test]
fn rank_by_empty() {
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![]).rank_by(|x| *x).collect();
    assert!(ranked.is_empty());
}

#[test]
fn rank_by_single() {
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![99]).rank_by(|x| *x).collect();
    assert_eq!(ranked, vec![(1, 99)]);
}

#[test]
fn rank_by_preserves_input_order() {
    // Confirm original order is preserved in output, not sorted order
    let input = vec![5, 3, 1, 4, 2];
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(input.clone())
        .rank_by(|x| *x)
        .collect();
    // values should appear in input order
    let vals: Vec<i32> = ranked.iter().map(|(_, v)| *v).collect();
    assert_eq!(vals, input);
}

// ── dense_rank_by ────────────────────────────────────────────────────────────

#[test]
fn dense_rank_by_unique_values() {
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![30, 10, 20])
        .dense_rank_by(|x| *x)
        .collect();
    assert_eq!(ranked, vec![(3, 30), (1, 10), (2, 20)]);
}

#[test]
fn dense_rank_by_with_ties_no_skip() {
    // [10, 30, 30, 20]: dense ranks 1,2,3,3 (no skip after tie)
    // Wait: sorted 10(rank1), 20(rank2), 30(rank3), 30(rank3)
    // In original order: 10→1, 30→3, 30→3, 20→2
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![10, 30, 30, 20])
        .dense_rank_by(|x| *x)
        .collect();
    assert_eq!(ranked[0], (1, 10));
    assert_eq!(ranked[1], (3, 30));
    assert_eq!(ranked[2], (3, 30));
    assert_eq!(ranked[3], (2, 20));
}

#[test]
fn rank_vs_dense_rank_differ_on_ties() {
    // [1, 3, 3, 4]
    // rank:       1, 2, 2, 4  (skips 3)
    // dense_rank: 1, 2, 2, 3  (no skip)
    let input = vec![1, 3, 3, 4];
    let rank: Vec<usize> = QueryBuilder::from(input.clone())
        .rank_by(|x| *x)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(r, _)| r)
        .collect();
    let dense: Vec<usize> = QueryBuilder::from(input)
        .dense_rank_by(|x| *x)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(r, _)| r)
        .collect();
    assert_eq!(rank, vec![1, 2, 2, 4]);
    assert_eq!(dense, vec![1, 2, 2, 3]);
}

#[test]
fn dense_rank_by_all_same() {
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![7, 7, 7])
        .dense_rank_by(|x| *x)
        .collect();
    assert!(ranked.iter().all(|(r, _)| *r == 1));
}

// ── lag ──────────────────────────────────────────────────────────────────────

#[test]
fn lag_1_basic() {
    let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![10, 20, 30])
        .lag(1)
        .collect();
    assert_eq!(result, vec![(None, 10), (Some(10), 20), (Some(20), 30)]);
}

#[test]
fn lag_2_offsets() {
    let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![1, 2, 3, 4])
        .lag(2)
        .collect();
    assert_eq!(
        result,
        vec![(None, 1), (None, 2), (Some(1), 3), (Some(2), 4)]
    );
}

#[test]
fn lag_0_self_reference() {
    // lag(0) pairs each element with itself
    let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![1, 2, 3])
        .lag(0)
        .collect();
    assert_eq!(
        result,
        vec![(Some(1), 1), (Some(2), 2), (Some(3), 3)]
    );
}

#[test]
fn lag_larger_than_len() {
    // All lags are None
    let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![1, 2])
        .lag(5)
        .collect();
    assert_eq!(result, vec![(None, 1), (None, 2)]);
}

#[test]
fn lag_empty() {
    let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![]).lag(1).collect();
    assert!(result.is_empty());
}

#[test]
fn lag_then_where_() {
    // Keep only elements where a previous value exists
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![10, 20, 30])
        .lag(1)
        .where_(|(prev, _)| prev.is_some())
        .select(|(prev, curr)| (prev.unwrap(), curr))
        .collect();
    assert_eq!(result, vec![(10, 20), (20, 30)]);
}

// ── lead ─────────────────────────────────────────────────────────────────────

#[test]
fn lead_1_basic() {
    let result: Vec<(i32, Option<i32>)> = QueryBuilder::from(vec![10, 20, 30])
        .lead(1)
        .collect();
    assert_eq!(result, vec![(10, Some(20)), (20, Some(30)), (30, None)]);
}

#[test]
fn lead_2_offsets() {
    let result: Vec<(i32, Option<i32>)> = QueryBuilder::from(vec![1, 2, 3, 4])
        .lead(2)
        .collect();
    assert_eq!(
        result,
        vec![(1, Some(3)), (2, Some(4)), (3, None), (4, None)]
    );
}

#[test]
fn lead_0_self_reference() {
    let result: Vec<(i32, Option<i32>)> = QueryBuilder::from(vec![1, 2, 3])
        .lead(0)
        .collect();
    assert_eq!(
        result,
        vec![(1, Some(1)), (2, Some(2)), (3, Some(3))]
    );
}

#[test]
fn lead_larger_than_len() {
    let result: Vec<(i32, Option<i32>)> = QueryBuilder::from(vec![1, 2])
        .lead(5)
        .collect();
    assert_eq!(result, vec![(1, None), (2, None)]);
}

#[test]
fn lead_empty() {
    let result: Vec<(i32, Option<i32>)> = QueryBuilder::from(vec![]).lead(1).collect();
    assert!(result.is_empty());
}

#[test]
fn lead_then_where_() {
    // Keep only elements that have a successor
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![10, 20, 30])
        .lead(1)
        .where_(|(_, next)| next.is_some())
        .select(|(curr, next)| (curr, next.unwrap()))
        .collect();
    assert_eq!(result, vec![(10, 20), (20, 30)]);
}

// ── chaining with other operations ───────────────────────────────────────────

#[test]
fn running_sum_chained_with_where_() {
    // Prefix sums, keep only those > 5
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .running_sum()
        .where_(|x| *x > 5)
        .collect();
    assert_eq!(result, vec![6, 10, 15]);
}

#[test]
fn moving_average_chained_then_order_by() {
    // flat_map keeps state as Filtered, allowing order_by afterwards
    let result: Vec<f64> = QueryBuilder::from(vec![3, 1, 2, 5, 4])
        .moving_average(2)
        .where_(|v| v.is_some())
        .flat_map(|v| std::iter::once(v.unwrap()))
        .order_by(|x| ordered_float(*x))
        .collect();
    // avg(3,1)=2.0, avg(1,2)=1.5, avg(2,5)=3.5, avg(5,4)=4.5 → sorted
    assert_eq!(result, vec![1.5, 2.0, 3.5, 4.5]);
}

/// Newtype wrapper so f64 can be used as an `Ord` key.
#[derive(PartialEq, PartialOrd)]
struct OrdF64(f64);
impl Eq for OrdF64 {}
impl Ord for OrdF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
fn ordered_float(x: f64) -> OrdF64 {
    OrdF64(x)
}

#[test]
fn rank_by_after_filter() {
    // filter to even numbers, then rank
    let ranked: Vec<(usize, i32)> = QueryBuilder::from(vec![3, 4, 1, 2, 5, 6])
        .where_(|x| *x % 2 == 0)
        .rank_by(|x| *x)
        .collect();
    // evens: [4, 2, 6] → ranks: 4→2, 2→1, 6→3
    assert_eq!(ranked, vec![(2, 4), (1, 2), (3, 6)]);
}

#[test]
fn lag_after_sort() {
    // sort ascending then lag
    let result: Vec<(Option<i32>, i32)> = QueryBuilder::from(vec![30, 10, 20])
        .where_(|_| true)
        .order_by(|x| *x)
        .lag(1)
        .collect();
    assert_eq!(result, vec![(None, 10), (Some(10), 20), (Some(20), 30)]);
}
