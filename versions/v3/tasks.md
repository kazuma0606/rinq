# RINQ v3.0 タスク管理

チェックボックスの凡例: `[ ]` 未着手 / `[~]` 進行中 / `[x]` 完了

---

## Phase A1: 並列処理（`feature = "parallel"`） ✅ 完了

### Cargo.toml
- [x] `rayon = { version = "1.10", optional = true }` を追加
- [x] `serde` / `serde_json` の optional 依存を追加
- [x] `[features]` セクションを追加（`default = []`, `parallel`, `serde`）
- [x] `[package.metadata.docs.rs]` に `all-features = true` を追加

### `ParallelQueryBuilder` 構造体
- [x] `src/parallel/mod.rs` を作成（`ParallelQueryBuilder<T, State>` 構造体）
- [x] `src/parallel/initial.rs` — `impl ParallelQueryBuilder<T, Initial>`
- [x] `src/parallel/filtered.rs` — `impl ParallelQueryBuilder<T, Filtered>`
- [x] `src/parallel/sorted.rs` — `impl ParallelQueryBuilder<T, Sorted>`
- [x] `src/parallel/shared.rs` — 終端操作（`par_count`, `par_sum`, `collect` 等）

### `QueryBuilder::into_parallel`
- [x] `src/core/builder/shared.rs` に `#[cfg(feature = "parallel")] fn into_parallel(self)` を追加

### 実装するメソッド
- [x] `par_where`（Initial/Filtered → Filtered）
- [x] `par_select`（Filtered → Filtered）
- [x] `par_flat_map`（Initial/Filtered → Filtered）
- [x] `par_order_by`（Initial/Filtered → Sorted）
- [x] `par_count`（全状態、終端）
- [x] `par_sum`（全状態、終端）
- [x] `par_min` / `par_max`（全状態、終端）
- [x] `par_any` / `par_all`（全状態、終端）
- [x] `collect`（全状態、終端）
- [x] `par_group_by`（全状態、終端）

### doc tests
- [x] `par_where` + `par_sum` の基本例
- [x] `into_parallel` → `par_select` → `collect` の例
- [x] `par_group_by` の例

### 統合テスト（`tests/rinq_parallel_tests.rs` を新規作成）
- [x] 基本的な `par_where` + `collect` の動作確認
- [x] `QueryBuilder::into_parallel` からの変換テスト
- [x] `par_sum` が逐次 `sum` と同じ結果を返すことを確認
- [x] `par_group_by` が逐次 `group_by` と同じ結果を返すことを確認（ただしHashMapなので順序は問わない）
- [x] 空コレクションに対する各 par_* 操作
- [x] `T: Send` が満たされる型での動作確認

### テスト確認
- [x] `cargo test` 全件通過（361テスト）
- [x] `cargo test --features parallel` 全件通過（427テスト: +66並列テスト）
- [x] `cargo clippy --features parallel -- -D warnings` ゼロ

### ✅ Phase A1 完了チェック
- [x] 上記すべて完了（2026-03-25）

---

## Phase A2: ウィンドウ分析関数 ✅ 完了

### 実装方針
全7メソッドを `src/core/builder/window.rs` に集約（`impl<T: 'static, State>` 単一ブロック）。
`into_vec()` ヘルパーを `shared.rs` の `pub(crate)` メソッドとして追加し、全ウィンドウ関数から再利用。

### `running_sum`
- [x] `src/core/builder/window.rs` に全状態共通実装（`impl<T: 'static, State>`）
- [x] doc test 追加（`[1,2,3,4,5]` → `[1,3,6,10,15]`）

### `running_average`
- [x] 全状態に `running_average` を実装
- [x] doc test 追加

### `moving_average`
- [x] `src/core/builder/iterators.rs` に `MovingAverageIterator` を追加
- [x] 全状態に `moving_average(window: usize) -> QueryBuilder<Option<f64>, Filtered>` を実装
- [x] `window == 0` の場合 `assert!` でパニックすることを確認・テスト済み
- [x] doc test 追加（先頭 n-1 件が `None`、`where_+select` で除外する例）

