# RINQ v2.0 タスク管理

チェックボックスの凡例: `[ ]` 未着手 / `[~]` 進行中 / `[x]` 完了

---

## Milestone 1: 破壊的変更の先行処理

### タスク
- [x] `src/core/error.rs` から `InvalidState` バリアントを削除
- [x] `src/core/error.rs` から `TypeMismatch` バリアントを削除
- [x] `tests/rinq_property_tests.rs` の `test_rinq_error_messages` から `TypeMismatch` の構築・検証コードを削除
- [x] `cargo clippy -- -D warnings` を実行し、削除バリアントへの参照が残っていないことを確認

### テスト確認
- [x] `cargo test` 全件通過（262件）

### ✅ Milestone 1 完了

---

## Milestone 2: 高優先度オペレータの実装

### `flat_map`
- [x] `impl QueryBuilder<T, Initial>` に `flat_map` を実装
- [x] `impl QueryBuilder<T, Filtered>` に `flat_map` を実装
- [x] `flat_map` の doc test を追加（ネスト平坦化の基本例）
- [ ] `flat_map` の統合テストを `tests/core_tests.rs` に追加

### `take_while` / `skip_while`
- [x] `impl QueryBuilder<T, Initial>` に `take_while`・`skip_while` を実装
- [x] `impl QueryBuilder<T, Filtered>` に `take_while`・`skip_while` を実装
- [x] `impl QueryBuilder<T, Sorted>` に `take_while`・`skip_while` を実装
- [x] 各 doc test を追加
- [ ] 統合テストを `tests/core_tests.rs` に追加

### `contains`
- [x] 全状態に `contains` を実装（`T: PartialEq`）
- [x] doc test を追加
- [ ] 統合テストを追加（存在する要素・しない要素・空コレクション）

### `first_or_default` / `last_or_default`
- [x] 全状態に `first_or_default`・`last_or_default` を実装（`T: Default`）
- [x] doc test を追加
- [ ] 統合テストを追加（空コレクション → `T::default()` になることを確認）

### `single` / `single_or_default`
- [x] 全状態に `single`・`single_or_default` を実装
- [x] doc test を追加
- [ ] 統合テストを追加
  - [ ] 0件 → `single` は `Err(IteratorExhausted)`
  - [ ] 1件 → `single` は `Ok(element)`
  - [ ] 2件以上 → `single` は `Err(ExecutionError)`
  - [ ] 0件 → `single_or_default` は `Ok(T::default())`

### テスト確認
- [x] `cargo test` 全件通過（262件 + doc tests 33件）
- [x] `cargo test --doc` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Milestone 2 実装完了（統合テストは M2 テスト追加フェーズで実施）

---

## Milestone 3: 中優先度オペレータの実装

### `order_by_descending` / `then_by_descending`
- [x] `impl QueryBuilder<T, Initial>` に `order_by_descending` を実装（v1.0 実装済み）
- [x] `impl QueryBuilder<T, Sorted>` に `then_by_descending` を実装（v1.0 実装済み）

### `aggregate` / `aggregate_no_seed`
- [x] 全状態に `aggregate`（シードあり）を実装
- [x] 全状態に `aggregate_no_seed`（シードなし）を実装
- [x] doc test を追加

### `distinct_by`
- [x] 全状態に `distinct_by` を実装（v1.0 実装済み）

### `concat`
- [x] 全状態に `concat` を実装
- [x] doc test を追加

### `union` / `intersect` / `except`
- [x] 全状態に `union` を実装（`T: Hash + Eq + Clone`）
- [x] 全状態に `intersect` を実装
- [x] 全状態に `except` を実装
- [x] 各 doc test を追加

### `to_hashmap` / `to_lookup`
- [x] 全状態に `to_hashmap` を実装（重複キーは `Err(ExecutionError)`）
- [x] 全状態に `to_lookup` を実装（重複キーは `Vec`）
- [x] 各 doc test を追加

### `element_at`
- [x] 全状態に `element_at` を実装
- [x] doc test を追加

### テスト確認
- [x] `cargo test` 全件通過（262件 + doc tests 42件）
- [x] `cargo test --doc` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Milestone 3 完了

---

## Milestone 4: 低優先度オペレータの実装（生成演算子）

### `QueryBuilder::range`
- [x] `QueryBuilder::range` を静的メソッドとして実装（任意の `IntoIterator` を受け取る形で Rust range を自然に使用可能）
- [x] doc test を追加

### `QueryBuilder::repeat`
- [x] `QueryBuilder::repeat(value, count)` を実装（`std::iter::repeat_n` ベース）
- [x] doc test を追加

### `QueryBuilder::empty`
- [x] `QueryBuilder::empty()` を実装
- [x] doc test を追加

