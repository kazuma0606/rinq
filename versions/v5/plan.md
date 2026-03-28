# RINQ v5.0 実装計画

**作成日**: 2026-03-28

---

## 目標

v4.0 で完成した機能セットを crates.io 初回公開（0.1.0）に向けて整理し、テスト・ベンチマーク・ドキュメントを補完したうえで、JOIN 操作・クエリ充実・統計拡張を追加する。公開 API は可能な限り後方互換を維持する。

---

## 設計方針

### 変更しないもの
- 型ステートパターン（`Initial → Filtered → Sorted / Projected`）
- 既存の全公開 API（破壊的変更なし）
- 依存クレートのメジャーバージョン

### 追加・変更するもの
- **ディレクトリ構造**: `rinq` 本体を `rinq/` サブディレクトリへ移動
- **バージョン統一**: 全クレートを `0.1.0` に統一
- **テスト補完**: 未テスト演算子の統合テスト
- **ベンチマーク拡充**: v4 全演算子のベンチマーク
- **新演算子（5D）**: ターミナル系 — `for_each`/`to_sorted_vec`/`take_last`/`skip_last`/`count_by`/`sum_by`/`average_by`/`reduce`/`all_unique`/`none`
- **新演算子（5E）**: クエリ系 — `frequencies`/`flatten`/`position`/`find`/`index_of`/`nth`/`batch`/`exactly_one`/`tee`
- **JOIN（5F）**: `inner_join`/`left_join`/`cross_join` + rinq-syntax `join` 節
- **rinq-stats 拡張（5G）**: 数値変換・時系列拡張・外れ値スコア・ValidationExt 強化
- **CI/CD・OSS 整備（5I）**: GitHub Actions CI・LICENSE・CONTRIBUTING.md・rinq/README.md

### 波及方針（新演算子と他ビルダー）
5D・5E・5F の新演算子は **QueryBuilder のみ** が v5.0 の対象。
MetricsQueryBuilder / ParallelQueryBuilder への追随は v5.1 以降。

---

## フェーズ構成

```
Phase 5A: 整理・品質向上（ディレクトリ再構成・バージョン統一・README 整備）
  ↓
Phase 5B: テスト補完（未テスト演算子・組み合わせテスト・エッジケース）
  ↓
Phase 5C: ベンチマーク拡充（v4 全演算子・Window 関数・rinq-stats）
  ↓
Phase 5D: ターミナル演算子強化
  ↓
Phase 5E: クエリ充実
  ↓
Phase 5F: JOIN 操作
  ↓
Phase 5G: rinq-stats 統計拡張
  ↓
Phase 5H: 公開準備（examples 拡充・dry-run）
  ↓
Phase 5I: CI/CD・OSS 整備（GitHub Actions・LICENSE・CONTRIBUTING・rinq/README）
```

各フェーズは `cargo test` 全件通過・`cargo clippy -- -D warnings` ゼロを確認してから完了とする。

---

## OSS 方針

### プロジェクトの思想

rinq は「公式クレート」を目指すものではない。**Rust をもっと便利に使えたら、より多くの人に価値を届けられる** という思想のもとで作られている。Rust のイテレータ API は強力だが、filter → sort → aggregate のパイプラインを毎回ゼロから書くのは冗長だ。rinq はその冗長さを取り除き、Rust を読みやすく・始めやすくすることを目的とする。

> "If rinq makes even one person think 'oh, Rust can be this readable' — that's the whole point."

### AI ツールの活用

このプロジェクト自体が AI 支援開発（Claude Code）で設計・実装されている。コントリビューターも AI ツールの使用を歓迎する。完璧な PR よりも、動いてテストが通る貢献を優先する。

- AI を使って書いたコードは `# AI-assisted` をコミットメッセージに含めることを**推奨（任意）**
- AI を使っていても `cargo test` / `cargo clippy` 通過は必須

### ドキュメントの言語方針