### `rank_by` / `dense_rank_by`
- [x] 全状態に `rank_by` を実装（戻り値: `(usize, T)`）
- [x] 全状態に `dense_rank_by` を実装
- [x] doc test で rank（スキップあり）vs dense_rank（スキップなし）を示す

### `lag` / `lead`
- [x] 全状態に `lag(n) -> QueryBuilder<(Option<T>, T), Filtered>` を実装
- [x] 全状態に `lead(n) -> QueryBuilder<(T, Option<T>), Filtered>` を実装
- [x] doc test 追加（先頭・末尾の `None` の扱い）

### 注意事項
- `order_by` は `Filtered` / `Initial` 状態のみ。`select` → `Projected<U>` に遷移後は使用不可。
  `flat_map(|v| std::iter::once(...))` で `Filtered` に留まるワークアラウンドを使用。

### 統合テスト（`tests/rinq_window_tests.rs`）
- [x] `running_sum` — 空 / 単一要素 / 複数要素 / filter後 / sort後
- [x] `running_average` — 精度確認 / 空 / filter後
- [x] `moving_average` — None位置 / window=1 / window=len / window>len / 空 / window=0パニック / None除外 / filter後
- [x] `rank_by` — 同値なし / 全同値 / スキップ確認 / 空 / 単一 / 入力順保持
- [x] `dense_rank_by` — 同値なし / スキップなし / rank vs dense_rank 比較 / 全同値
- [x] `lag(1)` / `lag(2)` / `lag(0)` / lag>len / 空 / where_チェーン
- [x] `lead(1)` / `lead(2)` / `lead(0)` / lead>len / 空 / where_チェーン
- [x] 複合チェーンテスト（running_sum+where_, ma+order_by等）

### テスト確認
- [x] `cargo test` 全件通過（472テスト: +45ウィンドウテスト）
- [x] `cargo clippy -- -D warnings` ゼロ
- [x] `cargo clippy --features parallel -- -D warnings` ゼロ

### ✅ Phase A2 完了チェック
- [x] 上記すべて完了（2026-03-25）

---

## Phase A3: 失敗許容パイプライン ✅ 完了

### 実装方針
- `try_select` / `try_where_` を `src/core/builder/try_ops.rs` に集約（`impl<T: 'static, State>`）。
- `TryQueryBuilder<T, E>` を `src/core/try_builder.rs` に配置。

### `TryQueryBuilder<T, E>` 型
- [x] `src/core/try_builder.rs` を作成
- [x] `collect_partitioned(self) -> (Vec<T>, Vec<E>)` を実装
- [x] `collect_results(self) -> Result<Vec<T>, E>` を実装

### `try_select`
- [x] 全状態共通 `src/core/builder/try_ops.rs` に実装
- [x] doc test 追加（`parse::<i32>()` + `collect_partitioned` の例）

### `try_where_`
- [x] 全状態に `try_where_` を実装
- [x] doc test 追加（`Ok(bool)` / `Err` 分岐の例）

### `lib.rs` への re-export
- [x] `TryQueryBuilder` をクレートルートに re-export

### 統合テスト（`tests/rinq_try_tests.rs`）
- [x] `try_select` + `collect_partitioned` — 全成功 / 一部失敗 / 全失敗 / 空 / 型変換 / 順序保持
- [x] `try_select` + `collect_results` — 全成功 / 一部失敗 / 空 / `RinqError` マッピング
- [x] `collect_results` が最初の Err で中断することを atomic カウンタで検証
- [x] `try_where_` — keep/drop/Err伝播 / `collect_partitioned` / 空 / チェーン後
- [x] `try_select` / `try_where_` を `where_` / `order_by` チェーン後に呼び出し
- [x] 100件 × 10個エラーの大規模ケース

### テスト確認
- [x] `cargo test` 全件通過（492テスト: +20 try テスト）
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Phase A3 完了チェック
- [x] 上記すべて完了（2026-03-25）

---

## Phase A4: serde 統合（`feature = "serde"`） ✅ 完了

