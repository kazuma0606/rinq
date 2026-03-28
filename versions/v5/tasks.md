# RINQ v5.0 タスク管理

チェックボックスの凡例: `[ ]` 未着手 / `[~]` 進行中 / `[x]` 完了

---

## Phase 5A: 整理・品質向上

### 5A-1: ディレクトリ再構成

- [x] `rinq/` ディレクトリを作成（`mkdir -p rinq`）
- [x] `git mv src rinq/src` でソースを移動
- [x] `git mv tests rinq/tests` でテストを移動
- [x] `git mv benches rinq/benches` でベンチマークを移動
- [x] `git mv examples rinq/examples` でサンプルを移動
- [x] ルート `Cargo.toml` を `[workspace]` のみに変更（`[package]` セクションを削除）
- [x] ルート `Cargo.toml` の `members` を `["rinq", "rinq-stats", "rinq-derive", "rinq-syntax"]` に更新
- [x] `rinq/Cargo.toml` を新規作成（version = "0.1.0"）
- [x] `rinq/Cargo.toml` の `[dev-dependencies]` の `rinq-derive` を `path = "../rinq-derive"` に更新
- [x] `cargo build` が通ることを確認

### 5A-2: サブクレート Cargo.toml 更新

- [x] `rinq-stats/Cargo.toml` — `version = "0.1.0"` に変更
- [x] `rinq-stats/Cargo.toml` — `rinq = { path = "../rinq" }` に更新
- [x] `rinq-stats/Cargo.toml` — `readme = "README.md"` に変更
- [x] `rinq-stats/Cargo.toml` — `repository = "https://github.com/kazuma0606/rinq"` に更新
- [x] `rinq-derive/Cargo.toml` — `version = "0.1.0"` に変更
- [x] `rinq-derive/Cargo.toml` — `[dev-dependencies]` の `rinq` を `path = "../rinq"` に更新
- [x] `rinq-derive/Cargo.toml` — `repository` を更新
- [x] `rinq-syntax/Cargo.toml` — `version = "0.1.0"` に変更
- [x] `rinq-syntax/Cargo.toml` — `[dev-dependencies]` の `rinq` を `path = "../rinq"` に更新
- [x] `rinq-syntax/Cargo.toml` — `repository` を更新

### 5A-3: README 整備

- [x] `rinq/README.md` を新規作成（クレート専用・バッジ・Quick Start・演算子一覧・Sub-crates）
- [x] `rinq-stats/README.md` を新規作成（StatisticsExt・SamplingExt・ValidationExt・TimeSeriesExt・OutlierExt クイックリファレンス）
- [x] `rinq-derive/README.md` を新規作成（derive(Queryable)・derive(QueryableFrom) クイックスタート・属性一覧）
- [x] `rinq-syntax/README.md` を新規作成（query! 構文リファレンス・binding semantics・Experimental 注記）

### 5A-4: CLAUDE.md 更新

- [x] `Commands` セクションを新ディレクトリ構造に合わせて更新（`cargo test --workspace` 等）
- [x] `Module Structure` 図を `rinq/src/` ベースに更新（v3/v4 追加モジュール含む）
- [x] 演算子テーブルに v3/v4 追加分を反映
- [x] テストファイル一覧を更新（rinq_v4_tests / timeseries_tests / outlier_tests 等）
- [x] `versions/` ディレクトリ説明を v1〜v5 に拡充
- [x] 4 クレートの構造を網羅した全面刷新

### Phase 5A テスト確認

- [x] `cargo build --workspace` 通過
- [x] `cargo test --workspace` 全件通過
- [x] `cargo clippy --workspace --all-features -- -D warnings` ゼロ

### ✅ Phase 5A 完了チェック
- [x] 上記すべて完了

---

## Phase 5B: テスト補完

### 5B-1: 未テスト演算子の統合テスト

`rinq/tests/rinq_v5_tests.rs` を新規作成（44 テスト）:

- [x] `tap_each` — 副作用カウンタ確認（AtomicUsize）
- [x] `tap_each` — 空コレクションで副作用が発生しないことを確認
- [x] `tap_each` — `where_` 後のチェーン中の位置確認
- [x] `tap_each` — 要素を変更しないことを確認
- [x] `tap_each` — 通過した値を記録できることを確認
- [x] `tap_collect` — 全収集後の副作用確認
- [x] `tap_collect` — 空コレクションでもクロージャが呼ばれる
- [x] `tap_collect` — 要素を変更しないことを確認
- [x] `tap_collect` — スライス長が `where_` 後の件数と一致
- [x] `pipe` — 外部 `fn` への委譲
- [x] `pipe` — 2 つの関数を連鎖
- [x] `pipe` — identity（同一要素を返す）
- [x] `pipe` — クロージャでの使用
- [x] `map` — `select` と等価な結果を確認
- [x] `map` — 型変換の確認
- [x] `collect_vec` — `collect::<Vec<_>>()` と等価な結果を確認
- [x] `collect_vec` — 空コレクション

### 5B-2: 組み合わせテスト

- [x] `MetricsQueryBuilder::new` + `count` ターミナル
- [x] `MetricsQueryBuilder::new` + `collect` ターミナル
- [x] `MetricsQueryBuilder` — 複数回実行でカウント累積
- [x] `rinq-derive` の `#[derive(Queryable)]` + `pairwise` / `scan` / `zip_with`
- [x] `rinq-derive` の field accessor + `order_by`
- [x] `rinq-derive` の predicate + `tap_each`
- [x] `rinq-syntax` の `query!` + `rinq-derive` の `#[derive(Queryable)]`（filter/select）
- [x] `rinq-syntax` の `query!` + `order_by` + derive struct
- [x] `rinq-syntax` の `query!` + `take desc` + derive struct
- [x] 100,000 件 `Vec<i32>` での `where_` + `order_by` + `count` の正常動作
- [x] 10,000 件 `Vec<i32>` での `group_by` の正常動作
- [x] `#[cfg(feature = "parallel")]` — `ParallelQueryBuilder` の filter + sum
- [x] `#[cfg(feature = "parallel")]` — parallel と sequential の count が一致
- [x] `#[cfg(feature = "serde")]` — `from_json` + filter + collect
- [x] `#[cfg(feature = "serde")]` — empty JSON array

### 5B-3: エッジケース補強

- [x] `pairwise()` — 0 要素 → 空 Vec
- [x] `pairwise()` — 1 要素 → 空 Vec
- [x] `pairwise()` — 2 要素 → 1 ペア
- [x] `pairwise()` — 3 要素 → 2 ペア
- [x] `intersperse()` — 空コレクション → 空 Vec
- [x] `intersperse()` — 1 要素 → そのまま 1 要素
- [x] `intersperse()` — 2 要素 → セパレータ 1 個
- [x] `dedup_by()` — 全同値 → 1 要素のみ返る
- [x] `dedup_by()` — 全異値 → 全要素返る
- [x] `dedup_by()` — タプルキー（first element でグルーピング）
- [x] `dedup_by()` — 文字列先頭文字キー
- [x] `unfold()` + `take(5)` — 無限生成の早期終了確認
- [x] `unfold()` + `take(0)` — 空
- [x] `unfold()` + `take` + `where_` パイプライン

### Phase 5B テスト確認

- [x] `cargo test --workspace` 全件通過
- [x] `cargo clippy --workspace --all-features -- -D warnings` ゼロ

### ✅ Phase 5B 完了チェック
- [x] 上記すべて完了

---

## Phase 5C: ベンチマーク拡充

### 5C-1: rinq_v4_benchmarks.rs 作成

`rinq/benches/rinq_v4_benchmarks.rs` を新規作成:

**functional グループ**
- [x] `scan_cumulative_sum` — 1000/10000 要素
- [x] `chunk_by_same_key` — 1000/10000 要素
- [x] `dedup_consecutive` — 1000/10000 要素
- [x] `zip_with_add` — 1000/10000 要素
- [x] `pairwise` — 1000/10000 要素
- [x] `intersperse` — 1000/10000 要素
- [x] `min_max` — 1000/10000 要素
- [x] `filter_map_parse` — 1000/10000 要素
- [x] `step_by_2` — 1000/10000 要素

**window グループ**
- [x] `running_sum` — 1000/10000 要素
- [x] `moving_average_10` — 1000/10000 要素
- [x] `rank_by` — 1000/10000 要素
- [x] `lag_1` — 1000/10000 要素
- [x] `lead_1` — 1000/10000 要素

**lifecycle グループ**
- [x] `tap_each_noop` — ゼロコスト確認
- [x] `tap_collect_vec` — Eager コスト確認
- [x] `pipe_identity` — ゼロコスト確認
- [x] `from_arc_cloned` — Arc クローンオーバーヘッド確認

