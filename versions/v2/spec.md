# RINQ v2.0 仕様書

**作成日**: 2026-03-25
**ステータス**: Draft

---

## 概要

RINQ v2.0 は v1.0 を基盤とし、LINQ 標準クエリ演算子との差分分析（`issues/linq-gap-analysis.md`）をもとにオペレータを大幅に拡充するメジャーアップデートです。

v1.0 の設計思想（型ステートパターンによるコンパイル時安全性・ゼロコスト抽象化・遅延評価）はそのまま継承し、以下の3つの柱で改善を行います。

1. **オペレータ拡充** — LINQ との主要な差分を埋める
2. **エラー型の整理** — デッドコードとなっているバリアントを削除し実態に即した設計へ
3. **モジュール分割** — builder.rs の肥大化に対応し、長期的な拡張に耐える構造へ

---

## v1.0 からの破壊的変更

| 項目 | v1.0 | v2.0 | 影響 |
|------|------|------|------|
| `RinqError::InvalidState` | あり | **削除** | パターンマッチの網羅性チェックに影響 |
| `RinqError::TypeMismatch` | あり | **削除** | 同上 |
| `src/core/builder.rs` | 単一ファイル | `src/core/builder/` に分割 | 内部パスの変更（公開APIは不変） |

---

## 遅延実行モデルの明示

v2.0 では実行モデルをドキュメントおよびコメントで明示します。すべてのオペレータは以下の3種別のいずれかに分類されます。

| 種別 | 動作 | 代表的なオペレータ |
|------|------|------------------|
| **即時実行** | 呼び出し時点でイテレータを全走査し、スカラー値または新コレクションを返す | `count`, `sum`, `average`, `min`, `max`, `first`, `last`, `any`, `all`, `collect`, `contains`, `single`, `aggregate`, `to_hashmap`, `to_lookup` |
| **遅延ストリーミング** | 要素を1つずつ生成する。全バッファ不要 | `where_`, `select`, `flat_map`, `take`, `take_while`, `skip`, `skip_while`, `inspect`, `concat`, `zip`, `enumerate` |
| **遅延非ストリーミング** | 遅延評価だが、最初の要素を生成する前に入力全体をバッファする | `order_by`, `order_by_descending`, `group_by`, `reverse`, `distinct`, `union`, `intersect`, `except`, `chunk`, `window` |

> **設計方針**: 非ストリーミングオペレータは `QueryData::SortedVec` パスを通るため、大規模データでのメモリ使用に注意が必要です。この特性を各メソッドのドキュメントコメントに明記します。

---

## モジュール構造

### v2.0 のディレクトリ構成

```
src/
  lib.rs
  core/
    mod.rs
    builder/
      mod.rs        — QueryBuilder<T,State> 構造体 + QueryData<T> enum (pub(crate))
      iterators.rs  — ChunkIterator, WindowIterator 等のカスタムイテレータアダプタ
      initial.rs    — impl QueryBuilder<T, Initial>
      filtered.rs   — impl QueryBuilder<T, Filtered>
      sorted.rs     — impl QueryBuilder<T, Sorted>
      shared.rs     — impl QueryBuilder<T, State>（状態横断の汎用メソッド）
      queryable.rs  — Queryable トレイト + 各コレクション型の impl
    error.rs
    state.rs
  metrics/
    mod.rs
    builder/
      mod.rs
      impl.rs
    collector.rs
```

v1.0 から変更されるのは内部ファイル構造のみです。公開 API（`rinq::*` のすべての識別子）は変わりません。

---

## エラー型（`core::error`）

### v2.0 の `RinqError`

```rust
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RinqError {
    /// クエリの構築が不正な場合
    #[error("Invalid query construction: {message}")]
    InvalidQuery { message: String },

    /// イテレータが枯渇した場合（空コレクションへの要素アクセス等）
    #[error("Iterator exhausted")]
    IteratorExhausted,

    /// クエリ実行中のランタイムエラー
    #[error("Query execution failed: {message}")]
    ExecutionError { message: String },
}

pub type RinqResult<T> = Result<T, RinqError>;
```

### 削除するバリアント

| バリアント | 削除理由 |
|-----------|---------|
| `InvalidState` | 型ステートパターンにより不正な状態遷移はコンパイル時に排除される。ランタイムで発生し得ないため概念的に矛盾している。 |
| `TypeMismatch` | 静的型付けのRustでは型の不一致はコンパイルエラーになる。ランタイムエラーとして定義する意味がない。 |

### 各バリアントの実際の使用箇所

