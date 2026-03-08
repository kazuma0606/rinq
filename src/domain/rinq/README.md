# RINQ (Rust Integrated Query) v0.1

RINQは、Rustの型システムとゼロコスト抽象化を活用した、型安全で高性能なクエリエンジンです。C# LINQにインスパイアされ、Rustのイディオムに最適化されています。

## Features

- **型安全**: コンパイル時にクエリの正当性を保証
- **ゼロコスト**: 手書きループと同等のパフォーマンス
- **流暢なAPI**: メソッドチェーンによる読みやすいクエリ記述
- **遅延評価**: 終端操作が呼ばれるまで実際の計算を遅延
- **豊富なコレクションサポート**: Vec, スライス, HashSet, BTreeSet, LinkedList, VecDequeなど

## Quick Start

### 基本的な使い方

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// フィルタリング
let evens: Vec<_> = QueryBuilder::from(data.clone())
    .where_(|x| *x % 2 == 0)
    .collect();
// 結果: [2, 4, 6, 8, 10]

// プロジェクション
let doubled: Vec<_> = QueryBuilder::from(data.clone())
    .select(|x| x * 2)
    .collect();
// 結果: [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

// ソート
let sorted: Vec<_> = QueryBuilder::from(vec![5, 2, 8, 1, 9])
    .order_by(|x| *x)
    .collect();
// 結果: [1, 2, 5, 8, 9]
```

### 複雑なクエリ

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let result: Vec<_> = QueryBuilder::from(data)
    .where_(|x| *x > 2)           // フィルタリング
    .order_by(|x| -*x)             // 降順ソート
    .take(5)                       // 最初の5要素
    .select(|x| x * 2)             // 2倍にする
    .collect();

// 結果: [20, 18, 16, 14, 12]
```

### ページネーション

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = (1..=100).collect::<Vec<_>>();

// ページ3を取得（1ページ10件）
let page_3: Vec<_> = QueryBuilder::from(data)
    .skip(20)  // 最初の20件をスキップ
    .take(10)  // 次の10件を取得
    .collect();

// 結果: [21, 22, 23, ..., 30]
```

### 複数キーソート

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let people = vec![
    ("Alice", 30),
    ("Bob", 25),
    ("Charlie", 30),
    ("David", 25),
];

let sorted: Vec<_> = QueryBuilder::from(people)
    .order_by(|p| p.1)      // 年齢でソート
    .then_by(|p| p.0)       // 次に名前でソート
    .collect();

// 結果: [("Bob", 25), ("David", 25), ("Alice", 30), ("Charlie", 30)]
```

### 終端操作

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5];

// count: 要素数を取得
let count = QueryBuilder::from(data.clone())
    .where_(|x| *x > 2)
    .count();
// 結果: 3

// first: 最初の要素を取得
let first = QueryBuilder::from(data.clone())
    .where_(|x| *x > 2)
    .first();
// 結果: Some(3)

// last: 最後の要素を取得
let last = QueryBuilder::from(data.clone())
    .where_(|x| *x > 2)
    .last();
// 結果: Some(5)

// any: 条件を満たす要素が存在するか
let has_even = QueryBuilder::from(data.clone())
    .any(|x| *x % 2 == 0);
// 結果: true

// all: 全要素が条件を満たすか
let all_positive = QueryBuilder::from(data.clone())
    .all(|x| *x > 0);
// 結果: true
```

### デバッグ

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5];

let result: Vec<_> = QueryBuilder::from(data)
    .where_(|x| *x > 2)
    .inspect(|x| println!("After filter: {}", x))
    .order_by(|x| -*x)
    .inspect(|x| println!("After sort: {}", x))
    .collect();
```

### Queryableトレイト

様々なコレクション型から直接クエリを開始できます：

```rust
use rusted_ca::domain::rinq::query_builder::Queryable;

// Vec
let vec_result: Vec<_> = vec![1, 2, 3, 4, 5]
    .into_query()
    .where_(|x| *x > 2)
    .collect();

// スライス（借用データ）
let data = vec![1, 2, 3, 4, 5];
let slice_result: Vec<_> = data.as_slice()
    .into_query()
    .where_(|x| *x > 2)
    .collect();
// 元のdataは変更されない

// HashSet
use std::collections::HashSet;
let set: HashSet<_> = vec![1, 2, 3, 4, 5].into_iter().collect();
let set_result: Vec<_> = set
    .into_query()
    .where_(|x| *x > 2)
    .order_by(|x| *x)
    .collect();
```

### メトリクス統合

rusted-caのメトリクス収集と統合：

```rust
use std::sync::Arc;
use rusted_ca::domain::rinq::{MetricsQueryBuilder, QueryBuilder};
use rusted_ca::shared::metrics::collector::MetricsCollector;

let metrics = Arc::new(MetricsCollector::new());
let data = vec![1, 2, 3, 4, 5];

let result: Vec<_> = MetricsQueryBuilder::new(
    QueryBuilder::from(data),
    metrics.clone(),
    "my_query".to_string(),
)
.where_(|x| *x > 2)
.collect();

// メトリクスが自動的に記録される
assert_eq!(metrics.get("query_my_query"), Some(1));
```

