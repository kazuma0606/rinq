# RINQ v4.0 タスク管理

チェックボックスの凡例: `[ ]` 未着手 / `[~]` 進行中 / `[x]` 完了

---

## Phase 4A: DX 強化

### D1: 型エイリアス

- [x] `src/lib.rs` に `InitialQuery<T>` / `FilteredQuery<T>` / `SortedQuery<T>` / `ProjectedQuery<U>` を追加
- [x] 型エイリアスを使った関数シグネチャの doc test を追加（`get_adults` / `make_range` の例）

### D2a: 型ステート診断トレイト

- [x] `src/core/state_diagnostics.rs` を新規作成
- [x] 内部マクロ `define_state_constraint!` を定義（ボイラープレート削減）
- [x] `SupportsSelect`（Filtered のみ）を定義
- [x] `SupportsThenBy`（Sorted のみ）を定義
- [x] `SupportsOrderBy`（Initial / Filtered）を定義
- [x] `src/core/mod.rs` に `state_diagnostics` をモジュール追加
- [x] コンパイルエラーが改善されることを手動確認（`trybuild` なし、一時ファイルで検証）

### D2b: 要素型 T の診断

- [x] `HashEqBound` トレイト（`distinct` / `union` 等向け）を定義
- [x] `distinct`（initial / filtered / sorted）/ `union` / `intersect` / `except` の境界に `HashEqBound` を適用
- [x] エラーメッセージ改善を手動確認（`User` に `Hash + Eq` がない場合、`HashEqBound` が明示される）

### D3: `rinq_explain!` マクロ

- [x] `src/macros/mod.rs` を新規作成
- [x] `rinq_explain!` を `macro_rules!` で実装（Option A: 総時間のみ）
- [x] `#[cfg(debug_assertions)]` / `#[cfg(not(debug_assertions))]` の動作確認
- [x] `src/lib.rs` から `#[macro_export]` で公開
- [x] doc test 追加（`rinq_explain!(query.collect::<Vec<_>>())` の例）

### D4: `pred!` マクロ

- [x] `src/macros/mod.rs` に `pred!` を追加
- [x] 単一条件（`pred!(age > 18)`）の展開確認
- [x] `&&` 連鎖（`pred!(age > 18 && active == true)`）の展開確認
- [x] doc test 追加
- [x] `src/lib.rs` から公開

### Phase 4A テスト確認

- [x] `cargo test` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Phase 4A 完了チェック
- [x] 上記すべて完了

---

## Phase 4B: ライフサイクル設計改善

### H1: `from_arc_cloned` / `from_arc_slice_cloned`

- [x] `src/core/builder/shared.rs` に `from_arc_cloned` を実装
- [x] `from_arc_slice_cloned`（`Arc<[T]>` 版）を実装
- [x] O(N) コピーを `///` コメントに明記
- [x] doc test 追加（`Arc<Vec<User>>` から `FilteredQuery<User>` を構築する例）
- [x] 統合テストに `from_arc_cloned` の基本動作・空 Vec・複数スレッドからの呼び出しを追加（`tests/rinq_v4_tests.rs`）

### H2a: `tap_each`

- [x] `src/core/builder/shared.rs` に `tap_each` を実装（`inspect` ラッパー相当）
- [x] doc test 追加

### H2b: `tap_collect`

- [x] `src/core/builder/shared.rs` に `tap_collect` を実装（全収集 → 副作用 → 再ラップ）
- [x] ドキュメントに `⚠ Eagerly collects all elements` を明記
- [x] doc test 追加（atomic カウンタの例）

### H2c: `pipe`

- [x] `src/core/builder/shared.rs` に `pipe` を実装（`FnOnce(Self) -> QueryBuilder<T2, S2>`）
- [x] doc test 追加（外部委譲の例）

### Phase 4B テスト確認

- [x] `cargo test` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Phase 4B 完了チェック
- [x] 上記すべて完了

---

## Phase 4C: クイックウィン演算子（Phase J）

### J1: `filter_map`

- [x] `src/core/builder/shared.rs` に `filter_map` を実装
- [x] 戻り値型が `QueryBuilder<U, Filtered>` であることを確認
- [x] doc test 追加（文字列を数値にパース、失敗を除外する例）

### J2: `map` alias

- [x] `src/core/builder/shared.rs` に `map` を `select` の alias として実装（Filtered state）
- [x] doc test 追加

### J3: `IntoQuery` トレイト

- [x] `src/core/builder/shared.rs` に `IntoQuery` トレイトを定義
- [x] `Vec<T>` への blanket impl を追加
- [x] doc test 追加（`users.into_query().where_(...).collect()` の例）

### J4: `collect_vec`

- [x] `src/core/builder/shared.rs` に `collect_vec` を実装
- [x] doc test 追加