| バリアント | 使用するオペレータ |
|-----------|------------------|
| `IteratorExhausted` | `first()`, `last()`, `single()`, `element_at()` — 空コレクションへのアクセス時 |
| `InvalidQuery` | `chunk(0)`, `window(0)` — 無効な引数が渡された時 |
| `ExecutionError` | `single()` — 要素が複数存在する時（期待値：1件のみ） |

> **設計方針**: `enum` のまま維持。`thiserror` derive によるパターンマッチ・`?` 演算子での伝播はRustの慣用表現であり、`Box<dyn Error>` スタイルへの変更は不要。

---

## 公開 API

### トップレベル re-export（変更なし）

| 識別子 | 説明 |
|--------|------|
| `rinq::QueryBuilder` | クエリビルダー本体 |
| `rinq::Queryable` | コレクションをクエリに変換するトレイト |
| `rinq::RinqError` | エラー型（v2.0でバリアント削減） |
| `rinq::RinqResult<T>` | `Result<T, RinqError>` エイリアス |
| `rinq::Initial` | 型状態: 初期 |
| `rinq::Filtered` | 型状態: フィルタ済み |
| `rinq::Projected<U>` | 型状態: 射影済み |
| `rinq::Sorted` | 型状態: ソート済み |
| `rinq::MetricsQueryBuilder` | メトリクス付きクエリビルダー |
| `rinq::MetricsCollector` | メトリクス収集器 |

---

## 型ステートパターン（変更なし）

v2.0 でも型ステートによるコンパイル時安全性を維持します。任意の操作連鎖（LINQ のような `Select → Where → Select`）はサポートしません。これは RINQ の設計思想であり、バグの温床になりやすい操作順序の誤りをコンパイル時に排除することを優先します。

```
Initial ──where_/take/skip/flat_map/...──→ Filtered ──select──→ Projected<U>
   │                                            │
   └──order_by/order_by_descending──→ Sorted ──┘
                                         │
                                    then_by/then_by_descending
```

再クエリが必要な場合は `collect()` して新たな `QueryBuilder` を生成するパターンを使います。

---

## `QueryBuilder<T, State>` — 新規・変更オペレータ

### `Initial` 状態の全メソッド

#### フィルタリング・パーティショニング

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `where_(predicate)` | `Filtered` | 遅延ストリーミング | 条件フィルタ |
| `take(n)` | `Filtered` | 遅延ストリーミング | 先頭 n 件 |
| `take_while(predicate)` | `Filtered` | 遅延ストリーミング | **[NEW]** 条件が偽になるまで取得 |
| `skip(n)` | `Filtered` | 遅延ストリーミング | 先頭 n 件スキップ |
| `skip_while(predicate)` | `Filtered` | 遅延ストリーミング | **[NEW]** 条件が偽になるまでスキップ |
| `distinct()` | `Filtered` | 遅延非ストリーミング | 重複除去（`Hash + Eq` 要求） |
| `distinct_by(key)` | `Filtered` | 遅延非ストリーミング | **[NEW]** キーによる重複除去 |

#### 変換・投影

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `inspect(f)` | `Initial` | 遅延ストリーミング | 副作用なし観察（デバッグ用） |
| `flat_map(f)` | `Filtered` | 遅延ストリーミング | **[NEW]** ネスト平坦化（SelectMany） |
| `enumerate()` | `Filtered` | 遅延ストリーミング | インデックス付与 |

#### シーケンス結合・集合演算

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `zip(other)` | `Filtered` | 遅延ストリーミング | 別イテレータとペアリング |
| `concat(other)` | `Filtered` | 遅延ストリーミング | **[NEW]** 2シーケンスを連結 |
| `union(other)` | `Filtered` | 遅延非ストリーミング | **[NEW]** 集合和（重複除去） |
| `intersect(other)` | `Filtered` | 遅延非ストリーミング | **[NEW]** 集合積 |
| `except(other)` | `Filtered` | 遅延非ストリーミング | **[NEW]** 集合差 |

#### ソート

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `order_by(key)` | `Sorted` | 遅延非ストリーミング | 昇順ソート |
| `order_by_descending(key)` | `Sorted` | 遅延非ストリーミング | **[NEW]** 降順ソート |

#### シーケンス分解

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `reverse()` | `Filtered` | 遅延非ストリーミング | 逆順 |
| `chunk(n)` | `Filtered` | 遅延ストリーミング | 固定サイズチャンク |
| `window(n)` | `Filtered` | 遅延ストリーミング | スライディングウィンドウ |

#### 即時実行（終端操作）

