# RINQ 進化計画 - 実装フェーズ

このドキュメントは、RINQ v0.1の完成後、次のフェーズでの機能拡張と進化の実装計画を定義します。

## 現状（v0.1完成時点）

✅ **完成している機能**:
- 型安全なクエリ構築（型状態パターン）
- フィルタリング（`where_`）
- プロジェクション（`select`）
- ソート（`order_by`, `then_by`）
- ページネーション（`take`, `skip`）
- 集約（`count`, `first`, `last`, `any`, `all`）
- デバッグ（`inspect`）
- 複数コレクション対応（Queryableトレイト）
- メトリクス統合（MetricsQueryBuilder）
- 115個のテスト、19個のベンチマーク

---

## Phase 1: RINQ v0.2 - 集約と変換の拡張 ⭐

**優先度**: 最高  
**期間**: 1-2週  
**spec-kit構造**: `.kiro/specs/rinq-v0.2/`

### 概要

In-Memoryコレクションに対する高度な集約、変換、グループ化機能を追加し、データ分析とレポーティングの用途で即座に実用的な価値を提供します。

### 主要機能

#### 1. グループ化と集約
```rust
// group_by() - キーでグループ化してHashMap/BTreeMapを返す
let grouped: HashMap<char, Vec<String>> = QueryBuilder::from(users)
    .group_by(|u| u.name.chars().next().unwrap());

// group_by_aggregate() - グループごとに集約
let summary = QueryBuilder::from(orders)
    .group_by_aggregate(
        |o| o.user_id,              // キーセレクタ
        |group| group.iter().map(|o| o.amount).sum() // 集約関数
    );
```

#### 2. 数値集約関数
```rust
// sum() - 合計
let total: i32 = QueryBuilder::from(vec![1, 2, 3, 4, 5]).sum();

// average() - 平均
let avg: f64 = QueryBuilder::from(vec![1, 2, 3, 4, 5]).average();

// min() / max() - 最小・最大
let min = QueryBuilder::from(vec![5, 2, 8, 1]).min(); // Some(1)
let max = QueryBuilder::from(vec![5, 2, 8, 1]).max(); // Some(8)

// min_by() / max_by() - キーセレクタ付き
let youngest = QueryBuilder::from(users)
    .min_by(|u| u.age);
```

#### 3. 重複排除
```rust
// distinct() - 重複を削除（Hashベース）
let unique: Vec<_> = QueryBuilder::from(vec![1, 2, 2, 3, 3, 3])
    .distinct()
    .collect(); // [1, 2, 3]

// distinct_by() - キーで重複判定
let unique_names = QueryBuilder::from(users)
    .distinct_by(|u| u.name.clone())
    .collect();
```

#### 4. シーケンス操作
```rust
// reverse() - 順序を反転
let reversed: Vec<_> = QueryBuilder::from(vec![1, 2, 3])
    .reverse()
    .collect(); // [3, 2, 1]

// chunk() - 固定サイズに分割
let chunks: Vec<Vec<_>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .chunk(2)
    .collect(); // [[1, 2], [3, 4], [5]]

// window() - スライディングウィンドウ
let windows: Vec<Vec<_>> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .window(3)
    .collect(); // [[1,2,3], [2,3,4], [3,4,5]]
```

#### 5. コレクション組み合わせ
```rust
// zip() - 2つのコレクションをペアリング
let zipped: Vec<(i32, char)> = QueryBuilder::from(vec![1, 2, 3])
    .zip(vec!['a', 'b', 'c'])
    .collect(); // [(1,'a'), (2,'b'), (3,'c')]

// enumerate() - インデックス付き
let indexed: Vec<(usize, i32)> = QueryBuilder::from(vec![10, 20, 30])
    .enumerate()
    .collect(); // [(0,10), (1,20), (2,30)]
```