| 対象 | 言語 | 理由 |
|---|---|---|
| `///` doc コメント（API ドキュメント） | **英語** | docs.rs に公開される |
| `rinq/README.md` 他サブクレート README | **英語** | crates.io のトップページになる |
| `CONTRIBUTING.md` | **英語** | コントリビューターへの入口 |
| `CHANGELOG.md` | **英語** | リリースノートとして参照される |
| `.github/` テンプレート | **英語** | Issue/PR を開く人向け |
| `versions/v*/` （設計ドキュメント） | **日本語のまま** | 内部 AI コーディング用の開発ログ。翻訳不要 |

`versions/` は内部の設計・実装ログであり、コントリビューターが貢献するために読む必要はない。`rinq/README.md` に以下の一文を添えることで十分に説明できる:

```markdown
## Development Process

This crate is developed with AI-assisted design. Internal planning documents
(`versions/`) are written in Japanese — the development log of how rinq
grew from v1 to v5.
```

### コントリビューターへの要求レベル

`versions/` の spec → plan → tasks → tests という開発フローは **AI コーディング最適化された内部ワークフロー** であり、コントリビューターに同じプロセスを求めない。

| 貢献規模 | 例 | 要求事項 |
|---|---|---|
| 小（typo・doc 修正） | README/コメントの誤字修正 | PR のみ。Issue 不要 |
| 中（バグ修正・テスト追加） | 既存演算子のバグ、テスト補完 | PR + 再現手順の記載 |
| 大（新演算子・新クレート） | 新しい演算子・機能追加 | Issue で設計議論 → 承認後に実装 |

いずれの規模でも `cargo test --workspace` と `cargo clippy --workspace -- -D warnings` の通過を必須とする。

---

## Phase 5A: 整理・品質向上

**目的**: ディレクトリ構造をサブクレートと統一し、公開準備のベースを整える。

### ファイル構成（変更）

```
rusted-ca/
  Cargo.toml             ← [workspace] のみに変更
  rinq/                  ← 新規: rinq クレート本体
    Cargo.toml           ← 新規
    src/                 ← git mv src rinq/src
    tests/               ← git mv tests rinq/tests
    benches/             ← git mv benches rinq/benches
    examples/            ← git mv examples rinq/examples
    README.md            ← git mv README.md rinq/README.md
  rinq-stats/
    Cargo.toml           ← path: "../rinq" に更新、version: "0.1.0"、readme: "README.md"
    README.md            ← 新規作成
  rinq-derive/
    Cargo.toml           ← path: "../rinq" に更新、version: "0.1.0"
    README.md            ← 新規作成
  rinq-syntax/
    Cargo.toml           ← path: "../rinq" に更新、version: "0.1.0"
    README.md            ← 新規作成
```

### 5A-1: git mv によるディレクトリ移動

```bash
git mv src rinq/src
git mv tests rinq/tests
git mv benches rinq/benches
git mv examples rinq/examples
git mv README.md rinq/README.md
```

**注意**: `git mv` を使うことでファイル履歴を保持する。

### 5A-2: rinq/Cargo.toml 新規作成

```toml
[package]
name = "rinq"
version = "0.1.0"
edition = "2024"
description = "Type-safe, zero-cost LINQ-inspired query engine for Rust — filter, sort, aggregate, window analytics, parallel execution, and statistical extensions."
license = "MIT"
repository = "https://github.com/yoshidev/rusted-ca"
keywords = ["query", "linq", "iterator", "collections", "analytics"]
categories = ["data-structures", "algorithms"]
readme = "README.md"

[features]
default  = []
parallel = ["dep:rayon"]
serde    = ["dep:serde", "dep:serde_json"]

[dependencies]
thiserror   = "1.0"
num-traits  = "0.2"
parking_lot = "0.12"
rayon       = { version = "1.10", optional = true }
serde       = { version = "1.0", optional = true, features = ["derive"] }
serde_json  = { version = "1.0", optional = true }

[dev-dependencies]
proptest    = "1.0"
criterion   = "0.5"
rinq-derive = { path = "../rinq-derive" }

[package.metadata.docs.rs]
all-features = true

[[bench]]
name    = "rinq_benchmarks"
harness = false

[[bench]]
name    = "rinq_v0_2_benchmarks"
harness = false

[[bench]]
name    = "rinq_v4_benchmarks"
harness = false
```

