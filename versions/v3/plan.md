# RINQ v3.0 実装計画

**作成日**: 2026-03-25

---

## 目標

v2.0 で完成した LINQ 互換コアを基盤に、Rust エコシステムとの統合・データサイエンス機能・独自演算子を段階的に追加する。v2.0 の公開 API は一切変更しない。

---

## 設計方針

### 変更しないもの
- 既存の全公開 API（`QueryBuilder`, `MetricsQueryBuilder`, `Queryable`, `RinqError` 等）
- 型ステートパターン（`Initial → Filtered → Sorted / Projected`）
- コアクレートの依存（`thiserror`, `num-traits`, `parking_lot`）

### 追加するもの（`rinq` 本体）
- `features = ["parallel"]` — `rayon` を使った `ParallelQueryBuilder`
- `features = ["serde"]` — JSON 入力対応
- ウィンドウ分析関数（`running_sum`, `running_average`, `moving_average`, `rank_by`, `dense_rank_by`, `lag`, `lead`）
- 失敗許容パイプライン（`try_select`, `try_where_`, `collect_partitioned`, `collect_results`）

### 新規クレート
- `rinq-stats` — 統計演算・複数ソース相関分析・サンプリング・バリデーション

---

## フェーズ構成

```
Phase A1: 並列処理（feature = "parallel"）
  ↓
Phase A2: ウィンドウ分析関数
  ↓
Phase A3: 失敗許容パイプライン
  ↓
Phase A4: serde 統合（feature = "serde"）
  ↓
Phase B1: rinq-stats — 単一ソース統計
  ↓
Phase B2: rinq-stats — QueryPair（複数ソース統計）
  ↓
Phase B3: rinq-stats — サンプリング
  ↓
Phase B4: rinq-stats — バリデーション
  ↓
Phase C: ドキュメント・CHANGELOG 整備・crates.io 公開準備
```

各フェーズは `cargo test` 全件通過・`cargo clippy -- -D warnings` ゼロを確認してから完了とする。

---

## Phase A1: 並列処理（`feature = "parallel"`）

**目的**: `rayon` を使ったデータ並列処理を `ParallelQueryBuilder<T, State>` として提供する。

### ファイル構成（追加）

```
src/
  parallel/
    mod.rs        — ParallelQueryBuilder<T, State> 構造体
    initial.rs    — impl ParallelQueryBuilder<T, Initial>
    filtered.rs   — impl ParallelQueryBuilder<T, Filtered>
    sorted.rs     — impl ParallelQueryBuilder<T, Sorted>
    shared.rs     — 状態横断の terminal ops
```

### 実装内容

```rust
#[cfg(feature = "parallel")]
pub struct ParallelQueryBuilder<T, State> {
    inner: rayon::vec::IntoIter<T>,
    _state: PhantomData<State>,
}

impl<T: Send> ParallelQueryBuilder<T, Initial> {
    pub fn from(data: Vec<T>) -> Self { ... }
}

// QueryBuilder → ParallelQueryBuilder への変換
impl<T: Send, State> QueryBuilder<T, State> {
    #[cfg(feature = "parallel")]
    pub fn into_parallel(self) -> ParallelQueryBuilder<T, State> { ... }
}
```

### 実装するメソッド

| メソッド | 対象状態 | 戻り値 | 制約 |
|----------|---------|--------|------|
| `par_where(pred)` | Initial/Filtered | `Filtered` | `F: Fn(&T) -> bool + Sync + Send` |
| `par_select(f)` | Filtered | `Filtered` | `F: Fn(T) -> U + Sync + Send` |
| `par_flat_map(f)` | Initial/Filtered | `Filtered` | `F: Fn(T) -> I + Sync + Send` |
| `par_order_by(key)` | Initial/Filtered | `Sorted` | `K: Ord + Send` |
| `par_count()` | * | `usize` | `T: Send` |
| `par_sum()` | * | `T` | `T: Send + Sum` |
| `par_min()` / `par_max()` | * | `Option<T>` | `T: Ord + Send` |
| `par_any(pred)` / `par_all(pred)` | * | `bool` | `F: Fn(&T) -> bool + Sync` |
| `collect()` | * | `Vec<T>` | `T: Send` |
| `par_group_by(key)` | * | `HashMap<K, Vec<T>>` | `K: Hash + Eq + Send` |

