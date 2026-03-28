// tests/rinq_v4_tests.rs
// Integration tests for Phase 4 features (4A–4D)

use rinq::{FilteredQuery, IntoQuery, QueryBuilder};
use std::sync::Arc;

// ── H1: from_arc_cloned / from_arc_slice_cloned ──────────────────────────────

#[test]
fn arc_cloned_basic() {
    let shared: Arc<Vec<i32>> = Arc::new(vec![1, 2, 3, 4, 5]);
    let result: Vec<i32> = QueryBuilder::from_arc_cloned(&shared)
        .where_(|x| x % 2 == 0)
        .collect();
    assert_eq!(result, vec![2, 4]);
}

#[test]
fn arc_cloned_empty_vec() {
    let shared: Arc<Vec<i32>> = Arc::new(vec![]);
    let result: Vec<i32> = QueryBuilder::from_arc_cloned(&shared).collect();
    assert!(result.is_empty());
}

#[test]
fn arc_cloned_does_not_consume_arc() {
    let shared: Arc<Vec<i32>> = Arc::new(vec![10, 20, 30]);
    let _q1: Vec<i32> = QueryBuilder::from_arc_cloned(&shared).collect();
    // Arc still accessible after the query
    let _q2: Vec<i32> = QueryBuilder::from_arc_cloned(&shared).collect();
    assert_eq!(Arc::strong_count(&shared), 1);
}

#[test]
fn arc_cloned_multithreaded() {
    use std::thread;
    let shared: Arc<Vec<i32>> = Arc::new((0..100).collect());
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let arc = shared.clone();
            thread::spawn(move || {
                let sum: i32 = QueryBuilder::from_arc_cloned(&arc)
                    .where_(move |x| x % 4 == i)
                    .sum();
                sum
            })
        })
        .collect();
    let total: i32 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    // Sum of 0..100 = 4950
    assert_eq!(total, (0..100i32).sum::<i32>());
}

#[test]
fn arc_slice_cloned_basic() {
    let shared: Arc<[i32]> = Arc::from(vec![1, 2, 3, 4, 5]);
    let result: Vec<i32> = QueryBuilder::from_arc_slice_cloned(&shared)
        .where_(|x| *x > 3)
        .collect();
    assert_eq!(result, vec![4, 5]);
}

#[test]
fn arc_slice_cloned_empty() {
    let shared: Arc<[i32]> = Arc::from(vec![]);
    let result: Vec<i32> = QueryBuilder::from_arc_slice_cloned(&shared).collect();
    assert!(result.is_empty());
}

// ── J3: IntoQuery trait ───────────────────────────────────────────────────────

#[test]
fn into_query_vec() {
    let result: Vec<i32> = vec![1, 2, 3, 4, 5]
        .into_query()
        .where_(|x| x % 2 == 0)
        .collect();
    assert_eq!(result, vec![2, 4]);
}

#[test]
fn into_query_chained() {
    let result: Vec<i32> = vec![5, 3, 1, 4, 2].into_query().order_by(|x| *x).collect();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

// ── J1: filter_map ────────────────────────────────────────────────────────────

#[test]
fn filter_map_all_some() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .filter_map(|x| Some(x * 10))
        .collect();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn filter_map_all_none() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .filter_map(|_| None::<i32>)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn filter_map_partial() {
    let strings = vec!["1", "two", "3", "four"];
    let result: Vec<i32> = QueryBuilder::from(strings)
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    assert_eq!(result, vec![1, 3]);
}

#[test]
fn filter_map_empty_input() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<&str>::new())
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    assert!(result.is_empty());
}

#[test]
fn filter_map_type_conversion() {
    let result: Vec<f64> = QueryBuilder::from(vec![1i32, 2, 3])
        .filter_map(|x| if x > 1 { Some(x as f64 * 1.5) } else { None })
        .collect();
    assert_eq!(result, vec![3.0, 4.5]);
}

#[test]
fn filter_map_after_chain() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4])
        .where_(|x| x % 2 == 0)
        .filter_map(|x| if x > 2 { Some(x * 10) } else { None })
        .collect();
    assert_eq!(result, vec![40]);
}

// ── J5: step_by ───────────────────────────────────────────────────────────────

#[test]
fn step_by_1() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5]).step_by(1).collect();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn step_by_2() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5]).step_by(2).collect();
    assert_eq!(result, vec![1, 3, 5]);
}

#[test]
fn step_by_larger_than_len() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3]).step_by(10).collect();
    assert_eq!(result, vec![1]);
}

#[test]
fn step_by_empty() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<i32>::new()).step_by(2).collect();
    assert!(result.is_empty());
}

#[test]
#[should_panic(expected = "step must be greater than 0")]
fn step_by_zero_panics() {
    let _ = QueryBuilder::from(vec![1, 2, 3])
        .step_by(0)
        .collect::<Vec<_>>();
}

// ── J6: cycle ─────────────────────────────────────────────────────────────────

#[test]
fn cycle_with_take() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3]).cycle().take(7).collect();
    assert_eq!(result, vec![1, 2, 3, 1, 2, 3, 1]);
}