### 5A-3: ルート Cargo.toml 変更

```toml
# workspace のみ — [package] セクションは全削除
[workspace]
members  = ["rinq", "rinq-stats", "rinq-derive", "rinq-syntax"]
resolver = "2"
```

### 5A-4: バージョン統一

全サブクレートの `version` を `"0.1.0"` に変更する。
`README.md` 内のインストール例も `rinq = "0.1"` に更新。

### 5A-5: README 新規作成

**rinq-stats/README.md**: StatisticsExt・SamplingExt・ValidationExt・TimeSeriesExt・OutlierExt のクイックリファレンス

**rinq-derive/README.md**: `#[derive(Queryable)]` / `#[derive(QueryableFrom)]` のクイックスタート・属性リスト

**rinq-syntax/README.md**: `query!` 構文リファレンス・binding semantics・Experimental 注記

### 5A-6: CLAUDE.md 更新

新ディレクトリ構造に合わせてモジュール構造図・コマンド・テストファイル一覧・演算子テーブルを全面更新。

---

## Phase 5B: テスト補完

**目的**: v4 で追加した演算子のうち専用テストが存在しないものをカバーする。

### ファイル構成（追加）

```
rinq/tests/
  rinq_v5_tests.rs    ← 新規: 5B の統合テスト
```

### 5B-1: 未テスト演算子の統合テスト

対象: `tap_each` / `tap_collect` / `pipe` / `cycle` / `step_by` / `map` / `collect_vec`

各演算子について最低 3 パターン（正常・空コレクション・エッジケース）をカバー。

### 5B-2: 組み合わせテスト

- `parallel` feature + `serde` feature + `MetricsQueryBuilder` の組み合わせ
- `rinq-derive` の `#[derive(Queryable)]` + `pairwise` / `scan` / `zip_with`
- `rinq-syntax` の `query!` + `rinq-derive` の `#[derive(Queryable)]`
- 100 万件の `Vec<i32>` での `where_` / `order_by` / `group_by` の正常動作

### 5B-3: エッジケース補強

- `pairwise()` — 0・1・2 要素
- `intersperse()` — 空・1 要素
- `dedup_by()` — 複合キー・全同値・全異値
- `unfold()` — `take` で早期終了
- `lag(0)` / `lead(0)` — 境界値

---

## Phase 5C: ベンチマーク拡充

**目的**: v4 で追加した全演算子のゼロコスト性能を数値で確認する。

### ファイル構成（追加）

```
rinq/benches/
  rinq_v4_benchmarks.rs    ← 新規
rinq-stats/benches/
  rinq_stats_benchmarks.rs ← 新規
```

**注意**: `rinq-stats/Cargo.toml` に `[[bench]]` エントリを追加する。

### 5C-1: v4 演算子ベンチマーク（rinq_v4_benchmarks.rs）

```
group: "functional"
  scan_cumulative_sum / chunk_by / dedup_consecutive / zip_with_add
  pairwise / intersperse / min_max / filter_map_parse / step_by_2

group: "window"
  running_sum / moving_average_10 / rank_by / lag_1 / lead_1

group: "lifecycle"
  tap_each_noop / tap_collect_vec / pipe_identity / from_arc_cloned

group: "generation"
  unfold_fib_take100 / cycle_take1000
```

各ベンチマークは `1_000` 要素と `10_000` 要素の 2 スケールを計測。

### 5C-2: rinq-stats ベンチマーク（rinq_stats_benchmarks.rs）

```
group: "statistics"   — variance / median / percentile_95 / histogram
group: "timeseries"   — ema / bollinger_bands
group: "outliers"     — zscore / iqr
group: "sampling"     — sample_n / stratified
```

---

## Phase 5D: ターミナル演算子強化

**目的**: `collect()` の前に使うターミナル系を充実させる。

### ファイル構成（変更）

```
rinq/src/core/builder/
  shared.rs     ← for_each / to_sorted_vec / to_sorted_vec_desc / take_last / skip_last
                   count_by / sum_by / average_by / reduce / all_unique / none を追加
```