### しきい値フォールバック

要素数が `RINQ_PARALLEL_THRESHOLD`（デフォルト 1024、環境変数で上書き可能）未満の場合、`rayon` の `ParallelIterator` は自動的に逐次処理に最適化される（rayon 既定動作）。

### Cargo.toml 変更

```toml
[features]
default  = []
parallel = ["dep:rayon"]
serde    = ["dep:serde", "dep:serde_json"]

[dependencies]
rayon      = { version = "1.10", optional = true }
serde      = { version = "1.0",  optional = true, features = ["derive"] }
serde_json = { version = "1.0",  optional = true }
```

---

## Phase A2: ウィンドウ分析関数

**目的**: SQL の分析関数に相当する、シーケンス全体の順序関係に基づく演算子を追加する。

### 追加先ファイル

- `src/core/builder/initial.rs` — `impl QueryBuilder<T, Initial>`
- `src/core/builder/filtered.rs` — `impl QueryBuilder<T, Filtered>`
- `src/core/builder/sorted.rs` — `impl QueryBuilder<T, Sorted>`

すべての戻り値の状態は `Filtered`（結果型が変わるため型ステート遷移が必要）。

### `running_sum` / `running_average`

```rust
/// **実行種別**: 遅延ストリーミング
/// 戻り値の T は T: Clone + Add<Output = T> を要求
pub fn running_sum(self) -> QueryBuilder<T, Filtered>
    where T: Clone + std::ops::Add<Output = T>

/// **実行種別**: 遅延ストリーミング
pub fn running_average(self) -> QueryBuilder<f64, Filtered>
    where T: Into<f64> + Clone
```

実装: `std::iter::scan` をラップ。各要素に累積値を保持する状態クロージャ。

### `moving_average`

```rust
/// **実行種別**: 遅延非ストリーミング（内部でバッファを維持）
/// 戻り値: QueryBuilder<Option<f64>, Filtered>
/// 先頭 n-1 件はウィンドウが満たされないため None を返す。
/// これにより不完全ウィンドウ値の誤使用をコンパイル時に防ぐ。
pub fn moving_average(self, window: usize) -> QueryBuilder<Option<f64>, Filtered>
    where T: Into<f64> + Clone
```

実装: `VecDeque<f64>` をスライディングバッファとして使用。`window` が 0 の場合はパニック。

### `rank_by` / `dense_rank_by`

```rust
/// **実行種別**: 遅延非ストリーミング（全要素のソートが必要）
/// rank: 同値はスキップあり（1, 2, 2, 4 ...）
/// dense_rank: 同値はスキップなし（1, 2, 2, 3 ...）
pub fn rank_by<K, F>(self, key: F) -> QueryBuilder<(usize, T), Filtered>
    where K: Ord, F: Fn(&T) -> K + 'static
pub fn dense_rank_by<K, F>(self, key: F) -> QueryBuilder<(usize, T), Filtered>
    where K: Ord, F: Fn(&T) -> K + 'static
```

実装: 全要素を収集してソート後、順位を割り当て。

### `lag` / `lead`

```rust
/// **実行種別**: 遅延非ストリーミング
/// lag(n): (n 個前の値, 現在値) ペア。先頭 n 件は (None, T)。
pub fn lag(self, n: usize) -> QueryBuilder<(Option<T>, T), Filtered>
    where T: Clone

/// **実行種別**: 遅延非ストリーミング
/// lead(n): (現在値, n 個後の値) ペア。末尾 n 件は (T, None)。
pub fn lead(self, n: usize) -> QueryBuilder<(T, Option<T>), Filtered>
    where T: Clone
```

---

## Phase A3: 失敗許容パイプライン

**目的**: `Result` 型を返す変換を自然に扱えるパイプライン演算子を追加する。

