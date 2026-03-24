# RINQ (Rust Integrated Query)

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-blue?logo=rust)](https://www.rust-lang.org/)

**RINQ** は、Rust の型システムとゼロコスト抽象化を活用した型安全なクエリエンジンです。C# の LINQ から着想を得つつ、Rust の型状態パターンによりコンパイル時にクエリの正当性を保証します。

## クイックスタート

```sh
cargo add rinq
```

```rust
use rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let result: Vec<_> = QueryBuilder::from(data)
    .where_(|x| x % 2 == 0)   // 偶数のみ
    .select(|x| x * 2)         // 2倍
    .collect();

println!("{:?}", result); // [4, 8, 12, 16, 20]
```

## 主な特徴

- **コンパイル時クエリ検証**: 型状態パターンにより無効なメソッドチェーンはコンパイルエラー
- **ゼロコスト抽象化**: 手書きループと同等のパフォーマンス
- **遅延評価**: 終端操作（`collect()` 等）が呼ばれるまで何も実行されない
- **メトリクス統合**: `MetricsQueryBuilder` でクエリ実行回数を自動計測

## 型状態パターン

```
Initial
  └─ where_() ──→ Filtered
                    ├─ order_by() ──→ Sorted ──→ then_by() ──→ ...
                    └─ select()  ──→ Projected<U>
```

無効なチェーン（例: `order_by()` 後に `where_()` を呼ぶ）はコンパイルエラーになります。

## API 一覧

| カテゴリ | メソッド |
|---|---|
| フィルタリング | `where_`, `take`, `skip` |
| 変換 | `select`, `inspect` |
| ソート | `order_by`, `then_by` |
| 数値集約 | `count`, `sum`, `average`, `min`, `max`, `min_by`, `max_by` |
| コレクション集約 | `group_by`, `partition` |
| 終端操作 | `first`, `last`, `any`, `all`, `collect` |
| シーケンス変換 | `distinct`, `reverse`, `chunk`, `window`, `zip` |

## 使用例

```rust
use rinq::QueryBuilder;

// グループ化
let scores = vec![("Alice", 85), ("Bob", 92), ("Alice", 78)];
let grouped = QueryBuilder::from(scores)
    .group_by(|(name, _)| *name);

// チャンク分割
let batches: Vec<Vec<_>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .chunk(2)
    .collect(); // [[1,2], [3,4], [5]]

// ウィンドウ処理
let windows: Vec<Vec<_>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .window(3)
    .collect(); // [[1,2,3], [2,3,4], [3,4,5]]
```

### メトリクス統合

```rust
use rinq::{MetricsQueryBuilder, MetricsCollector, QueryBuilder};
use std::sync::Arc;

let metrics = Arc::new(MetricsCollector::new());

let result: Vec<_> = MetricsQueryBuilder::new(
    QueryBuilder::from(vec![1, 2, 3, 4, 5]),
    metrics.clone(),
    "my_query".to_string(),
)
.where_(|x| *x > 2)
.collect();

assert_eq!(metrics.get("query_my_query"), Some(1));
```

## テスト

```sh
cargo test                            # 全テスト（262件）
cargo test --test core_tests          # コアテスト
cargo test --test rinq_property_tests # プロパティベーステスト
cargo test --doc                      # doctest（25件）
cargo bench --no-run                  # ベンチマークのコンパイル確認
```

## モジュール構成

```
src/
  lib.rs          — 公開 API の flat re-export
  core/
    builder.rs    — QueryBuilder<T, State> + Queryable トレイト
    error.rs      — RinqError, RinqResult
    state.rs      — 型状態マーカー
  metrics/
    builder.rs    — MetricsQueryBuilder<T, State>
    collector.rs  — MetricsCollector
```

## ドキュメント

- `docs/implementation.md` — 今後のロードマップ（Join / Parallel / Async / WASM）
- `versions/v1/spec.md` — v1.0 仕様書
- `CHANGELOG.md` — バージョン履歴

## ライセンス

MIT License

## Author

Yoshimura Hisanori
