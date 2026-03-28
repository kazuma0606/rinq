// rinq-stats/benches/rinq_stats_benchmarks.rs
//
// Criterion benchmarks for the rinq-stats extension crate.
// Groups: descriptive, sampling, timeseries, outliers, validation

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rinq::QueryBuilder;
use rinq_stats::{OutlierExt, SamplingExt, StatisticsExt, TimeSeriesExt, ValidationExt};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_f64_data(n: usize) -> Vec<f64> {
    (0..n).map(|i| (i as f64) * 0.1 + 1.0).collect()
}

fn make_noisy_data(n: usize) -> Vec<f64> {
    let mut v: Vec<f64> = (0..n).map(|i| (i % 100) as f64).collect();
    // Inject a few outliers
    if n > 10 {
        v[n / 4] = 1_000.0;
        v[n / 2] = -1_000.0;
    }
    v
}

fn make_i32_data(n: usize) -> Vec<i32> {
    (0..n as i32).map(|i| i % 50).collect()
}

const SIZES: &[usize] = &[1_000, 10_000];

// ---------------------------------------------------------------------------
// Group: descriptive — StatisticsExt
// ---------------------------------------------------------------------------

fn bench_variance(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/variance");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| QueryBuilder::from(black_box(d.clone())).variance())
        });
    }
    group.finish();
}

fn bench_std_dev(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/std_dev");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| QueryBuilder::from(black_box(d.clone())).std_dev())
        });
    }
    group.finish();
}

fn bench_median(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/median");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| QueryBuilder::from(black_box(d.clone())).median())
        });
    }
    group.finish();
}

fn bench_percentile_p95(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/percentile_p95");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| QueryBuilder::from(black_box(d.clone())).percentile(0.95))
        });
    }
    group.finish();
}

fn bench_mode_i32(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/mode_i32");
    for &size in SIZES {
        let data = make_i32_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| QueryBuilder::from(black_box(d.clone())).mode())
        });
    }
    group.finish();
}

fn bench_skewness(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/skewness");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| QueryBuilder::from(black_box(d.clone())).skewness())
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group: sampling — SamplingExt
// ---------------------------------------------------------------------------

fn bench_sample_n(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/sample_n");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let mut rng = SmallRng::seed_from_u64(42);
                let _: Vec<f64> = QueryBuilder::from(black_box(d.clone()))
                    .sample_n(&mut rng, size / 10)
                    .collect();
            })
        });
    }
    group.finish();
}

fn bench_sample_fraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/sample_fraction");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                let mut rng = SmallRng::seed_from_u64(0);
                let _: Vec<f64> = QueryBuilder::from(black_box(d.clone()))
                    .sample_fraction(&mut rng, 0.1)
                    .collect();
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group: timeseries — TimeSeriesExt
// ---------------------------------------------------------------------------

fn bench_ema(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/ema_alpha0_1");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                QueryBuilder::from(black_box(d.clone()))
                    .exponential_moving_average(black_box(0.1))
            })
        });
    }
    group.finish();
}

fn bench_bollinger_bands(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/bollinger_bands_w20");
    for &size in SIZES {
        let data = make_f64_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                QueryBuilder::from(black_box(d.clone()))
                    .bollinger_bands(black_box(20), black_box(2.0))
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group: outliers — OutlierExt
// ---------------------------------------------------------------------------

fn bench_remove_outliers_zscore(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/remove_outliers_zscore");
    for &size in SIZES {
        let data = make_noisy_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                QueryBuilder::from(black_box(d.clone()))
                    .remove_outliers_zscore(black_box(2.0))
            })
        });
    }
    group.finish();
}

fn bench_remove_outliers_iqr(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/remove_outliers_iqr");
    for &size in SIZES {
        let data = make_noisy_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                QueryBuilder::from(black_box(d.clone())).remove_outliers_iqr()
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group: validation — ValidationExt
// ---------------------------------------------------------------------------

fn bench_validation_single_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/validation_single_rule");
    for &size in SIZES {
        let data = make_i32_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                QueryBuilder::from(black_box(d.clone()))
                    .validate(|x| *x >= 0, "non_negative", "must be >= 0")
                    .collect_validated()
            })
        });
    }
    group.finish();
}

fn bench_validation_multi_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats/validation_multi_rule");
    for &size in SIZES {
        let data = make_i32_data(size);
        group.bench_with_input(BenchmarkId::new("rinq", size), &data, |b, d| {
            b.iter(|| {
                QueryBuilder::from(black_box(d.clone()))
                    .validate(|x| *x >= 0, "non_negative", "must be >= 0")
                    .validate(|x| *x < 100, "bounded", "must be < 100")
                    .validate(|x| *x % 2 == 0, "even", "must be even")
                    .collect_validated()
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// criterion_group / criterion_main
// ---------------------------------------------------------------------------

criterion_group!(
    descriptive,
    bench_variance,
    bench_std_dev,
    bench_median,
    bench_percentile_p95,
    bench_mode_i32,
    bench_skewness,
);

criterion_group!(sampling, bench_sample_n, bench_sample_fraction,);

criterion_group!(timeseries, bench_ema, bench_bollinger_bands,);

criterion_group!(
    outliers,
    bench_remove_outliers_zscore,
    bench_remove_outliers_iqr,
);

criterion_group!(
    validation,
    bench_validation_single_rule,
    bench_validation_multi_rule,
);

criterion_main!(descriptive, sampling, timeseries, outliers, validation);
