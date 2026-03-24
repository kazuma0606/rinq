# RINQ v1.0 仕様書

## 概要

RINQ (Rust Integrated Query) は、Rust の型システムを活用した型安全なクエリエンジンです。
C# の LINQ に着想を得た流暢な API を提供し、コンパイル時検証とゼロコスト抽象化を両立します。

---

## プロジェクト構造

### v1.0 のディレクトリ構成

```
rinq/
├── Cargo.toml
├── CHANGELOG.md
├── src/
│   ├── lib.rs               # 公開 API（トップレベル re-export）
│   ├── core/                # rinq::core::* — 純粋なクエリエンジン
│   │   ├── mod.rs
│   │   ├── builder.rs       # QueryBuilder 本体
│   │   ├── state.rs         # 型状態パターン定義
│   │   └── error.rs         # RinqError, RinqResult
│   └── metrics/             # rinq::metrics::* — メトリクス統合（core に非依存）
│       ├── mod.rs
│       ├── builder.rs       # MetricsQueryBuilder
│       └── collector.rs     # MetricsCollector
├── tests/
│   ├── core_tests.rs        # QueryBuilder 単体テスト・プロパティテスト
│   ├── property_tests.rs    # 型状態パターン検証プロパティテスト
│   ├── v0_2_tests.rs        # v0.2 機能テスト（集計・グループ化等）
│   ├── immutability_tests.rs
│   └── metrics_tests.rs     # MetricsQueryBuilder 統合テスト
├── benches/
│   ├── core_benchmarks.rs   # フィルタ・変換のゼロコスト検証
│   └── v0_2_benchmarks.rs   # 集計・グループ化のパフォーマンス検証
├── examples/
│   └── basic_usage.rs
└── versions/
    └── v1/
        └── spec.md          # 本文書
```

### 変更の背景（v0.2 → v1.0）

v0.2 まで RINQ は `rusted-ca`（Clean Architecture サンプル Web アプリ）の内部実装として
`src/domain/rinq/` に埋め込まれていた。v1.0 ではプロジェクトのルートを RINQ クレート自体とし、
Web アプリ固有のコードをすべて削除して独立したライブラリクレートとして整備した。

---

## 公開 API

### クレート名とインポート

```rust
// v0.2 まで（rusted-ca 埋め込み時）
use rusted_ca::domain::rinq::QueryBuilder;

// v1.0 から（独立クレート）
use rinq::QueryBuilder;
use rinq::core::builder::QueryBuilder;  // 明示的なパス
use rinq::core::*;                       // 名前空間インポート
```

### トップレベル re-export 一覧

| 識別子 | 説明 |
|--------|------|
| `rinq::QueryBuilder` | クエリビルダー本体 |
| `rinq::Queryable` | コレクションをクエリに変換するトレイト |
| `rinq::RinqError` | エラー型 |
| `rinq::RinqResult<T>` | `Result<T, RinqError>` エイリアス |
| `rinq::Filtered` | 型状態: フィルタ済み |
| `rinq::Initial` | 型状態: 初期 |
| `rinq::Projected<U>` | 型状態: 射影済み |
| `rinq::Sorted` | 型状態: ソート済み |
| `rinq::MetricsQueryBuilder` | メトリクス付きクエリビルダー |
| `rinq::MetricsCollector` | メトリクス収集器 |

---

## コアモジュール仕様（`rinq::core`）

### 型状態パターン（`core::state`）

クエリ構築の有効なシーケンスをコンパイル時に強制する型レベルの状態機械。

| 状態 | 型 | 遷移元 |
|------|----|--------|
| Initial | `Initial` | `QueryBuilder::from()` |
| Filtered | `Filtered` | `where_()`, `take()`, `skip()`, `distinct()`, `reverse()`, `chunk()`, `window()`, `zip()`, `enumerate()` |
| Sorted | `Sorted` | `order_by()`, `order_by_descending()` |
| Projected | `Projected<U>` | `select()` |

**設計方針**: 状態は `PhantomData` として保持し、実行時コストはゼロ。

### エラー型（`core::error`）

```rust
pub enum RinqError {
    InvalidQuery { message: String },
    IteratorExhausted,
    ExecutionError { message: String },
    InvalidState { message: String },
    TypeMismatch { expected: String, actual: String },
}

pub type RinqResult<T> = Result<T, RinqError>;
```

**v0.2 との変更点**: `RinqDomainError` → `RinqError`（"Domain" プレフィックスを削除）。
standalone クレートとしての命名に統一した。

### `QueryBuilder<T, State>`（`core::builder`）

#### 内部構造