**generation グループ**
- [x] `unfold_fib_take100` — unfold_bounded vs 手書きフィボナッチ
- [x] `cycle_take1000` — cycle + take

### 5C-2: rinq_stats_benchmarks.rs 作成

`rinq-stats/benches/rinq_stats_benchmarks.rs` を新規作成:

- [x] `rinq-stats/Cargo.toml` に `[dev-dependencies] criterion = "0.5"` を追加
- [x] `rinq-stats/Cargo.toml` に `[[bench]] name = "rinq_stats_benchmarks" harness = false` を追加

**statistics グループ**
- [x] `variance_1000` / `variance_10000`
- [x] `median_1000` / `median_10000`
- [x] `percentile_95_1000` / `percentile_95_10000`
- [x] `mode_i32` / `skewness`（histogram は API 確認後追加予定）

**timeseries グループ**
- [x] `ema_alpha01_10000`
- [x] `bollinger_window20_10000`

**outliers グループ**
- [x] `zscore_threshold2_10000`
- [x] `iqr_10000`

**sampling グループ**
- [x] `sample_n_100_from_10000`
- [x] `sample_fraction_10pct_10000`

**validation グループ**
- [x] `validation_single_rule_10000`
- [x] `validation_multi_rule_10000`

### Phase 5C テスト確認

- [x] `cargo bench --no-run --workspace` 全件通過（コンパイルエラーなし）
- [x] `cargo clippy --workspace -- -D warnings` ゼロ

### ✅ Phase 5C 完了チェック
- [x] 上記すべて完了

---

## Phase 5D: ターミナル演算子強化

### 実装対象

`rinq/src/core/builder/shared.rs` に以下を追加:

- [x] `for_each(f: FnMut(T))` — 消費型ターミナル
- [x] `for_each` の doc test（ループ副作用の例）
- [x] `to_sorted_vec(key)` — `order_by + collect` のショートハンド
- [x] `to_sorted_vec_desc(key)` — `order_by_descending + collect`
- [x] `to_sorted_vec` の doc test
- [x] `take_last(n: usize) -> Vec<T>` — 末尾 n 件（Eager 収集）
- [x] `skip_last(n: usize) -> Vec<T>` — 末尾 n 件を除く（Eager 収集）
- [x] `take_last` / `skip_last` のドキュメントに `⚠ Eagerly collects all elements` を明記
- [x] `take_last` / `skip_last` の doc test
- [x] `count_by(pred: Fn(&T) -> bool) -> usize`
- [x] `count_by` の doc test
- [x] `sum_by<N, F>(key: F) -> N where F: Fn(T) -> N, N: Default + Add<Output = N>`
- [x] `average_by<F>(key: F) -> Option<f64> where F: Fn(T) -> f64`
- [x] `sum_by` / `average_by` の doc test
- [x] `reduce<F>(f: F) -> Option<T> where F: FnMut(T, T) -> T` — `aggregate_no_seed` の alias
- [x] `reduce` の doc test
- [x] `all_unique() -> bool where T: Hash + Eq`
- [x] `all_unique` の doc test
- [x] `none<F>(pred: F) -> bool where F: Fn(&T) -> bool`
- [x] `none` の doc test

### テスト追加（tests/rinq_v5_tests.rs）

- [x] `for_each` — 副作用が全要素に適用されることを確認
- [x] `to_sorted_vec` — `order_by + collect` と同一結果を確認
- [x] `take_last(3)` — 末尾 3 件の確認
- [x] `skip_last(2)` — 末尾 2 件を除いた確認
- [x] `take_last(0)` — 空 Vec
- [x] `count_by` — 条件一致件数の確認
- [x] `sum_by` / `average_by` — 集計値の確認
- [x] `reduce` — `aggregate_no_seed` と等価な結果を確認
- [x] `all_unique` — 重複あり/なしの確認
- [x] `none` — `any` の否定と等価な結果を確認

### Phase 5D テスト確認

- [x] `cargo test --workspace` 全件通過
- [x] `cargo clippy --workspace --all-features -- -D warnings` ゼロ

### ✅ Phase 5D 完了チェック
- [x] 上記すべて完了

---

## Phase 5E: クエリ充実

### 実装対象

`rinq/src/core/builder/shared.rs` に以下を追加:

- [x] `frequencies() -> HashMap<T, usize> where T: Hash + Eq`
- [x] `frequencies` の doc test（出現回数カウントの例）
- [x] `flatten<U>(self) -> QueryBuilder<U, Filtered> where T: IntoIterator<Item = U>`
- [x] `flatten` の doc test（ネスト Vec のフラット化の例）
- [x] `position<F>(pred: F) -> Option<usize>`
- [x] `position` の doc test
- [x] `find<F>(pred: F) -> Option<T>` — 述語付き先頭要素
- [x] `find` の doc test
- [x] `index_of(value: &T) -> Option<usize> where T: PartialEq`
- [x] `index_of` の doc test
- [x] `nth(n: usize)` — `element_at(n)` の alias
- [x] `nth` の doc test
- [x] `batch(n: usize)` — `chunk(n)` の alias（名称差の doc コメント明記）
- [x] `batch` の doc test
- [x] `exactly_one()` — `single()` の alias
- [x] `exactly_one` の doc test
- [x] `tee() -> (Vec<T>, Vec<T>) where T: Clone`
- [x] `tee` のドキュメントに `⚠ Clones all elements` を明記
- [x] `tee` の doc test

### テスト追加

- [x] `frequencies` — 出現回数マップの確認
- [x] `frequencies` — 空コレクション → 空 HashMap
- [x] `flatten` — ネスト Vec のフラット化確認
- [x] `position` — マッチあり/なしの確認
- [x] `find` — `where_.first()` と等価な結果を確認
- [x] `index_of` — マッチあり/なしの確認
- [x] `tee` — 2 つの Vec が同一内容であることを確認
- [x] `tee` — 独立した Vec（一方を変更しても他方に影響なし）

### Phase 5E テスト確認

- [x] `cargo test --workspace` 全件通過
- [x] `cargo clippy --workspace --all-features -- -D warnings` ゼロ

### ✅ Phase 5E 完了チェック
- [x] 上記すべて完了

---

## Phase 5F: JOIN 操作

### rinq 本体 — join.rs 作成

- [x] `rinq/src/core/builder/join.rs` を新規作成
- [x] `mod.rs` に `mod join;` を追加
- [x] `inner_join` を実装（右辺 HashMap 戦略、O(N+M)）
- [x] `inner_join` のドキュメントに `⚠ Right side is eagerly collected` を明記
- [x] `inner_join` の doc test
- [x] `left_join` を実装
- [x] `left_join` の doc test
- [x] `cross_join` を実装（右辺 Vec 収集後に直積）
- [x] `cross_join` のドキュメントに `⚠ O(N×M) — use with caution on large collections` を明記
- [x] `cross_join` の doc test

### rinq-syntax — join 節対応

- [x] `rinq-syntax/src/ast.rs` に `Clause::Join { binding, source_tokens, left_key, right_key, is_left }` を追加
- [x] `rinq-syntax/src/parser.rs` に `join` / `on` カスタムキーワードを追加
- [x] `join x in source on left.key == right.key` のパースを実装
- [x] `left join` キーワードのサポート（`peek_left_join` ヘルパー）
- [x] `rinq-syntax/src/codegen.rs` に `Clause::Join` の展開ロジックを追加（closure_pat でタプルパターンを追跡）
- [x] JOIN 後に `where` / `select` が続くケースのコード生成を確認

### テスト追加

- [x] `inner_join` — 基本マッチング（全ペアが返る）
- [x] `inner_join` — 一部のみマッチ（非マッチは除外）
- [x] `inner_join` — 右辺が空 → 空 Vec
- [x] `left_join` — 全マッチ（Option が Some）
- [x] `left_join` — 一部不マッチ（None が含まれる）
- [x] `cross_join` — 直積の確認（2×3 = 6 要素）
- [x] `cross_join` — 一方が空 → 空 Vec
- [x] `query!` + `join` 節 — `inner_join` への展開確認
- [x] `query!` + `left join` 節 — `left_join` への展開確認
- [x] `query!` + `join` + `where` — JOIN 後のフィルタ確認

### Phase 5F テスト確認

- [x] `cargo test --workspace` 全件通過（107テスト）
- [x] `cargo clippy --workspace --all-features -- -D warnings` ゼロ

### ✅ Phase 5F 完了チェック
- [x] 上記すべて完了

---

## Phase 5G: rinq-stats 統計拡張

### 5G-1: transform.rs 新規作成

