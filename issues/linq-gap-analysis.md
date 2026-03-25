# LINQ Gap Analysis — RINQへの改善案

**作成日**: 2026-03-24
**更新日**: 2026-03-24
**参照**: [LINQ 標準クエリ演算子 (Microsoft Learn)](https://learn.microsoft.com/ja-jp/dotnet/csharp/linq/standard-query-operators/)

---

## 概要

本ドキュメントは、C# LINQ の標準クエリ演算子ドキュメントを参照し、RINQ（v1.0）との差分を分析した結果をまとめたものです。

---

## 遅延実行の2分類

LINQは遅延実行を2種類に分けており、RINQの設計にも反映できる重要な概念です。

| 種別 | 動作 | 該当オペレータ例 |
|---|---|---|
| **即時実行** | 即座に全データを読み、スカラーを返す | `count`, `sum`, `average`, `min`, `max`, `first`, `last`, `any`, `all`, `collect` |
| **遅延ストリーミング** | 1要素ずつ処理、バッファ不要 | `where_`, `select`, `take`, `skip`, `concat`, `flat_map` |
| **遅延非ストリーミング** | 全データをバッファしてから出力 | `order_by`, `group_by`, `reverse`, `intersect`, `except` |

> **現状**: RINQの `QueryData<T>` の `SortedVec` パスが非ストリーミング用バッファを担っているが、この区別はドキュメントに明示されていない。コード・ドキュメントへの明記を推奨。

---

## 不足オペレータ一覧

### フィルタリング

| LINQ | RINQ | 状態 |
|---|---|---|
| `Where` | `where_` | ✅ 実装済み |
| `Take` | `take` | ✅ 実装済み |
| `TakeWhile` | — | ❌ **未実装** |
| `Skip` | `skip` | ✅ 実装済み |
| `SkipWhile` | — | ❌ **未実装** |
| `Distinct` | `distinct` | ✅ 実装済み |
| `OfType` | — | ❌ **未実装**（型フィルタリングキャスト） |

### 変換・投影

| LINQ | RINQ | 状態 |
|---|---|---|
| `Select` | `select` | ✅ 実装済み |
| `SelectMany` | — | ❌ **未実装**（ネスト平坦化 = `flat_map`） |
| `Cast` | — | ❌ **未実装** |

### ソート

| LINQ | RINQ | 状態 |
|---|---|---|
| `OrderBy` | `order_by` | ✅ 実装済み |
| `OrderByDescending` | — | ❌ **未実装** |
| `ThenBy` | `then_by` | ✅ 実装済み |
| `ThenByDescending` | — | ❌ **未実装** |

### スカラー集計（即時実行）

| LINQ | RINQ | 状態 |
|---|---|---|
| `Count` | `count` | ✅ 実装済み |
| `Sum` | `sum` | ✅ 実装済み |
| `Average` | `average` | ✅ 実装済み |
| `Min` / `Max` | `min` / `max` / `min_by` / `max_by` | ✅ 実装済み |
| `Aggregate` | — | ❌ **未実装**（汎用fold/reduce） |
| `Contains` | — | ❌ **未実装** |
| `SequenceEqual` | — | ❌ **未実装** |

### コレクション変換・グループ化

| LINQ | RINQ | 状態 |
|---|---|---|
| `GroupBy` | `group_by` | ✅ 実装済み |
| `ToList` / `ToArray` | `collect` | ✅ 実装済み |
| `ToDictionary` | — | ❌ **未実装** |
| `ToLookup` | — | ❌ **未実装**（重複キー対応辞書） |

### 要素アクセス・存在確認

| LINQ | RINQ | 状態 |
|---|---|---|
| `First` | `first` | ✅ 実装済み |
| `FirstOrDefault` | — | ❌ **未実装** |
| `Last` | `last` | ✅ 実装済み |
| `LastOrDefault` | — | ❌ **未実装** |
| `Single` | — | ❌ **未実装**（要素が1つのみの場合に返す） |
| `SingleOrDefault` | — | ❌ **未実装** |
| `ElementAt` | — | ❌ **未実装**（インデックスアクセス） |
| `Any` | `any` | ✅ 実装済み |
| `All` | `all` | ✅ 実装済み |
| `DefaultIfEmpty` | — | ❌ **未実装** |

### シーケンス操作

| LINQ | RINQ | 状態 |
|---|---|---|
| `Concat` | — | ❌ **未実装**（2シーケンスの連結） |
| `Union` | — | ❌ **未実装**（集合和、重複除去） |
| `Intersect` | — | ❌ **未実装**（集合積） |
| `Except` | — | ❌ **未実装**（集合差） |
| `Zip` | `zip` | ✅ 実装済み |
| `Reverse` | `reverse` | ✅ 実装済み |
| `Chunk` | `chunk` | ✅ 実装済み |

### 生成演算子

| LINQ | RINQ | 状態 |
|---|---|---|
| `Range` | — | ❌ **未実装**（数値範囲の生成） |
| `Repeat` | — | ❌ **未実装**（要素の繰り返し生成） |
| `Empty` | — | ❌ **未実装** |

### 結合（Join）

| LINQ | RINQ | 状態 |
|---|---|---|
| `Join` | — | ❌ **未実装**（内部結合） |
| `GroupJoin` | — | ❌ **未実装**（外部結合・階層結合） |

> **備考**: `docs/implementation.md` の Phase 2 ロードマップに記載済み。

---

## 実装優先度

### 高優先度（日常的に使用、実装コスト低）

1. **`flat_map`** (SelectMany) — ネストされたコレクションの平坦化。最も実用頻度が高い。
2. **`take_while` / `skip_while`** — 既存 `take`/`skip` の述語ベース拡張。
3. **`contains`** — 線形探索の存在確認。即時実行。
4. **`first_or_default` / `last_or_default`** — `Option<T>` を返すバリアント。
5. **`single` / `single_or_default`** — 要素数のアサーション付き取得。

### 中優先度（実用性が高い、やや複雑）

6. **`order_by_descending` / `then_by_descending`** — ソートの完全性。
7. **`aggregate`** — カスタム畳み込み（シードあり/なし）。
8. **`distinct_by`** — KeySelector 付き distinct。
9. **`concat`** — 2シーケンスの連結。ストリーミング遅延実行。
10. **`union` / `intersect` / `except`** — 集合演算。
11. **`to_hashmap` / `to_lookup`** — コレクション変換。
12. **`element_at`** — インデックスアクセス。

### 低優先度（特殊用途 or 大きな設計変更が必要）

13. **`QueryBuilder::range` / `repeat` / `empty`** — 静的生成演算子。
14. **`join` / `group_join`** — Phase 2 ロードマップ済み。
15. **`rinq!` マクロ** — クエリ構文糖衣。

---

## 設計上の考察

### 型ステートと操作連鎖の制約

RINQの型ステートパターン（`Initial → Filtered → Sorted / Projected`）はLINQにはないコンパイル時安全性を提供する設計上の強みです。

一方で、LINQでは任意の順序で演算子を連鎖できます（例: `Where → Select → Where → SelectMany → OrderBy → Take`）。RINQの線形ステート進行では、`select` 後に再度 `where_` をかけるといったパターンが表現できません。

**対応方針の選択肢**:
- 「制約あり = RINQ の設計思想」として明示的にドキュメント化する
- 型ステートを再設計して柔軟な連鎖を許可する（破壊的変更）

### `IOrderedEnumerable` との対応

LINQの `IOrderedEnumerable<T>` は `ThenBy`/`ThenByDescending` を型安全に連鎖させるための専用型です。RINQの `Sorted` 型ステートは同じ制約を正しくモデル化しており、この設計は良好です。

---

## 実装例（API設計案）

```rust
// flat_map (SelectMany)
builder.flat_map(|item| item.tags)

// take_while / skip_while
builder.take_while(|x| *x < 10)
builder.skip_while(|x| *x < 10)

// aggregate (seed あり)
builder.aggregate(0, |acc, x| acc + x)

// contains
builder.contains(&42)

// single / single_or_default
builder.single()                // 要素が1つでなければ Err
builder.single_or_default()    // 要素が0または1つ

// *OrDefault 系
builder.first_or_default()     // Option<T> ではなく T (Default 実装前提)
builder.last_or_default()

// 降順ソート
builder.order_by_descending(|x| x.score)
builder.then_by_descending(|x| x.name)

// 集合演算
builder.concat(other)
builder.union(other)
builder.intersect(other)
builder.except(other)

// コレクション変換
builder.to_hashmap(|x| x.id)           // 重複キーは Err
builder.to_lookup(|x| x.category)      // 重複キーは Vec

// 生成演算子（静的コンストラクタ）
QueryBuilder::range(1..=100)
QueryBuilder::repeat(value, count)
QueryBuilder::empty::<T>()
```

---

## コード構造の改善案

### builder.rs の分割

`src/core/builder.rs` は 1910行、`src/metrics/builder.rs` は 1043行に達しており、今後の機能追加に伴いさらに肥大化する見込みです。以下の構成への分割を推奨します。

```
src/core/
  builder/
    mod.rs        — QueryBuilder<T,State> 構造体 + QueryData<T> enum（pub(crate)）
    iterators.rs  — ChunkIterator, WindowIterator（カスタムイテレータアダプタ）
    initial.rs    — impl QueryBuilder<T, Initial>（構築・変換系）
    filtered.rs   — impl QueryBuilder<T, Filtered>（フィルタ後の操作）
    sorted.rs     — impl QueryBuilder<T, Sorted>（ソート後の操作）
    shared.rs     — impl QueryBuilder<T, State>（状態横断の汎用メソッド）
    queryable.rs  — Queryable トレイト + Vec/HashSet 等の impl
  error.rs
  state.rs
```

**タイミング**: 機能追加（`flat_map`、`concat` 等）をある程度実装してから一括で行う方が効率的。

---

### RinqError の整理

#### 調査結果

`src/` 内の全ソースコードを調査した結果、**5つのエラーバリアントのうち、ライブラリの実装コードで実際に生成・返却されているものは0件**でした。

| バリアント | `src/` での使用 | `tests/` での使用 | 判定 |
|---|---|---|---|
| `InvalidQuery` | なし | フォーマット確認のみ | ⚠️ 未使用 |
| `IteratorExhausted` | なし | フォーマット確認のみ | ⚠️ 未使用 |
| `ExecutionError` | なし | フォーマット確認のみ | ⚠️ 未使用 |
| `InvalidState` | なし | **なし** | ❌ 完全デッドコード |
| `TypeMismatch` | なし | フォーマット確認のみ | ⚠️ 未使用（静的型付けのRustでは概念的にも不要） |

テスト（`rinq_property_tests.rs:1539`）はエラーを構築してメッセージ文字列を確認しているだけで、ライブラリがエラーを実際に返すパスのテストではありません。

#### 推奨対応

- **`InvalidState` を削除**: 型ステートパターンによってコンパイル時に不正状態を排除しているため、ランタイムで発生し得ない。概念的にも矛盾している。
- **`TypeMismatch` を削除**: 静的型付けのRustでは型の不一致はコンパイルエラーになるため、ランタイムエラーとして定義する意味がない。
- **残り3バリアントは実際に使う箇所に合わせて整理**: `first()`/`last()` が空コレクションに対して返す場合等、実際の使用箇所に合わせてバリアントを見直す。

#### enum vs struct+trait について

**enum のまま維持を推奨**。`thiserror` を使った enum Error はRustの慣用表現であり、パターンマッチによる網羅性チェック・`?` 演算子での伝播が自然にできます。struct+trait（`Box<dyn Error>` スタイル）はプラグインシステムや動的エラー合成が必要な場合に有効ですが、純粋なクエリエンジンであるRINQには不要です。