### 各演算子設計

**`for_each(f)`**
```rust
pub fn for_each<F>(self, mut f: F)
where F: FnMut(T)
// tap_each との違い: Self を返さない消費型ターミナル
```

**`to_sorted_vec(key)`**
```rust
pub fn to_sorted_vec<K, F>(self, key: F) -> Vec<T>
where
    F: Fn(&T) -> K + 'static,
    K: Ord + 'static,
// order_by(key).collect() の等価ショートハンド
```

**`take_last(n)` / `skip_last(n)`**
```rust
pub fn take_last(self, n: usize) -> Vec<T>
pub fn skip_last(self, n: usize) -> Vec<T>
// ⚠ Eagerly collects all elements
```

**`count_by(pred)`**
```rust
pub fn count_by<F>(self, pred: F) -> usize
where F: Fn(&T) -> bool
// where_(pred).count() より効率的（中間コレクションなし）
```

**`sum_by(key)` / `average_by(key)`**
```rust
pub fn sum_by<N, F>(self, key: F) -> N
where F: Fn(T) -> N, N: Default + std::ops::Add<Output = N>

pub fn average_by<F>(self, key: F) -> Option<f64>
where F: Fn(T) -> f64
```

**`reduce(f)`**
```rust
pub fn reduce<F>(self, f: F) -> Option<T>
where F: FnMut(T, T) -> T
// aggregate_no_seed の alias
```

**`all_unique()`**
```rust
pub fn all_unique(self) -> bool
where T: Hash + Eq + 'static
```

**`none(pred)`**
```rust
pub fn none<F>(self, pred: F) -> bool
where F: Fn(&T) -> bool
// !self.any(pred) と等価
```

---

## Phase 5E: クエリ充実

**目的**: 言語的に自然な名前の別名と、よく使うパターンを演算子として提供する。

### ファイル構成（変更）

```
rinq/src/core/builder/
  shared.rs      ← frequencies / flatten / position / find / index_of / nth /
                    batch / exactly_one / tee を追加
```

### 各演算子設計

**`frequencies()`**
```rust
pub fn frequencies(self) -> HashMap<T, usize>
where T: Hash + Eq + 'static
```

**`flatten()`**
```rust
pub fn flatten<U>(self) -> QueryBuilder<U, Filtered>
where T: IntoIterator<Item = U> + 'static, U: 'static
// flat_map(|x| x) の alias
```

**`position(pred)`**
```rust
pub fn position<F>(self, pred: F) -> Option<usize>
where F: Fn(&T) -> bool
```

**`find(pred)`**
```rust
pub fn find<F>(self, pred: F) -> Option<T>
where F: Fn(&T) -> bool
// first(pred) の alias
```

**`index_of(value)`**
```rust
pub fn index_of(self, value: &T) -> Option<usize>
where T: PartialEq
```

**`nth(n)`** — `element_at(n)` の alias

**`batch(n)`** — `chunk(n)` の alias

**`exactly_one()`** — `single()` の alias

**`tee()`**
```rust
pub fn tee(self) -> (Vec<T>, Vec<T>)
where T: Clone + 'static
// 同一ストリームを 2 つの Vec に複製
```

---

## Phase 5F: JOIN 操作

**目的**: 2 つの QueryBuilder を結合する演算子を追加する。

### ファイル構成（追加・変更）

```
rinq/src/core/builder/
  join.rs        ← 新規: inner_join / left_join / cross_join
  mod.rs         ← join モジュールを追加
rinq-syntax/src/
  parser.rs      ← join 節のパース追加
  codegen.rs     ← inner_join / left_join への展開追加
  ast.rs         ← Clause::Join を追加
```

### 演算子設計

**`inner_join`**
```rust
pub fn inner_join<U, K, FK, GK>(
    self,
    other: QueryBuilder<U, impl TypeState>,
    left_key: FK,
    right_key: GK,
) -> QueryBuilder<(T, U), Filtered>
where
    K: Hash + Eq + 'static,
    FK: Fn(&T) -> K + 'static,
    GK: Fn(&U) -> K + 'static,
    U: Clone + 'static,
    T: Clone + 'static,
```