## API Reference - v0.1

### QueryBuilder Methods

#### フィルタリング

- `where_(predicate)` - 条件を満たす要素のみを抽出

#### ソート

- `order_by(key_selector)` - 昇順でソート
- `order_by_descending(key_selector)` - 降順でソート
- `then_by(key_selector)` - 二次ソートキーを追加（昇順）
- `then_by_descending(key_selector)` - 二次ソートキーを追加（降順）

#### プロジェクション

- `select(projection)` - 各要素を変換

#### ページネーション

- `take(n)` - 最初のn要素を取得
- `skip(n)` - 最初のn要素をスキップ

#### 終端操作

- `collect()` - コレクションに収集
- `count()` - 要素数を取得
- `first()` - 最初の要素を取得（Option）
- `last()` - 最後の要素を取得（Option）
- `any(predicate)` - 条件を満たす要素が存在するか
- `all(predicate)` - 全要素が条件を満たすか

#### デバッグ

- `inspect(f)` - 要素を観察（非破壊的）

---

## v0.2 Features 🎉

RINQ v0.2では、データ分析と変換のための強力な新機能が追加されました。

### 数値集約（Numeric Aggregations）

データの統計的分析を簡単に実行できます。

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5];

// 合計を計算
let total: i32 = QueryBuilder::from(data.clone()).sum();
// 結果: 15

// 平均を計算
let average = QueryBuilder::from(data.clone()).average().unwrap();
// 結果: 3.0

// 最小値と最大値
let min = QueryBuilder::from(data.clone()).min().unwrap();
let max = QueryBuilder::from(data.clone()).max().unwrap();
// 結果: min=1, max=5
```

キーセレクタを使用して、構造体のフィールドで最小/最大を検索：

```rust
#[derive(Debug, Clone)]
struct User { name: String, age: u32 }

let users = vec![
    User { name: "Alice".into(), age: 30 },
    User { name: "Bob".into(), age: 25 },
    User { name: "Charlie".into(), age: 35 },
];

// 最年少のユーザー
let youngest = QueryBuilder::from(users.clone())
    .min_by(|u| u.age)
    .unwrap();
// 結果: Bob (age 25)

// 最年長のユーザー
let oldest = QueryBuilder::from(users)
    .max_by(|u| u.age)
    .unwrap();
// 結果: Charlie (age 35)
```

### グルーピング（Grouping Operations）

データをカテゴリ別に分類し、グループごとに集約できます。

```rust
use rusted_ca::domain::rinq::QueryBuilder;
use std::collections::HashMap;

let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// 偶数/奇数でグループ化
let groups: HashMap<i32, Vec<i32>> = QueryBuilder::from(data.clone())
    .group_by(|x| x % 2);
// 結果: {0: [2,4,6,8,10], 1: [1,3,5,7,9]}

// グループごとに集約を適用
let group_sums: HashMap<i32, i32> = QueryBuilder::from(data)
    .group_by_aggregate(
        |x| x % 3,           // キー: 3で割った余り
        |group| group.iter().sum(),  // 各グループの合計
    );
// 結果: {0: 18, 1: 22, 2: 14}
```

### 重複排除（Deduplication）

データクリーニングワークフローに最適です。

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 2, 3, 3, 3, 4, 5, 5];

// 重複を削除（最初の出現のみ保持）
let unique: Vec<i32> = QueryBuilder::from(data)
    .distinct()
    .collect();
// 結果: [1, 2, 3, 4, 5]

// キーベースの重複排除
#[derive(Debug, Clone, PartialEq)]
struct Product { id: u32, name: String }

let products = vec![
    Product { id: 1, name: "Apple".into() },
    Product { id: 1, name: "Apple v2".into() },  // 同じid
    Product { id: 2, name: "Banana".into() },
];

let unique_products: Vec<Product> = QueryBuilder::from(products)
    .distinct_by(|p| p.id)
    .collect();
// 結果: 2つのProduct（id: 1と2、最初の出現のみ）
```

### シーケンス変換（Sequence Transformations）

バッチ処理や時系列分析に便利な変換操作です。

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![1, 2, 3, 4, 5];

// 逆順に変換
let reversed: Vec<i32> = QueryBuilder::from(data.clone())
    .reverse()
    .collect();
// 結果: [5, 4, 3, 2, 1]

// チャンクに分割（バッチ処理）
let chunks: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7])
    .chunk(3)
    .collect();
// 結果: [[1,2,3], [4,5,6], [7]]

// スライディングウィンドウ（時系列分析）
let windows: Vec<Vec<i32>> = QueryBuilder::from(data)
    .window(3)
    .collect();
// 結果: [[1,2,3], [2,3,4], [3,4,5]]
```

### コレクション結合（Collection Combinations）

データの相関、インデックス付け、分割に使用します。

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let numbers = vec![1, 2, 3, 4, 5];
let letters = vec!['a', 'b', 'c'];

// 2つのコレクションをペアリング（最短の長さに合わせる）
let paired: Vec<(i32, char)> = QueryBuilder::from(numbers.clone())
    .zip(letters.into_iter())
    .collect();
// 結果: [(1,'a'), (2,'b'), (3,'c')]

// インデックスを追加
let indexed: Vec<(usize, i32)> = QueryBuilder::from(numbers.clone())
    .enumerate()
    .collect();
// 結果: [(0,1), (1,2), (2,3), (3,4), (4,5)]

// 条件で2つのコレクションに分割
let (small, large) = QueryBuilder::from(numbers)
    .partition(|x| *x < 3);
// 結果: small=[1,2], large=[3,4,5]
```