| メソッド | 戻り値型 | 説明 |
|----------|---------|------|
| `collect::<B>()` | `B: FromIterator<T>` | コレクションに収集 |
| `count()` | `usize` | 要素数 |
| `sum()` | `T` | 合計 |
| `average()` | `Option<f64>` | 平均 |
| `min()` | `Option<T>` | 最小値 |
| `max()` | `Option<T>` | 最大値 |
| `min_by(key)` | `Option<T>` | キーで最小の要素 |
| `max_by(key)` | `Option<T>` | キーで最大の要素 |
| `aggregate(seed, f)` | `Acc` | **[NEW]** カスタム畳み込み（シードあり） |
| `aggregate_no_seed(f)` | `Option<T>` | **[NEW]** カスタム畳み込み（シードなし） |
| `first()` | `Option<T>` | 先頭要素 |
| `first_or_default()` | `T` | **[NEW]** 先頭要素（空なら `T::default()`、`Default` 要求） |
| `last()` | `Option<T>` | 末尾要素 |
| `last_or_default()` | `T` | **[NEW]** 末尾要素（空なら `T::default()`、`Default` 要求） |
| `single()` | `RinqResult<T>` | **[NEW]** 要素が1件のみの場合に返す。0件→`IteratorExhausted`、2件以上→`ExecutionError` |
| `single_or_default()` | `RinqResult<T>` | **[NEW]** 0件→`T::default()`、1件→その要素、2件以上→`ExecutionError` |
| `element_at(index)` | `Option<T>` | **[NEW]** インデックスアクセス（範囲外は `None`） |
| `any(predicate)` | `bool` | いずれかが条件を満たすか |
| `all(predicate)` | `bool` | すべてが条件を満たすか |
| `contains(value)` | `bool` | **[NEW]** 線形探索による存在確認（`PartialEq` 要求） |
| `group_by(key)` | `HashMap<K, Vec<T>>` | グループ化 |
| `group_by_aggregate(key, agg)` | `HashMap<K, R>` | 集計付きグループ化 |
| `partition(predicate)` | `(Vec<T>, Vec<T>)` | 条件で2分割 |
| `to_hashmap(key)` | `RinqResult<HashMap<K, T>>` | **[NEW]** キーセレクタで辞書化（重複キーは `ExecutionError`） |
| `to_lookup(key)` | `HashMap<K, Vec<T>>` | **[NEW]** キーセレクタで辞書化（重複キーは `Vec`） |

### `Filtered` 状態のメソッド

`Initial` 状態のすべてのメソッドに加え、以下が追加されます。

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `select(projection)` | `Projected<U>` | 遅延ストリーミング | 型変換・射影 |

### `Sorted` 状態のメソッド

| メソッド | 戻り値の状態 | 実行種別 | 説明 |
|----------|-------------|----------|------|
| `then_by(key)` | `Sorted` | 遅延非ストリーミング | 第2ソートキー（昇順） |
| `then_by_descending(key)` | `Sorted` | 遅延非ストリーミング | **[NEW]** 第2ソートキー（降順） |
| `min()` | *(即時)* | 即時実行 | O(1)（ソート済み先頭） |
| `max()` | *(即時)* | 即時実行 | O(1)（ソート済み末尾） |
| その他終端操作 | — | 即時実行 | `Initial`/`Filtered` と同等 |

### 生成演算子（静的コンストラクタ）

| メソッド | 説明 |
|----------|------|
| `QueryBuilder::range(range)` | **[NEW]** 数値範囲からビルダーを生成（例: `1..=100`） |
| `QueryBuilder::repeat(value, n)` | **[NEW]** 要素を n 回繰り返すビルダーを生成 |
| `QueryBuilder::empty::<T>()` | **[NEW]** 空のビルダーを生成 |

---

## `Queryable` トレイト（変更なし）

```rust
pub trait Queryable<T> {
    fn into_query(self) -> QueryBuilder<T, Initial>;
}
```

実装済み: `Vec<T>`, `&[T]`, `[T; N]`, `HashSet<T>`, `BTreeSet<T>`, `LinkedList<T>`, `VecDeque<T>`

---

## メトリクスモジュール（`rinq::metrics`）

`MetricsQueryBuilder` は `QueryBuilder` の新規オペレータをすべて包含します。メトリクスキーの命名規則は v1.0 から変更ありません。

新規オペレータのキー例:
- `flat_map` 後の `collect()` → `query_{name}`
- `single()` → `query_{name}_single`
- `aggregate()` → `query_{name}_aggregate`

---

## 依存関係

| クレート | バージョン | 用途 | 変更 |
|----------|-----------|------|------|
| `thiserror` | 1.0 | `RinqError` derive | 変更なし |
| `num-traits` | 0.2 | `average()` の `ToPrimitive` | 変更なし |
| `parking_lot` | 0.12 | `MetricsCollector` の RwLock | 変更なし |