- [x] `rinq-stats/src/transform.rs` を新規作成
- [x] `NormalizeExt` トレイトを定義
- [x] `normalize()` を実装（全同値の場合は全要素 0.0）
- [x] `standardize()` を実装（std_dev=0 の場合は全要素 0.0）
- [x] `weighted_average(weight_fn)` を実装
- [x] `outlier_scores_zscore()` を実装（除去ではなくスコア Vec を返す）
- [x] `percentile_filter(lo_pct, hi_pct)` を実装
- [x] `cumulative_distribution()` を実装
- [x] `rinq-stats/src/lib.rs` に re-export を追加
- [x] `rinq-stats/tests/transform_tests.rs` を新規作成（各メソッドのテスト）

### 5G-2: timeseries.rs 拡張

- [x] `simple_moving_average(window)` を追加
- [x] `weighted_moving_average(window)` を追加（線形加重）
- [x] `rate_of_change(period)` を追加
- [x] `seasonal_decompose(period)` を追加（trend + seasonal + residual の 3 Vec を返す構造体）
- [x] 各メソッドのテストを `rinq-stats/tests/timeseries_tests.rs` に追記

### 5G-3: outliers.rs 拡張

- [x] `remove_outliers_modified_zscore(threshold)` を追加（MAD ベース）
- [x] `outlier_scores_iqr()` を追加（スコア返却）
- [x] 各メソッドのテストを `rinq-stats/tests/outlier_tests.rs` に追記

### 5G-4: validation.rs 拡張

- [x] `validate_range(field_fn, min, max, rule_name)` を追加
- [x] `validate_unique(key_fn, rule_name)` を追加（RefCell<HashSet> で重複チェック）
- [x] `validate_non_empty(rule_name)` を追加
- [x] `report() -> Vec<String>` を追加（ValidationError を文字列リストとして返す）
- [x] 各メソッドのテストを `rinq-stats/tests/validation_tests.rs` に追記

### Phase 5G テスト確認

- [x] `cargo test -p rinq-stats` 全件通過
- [x] `cargo clippy -p rinq-stats -- -D warnings` ゼロ

### ✅ Phase 5G 完了チェック
- [x] 上記すべて完了

---

## Phase 5H: 公開準備

### 5H-1: examples 整理・拡充

- [x] `rinq/examples/basic_usage.rs` を確認（既存 `rinq_basic_usage.rs` をリネーム）
- [x] `rinq/examples/window_analytics.rs` を新規作成
- [x] `rinq/examples/functional_ops.rs` を新規作成
- [x] `rinq/examples/join_example.rs` を新規作成
- [x] `rinq/examples/parallel_example.rs` を新規作成
- [x] `rinq/examples/metrics_example.rs` を新規作成
- [x] `rinq-stats/examples/statistics.rs` を新規作成
- [x] `rinq-stats/examples/timeseries.rs` を新規作成
- [x] `rinq-stats/examples/validation.rs` を新規作成
- [x] `rinq-derive/examples/derive_example.rs` を新規作成（`rinq_derive_example.rs` を移動・リネーム）
- [x] `rinq-syntax/examples/syntax_example.rs` を新規作成
- [x] 全 Cargo.toml に `[[example]]` エントリを追加

### 5H-2: ドキュメント最終確認

- [x] `cargo doc --no-deps --all-features --workspace` — 警告ゼロを確認
- [x] `cargo test --doc` — 全 doc test 通過を確認

### 5H-3: 最終品質チェック

- [x] `cargo test --workspace` 全件通過
- [x] `cargo test --workspace --all-features` 全件通過
- [x] `cargo clippy --workspace --all-features -- -D warnings` ゼロ
- [x] `cargo bench --no-run --workspace` 全件通過

### 5H-4: crates.io dry-run

- [x] `cargo publish --dry-run -p rinq` — エラーゼロを確認
- [x] `cargo publish --dry-run -p rinq-stats` — rinq 未公開のため rinq-stats の dry-run は rinq 公開後に実施
- [x] `cargo publish --dry-run -p rinq-derive` — エラーゼロを確認
- [x] `cargo publish --dry-run -p rinq-syntax` — エラーゼロを確認

### 5H-5: CHANGELOG.md 更新

- [x] `CHANGELOG.md` に `## [v0.1.0] - 2026-03-28` エントリを追加
- [x] Phase 5A〜5H の追加内容を列挙

### ✅ Phase 5H 完了チェック / RINQ v0.1.0 リリース
- [x] 上記すべて完了

---

## Phase 5I: CI/CD・OSS 整備