**実装方針**: 右辺を HashMap に収集してから左辺を走査（O(N+M)）。

**`left_join`**
```rust
pub fn left_join<U, K, FK, GK>(
    self,
    other: QueryBuilder<U, impl TypeState>,
    left_key: FK,
    right_key: GK,
) -> QueryBuilder<(T, Option<U>), Filtered>
```

**`cross_join`**
```rust
pub fn cross_join<U>(
    self,
    other: QueryBuilder<U, impl TypeState>,
) -> QueryBuilder<(T, U), Filtered>
where U: Clone + 'static, T: Clone + 'static
// O(N×M): 右辺を Vec に収集して左辺×右辺の直積
```

### rinq-syntax 拡張

```
query! {
    from order in orders
    join customer in customers on order.customer_id == customer.id
    where order.total > 100.0
    select (order, customer)
}
```

展開: `.inner_join(customers, |order| order.customer_id, |customer| customer.id)`

`left join` → `.left_join(...)` に展開。

---

## Phase 5G: rinq-stats 統計拡張

**目的**: 数値変換・時系列・外れ値スコア・ValidationExt を拡張する。

### ファイル構成（変更・追加）

```
rinq-stats/src/
  transform.rs    ← 新規: normalize / standardize / weighted_average /
                           outlier_scores_zscore / percentile_filter / cumulative_distribution
  timeseries.rs   ← 既存: simple_moving_average / weighted_moving_average /
                           rate_of_change / seasonal_decompose を追加
  outliers.rs     ← 既存: remove_outliers_modified_zscore / outlier_scores_iqr を追加
  validation.rs   ← 既存: validate_range / validate_unique / validate_non_empty / report を追加
```

### 重要な設計メモ

**`normalize()`** — `(x - min) / (max - min)`: 全要素を 2 回走査（min/max 取得 → 正規化）。全同値の場合は全要素 0.0 を返す。

**`standardize()`** — `(x - mean) / std_dev`: 全要素を 2 回走査（mean/std_dev 取得 → 変換）。std_dev=0 の場合は全要素 0.0 を返す。

**`validate_unique(key_fn)`** — `HashMap` で出現カウント → 重複があれば ValidationError を記録。

**`report()`** — `ValidationResult` 全件を文字列形式（`Vec<String>`）で返す。構造化 JSON は v6 候補。

---

## Phase 5H: 公開準備

**目的**: crates.io への初回公開（`cargo publish --dry-run`）を通過させる。

### examples 整理

既存の `rinq_basic_usage.rs` および `rinq_derive_example.rs` を新ディレクトリ構造に移動し、以下を追加:

```
rinq/examples/
  basic_usage.rs
  window_analytics.rs
  functional_ops.rs
  join_example.rs
  parallel_example.rs
  metrics_example.rs
rinq-stats/examples/
  statistics.rs
  timeseries.rs
  validation.rs
rinq-derive/examples/
  derive_example.rs   ← rinq_derive_example.rs を移動・リネーム
rinq-syntax/examples/
  syntax_example.rs
```

### 最終チェックリスト

```bash
cargo test --workspace
cargo test --workspace --all-features
cargo test --doc
cargo doc --no-deps --all-features --workspace
cargo clippy --workspace --all-features -- -D warnings
cargo bench --no-run --workspace
cargo publish --dry-run -p rinq
cargo publish --dry-run -p rinq-stats
cargo publish --dry-run -p rinq-derive
cargo publish --dry-run -p rinq-syntax
```

---

## リスク・注意事項

### git mv による履歴保持

`cp -r` ではなく `git mv` を使うこと。ただし Windows の Git Bash では `git mv src rinq/src` が失敗する場合がある（`rinq/` が先に存在しないとエラー）。事前に `mkdir -p rinq` を実行しておく。

### `rinq-syntax` テストの `path` 参照

`rinq-syntax/tests/syntax_tests.rs` 内に `rinq = { path = ".." }` の dev-dependency がある。移動後は `path = "../rinq"` に変更する。