### v0.1とv0.2の組み合わせ

全ての操作は互いにシームレスに連携します：

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let data = vec![5, 2, 8, 1, 9, 5, 2, 3];

// 複雑なクエリチェーン
let result: i32 = QueryBuilder::from(data)
    .where_(|x| *x > 2)        // v0.1: フィルタリング
    .distinct()                 // v0.2: 重複排除
    .order_by(|x| *x)          // v0.1: ソート
    .take(3)                    // v0.1: 上位3件
    .sum();                     // v0.2: 合計
// 結果: 5 + 8 + 9 = 22
```

## API Reference - v0.2

### 数値集約メソッド

- `sum()` - 全要素の合計（`T: Sum`が必要）
- `average()` - 全要素の平均（`T: ToPrimitive`が必要、`Option<f64>`を返す）
- `min()` - 最小値（`T: Ord`が必要、`Option<T>`を返す）
- `max()` - 最大値（`T: Ord`が必要、`Option<T>`を返す）
- `min_by(key_selector)` - キー値が最小の要素を返す
- `max_by(key_selector)` - キー値が最大の要素を返す

### グルーピングメソッド

- `group_by(key_selector)` - キー関数でグループ化、`HashMap<K, Vec<T>>`を返す（終端操作）
- `group_by_aggregate(key_selector, aggregator)` - グループ化して各グループに集約関数を適用、`HashMap<K, R>`を返す（終端操作）

### 重複排除メソッド

- `distinct()` - 重複要素を削除（`T: Eq + Hash + Clone`が必要）
- `distinct_by(key_selector)` - キーベースの重複排除（`K: Eq + Hash`が必要）

### シーケンス変換メソッド

- `reverse()` - 要素の順序を逆転
- `chunk(size)` - `size`個ずつのチャンクに分割（`size`が0の場合panic）
- `window(size)` - スライディングウィンドウを作成（`T: Clone`が必要、`size`が0の場合panic）

### コレクション結合メソッド

- `zip(other)` - 別のイテレータとペアリング（最短の長さに合わせる）
- `enumerate()` - インデックスを追加（0から開始）
- `partition(predicate)` - 条件で2つのコレクションに分割、`(Vec<T>, Vec<T>)`を返す（終端操作）

## Performance

RINQは、ゼロコスト抽象化を実現しています：

- すべてのメソッドは`#[inline]`属性で最適化
- Rustの標準イテレータを活用し、コンパイラの最適化を利用
- 遅延評価により、不要な計算を回避
- 手書きループとほぼ同等のパフォーマンス

ベンチマークを実行：

```bash
cargo bench --bench rinq_benchmarks
```

## Error Handling

RINQは、エラーを優雅に処理します：

- `first()`, `last()` - 要素が見つからない場合は`None`を返す（パニックしない）
- `any()`, `all()` - 空のコレクションでも正しい結果を返す
- 型安全により、コンパイル時に多くのエラーを検出

```rust
use rusted_ca::domain::rinq::QueryBuilder;

let empty: Vec<i32> = vec![];

// パニックしない
assert_eq!(QueryBuilder::from(empty.clone()).first(), None);
assert_eq!(QueryBuilder::from(empty.clone()).last(), None);
assert_eq!(QueryBuilder::from(empty.clone()).any(|_| true), false);
assert_eq!(QueryBuilder::from(empty.clone()).all(|_| false), true);
```

## Architecture Integration

RINQは、rusted-caのクリーンアーキテクチャと完全に統合されています：

- **Domain層**: `RinqDomainError`でドメインエラーを表現
- **Application層**: `ApplicationError`に自動変換
- **Metrics**: `MetricsQueryBuilder`でメトリクス収集

## Type State Pattern

RINQは型状態パターンを使用して、コンパイル時にクエリの正当性を保証：

- `Initial` - 初期状態
- `Filtered` - フィルタ後の状態
- `Sorted` - ソート後の状態
- `Projected<U>` - プロジェクション後の状態

無効な操作はコンパイルエラーになります：

```rust
// コンパイルエラー: select()の後にorder_by()は呼べない
// let _ = QueryBuilder::from(vec![1, 2, 3])
//     .select(|x| x * 2)
//     .order_by(|x| *x);
```

## Testing

RINQは、包括的なテストでカバーされています：

- **プロパティベーステスト**: 102個のproptestで様々な入力パターンを検証
- **統合テスト**: メトリクス収集、エラー処理の統合を検証
- **不変性テスト**: 元のコレクションが変更されないことを検証

```bash
# すべてのテストを実行
cargo test

# RINQテストのみ実行
cargo test rinq
```

## License

This project is part of rusted-ca and follows the same license.