#### 6. 条件分岐
```rust
// partition() - 条件で2つに分割
let (evens, odds) = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .partition(|x| *x % 2 == 0);
// evens: [2, 4], odds: [1, 3, 5]
```

### 技術的考慮事項

**型状態の拡張**:
```rust
struct Grouped;    // group_by後の状態
struct Chunked;    // chunk後の状態
```

**新しいQueryData variant**:
```rust
enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec { items: Vec<T>, comparator: Box<dyn Fn(&T, &T) -> Ordering> },
    Grouped(HashMap<K, Vec<T>>),  // 新規
    Reversed(VecDeque<T>),         // 新規
}
```

### テスト戦略

- プロパティテスト: 各新機能に3-5個
- 単体テスト: エッジケース（空、単一要素など）
- ベンチマーク: 手書きループとの比較

### 期待される成果

- 実用性の大幅向上
- データ分析・レポーティングで即座に使用可能
- 既存のイテレータAPIとの互換性維持

---

## Phase 2: RINQ Join - 複数コレクション操作

**優先度**: 高  
**期間**: 2-3週  
**spec-kit構造**: `.kiro/specs/rinq-join/`

### 概要

複数のコレクション間の結合操作を提供し、リレーショナルなデータ処理を可能にします。

### 主要機能

#### 1. 内部結合
```rust
let orders = vec![
    Order { id: 1, user_id: 1, amount: 100 },
    Order { id: 2, user_id: 2, amount: 200 },
];

let users = vec![
    User { id: 1, name: "Alice" },
    User { id: 2, name: "Bob" },
];

let result = QueryBuilder::from(orders)
    .join(
        users,
        |order| order.user_id,        // 左キー
        |user| user.id,                // 右キー
        |order, user| (order.id, user.name, order.amount)  // 結果プロジェクション
    )
    .collect();
// [(1, "Alice", 100), (2, "Bob", 200)]
```

#### 2. 外部結合
```rust
// left_join() - 左外部結合
let result = QueryBuilder::from(orders)
    .left_join(
        users,
        |order| order.user_id,
        |user| user.id,
        |order, user_opt| (order.id, user_opt.map(|u| u.name))
    )
    .collect();

// right_join() - 右外部結合
// full_join() - 完全外部結合
```

#### 3. 直積
```rust
// cross_join() - デカルト積
let colors = vec!["Red", "Blue"];
let sizes = vec!["S", "M", "L"];

let products = QueryBuilder::from(colors)
    .cross_join(sizes)
    .select(|(color, size)| format!("{} {}", color, size))
    .collect();
// ["Red S", "Red M", "Red L", "Blue S", "Blue M", "Blue L"]
```

#### 4. 集合演算
```rust
// union() - 和集合
let combined = QueryBuilder::from(set1)
    .union(set2)
    .collect();

// intersect() - 積集合
let common = QueryBuilder::from(set1)
    .intersect(set2)
    .collect();

// except() - 差集合
let diff = QueryBuilder::from(set1)
    .except(set2)
    .collect();

// concat() - 単純連結
let concatenated = QueryBuilder::from(vec1)
    .concat(vec2)
    .collect();
```

### 技術的考慮事項

**結合アルゴリズム**:
- Hash Join: 大きなコレクション用（O(n + m)）
- Nested Loop Join: 小さなコレクション用（O(n * m)）
- 自動選択ロジック

**メモリ効率**:
- 小さい方のコレクションをHashMapに変換
- 遅延評価を可能な限り維持

**型システム**:
```rust
struct Joined;    // join後の状態
struct Unioned;   // union後の状態
```

### 期待される成果

- リレーショナルなデータ処理が可能に
- 複数データソースの統合が容易に
- データベース統合の準備

---

## Phase 3: RINQ Parallel - 並列処理版

**優先度**: 中  
**期間**: 2-3週  
**spec-kit構造**: `.kiro/specs/rinq-parallel/`

### 概要

