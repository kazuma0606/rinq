// tests/rinq_parallel_tests.rs
// Phase A1 integration tests for ParallelQueryBuilder
// Run with: cargo test --features parallel --test rinq_parallel_tests

#![cfg(feature = "parallel")]

use rinq::QueryBuilder;
use rinq::parallel::ParallelQueryBuilder;
use std::collections::HashMap;

// ── into_parallel() conversion ──────────────────────────────────────────────

#[test]
fn into_parallel_from_initial() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3]).into_parallel().collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn into_parallel_from_filtered() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|x| *x > 2)
        .into_parallel()
        .collect();
    let mut r = result;
    r.sort();
    assert_eq!(r, vec![3, 4, 5]);
}

#[test]
fn into_parallel_from_sorted() {
    let result: Vec<i32> = QueryBuilder::from(vec![3, 1, 2])
        .where_(|_| true)
        .order_by(|x| *x)
        .into_parallel()
        .collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn into_parallel_empty_vec() {
    let result: Vec<i32> = QueryBuilder::from(vec![]).into_parallel().collect();
    assert!(result.is_empty());
}

// ── par_where ───────────────────────────────────────────────────────────────

#[test]
fn par_where_basic_filter() {
    let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
        .par_where(|x| *x % 2 == 0)
        .collect();
    result.sort();
    assert_eq!(result, vec![2, 4]);
}

#[test]
fn par_where_empty_input() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![])
        .par_where(|x| *x > 0)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn par_where_no_match() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3])
        .par_where(|x| *x > 100)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn par_where_all_match() {
    let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3])
        .par_where(|x| *x > 0)
        .collect();
    result.sort();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn par_where_chained() {
    let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
        .par_where(|x| *x % 2 == 0)
        .par_where(|x| *x > 2)
        .collect();
    result.sort();
    assert_eq!(result, vec![4, 6]);
}

// ── par_select ──────────────────────────────────────────────────────────────

#[test]
fn par_select_transform() {
    let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3])
        .par_where(|_| true)
        .par_select(|x| x * 2)
        .collect();
    result.sort();
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn par_select_type_change() {
    let mut result: Vec<String> = ParallelQueryBuilder::from(vec![1, 2, 3])
        .par_where(|_| true)
        .par_select(|x| x.to_string())
        .collect();
    result.sort();
    assert_eq!(result, vec!["1", "2", "3"]);
}

#[test]
fn par_select_empty_input() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![])
        .par_where(|_| true)
        .par_select(|x: i32| x * 2)
        .collect();
    assert!(result.is_empty());
}

// ── par_flat_map ─────────────────────────────────────────────────────────────

#[test]
fn par_flat_map_basic() {
    let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![vec![1, 2], vec![3, 4], vec![5]])
        .par_where(|v| !v.is_empty())
        .par_flat_map(|v| v)
        .collect();
    result.sort();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn par_flat_map_from_initial() {
    let mut result: Vec<i32> = ParallelQueryBuilder::from(vec![1, 2, 3])
        .par_flat_map(|x| vec![x, x * 10])
        .collect();
    result.sort();
    assert_eq!(result, vec![1, 2, 3, 10, 20, 30]);
}

#[test]
fn par_flat_map_empty_inner() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![vec![], vec![], vec![]])
        .par_where(|_| true)
        .par_flat_map(|v: Vec<i32>| v)
        .collect();
    assert!(result.is_empty());
}

// ── par_order_by ─────────────────────────────────────────────────────────────

#[test]
fn par_order_by_ascending() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![3, 1, 4, 1, 5])
        .par_where(|_| true)
        .par_order_by(|x| *x)
        .collect();
    assert_eq!(result, vec![1, 1, 3, 4, 5]);
}