```rust
enum QueryData<T> {
    Iterator(Box<dyn Iterator<Item = T>>),
    SortedVec {
        items: Vec<T>,
        comparator: Box<dyn Fn(&T, &T) -> Ordering>,
    },
}
```

- `Iterator` 変形: フィルタ・変換・ページングなど遅延評価
- `SortedVec` 変形: ソート後の状態で O(1) min/max を実現

#### `Initial` 状態のメソッド

| メソッド | 戻り値の状態 | 説明 |
|----------|-------------|------|
| `from(source)` | `Initial` | コレクションからビルダーを生成 |
| `where_(predicate)` | `Filtered` | フィルタリング |
| `order_by(key)` | `Sorted` | 昇順ソート |
| `order_by_descending(key)` | `Sorted` | 降順ソート |
| `take(n)` | `Filtered` | 先頭 n 件 |
| `skip(n)` | `Filtered` | 先頭 n 件スキップ |
| `inspect(f)` | `Initial` | 副作用なし観察 |
| `sum()` | *(終端)* | 合計 |
| `average()` | *(終端)* | 平均（`Option<f64>`） |
| `min()` | *(終端)* | 最小値 |
| `max()` | *(終端)* | 最大値 |
| `min_by(key)` | *(終端)* | キーで最小の要素 |
| `max_by(key)` | *(終端)* | キーで最大の要素 |
| `group_by(key)` | *(終端)* | `HashMap<K, Vec<T>>` |
| `group_by_aggregate(key, agg)` | *(終端)* | `HashMap<K, R>` |
| `distinct()` | `Filtered` | 重複除去 |
| `distinct_by(key)` | `Filtered` | キーによる重複除去 |
| `reverse()` | `Filtered` | 逆順 |
| `chunk(n)` | `Filtered` | 固定サイズチャンク |
| `window(n)` | `Filtered` | スライディングウィンドウ |
| `zip(other)` | `Filtered` | 別イテレータとペアリング |
| `enumerate()` | `Filtered` | インデックス付与 |
| `partition(predicate)` | *(終端)* | `(Vec<T>, Vec<T>)` |

#### `Filtered` 状態のメソッド

`Initial` 状態と同じメソッドセットを持つ（`select()` が追加）。

| 追加メソッド | 戻り値の状態 | 説明 |
|-------------|-------------|------|
| `select(projection)` | `Projected<U>` | 型変換・射影 |

#### `Sorted` 状態のメソッド

| メソッド | 戻り値の状態 | 説明 |
|----------|-------------|------|
| `then_by(key)` | `Sorted` | 第 2 ソートキー（昇順） |
| `then_by_descending(key)` | `Sorted` | 第 2 ソートキー（降順） |
| `take(n)` | `Filtered` | 先頭 n 件 |
| `skip(n)` | `Filtered` | 先頭 n 件スキップ |
| `inspect(f)` | `Filtered` | 副作用なし観察 |
| `min()` | *(終端)* | O(1) 最小値（ソート済み最適化） |
| `max()` | *(終端)* | O(1) 最大値（ソート済み最適化） |
| その他集計・変換 | — | `Initial` / `Filtered` と同等 |

#### 全状態の終端操作

| メソッド | 説明 |
|----------|------|
| `collect::<B>()` | `B: FromIterator<T>` に収集 |
| `count()` | 要素数 |
| `first()` | 先頭要素 `Option<T>` |
| `last()` | 末尾要素 `Option<T>` |
| `any(predicate)` | いずれかが条件を満たすか |
| `all(predicate)` | すべてが条件を満たすか |

#### `Queryable` トレイト

各コレクション型への `into_query()` 実装を提供する。

```rust
pub trait Queryable<T> {
    fn into_query(self) -> QueryBuilder<T, Initial>;
}
```

実装済み: `Vec<T>`, `&[T]`, `[T; N]`, `HashSet<T>`, `BTreeSet<T>`, `LinkedList<T>`, `VecDeque<T>`

---

## メトリクスモジュール仕様（`rinq::metrics`）

### 設計方針

- `core` モジュールへの依存はあるが、`core` は `metrics` を知らない
- メトリクスが不要なユーザーには `MetricsCollector` のコストが一切かからない
- `MetricsQueryBuilder` は `QueryBuilder` の薄いラッパーであり、機能の差分はない

### `MetricsCollector`（`metrics::collector`）

```rust
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, u64>>>,
}
```

| メソッド | 説明 |
|----------|------|
| `new()` | 新規コレクター生成 |
| `increment(key)` | カウンタをインクリメント |
| `get(key)` | カウンタ値取得 |
| `record_query_execution(name, duration)` | クエリ実行を記録（`query_{name}` キー） |