### Cargo.lock の再生成

ディレクトリ移動後に `cargo build` を実行すると `Cargo.lock` が自動更新される。差分が大きくなるが正常動作。

### JOIN の eager 実装

`inner_join` / `left_join` は右辺全体を `HashMap` に収集する。大量データに対して使う場合はドキュメントに明記する（`⚠ Right side is eagerly collected`）。

### `flatten()` の T: IntoIterator 制約

`flatten()` は `T` が `IntoIterator` を実装している場合のみ使える。`T = Vec<U>` が典型的なユースケース。メソッド名が `std::iter::Flatten` と衝突しないか確認する。

### `tee()` のメモリ

`tee()` は全要素を Clone して 2 倍のメモリを消費する。ドキュメントに `⚠ Clones all elements` を明記。

---

## Phase 5I: CI/CD・OSS 整備

**目的**: GitHub Actions による自動化と OSS としての受け入れ体制を整える。5H（公開準備）完了後に実施する。

### ファイル構成（追加）

```
rusted-ca/
  LICENSE                           ← MIT ライセンステキスト
  CONTRIBUTING.md                   ← コントリビューションガイド
  .github/
    workflows/
      ci.yml                        ← GitHub Actions CI
    ISSUE_TEMPLATE/
      bug_report.md                 ← バグ報告テンプレート
      feature_request.md            ← 機能要求テンプレート
    PULL_REQUEST_TEMPLATE.md        ← PR テンプレート
  rinq/
    README.md                       ← rinq クレート専用 README（新規）
```

### 5I-1: GitHub Actions CI (`ci.yml`)

3 つのジョブを並列実行する:

```yaml
name: CI

on:
  push:
    branches: [main, dev]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --all-features

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-features -- -D warnings

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check
```

**採用ライブラリ:**
- `dtolnay/rust-toolchain@stable` — Rust 公式ツールチェーン（メンテが活発）
- `Swatinem/rust-cache@v2` — ビルドキャッシュ（CI 時間を大幅短縮）

### 5I-2: LICENSE

MIT ライセンス（2026 Copyright kazuma0606）。

### 5I-3: CONTRIBUTING.md

コントリビューターへの要求レベルを **貢献規模** で分ける:

| 規模 | 例 | 要求事項 |
|---|---|---|
| 小（typo・doc 修正） | README/コメントの誤字修正 | PR のみ。issue 不要 |
| 中（バグ修正・テスト追加） | 既存演算子のバグ、テスト補完 | PR + 再現手順の記載 |
| 大（新演算子・新クレート） | 新しい演算子・機能追加 | Issue で設計議論 → 承認後に実装 |

**AI ツールの使用について明確に歓迎する:**
- このプロジェクト自体が AI 支援開発で構築されているため、AI ツールの使用を積極的に認める
- AI 生成コードは `# AI-assisted` をコミットメッセージに含めることを推奨（任意）
- AI を使っても `cargo test` / `cargo clippy` の通過は必須

### 5I-4: rinq/README.md（クレート専用）

ルートの README とは別に rinq クレートに特化した内容を作成:

```
## rinq

Type-safe, zero-cost LINQ-inspired query engine for Rust.

バッジ（CI / crates.io version / docs.rs / license）

### Quick Start（コードブロック）
### Feature Flags
### State Machine（表）
### Operator Reference（全演算子の表）
### Sub-crates（rinq-stats / rinq-derive / rinq-syntax へのリンク）
```

### 5I-5: Issue/PR テンプレート

**bug_report.md**: 再現手順・期待動作・実際の動作・環境（Rust バージョン）
**feature_request.md**: 提案する演算子のシグネチャ・ユースケース・既存演算子との比較
**PULL_REQUEST_TEMPLATE.md**: チェックリスト（`cargo test` 通過・`cargo clippy` 通過・doc test 追加・CHANGELOG 記載）

### Cargo.toml の repository URL 更新

```toml
# 全クレートの Cargo.toml
repository = "https://github.com/kazuma0606/rinq"
```