### 実装方針
- `from_json` / `from_json_value` を `src/core/builder/serde_ops.rs` に集約（`#[cfg(feature = "serde")]`）
- `src/serde/mod.rs` は `QueryBuilder` の re-export のみ（`use rinq::serde::QueryBuilder` パターン用）
- `serde_json::PartialEq<Value> for i32` が既存テストの型推論を壊す問題 → `assert_eq!(r, vec![])` を `assert!(r.is_empty())` に変更（`core_tests.rs` 8箇所 + `initial.rs` doc test 1箇所）

### 実装
- [x] `src/serde/mod.rs` を作成（`QueryBuilder` re-export）
- [x] `src/core/builder/serde_ops.rs` を作成
- [x] `QueryBuilder::<T, Initial>::from_json(json: &str) -> RinqResult<Self>` を実装（`T: DeserializeOwned`）
- [x] `QueryBuilder::<Value, Initial>::from_json_value(json: &str) -> RinqResult<Self>` を実装

### doc tests
- [x] `#[derive(Deserialize)]` した `Point` 構造体への `from_json` 例
- [x] `from_json_value` の例（`v["age"]` アクセス）

### 統合テスト（`tests/rinq_serde_tests.rs`）
- [x] `from_json` — 配列パース / 空配列 / 不正JSON / スキーマ不一致 / i32配列 / String配列
- [x] `from_json` + フィルタ・ソート・count・ネスト構造体
- [x] `from_json_value` — 基本 / 空 / 不正JSON / 動的フィールドアクセス
- [x] `use rinq::serde::QueryBuilder` パスで動作確認
- [x] `from_json` + `running_sum` / `try_select` との組み合わせ

### テスト確認
- [x] `cargo test` 全件通過（デフォルト feature）
- [x] `cargo test --features serde` 全件通過（509テスト: +17 serdeテスト）
- [x] `cargo clippy --features serde -- -D warnings` ゼロ

### ✅ Phase A4 完了チェック
- [x] 上記すべて完了（2026-03-25）

---

## Phase B1: `rinq-stats` 単一ソース統計 ✅ 完了

### クレート作成
- [x] `Cargo.toml` にワークスペース設定を追加（`members = [".", "rinq-stats"]`）
- [x] `rinq-stats/Cargo.toml` を作成（`rinq`, `rand` (small_rng feature), `log` 依存）
- [x] `rinq-stats/src/lib.rs` を作成
- [x] `rinq-stats/src/statistics.rs` — `StatisticsExt` トレイト実装
- [x] `rinq-stats/src/types.rs` — `HistogramBucket` データ型

### 実装するメソッド
- [x] `variance` / `std_dev`（母分散・母標準偏差）
- [x] `median`（ソート後の中央値、偶数個は平均）
- [x] `mode`（最頻値、`T: Eq + Hash + Clone`）
- [x] `percentile(p: f64)` / `quantile(p: f64)`（nearest rank 法）
- [x] `skewness`（第3標準化モーメント）/ `kurtosis`（超過尖度）
- [x] `histogram(buckets: usize) -> Vec<HistogramBucket>`
- [x] `frequency_table() -> HashMap<T, usize>`

### 統合テスト（`rinq-stats/tests/statistics_tests.rs`）: 42件
- [x] `variance` — 既知値確認 / 単一 / 空 / 同値 / 2要素
- [x] `std_dev` — `√variance` との一致 / 既知値 / 空
- [x] `median` — 奇数個 / 偶数個 / 単一 / 空 / ソート済み / 逆順
- [x] `mode` — 単一最頻値 / 全ユニーク / 空 / 単一要素
- [x] `percentile` — p50=median / p0=min / p1=max / 空 / quantile alias
- [x] `skewness` — 対称=0 / 右歪み(+) / 左歪み(-) / 要素数不足
- [x] `kurtosis` — 一様(<0) / 尖鋒(>0) / 要素数不足
- [x] `histogram` — バケット数 / 全件合計 / 空 / 0バケット / 単一バケット / 同値
- [x] `frequency_table` — 基本 / 合計 / 空 / 全ユニーク
- [x] filter後・sort後のチェーンテスト