Rayonを活用した並列処理版QueryBuilderを提供し、マルチコアを活用した高速なデータ処理を実現します。

### 主要機能

#### 1. 並列クエリビルダー
```rust
use rusted_ca::domain::rinq::ParallelQueryBuilder;

let data: Vec<i32> = (0..1_000_000).collect();

let result: Vec<_> = ParallelQueryBuilder::from(data)
    .par_where(|x| expensive_predicate(x))  // 並列フィルタ
    .par_select(|x| expensive_transform(x)) // 並列マップ
    .collect();
```

#### 2. 並列集約
```rust
// par_sum() - 並列合計
let total: i32 = ParallelQueryBuilder::from(large_dataset)
    .par_where(|x| *x > 0)
    .par_sum();

// par_count(), par_min(), par_max()
let stats = ParallelQueryBuilder::from(data)
    .par_aggregate(|group| (group.sum(), group.count(), group.min(), group.max()));
```

#### 3. スレッドプール制御
```rust
// スレッド数の制御
let result = ParallelQueryBuilder::from(data)
    .with_threads(4)
    .par_where(|x| *x > 0)
    .collect();

// 閾値設定（小さいコレクションは並列化しない）
let result = ParallelQueryBuilder::from(data)
    .with_threshold(1000)  // 1000要素未満は逐次処理
    .par_where(|x| *x > 0)
    .collect();
```

### 技術的考慮事項

**Rayon統合**:
```rust
use rayon::prelude::*;

impl<T: Send> ParallelQueryBuilder<T, Initial> {
    pub fn par_where<F>(self, predicate: F) -> ParallelQueryBuilder<T, Filtered>
    where
        F: Fn(&T) -> bool + Sync + Send,
    {
        // Rayonのpar_iter()を活用
    }
}
```

**トレイト境界**:
- `T: Send` - スレッド間でムーブ可能
- `F: Sync` - 複数スレッドから参照可能
- `F: Send` - スレッド間で転送可能

**パフォーマンス最適化**:
- Work-stealing scheduler
- データ局所性の考慮
- オーバーヘッドの最小化

### 期待される成果

- 大規模データセットで10x-100xの高速化
- CPUコアの効率的な活用
- 既存のQueryBuilderと同じAPI体験

---

## Phase 4: RINQ Async - 非同期版

**優先度**: 中  
**期間**: 2-3週  
**spec-kit構造**: `.kiro/specs/rinq-async/`

### 概要

非同期I/O操作をサポートし、Stream traitを活用した非同期データ処理を実現します。

### 主要機能

#### 1. 非同期クエリビルダー
```rust
use rusted_ca::domain::rinq::AsyncQueryBuilder;
use futures::stream::StreamExt;

// 非同期データソース
let stream = fetch_data_from_api().await;

let result: Vec<_> = AsyncQueryBuilder::from_stream(stream)
    .where_(|x| async move { validate(x).await })
    .select(|x| async move { transform(x).await })
    .collect()
    .await;
```

#### 2. 非同期集約
```rust
// async fn count(), first(), last()
let count = AsyncQueryBuilder::from_stream(stream)
    .where_(|x| async move { *x > 0 })
    .count()
    .await;

// async fn any(), all()
let has_valid = AsyncQueryBuilder::from_stream(stream)
    .any(|x| async move { is_valid(x).await })
    .await;
```

#### 3. Stream統合
```rust
use futures::stream::Stream;

// Streamから直接クエリ
let result = AsyncQueryBuilder::from_stream(
    tokio::fs::read_dir("./data")
        .flat_map(|entry| read_json(entry))
)
.where_(|item| async move { item.is_valid() })
.collect()
.await;
```

#### 4. バッチ処理
```rust
// buffer() - バッチ処理
let results = AsyncQueryBuilder::from_stream(api_stream)
    .buffer(100)  // 100件ずつバッチ処理
    .select_async(|batch| async move {
        process_batch(batch).await
    })
    .collect()
    .await;
```

