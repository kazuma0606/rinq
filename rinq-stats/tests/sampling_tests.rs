// rinq-stats/tests/sampling_tests.rs
// Phase B3 integration tests for SamplingExt.

use rand::SeedableRng;
use rand::rngs::SmallRng;
use rinq::QueryBuilder;
use rinq_stats::SamplingExt;
use std::collections::HashSet;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

// ── sample_fraction ───────────────────────────────────────────────────────────

#[test]
fn sample_fraction_size_within_bound() {
    let data: Vec<i32> = (1..=100).collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .sample_fraction(&mut rng(0), 0.1)
        .collect();
    // ceil(100 * 0.1) = 10
    assert!(sample.len() <= 10, "len={}", sample.len());
}

#[test]
fn sample_fraction_zero_returns_empty() {
    let data: Vec<i32> = (1..=100).collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .sample_fraction(&mut rng(1), 0.0)
        .collect();
    assert!(sample.is_empty());
}

#[test]
fn sample_fraction_one_returns_all() {
    let data: Vec<i32> = (1..=10).collect();
    let sample: Vec<i32> = QueryBuilder::from(data.clone())
        .sample_fraction(&mut rng(2), 1.0)
        .collect();
    assert_eq!(sample.len(), data.len());
}

#[test]
fn sample_fraction_elements_from_source() {
    let data: Vec<i32> = (1..=50).collect();
    let source: HashSet<i32> = data.iter().cloned().collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .sample_fraction(&mut rng(3), 0.2)
        .collect();
    assert!(
        sample.iter().all(|x| source.contains(x)),
        "sample contains elements not in source"
    );
}

#[test]
fn sample_fraction_empty_input() {
    let sample: Vec<i32> = QueryBuilder::from(vec![])
        .sample_fraction(&mut rng(4), 0.5)
        .collect();
    assert!(sample.is_empty());
}

// ── sample_n ─────────────────────────────────────────────────────────────────

#[test]
fn sample_n_exact_count() {
    let data: Vec<i32> = (1..=20).collect();
    let sample: Vec<i32> = QueryBuilder::from(data).sample_n(&mut rng(10), 5).collect();
    assert_eq!(sample.len(), 5);
}

#[test]
fn sample_n_larger_than_source() {
    // k > n → return all n elements
    let data: Vec<i32> = (1..=5).collect();
    let sample: Vec<i32> = QueryBuilder::from(data.clone())
        .sample_n(&mut rng(11), 100)
        .collect();
    assert_eq!(sample.len(), data.len());
}

#[test]
fn sample_n_zero() {
    let sample: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .sample_n(&mut rng(12), 0)
        .collect();
    assert!(sample.is_empty());
}

#[test]
fn sample_n_elements_from_source() {
    let data: Vec<i32> = (100..200).collect();
    let source: HashSet<i32> = data.iter().cloned().collect();
    let sample: Vec<i32> = QueryBuilder::from(data).sample_n(&mut rng(13), 30).collect();
    assert!(sample.iter().all(|x| source.contains(x)));
}

#[test]
fn sample_n_no_duplicates_when_k_le_n() {
    let data: Vec<i32> = (1..=50).collect();
    let sample: Vec<i32> = QueryBuilder::from(data).sample_n(&mut rng(14), 10).collect();
    let unique: HashSet<i32> = sample.iter().cloned().collect();
    assert_eq!(sample.len(), unique.len(), "unexpected duplicates in reservoir sample");
}

#[test]
fn sample_n_empty_source() {
    let sample: Vec<i32> = QueryBuilder::from(vec![])
        .sample_n(&mut rng(15), 5)
        .collect();
    assert!(sample.is_empty());
}

// ── stratified_sample ─────────────────────────────────────────────────────────

#[test]
fn stratified_sample_per_group_limit() {
    // 10 even + 10 odd numbers, sample max 2 per group
    let data: Vec<i32> = (1..=20).collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .stratified_sample(&mut rng(20), |x| x % 2, 2)
        .collect();
    // 2 groups × 2 → at most 4
    assert!(sample.len() <= 4, "len={}", sample.len());
}

#[test]
fn stratified_sample_covers_all_groups() {
    // 4 groups of 10 each, sample 3 per group → should get items from each
    let data: Vec<i32> = (0..40).collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .stratified_sample(&mut rng(21), |x| x % 4, 3)
        .collect();
    // All 4 groups should be represented (3 each)
    let groups: HashSet<i32> = sample.iter().map(|x| x % 4).collect();
    assert_eq!(groups.len(), 4, "not all groups represented: {groups:?}");
    assert_eq!(sample.len(), 12); // exactly 4 × 3
}

#[test]
fn stratified_sample_elements_from_source() {
    let data: Vec<i32> = (1..=30).collect();
    let source: HashSet<i32> = data.iter().cloned().collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .stratified_sample(&mut rng(22), |x| x % 3, 2)
        .collect();
    assert!(sample.iter().all(|x| source.contains(x)));
}

#[test]
fn stratified_sample_empty_source() {
    let sample: Vec<i32> = QueryBuilder::from(vec![])
        .stratified_sample(&mut rng(23), |x: &i32| x % 2, 5)
        .collect();
    assert!(sample.is_empty());
}

// ── bootstrap_sample ──────────────────────────────────────────────────────────

#[test]
fn bootstrap_sample_exact_count() {
    let data = vec![1, 2, 3, 4, 5];
    let sample: Vec<i32> = QueryBuilder::from(data).bootstrap_sample(&mut rng(30), 20);
    assert_eq!(sample.len(), 20);
}

#[test]
fn bootstrap_sample_only_source_elements() {
    let data = vec![10, 20, 30];
    let sample: Vec<i32> = QueryBuilder::from(data.clone())
        .bootstrap_sample(&mut rng(31), 100);
    assert!(sample.iter().all(|x| data.contains(x)));
}

#[test]
fn bootstrap_sample_can_repeat() {
    // With 100 draws from 3 elements, duplicates are expected
    let data = vec![1, 2, 3];
    let sample: Vec<i32> = QueryBuilder::from(data).bootstrap_sample(&mut rng(32), 100);
    let unique: HashSet<i32> = sample.iter().cloned().collect();
    // All 3 values should appear
    assert_eq!(unique.len(), 3);
    // Should have duplicates
    assert!(sample.len() > unique.len());
}

#[test]
fn bootstrap_sample_empty_source() {
    let sample: Vec<i32> = QueryBuilder::from(vec![]).bootstrap_sample(&mut rng(33), 10);
    assert!(sample.is_empty());
}

#[test]
fn bootstrap_sample_zero_n() {
    let sample: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .bootstrap_sample(&mut rng(34), 0);
    assert!(sample.is_empty());
}

// ── chaining with rinq operations ─────────────────────────────────────────────

#[test]
fn sample_n_after_filter() {
    let data: Vec<i32> = (1..=100).collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .where_(|x| *x % 2 == 0) // 50 even numbers
        .sample_n(&mut rng(40), 10)
        .collect();
    assert_eq!(sample.len(), 10);
    assert!(sample.iter().all(|x| x % 2 == 0));
}

#[test]
fn sample_fraction_after_sort() {
    let data: Vec<i32> = (1..=20).collect();
    let sample: Vec<i32> = QueryBuilder::from(data)
        .where_(|_| true)
        .order_by(|x| *x)
        .sample_fraction(&mut rng(41), 0.25)
        .collect();
    assert!(sample.len() <= 5);
}
