//! RINQ v2.0 E2E シナリオテスト
//!
//! 現実に近いユースケースで複合クエリの動作を検証する。
//! 個々のメソッドではなく「メソッドが連鎖したときの振る舞い」に焦点を当てる。

use rinq::{QueryBuilder, RinqError};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────
// 共通データ型
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct Order {
    product_id: u32,
    category: &'static str,
    amount: f64,
    is_shipped: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
struct LogEntry {
    level: LogLevel,
    message: &'static str,
    user_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    name: &'static str,
    age: u32,
    active: bool,
}

#[derive(Debug, Clone)]
struct SensorReading {
    sensor_id: u8,
    temperature: f64,
}

// ─────────────────────────────────────────────────────────────────
// Scenario 1: 売上データ分析パイプライン
// ─────────────────────────────────────────────────────────────────

fn sample_orders() -> Vec<Order> {
    vec![
        Order { product_id: 1, category: "Electronics", amount: 299.99, is_shipped: true  },
        Order { product_id: 2, category: "Books",       amount:  19.99, is_shipped: true  },
        Order { product_id: 3, category: "Electronics", amount: 499.99, is_shipped: false }, // 未発送
        Order { product_id: 4, category: "Clothing",    amount:  59.99, is_shipped: true  },
        Order { product_id: 5, category: "Books",       amount:  14.99, is_shipped: true  },
        Order { product_id: 6, category: "Electronics", amount: 899.99, is_shipped: true  },
        Order { product_id: 7, category: "Clothing",    amount:  89.99, is_shipped: true  },
        Order { product_id: 8, category: "Books",       amount:  24.99, is_shipped: false }, // 未発送
        Order { product_id: 9, category: "Electronics", amount: 149.99, is_shipped: true  },
    ]
}

#[test]
fn scenario1_sales_analysis_pipeline() {
    let orders = sample_orders();

    // 発送済みのカテゴリ別売上合計を計算し、トップ 2 カテゴリを特定する
    let category_totals: HashMap<&str, f64> = QueryBuilder::from(orders.clone())
        .where_(|o| o.is_shipped)
        .group_by_aggregate(
            |o| o.category,
            |group| group.iter().map(|o| o.amount).sum::<f64>(),
        );

    // Electronics: 299.99 + 899.99 + 149.99 = 1349.97（未発送の499.99は除外）
    // Books: 19.99 + 14.99 = 34.98（未発送の24.99は除外）
    // Clothing: 59.99 + 89.99 = 149.98
    assert!((category_totals["Electronics"] - 1349.97).abs() < 0.01);
    assert!((category_totals["Books"]       -   34.98).abs() < 0.01);
    assert!((category_totals["Clothing"]    -  149.98).abs() < 0.01);

    // 未発送オーダーが除外されていることを確認
    let shipped_count = QueryBuilder::from(orders)
        .where_(|o| o.is_shipped)
        .count();
    assert_eq!(shipped_count, 7);

    // 全カテゴリの売上合計が発送済みオーダーの合計と一致すること
    let total_from_categories: f64 = category_totals.values().sum();
    let total_from_orders: f64 = QueryBuilder::from(sample_orders())
        .where_(|o| o.is_shipped)
        .aggregate(0.0, |acc, o| acc + o.amount);
    assert!((total_from_categories - total_from_orders).abs() < 0.01);
}

#[test]
fn scenario1_top_categories_by_revenue() {
    let orders = sample_orders();

    let mut totals: Vec<(&str, f64)> = QueryBuilder::from(orders)
        .where_(|o| o.is_shipped)
        .group_by_aggregate(
            |o| o.category,
            |group| group.iter().map(|o| o.amount).sum::<f64>(),
        )
        .into_iter()
        .collect();

    // 売上降順でソート
    totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let top2: Vec<&str> = QueryBuilder::from(totals)
        .take(2)
        .select(|(cat, _)| cat)
        .collect();

    assert_eq!(top2[0], "Electronics");
    assert_eq!(top2[1], "Clothing");
}

// ─────────────────────────────────────────────────────────────────
// Scenario 2: ログ解析パイプライン
// ─────────────────────────────────────────────────────────────────

fn sample_logs() -> Vec<LogEntry> {
    vec![
        LogEntry { level: LogLevel::Info,  message: "Server started",        user_id: None      },
        LogEntry { level: LogLevel::Error, message: "DB connection failed",   user_id: Some(42)  },
        LogEntry { level: LogLevel::Warn,  message: "Slow query detected",    user_id: Some(7)   },
        LogEntry { level: LogLevel::Error, message: "DB connection failed",   user_id: Some(99)  }, // 重複メッセージ
        LogEntry { level: LogLevel::Info,  message: "Request processed",      user_id: Some(42)  },
        LogEntry { level: LogLevel::Error, message: "Null pointer exception", user_id: None      },
        LogEntry { level: LogLevel::Warn,  message: "Memory usage high",      user_id: Some(7)   },
        LogEntry { level: LogLevel::Info,  message: "Cache hit",              user_id: None      },
    ]
}

#[test]
fn scenario2_log_analysis_pipeline() {
    let logs = sample_logs();

    // ERROR / WARN のみのレベル別カウント
    let level_counts: HashMap<String, usize> = QueryBuilder::from(logs.clone())
        .where_(|l| matches!(l.level, LogLevel::Error | LogLevel::Warn))
        .group_by_aggregate(
            |l| format!("{:?}", l.level),
            |group| group.len(),
        );

    assert_eq!(level_counts["Error"], 3);
    assert_eq!(level_counts["Warn"],  2);
    assert!(!level_counts.contains_key("Info"));

    // 重複メッセージを除いた ERROR 一覧
    let unique_errors: Vec<&str> = QueryBuilder::from(logs.clone())
        .where_(|l| l.level == LogLevel::Error)
        .distinct_by(|l| l.message)
        .select(|l| l.message)
        .collect();

    assert_eq!(unique_errors.len(), 2); // "DB connection failed" と "Null pointer exception"

    // user_id が存在するエラーのユーザーIDを収集
    let affected_users: Vec<u32> = QueryBuilder::from(logs)
        .where_(|l| l.level == LogLevel::Error)
        .flat_map(|l| l.user_id.into_iter())
        .distinct()
        .order_by(|id| *id)
        .collect();

    assert_eq!(affected_users, vec![42, 99]);
}

// ─────────────────────────────────────────────────────────────────
// Scenario 3: 在庫管理 — 集合演算パイプライン
// ─────────────────────────────────────────────────────────────────

#[test]
fn scenario3_inventory_set_operations() {
    let warehouse_a: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
    let warehouse_b: Vec<u32> = vec![4, 5, 6, 7, 8, 9];

    // 共通在庫
    let mut common: Vec<u32> = QueryBuilder::from(warehouse_a.clone())
        .intersect(warehouse_b.clone())
        .collect();
    common.sort();
    assert_eq!(common, vec![4, 5, 6]);

    // 倉庫Aのみ
    let mut a_only: Vec<u32> = QueryBuilder::from(warehouse_a.clone())
        .except(warehouse_b.clone())
        .collect();
    a_only.sort();
    assert_eq!(a_only, vec![1, 2, 3]);

    // 倉庫Bのみ
    let mut b_only: Vec<u32> = QueryBuilder::from(warehouse_b.clone())
        .except(warehouse_a.clone())
        .collect();
    b_only.sort();
    assert_eq!(b_only, vec![7, 8, 9]);

    // 集合の分配法則: union の要素数 = A + B - A∩B
    let mut union_result: Vec<u32> = QueryBuilder::from(warehouse_a.clone())
        .union(warehouse_b.clone())
        .collect();
    union_result.sort();
    assert_eq!(union_result.len(), 9); // 1〜9

    // intersect + a_only + b_only の合計 = union の要素数
    assert_eq!(common.len() + a_only.len() + b_only.len(), union_result.len());

    // concat + distinct で union と同じ結果
    let mut concat_dedup: Vec<u32> = QueryBuilder::from(warehouse_a)
        .concat(warehouse_b)
        .distinct()
        .collect();
    concat_dedup.sort();
    assert_eq!(concat_dedup, union_result);
}

// ─────────────────────────────────────────────────────────────────
// Scenario 4: ページング付き検索
// ─────────────────────────────────────────────────────────────────

fn sample_users() -> Vec<User> {
    (0..50u32)
        .map(|i| User {
            id:     i,
            name:   if i % 2 == 0 { "Alice" } else { "Bob" }, // 簡略化
            age:    20 + (i % 30),
            active: i % 5 != 0, // 5の倍数は非アクティブ
        })
        .collect()
}

#[test]
fn scenario4_pagination() {
    let users = sample_users();

    // アクティブユーザーのみ、ID順に2ページ目（11〜20件目、0-indexed）
    let page_size = 10usize;
    let page = 1usize; // 0始まり

    let page_result: Vec<u32> = QueryBuilder::from(users.clone())
        .where_(|u| u.active)
        .order_by(|u| u.id)
        .skip(page * page_size)
        .take(page_size)
        .select(|u| u.id)
        .collect();

    assert_eq!(page_result.len(), 10);

    // 最初のページと重複しないこと
    let page0: Vec<u32> = QueryBuilder::from(users.clone())
        .where_(|u| u.active)
        .order_by(|u| u.id)
        .take(page_size)
        .select(|u| u.id)
        .collect();

    let overlap = QueryBuilder::from(page_result.clone())
        .intersect(page0)
        .count();
    assert_eq!(overlap, 0);

    // skip が要素数を超えた場合は空
    let beyond: Vec<u32> = QueryBuilder::from(users.clone())
        .where_(|u| u.active)
        .skip(10000)
        .select(|u| u.id)
        .collect();
    assert!(beyond.is_empty());

    // element_at でページ先頭要素に直接アクセスできること
    let active_sorted: Vec<User> = QueryBuilder::from(users)
        .where_(|u| u.active)
        .order_by(|u| u.id)
        .collect();

    let first_on_page1 = QueryBuilder::from(active_sorted.clone())
        .skip(page * page_size)
        .element_at(0);
    assert!(first_on_page1.is_some());
    assert_eq!(first_on_page1.unwrap().id, page_result[0]);
}

// ─────────────────────────────────────────────────────────────────
// Scenario 5: センサーデータ統計サマリー
// ─────────────────────────────────────────────────────────────────

fn sample_sensor_readings() -> Vec<SensorReading> {
    vec![
        SensorReading { sensor_id: 1, temperature: 22.5 },
        SensorReading { sensor_id: 2, temperature: 19.0 },
        SensorReading { sensor_id: 1, temperature: 23.0 },
        SensorReading { sensor_id: 3, temperature: 150.0 }, // 異常値
        SensorReading { sensor_id: 2, temperature: 18.5 },
        SensorReading { sensor_id: 1, temperature: -60.0 }, // 異常値
        SensorReading { sensor_id: 3, temperature: 21.0 },
        SensorReading { sensor_id: 2, temperature: 20.0 },
        SensorReading { sensor_id: 1, temperature: 24.0 },
        SensorReading { sensor_id: 3, temperature: 22.0 },
    ]
}

#[test]
fn scenario5_sensor_statistics() {
    let readings = sample_sensor_readings();

    // 異常値除外 (-50.0..=100.0)
    let valid: Vec<SensorReading> = QueryBuilder::from(readings.clone())
        .where_(|r| r.temperature >= -50.0 && r.temperature <= 100.0)
        .collect();

    assert_eq!(valid.len(), 8); // 150.0 と -60.0 を除外

    // 異常値が除外されていること
    let max_temp = QueryBuilder::from(valid.clone()).max_by(|r| r.temperature as i64);
    assert!(max_temp.unwrap().temperature <= 100.0);

    // センサー別の観測数
    let sensor_counts: HashMap<u8, usize> = QueryBuilder::from(valid.clone())
        .group_by_aggregate(
            |r| r.sensor_id,
            |group| group.len(),
        );

    assert_eq!(sensor_counts[&1], 3); // 22.5, 23.0, 24.0（-60.0は除外）
    assert_eq!(sensor_counts[&2], 3); // 19.0, 18.5, 20.0
    assert_eq!(sensor_counts[&3], 2); // 21.0, 22.0（150.0は除外）

    // センサー別平均温度
    let sensor_avgs: HashMap<u8, f64> = QueryBuilder::from(valid.clone())
        .group_by_aggregate(
            |r| r.sensor_id,
            |group| {
                let sum: f64 = group.iter().map(|r| r.temperature).sum();
                sum / group.len() as f64
            },
        );

    // センサー1の平均: (22.5 + 23.0 + 24.0) / 3 = 23.1666...
    assert!((sensor_avgs[&1] - (22.5 + 23.0 + 24.0) / 3.0).abs() < 1e-9);

    // 最も観測数が多いセンサー
    let max_count_sensor = QueryBuilder::from(sensor_counts.clone().into_iter().collect::<Vec<_>>())
        .max_by(|(_, count)| *count);
    assert!(max_count_sensor.is_some());

    // 全センサーの総観測数 = valid の件数
    let total: usize = sensor_counts.values().sum();
    assert_eq!(total, valid.len());
}

// ─────────────────────────────────────────────────────────────────
// Scenario 6: zip による時系列比較
// ─────────────────────────────────────────────────────────────────

#[test]
fn scenario6_time_series_comparison() {
    let current_year  = vec![110.0, 95.0, 130.0, 120.0, 140.0, 100.0];
    let previous_year = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

    // 前年比を計算（current / previous）
    // enumerate → where_ → select の順（select後はProjected状態なのでenumerateが使えない）
    let growth_rates: Vec<(usize, f64)> = QueryBuilder::from(current_year.clone())
        .zip(previous_year.clone())
        .enumerate()                                              // (usize, (f64, f64))
        .where_(|(_, (cur, prev))| cur / prev >= 1.1)            // 10%以上成長した月
        .select(|(idx, (cur, prev))| (idx, cur / prev))
        .collect();

    // 110% (month 0), 130% (month 2), 120% (month 3), 140% (month 4)
    assert_eq!(growth_rates.len(), 4);
    assert!(growth_rates.iter().all(|(_, r)| *r >= 1.1));

    // enumerate で index が 0 始まりであること
    let first_month_idx = growth_rates[0].0;
    assert_eq!(first_month_idx, 0); // 110/100=1.1 が最初に該当

    // zip が短い方で終了すること
    let longer  = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let shorter = vec![10.0, 20.0, 30.0];
    let zipped: Vec<(f64, f64)> = QueryBuilder::from(longer)
        .zip(shorter)
        .collect();
    assert_eq!(zipped.len(), 3);

    // 前年比の合計が手動計算と一致すること
    let total_ratio: f64 = QueryBuilder::from(current_year)
        .zip(previous_year)
        .select(|(cur, prev)| cur / prev)
        .aggregate(0.0, |acc, r| acc + r);
    // 1.1 + 0.95 + 1.3 + 1.2 + 1.4 + 1.0 = 6.95
    assert!((total_ratio - 6.95).abs() < 1e-9);
}

// ─────────────────────────────────────────────────────────────────
// Scenario 7: エラーハンドリングの網羅
// ─────────────────────────────────────────────────────────────────

#[test]
fn scenario7_error_handling_coverage() {
    // single: 空 → IteratorExhausted
    let empty: Vec<i32> = vec![];
    let result = QueryBuilder::from(empty.clone()).single();
    assert!(matches!(result, Err(RinqError::IteratorExhausted)));

    // single: 複数 → ExecutionError
    let result = QueryBuilder::from(vec![1, 2, 3]).single();
    assert!(matches!(result, Err(RinqError::ExecutionError { .. })));

    // single: 1件 → Ok
    let result = QueryBuilder::from(vec![42]).single();
    assert_eq!(result.unwrap(), 42);

    // single_or_default: 空 → Ok(0)
    let result: Result<i32, _> = QueryBuilder::from(empty.clone()).single_or_default();
    assert_eq!(result.unwrap(), 0);

    // single_or_default: 複数 → ExecutionError
    let result: Result<i32, _> = QueryBuilder::from(vec![1, 2]).single_or_default();
    assert!(matches!(result, Err(RinqError::ExecutionError { .. })));

    // first_or_default: 空 → T::default() でパニックしない
    let result: i32 = QueryBuilder::from(empty.clone()).first_or_default();
    assert_eq!(result, 0);

    let result: String = QueryBuilder::from(Vec::<String>::new()).first_or_default();
    assert_eq!(result, "");

    // last_or_default: 空 → T::default()
    let result: i32 = QueryBuilder::from(empty).last_or_default();
    assert_eq!(result, 0);

    // to_hashmap: 重複キー → ExecutionError
    let data = vec![(1u32, "a"), (1u32, "b"), (2u32, "c")];
    let result = QueryBuilder::from(data).to_hashmap(|(k, _)| *k);
    assert!(matches!(result, Err(RinqError::ExecutionError { .. })));

    // to_hashmap: ユニークキー → Ok
    let data = vec![(1u32, "a"), (2u32, "b"), (3u32, "c")];
    let result = QueryBuilder::from(data).to_hashmap(|(k, _)| *k);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);

    // to_lookup: 重複キーでも Ok (Vec に蓄積)
    let data = vec![(1u32, "a"), (1u32, "b"), (2u32, "c")];
    let lookup = QueryBuilder::from(data).to_lookup(|(k, _)| *k);
    assert_eq!(lookup[&1].len(), 2);
    assert_eq!(lookup[&2].len(), 1);
}

