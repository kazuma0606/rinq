# RINQ v3.0 仕様書

**作成日**: 2026-03-25
**ステータス**: Draft

---

## 概要

RINQ v3.0 は v2.0 で完成したコアクエリエンジンを基盤に、**Rustらしい独自性**と**データサイエンス用途**への対応を軸とする拡張リリースです。

LINQ互換の演算子拡充（v2.0の主目標）から一歩進み、以下の3方針で設計を進めます。

1. **Rustエコシステムとの統合** — `rayon`（並列）、`serde`（JSON）との自然な連携
2. **データサイエンス機能** — 統計演算・複数データソース間の相関分析・サンプリング
3. **LINQにない独自演算子** — Rustの型システムを活かした失敗許容パイプライン・再利用可能クエリフラグメント・ウィンドウ分析関数

### スコープ外

- **SQL統合・DB操作**: 別プロジェクト [`oxide`](https://github.com/kazuma0606/oxide) で開発。RINQはin-memoryクエリエンジンの役割を維持する。
- **WASM対応・procマクロ**: docs/implementation.md の Phase 5/6 として据え置き。

---

## v2.0 からの位置づけ

```
v1.0  コアエンジン確立（QueryBuilder, 型ステートパターン, Queryable）
  ↓
v2.0  LINQ差分の補完（flat_map, aggregate, 集合演算, 生成演算子 等）
  ↓
v3.0  Rust独自拡張（並列, 統計, ウィンドウ関数, serde統合 等）  ← 本文書
```

v3.0 は **破壊的変更なし**を原則とします。すべての新機能は新規メソッド・新規クレートの追加であり、v2.0 のコードはそのままビルドできます。

---

## クレート構成

v3.0 からマルチクレート構成へ移行します。

```
rinq              — コアクエリエンジン（現在のコード + v3新規演算子）
                    feature flags:
                      parallel  rayon による並列処理
                      serde     JSON / serde_json 統合
rinq-stats        — 統計演算・複数ソース相関分析・サンプリング
rinq-macro        — proc-macro (derive, query!マクロ) ← Phase 6 で着手
```

### feature flags（`rinq` 本体）

```toml
[features]
default = []
parallel = ["dep:rayon"]
serde    = ["dep:serde", "dep:serde_json"]

[dependencies]
rayon      = { version = "1.10", optional = true }
serde      = { version = "1.0",  optional = true, features = ["derive"] }
serde_json = { version = "1.0",  optional = true }
```

使用側:

```toml
rinq = { version = "0.3", features = ["parallel", "serde"] }
```

---

## 1. 並列処理（`feature = "parallel"`）

### 設計方針

- `ParallelQueryBuilder<T, State>` を独立型として追加。`QueryBuilder` とは型を分離する。
  - 理由: `rayon::ParallelIterator` は `std::iter::Iterator` と異なるトレイトであり、同一型での切り替えは型システムと相性が悪い。
- `QueryBuilder::into_parallel()` で変換エントリポイントを提供。
- 要素数が内部しきい値未満の場合は自動的に逐次処理にフォールバックする（ユーザーが意識しない）。

### 公開 API

```rust
use rinq::ParallelQueryBuilder;

// from で直接生成
let result: Vec<i32> = ParallelQueryBuilder::from(data)
    .par_where(|x| expensive_predicate(x))
    .par_select(|x| heavy_transform(x))
    .collect();

// QueryBuilder から変換
let result: i32 = QueryBuilder::from(data)
    .where_(|x| *x > 0)
    .into_parallel()           // → ParallelQueryBuilder
    .par_sum();
```

### 実装するメソッド

| メソッド | 説明 | 制約 |
|----------|------|------|
| `par_where(pred)` | 並列フィルタ | `F: Fn(&T) -> bool + Sync + Send` |
| `par_select(f)` | 並列射影 | `F: Fn(T) -> U + Sync + Send` |
| `par_flat_map(f)` | 並列ネスト平坦化 | `F: Fn(T) -> I + Sync + Send` |
| `par_count()` | 並列カウント | `T: Send` |
| `par_sum()` | 並列合計 | `T: Send + Sum` |
| `par_min()` / `par_max()` | 並列最小・最大 | `T: Ord + Send` |
| `par_any(pred)` / `par_all(pred)` | 並列存在確認 | `F: Fn(&T) -> bool + Sync` |
| `collect()` | 並列収集 | `T: Send` |
| `par_group_by(key)` | 並列グループ化 | `K: Hash + Eq + Send` |

### トレイト制約

`T: Send` が必須です。共有参照を渡す場合は `T: Sync` も要求されます。クロージャは原則 `Fn + Sync + Send` です。

---

## 2. ウィンドウ分析関数（SQL-style analytics）

現在の `window(n)` は「スライディングウィンドウで `Vec<T>` に切り出す」操作ですが、SQLの分析関数は**シーケンス全体の順序関係に基づいて各要素に値を付与する**操作です。意味が異なるため、別メソッドとして追加します。

### 累積集約（running aggregations）

```rust
// 累積和（各位置での prefix sum）
let cumsum: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .running_sum()
    .collect();
// → [1, 3, 6, 10, 15]

// 累積平均
let running_avg: Vec<f64> = QueryBuilder::from(daily_sales)
    .running_average()
    .collect();

// 移動平均（直近 n 件の平均）
// 戻り値の型は QueryBuilder<Option<f64>, Filtered>
// 先頭 n-1 件はウィンドウが満たされていないため None を返す。
// これにより、不完全なウィンドウ値を誤って使用することをコンパイル時に防ぐ。
let ma: Vec<Option<f64>> = QueryBuilder::from(prices)
    .moving_average(7)   // 7日移動平均
    .collect();
// → [None, None, None, None, None, None, Some(avg_7), Some(avg_8), ...]

// None を除外して使いたい場合は flat_map で明示的に対処する
let ma_values: Vec<f64> = QueryBuilder::from(prices)
    .moving_average(7)
    .where_(|v| v.is_some())
    .select(|v| v.unwrap())
    .collect();
```

### ランキング

```rust
// rank: 同値はスキップあり（1, 2, 2, 4 ...）
// dense_rank: 同値はスキップなし（1, 2, 2, 3 ...）
let ranked: Vec<(usize, Product)> = QueryBuilder::from(products)
    .rank_by(|p| p.price)
    .collect();

let dense: Vec<(usize, Product)> = QueryBuilder::from(products)
    .dense_rank_by(|p| p.score)
    .collect();
```

### lag / lead（前後参照）

```rust
// lag(n): 現在の要素と n 個前の要素をペアにする
let with_prev: Vec<(Option<i32>, i32)> = QueryBuilder::from(time_series)
    .lag(1)
    .collect();
// → [(None, 10), (Some(10), 20), (Some(20), 30), ...]

// lead(n): 現在の要素と n 個後の要素をペアにする
let with_next: Vec<(i32, Option<i32>)> = QueryBuilder::from(time_series)
    .lead(1)
    .collect();
```

### 対象状態

ウィンドウ分析関数はすべて `Initial` / `Filtered` / `Sorted` 状態で使用可能とします。戻り値の状態は `Filtered` とします（結果型が変わるため）。

---

## 3. 失敗許容パイプライン

LINQには存在しない、Rustの `Result` / `Option` 型を活かした演算子群です。

### `try_select`

変換が失敗する可能性がある射影。`Err` を個別に回収するか、最初のエラーで中断するかを選択できます。

```rust
// 失敗した要素を Err として収集し続ける（中断しない）
let (ok_values, errors): (Vec<i32>, Vec<_>) = QueryBuilder::from(strings)
    .try_select(|s| s.parse::<i32>())
    .collect_partitioned();        // → (Ok側, Err側) に分割

// 最初の Err で中断して Err を返す（標準的な ? 伝播）
let result: RinqResult<Vec<i32>> = QueryBuilder::from(strings)
    .try_select(|s| s.parse::<i32>().map_err(|e| RinqError::ExecutionError { message: e.to_string() }))
    .collect_results();
```

### `try_where_`

フィルタ条件が `Result<bool, E>` を返す場合:

```rust
let filtered = QueryBuilder::from(records)
    .try_where_(|r| validate_external(r))  // → Result<bool, _>
    .collect_results();
```

### `collect_partitioned` の戻り値型

```rust
// try_select 後の専用ターミナル
fn collect_partitioned(self) -> (Vec<T>, Vec<E>)
```

---

## 4. 再利用可能なクエリフラグメント

大規模コードベースでクエリロジックを名前付きで定義し、複数箇所で組み合わせる機能です。

```rust
use rinq::fragment::{QueryFilter, QuerySorter, QueryPaginator};

// 定義（クロージャを名前付きフラグメントとして保持）
let adult_filter  = QueryFilter::new(|u: &User| u.age >= 18);
let by_name_asc   = QuerySorter::new(|u: &User| u.name.clone());
let first_page    = QueryPaginator::new(0, 20);

// 組み合わせる（any order）
let result: Vec<User> = QueryBuilder::from(users)
    .apply_filter(adult_filter)
    .apply_sort(by_name_asc)
    .apply_paginator(first_page)
    .collect();
```

### `apply` メソッド群

| メソッド | フラグメント型 | 状態遷移 |
|----------|--------------|---------|
| `apply_filter(f)` | `QueryFilter<T>` | `Initial → Filtered` |
| `apply_sort(s)` | `QuerySorter<T>` | `Initial/Filtered → Sorted` |
| `apply_paginator(p)` | `QueryPaginator` | `* → Filtered` |

フラグメントは `Clone` を実装し、複数クエリで共有できます。

---

## 5. serde 統合（`feature = "serde"`）

### JSON からの直接クエリ

```rust
use rinq::serde::QueryBuilder as SerdeQueryBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct User { id: u32, name: String, age: u32 }

// JSON バイト列 / 文字列から構築
let json = r#"[{"id":1,"name":"Alice","age":30},{"id":2,"name":"Bob","age":17}]"#;

let adults: Vec<User> = SerdeQueryBuilder::from_json::<User>(json)?
    .where_(|u| u.age >= 18)
    .order_by(|u| u.age)
    .collect();
```

### `from_json_value`（`serde_json::Value` から）

スキーマが不明な場合のダイナミッククエリ:

```rust
use serde_json::Value;

let adults: Vec<Value> = SerdeQueryBuilder::from_json_value(json)?
    .where_(|v| v["age"].as_u64().unwrap_or(0) >= 18)
    .collect();
```

### 設計方針

- `SerdeQueryBuilder::from_json` は `serde_json::from_str` のラッパー。デシリアライズ失敗は `RinqError::ExecutionError` として返す。
- 型が `Deserialize` を実装していれば任意の構造体に適用できる（スキーマ変更はコンパイルエラーとして検出される）。
- `Queryable` トレイトの `serde` feature 向け拡張として実装し、コア API を汚さない。

---

## 6. `rinq-stats` クレート

統計演算は依存クレートが増えやすいため、`rinq` 本体とは独立した別クレートとします。

```toml
# 使用側
[dependencies]
rinq       = "0.3"
rinq-stats = "0.1"
```

### 6-1. 単一ソース統計（`StatisticsExt` トレイト）

`QueryBuilder` に `StatisticsExt` トレイトを impl することで追加します（拡張トレイトパターン）。

```rust
use rinq_stats::StatisticsExt;

let data = QueryBuilder::from(prices);

// 分布
data.variance()                // → f64  母分散
data.std_dev()                 // → f64  標準偏差
data.median()                  // → Option<f64>  中央値
data.mode()                    // → Option<T>   最頻値 (T: Hash + Eq)
data.percentile(0.95)          // → Option<f64>  パーセンタイル
data.quantile(0.25)            // → Option<f64>  四分位数（p=0.25/0.75 等）

// 形状
data.skewness()                // → f64  歪度
data.kurtosis()                // → f64  尖度

// 分布の可視化補助
data.histogram(10)             // → Vec<(RangeInclusive<f64>, usize)>  度数分布表
data.frequency_table()         // → HashMap<T, usize>  度数表 (T: Hash + Eq)
```

すべて即時実行です。`f64` を返すメソッドは `T: Into<f64>` を要求します。

### 6-2. 複数ソース統計（`QueryPair`）

2つのデータ系列間の関係を分析するための型です。共変性・相関係数の計算が主な用途です。

```rust
use rinq_stats::QueryPair;

let prices  = vec![100.0, 102.0, 98.0, 105.0, 97.0];
let volumes = vec![1000.0, 950.0, 1100.0, 800.0, 1200.0];

let pair = QueryPair::new(prices, volumes);

pair.covariance()              // → f64  共分散
pair.pearson_correlation()     // → f64  Pearson積率相関係数（-1.0 〜 1.0）
pair.spearman_correlation()    // → f64  Spearman順位相関係数（順序データ向け）
pair.kendall_tau()             // → f64  Kendall's τ（外れ値に頑健）
pair.linear_regression()       // → (slope: f64, intercept: f64)  最小二乗法
```

#### `QueryPair` の構築

```rust
// Vec から直接
let pair = QueryPair::new(vec_x, vec_y);

// QueryBuilder から
let pair = QueryPair::from_builders(
    QueryBuilder::from(raw_x).where_(|x| !x.is_nan()),
    QueryBuilder::from(raw_y).where_(|y| !y.is_nan()),
);
```

#### 長さ不一致の扱い

現実の統計分析では、2つのデータ系列の長さが一致しない場合があります（欠損値・計測タイミングのずれ等）。`QueryPair::new` は**短い方に合わせてtruncate**し、長さ不一致があった場合は `log::warn!` で通知します。

```rust
// 長さが異なる場合: 短い方 (4件) に合わせて truncate、warn を発行
let pair = QueryPair::new(
    vec![1.0, 2.0, 3.0, 4.0, 5.0],  // 5件
    vec![10.0, 20.0, 30.0, 40.0],   // 4件
);
// [WARN rinq_stats] QueryPair: length mismatch (5 vs 4), truncated to 4

// 長さが一致することを型レベルで保証したい場合は try_new を使う
let pair = QueryPair::try_new(vec_x, vec_y)?;  // → RinqResult<QueryPair>
// 長さ不一致なら Err(RinqError::ExecutionError { ... })
```

`log` クレートはオプション依存とし、`rinq-stats` の `Cargo.toml` に追加します。

#### 内部実装

```
covariance(x, y) = (1/n) * Σ (x_i - x̄)(y_i - ȳ)
pearson(x, y)    = cov(x, y) / (σ_x * σ_y)
```

Welford のオンラインアルゴリズムを使用し、2パスなしで計算します（精度とメモリ効率を両立）。

### 6-3. サンプリング（`SamplingExt` トレイト）

統計検定・機械学習のデータ分割で使用します。

```rust
use rinq_stats::SamplingExt;

// ランダムサンプリング（reservoir sampling — データ全体を知らなくても均等確率）
let sample: Vec<T> = QueryBuilder::from(large_dataset)
    .sample_fraction(&mut rng, 0.1)     // ランダム10%
    .collect();

let sample: Vec<T> = QueryBuilder::from(large_dataset)
    .sample_n(&mut rng, 1000)           // ランダムN件
    .collect();

// 層化抽出（カテゴリごとに均等サンプリング）
let sample: Vec<T> = QueryBuilder::from(dataset)
    .stratified_sample(&mut rng, |x| &x.category, 100)  // カテゴリごと最大100件
    .collect();

// 復元抽出（ブートストラップ法用）
let bootstrap: Vec<T> = QueryBuilder::from(dataset)
    .bootstrap_sample(&mut rng, 1000)   // 重複ありランダム1000件
    .collect();
```

乱数生成器は `rand::Rng` を受け取ります。`rand` は `rinq-stats` の依存として追加します。

### 6-4. データ品質バリデーション（`ValidationExt` トレイト）

ETL パイプラインやデータ前処理での使用を想定した演算子です。統計処理の前段として自然に置けるため `rinq-stats` に含めます。

```rust
use rinq_stats::ValidationExt;

// 複数ルールを連鎖させる（違反をすべて収集してから返す）
let result = QueryBuilder::from(records)
    .validate(|r| r.price > 0.0,       "price must be positive")
    .validate(|r| !r.name.is_empty(),  "name is required")
    .collect_validated();   // → Result<Vec<T>, Vec<ValidationError>>

// ValidationError の構造
pub struct ValidationError {
    pub rule: String,    // バリデーションルール名（第2引数の文字列）
    pub message: String, // エラーメッセージ
    pub index: usize,    // 違反した要素のインデックス
}
```

`collect_validated()` はすべての要素を走査し、**違反をすべて収集してから**返します（最初のエラーで中断しない）。違反がなければ `Ok(Vec<T>)` を返します。

---

## 依存関係（`rinq-stats`）

| クレート | バージョン | 用途 |
|----------|-----------|------|
| `rinq` | 0.3 | コアクエリエンジン |
| `rand` | 0.8 | サンプリング用乱数生成 |
| `log` | 0.4 | `QueryPair` 長さ不一致の警告 |

---

## 実装優先度

### Phase A（`rinq` 本体への追加）

```
A1: 並列処理（feature = "parallel"）
    ParallelQueryBuilder + par_where / par_select / par_sum 等
A2: ウィンドウ分析関数
    running_sum / running_average / moving_average（→ Option<f64>）/ rank_by / lag / lead
A3: 失敗許容パイプライン
    try_select / collect_partitioned / collect_results
A4: serde 統合（feature = "serde"）
    SerdeQueryBuilder::from_json / from_json_value
```

### Phase B（`rinq-stats` 新規クレート）

```
B1: 単一ソース統計（StatisticsExt）
    variance / std_dev / median / mode / percentile / histogram
B2: 複数ソース統計（QueryPair）
    covariance / pearson_correlation / linear_regression
    QueryPair::new（truncate + log警告）/ QueryPair::try_new（Err）
B3: サンプリング（SamplingExt）
    sample_n / sample_fraction / stratified_sample / bootstrap_sample
B4: データ品質バリデーション（ValidationExt）
    validate / collect_validated
```

### Phase C（将来）

```
C1: 再利用可能クエリフラグメント（QueryFilter / QuerySorter / QueryPaginator）
C2: rinq-macro（proc-macro derive, query! マクロ）← docs/implementation.md Phase 6
C3: Apache Arrow / Polars 統合（rinq-arrow）
```

---

## 依存関係（v3.0 追加分）

### `rinq` 本体

| クレート | バージョン | 用途 | 追加条件 |
|----------|-----------|------|---------|
| `rayon` | 1.10 | 並列イテレータ | `feature = "parallel"` |
| `serde` | 1.0 | シリアライゼーション | `feature = "serde"` |
| `serde_json` | 1.0 | JSON パース | `feature = "serde"` |

### `rinq-stats`

| クレート | バージョン | 用途 |
|----------|-----------|------|
| `rinq` | 0.3 | コアクエリエンジン |
| `rand` | 0.8 | サンプリング用乱数生成 |

---

## 設計原則（v3.0 全体）

1. **ゼロコスト原則を維持**: 並列・統計演算も、使わない機能はコンパイル後のバイナリに含まれない（feature flag + 別クレート）。
2. **破壊的変更なし**: v2.0 のすべての公開 API はそのまま動作する。
3. **型ステートパターンの継承**: 新規演算子も `Initial → Filtered → Sorted / Projected` の遷移規則に従う。ウィンドウ分析関数は `Filtered` を返す。
4. **エラー処理はRust慣用**: `Result` / `Option` を適切に使い分け、`panic` は不変条件の違反（`chunk(0)` 等）のみに限定する。
5. **SQL統合はスコープ外**: DB操作は `oxide` クレートで担い、RINQ は in-memory クエリエンジンとして完結させる。

---

## 使用例（v3.0 全機能）

```rust
use rinq::{QueryBuilder, ParallelQueryBuilder};
use rinq::serde::QueryBuilder as SerdeQueryBuilder;
use rinq_stats::{StatisticsExt, QueryPair, SamplingExt};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Trade { symbol: String, price: f64, volume: f64 }

// JSON から型安全クエリ
let json = fetch_api_response();  // REST APIレスポンス
let trades: Vec<Trade> = SerdeQueryBuilder::from_json::<Trade>(&json)?
    .where_(|t| t.symbol == "AAPL")
    .order_by(|t| t.price)
    .collect();

// 並列処理で重い変換
let processed: Vec<f64> = ParallelQueryBuilder::from(trades.clone())
    .par_select(|t| complex_transform(t.price))
    .collect();

// 累積和・移動平均（先頭4件は None）
let prices: Vec<f64> = trades.iter().map(|t| t.price).collect();
let moving_avg: Vec<Option<f64>> = QueryBuilder::from(prices.clone())
    .moving_average(5)
    .collect();

// 統計
let volumes: Vec<f64> = trades.iter().map(|t| t.volume).collect();
let std = QueryBuilder::from(prices.clone()).std_dev();
let pair = QueryPair::new(prices, volumes);
let corr = pair.pearson_correlation();  // 価格と出来高の相関

// 層化サンプリング
let mut rng = rand::thread_rng();
let sample: Vec<Trade> = QueryBuilder::from(trades)
    .stratified_sample(&mut rng, |t| &t.symbol, 100)
    .collect();
```

---

## crates.io 公開に向けた API ドキュメント方針

### Rustのドキュメント体系

Rustには `rustdoc` という公式ドキュメント生成ツールが組み込まれており、crates.io に公開すると **[docs.rs](https://docs.rs)** が自動的にビルド・ホストします。手動でAPIリファレンスを別途用意する必要はありませんが、`///` コメントの品質がそのままドキュメントの品質になります。

### 必須の対応

#### 1. `#![warn(missing_docs)]` をクレートルートに追加

```rust
// src/lib.rs
#![warn(missing_docs)]

//! # RINQ — Rust Integrated Query
//!
//! 型安全・ゼロコストなクエリエンジン。C# の LINQ に着想を得た流暢な API を提供します。
//!
//! ## 基本的な使い方
//!
//! ```rust
//! use rinq::QueryBuilder;
//!
//! let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
//!     .where_(|x| *x > 2)
//!     .order_by(|x| *x)
//!     .collect();
//! assert_eq!(result, vec![3, 4, 5]);
//! ```
```

`missing_docs` は公開 API に `///` コメントがない場合に警告を出します。ゼロ警告ポリシー（`-D warnings`）と組み合わせることで、公開時のドキュメント漏れをコンパイルエラーとして検出できます。

#### 2. 各メソッドの `///` コメント標準構成

```rust
/// 条件を満たす要素が現れる間、要素を取得します。
///
/// 最初に条件が偽になった時点で停止します。その後の要素は条件に関わらず無視されます。
///
/// **実行種別**: 遅延ストリーミング
///
/// # Examples
///
/// ```rust
/// use rinq::QueryBuilder;
///
/// let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 1])
///     .take_while(|x| *x < 4)
///     .collect();
/// assert_eq!(result, vec![1, 2, 3]);
/// ```
///
/// # Panics
///
/// パニックしません。
pub fn take_while<F>(self, predicate: F) -> QueryBuilder<T, Filtered>
```

各セクションの役割:

| セクション | 内容 | 必須 |
|-----------|------|------|
| 1行目 | 何をするメソッドかの要約（動詞から始める） | 必須 |
| 本文 | 詳細な動作説明・エッジケース | 推奨 |
| `**実行種別**` | 即時/遅延ストリーミング/遅延非ストリーミング | 必須（RINQ規約） |
| `# Examples` | コピペで動くコード例（doc test として実行される） | 必須 |
| `# Panics` | パニックする条件（しない場合も明記） | 必須 |
| `# Errors` | `RinqResult` を返すメソッドのエラー条件 | 該当時必須 |

#### 3. モジュール・クレートレベルの `//!` コメント

```rust
// src/core/builder/mod.rs
//! クエリビルダーの中核型。
//!
//! [`QueryBuilder<T, State>`] は遅延評価の流暢な API を提供します。
//! 型パラメータ `State` によりコンパイル時に操作順序を検証します。
```

#### 4. `#[doc(alias = "...")]` で検索性を向上

C# / LINQ ユーザーが検索しやすいよう別名を付与します:

```rust
#[doc(alias = "SelectMany")]
pub fn flat_map<U, I, F>(self, f: F) -> QueryBuilder<U, Filtered> { ... }

#[doc(alias = "Where")]
pub fn where_<F>(self, predicate: F) -> QueryBuilder<T, Filtered> { ... }

#[doc(alias = "ToList")]
pub fn collect<B: FromIterator<T>>(self) -> B { ... }
```

#### 5. `README.md` = crates.io のトップページ

`Cargo.toml` に以下を追加することで、`README.md` が crates.io のクレートページに表示されます:

```toml
[package]
readme = "README.md"
```

README には最低限以下を含めます:

- クレートの一言説明
- インストール方法（`Cargo.toml` への追記例）
- 最短の使用例（コードブロック）
- feature flags の一覧
- ライセンス

#### 6. `Cargo.toml` のメタデータ

```toml
[package]
name        = "rinq"
version     = "0.1.0"
edition     = "2024"
description = "Type-safe, zero-cost query engine for Rust, inspired by C# LINQ"
license     = "MIT OR Apache-2.0"          # Rustエコシステムの標準的な二重ライセンス
repository  = "https://github.com/<user>/rinq"
keywords    = ["query", "linq", "iterator", "filter", "data"]  # 最大5つ
categories  = ["data-structures", "algorithms"]                 # crates.io のカテゴリ
readme      = "README.md"
```

`keywords` と `categories` は crates.io での検索・発見性に直結します。

### docs.rs の自動ビルド設定

feature flag 付きドキュメントを docs.rs に生成させるには `Cargo.toml` に追記します:

```toml
[package.metadata.docs.rs]
all-features = true        # すべての feature を有効にしてドキュメントをビルド
rustdoc-args = ["--cfg", "docsrs"]  # feature gate されたアイテムに "Available on feature X" バッジを付ける
```

### doc test の位置づけ

RINQ はすでに多数の doc test を持っています。v3.0 でも各新規メソッドに `# Examples` セクションを設け、`cargo test --doc` で全件通過することをリリース基準とします。doc test はAPIの使用例であり同時に回帰テストでもあるため、実装と乖離した場合にコンパイルエラーで検出できます。