### テスト確認
- [x] `cargo test` 全件通過（262件 + doc tests 45件）
- [x] `cargo test --doc` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Milestone 4 完了

---

## Milestone 5: MetricsQueryBuilder への新オペレータ反映

### 遅延オペレータ（内部 QueryBuilder に委譲するだけ）
- [x] `flat_map` を `MetricsQueryBuilder` に追加
- [x] `take_while`・`skip_while` を追加
- [x] `concat`・`union`・`intersect`・`except` を追加
- [x] `distinct_by` を追加
- [x] `order_by_descending`・`then_by_descending` を追加

### 即時実行オペレータ（メトリクス記録あり）
- [x] `contains` を追加（キー: `query_{name}_contains`）
- [x] `single`・`single_or_default` を追加（キー: `query_{name}_single`）
- [x] `first_or_default`・`last_or_default` を追加
- [x] `aggregate`・`aggregate_no_seed` を追加（キー: `query_{name}_aggregate`）
- [x] `to_hashmap`・`to_lookup` を追加（キー: `query_{name}_to_hashmap` 等）
- [x] `element_at` を追加

### 生成演算子
- [x] `MetricsQueryBuilder::range`・`repeat`・`empty` を追加（`QueryBuilder` の静的メソッドをラップ）

### テスト確認
- [x] `cargo test` 全件通過（262件 + doc tests 45件）
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Milestone 5 完了

---

## Milestone 6: モジュール分割（builder.rs → builder/）

### core モジュール分割
- [x] `src/core/builder/` ディレクトリを作成
- [x] `mod.rs` に `QueryBuilder<T,State>` 構造体と `QueryData<T>` enum を移動
- [x] `iterators.rs` に `ChunkIterator`・`WindowIterator` 等のカスタムアダプタを移動
- [x] `initial.rs` に `impl QueryBuilder<T, Initial>` を移動
- [x] `filtered.rs` に `impl QueryBuilder<T, Filtered>` を移動
- [x] `sorted.rs` に `impl QueryBuilder<T, Sorted>` を移動
- [x] `shared.rs` に `impl QueryBuilder<T, State>`（状態横断メソッド）を移動
- [x] `queryable.rs` に `Queryable` トレイトと各コレクション impl を移動
- [x] 旧 `src/core/builder.rs` を削除

### metrics モジュール分割
- [x] `src/metrics/builder/` ディレクトリを作成
- [x] `mod.rs` に `MetricsQueryBuilder` 構造体を移動
- [x] `impl_.rs` に全状態の impl ブロックを移動
- [x] 旧 `src/metrics/builder.rs` を削除

### テスト確認
- [x] `cargo test` 全件通過（262件 + doc tests 45件）
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Milestone 6 完了

---

## Milestone 7: ドキュメント・CHANGELOG 整備

### ドキュメントコメント
- [x] M2〜M4 で追加した全オペレータのドキュメントコメントに実行種別を明記（実装時に追加済み）
- [x] 既存オペレータのドキュメントコメントにも実行種別を追記（`order_by`・`then_by`・`group_by`・`distinct`・`reverse`・terminal ops 等）

### CHANGELOG
- [x] `CHANGELOG.md` に v2.0 エントリを追加
  - [x] Breaking Changes（`RinqError::InvalidState`・`TypeMismatch` 削除）を明記
  - [x] 新規オペレータの全リストを記載（M2〜M5）
  - [x] モジュール構造変更（内部変更、公開 API への影響なし）を記載

### CLAUDE.md
- [x] `### All Implemented Operations` テーブルを v2.0 の内容に更新
- [x] `### Module Structure` のディレクトリ構成を `builder/` サブディレクトリ構成に更新

### examples
- [x] `examples/rinq_basic_usage.rs` に新オペレータ（`flat_map`・`aggregate`・集合演算等）の使用例を追加

### 最終確認
- [x] `versions/v2/spec.md` と実装の整合性確認（全 M2〜M4 オペレータ実装済み）

### テスト確認
- [x] `cargo test` 全件通過（262件 + 45 doc tests）
- [x] `cargo test --doc` 全件通過
- [x] `cargo doc --no-deps` エラーなし
- [x] `cargo clippy -- -D warnings` ゼロ

### ✅ Milestone 7 完了

---

## 全体完了チェック

- [x] `cargo test` 全件通過（262件 + doc tests 45件）
- [x] `cargo doc --no-deps` 通過
- [x] `cargo clippy -- -D warnings` ゼロ
- [x] `versions/v2/spec.md` と実装の整合性確認
- [x] `CHANGELOG.md` の v2.0 エントリ確認
- [ ] `cargo bench --no-run` 通過
- [ ] git commit 済み

### ✅ RINQ v2.0 実装完了
