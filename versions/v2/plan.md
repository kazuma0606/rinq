# RINQ v2.0 実装計画

**作成日**: 2026-03-25

---

## 目標

v1.0 の型ステートパターン・ゼロコスト抽象化・遅延評価という設計思想を維持しながら、LINQ 標準クエリ演算子との主要な差分を埋め、コード構造を長期的な拡張に耐える形に整備する。

---

## 設計方針

### 変更しないもの
- 型ステートパターン（`Initial → Filtered → Sorted / Projected`）
- 既存の全公開メソッドのシグネチャ
- `MetricsQueryBuilder` の外部 API
- `Queryable` トレイトのインターフェース
- 依存クレート（`thiserror`, `num-traits`, `parking_lot`）

### 変更するもの
- `RinqError` から `InvalidState`・`TypeMismatch` の2バリアントを削除
- `src/core/builder.rs`（1910行）を `src/core/builder/` サブディレクトリに分割
- `src/metrics/builder.rs`（1043行）を `src/metrics/builder/` サブディレクトリに分割

### 追加するもの
- 新規オペレータ（優先度順に M2〜M4 で段階的に実装）
- 全オペレータのドキュメントコメントに実行種別（即時/遅延ストリーミング/遅延非ストリーミング）を明記
- 新規オペレータのベンチマーク

---

## フェーズ構成

```
M1: 破壊的変更の先行処理（エラー型整理・テスト修正）
  ↓
M2: 高優先度オペレータの実装
    flat_map / take_while / skip_while / contains
    first_or_default / last_or_default / single / single_or_default
  ↓
M3: 中優先度オペレータの実装
    order_by_descending / then_by_descending / aggregate / distinct_by
    concat / union / intersect / except
    to_hashmap / to_lookup / element_at
  ↓
M4: 低優先度オペレータの実装（生成演算子）
    QueryBuilder::range / repeat / empty
  ↓
M5: MetricsQueryBuilder への新オペレータ反映
  ↓
M6: モジュール分割（builder.rs → builder/）
  ↓
M7: ドキュメント・CHANGELOG 整備
```

各マイルストーンは **`cargo test` 全件通過・`cargo clippy -- -D warnings` 警告ゼロを確認してから完了**とする。

---

## 各フェーズ詳細

### M1: 破壊的変更の先行処理

**目的**: 後続の実装に影響する破壊的変更を最初に済ませ、以降のマイルストーンを安定した基盤の上で進める。

**作業**:
- `src/core/error.rs` から `InvalidState`・`TypeMismatch` を削除
- `tests/rinq_property_tests.rs` の `test_rinq_error_messages` を修正
  - `TypeMismatch` の構築・検証コードを削除
  - `InvalidState` に関するコードが存在する場合は削除
- `cargo clippy` を実行し、削除バリアントを参照している箇所がないことを確認

**確認**: `cargo test` 全件通過

---

### M2: 高優先度オペレータの実装

**目的**: 日常的な用途で最も使用頻度が高く、実装コストが低いオペレータを追加する。

#### `flat_map` (SelectMany)

- **対象状態**: `Initial`, `Filtered`
- **戻り値の状態**: `Filtered`
- **実行種別**: 遅延ストリーミング
- **シグネチャ**: `fn flat_map<U, I, F>(self, f: F) -> QueryBuilder<U, Filtered>`
  - `F: Fn(T) -> I + 'static`, `I: IntoIterator<Item = U>`, `U: 'static`
- **実装**: `std::iter::Iterator::flat_map` をラップ

#### `take_while`

- **対象状態**: `Initial`, `Filtered`, `Sorted`
- **戻り値の状態**: `Filtered`
- **実行種別**: 遅延ストリーミング
- **シグネチャ**: `fn take_while<F>(self, predicate: F) -> QueryBuilder<T, Filtered>`
  - `F: Fn(&T) -> bool + 'static`

#### `skip_while`

- **対象状態**: `Initial`, `Filtered`, `Sorted`
- **戻り値の状態**: `Filtered`
- **実行種別**: 遅延ストリーミング
- **シグネチャ**: `fn skip_while<F>(self, predicate: F) -> QueryBuilder<T, Filtered>`
  - `F: Fn(&T) -> bool + 'static`

#### `contains`

- **対象状態**: `Initial`, `Filtered`, `Sorted`
- **実行種別**: 即時実行
- **シグネチャ**: `fn contains(self, value: &T) -> bool`
  - `T: PartialEq`