// ─────────────────────────────────────────────────────────────────
// Scenario 8: 生成演算子 × 変換チェーン
// ─────────────────────────────────────────────────────────────────

#[test]
fn scenario8_generation_operators() {
    // range: 1..=10 の二乗和 = 385
    // Initial 状態に select はないため flat_map で Filtered 経由
    let sum_of_squares: i32 = QueryBuilder::range(1..=10i32)
        .flat_map(|x| std::iter::once(x * x))
        .sum();
    assert_eq!(sum_of_squares, 385);

    // repeat: 5 回繰り返し + enumerate + select でシリアル番号付与
    // Initial → enumerate → Filtered → select → Projected
    let messages: Vec<(usize, String)> = QueryBuilder::repeat("ping".to_string(), 5)
        .enumerate()
        .select(|(i, msg)| (i, format!("{}-{}", msg, i)))
        .collect();
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0], (0, "ping-0".to_string()));
    assert_eq!(messages[4], (4, "ping-4".to_string()));

    // empty + concat で通常クエリと同じ動作
    // empty() はターボフィッシュ不要（型推論 or 変数注釈で解決）
    let base = vec![1i32, 2, 3];
    let result_normal: Vec<i32> = QueryBuilder::from(base.clone()).collect();
    let empty_builder: QueryBuilder<i32, _> = QueryBuilder::empty();
    let result_concat: Vec<i32> = empty_builder.concat(base).collect();
    assert_eq!(result_normal, result_concat);

    // range + aggregate で fold と同じ結果
    let product_via_aggregate = QueryBuilder::range(1..=5i64)
        .aggregate(1i64, |acc, x| acc * x);
    assert_eq!(product_via_aggregate, 120); // 5! = 120

    // range + filter + count
    let primes_under_20: usize = QueryBuilder::range(2..20i32)
        .where_(|&n| (2..n).all(|d| n % d != 0))
        .count();
    assert_eq!(primes_under_20, 8); // 2,3,5,7,11,13,17,19

    // empty に対する集約操作が安全に動作すること
    let e1: QueryBuilder<i32, _> = QueryBuilder::empty();
    let e2: QueryBuilder<i32, _> = QueryBuilder::empty();
    let e3: QueryBuilder<i32, _> = QueryBuilder::empty();
    let e4: QueryBuilder<i32, _> = QueryBuilder::empty();
    assert_eq!(e1.count(), 0);
    assert_eq!(e2.sum(), 0);
    assert!(e3.first().is_none());
    assert!(e4.min().is_none());
}

