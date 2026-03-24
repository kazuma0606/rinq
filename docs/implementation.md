# RINQ 実装ロードマップ

このドキュメントは RINQ v1.0 リリース以降の機能拡張計画を定義します。

## 現状（v1.0 リリース済み）

```
rinq v1.0.0
├── rinq::core::*          型安全クエリエンジン
│   ├── QueryBuilder<T, State>   遅延評価フルエントAPI
│   ├── Queryable トレイト        Vec / &[T] / HashSet / BTreeSet 対応
│   ├── RinqError / RinqResult   thiserror ベースエラー型
│   └── 型状態: Initial → Filtered → Sorted / Projected<U>
└── rinq::metrics::*       メトリクス統合
    ├── MetricsQueryBuilder      クエリ実行回数を自動計測
    └── MetricsCollector         parking_lot::RwLock ベースカウンタ
```

**実装済みオペレーション一覧**

| カテゴリ | メソッド |
|---|---|
| フィルタリング | `where_`, `take`, `skip` |
| 変換 | `select`, `inspect` |
| ソート | `order_by`, `then_by` |
| 集約（スカラー） | `count`, `sum`, `average`, `min`, `max`, `min_by`, `max_by` |
| 集約（コレクション） | `group_by`, `partition` |
| 終端 | `first`, `last`, `any`, `all`, `collect` |
| シーケンス変換 | `distinct`, `reverse`, `chunk`, `window`, `zip` |

テスト数: 262件（単体 + 統合 + プロパティ + doctest）

---

## Phase 2: RINQ Join — 複数コレクション結合

**優先度**: 高

### 主要機能

```rust
// 内部結合
let result = QueryBuilder::from(orders)
    .join(
        users,
        |order| order.user_id,
        |user| user.id,
        |order, user| (order.id, user.name, order.amount),
    )
    .collect();

// 左外部結合
let result = QueryBuilder::from(orders)
    .left_join(
        users,
        |order| order.user_id,
        |user| user.id,
        |order, user_opt| (order.id, user_opt.map(|u| u.name)),
    )
    .collect();

// 集合演算
let union      = QueryBuilder::from(set1).union(set2).collect();
let intersect  = QueryBuilder::from(set1).intersect(set2).collect();
let except     = QueryBuilder::from(set1).except(set2).collect();
let concat     = QueryBuilder::from(vec1).concat(vec2).collect();
```

### 技術的考慮事項

- **Hash Join**: 大きなコレクション用（O(n + m)）。小さい側を `HashMap` に変換。
- **Nested Loop Join**: 小さなコレクション用（O(n × m)）。
- 型状態 `Joined` を追加し、結合後の状態を明示。
- `T: Hash + Eq` 制約が必要なメソッドと不要なメソッドを分離。

---

## Phase 3: RINQ Parallel — Rayon 並列処理

**優先度**: 中

### 主要機能

```rust
use rinq::ParallelQueryBuilder;

let data: Vec<i32> = (0..1_000_000).collect();

let result: Vec<_> = ParallelQueryBuilder::from(data)
    .par_where(|x| expensive_predicate(x))
    .par_select(|x| expensive_transform(x))
    .collect();

let total: i32 = ParallelQueryBuilder::from(large_data)
    .par_where(|x| *x > 0)
    .par_sum();
```

### 技術的考慮事項

- `rayon` を `[dependencies]` に追加（`feature = ["parallel"]` でオプション化を検討）。
- `T: Send` / `F: Sync + Send` 制約。
- 要素数が閾値未満の場合は自動的に逐次処理にフォールバック。
- `QueryBuilder<T, State>` と `ParallelQueryBuilder<T, State>` で API を揃える。

---

## Phase 4: RINQ Async — Stream 非同期処理

**優先度**: 中

### 主要機能

```rust
use rinq::AsyncQueryBuilder;

let result: Vec<_> = AsyncQueryBuilder::from_stream(api_stream)
    .where_(|x| async move { validate(x).await })
    .select(|x| async move { transform(x).await })
    .collect()
    .await;

let count = AsyncQueryBuilder::from_stream(stream)
    .where_(|x| async move { *x > 0 })
    .count()
    .await;
```

### 技術的考慮事項

- `futures` / `tokio-stream` との統合。
- `Pin<Box<dyn Stream<Item = T> + Send>>` をコア型に。
- バックプレッシャー対応: `buffer(n)` で N 件単位のバッチ処理。
- `tokio` を `[dev-dependencies]` に追加（本体は依存させない）。

---

## Phase 5: RINQ WASM — ブラウザ / Node.js 対応

**優先度**: 低

### 主要機能

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmQueryBuilder { /* ... */ }

#[wasm_bindgen]
impl WasmQueryBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Self { todo!() }
    pub fn where_clause(&self, predicate: &js_sys::Function) -> Self { todo!() }
    pub fn collect(&self) -> JsValue { todo!() }
}
```

```typescript
// 自動生成される TypeScript 型定義
export class QueryBuilder<T> {
  constructor(data: T[]);
  where(predicate: (x: T) => boolean): QueryBuilder<T>;
  select<U>(projection: (x: T) => U): QueryBuilder<U>;
  collect(): T[];
}
```

### 技術的考慮事項

- `wasm-bindgen` + `wasm-pack` を使用。
- コアロジックは `#[cfg(not(target_arch = "wasm32"))]` で分岐不要になるよう設計。
- npm パッケージ名: `@rinq/core`。

---

## Phase 6: RINQ Macro — コンパイル時クエリ最適化

**優先度**: 低（実験的）

### 主要機能

```rust
use rinq::query;

// SQL 風マクロ構文
let result = query! {
    from x in users
    where x.age > 18
    select x.name
    order_by x.age
    take 10
};
```

### 技術的考慮事項

- `rinq-macro` として別クレートで開発（proc-macro クレートは分離が必須）。
- `query!` マクロは `QueryBuilder` メソッドチェーンに展開。
- `syn` + `quote` を使用。

---

## 推奨実装順序

```
v1.0 ✅ リリース済み
  ↓
Phase 2: Join        ← 実用性が高く、既存 API を壊さない
  ↓
Phase 3: Parallel    ← パフォーマンス訴求
  ↓
Phase 4: Async       ← 非同期エコシステムとの統合
  ↓
Phase 5: WASM        ← JavaScript 展開
  ↓
Phase 6: Macro       ← DX 向上（実験的）
```

各 Phase の実装前に `versions/v<N>/` 以下に `spec.md`、`plan.md`、`tasks.md` を作成してから着手する。