スレッドセーフ（`parking_lot::RwLock` 使用）。

### `MetricsQueryBuilder<T, State>`（`metrics::builder`）

`QueryBuilder<T, State>` と完全に同じメソッドセットを提供する。各終端操作と一部の中間操作で `MetricsCollector::record_query_execution()` を呼び出す。

メトリクスキーの命名規則:
- `collect()` → `query_{operation_name}`
- `count()` → `query_{operation_name}_count`
- `first()` → `query_{operation_name}_first`
- `sum()` → `query_{operation_name}_sum`
- 等

---

## 依存関係

### 本番依存

| クレート | バージョン | 用途 |
|----------|-----------|------|
| `thiserror` | 1.0 | `RinqError` の derive マクロ |
| `num-traits` | 0.2 | `average()` の `ToPrimitive` 変換 |
| `parking_lot` | 0.12 | `MetricsCollector` のスレッドセーフな RwLock |

### 開発依存

| クレート | バージョン | 用途 |
|----------|-----------|------|
| `proptest` | 1.0 | プロパティベーステスト |
| `criterion` | 0.5 | ベンチマーク |

---

## テスト方針

| ファイル | 種別 | 内容 |
|----------|------|------|
| `tests/core_tests.rs` | 検証テスト | `QueryBuilder` の全操作を検証。`proptest` による不変性・フィルタ正確性の検証を含む。 |
| `tests/property_tests.rs` | プロパティテスト | 型状態パターンが有効なクエリ構築を強制することを検証（コンパイル時保証のランタイム確認）。 |
| `tests/v0_2_tests.rs` | プロパティテスト | v0.2 追加機能（集計・グループ化・重複除去・変換）の正確性を手動計算と比較検証。 |
| `tests/immutability_tests.rs` | プロパティテスト | クエリ実行が元のコレクションを変更しないことを検証。 |
| `tests/metrics_tests.rs` | 統合テスト | `MetricsQueryBuilder` とメトリクス収集の動作を検証。 |

---

## パフォーマンス特性

### ゼロコスト抽象化

- 全メソッドに `#[inline]` 属性を付与
- 型状態は `PhantomData` のみで実行時オーバーヘッドなし
- イテレータアダプタによる遅延評価（終端操作まで計算しない）

### ベンチマーク目標（criterion による検証）

| 操作 | 目標オーバーヘッド |
|------|------------------|
| filter | 手書きループと同等 |
| filter + map | 手書きループと同等 |
| sum / average | ≤5% |
| group_by | ≤10%（HashMap コスト） |
| distinct | 手書き HashSet と同等 |

### `Sorted` 状態での最適化

`order_by()` 後の `min()` / `max()` は O(1)（ソート済みベクタの先頭/末尾アクセス）。

---

## v0.2 からの変更点

| 項目 | v0.2 | v1.0 |
|------|------|------|
| クレート名 | `rusted-ca` | `rinq` |
| インポートパス | `rusted_ca::domain::rinq::*` | `rinq::*` |
| エラー型名 | `RinqDomainError` | `RinqError` |
| コアモジュール | `domain::rinq` | `core` |
| メトリクスモジュール | `shared::metrics` | `metrics` |
| 削除されたコード | Web アプリ全体（presentation, application, infrastructure, domain entities） | — |
| RINQ の機能 | 変更なし | 変更なし（後方非互換は命名のみ） |

---

## 使用例

```rust
use rinq::QueryBuilder;

// フィルタリングと変換
let result: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    .where_(|x| x % 2 == 0)
    .select(|x| x * 2)
    .collect();
// [4, 8, 12, 16, 20]

// ソートと集計
let avg = QueryBuilder::from(vec![5, 2, 8, 1, 9, 3])
    .where_(|x| *x > 2)
    .order_by(|x| *x)
    .average();
// Some(5.4)

// グループ化
use std::collections::HashMap;
let groups: HashMap<i32, Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
    .group_by(|x| x % 2);
// {0: [2, 4, 6], 1: [1, 3, 5]}

// メトリクス付きクエリ
use rinq::{MetricsQueryBuilder, MetricsCollector};
use std::sync::Arc;

let metrics = Arc::new(MetricsCollector::new());
let result: Vec<i32> = MetricsQueryBuilder::new(
    QueryBuilder::from(vec![1, 2, 3, 4, 5]),
    metrics.clone(),
    "my_query".to_string(),
)
.where_(|x| *x > 2)
.collect();
// [3, 4, 5]

assert_eq!(metrics.get("query_my_query"), Some(1));
```