## Phase B2: `rinq-stats` — `QueryPair` ✅ 完了

### 実装（`rinq-stats/src/pair.rs`）
- [x] `QueryPair::new`（truncate + `log::warn!`）
- [x] `QueryPair::try_new`（長さ不一致は `Err(QueryPairError)`）
- [x] `QueryPair::from_builders`（2つの `QueryBuilder` から構築）
- [x] `covariance`（2パス法）
- [x] `pearson_correlation`（`covariance / (σ_x * σ_y)`）
- [x] `spearman_correlation`（順位変換後の Pearson）
- [x] `kendall_tau`（concordant/discordant ペア計数, O(n²)）
- [x] `linear_regression`（最小二乗法、`(slope, intercept)` 返却）

### 統合テスト（`rinq-stats/tests/pair_tests.rs`）: 28件

## Phase B3: `rinq-stats` — サンプリング ✅ 完了

### 実装（`rinq-stats/src/sampling.rs`）
- [x] `sample_fraction` — Vitter Algorithm R（ceil(fraction×n) 件）
- [x] `sample_n` — Vitter Algorithm R（ちょうど k 件 or 全件）
- [x] `stratified_sample` — グループごとに `reservoir_sample`
- [x] `bootstrap_sample` — 復元抽出（`rng.gen_range` ベース）

### 統合テスト（`rinq-stats/tests/sampling_tests.rs`）: 22件

### テスト確認
- [x] `cargo test -p rinq-stats` 全件通過（116テスト: 42統計+28ペア+22サンプリング+24doc）
- [x] `cargo clippy -p rinq-stats -- -D warnings` ゼロ
- [x] `cargo test -p rinq` 全件通過（影響なし）

### ✅ Phase B1/B2/B3 完了チェック
- [x] 上記すべて完了（2026-03-25）

---

## Phase B4: `rinq-stats` — バリデーション ✅ 完了

### 実装（`rinq-stats/src/validation.rs`）
- [x] `rinq-stats/src/validation.rs` を作成
- [x] `ValidationExt` トレイト（`validate` メソッド）
- [x] `ValidationQueryBuilder<T>` 型（ルールを蓄積するビルダー）
- [x] `collect_validated() -> Result<Vec<T>, Vec<ValidationError>>` — 全違反を収集
- [x] `collect_valid() -> Vec<T>` — 全ルールを通過した要素のみ返す
- [x] `collect_invalid() -> Vec<(T, Vec<ValidationError>)>` — 違反要素と理由を返す
- [x] `ValidationError` 構造体（`rule`, `message`, `index`; `Display`, `Clone`, `PartialEq`）

### 統合テスト（`rinq-stats/tests/validation_tests.rs`）: 24件
- [x] 全要素がルールを満たす → `Ok(Vec<T>)`
- [x] 一部の要素が違反する → `Err(Vec<ValidationError>)` でインデックスを確認
- [x] 複数ルールの連鎖 → 複数ルールの違反が同一要素に対して正しく収集される
- [x] 空コレクション → `Ok(vec![])`
- [x] `ValidationError.index` が元のシーケンス内での位置と一致することを確認

### テスト確認
- [x] `cargo test -p rinq-stats` 全件通過（142件: 42統計+28ペア+22サンプリング+24バリデーション+26doc）
- [x] `cargo clippy -p rinq-stats -- -D warnings` ゼロ

### ✅ Phase B4 完了チェック
- [x] 上記すべて完了（2026-03-25）

---

## Phase C: ドキュメント・公開準備 ✅ 完了

### `rinq` 本体
- [x] `src/lib.rs` に `#![warn(missing_docs)]` を追加
- [x] `src/lib.rs` のクレートレベル `//!` コメントを整備（概要・クイックスタート・feature flags）
- [x] `RinqError` 全フィールドに `///` コメントを追加
- [x] `#![allow(missing_docs)]` を内部実装モジュール（`core`, `metrics`, `parallel`, `serde`）に追加（詳細実装は非公開）
- [x] `Cargo.toml` に `description`, `license`, `repository`, `keywords`, `categories`, `readme` を追加（version 3.0.0 に更新）
- [x] `[package.metadata.docs.rs] all-features = true` は既存