#[test]
fn par_order_by_from_initial() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![5, 3, 1, 4, 2])
        .par_order_by(|x| *x)
        .collect();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn par_order_by_strings() {
    let result: Vec<&str> = ParallelQueryBuilder::from(vec!["banana", "apple", "cherry"])
        .par_order_by(|s| s.len())
        .collect();
    // "apple" (5) < "banana"/"cherry" (6)
    assert_eq!(result[0], "apple");
    assert_eq!(result[1..].iter().filter(|&&s| s == "banana").count(), 1);
    assert_eq!(result[1..].iter().filter(|&&s| s == "cherry").count(), 1);
}

#[test]
fn par_order_by_then_filter() {
    let result: Vec<i32> = ParallelQueryBuilder::from(vec![5, 3, 1, 4, 2])
        .par_order_by(|x| *x)
        .par_where(|x| *x > 2)
        .collect();
    assert_eq!(result, vec![3, 4, 5]);
}

// ── par_count ────────────────────────────────────────────────────────────────

#[test]
fn par_count_basic() {
    let count = ParallelQueryBuilder::from(vec![1, 2, 3]).par_count();
    assert_eq!(count, 3);
}

#[test]
fn par_count_empty() {
    let count: usize = ParallelQueryBuilder::from(vec![0i32; 0]).par_count();
    assert_eq!(count, 0);
}

#[test]
fn par_count_after_filter() {
    let count = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5])
        .par_where(|x| *x > 2)
        .par_count();
    assert_eq!(count, 3);
}

// ── par_sum ──────────────────────────────────────────────────────────────────

#[test]
fn par_sum_basic() {
    let total: i32 = ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5]).par_sum();
    assert_eq!(total, 15);
}

#[test]
fn par_sum_empty() {
    let total: i32 = ParallelQueryBuilder::from(vec![]).par_sum();
    assert_eq!(total, 0);
}

#[test]
fn par_sum_matches_sequential() {
    let data = vec![10, 20, 30, 40, 50];
    let seq: i32 = QueryBuilder::from(data.clone()).where_(|_| true).sum();
    let par: i32 = ParallelQueryBuilder::from(data).par_sum();
    assert_eq!(seq, par);
}

// ── par_min / par_max ────────────────────────────────────────────────────────

#[test]
fn par_min_basic() {
    let min = ParallelQueryBuilder::from(vec![3, 1, 4, 1, 5]).par_min();
    assert_eq!(min, Some(1));
}

#[test]
fn par_min_empty() {
    let min: Option<i32> = ParallelQueryBuilder::from(vec![]).par_min();
    assert_eq!(min, None);
}

#[test]
fn par_max_basic() {
    let max = ParallelQueryBuilder::from(vec![3, 1, 4, 1, 5]).par_max();
    assert_eq!(max, Some(5));
}

#[test]
fn par_max_empty() {
    let max: Option<i32> = ParallelQueryBuilder::from(vec![]).par_max();
    assert_eq!(max, None);
}

#[test]
fn par_min_by_key() {
    let min = ParallelQueryBuilder::from(vec!["hello", "hi", "hey"]).par_min_by(|s| s.len());
    assert_eq!(min, Some("hi"));
}

#[test]
fn par_max_by_key() {
    let max = ParallelQueryBuilder::from(vec!["hello", "hi", "hey"]).par_max_by(|s| s.len());
    assert_eq!(max, Some("hello"));
}

// ── par_any / par_all ────────────────────────────────────────────────────────

#[test]
fn par_any_found() {
    let found = ParallelQueryBuilder::from(vec![1, 2, 3]).par_any(|x| *x == 2);
    assert!(found);
}

#[test]
fn par_any_not_found() {
    let found = ParallelQueryBuilder::from(vec![1, 2, 3]).par_any(|x| *x == 99);
    assert!(!found);
}

#[test]
fn par_any_empty() {
    let found = ParallelQueryBuilder::from(vec![0i32; 0]).par_any(|x| *x > 0);
    assert!(!found);
}

#[test]
fn par_all_true() {
    let all = ParallelQueryBuilder::from(vec![1, 2, 3]).par_all(|x| *x > 0);
    assert!(all);
}