### 技術的考慮事項

**Stream trait**:
```rust
use futures::stream::Stream;

pub struct AsyncQueryBuilder<T, State> {
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
    _state: PhantomData<State>,
}
```

**非同期トレイト境界**:
- `Future + Send + 'static`
- `Stream + Send + 'static`

**バックプレッシャー**:
- buffering戦略
- バッチサイズの最適化

### 期待される成果

- 非同期I/Oとのシームレスな統合
- API、ファイルシステム、データベースストリームの処理
- バックプレッシャーの自動管理

---

## Phase 5: RINQ Join - 結合操作

**優先度**: 高  
**期間**: 2-3週  
**spec-kit構造**: `.kiro/specs/rinq-join/`

### 概要

複数コレクション間の結合と集合演算を提供し、リレーショナルなデータ処理を実現します。

### 主要機能

（詳細は前述のPhase 2を参照）

#### 追加の考慮事項

**結合戦略の選択**:
```rust
// 自動的に最適な結合方法を選択
let result = QueryBuilder::from(large_collection)
    .join_with_strategy(
        small_collection,
        JoinStrategy::HashJoin,  // または NestedLoop
        |l| l.key,
        |r| r.key,
        |l, r| (l, r)
    )
    .collect();
```

**メモリ制約**:
- スピル戦略（メモリ不足時にディスクに退避）
- ストリーミング結合

---

## Phase 6: RINQ Docs - インタラクティブドキュメント 🌟

**優先度**: 高（独創的な価値）  
**期間**: 3-4週  
**spec-kit構造**: `.kiro/specs/rinq-docs/`

### 概要

Axumベースのインタラクティブドキュメントサーバーを構築し、ブラウザ上でRINQを学習・試用できる環境を提供します。

### 主要機能

#### 1. インタラクティブPlayground
```
┌─────────────────────────────────────────────────────┐
│  RINQ Playground                                    │
├──────────────────┬──────────────────────────────────┤
│  Examples ▼      │  Code Editor                     │
│                  │                                  │
│  • Basic Filter  │  use rinq::QueryBuilder;         │
│  • Sort & Take   │                                  │
│  • Group By      │  let data = vec![1,2,3,4,5];    │
│  • Join          │  let result: Vec<_> =           │
│  • Complex       │    QueryBuilder::from(data)     │
│                  │      .where_(|x| *x > 2)        │
│  Datasets ▼      │      .collect();                │
│                  │                                  │
│  • Users (50)    │  [▶ Run] [📋 Copy] [🔗 Share]  │
│  • Products      │                                  │
│  • Orders        ├──────────────────────────────────┤
│                  │  Output                          │
│  Complexity:     │                                  │
│  O(n) 🟢        │  ✓ [3, 4, 5]                    │
│                  │                                  │
│  Memory:         │  ⏱ 0.3ms  💾 48 bytes          │
│  48 bytes 🟢    │  🔍 3 iterations                │
└──────────────────┴──────────────────────────────────┘
```

#### 2. APIリファレンス
```
/api/query-builder
  - メソッド一覧
  - インタラクティブな例
  - 型シグネチャの説明
  - パフォーマンス特性

/api/terminal-operations
  - collect, count, first, last, any, all
  - それぞれライブデモ

/api/queryable
  - サポートされるコレクション型
  - 使用例
```

#### 3. ビジュアライゼーション
```rust
// クエリ実行の可視化
.where_(|x| *x > 2)     → [3,4,5,6,7,8,9,10]  (8 items)
.order_by(|x| -*x)      → [10,9,8,7,6,5,4,3]  (sorted)
.take(3)                → [10,9,8]             (3 items)

// パフォーマンスグラフ
RINQ:   ████░░░░░░ 0.3ms
Manual: ████░░░░░░ 0.3ms
```