### J5: `step_by`

- [x] `src/core/builder/shared.rs` に `step_by` を実装
- [x] `step == 0` でパニックすることを確認
- [x] doc test 追加

### J6: `cycle`

- [x] `src/core/builder/shared.rs` に `cycle` を実装
- [x] ドキュメントに `# Infinite loop` セクションを明記
- [x] doc test 追加（`take(7)` と組み合わせる例）

### Phase 4C テスト確認

- [x] `cargo test` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Phase 4C 完了チェック
- [x] 上記すべて完了

---

## Phase 4D: 新演算子（Phase E）

### 共通準備

- [x] `src/core/builder/functional.rs` を新規作成
- [x] `src/core/builder/mod.rs` に `functional` モジュールを追加

### E1: `scan`

- [x] `functional.rs` に `scan` を実装（B: Clone、ScanIter アダプタ）
- [x] doc test 追加（累積積 `[1,2,6,24,120]` の例）

### E2: `chunk_by`

- [x] `src/core/builder/iterators.rs` に `ChunkByIterator` を追加
- [x] `functional.rs` に `chunk_by` を実装
- [x] doc test 追加（`[1,1,2,2,3,1,1]` → `[[1,1],[2,2],[3],[1,1]]` の例）

### E3: `dedup` / `dedup_by`

- [x] `functional.rs` に `dedup` を実装（`PartialEq` ベース）
- [x] `functional.rs` に `dedup_by` を実装（キー関数ベース）
- [x] doc test 追加（非連続重複は残る例）

### E4: `zip_with`

- [x] `functional.rs` に `zip_with` を実装
- [x] doc test 追加（要素ごとの加算の例）

### E5: `pairwise`

- [x] `functional.rs` に `pairwise` を実装（PairwiseIter アダプタ）
- [x] doc test 追加（`[1,2,3,4]` → `[(1,2),(2,3),(3,4)]` の例）

### E6: `unfold` / `unfold_bounded`

- [x] `src/core/builder/iterators.rs` に `UnfoldIter` を実装
- [x] `UnfoldBoundedIter`（max カウンタ付き）を実装
- [x] `functional.rs` に `QueryBuilder<T, Initial>::unfold` を追加
- [x] `QueryBuilder<T, Initial>::unfold_bounded` を追加
- [x] `debug_assertions` 時のランタイム警告を実装
- [x] doc test 追加（フィボナッチ / `unfold_bounded` の例）

### E7: `intersperse`

- [x] `functional.rs` に `intersperse` を実装（IntersperseIter アダプタ）
- [x] doc test 追加（空白区切り連結の例）

### E8: `min_max`

- [x] `functional.rs` に `min_max` を実装（単一走査）
- [x] doc test 追加

### Phase 4D テスト確認

- [x] `cargo test` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Phase 4D 完了チェック
- [x] 上記すべて完了

---

## Phase 4E: `rinq-derive` クレート

### クレート作成

- [x] `rinq-derive/Cargo.toml` を作成（`proc-macro = true`、`syn` / `quote` / `proc-macro2` 依存）
- [x] `Cargo.toml`（ルート）のワークスペースに `rinq-derive` を追加
- [x] `rinq-derive/src/lib.rs` を作成（`derive(Queryable)` / `derive(QueryableFrom)` のエントリポイント）

### F1: `#[derive(Queryable)]`

- [x] `rinq-derive/src/queryable.rs` を作成
- [x] フィールドアクセサ生成ロジック（`by_*` 関数）を実装
- [x] 型付き述語モジュール（`user_fields::Age` 等）の生成ロジックを実装
- [x] `#[queryable(skip)]` 属性のサポート
- [x] `#[queryable(rename = "...")]` 属性のサポート
- [x] `#[queryable(key)]` 属性のサポート
- [x] `order_by` 向け（`&T` 参照版）と `group_by` 向け（所有版）のアクセサを分ける設計を実装
- [x] `Span::mixed_site()` による hygiene 対応
- [x] `rinq-derive/tests/derive_tests.rs` を作成
  - [x] 基本的な `#[derive(Queryable)]` の展開確認
  - [x] `Age::gt(18)` / `Age::lt(50)` / `Age::between(20, 40)` が正しく動作
  - [x] `Active::is_true()` / `Active::is_false()` が正しく動作
  - [x] `Name::contains("Alice")` が正しく動作
  - [x] `#[queryable(skip)]` でアクセサが生成されないことを確認
  - [x] `#[queryable(rename = "...")]` で関数名が変わることを確認
  - [x] ユーザー変数との名前衝突テスト（`user` / `__it` 等）

### F2: `#[derive(QueryableFrom)]`

- [x] `rinq-derive/src/from.rs` を作成
- [x] `From<MyCollection>` への `impl` 生成ロジックを実装
- [x] `IntoQuery` との連携テスト（`list.into_query()` が動作することを確認）