### `rinq-stats`
- [x] `Cargo.toml` に `description`, `license`, `repository`, `keywords`, `categories`, `readme` を追加（version 3.0.0 に更新）
- [x] `[package.metadata.docs.rs] all-features = true` を追加

### `README.md`
- [x] インストール方法（`Cargo.toml` への追記例）
- [x] 最短のクイックスタートコード
- [x] feature flags 一覧
- [x] `rinq-stats` へのリンク（全トレイト一覧付き）
- [x] ライセンス表記

### CHANGELOG
- [x] `CHANGELOG.md` に v3.0 エントリを追加
- [x] Breaking Changes なし を明記
- [x] Phase A1〜B4 および Phase C の全追加項目を列挙

### 最終確認
- [x] `cargo test` 全件通過（`rinq` + `rinq-stats`）
- [x] `cargo test --features parallel,serde` 全件通過
- [x] `cargo test --doc` 全件通過
- [x] `cargo doc --no-deps --all-features` エラーなし
- [x] `cargo clippy --all-features -- -D warnings` ゼロ
- [x] `cargo bench --no-run` 通過

### ✅ Phase C 完了チェック / RINQ v3.0 リリース
- [x] 上記すべて完了（2026-03-25）

---

## 付録: E2E テストで判明した型ステートの制約

> 出典: `tests/rinq_e2e_scenarios.rs` 実装時（2026-03-25）
> 参照: `tests/e2e_results_2026-03-25.md`

新規演算子の実装・doc test 作成・ユーザー向けコード例を書く際に必ず確認すること。

### 制約 1: `Projected<U>` 状態では `collect` 以外の操作は使えない

- `select` の後は `Projected<U>` 状態になり、`enumerate` / `where_` / `flat_map` 等は使えない
- `enumerate` を使う場合は `select` より前に置く

```rust
// NG
.select(|x| x * 2).enumerate()
// OK
.enumerate().where_(|(i, _)| i % 2 == 0).select(|(_, x)| x * 2)
```

**実装タスクへの影響**:
- [ ] Phase A2 ウィンドウ分析関数の doc test で `select` → `enumerate` の順になっていないか確認
- [ ] Phase A3 `try_select` の doc test でも同様に確認

### 制約 2: `Initial` 状態に `select` は存在しない

- `range` / `repeat` / `empty` の直後に `select` は使えない
- `flat_map` で `Filtered` に遷移させてから `select` を使う

```rust
// NG
QueryBuilder::range(1..=10i32).select(|x| x * x)
// OK
QueryBuilder::range(1..=10i32).flat_map(|x| std::iter::once(x * x))
```

**実装タスクへの影響**:
- [ ] Phase A2 ウィンドウ分析関数を `Initial` 状態に実装する際、`Filtered` への遷移経由で設計すること
- [ ] Phase C doc test で生成演算子（`range` / `repeat`）に `select` を直接繋げるコード例を書かない

### 制約 3: `QueryBuilder::empty()` にターボフィッシュは使えない

- `empty::<T>()` の構文はコンパイルエラー
- 型は変数の型注釈または使用文脈の推論で解決する

```rust
// NG
QueryBuilder::empty::<i32>()
// OK
let b: QueryBuilder<i32, _> = QueryBuilder::empty();
```

**実装タスクへの影響**:
- [ ] Phase C doc test で `empty()` の使用例を正しい構文で記載すること

### 制約 4: `MetricsQueryBuilder::new` の引数順序と型

- 引数順: `(inner, Arc<MetricsCollector>, operation_name: String)`
- `MetricsCollector` は `Arc` でラップが必要

**実装タスクへの影響**:
- [ ] Phase A1 `ParallelQueryBuilder` と `MetricsCollector` の統合を検討する際、同シグネチャに合わせること