#### `first_or_default` / `last_or_default`

- **対象状態**: 全状態
- **実行種別**: 即時実行
- **シグネチャ**: `fn first_or_default(self) -> T` / `fn last_or_default(self) -> T`
  - `T: Default`
- **動作**: 空コレクションの場合 `T::default()` を返す

#### `single` / `single_or_default`

- **対象状態**: 全状態
- **実行種別**: 即時実行
- **シグネチャ**: `fn single(self) -> RinqResult<T>` / `fn single_or_default(self) -> RinqResult<T>`
  - `single_or_default` は `T: Default`
- **動作**:

| ケース | `single` | `single_or_default` |
|--------|----------|---------------------|
| 0件 | `Err(IteratorExhausted)` | `Ok(T::default())` |
| 1件 | `Ok(element)` | `Ok(element)` |
| 2件以上 | `Err(ExecutionError)` | `Err(ExecutionError)` |

**確認**: `cargo test` 全件通過 / 各オペレータの doc test を含む

---

### M3: 中優先度オペレータの実装

**目的**: 実用性が高いが実装にやや複雑さが伴うオペレータを追加する。

#### `order_by_descending` / `then_by_descending`

- v1.0 の `order_by` / `then_by` の実装を参考に、`Ordering` を反転させる形で実装
- `order_by_descending`: `Initial` → `Sorted`
- `then_by_descending`: `Sorted` → `Sorted`

#### `aggregate` / `aggregate_no_seed`

- **実行種別**: 即時実行
- **シグネチャ**:
  ```rust
  fn aggregate<Acc, F>(self, seed: Acc, f: F) -> Acc
      where F: Fn(Acc, T) -> Acc
  fn aggregate_no_seed<F>(self, f: F) -> Option<T>
      where F: Fn(T, T) -> T
  ```

#### `distinct_by`

- **実行種別**: 遅延非ストリーミング
- **シグネチャ**: `fn distinct_by<K, F>(self, key: F) -> QueryBuilder<T, Filtered>`
  - `K: Hash + Eq`, `F: Fn(&T) -> K + 'static`
- **実装**: `HashSet<K>` で既出キーを追跡

#### `concat`

- **実行種別**: 遅延ストリーミング
- **シグネチャ**: `fn concat(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered>`
- **実装**: `std::iter::Iterator::chain` をラップ

#### `union` / `intersect` / `except`

- **実行種別**: 遅延非ストリーミング（内部 `HashSet` を使用）
- **制約**: `T: Hash + Eq`
- **シグネチャ**:
  ```rust
  fn union(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered>
  fn intersect(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered>
  fn except(self, other: impl IntoIterator<Item = T> + 'static) -> QueryBuilder<T, Filtered>
  ```
- **動作**:
  - `union`: 両方の要素を重複除去して返す
  - `intersect`: 両方に含まれる要素のみ返す
  - `except`: `self` に含まれ `other` に含まれない要素のみ返す

#### `to_hashmap` / `to_lookup`

- **実行種別**: 即時実行
- **シグネチャ**:
  ```rust
  fn to_hashmap<K, F>(self, key: F) -> RinqResult<HashMap<K, T>>
      where K: Hash + Eq, F: Fn(&T) -> K
  fn to_lookup<K, F>(self, key: F) -> HashMap<K, Vec<T>>
      where K: Hash + Eq, F: Fn(&T) -> K
  ```
- `to_hashmap` は重複キーが発生した場合 `Err(ExecutionError)` を返す

#### `element_at`

- **実行種別**: 即時実行
- **シグネチャ**: `fn element_at(self, index: usize) -> Option<T>`
- **実装**: `Iterator::nth` をラップ

**確認**: `cargo test` 全件通過

---

### M4: 低優先度オペレータの実装（生成演算子）

**目的**: データソースを必要としない静的なビルダー生成メソッドを追加する。

#### `QueryBuilder::range`

- **シグネチャ**: `fn range<R>(range: R) -> QueryBuilder<i64, Initial>`
  - `R: Iterator<Item = i64> + 'static`
  - または `RangeBounds` を受け取る形（実装時に決定）
- **用途**: テストデータの生成、合成シーケンスの構築

#### `QueryBuilder::repeat`

- **シグネチャ**: `fn repeat(value: T, count: usize) -> QueryBuilder<T, Initial>`
  - `T: Clone + 'static`