#[test]
fn cycle_empty_collection() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<i32>::new())
        .cycle()
        .take(5)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn cycle_round_robin() {
    // Simulate round-robin assignment
    let workers = vec!["A", "B", "C"];
    let assignments: Vec<&str> = QueryBuilder::from(workers).cycle().take(7).collect();
    assert_eq!(assignments, vec!["A", "B", "C", "A", "B", "C", "A"]);
}

// ── E1: scan ──────────────────────────────────────────────────────────────────

#[test]
fn scan_cumulative_product() {
    let result: Vec<i64> = QueryBuilder::from(vec![1i64, 2, 3, 4, 5])
        .scan(1i64, |acc, x| acc * x)
        .collect();
    assert_eq!(result, vec![1, 2, 6, 24, 120]);
}

#[test]
fn scan_empty() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<i32>::new())
        .scan(0, |acc, x| acc + x)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn scan_running_sum() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4])
        .scan(0, |acc, x| acc + x)
        .collect();
    assert_eq!(result, vec![1, 3, 6, 10]);
}

#[test]
fn scan_string_concat() {
    let result: Vec<String> = QueryBuilder::from(vec!["a", "b", "c"])
        .scan(String::new(), |acc, x| acc + x)
        .collect();
    assert_eq!(result, vec!["a", "ab", "abc"]);
}

// ── E2: chunk_by ──────────────────────────────────────────────────────────────

#[test]
fn chunk_by_basic() {
    let data = vec![1, 1, 2, 2, 3, 1, 1];
    let chunks: Vec<Vec<i32>> = QueryBuilder::from(data).chunk_by(|x| *x).collect();
    assert_eq!(chunks, vec![vec![1, 1], vec![2, 2], vec![3], vec![1, 1]]);
}

#[test]
fn chunk_by_all_same() {
    let chunks: Vec<Vec<i32>> = QueryBuilder::from(vec![5, 5, 5]).chunk_by(|x| *x).collect();
    assert_eq!(chunks, vec![vec![5, 5, 5]]);
}

#[test]
fn chunk_by_all_different() {
    let chunks: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3]).chunk_by(|x| *x).collect();
    assert_eq!(chunks, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn chunk_by_empty() {
    let chunks: Vec<Vec<i32>> = QueryBuilder::from(Vec::<i32>::new())
        .chunk_by(|x| *x)
        .collect();
    assert!(chunks.is_empty());
}

#[test]
fn chunk_by_single_element() {
    let chunks: Vec<Vec<i32>> = QueryBuilder::from(vec![42]).chunk_by(|x| *x).collect();
    assert_eq!(chunks, vec![vec![42]]);
}

// ── E3: dedup / dedup_by ─────────────────────────────────────────────────────

#[test]
fn dedup_consecutive() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 1, 2, 3, 3, 1]).dedup().collect();
    assert_eq!(result, vec![1, 2, 3, 1]);
}

#[test]
fn dedup_non_consecutive_preserved() {
    // Non-consecutive duplicates stay
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 1, 2]).dedup().collect();
    assert_eq!(result, vec![1, 2, 1, 2]);
}

#[test]
fn dedup_all_same() {
    let result: Vec<i32> = QueryBuilder::from(vec![7, 7, 7]).dedup().collect();
    assert_eq!(result, vec![7]);
}

#[test]
fn dedup_all_different() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3]).dedup().collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn dedup_empty() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<i32>::new()).dedup().collect();
    assert!(result.is_empty());
}

#[test]
fn dedup_by_key() {
    #[derive(Clone, Debug, PartialEq)]
    struct Item {
        cat: &'static str,
        val: i32,
    }
    let data = vec![
        Item { cat: "a", val: 1 },
        Item { cat: "a", val: 2 },
        Item { cat: "b", val: 3 },
        Item { cat: "b", val: 4 },
        Item { cat: "a", val: 5 },
    ];
    let result: Vec<Item> = QueryBuilder::from(data).dedup_by(|x| x.cat).collect();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].cat, "a");
    assert_eq!(result[1].cat, "b");
    assert_eq!(result[2].cat, "a");
}

// ── E4: zip_with ─────────────────────────────────────────────────────────────

#[test]
fn zip_with_same_length() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .zip_with(vec![10, 20, 30], |a, b| a + b)
        .collect();
    assert_eq!(result, vec![11, 22, 33]);
}

#[test]
fn zip_with_left_longer() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4])
        .zip_with(vec![10, 20], |a, b| a + b)
        .collect();
    assert_eq!(result, vec![11, 22]);
}

#[test]
fn zip_with_right_longer() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2])
        .zip_with(vec![10, 20, 30], |a, b| a + b)
        .collect();
    assert_eq!(result, vec![11, 22]);
}

#[test]
fn zip_with_empty() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<i32>::new())
        .zip_with(vec![1, 2, 3], |a, b| a + b)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn zip_with_type_conversion() {
    let prices = vec![100.0f64, 200.0, 300.0];
    let taxes = vec![0.1f64, 0.2, 0.05];
    let totals: Vec<f64> = QueryBuilder::from(prices)
        .zip_with(taxes, |price, tax| price * (1.0 + tax))
        .collect();
    assert!((totals[0] - 110.0).abs() < 1e-9);
    assert!((totals[1] - 240.0).abs() < 1e-9);
    assert!((totals[2] - 315.0).abs() < 1e-9);
}