#### 4. チュートリアルモード
```
Step 1: Create a QueryBuilder
  ✓ let query = QueryBuilder::from(data);

Step 2: Apply a filter
  → Try adding .where_(|x| *x > 5)
  [Next]

Step 3: Collect the results
  → Try adding .collect()
  [Complete]
```

#### 5. コード実行エンジン

**Option A: Rust Playground API**
```rust
// https://play.rust-lang.org/ のAPIを使用
async fn execute_code(code: &str) -> Result<Output> {
    let request = PlaygroundRequest {
        code,
        edition: "2021",
        mode: "debug",
    };
    // API呼び出し
}
```

**Option B: WebAssembly**
```rust
// RINQをWASMにコンパイル
// ブラウザ上で直接実行
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn execute_query(code: &str) -> String {
    // サンドボックス内で実行
}
```

### 技術スタック

**バックエンド**: Axum
```rust
// docs-server/src/main.rs
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_page))
        .route("/playground", get(playground_page))
        .route("/api/execute", post(execute_code))
        .route("/api/reference/:method", get(method_reference));
    
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**フロントエンド**:
- **Option A**: HTMX + Alpine.js（シンプル、軽量）
- **Option B**: Yew/Leptos（Rust製、フルフィーチャー）

**エディター**: Monaco Editor（VS Codeと同じ）

### ディレクトリ構造
```
docs-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # サーバーエントリポイント
│   ├── handlers/
│   │   ├── playground.rs    # Playground API
│   │   ├── reference.rs     # APIリファレンス
│   │   └── execute.rs       # コード実行
│   ├── templates/           # HTMLテンプレート
│   │   ├── index.html
│   │   ├── playground.html
│   │   └── reference.html
│   └── static/              # CSS, JS
│       ├── css/
│       └── js/
└── examples/                # プリセット例
    ├── basic.rs
    ├── advanced.rs
    └── datasets.json
```

### 期待される成果

- ユーザーがブラウザでRINQを試せる
- ドキュメントの質の向上
- コミュニティの拡大
- フィードバックの収集

---

## Phase 7: RINQ WASM - ブラウザ統合

**優先度**: 中  
**期間**: 2週  
**spec-kit構造**: `.kiro/specs/rinq-wasm/`

### 概要

RINQをWebAssemblyにコンパイルし、ブラウザやNode.js上で直接実行可能にします。

### 主要機能

#### 1. WASM バインディング
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmQueryBuilder {
    inner: QueryBuilder<JsValue, Initial>,
}

#[wasm_bindgen]
impl WasmQueryBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue) -> Self { }
    
    pub fn where_clause(&self, predicate: &js_sys::Function) -> Self { }
    
    pub fn collect(&self) -> JsValue { }
}
```

#### 2. TypeScript型定義
```typescript
// rinq.d.ts
export class QueryBuilder<T> {
  constructor(data: T[]);
  where(predicate: (x: T) => boolean): QueryBuilder<T>;
  select<U>(projection: (x: T) => U): QueryBuilder<U>;
  orderBy<K>(keySelector: (x: T) => K): QueryBuilder<T>;
  collect(): T[];
}
```

#### 3. NPMパッケージ
```json
// package.json
{
  "name": "@rusted-ca/rinq",
  "version": "0.3.0",
  "description": "Type-safe query engine for JavaScript/TypeScript",
  "main": "pkg/rinq.js",
  "types": "pkg/rinq.d.ts"
}
```

### 技術的考慮事項

- `wasm-bindgen`、`wasm-pack`の使用
- JsValueとRust型の相互変換
- パフォーマンス最適化（サイズ削減）

---

## Phase 8: RINQ Compile - コンパイル時最適化

**優先度**: 低（実験的）  
**期間**: 3-4週  
**spec-kit構造**: `.kiro/specs/rinq-compile/`

### 概要

procedural macroを使用したコンパイル時クエリ最適化を実現します。

