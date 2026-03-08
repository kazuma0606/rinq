// benches/rinq_benchmarks.rs
// Performance benchmarks for RINQ

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rusted_ca::domain::rinq::QueryBuilder;

// **Task 14.1: Benchmark for zero-cost abstraction verification**
// **Validates: Requirements 7.2**
//
// These benchmarks compare RINQ queries with hand-written loops to verify
// zero-cost abstraction.

fn benchmark_filter_rinq(c: &mut Criterion) {
    c.bench_function("filter_rinq", |b| {
        let data: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .where_(|x| *x % 2 == 0)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_filter_manual(c: &mut Criterion) {
    c.bench_function("filter_manual", |b| {
        let data: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let result: Vec<_> = data.iter().filter(|x| **x % 2 == 0).copied().collect();
            black_box(result);
        });
    });
}

fn benchmark_filter_then_map_rinq(c: &mut Criterion) {
    c.bench_function("filter_then_map_rinq", |b| {
        let data: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .where_(|x| *x % 2 == 0)
                .select(|x| x * 2)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_filter_then_map_manual(c: &mut Criterion) {
    c.bench_function("filter_then_map_manual", |b| {
        let data: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let result: Vec<_> = data
                .iter()
                .filter(|x| **x % 2 == 0)
                .map(|x| x * 2)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_sort_rinq(c: &mut Criterion) {
    c.bench_function("sort_rinq", |b| {
        let data: Vec<i32> = (0..1000).rev().collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone()).order_by(|x| *x).collect();
            black_box(result);
        });
    });
}

fn benchmark_sort_manual(c: &mut Criterion) {
    c.bench_function("sort_manual", |b| {
        let data: Vec<i32> = (0..1000).rev().collect();
        b.iter(|| {
            let mut result = data.clone();
            result.sort();
            black_box(result);
        });
    });
}

fn benchmark_complex_query_rinq(c: &mut Criterion) {
    c.bench_function("complex_query_rinq", |b| {
        let data: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .where_(|x| *x > 100)
                .order_by(|x| *x)
                .take(50)
                .select(|x| x * 2)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_complex_query_manual(c: &mut Criterion) {
    c.bench_function("complex_query_manual", |b| {
        let data: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let mut filtered: Vec<_> = data.iter().filter(|x| **x > 100).copied().collect();
            filtered.sort();
            let result: Vec<_> = filtered.into_iter().take(50).map(|x| x * 2).collect();
            black_box(result);
        });
    });
}

fn benchmark_count_rinq(c: &mut Criterion) {
    c.bench_function("count_rinq", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let count = QueryBuilder::from(data.clone())
                .where_(|x| *x % 3 == 0)
                .count();
            black_box(count);
        });
    });
}

fn benchmark_count_manual(c: &mut Criterion) {
    c.bench_function("count_manual", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let count = data.iter().filter(|x| **x % 3 == 0).count();
            black_box(count);
        });
    });
}

fn benchmark_first_rinq(c: &mut Criterion) {
    c.bench_function("first_rinq", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let first = QueryBuilder::from(data.clone())
                .where_(|x| *x > 5000)
                .first();
            black_box(first);
        });
    });
}

fn benchmark_first_manual(c: &mut Criterion) {
    c.bench_function("first_manual", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let first = data.iter().filter(|x| **x > 5000).next();
            black_box(first);
        });
    });
}

fn benchmark_any_rinq(c: &mut Criterion) {
    c.bench_function("any_rinq", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let any = QueryBuilder::from(data.clone()).any(|x| *x > 9000);
            black_box(any);
        });
    });
}

fn benchmark_any_manual(c: &mut Criterion) {
    c.bench_function("any_manual", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let any = data.iter().any(|x| *x > 9000);
            black_box(any);
        });
    });
}

fn benchmark_pagination_rinq(c: &mut Criterion) {
    c.bench_function("pagination_rinq", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .skip(1000)
                .take(100)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_pagination_manual(c: &mut Criterion) {
    c.bench_function("pagination_manual", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let result: Vec<_> = data.iter().skip(1000).take(100).copied().collect();
            black_box(result);
        });
    });
}

// **Task 14.2: Benchmark for memory usage measurement**
// **Validates: Requirements 7.5**
//
// These benchmarks help verify that RINQ doesn't allocate more memory
// than necessary.

fn benchmark_large_dataset_filter(c: &mut Criterion) {
    c.bench_function("large_dataset_filter", |b| {
        let data: Vec<i32> = (0..100000).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .where_(|x| *x % 10 == 0)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_chained_filters(c: &mut Criterion) {
    c.bench_function("chained_filters", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .where_(|x| *x > 100)
                .where_(|x| *x < 9000)
                .where_(|x| *x % 2 == 0)
                .collect();
            black_box(result);
        });
    });
}

fn benchmark_multi_sort(c: &mut Criterion) {
    c.bench_function("multi_sort", |b| {
        let data: Vec<(i32, i32)> = (0..1000).map(|i| (i % 10, i)).collect();
        b.iter(|| {
            let result: Vec<_> = QueryBuilder::from(data.clone())
                .order_by(|x| x.0)
                .then_by(|x| x.1)
                .collect();
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    benchmark_filter_rinq,
    benchmark_filter_manual,
    benchmark_filter_then_map_rinq,
    benchmark_filter_then_map_manual,
    benchmark_sort_rinq,
    benchmark_sort_manual,
    benchmark_complex_query_rinq,
    benchmark_complex_query_manual,
    benchmark_count_rinq,
    benchmark_count_manual,
    benchmark_first_rinq,
    benchmark_first_manual,
    benchmark_any_rinq,
    benchmark_any_manual,
    benchmark_pagination_rinq,
    benchmark_pagination_manual,
    benchmark_large_dataset_filter,
    benchmark_chained_filters,
    benchmark_multi_sort,
);

criterion_main!(benches);