### `try_select`

```rust
/// 変換が Result<U, E> を返す select。
/// collect_partitioned() または collect_results() で終端する。
pub fn try_select<U, E, F>(self, f: F) -> TryQueryBuilder<U, E>
    where F: Fn(T) -> Result<U, E> + 'static
```

`TryQueryBuilder<T, E>` は `QueryBuilder` の薄いラッパーで `Vec<Result<T, E>>` を内部に持つ。

### `try_where_`

```rust
/// フィルタ条件が Result<bool, E> を返す where_。
pub fn try_where_<E, F>(self, predicate: F) -> TryQueryBuilder<T, E>
    where F: Fn(&T) -> Result<bool, E> + 'static
```

### 終端操作

```rust
impl<T, E> TryQueryBuilder<T, E> {
    /// Ok側・Err側に分割して収集（中断しない）
    pub fn collect_partitioned(self) -> (Vec<T>, Vec<E>)

    /// 最初の Err で中断（標準的な ? 伝播と同等）
    pub fn collect_results(self) -> Result<Vec<T>, E>
}
```

---

## Phase A4: serde 統合（`feature = "serde"`）

**目的**: JSON 文字列から直接 `QueryBuilder` を構築できるようにする。

### 実装

```rust
// src/serde/mod.rs（feature = "serde" でのみコンパイル）
#[cfg(feature = "serde")]
impl<T: serde::de::DeserializeOwned + 'static> QueryBuilder<T, Initial> {
    pub fn from_json(json: &str) -> RinqResult<Self> {
        let items: Vec<T> = serde_json::from_str(json)
            .map_err(|e| RinqError::ExecutionError { message: e.to_string() })?;
        Ok(QueryBuilder::from(items))
    }

    pub fn from_json_value(json: &str) -> RinqResult<QueryBuilder<serde_json::Value, Initial>> {
        let items: Vec<serde_json::Value> = serde_json::from_str(json)
            .map_err(|e| RinqError::ExecutionError { message: e.to_string() })?;
        Ok(QueryBuilder::from(items))
    }
}
```

---

## Phase B1: `rinq-stats` — 単一ソース統計

**目的**: `rinq-stats` クレートを新規作成し、`StatisticsExt` 拡張トレイトを提供する。

### ファイル構成

```
rinq-stats/
  Cargo.toml
  src/
    lib.rs
    statistics.rs   — StatisticsExt トレイト実装
    types.rs        — HistogramBucket 等のデータ型
```

### `StatisticsExt` トレイト

```rust
pub trait StatisticsExt<T>: Sized {
    fn variance(self) -> f64      where T: Into<f64>;
    fn std_dev(self) -> f64       where T: Into<f64>;
    fn median(self) -> Option<f64> where T: Into<f64> + Clone + PartialOrd;
    fn mode(self) -> Option<T>    where T: Eq + Hash + Clone;
    fn percentile(self, p: f64) -> Option<f64> where T: Into<f64> + Clone + PartialOrd;
    fn quantile(self, p: f64) -> Option<f64>   where T: Into<f64> + Clone + PartialOrd;
    fn skewness(self) -> f64      where T: Into<f64>;
    fn kurtosis(self) -> f64      where T: Into<f64>;
    fn histogram(self, buckets: usize) -> Vec<HistogramBucket> where T: Into<f64>;
    fn frequency_table(self) -> HashMap<T, usize> where T: Eq + Hash + Clone;
}

impl<T: 'static, S: TypeState> StatisticsExt<T> for QueryBuilder<T, S> { ... }
```

---

## Phase B2: `rinq-stats` — `QueryPair`

**目的**: 2つのデータ系列間の関係を分析する `QueryPair` 型を実装する。

### `QueryPair` の構築