// ── E5: pairwise ─────────────────────────────────────────────────────────────

#[test]
fn pairwise_basic() {
    let pairs: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2, 3, 4]).pairwise().collect();
    assert_eq!(pairs, vec![(1, 2), (2, 3), (3, 4)]);
}

#[test]
fn pairwise_empty() {
    let pairs: Vec<(i32, i32)> = QueryBuilder::from(Vec::<i32>::new()).pairwise().collect();
    assert!(pairs.is_empty());
}

#[test]
fn pairwise_single_element() {
    let pairs: Vec<(i32, i32)> = QueryBuilder::from(vec![42]).pairwise().collect();
    assert!(pairs.is_empty());
}

#[test]
fn pairwise_after_select() {
    // pairwise then map to compute differences
    let diffs: Vec<i32> = QueryBuilder::from(vec![10, 15, 13, 20])
        .pairwise()
        .filter_map(|(a, b)| Some(b - a))
        .collect();
    assert_eq!(diffs, vec![5, -2, 7]);
}

// ── E6: unfold / unfold_bounded ───────────────────────────────────────────────

#[test]
fn unfold_fibonacci_with_take() {
    let fibs: Vec<u64> = QueryBuilder::unfold((0u64, 1u64), |(a, b)| Some((a, (b, a + b))))
        .take(7)
        .collect();
    assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8]);
}

#[test]
fn unfold_finite_termination() {
    // Count down from 5 to 1
    let result: Vec<i32> =
        QueryBuilder::unfold(5i32, |n| if n > 0 { Some((n, n - 1)) } else { None }).collect();
    assert_eq!(result, vec![5, 4, 3, 2, 1]);
}

#[test]
fn unfold_empty_on_first_none() {
    let result: Vec<i32> = QueryBuilder::unfold(0i32, |_| None::<(i32, i32)>).collect();
    assert!(result.is_empty());
}

#[test]
fn unfold_bounded_fibonacci() {
    let fibs: Vec<u64> =
        QueryBuilder::unfold_bounded((0u64, 1u64), |(a, b)| Some((a, (b, a + b))), 8).collect();
    assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13]);
}

#[test]
fn unfold_bounded_respects_limit() {
    // Would be infinite, but capped at 5
    let result: Vec<i32> = QueryBuilder::unfold_bounded(0i32, |n| Some((n, n + 1)), 5).collect();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

// ── E7: intersperse ───────────────────────────────────────────────────────────

#[test]
fn intersperse_basic() {
    let result: Vec<&str> = QueryBuilder::from(vec!["a", "b", "c"])
        .intersperse(", ")
        .collect();
    assert_eq!(result, vec!["a", ", ", "b", ", ", "c"]);
}

#[test]
fn intersperse_empty() {
    let result: Vec<i32> = QueryBuilder::from(Vec::<i32>::new())
        .intersperse(0)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn intersperse_single() {
    let result: Vec<i32> = QueryBuilder::from(vec![42]).intersperse(0).collect();
    assert_eq!(result, vec![42]);
}

#[test]
fn intersperse_two_elements() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2]).intersperse(0).collect();
    assert_eq!(result, vec![1, 0, 2]);
}

// ── E8: min_max ───────────────────────────────────────────────────────────────

#[test]
fn min_max_basic() {
    let result = QueryBuilder::from(vec![3, 1, 4, 1, 5, 9]).min_max();
    assert_eq!(result, Some((1, 9)));
}

#[test]
fn min_max_empty() {
    let result = QueryBuilder::from(Vec::<i32>::new()).min_max();
    assert_eq!(result, None);
}

#[test]
fn min_max_all_same() {
    let result = QueryBuilder::from(vec![7, 7, 7]).min_max();
    assert_eq!(result, Some((7, 7)));
}

#[test]
fn min_max_two_elements() {
    let result = QueryBuilder::from(vec![10, 3]).min_max();
    assert_eq!(result, Some((3, 10)));
}

#[test]
fn min_max_after_filter() {
    let result = QueryBuilder::from(vec![1, 10, 2, 9, 3])
        .where_(|x| x % 2 == 0)
        .min_max();
    assert_eq!(result, Some((2, 10)));
}

// ── D2b: HashEqBound error message check (positive case) ──────────────────────

#[test]
fn distinct_works_with_hash_eq() {
    // This should compile and run: i32 implements Hash + Eq
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 2, 3, 1]).distinct().collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn union_works_with_hash_eq() {
    let mut result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .union(vec![2, 3, 4])
        .collect();
    result.sort();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

// ── Type alias usability ───────────────────────────────────────────────────────

fn adults_only(ages: Vec<u32>) -> FilteredQuery<u32> {
    QueryBuilder::from(ages).where_(|&a| a >= 18)
}

#[test]
fn type_alias_in_function_signature() {
    let result: Vec<u32> = adults_only(vec![10, 18, 25, 15]).collect();
    assert_eq!(result, vec![18, 25]);
}
