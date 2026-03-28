// benches/rinq_v4_benchmarks.rs
// Benchmarks for operators added in v4 (Phase 4B–4D).
//
// Each benchmark compares rinq against an equivalent hand-written iterator
// expression to verify zero-cost abstraction.
//
// Run:  cargo bench --bench rinq_v4_benchmarks

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rinq::QueryBuilder;
use std::sync::Arc;

// ── helpers ──────────────────────────────────────────────────────────────────

fn ints(n: usize) -> Vec<i32> {
    (0..n as i32).collect()
}

fn floats(n: usize) -> Vec<f64> {
    (0..n).map(|i| (i % 100) as f64 + 0.5).collect()
}

fn str_data(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("{}", i % 20)).collect()
}

// ── functional ───────────────────────────────────────────────────────────────

fn bench_scan_cumulative_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/scan_cumulative_sum");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .scan(0i32, |acc, x| acc + x)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("std_iter", size), &data, |b, d| {
            b.iter(|| {
                let mut acc = 0i32;
                let result = d
                    .iter()
                    .map(|&x| { acc += x; acc })
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_chunk_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/chunk_by");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .chunk_by(|&x| x % 10)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, d| {
            b.iter(|| {
                let mut result: Vec<Vec<i32>> = Vec::new();
                let mut current: Vec<i32> = Vec::new();
                let mut last_key: Option<i32> = None;
                for &x in d.iter() {
                    let key = x % 10;
                    if last_key == Some(key) || last_key.is_none() {
                        current.push(x);
                    } else {
                        result.push(std::mem::take(&mut current));
                        current.push(x);
                    }
                    last_key = Some(key);
                }
                if !current.is_empty() { result.push(current); }
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_dedup_consecutive(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/dedup_consecutive");
    for size in [1_000usize, 10_000] {
        // data with runs of duplicates
        let data: Vec<i32> = (0..size as i32).flat_map(|x| vec![x, x]).collect();
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .dedup()
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("manual_dedup", size), &data, |b, d| {
            b.iter(|| {
                let mut last: Option<i32> = None;
                let result = d.iter().filter(|&&x| {
                    if last == Some(x) { false } else { last = Some(x); true }
                }).copied().collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_zip_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/zip_with_add");
    for size in [1_000usize, 10_000] {
        let a = ints(size);
        let b = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &(a.clone(), b.clone()), |bench, (a, b)| {
            bench.iter(|| {
                let result = QueryBuilder::from(a.clone())
                    .zip_with(b.clone(), |x, y| x + y)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("std_zip_map", size), &(a.clone(), b.clone()), |bench, (a, b)| {
            bench.iter(|| {
                let result = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_pairwise(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/pairwise");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .pairwise()
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("windows_2", size), &data, |b, d| {
            b.iter(|| {
                let result = d
                    .windows(2)
                    .map(|w| (w[0], w[1]))
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_intersperse(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/intersperse");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .intersperse(-1)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("manual", size), &data, |b, d| {
            b.iter(|| {
                let mut result = Vec::with_capacity(d.len() * 2);
                for (i, &x) in d.iter().enumerate() {
                    if i > 0 { result.push(-1); }
                    result.push(x);
                }
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_min_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/min_max");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq_single_pass", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone()).min_max();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("std_two_pass", size), &data, |b, d| {
            b.iter(|| {
                let mn = d.iter().copied().min();
                let mx = d.iter().copied().max();
                black_box((mn, mx));
            });
        });
    }
    group.finish();
}

fn bench_filter_map_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/filter_map_parse");
    for size in [1_000usize, 10_000] {
        let data = str_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .filter_map(|s: String| s.parse::<i32>().ok())
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("std_iter", size), &data, |b, d| {
            b.iter(|| {
                let result = d.iter()
                    .filter_map(|s| s.parse::<i32>().ok())
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_step_by_2(c: &mut Criterion) {
    let mut group = c.benchmark_group("functional/step_by_2");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .step_by(2)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("std_iter", size), &data, |b, d| {
            b.iter(|| {
                let result = d.iter().step_by(2).copied().collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

// ── window analytics ─────────────────────────────────────────────────────────

fn bench_running_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("window/running_sum");
    for size in [1_000usize, 10_000] {
        let data: Vec<i32> = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .running_sum()
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("manual_fold", size), &data, |b, d| {
            b.iter(|| {
                let mut acc = 0i32;
                let result = d.iter().map(|&x| { acc += x; acc }).collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_moving_average_10(c: &mut Criterion) {
    let mut group = c.benchmark_group("window/moving_average_10");
    for size in [1_000usize, 10_000] {
        let data = floats(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .moving_average(10)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("windows_map", size), &data, |b, d| {
            b.iter(|| {
                let result = d.windows(10).map(|w| {
                    let sum: f64 = w.iter().sum();
                    Some(sum / 10.0)
                }).collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_rank_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("window/rank_by");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .rank_by(|&x| x)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("sort_enumerate", size), &data, |b, d| {
            b.iter(|| {
                let mut sorted = d.clone();
                sorted.sort();
                let result = sorted.into_iter().enumerate().collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_lag_1(c: &mut Criterion) {
    let mut group = c.benchmark_group("window/lag_1");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .lag(1)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("zip_shifted", size), &data, |b, d| {
            b.iter(|| {
                let shifted = std::iter::once(None).chain(d.iter().copied().map(Some));
                let result = shifted.zip(d.iter().copied()).collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_lead_1(c: &mut Criterion) {
    let mut group = c.benchmark_group("window/lead_1");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .lead(1)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("zip_shifted", size), &data, |b, d| {
            b.iter(|| {
                let shifted = d.iter().copied().skip(1).map(Some).chain(std::iter::once(None));
                let result = d.iter().copied().zip(shifted).collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

// ── lifecycle ─────────────────────────────────────────────────────────────────

fn bench_tap_each_noop(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/tap_each_noop");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("with_tap_each", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .tap_each(|_| {})
                    .where_(|&x| x % 2 == 0)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("without_tap_each", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .where_(|&x| x % 2 == 0)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_tap_collect_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/tap_collect_vec");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("with_tap_collect", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .tap_collect(|_| {})
                    .where_(|&x| x % 2 == 0)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("without_tap_collect", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .where_(|&x| x % 2 == 0)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_pipe_identity(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/pipe_identity");
    for size in [1_000usize, 10_000] {
        let data = ints(size);
        group.bench_with_input(BenchmarkId::new("with_pipe", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .where_(|_| true)
                    .pipe(|q| q)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("without_pipe", size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .where_(|_| true)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_from_arc_cloned(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/from_arc_cloned");
    for size in [1_000usize, 10_000] {
        let arc = Arc::new(ints(size));
        group.bench_with_input(BenchmarkId::new("arc_cloned", size), &arc, |b, a| {
            b.iter(|| {
                let result = QueryBuilder::from_arc_cloned(a)
                    .where_(|&x| x % 2 == 0)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        let direct = ints(size);
        group.bench_with_input(BenchmarkId::new("direct_clone", size), &direct, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .where_(|&x| x % 2 == 0)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

// ── generation ────────────────────────────────────────────────────────────────

fn bench_unfold_fib_take100(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation/unfold_fib_take100");
    group.bench_function("rinq_unfold_bounded", |b| {
        b.iter(|| {
            let result = QueryBuilder::unfold_bounded(
                (0u64, 1u64),
                |(a, b)| Some((a, (b, a + b))),
                100,
            )
            .collect::<Vec<_>>();
            black_box(result);
        });
    });
    group.bench_function("manual_vec", |b| {
        b.iter(|| {
            let mut result = Vec::with_capacity(100);
            let (mut a, mut b) = (0u64, 1u64);
            for _ in 0..100 {
                result.push(a);
                let next = a + b;
                a = b;
                b = next;
            }
            black_box(result);
        });
    });
    group.finish();
}

fn bench_cycle_take1000(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation/cycle_take1000");
    for base_size in [10usize, 100] {
        let data = ints(base_size);
        group.bench_with_input(BenchmarkId::new("rinq_cycle", base_size), &data, |b, d| {
            b.iter(|| {
                let result = QueryBuilder::from(d.clone())
                    .cycle()
                    .take(1000)
                    .collect::<Vec<_>>();
                black_box(result);
            });
        });
        group.bench_with_input(BenchmarkId::new("std_cycle", base_size), &data, |b, d| {
            b.iter(|| {
                let result = d.iter().copied().cycle().take(1000).collect::<Vec<_>>();
                black_box(result);
            });
        });
    }
    group.finish();
}

// ── criterion groups ──────────────────────────────────────────────────────────

criterion_group!(
    functional,
    bench_scan_cumulative_sum,
    bench_chunk_by,
    bench_dedup_consecutive,
    bench_zip_with,
    bench_pairwise,
    bench_intersperse,
    bench_min_max,
    bench_filter_map_parse,
    bench_step_by_2,
);

criterion_group!(
    window,
    bench_running_sum,
    bench_moving_average_10,
    bench_rank_by,
    bench_lag_1,
    bench_lead_1,
);

criterion_group!(
    lifecycle,
    bench_tap_each_noop,
    bench_tap_collect_vec,
    bench_pipe_identity,
    bench_from_arc_cloned,
);

criterion_group!(
    generation,
    bench_unfold_fib_take100,
    bench_cycle_take1000,
);

criterion_main!(functional, window, lifecycle, generation);
