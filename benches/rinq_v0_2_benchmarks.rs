// benches/rinq_v0_2_benchmarks.rs
// RINQ v0.2 Performance Benchmarks
//
// Validates zero-cost abstraction principle by comparing RINQ operations
// against manual implementations

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rusted_ca::domain::rinq::QueryBuilder;
use std::collections::HashMap;

// Benchmark data generators
fn generate_data(size: usize) -> Vec<i32> {
    (0..size as i32).collect()
}

fn generate_mixed_data(size: usize) -> Vec<i32> {
    (0..size as i32).map(|i| i % 100).collect()
}

// ============================================================================
// Phase 1: Numeric Aggregations Benchmarks
// ============================================================================

fn bench_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum");

    for size in [100, 1000, 10000] {
        let data = generate_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: i32 = QueryBuilder::from(data.clone()).sum();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let _: i32 = black_box(data.iter().sum());
            });
        });
    }

    group.finish();
}

fn bench_average(c: &mut Criterion) {
    let mut group = c.benchmark_group("average");

    for size in [100, 1000, 10000] {
        let data = generate_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _ = QueryBuilder::from(data.clone()).average();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                if data.is_empty() {
                    None
                } else {
                    let sum: f64 = black_box(data.iter().map(|x| *x as f64).sum());
                    Some(sum / data.len() as f64)
                }
            });
        });
    }

    group.finish();
}

fn bench_min_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("min_max");

    for size in [100, 1000, 10000] {
        let data = generate_mixed_data(size);

        group.bench_with_input(BenchmarkId::new("rinq_min", size), &data, |b, data| {
            b.iter(|| {
                let _ = QueryBuilder::from(data.clone()).min();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual_min", size), &data, |b, data| {
            b.iter(|| {
                let _ = black_box(data.iter().min());
            });
        });

        group.bench_with_input(BenchmarkId::new("rinq_max", size), &data, |b, data| {
            b.iter(|| {
                let _ = QueryBuilder::from(data.clone()).max();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual_max", size), &data, |b, data| {
            b.iter(|| {
                let _ = black_box(data.iter().max());
            });
        });
    }

    group.finish();
}

// ============================================================================
// Phase 2: Grouping Benchmarks
// ============================================================================

fn bench_group_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("group_by");

    for size in [100, 1000, 10000] {
        let data = generate_mixed_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: HashMap<i32, Vec<i32>> =
                    QueryBuilder::from(data.clone()).group_by(|x| x % 10);
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let mut groups: HashMap<i32, Vec<i32>> = HashMap::new();
                for item in data {
                    let key = item % 10;
                    groups.entry(key).or_insert_with(Vec::new).push(*item);
                }
                black_box(groups);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Phase 3: Deduplication Benchmarks
// ============================================================================

fn bench_distinct(c: &mut Criterion) {
    let mut group = c.benchmark_group("distinct");

    for size in [100, 1000, 10000] {
        let data = generate_mixed_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: Vec<i32> = QueryBuilder::from(data.clone()).distinct().collect();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                use std::collections::HashSet;
                let mut seen = HashSet::new();
                let result: Vec<i32> = data.iter().filter(|x| seen.insert(**x)).copied().collect();
                black_box(result);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Phase 4: Sequence Transformations Benchmarks
// ============================================================================

fn bench_reverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("reverse");

    for size in [100, 1000, 10000] {
        let data = generate_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: Vec<i32> = QueryBuilder::from(data.clone()).reverse().collect();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let mut result = data.clone();
                result.reverse();
                black_box(result);
            });
        });
    }

    group.finish();
}

fn bench_chunk(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk");

    for size in [100, 1000, 10000] {
        let data = generate_data(size);
        let chunk_size = 10;

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: Vec<Vec<i32>> = QueryBuilder::from(data.clone()).chunk(chunk_size).collect();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let result: Vec<Vec<i32>> = data.chunks(chunk_size).map(|c| c.to_vec()).collect();
                black_box(result);
            });
        });
    }

    group.finish();
}

fn bench_window(c: &mut Criterion) {
    let mut group = c.benchmark_group("window");

    for size in [100, 1000, 5000] {
        let data = generate_data(size);
        let window_size = 5;

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: Vec<Vec<i32>> = QueryBuilder::from(data.clone())
                    .window(window_size)
                    .collect();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let result: Vec<Vec<i32>> = data.windows(window_size).map(|w| w.to_vec()).collect();
                black_box(result);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Phase 5: Collection Combinations Benchmarks
// ============================================================================

fn bench_enumerate(c: &mut Criterion) {
    let mut group = c.benchmark_group("enumerate");

    for size in [100, 1000, 10000] {
        let data = generate_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _: Vec<(usize, i32)> = QueryBuilder::from(data.clone()).enumerate().collect();
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let result: Vec<(usize, i32)> = data.iter().copied().enumerate().collect();
                black_box(result);
            });
        });
    }

    group.finish();
}

fn bench_partition(c: &mut Criterion) {
    let mut group = c.benchmark_group("partition");

    for size in [100, 1000, 10000] {
        let data = generate_mixed_data(size);

        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, data| {
            b.iter(|| {
                let _ = QueryBuilder::from(data.clone()).partition(|x| *x < 50);
            });
        });

        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, data| {
            b.iter(|| {
                let mut left = Vec::new();
                let mut right = Vec::new();
                for item in data {
                    if *item < 50 {
                        left.push(*item);
                    } else {
                        right.push(*item);
                    }
                }
                black_box((left, right));
            });
        });
    }

    group.finish();
}

// ============================================================================
// Complex Chain Benchmarks
// ============================================================================

fn bench_complex_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_chain");

    let data = generate_mixed_data(1000);

    group.bench_function("rinq_v0.2_chain", |b| {
        b.iter(|| {
            let _: i32 = QueryBuilder::from(data.clone())
                .where_(|x| *x > 10)
                .distinct()
                .order_by(|x| *x)
                .take(50)
                .sum();
        });
    });

    group.bench_function("manual_chain", |b| {
        b.iter(|| {
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            let mut filtered_distinct: Vec<i32> = data
                .iter()
                .filter(|x| **x > 10 && seen.insert(**x))
                .copied()
                .collect();
            filtered_distinct.sort();
            let sum: i32 = filtered_distinct.iter().take(50).sum();
            black_box(sum);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sum,
    bench_average,
    bench_min_max,
    bench_group_by,
    bench_distinct,
    bench_reverse,
    bench_chunk,
    bench_window,
    bench_enumerate,
    bench_partition,
    bench_complex_chain
);
criterion_main!(benches);