#[test]
fn par_all_false() {
    let all = ParallelQueryBuilder::from(vec![1, 2, 3]).par_all(|x| *x > 1);
    assert!(!all);
}

#[test]
fn par_all_empty() {
    // vacuously true
    let all = ParallelQueryBuilder::from(vec![0i32; 0]).par_all(|x| *x > 100);
    assert!(all);
}

// ── par_group_by ─────────────────────────────────────────────────────────────

#[test]
fn par_group_by_basic() {
    let groups: HashMap<i32, Vec<i32>> =
        ParallelQueryBuilder::from(vec![1, 2, 3, 4, 5]).par_group_by(|x| x % 2);
    assert_eq!(groups[&0].len(), 2); // 2, 4
    assert_eq!(groups[&1].len(), 3); // 1, 3, 5
}

#[test]
fn par_group_by_empty() {
    let groups: HashMap<i32, Vec<i32>> =
        ParallelQueryBuilder::from(vec![]).par_group_by(|x: &i32| x % 2);
    assert!(groups.is_empty());
}

#[test]
fn par_group_by_matches_sequential() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let seq: HashMap<i32, Vec<i32>> = QueryBuilder::from(data.clone())
        .where_(|_| true)
        .group_by(|x| x % 3);
    let par: HashMap<i32, Vec<i32>> = ParallelQueryBuilder::from(data).par_group_by(|x| x % 3);

    assert_eq!(seq.len(), par.len());
    for (k, mut seq_v) in seq {
        let mut par_v = par[&k].clone();
        seq_v.sort();
        par_v.sort();
        assert_eq!(seq_v, par_v, "mismatch for key {k}");
    }
}

// ── result parity: parallel vs sequential ───────────────────────────────────

#[test]
fn filter_results_match_sequential() {
    let data: Vec<i32> = (1..=100).collect();

    let mut seq: Vec<i32> = QueryBuilder::from(data.clone())
        .where_(|x| x % 7 == 0)
        .collect();
    let mut par: Vec<i32> = ParallelQueryBuilder::from(data)
        .par_where(|x| x % 7 == 0)
        .collect();

    seq.sort();
    par.sort();
    assert_eq!(seq, par);
}

#[test]
fn sort_results_match_sequential() {
    let data: Vec<i32> = (1..=20).rev().collect();

    let seq: Vec<i32> = QueryBuilder::from(data.clone())
        .where_(|_| true)
        .order_by(|x| *x)
        .collect();
    let par: Vec<i32> = ParallelQueryBuilder::from(data)
        .par_order_by(|x| *x)
        .collect();

    assert_eq!(seq, par);
}

#[test]
fn min_max_match_sequential() {
    let data: Vec<i32> = (1..=50).collect();

    let seq_min = QueryBuilder::from(data.clone()).where_(|_| true).min();
    let seq_max = QueryBuilder::from(data.clone()).where_(|_| true).max();
    let par_min = ParallelQueryBuilder::from(data.clone()).par_min();
    let par_max = ParallelQueryBuilder::from(data).par_max();

    assert_eq!(seq_min, par_min);
    assert_eq!(seq_max, par_max);
}

// ── large collection smoke test ──────────────────────────────────────────────

#[test]
fn large_collection_filter_and_sum() {
    let data: Vec<i64> = (1..=100_000).collect();

    let par_sum: i64 = ParallelQueryBuilder::from(data.clone())
        .par_where(|x| x % 2 == 0)
        .par_sum();

    // Sum of even numbers 1..=100_000 = sum of 2+4+...+100_000 = 2*(1+2+...+50_000)
    let expected: i64 = 2 * (50_000 * 50_001 / 2);
    assert_eq!(par_sum, expected);
}

#[test]
fn large_collection_group_by_count() {
    let data: Vec<i32> = (0..10_000).collect();
    let groups: HashMap<i32, Vec<i32>> = ParallelQueryBuilder::from(data).par_group_by(|x| x % 10);
    assert_eq!(groups.len(), 10);
    for v in groups.values() {
        assert_eq!(v.len(), 1_000);
    }
}