### Phase 4E テスト確認

- [x] `cargo test -p rinq-derive` 全件通過
- [x] `cargo clippy -p rinq-derive -- -D warnings` ゼロ
- [x] `rinq-derive` を使った統合例が `examples/` に動作することを確認

### ✅ Phase 4E 完了チェック
- [x] 上記すべて完了

---

## Phase 4F: `rinq-syntax` クレート

### G4: `rinq::__macro_support` 安定 API（rinq 本体側）

- [x] `src/__macro_support.rs` を新規作成
- [x] `__macro_support::from` を実装（エントリポイントのみ；メソッドチェーンは直接 QueryBuilder API を使用）
- [x] `#[doc(hidden)]` で一般ユーザーには非表示にしつつ `pub` で外部クレートから呼び出し可能に
- [x] `src/lib.rs` から `pub mod __macro_support` として公開
- [x] `__macro_support` のシグネチャ変更時に `#[deprecated]` 移行期間を設けるポリシーをコメントに明記

### クレート作成

- [x] `rinq-syntax/Cargo.toml` を作成（`proc-macro = true`、`syn` / `quote` / `proc-macro2` 依存）
- [x] ルートの `Cargo.toml` ワークスペースに `rinq-syntax` を追加
- [x] `rinq-syntax/src/lib.rs` を作成

### G1: `query!` マクロ — 基本構文

- [x] `rinq-syntax/src/parser.rs` を作成（`from` / `where` / `order_by` / `select` 節のパーサー）
- [x] `rinq-syntax/src/codegen.rs` を作成（`__macro_support::from` + QueryBuilder チェーンへの変換）
- [x] `from x in source` → `::rinq::__macro_support::from(source)` に展開
- [x] `where predicate` → `.where_(|x| { predicate })` に展開
- [x] `order_by key` → `.order_by(|x| key)` に展開（`desc` 指定で `order_by_descending`）
- [x] `select expr` → `.select(|x| { expr }).collect::<Vec<_>>()` に展開（省略時は直接 collect）
- [x] `take n` / `skip n` に対応
- [x] `rinq-syntax/src/ast.rs` を作成（`QueryInput` / `Clause` / `SortKey`）

### G2: 複数 `where` / 複数ソートキー

- [x] 複数の `where` 節を順番に `.where_()` チェーンに展開
- [x] `order_by key1, key2` を `order_by` + `then_by` に展開
- [x] 別行 `then_by` 節もサポート

### G3: エラーメッセージの改善

- [x] `order_by` の後に `where` が来た場合のカスタムエラーメッセージ
- [x] `from` が 2 回来た場合（v4.0 スコープ外 JOIN）のエラーメッセージ
- [x] `proc_macro::Span` を用いてエラーが展開前のソース位置を指すことを確認

### Phase 4F テスト確認

- [x] `cargo test -p rinq-syntax` 全件通過（15 テスト）
- [x] `cargo clippy -p rinq-syntax -- -D warnings` ゼロ
- [x] 基本クエリ（`from` / `where` / `order_by` / `select`）が期待通りに展開することを確認
- [x] `rinq` 本体の内部実装変更後に `rinq-syntax` が影響を受けないことを確認（`__macro_support` 層で隔離）

### ✅ Phase 4F 完了チェック
- [x] 上記すべて完了

---

## Phase 4G: `rinq-stats` 拡張

### I1: 時系列演算子

- [x] `rinq-stats/src/timeseries.rs` を新規作成
- [x] `TimeSeriesExt` トレイトを定義
- [x] `exponential_moving_average(alpha: f64)` を実装
  - [x] `alpha` が `(0.0, 1.0]` の範囲外の場合 `assert!` でパニック
- [x] `bollinger_bands(window: usize, sigma: f64)` を実装
  - [x] window=0 / window>len / window=1（std_dev=0）等の境界条件を確認
- [x] `rinq-stats/src/lib.rs` に re-export を追加（`TimeSeriesExt`, `BollingerPoint`）
- [x] `rinq-stats/tests/timeseries_tests.rs` を作成（12 テスト）
  - [x] EMA — 基本値確認 / alpha=1.0（現在値のみ）/ alpha=0.5 / 空 / 単一要素
  - [x] ボリンジャーバンド — 基本値確認 / 中央バンド = 移動平均との一致 / 空 / window > len

### I2: 外れ値検出

- [x] `rinq-stats/src/outliers.rs` を新規作成
- [x] `OutlierExt` トレイトを定義
- [x] `remove_outliers_zscore(threshold: f64)` を実装（2 パス）
- [x] `remove_outliers_iqr()` を実装（四分位範囲ベース）
- [x] `rinq-stats/src/lib.rs` に re-export を追加（`OutlierExt`）
- [x] `rinq-stats/tests/outlier_tests.rs` を作成（10 テスト）
  - [x] z-score — 外れ値あり / 外れ値なし / 空 / threshold=0 / 全同値
  - [x] IQR — 基本 / 対称分布 / 歪み分布 / 空 / 4 件未満