```rust
impl<X: Into<f64> + Clone, Y: Into<f64> + Clone> QueryPair<X, Y> {
    /// 長さが異なる場合は短い方に合わせて truncate し log::warn! を発行する。
    pub fn new(x: Vec<X>, y: Vec<Y>) -> Self

    /// 長さが異なる場合は Err(RinqError::ExecutionError) を返す厳密版。
    pub fn try_new(x: Vec<X>, y: Vec<Y>) -> RinqResult<Self>

    /// QueryBuilder から構築。
    pub fn from_builders(
        x: QueryBuilder<X, impl TypeState>,
        y: QueryBuilder<Y, impl TypeState>,
    ) -> Self
}
```

### 分析メソッド

```rust
impl QueryPair<f64, f64> {
    pub fn covariance(&self) -> f64
    pub fn pearson_correlation(&self) -> f64
    pub fn spearman_correlation(&self) -> f64
    pub fn kendall_tau(&self) -> f64
    pub fn linear_regression(&self) -> (f64, f64)  // (slope, intercept)
}
```

内部実装: Welford のオンラインアルゴリズム（2 パス不要、数値的に安定）。

---

## Phase B3: `rinq-stats` — サンプリング

**目的**: `SamplingExt` 拡張トレイトとして reservoir sampling を提供する。

```rust
pub trait SamplingExt<T>: Sized {
    fn sample_fraction<R: Rng>(self, rng: &mut R, fraction: f64) -> QueryBuilder<T, Filtered>;
    fn sample_n<R: Rng>(self, rng: &mut R, n: usize) -> QueryBuilder<T, Filtered>;
    fn stratified_sample<K, F, R: Rng>(
        self, rng: &mut R, key: F, n_per_stratum: usize,
    ) -> QueryBuilder<T, Filtered>
    where K: Eq + Hash, F: Fn(&T) -> K;
    fn bootstrap_sample<R: Rng>(self, rng: &mut R, n: usize) -> QueryBuilder<T, Filtered>
    where T: Clone;
}
```

`sample_fraction` / `sample_n` は Vitter の Algorithm R（reservoir sampling）で実装し、データ全量を事前に知らなくても均等確率を保証する。

---

## Phase B4: `rinq-stats` — バリデーション

**目的**: `ValidationExt` 拡張トレイトとして ETL 前処理向けバリデーションを提供する。

```rust
pub trait ValidationExt<T>: Sized {
    fn validate(self, rule: impl Fn(&T) -> bool + 'static, message: &str) -> ValidatingBuilder<T>;
}

pub struct ValidatingBuilder<T> {
    inner: Box<dyn Iterator<Item = T>>,
    rules: Vec<(Box<dyn Fn(&T) -> bool>, String)>,
}

impl<T> ValidatingBuilder<T> {
    /// 全要素・全ルールを評価し、違反をすべて収集してから返す。
    pub fn collect_validated(self) -> Result<Vec<T>, Vec<ValidationError>>
}

pub struct ValidationError {
    pub rule: String,
    pub message: String,
    pub index: usize,
}
```

---

## Phase C: ドキュメント・公開準備

**目的**: crates.io への初回公開に向けてドキュメントとメタデータを整備する。

### 作業

- `#![warn(missing_docs)]` を `src/lib.rs` に追加（ゼロ警告ポリシーと連動）
- 全公開 API に `///` コメント（要約・実行種別・`# Examples`・`# Panics`/`# Errors`）
- `#[doc(alias = "...")]` で LINQ 対応名を付与（`SelectMany`, `Where`, `ToList` 等）
- `Cargo.toml` メタデータ追加（`description`, `license`, `repository`, `keywords`, `categories`, `readme`）
- `[package.metadata.docs.rs]` で `all-features = true` 設定
- `README.md` を最終整備（インストール例・クイックスタート・feature flags 一覧）
- `CHANGELOG.md` に v3.0 エントリを追加

### リリース判断基準

- `cargo test` 全件通過（`rinq` + `rinq-stats`）
- `cargo test --doc` 全件通過
- `cargo doc --no-deps` エラーなし
- `cargo clippy -- -D warnings` ゼロ
- `cargo bench --no-run` 通過
- `cargo publish --dry-run` 通過

---

## リスク・注意事項

### ⚠️ E2E テストで判明した型ステートの制約（2026-03-25 確認）

