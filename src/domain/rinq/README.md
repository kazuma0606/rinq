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

## API Reference

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