// ─────────────────────────────────────────────────────────────────
// Bonus: aggregate_no_seed / chunk / window の複合確認
// ─────────────────────────────────────────────────────────────────

#[test]
fn scenario_bonus_sequence_operations() {
    // aggregate_no_seed: 空 → None、単一要素 → Some
    let result = QueryBuilder::from(vec![10i32]).aggregate_no_seed(|a, b| a + b);
    assert_eq!(result, Some(10));

    let e5: QueryBuilder<i32, _> = QueryBuilder::empty();
    let result: Option<i32> = e5.aggregate_no_seed(|a, b| a + b);
    assert!(result.is_none());

    // chunk: 合計要素数が保存されること
    let data: Vec<i32> = (1..=10).collect();
    let chunks: Vec<Vec<i32>> = QueryBuilder::from(data.clone())
        .chunk(3)
        .collect();
    assert_eq!(chunks.len(), 4); // [1,2,3], [4,5,6], [7,8,9], [10]
    assert_eq!(chunks.last().unwrap().len(), 1);
    let total: usize = chunks.iter().map(|c| c.len()).sum();
    assert_eq!(total, data.len());

    // window: 各ウィンドウのサイズが window_size と一致すること
    let windows: Vec<Vec<i32>> = QueryBuilder::from(data.clone())
        .window(3)
        .collect();
    assert!(windows.iter().all(|w| w.len() == 3));
    assert_eq!(windows.len(), data.len() - 3 + 1); // 8

    // partition: 2つのベクタの合計が元の件数と一致
    let (evens, odds): (Vec<i32>, Vec<i32>) = QueryBuilder::from(data.clone())
        .partition(|x| x % 2 == 0);
    assert_eq!(evens.len() + odds.len(), data.len());
    assert!(evens.iter().all(|x| x % 2 == 0));
    assert!(odds.iter().all(|x| x % 2 != 0));

    // skip_while + take_while の組み合わせ（先頭の小さい値をスキップし、大きい値だけ取る）
    let data2 = vec![1i32, 2, 3, 10, 11, 12, 1, 2];
    let middle: Vec<i32> = QueryBuilder::from(data2)
        .skip_while(|x| *x < 5)
        .take_while(|x| *x >= 10)
        .collect();
    assert_eq!(middle, vec![10, 11, 12]);

    // reverse の対合性（二回 reverse すると元に戻る）
    let original: Vec<i32> = (1..=5).collect();
    let double_reversed: Vec<i32> = QueryBuilder::from(original.clone())
        .reverse()
        .reverse()
        .collect();
    // reverse は Filtered を返すので collect 後に再度 QueryBuilder に渡す
    assert_eq!(double_reversed, original);
}

// ─────────────────────────────────────────────────────────────────
// Bonus: MetricsQueryBuilder の E2E 確認
// ─────────────────────────────────────────────────────────────────

#[test]
fn scenario_metrics_e2e() {
    use rinq::MetricsQueryBuilder;
    use std::sync::Arc;

    let collector = Arc::new(rinq::MetricsCollector::default());

    // 同じコレクターで複数のクエリを実行し、メトリクスが蓄積されること
    // new(inner, Arc<MetricsCollector>, operation_name: String)
    let _: Vec<i32> = MetricsQueryBuilder::new(
        QueryBuilder::from(vec![1, 2, 3, 4, 5]),
        collector.clone(),
        "sales_query".to_string(),
    )
    .where_(|x| *x > 2)
    .collect();

    let count = MetricsQueryBuilder::new(
        QueryBuilder::from(vec![10, 20, 30]),
        collector.clone(),
        "count_query".to_string(),
    )
    .count();
    assert_eq!(count, 3);

    // collector.get(key) で個別キーを取得
    assert_eq!(collector.get("query_sales_query"), Some(1));
    assert_eq!(collector.get("query_count_query_count"), Some(1));
}