### I3: `ValidationExt` 拡張

- [x] `rinq-stats/src/validation.rs` に `validate_if` を追加（条件付き検証）
- [x] `validate_with` を追加（動的メッセージファクトリ）
- [x] `ValidationRule<T>` 内部型に `CheckFn<T>` エイリアスを導入（Clippy 対応）
- [x] `rinq-stats/tests/validation_tests.rs` に追加テストを追記（6 テスト）
  - [x] `validate_if` — condition=false でスキップ / condition=true で検証実行
  - [x] `validate_with` — カスタムメッセージ生成 / 全通過 / 複数失敗の正確なメッセージ

### Phase 4G テスト確認

- [x] `cargo test -p rinq-stats` 全件通過（144 テスト）
- [x] `cargo clippy -p rinq-stats -- -D warnings` ゼロ

### ✅ Phase 4G 完了チェック
- [x] 上記すべて完了

---

## Phase 4H: ドキュメント・公開準備

### コードコメントの英語化

- [x] `src/core/state.rs` の `Filtered` 状態説明を英語コメントで更新（「chainable intermediate state」と明記）
- [x] Phase 4A〜4D で追加した全メソッドに英語 `///` コメントを確認・補完（実装時に付与済み）
- [x] Phase 4E〜4G の全公開 API に英語 `///` コメントを確認・補完（実装時に付与済み）

### メタデータ整備

- [x] `rinq-derive/Cargo.toml` — `description` / `license` / `repository` / `keywords` / `categories` / `readme` / `[package.metadata.docs.rs]` 整備済み
- [x] `rinq-syntax/Cargo.toml` — 同様のメタデータ整備済み（experimental 注記あり）
- [x] `[package.metadata.docs.rs] all-features = true` — 全クレート（rinq / rinq-stats / rinq-derive / rinq-syntax）に追加済み
- [x] `rinq` / `rinq-stats` のバージョンを 4.0.0 に統一

### README.md

- [x] v4 の新機能セクションを追加
  - [x] 型エイリアス（`FilteredQuery<T>` 等）の使用例
  - [x] 新演算子一覧（`scan`, `chunk_by`, `filter_map`, `pipe` 等）
  - [x] `rinq-derive` のクイックスタート（`#[derive(Queryable)]` の例）
  - [x] `rinq-syntax` のクイックスタート（`query!` の例、experimental 注記付き）
  - [x] `rinq-stats` v4 拡張（EMA・ボリンジャーバンド・外れ値検出・`validate_if/validate_with`）

### CHANGELOG.md

- [x] `CHANGELOG.md` に v4.0.0 エントリを追加
- [x] Breaking Changes なし を明記
- [x] Phase 4A〜4H の全追加項目を列挙

### 最終確認

- [x] `cargo test` 全件通過（rinq + rinq-stats + rinq-derive）
- [x] `cargo test --all-features` 全件通過（type-inference 修正含む）
- [x] `cargo test -p rinq-syntax` 全件通過（15 テスト）
- [x] `cargo test --doc` 全件通過
- [x] `cargo doc --no-deps --all-features` 警告ゼロ（doc リンク修正含む）
- [x] `cargo clippy --all-features -- -D warnings` ゼロ
- [x] `cargo bench --no-run` 通過

### ✅ Phase 4H 完了チェック / RINQ v4.0 リリース
- [x] 上記すべて完了

---

## 付録: v4 で明確化した設計制約

### `Filtered` 状態の意味論

`scan` / `pairwise` / `unfold` 等が `Filtered` を返すのは「フィルタした」からではなく、
「連鎖可能な中間状態」を意味するため。新演算子の戻り値を決める際は以下の表を参照:

| 状態 | 意味 |
|---|---|
| `Initial` | 生成直後。`where_`/`order_by` 等への入口 |
| `Filtered` | 連鎖可能な中間状態。`select`/`order_by` 等に進める |
| `Sorted` | ソート済み。`then_by` / 終端操作のみ |
| `Projected<U>` | 射影済み。`collect()` のみ |

- [ ] Phase 4D の新演算子すべてが上記の状態遷移方針に従っていることを確認

### `unfold` / `cycle` の無限ループ

- [ ] `unfold` の doc test が `unfold_bounded` または `take` と組み合わせていることを確認
- [ ] `cycle` の doc test が `take` と組み合わせていることを確認

### `tap_collect` の eager 化

- [ ] `tap_collect` の `///` コメントに `⚠ Eagerly collects all elements` が含まれていることを確認
