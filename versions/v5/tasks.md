# RINQ v5.0 タスク管理

チェックボックスの凡例: `[ ]` 未着手 / `[~]` 進行中 / `[x]` 完了

---

## Phase 5A: 整理・品質向上

### 5A-1: ディレクトリ再構成

- [ ] `rinq/` ディレクトリを作成（`mkdir -p rinq`）
- [ ] `git mv src rinq/src` でソースを移動
- [ ] `git mv tests rinq/tests` でテストを移動
- [ ] `git mv benches rinq/benches` でベンチマークを移動
- [ ] `git mv examples rinq/examples` でサンプルを移動
- [ ] `git mv README.md rinq/README.md` で README を移動
- [ ] ルート `Cargo.toml` を `[workspace]` のみに変更（`[package]` セクションを削除）
- [ ] ルート `Cargo.toml` の `members` を `["rinq", "rinq-stats", "rinq-derive", "rinq-syntax"]` に更新
- [ ] `rinq/Cargo.toml` を新規作成（既存の package セクションをベースに `version = "0.1.0"`）
- [ ] `rinq/Cargo.toml` の `[dev-dependencies]` の `rinq-derive` を `path = "../rinq-derive"` に更新
- [ ] `rinq/Cargo.toml` に `[[bench]] name = "rinq_v4_benchmarks"` エントリを追加
- [ ] `cargo build` が通ることを確認

### 5A-2: サブクレート Cargo.toml 更新

- [ ] `rinq-stats/Cargo.toml` — `version = "0.1.0"` に変更
- [ ] `rinq-stats/Cargo.toml` — `rinq = { path = "../rinq" }` に更新
- [ ] `rinq-stats/Cargo.toml` — `readme = "README.md"` に変更
- [ ] `rinq-derive/Cargo.toml` — `version = "0.1.0"` に変更
- [ ] `rinq-derive/Cargo.toml` — `[dev-dependencies]` の `rinq` を `path = "../rinq"` に更新
- [ ] `rinq-syntax/Cargo.toml` — `version = "0.1.0"` に変更
- [ ] `rinq-syntax/Cargo.toml` — `[dev-dependencies]` の `rinq` を `path = "../rinq"` に更新

### 5A-3: README 整備

- [ ] `rinq-stats/README.md` を新規作成（StatisticsExt・SamplingExt・ValidationExt・TimeSeriesExt・OutlierExt クイックリファレンス）
- [ ] `rinq-derive/README.md` を新規作成（derive(Queryable)・derive(QueryableFrom) クイックスタート・属性一覧）
- [ ] `rinq-syntax/README.md` を新規作成（query! 構文リファレンス・binding semantics・Experimental 注記）
- [ ] `rinq/README.md` のインストール例を `rinq = "0.1"` / `rinq-stats = "0.1"` / `rinq-derive = "0.1"` / `rinq-syntax = "0.1"` に更新

### 5A-4: CLAUDE.md 更新

- [ ] `Commands` セクションを新ディレクトリ構造に合わせて更新（`cargo test --workspace` 等）
- [ ] `Module Structure` 図を `rinq/src/` ベースに更新
- [ ] 演算子テーブルに v3/v4 追加分を反映（scan/chunk_by/dedup/zip_with/pairwise/intersperse/min_max/unfold/tap_each/tap_collect/pipe/filter_map/step_by/cycle 等）
- [ ] テストファイル一覧を更新（rinq_v4_tests / rinq_v5_tests / timeseries_tests / outlier_tests 等）
- [ ] `versions/` ディレクトリ説明を v3〜v5 に拡充

### Phase 5A テスト確認

- [ ] `cargo build --workspace` 通過
- [ ] `cargo test --workspace` 全件通過
- [ ] `cargo clippy --workspace -- -D warnings` ゼロ

### ✅ Phase 5A 完了チェック
- [ ] 上記すべて完了

---

## Phase 5B: テスト補完

### 5B-1: 未テスト演算子の統合テスト

`rinq/tests/rinq_v5_tests.rs` を新規作成:

- [ ] `tap_each` — 副作用カウンタ確認（AtomicUsize）
- [ ] `tap_each` — 空コレクションで副作用が発生しないことを確認
- [ ] `tap_each` — チェーン中の位置（`where_` の前後どちらでも動作）
- [ ] `tap_collect` — 全収集後の副作用確認
- [ ] `tap_collect` — Eager 化の検証（チェーン後の要素数が変わらない）
- [ ] `tap_collect` — 空コレクション
- [ ] `pipe` — 外部 `fn` への委譲（`fn add_filter(q: FilteredQuery<i32>) -> FilteredQuery<i32>`）
- [ ] `pipe` — 戻り値型が変わる変換
- [ ] `cycle` + `take(n)` — 正常ループ確認（`[1,2,3].cycle().take(7)` → `[1,2,3,1,2,3,1]`）
- [ ] `cycle` — 空コレクションからの cycle（`take` しても空）
- [ ] `step_by(1)` — 全要素が返る
- [ ] `step_by(2)` — 偶数インデックスのみ返る
- [ ] `step_by(3)` — 3 件ごと
- [ ] `step_by(0)` — panic が発生することを `#[should_panic]` で確認
- [ ] `map` — `select` と等価な結果を確認
- [ ] `collect_vec` — `collect::<Vec<_>>()` と等価な結果を確認

### 5B-2: 組み合わせテスト

- [ ] `#[cfg(all(feature = "parallel", feature = "serde"))]` — `ParallelQueryBuilder` + `from_json` の組み合わせ
- [ ] `MetricsQueryBuilder` + `parallel` feature の組み合わせ
- [ ] `rinq-derive` の `#[derive(Queryable)]` + `pairwise` / `scan` / `zip_with`
- [ ] `rinq-syntax` の `query!` + `rinq-derive` の `#[derive(Queryable)]`（`User` 構造体を使った統合テスト）
- [ ] 100 万件 `Vec<i32>` での `where_(|&x| x % 2 == 0).order_by(|x| *x).count()` の正常動作

### 5B-3: エッジケース補強

- [ ] `pairwise()` — 0 要素 → 空 Vec
- [ ] `pairwise()` — 1 要素 → 空 Vec
- [ ] `pairwise()` — 2 要素 → 1 ペア
- [ ] `intersperse()` — 空コレクション → 空 Vec
- [ ] `intersperse()` — 1 要素 → そのまま 1 要素
- [ ] `dedup_by()` — 全同値 → 1 要素のみ返る
- [ ] `dedup_by()` — 全異値 → 全要素返る
- [ ] `dedup_by()` — 複合キー（タプル）
- [ ] `unfold()` + `take(10)` — 無限生成の早期終了確認
- [ ] `lag(0)` — 全要素が `(x, x)` ペアになる
- [ ] `lead(0)` — 全要素が `(x, x)` ペアになる

### Phase 5B テスト確認

- [ ] `cargo test --workspace` 全件通過
- [ ] `cargo test --workspace --all-features` 全件通過
- [ ] `cargo clippy --workspace --all-features -- -D warnings` ゼロ

### ✅ Phase 5B 完了チェック
- [ ] 上記すべて完了

---

## Phase 5C: ベンチマーク拡充

### 5C-1: rinq_v4_benchmarks.rs 作成

`rinq/benches/rinq_v4_benchmarks.rs` を新規作成:

**functional グループ**
- [ ] `scan_cumulative_sum` — 1000/10000 要素
- [ ] `chunk_by_same_key` — 1000/10000 要素
- [ ] `dedup_consecutive` — 1000/10000 要素
- [ ] `zip_with_add` — 1000/10000 要素
- [ ] `pairwise` — 1000/10000 要素
- [ ] `intersperse` — 1000/10000 要素
- [ ] `min_max` — 1000/10000 要素
- [ ] `filter_map_parse` — 1000/10000 要素
- [ ] `step_by_2` — 1000/10000 要素

**window グループ**
- [ ] `running_sum` — 1000/10000 要素
- [ ] `moving_average_10` — 1000/10000 要素
- [ ] `rank_by` — 1000/10000 要素
- [ ] `lag_1` — 1000/10000 要素
- [ ] `lead_1` — 1000/10000 要素

**lifecycle グループ**
- [ ] `tap_each_noop` — ゼロコスト確認
- [ ] `tap_collect_vec` — Eager コスト確認
- [ ] `pipe_identity` — ゼロコスト確認
- [ ] `from_arc_cloned` — Arc クローンオーバーヘッド確認

**generation グループ**
- [ ] `unfold_fib_take100` — unfold_bounded vs 手書きフィボナッチ
- [ ] `cycle_take1000` — cycle + take

### 5C-2: rinq_stats_benchmarks.rs 作成

`rinq-stats/benches/rinq_stats_benchmarks.rs` を新規作成:

- [ ] `rinq-stats/Cargo.toml` に `[dev-dependencies] criterion = "0.5"` を追加
- [ ] `rinq-stats/Cargo.toml` に `[[bench]] name = "rinq_stats_benchmarks" harness = false` を追加

**statistics グループ**
- [ ] `variance_1000` / `variance_10000`
- [ ] `median_1000` / `median_10000`
- [ ] `percentile_95_1000` / `percentile_95_10000`
- [ ] `histogram_10buckets_1000`

**timeseries グループ**
- [ ] `ema_alpha02_10000`
- [ ] `bollinger_window20_10000`

**outliers グループ**
- [ ] `zscore_threshold2_10000`
- [ ] `iqr_10000`

**sampling グループ**
- [ ] `sample_n_100_from_10000`
- [ ] `stratified_sample_5groups_10000`

### Phase 5C テスト確認

- [ ] `cargo bench --no-run --workspace` 全件通過（コンパイルエラーなし）
- [ ] `cargo clippy --workspace -- -D warnings` ゼロ

### ✅ Phase 5C 完了チェック
- [ ] 上記すべて完了

---

## Phase 5D: ターミナル演算子強化

### 実装対象

`rinq/src/core/builder/shared.rs` に以下を追加:

- [ ] `for_each(f: FnMut(T))` — 消費型ターミナル
- [ ] `for_each` の doc test（ループ副作用の例）
- [ ] `to_sorted_vec(key)` — `order_by + collect` のショートハンド
- [ ] `to_sorted_vec_desc(key)` — `order_by_descending + collect`
- [ ] `to_sorted_vec` の doc test
- [ ] `take_last(n: usize) -> Vec<T>` — 末尾 n 件（Eager 収集）
- [ ] `skip_last(n: usize) -> Vec<T>` — 末尾 n 件を除く（Eager 収集）
- [ ] `take_last` / `skip_last` のドキュメントに `⚠ Eagerly collects all elements` を明記
- [ ] `take_last` / `skip_last` の doc test
- [ ] `count_by(pred: Fn(&T) -> bool) -> usize`
- [ ] `count_by` の doc test
- [ ] `sum_by<N, F>(key: F) -> N where F: Fn(T) -> N, N: Default + Add<Output = N>`
- [ ] `average_by<F>(key: F) -> Option<f64> where F: Fn(T) -> f64`
- [ ] `sum_by` / `average_by` の doc test
- [ ] `reduce<F>(f: F) -> Option<T> where F: FnMut(T, T) -> T` — `aggregate_no_seed` の alias
- [ ] `reduce` の doc test
- [ ] `all_unique() -> bool where T: Hash + Eq`
- [ ] `all_unique` の doc test
- [ ] `none<F>(pred: F) -> bool where F: Fn(&T) -> bool`
- [ ] `none` の doc test

### テスト追加（tests/rinq_v5_tests.rs または新ファイル）

- [ ] `for_each` — 副作用が全要素に適用されることを確認
- [ ] `to_sorted_vec` — `order_by + collect` と同一結果を確認
- [ ] `take_last(3)` — 末尾 3 件の確認
- [ ] `skip_last(2)` — 末尾 2 件を除いた確認
- [ ] `take_last(0)` — 空 Vec
- [ ] `count_by` — 条件一致件数の確認
- [ ] `sum_by` / `average_by` — 集計値の確認
- [ ] `reduce` — `aggregate_no_seed` と等価な結果を確認
- [ ] `all_unique` — 重複あり/なしの確認
- [ ] `none` — `any` の否定と等価な結果を確認

### Phase 5D テスト確認

- [ ] `cargo test --workspace` 全件通過
- [ ] `cargo clippy --workspace -- -D warnings` ゼロ

### ✅ Phase 5D 完了チェック
- [ ] 上記すべて完了

---

## Phase 5E: クエリ充実

### 実装対象

`rinq/src/core/builder/shared.rs` に以下を追加:

- [ ] `frequencies() -> HashMap<T, usize> where T: Hash + Eq`
- [ ] `frequencies` の doc test（出現回数カウントの例）
- [ ] `flatten<U>(self) -> QueryBuilder<U, Filtered> where T: IntoIterator<Item = U>`
- [ ] `flatten` の doc test（ネスト Vec のフラット化の例）
- [ ] `position<F>(pred: F) -> Option<usize>`
- [ ] `position` の doc test
- [ ] `find<F>(pred: F) -> Option<T>` — `first(pred)` の alias
- [ ] `find` の doc test
- [ ] `index_of(value: &T) -> Option<usize> where T: PartialEq`
- [ ] `index_of` の doc test
- [ ] `nth(n: usize)` — `element_at(n)` の alias
- [ ] `nth` の doc test
- [ ] `batch(n: usize)` — `chunk(n)` の alias（名称差の doc コメント明記）
- [ ] `batch` の doc test
- [ ] `exactly_one()` — `single()` の alias
- [ ] `exactly_one` の doc test
- [ ] `tee() -> (Vec<T>, Vec<T>) where T: Clone`
- [ ] `tee` のドキュメントに `⚠ Clones all elements` を明記
- [ ] `tee` の doc test

### テスト追加

- [ ] `frequencies` — 出現回数マップの確認
- [ ] `frequencies` — 空コレクション → 空 HashMap
- [ ] `flatten` — ネスト Vec のフラット化確認
- [ ] `position` — マッチあり/なしの確認
- [ ] `find` — `first` と等価な結果を確認
- [ ] `index_of` — マッチあり/なしの確認
- [ ] `tee` — 2 つの Vec が同一内容であることを確認
- [ ] `tee` — 独立した Vec（一方を変更しても他方に影響なし）

### Phase 5E テスト確認

- [ ] `cargo test --workspace` 全件通過
- [ ] `cargo clippy --workspace -- -D warnings` ゼロ

### ✅ Phase 5E 完了チェック
- [ ] 上記すべて完了

---

## Phase 5F: JOIN 操作

### rinq 本体 — join.rs 作成

- [ ] `rinq/src/core/builder/join.rs` を新規作成
- [ ] `mod.rs` に `pub mod join;` を追加
- [ ] `inner_join` を実装（右辺 HashMap 戦略、O(N+M)）
- [ ] `inner_join` のドキュメントに `⚠ Right side is eagerly collected` を明記
- [ ] `inner_join` の doc test
- [ ] `left_join` を実装
- [ ] `left_join` の doc test
- [ ] `cross_join` を実装（右辺 Vec 収集後に直積）
- [ ] `cross_join` のドキュメントに `⚠ O(N×M) — use with caution on large collections` を明記
- [ ] `cross_join` の doc test
- [ ] `src/lib.rs` に JOIN 演算子の言及を追加

### rinq-syntax — join 節対応

- [ ] `rinq-syntax/src/ast.rs` に `Clause::Join { binding, source_tokens, left_key, right_key, is_left }` を追加
- [ ] `rinq-syntax/src/parser.rs` に `join` カスタムキーワードを追加
- [ ] `join x in source on left.key == right.key` のパースを実装
- [ ] `left join` キーワードのサポート
- [ ] `rinq-syntax/src/codegen.rs` に `Clause::Join` の展開ロジックを追加
- [ ] JOIN 後に `where` / `select` が続くケースのコード生成を確認

### テスト追加

- [ ] `inner_join` — 基本マッチング（全ペアが返る）
- [ ] `inner_join` — 一部のみマッチ（非マッチは除外）
- [ ] `inner_join` — 右辺が空 → 空 Vec
- [ ] `left_join` — 全マッチ（Option が Some）
- [ ] `left_join` — 一部不マッチ（None が含まれる）
- [ ] `cross_join` — 直積の確認（2×3 = 6 要素）
- [ ] `cross_join` — 一方が空 → 空 Vec
- [ ] `query!` + `join` 節 — `inner_join` への展開確認
- [ ] `query!` + `left join` 節 — `left_join` への展開確認

### Phase 5F テスト確認

- [ ] `cargo test --workspace` 全件通過
- [ ] `cargo test -p rinq-syntax` 全件通過
- [ ] `cargo clippy --workspace -- -D warnings` ゼロ

### ✅ Phase 5F 完了チェック
- [ ] 上記すべて完了

---

## Phase 5G: rinq-stats 統計拡張

### 5G-1: transform.rs 新規作成

- [ ] `rinq-stats/src/transform.rs` を新規作成
- [ ] `NormalizeExt` トレイトを定義
- [ ] `normalize()` を実装（全同値の場合は全要素 0.0）
- [ ] `standardize()` を実装（std_dev=0 の場合は全要素 0.0）
- [ ] `weighted_average(weight_fn)` を実装
- [ ] `outlier_scores_zscore()` を実装（除去ではなくスコア Vec を返す）
- [ ] `percentile_filter(lo_pct, hi_pct)` を実装
- [ ] `cumulative_distribution()` を実装
- [ ] `rinq-stats/src/lib.rs` に re-export を追加
- [ ] `rinq-stats/tests/transform_tests.rs` を新規作成（各メソッドのテスト）