### 5I-1: GitHub Actions CI

- [ ] `.github/workflows/` ディレクトリを作成
- [ ] `.github/workflows/ci.yml` を新規作成（test / clippy / fmt の 3 ジョブ）
- [ ] `test` ジョブ — `cargo test --workspace --all-features`
- [ ] `clippy` ジョブ — `cargo clippy --workspace --all-features -- -D warnings`
- [ ] `fmt` ジョブ — `cargo fmt --all --check`
- [ ] `Swatinem/rust-cache@v2` によるキャッシュを全ジョブに設定
- [ ] `push: branches: [main, dev]` / `pull_request: branches: [main]` トリガーを設定
- [ ] GitHub 上で CI が実際に通ることを確認

### 5I-2: LICENSE

- [ ] `LICENSE` ファイルを新規作成（MIT、Copyright 2026 kazuma0606）

### 5I-3: CONTRIBUTING.md

- [ ] `CONTRIBUTING.md` を新規作成
- [ ] 貢献規模別の要求事項（小/中/大）を記載
- [ ] **AI ツールの使用を明示的に歓迎する旨を記載**（このプロジェクト自体が AI 支援開発であることを明記）
- [ ] AI 使用時の任意表記（コミットメッセージへの `# AI-assisted` 追記）を提案として記載
- [ ] PR 前の必須チェック（`cargo test` / `cargo clippy` 通過）を明記
- [ ] 大きな変更は Issue での事前議論を推奨する旨を記載
- [ ] `versions/` ディレクトリについての説明（内部 AI コーディング用、コントリビューターは不要）

### 5I-4: rinq/README.md（クレート専用）

- [ ] `rinq/README.md` を新規作成（ルートの README とは別）
- [ ] CI バッジを追加（`https://github.com/kazuma0606/rinq/actions/workflows/ci.yml`）
- [ ] crates.io バッジを追加（`https://crates.io/crates/rinq`）
- [ ] docs.rs バッジを追加（`https://docs.rs/rinq`）
- [ ] license バッジを追加
- [ ] Quick Start コードブロック（6〜10 行で完結する例）
- [ ] Feature Flags テーブル（parallel / serde）
- [ ] State Machine テーブル
- [ ] 全演算子リファレンス（カテゴリ別テーブル）
- [ ] Sub-crates セクション（rinq-stats / rinq-derive / rinq-syntax へのリンクと一行説明）

### 5I-5: Issue/PR テンプレート

- [ ] `.github/ISSUE_TEMPLATE/bug_report.md` を新規作成（再現手順・期待動作・Rust バージョン）
- [ ] `.github/ISSUE_TEMPLATE/feature_request.md` を新規作成（シグネチャ案・ユースケース・既存演算子との比較）
- [ ] `.github/PULL_REQUEST_TEMPLATE.md` を新規作成（`cargo test` / `cargo clippy` / doc test / CHANGELOG チェックリスト）

### 5I-6: repository URL を全 Cargo.toml に反映

- [ ] `rinq/Cargo.toml` — `repository = "https://github.com/kazuma0606/rinq"` に更新
- [ ] `rinq-stats/Cargo.toml` — 同上
- [ ] `rinq-derive/Cargo.toml` — 同上
- [ ] `rinq-syntax/Cargo.toml` — 同上

### Phase 5I テスト確認

- [ ] `cargo test --workspace` 全件通過（ローカルで最終確認）
- [ ] GitHub Actions CI が green になることを確認（push して確認）

### ✅ Phase 5I 完了チェック / v5 全フェーズ完了
- [ ] 上記すべて完了

---

## 付録: v5 で明確化した設計制約

### JOIN の状態遷移

`inner_join` / `left_join` / `cross_join` はすべて `QueryBuilder<(T, U), Filtered>` を返す。
JOIN 後のチェーンは `where_` / `select` / `order_by` へ進める。

### ターミナル演算子と Eager 化

`take_last` / `skip_last` / `tee` / JOIN 演算子（右辺収集）は内部で Eager 収集を行う。
ドキュメントに `⚠` 記法で明記し、パイプライン中の位置に注意を促す。

### Alias の方針

`reduce` / `find` / `nth` / `batch` / `exactly_one` はそれぞれ `aggregate_no_seed` / `first` / `element_at` / `chunk` / `single` の alias として実装する。
alias 側の doc コメントには `/// Alias for [`original`].` を明記する。