以下は `tests/rinq_e2e_scenarios.rs` の実装時に発見した、既存の型ステートに関する仕様上の制約です。
v3.0 で新規演算子を実装する際・ユーザー向けドキュメントを書く際に必ず考慮してください。

#### 制約 1: `Projected<U>` 状態では `collect` 以外の操作は使えない

`select` の後は `Projected<U>` 状態になり、`enumerate` / `where_` / `flat_map` 等はすべて使えない。
`enumerate` を使う場合は **`select` より前**に置く必要がある。

```rust
// NG: select 後に enumerate は不可（Projected<U> 状態）
.select(|x| x * 2).enumerate()

// OK: enumerate → where_ → select の順（Filtered 状態を保つ）
.enumerate().where_(|(i, _)| i % 2 == 0).select(|(_, x)| x * 2)
```

→ 新規演算子の doc test を書く際はこの順序に従うこと。

#### 制約 2: `Initial` 状態に `select` は存在しない

`QueryBuilder::range` / `QueryBuilder::repeat` / `QueryBuilder::empty` の戻り値は `Initial` 状態であり、`select` は `Filtered` 状態にのみ存在する。生成演算子の直後に変換が必要な場合は `flat_map` で `Filtered` に遷移させる。

```rust
// NG: Initial 状態に select は不可
QueryBuilder::range(1..=10i32).select(|x| x * x)

// OK: flat_map で Filtered に遷移
QueryBuilder::range(1..=10i32).flat_map(|x| std::iter::once(x * x))
```

→ Phase A2 のウィンドウ分析関数（`running_sum` 等）を `Initial` 状態に実装する際も同様に、`Filtered` への遷移を経由する設計とすること。

#### 制約 3: `QueryBuilder::empty()` にターボフィッシュは使えない

`empty()` は型パラメータを持たないため `empty::<T>()` の構文はコンパイルエラーになる。型は変数の型注釈、または使用文脈からの推論で解決する。

```rust
// NG: ターボフィッシュは使えない
QueryBuilder::empty::<i32>()

// OK: 型注釈で明示
let b: QueryBuilder<i32, _> = QueryBuilder::empty();
// または使用文脈から推論される場合はそのまま
QueryBuilder::empty().concat(some_vec_of_i32)
```

→ doc test の例示コードでこの構文を使わないよう注意すること（Phase C）。

#### 制約 4: `MetricsQueryBuilder::new` の引数順序と型

引数順は `(inner: QueryBuilder, metrics: Arc<MetricsCollector>, operation_name: String)` であり、`collector` は `Arc` でラップする必要がある。

```rust
// OK
MetricsQueryBuilder::new(
    QueryBuilder::from(data),
    Arc::new(MetricsCollector::new()),
    "query_name".to_string(),
)
```

→ v3.0 で並列クエリのメトリクス対応（`ParallelQueryBuilder` + `MetricsCollector` の統合）を検討する際に同じシグネチャに合わせること。

---

### `ParallelQueryBuilder` の型パラメータ

rayon の `ParallelIterator` は `Iterator` と別トレイトのため、`QueryBuilder` との共通化はできない。型変換（`into_parallel()`）でコレクションに一度マテリアライズするため、遅延実行のメリットが部分的に失われる点をドキュメントに明記する。

### `moving_average` の `Option<f64>` 戻り値

`select` 等で型が変わるため、`moving_average` 後の `where_` では `Option<f64>` を扱う必要がある。spec に記載のとおり、ユーザーが `where_(|v| v.is_some())` で明示的に対処する設計とする。

### `rinq-stats` と `rinq` のバージョン同期

`rinq-stats` は `rinq` の公開 API に依存するため、`rinq` のメジャーバンプ時には `rinq-stats` も追従が必要。`rinq = "~0.x"` のように緩めの制約を Cargo.toml に記載する。

### Welford アルゴリズムの精度

Welford の逐次更新式は数値的に安定だが、極端に大きな値や小さな値が混在する場合は浮動小数点誤差が蓄積する。`f64` の精度を超えるケースは仕様の制限として文書化する。