### 5G-2: timeseries.rs 拡張

- [ ] `simple_moving_average(window)` を追加
- [ ] `weighted_moving_average(window)` を追加（線形加重）
- [ ] `rate_of_change(period)` を追加
- [ ] `seasonal_decompose(period)` を追加（trend + seasonal + residual の 3 Vec を返す構造体）
- [ ] 各メソッドのテストを `rinq-stats/tests/timeseries_tests.rs` に追記

### 5G-3: outliers.rs 拡張

- [ ] `remove_outliers_modified_zscore(threshold)` を追加（MAD ベース）
- [ ] `outlier_scores_iqr()` を追加（スコア返却）
- [ ] 各メソッドのテストを `rinq-stats/tests/outlier_tests.rs` に追記

### 5G-4: validation.rs 拡張

- [ ] `validate_range(field_fn, min, max, rule_name)` を追加
- [ ] `validate_unique(key_fn, rule_name)` を追加（HashMap で重複チェック）
- [ ] `validate_non_empty(rule_name)` を追加
- [ ] `report() -> Vec<String>` を追加（ValidationError を文字列リストとして返す）
- [ ] 各メソッドのテストを `rinq-stats/tests/validation_tests.rs` に追記

### Phase 5G テスト確認

- [ ] `cargo test -p rinq-stats` 全件通過
- [ ] `cargo clippy -p rinq-stats -- -D warnings` ゼロ

### ✅ Phase 5G 完了チェック
- [ ] 上記すべて完了

---

## Phase 5H: 公開準備

### 5H-1: examples 整理・拡充

- [ ] `rinq/examples/basic_usage.rs` を確認（既存 `rinq_basic_usage.rs` をリネーム）
- [ ] `rinq/examples/window_analytics.rs` を新規作成
- [ ] `rinq/examples/functional_ops.rs` を新規作成
- [ ] `rinq/examples/join_example.rs` を新規作成
- [ ] `rinq/examples/parallel_example.rs` を新規作成
- [ ] `rinq/examples/metrics_example.rs` を新規作成
- [ ] `rinq-stats/examples/statistics.rs` を新規作成
- [ ] `rinq-stats/examples/timeseries.rs` を新規作成
- [ ] `rinq-stats/examples/validation.rs` を新規作成
- [ ] `rinq-derive/examples/derive_example.rs` を新規作成（`rinq_derive_example.rs` を移動・リネーム）
- [ ] `rinq-syntax/examples/syntax_example.rs` を新規作成
- [ ] `rinq/Cargo.toml` に全 examples の `[[example]]` エントリを確認

### 5H-2: ドキュメント最終確認

- [ ] `cargo doc --no-deps --all-features --workspace` — 警告ゼロを確認
- [ ] `cargo test --doc` — 全 doc test 通過を確認
- [ ] 全 `pub fn` に `///` コメントがあることを `cargo doc` の `missing_docs` 警告で確認
- [ ] `rinq/src/lib.rs` のクレートトップ docs に v5 新演算子・JOIN・rinq-stats 拡張の言及を追加

### 5H-3: 最終品質チェック

- [ ] `cargo test --workspace` 全件通過
- [ ] `cargo test --workspace --all-features` 全件通過
- [ ] `cargo clippy --workspace --all-features -- -D warnings` ゼロ
- [ ] `cargo bench --no-run --workspace` 全件通過

### 5H-4: crates.io dry-run

- [ ] `cargo publish --dry-run -p rinq` — エラーゼロを確認
- [ ] `cargo publish --dry-run -p rinq-stats` — エラーゼロを確認
- [ ] `cargo publish --dry-run -p rinq-derive` — エラーゼロを確認
- [ ] `cargo publish --dry-run -p rinq-syntax` — エラーゼロを確認

### 5H-5: CHANGELOG.md 更新

- [ ] `CHANGELOG.md` に `## [v0.1.0] - 2026-XX-XX` エントリを追加
- [ ] Phase 5A〜5H の追加内容を列挙

### ✅ Phase 5H 完了チェック / RINQ v0.1.0 リリース
- [ ] 上記すべて完了

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