---

## テスト方針

v1.0 のテストファイル構成を継承しつつ、新規オペレータのテストを追加します。

| ファイル | 追加内容 |
|----------|---------|
| `tests/core_tests.rs` | 新規オペレータ（`flat_map`, `take_while`, `contains` 等）の基本動作テスト |
| `tests/rinq_v0_2_tests.rs` | 集合演算（`union`, `intersect`, `except`）、コレクション変換（`to_hashmap`, `to_lookup`）のテスト |
| `tests/rinq_property_tests.rs` | `single()` のエラー条件、`aggregate()` の結合則、エラーバリアント削除に伴うテスト更新 |

---

## パフォーマンス特性（追加分）

| 操作 | 計算量 | 備考 |
|------|-------|------|
| `flat_map` | O(n×m) | ネストの深さ m に依存 |
| `take_while` / `skip_while` | O(k) | k = 条件が成立する要素数 |
| `contains` | O(n) | 線形探索。ソート済みデータには `Sorted` 状態後に使用 |
| `union` / `intersect` / `except` | O(n + m) | 内部 `HashSet` を使用 |
| `concat` | O(1) per element | ストリーミングのため追加バッファなし |
| `to_hashmap` / `to_lookup` | O(n) | HashMap 構築コスト |
| `aggregate` | O(n) | クロージャのコストに依存 |
| `single` | O(n) | 全走査して件数を確認 |

---

## v1.0 からの変更点まとめ

| カテゴリ | 変更内容 |
|---------|---------|
| **破壊的変更** | `RinqError::InvalidState`、`RinqError::TypeMismatch` を削除 |
| **新規オペレータ（高優先度）** | `flat_map`, `take_while`, `skip_while`, `contains`, `first_or_default`, `last_or_default`, `single`, `single_or_default` |
| **新規オペレータ（中優先度）** | `order_by_descending`, `then_by_descending`, `aggregate`, `aggregate_no_seed`, `distinct_by`, `concat`, `union`, `intersect`, `except`, `to_hashmap`, `to_lookup`, `element_at` |
| **新規オペレータ（低優先度）** | `QueryBuilder::range`, `QueryBuilder::repeat`, `QueryBuilder::empty` |
| **内部構造** | `src/core/builder.rs` → `src/core/builder/` に分割 |
| **ドキュメント** | 全オペレータに実行種別（即時/遅延ストリーミング/遅延非ストリーミング）を明記 |
| **非対応（Phase 3）** | `join`, `group_join`（`docs/implementation.md` のロードマップ維持） |

---

## 使用例

```rust
use rinq::QueryBuilder;

// flat_map でネスト平坦化
let words = vec![
    vec!["hello", "world"],
    vec!["foo", "bar"],
];
let result: Vec<&str> = QueryBuilder::from(words)
    .flat_map(|v| v)
    .collect();
// ["hello", "world", "foo", "bar"]

// take_while / skip_while
let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 1, 2])
    .take_while(|x| *x < 4)
    .collect();
// [1, 2, 3]

// aggregate でカスタム畳み込み
let product = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .aggregate(1, |acc, x| acc * x);
// 120

// single で一意性の保証
let result = QueryBuilder::from(vec![42])
    .single();
// Ok(42)

let err = QueryBuilder::from(vec![1, 2, 3])
    .single();
// Err(RinqError::ExecutionError { ... })

// 集合演算
let a = QueryBuilder::from(vec![1, 2, 3, 4]);
let b = vec![3, 4, 5, 6];
let union: Vec<i32> = a.union(b).collect();
// [1, 2, 3, 4, 5, 6]

// 降順ソートと then_by_descending
let data = vec![(3, "a"), (1, "c"), (3, "b"), (1, "a")];
let result: Vec<_> = QueryBuilder::from(data)
    .order_by_descending(|(n, _)| *n)
    .then_by(|(_, s)| *s)
    .collect();
// [(3, "a"), (3, "b"), (1, "a"), (1, "c")]

// to_lookup で重複キー対応辞書
use std::collections::HashMap;
let data = vec![("a", 1), ("b", 2), ("a", 3)];
let lookup: HashMap<&str, Vec<i32>> = QueryBuilder::from(data)
    .to_lookup(|(k, _)| *k)
    .into_iter()
    .map(|(k, vs)| (k, vs.into_iter().map(|(_, v)| v).collect()))
    .collect();
// {"a": [1, 3], "b": [2]}

// 生成演算子
let squares: Vec<i32> = QueryBuilder::range(1..=5)
    .select(|x| x * x)
    .collect();
// [1, 4, 9, 16, 25]
```