### 主要機能

#### 1. クエリマクロ
```rust
use rinq::query;

let result = query! {
    from data in users
    where data.age > 18
    select data.name
    order_by data.age
    take 10
};

// コンパイル時に以下のように展開：
let result: Vec<String> = users.into_iter()
    .filter(|data| data.age > 18)
    .map(|data| data.name)
    .collect::<Vec<_>>()
    .sort_by_key(|data| data.age)
    .take(10)
    .collect();
```

#### 2. 型推論の強化
```rust
// 型注釈なしで動作
let result = query! {
    from x in vec![1, 2, 3, 4, 5]
    where x > 2
    select x * 2
};
// 自動的に Vec<i32> と推論
```

### 期待される成果

- ゼロオーバーヘッド
- SQL風のシンタックス
- 完全な型安全性

---

## 実装の推奨順序

### 🎯 推奨ルート（実用性重視）

```
v0.1 ✅
  ↓
v0.2 (集約・変換) ← 最優先
  ↓
RINQ Docs (ドキュメント) ← 同時進行も可
  ↓
RINQ Join (結合操作)
  ↓
RINQ Parallel (並列処理) または RINQ Async (非同期)
  ↓
RINQ WASM (ブラウザ対応)
```

### 🔬 実験ルート（革新性重視）

```
v0.1 ✅
  ↓
RINQ Docs (インタラクティブ) ← コミュニティ構築
  ↓
v0.2 (集約・変換)
  ↓
RINQ WASM (ブラウザ対応) ← Docsと統合
  ↓
RINQ Compile (マクロ)
```

---

## spec-kit での実装手順

各フェーズで以下の3ファイルを作成：

### 1. `requirements.md`
```markdown
# RINQ v0.2 - 集約と変換の拡張

## User Stories

### Story 1: グループ化
AS A データアナリスト
I WANT データをキーでグループ化したい
SO THAT カテゴリごとに集約できる

#### Acceptance Criteria
- WHEN group_by()を呼ぶ THEN HashMap<K, Vec<T>>が返される
- WHEN 空のコレクションをグループ化 THEN 空のHashMapが返される
...
```

### 2. `design.md`
```markdown
# Design Document: RINQ v0.2

## Architecture

### 新しい型状態
- Grouped<K, V>
- Chunked

### QueryData拡張
enum QueryData<T> {
    Iterator(...),
    SortedVec {...},
    Grouped(HashMap<K, Vec<T>>),  // 新規
}

## API Design

### group_by()
...
```

### 3. `tasks.md`
```markdown
# Tasks: RINQ v0.2

- [ ] 1. group_by()の実装
  - [ ] 1.1 HashMap版
  - [ ] 1.2 BTreeMap版
  - [ ] 1.3 プロパティテスト
- [ ] 2. 数値集約
  - [ ] 2.1 sum()
  - [ ] 2.2 average()
  ...
```

---

## 🎨 私の個人的な推奨

**Option A: v0.2 + Docs 並行** ⭐ 最もバランスが良い
- v0.2で実装を進めながら
- Docsで使い方を配信
- 相乗効果が高い

**Option B: Docs 先行** 🚀 コミュニティ重視
- v0.1の価値を最大化
- フィードバックを得てからv0.2を設計
- 独創的

**Option C: v0.2 → Join → Parallel** 💪 機能重視
- 段階的に機能を拡充
- 確実な価値提供

---

## 次のステップ

1. どのフェーズから始めるか決定
2. `.kiro/specs/<phase-name>/` フォルダ作成
3. requirements.md、design.md、tasks.md を作成
4. spec-kitで実装開始

**どのフェーズから始めたいですか？複数並行も可能です！**

特にRINQ Docsは、他のRustプロジェクトにはない独創的な機能になると思います。WebAssemblyでブラウザ上で直接実行できるPlaygroundは、学習体験を革新的にします。