#### `QueryBuilder::empty`

- **シグネチャ**: `fn empty() -> QueryBuilder<T, Initial>`

**確認**: `cargo test` 全件通過

---

### M5: MetricsQueryBuilder への新オペレータ反映

**目的**: `QueryBuilder` に追加したすべての新規オペレータを `MetricsQueryBuilder` にも実装する。

**作業**:
- M2〜M4 で追加した各オペレータの `MetricsQueryBuilder` ラッパーを実装
- 即時実行オペレータ（`contains`, `single`, `aggregate`, `to_hashmap` 等）にはメトリクス記録を追加
- 遅延オペレータ（`flat_map`, `take_while`, `concat` 等）はそのまま内部 `QueryBuilder` に委譲
- メトリクスキー命名規則の適用:
  - `single()` → `query_{name}_single`
  - `aggregate()` → `query_{name}_aggregate`
  - `contains()` → `query_{name}_contains`

**確認**: `cargo test --test metrics_tests` 全件通過

---

### M6: モジュール分割（builder.rs → builder/）

**目的**: `src/core/builder.rs`（M2〜M4 実装後は 2500行超の見込み）と `src/metrics/builder.rs` を分割し、長期的な保守性を確保する。

**作業（core）**:
```
src/core/builder.rs → src/core/builder/
  mod.rs         QueryBuilder<T,State> 構造体 + QueryData<T> enum (pub(crate))
  iterators.rs   ChunkIterator, WindowIterator 等
  initial.rs     impl QueryBuilder<T, Initial>
  filtered.rs    impl QueryBuilder<T, Filtered>
  sorted.rs      impl QueryBuilder<T, Sorted>
  shared.rs      impl QueryBuilder<T, State>（状態横断メソッド）
  queryable.rs   Queryable トレイト + 各コレクション impl
```

**作業（metrics）**:
```
src/metrics/builder.rs → src/metrics/builder/
  mod.rs    MetricsQueryBuilder 構造体
  impl.rs   全状態の impl ブロック
```

**注意事項**:
- `QueryData<T>` は `pub(crate)` にして各サブモジュールからアクセス可能にする
- `src/core/mod.rs` の re-export パスを更新（`builder::` → `builder::mod::` の変更が不要なよう `mod.rs` で再エクスポート）
- 公開 API（`rinq::*`）は一切変更しない
- 分割はリファクタリングのみ。このフェーズで機能変更をしない

**確認**: `cargo test` 全件通過 / `cargo clippy -- -D warnings` ゼロ

---

### M7: ドキュメント・CHANGELOG 整備

**目的**: コードの変更をドキュメントに反映し、リリース準備を完了させる。

**作業**:
- `CHANGELOG.md` に v2.0 エントリを追加
  - Breaking Changes（`RinqError` バリアント削除）を明記
  - 新規オペレータの全リストを記載
- `CLAUDE.md` の `### All Implemented Operations` テーブルを v2.0 の内容に更新
- `examples/rinq_basic_usage.rs` に新オペレータの使用例を追加
- `versions/v2/spec.md` と実装の整合性を最終確認
- 全オペレータのドキュメントコメントに実行種別を明記（`/// **実行種別**: 遅延ストリーミング` 等）

**確認**: `cargo test --doc` 全件通過 / `cargo doc --no-deps` エラーなし

---

## リスク・注意事項

### `flat_map` の型パラメータ

`flat_map` は出力型 `U` が入力型 `T` と異なる場合があるため、`QueryBuilder<U, Filtered>` を返す必要があります。既存の `select` の実装（`Projected<U>` 遷移）を参考にしつつ、`flat_map` は直接 `Filtered` に遷移させる点に注意してください。

### `union` / `intersect` / `except` の `T: Hash + Eq` 制約

これらのオペレータは `T: Hash + Eq` を要求するため、浮動小数点型（`f32`, `f64`）などには適用できません。コンパイルエラーとして自然に弾かれますが、ドキュメントに明記してください。

### M6 の分割タイミング

モジュール分割（M6）は機能追加が完了した後に行います。分割中は `cargo test` を頻繁に実行し、リファクタリングによる動作変更がないことを確認してください。

### proptest のオーバーフロー

新規オペレータの proptest クロージャでは、既存テストと同様に `saturating_mul`・`saturating_neg` 等の飽和演算を使用し、ランダム `i32` 入力によるオーバーフローパニックを防いでください。
