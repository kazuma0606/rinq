# RINQ v5.0 仕様書

> ステータス: 草案
> 対象バージョン: 0.1.0（crates.io 初回公開）
> 前提: v4.0 実装完了済み

---

## 1. 背景と動機

### 1.1 v4 までの達成事項

| フェーズ | 内容 |
|---|---|
| v1 | コア QueryBuilder — where_/order_by/select/collect/sum 等 |
| v2 | 高優先度演算子 — flat_map/take_while/skip_while/contains/single/concat/union 等 |
| v3 | 並列実行 (rayon)・Window 分析・TryQueryBuilder・Serde・rinq-stats (統計/サンプリング/検証) |
| v4 | DX 強化・型エイリアス・rinq-derive・rinq-syntax・functional 演算子 (scan/chunk_by/pairwise 等)・rinq-stats 拡張 (EMA/ボリンジャーバンド/外れ値) |

### 1.2 v4 終了時点の既知ギャップ（監査結果）

#### 構造的問題
- `rinq` クレートが `src/` をリポジトリルートに置いており、他サブクレートとディレクトリ構造が一致していない
- 全クレートのバージョンが統一されていない（rinq/rinq-stats: 4.0.0、derive/syntax: 4.0.0）
- crates.io 公開に向けたバージョン整理が未完（正式版は 0.1.0 から開始）
- `rinq-derive/README.md`、`rinq-syntax/README.md`、`rinq-stats/README.md` が存在しない

#### テスト不足
- `tap_each()`、`tap_collect()`、`pipe()` の専用テストなし
- `cycle()`、`step_by()` のテストなし
- `dedup()`、`dedup_by()`、`pairwise()`、`intersperse()` は v4_tests のみで薄い
- `parallel` + `serde` + `metrics` の組み合わせテストなし

#### ベンチマーク不足
- v4 で追加した全演算子（scan/chunk_by/dedup/zip_with/pairwise/intersperse/min_max/unfold）がベンチマーク対象外
- Window 関数（running_sum/moving_average/rank_by/lag/lead）がベンチマーク対象外
- filter_map/step_by/cycle/tap_each/tap_collect/pipe がベンチマーク対象外

#### ドキュメント
- `CLAUDE.md` が v2 時代の記述のまま（v3/v4 追加演算子・モジュール構造・テストファイルが未記載）
- `src/lib.rs` のクレートトップ docs に v4 演算子への言及なし
- MetricsQueryBuilder/ParallelQueryBuilder/TryQueryBuilder のモジュールレベル docs が薄い

---

## 2. 設計原則

v5 以降は以下の指針に従う。

1. **Zero-overhead — API 追加は既存のゼロコスト保証を崩さない**
2. **型安全 — 新演算子も型ステートパターンに従い、無効なチェーンはコンパイルエラー**
3. **直交性 — 各演算子は他と独立して使えること。JOIN 等の複合機能は専用フェーズで扱う**
4. **段階的公開 — 実験的機能は `#[cfg(feature = "unstable")]` または rinq-syntax の experimental 扱いで提供**
5. **ドキュメントファースト — 新機能は仕様→テスト→実装の順で進める**

---

## 3. フェーズ一覧

| フェーズ | 区分 | 内容 |
|---|---|---|
| **5A** | 整理・品質向上 | ディレクトリ再構成・バージョン統一・README 整備・CLAUDE.md 更新 |
| **5B** | テスト補完 | 未テスト演算子の統合テスト追加・組み合わせテスト |
| **5C** | ベンチマーク拡充 | v4 全演算子のベンチマーク |
| **5D** | ターミナル強化 | for_each/to_sorted_vec/take_last/skip_last/count_by/reduce/all_unique 等 |
| **5E** | クエリ充実 | frequencies/flatten/position/batch/nth 等 |
| **5F** | JOIN 操作 | inner_join/left_join/cross_join・rinq-syntax 拡張 |
| **5G** | 統計拡張 | weighted_average/normalize/standardize/outlier_score/percentile_range 等 |
| **5H** | 公開準備 | 最終ドキュメント・examples 拡充・crates.io dry-run |

---

## 4. Phase 5A — 整理・品質向上

### 5A-1: ディレクトリ再構成

**現状:**
```
rusted-ca/            ← workspace & rinq パッケージが同一
  Cargo.toml          ← [workspace] + [package] が混在
  src/
  tests/
  benches/
  examples/
  rinq-stats/
  rinq-derive/
  rinq-syntax/
```

**目標:**
```
rusted-ca/
  Cargo.toml          ← [workspace] のみ
  rinq/               ← rinq クレート本体
    Cargo.toml
    src/
    tests/
    benches/
    examples/
  rinq-stats/
  rinq-derive/
  rinq-syntax/
```

変更点:
- `git mv src rinq/src` 等で履歴保持移動
- ルート `Cargo.toml` から `[package]` セクションを除去し `members = ["rinq", ...]` に更新
- `rinq/Cargo.toml` 新規作成
- `rinq-stats/Cargo.toml`・`rinq-derive/Cargo.toml`・`rinq-syntax/Cargo.toml` のパス参照を `../rinq` に更新
- `readme` パスを各 Cargo.toml で調整

### 5A-2: バージョン統一

全クレートを **0.1.0** に統一（crates.io 初回公開用）。

```toml
# rinq/Cargo.toml
version = "0.1.0"
# rinq-stats/Cargo.toml
version = "0.1.0"
# rinq-derive/Cargo.toml
version = "0.1.0"
# rinq-syntax/Cargo.toml
version = "0.1.0"
```

`README.md` 内の `rinq = "4"` 等のバージョン参照も `0.1` に更新。

### 5A-3: README 整備

以下の README を新規作成:

**`rinq-derive/README.md`**:
- `#[derive(Queryable)]` / `#[derive(QueryableFrom)]` のクイックスタート
- 属性リスト (`skip` / `rename` / `key`)
- 生成されるコードの概要
- フィールド型別 predicate 一覧表

**`rinq-syntax/README.md`**:
- `query!` マクロの構文リファレンス
- 節ごとの説明と binding semantics
- Experimental ステータスの注記
- 既知の制限（`from` を含む expression は clause 境界と誤認される可能性）

**`rinq-stats/README.md`**:
- インストール方法
- `StatisticsExt` / `SamplingExt` / `ValidationExt` / `TimeSeriesExt` / `OutlierExt` のクイックリファレンス
- `QueryPair` の使用例

### 5A-4: CLAUDE.md 更新

以下を全面刷新:
- モジュール構造図を新ディレクトリ構造に合わせて更新
- 全演算子テーブルに v3/v4 追加分を反映
- テストコマンド一覧をすべてのテストファイル・サブクレートをカバーするよう拡充
- `versions/v3/`・`versions/v4/`・`versions/v5/` のパス説明を追加

---

## 5. Phase 5B — テスト補完

### 5B-1: 未テスト演算子の統合テスト

`tests/rinq_v5_tests.rs` を新規作成し以下をカバー:

| 演算子 | テスト内容 |
|---|---|
| `tap_each(f)` | カウンタ副作用・空コレクション・チェーン中の位置 |
| `tap_collect(f)` | 全収集後の副作用確認・eager 化の検証 |
| `pipe(f)` | 外部関数への委譲・戻り値型の変化 |
| `cycle()` + `take(n)` | 正常ループ・空コレクションからの cycle |
| `step_by(n)` | n=1/2/3・n=0 パニック・大きい n |
| `map()` | select と等価であることの確認 |
| `collect_vec()` | `collect::<Vec<_>>()` との等価性 |

### 5B-2: 組み合わせテスト

```
parallel + serde + metrics の組み合わせ
rinq-derive + v4 演算子（pairwise/scan/zip_with）
rinq-syntax + rinq-derive の統合
大量データ（100万件）での全演算子の正常動作
```

### 5B-3: エッジケース補強

現在のテストで薄い部分:
- `pairwise()` — 0/1/2 要素の境界
- `intersperse()` — 空コレクション・1 要素
- `dedup_by()` — 複合キー・全同値・全異値
- `unfold()` — 無限ループの early termination（take で止める）
- `lag(0)` / `lead(0)` — 境界値

---

## 6. Phase 5C — ベンチマーク拡充

### 5C-1: v4 演算子ベンチマーク

新規ファイル `rinq/benches/rinq_v4_benchmarks.rs`:

```rust
// 各演算子を 1000/10000 要素の Vec<i32> または Vec<f64> で計測
// rinq vs 標準ライブラリ手書きループの対比を基本とする

group: "functional"
  - scan_cumulative_sum (rinq vs Iterator::scan)
  - chunk_by_same_key   (rinq vs slice::windows + manual)
  - dedup_consecutive   (rinq vs Iterator::dedup)
  - zip_with_add        (rinq vs zip().map())
  - pairwise            (rinq vs windows(2))
  - intersperse         (rinq vs Iterator::intersperse)
  - min_max             (rinq vs iter.min()+iter.max() 2パス)
  - filter_map_parse    (rinq vs Iterator::filter_map)
  - step_by_2           (rinq vs Iterator::step_by)

group: "window"
  - running_sum         (rinq vs cumulative fold)
  - moving_average_10   (rinq vs windows(10).map)
  - rank_by             (rinq vs sort + enumerate)
  - lag_1               (rinq vs zip with shifted)
  - lead_1              (rinq vs zip with shifted)

group: "lifecycle"
  - tap_each_noop       (ゼロコスト確認)
  - tap_collect_vec     (eager コスト確認)
  - pipe_identity       (ゼロコスト確認)
  - from_arc_cloned     (Arc clone overhead)

group: "generation"
  - unfold_fib_take100  (unfold_bounded vs recursive)
  - cycle_take1000      (cycle + take vs repeat 手書き)
```

### 5C-2: rinq-stats ベンチマーク

新規ファイル `rinq-stats/benches/rinq_stats_benchmarks.rs`:

```rust
group: "statistics"
  - variance_1000
  - median_1000
  - percentile_95_1000
  - histogram_10buckets_1000

group: "timeseries"
  - ema_alpha02_10000
  - bollinger_window20_10000

group: "outliers"
  - zscore_threshold2_10000
  - iqr_10000

group: "sampling"
  - sample_n_100_from_10000
  - stratified_sample_5groups_10000
```

---

## 7. Phase 5D — ターミナル・コレクション演算子の強化

### 新規演算子一覧

| 演算子 | シグネチャ | 説明 |
|---|---|---|
| `for_each(f)` | `FnMut(T)` | 消費型の副作用ターミナル（tap_each の消費版） |
| `to_sorted_vec(key)` | `Fn(&T) -> K` | order_by + collect のショートハンド |
| `to_sorted_vec_desc(key)` | `Fn(&T) -> K` | order_by_descending + collect |
| `take_last(n)` | `usize` | 末尾 n 件を Vec で返す |
| `skip_last(n)` | `usize` | 末尾 n 件を除いた Vec を返す |
| `count_by(pred)` | `Fn(&T) -> bool` | 条件一致件数を返す |
| `sum_by(key)` | `Fn(T) -> N` | key 関数を通じた合計 |
| `average_by(key)` | `Fn(T) -> f64` | key 関数を通じた平均 |
| `reduce(f)` | `FnMut(T, T) -> T` | aggregate_no_seed の alias |
| `all_unique()` | — | 全要素が重複なしか（`T: Hash + Eq`） |
| `none(pred)` | `Fn(&T) -> bool` | any の否定（`!any(pred)`） |

### 設計メモ

**`for_each`** — `tap_each` との違い: こちらは T を消費し QueryBuilder を返さない。最終ターミナルとして機能。

```rust
// 例
QueryBuilder::from(users)
    .where_(|u| u.active)
    .for_each(|u| println!("{}", u.name));
```

**`take_last(n)` / `skip_last(n)`** — ストリーミング不可（全評価後にスライス）。doc コメントに `⚠ Eagerly collects all elements` を明記。

**`count_by(pred)`** — `where_(pred).count()` のショートハンドだが、中間コレクションを生成しない点でより効率的。

---

## 8. Phase 5E — クエリ充実

### 新規演算子一覧

| 演算子 | シグネチャ | 説明 |
|---|---|---|
| `frequencies()` | — | `HashMap<T, usize>` で出現回数をカウント（`T: Hash + Eq + Clone`） |
| `flatten()` | — | `flat_map(\|x\| x)` の alias（`T: IntoIterator`） |
| `position(pred)` | `Fn(&T) -> bool` | 最初にマッチした要素のインデックス（`Option<usize>`） |
| `nth(n)` | `usize` | n 番目の要素（`element_at` の alias） |
| `batch(n)` | `usize` | chunk の alias（語彙の統一） |
| `find(pred)` | `Fn(&T) -> bool` | first(pred) より自然な名前（first の alias） |
| `index_of(value)` | `&T` | 最初の一致インデックス（`T: PartialEq`） |
| `exactly_one()` | — | single の alias（より直感的な名前） |
| `tee()` | — | 同一ストリームを 2 つの Vec に複製する（LazyStream から Eager Clone） |

### 設計メモ

**`frequencies()`:**
```rust
let freq = QueryBuilder::from(vec!["a", "b", "a", "c", "a", "b"])
    .frequencies();
// {"a": 3, "b": 2, "c": 1}
```

**`flatten()`:**
```rust
let result: Vec<i32> = QueryBuilder::from(vec![vec![1,2], vec![3], vec![4,5]])
    .flatten()
    .collect();
// [1, 2, 3, 4, 5]
```
`flat_map(|x| x)` と等価だが意図が明確。

**`position(pred)` と `find(pred)`** — C# LINQ の `First(pred)`・`FindIndex(pred)` に相当。

---

## 9. Phase 5F — JOIN 操作

### 9.1 概要

v4 rinq-syntax の `from` が 2 回来た場合に「JOIN 非対応」エラーを出していた。v5 で正式対応。

### 9.2 新演算子

```rust
// inner_join: キーが一致するペアのみ
QueryBuilder::from(orders)
    .inner_join(QueryBuilder::from(customers), |o| o.customer_id, |c| c.id)
    // → QueryBuilder<(Order, Customer), Filtered>

// left_join: 左辺の全要素＋右辺の一致（Option）
QueryBuilder::from(orders)
    .left_join(QueryBuilder::from(customers), |o| o.customer_id, |c| c.id)
    // → QueryBuilder<(Order, Option<Customer>), Filtered>

// cross_join: 直積
QueryBuilder::from(xs).cross_join(QueryBuilder::from(ys))
    // → QueryBuilder<(X, Y), Filtered>
```

### 9.3 rinq-syntax 拡張

```
query! {
    from order in orders
    join customer in customers on order.customer_id == customer.id
    where order.total > 100.0
    select (order, customer)
}
```

`join` 節は `inner_join` に展開。`left join` キーワードで `left_join` に展開。

### 9.4 実装上の制約

- JOIN は eager（両辺を Vec に収集後に実行）。O(N×M) または O(N+M)（HashMap 利用時）。
- Generic key 型は `Hash + Eq` を要求。
- 3 way JOIN は v5 スコープ外（2 way JOIN のネストで対応可能）。

---

## 10. Phase 5G — rinq-stats 統計拡張

### 10.1 数値変換

| メソッド | 説明 |
|---|---|
| `normalize()` | Min-Max 正規化 → [0.0, 1.0] の Vec<f64> |
| `standardize()` | Z スコア正規化（mean=0, std_dev=1）の Vec<f64> |
| `weighted_average(weight_fn)` | 重み付き平均 |
| `outlier_scores_zscore()` | 除去ではなくスコア返却（Vec<f64>） |
| `percentile_filter(lo, hi)` | パーセンタイル範囲でフィルタ |
| `cumulative_distribution()` | CDF（累積分布関数）の Vec<f64> |

### 10.2 時系列拡張

| メソッド | 説明 |
|---|---|
| `simple_moving_average(window)` | 単純移動平均（rinq 本体の moving_average と連携） |
| `weighted_moving_average(window)` | 線形加重移動平均 |
| `rate_of_change(period)` | 変化率 `(x[i] - x[i-n]) / x[i-n]` |
| `seasonal_decompose(period)` | 加法モデルによる季節分解（trend + seasonal + residual） |

### 10.3 外れ値検出拡張

| メソッド | 説明 |
|---|---|
| `remove_outliers_modified_zscore(threshold)` | 修正 Z スコア法（外れ値に頑健）|
| `outlier_scores_iqr()` | IQR ベースのスコア返却 |

### 10.4 ValidationExt 拡張

| メソッド | 説明 |
|---|---|
| `validate_range(field, min, max)` | 数値範囲チェックのショートハンド |
| `validate_unique(key_fn)` | コレクション内での一意性チェック |
| `validate_non_empty()` | コレクション自体が空でないことを確認 |
| `report()` | ValidationError 一覧を構造化レポートとして返す |

---

## 11. Phase 5H — 公開準備

### ドキュメント最終確認

- 全 `pub fn` に `///` コメント（missing_docs 警告ゼロ）
- 全演算子に最低 1 件の doc test
- `cargo doc --no-deps --all-features` で警告ゼロ
- `cargo test --doc` で全 doc test 通過

### examples 拡充

| ファイル | 内容 |
|---|---|
| `rinq/examples/basic_usage.rs` | フィルタ・ソート・集計の基本 |
| `rinq/examples/window_analytics.rs` | running_sum/moving_average/rank_by/lag/lead |
| `rinq/examples/functional_ops.rs` | scan/chunk_by/pairwise/unfold |
| `rinq/examples/join_example.rs` | inner_join/left_join の使用例 |
| `rinq/examples/parallel_example.rs` | ParallelQueryBuilder の使用例 |
| `rinq/examples/metrics_example.rs` | MetricsQueryBuilder の使用例 |
| `rinq-stats/examples/statistics.rs` | variance/median/percentile/histogram |
| `rinq-stats/examples/timeseries.rs` | EMA/ボリンジャーバンド |
| `rinq-stats/examples/validation.rs` | validate_if/validate_with/validate_unique |
| `rinq-derive/examples/derive_example.rs` | 既存の rinq_derive_example.rs を移動 |
| `rinq-syntax/examples/syntax_example.rs` | query! マクロの全節使用例 |

### crates.io dry-run

```bash
cargo publish --dry-run -p rinq
cargo publish --dry-run -p rinq-stats
cargo publish --dry-run -p rinq-derive
cargo publish --dry-run -p rinq-syntax
```

すべてエラーゼロを確認してから本番 publish。

---

## 12. 未採用検討案と理由

以下の案は検討したが v5 スコープから除外した。

| 案 | 除外理由 |
|---|---|
| Async 対応 (`async fn collect()`) | tokio/async-std への依存が大きく別クレートが適切。v6 候補 |
| SQL 生成バックエンド | コンセプトが大きく変わる。専用クレートに分離が望ましい |
| Apache Arrow 統合 | データサイエンス向け別クレートとして検討 |
| Isolation Forest 外れ値検出 | 実装コストが高く rinq-stats の範囲を超える。外部クレート連携を検討 |
| `query!` マクロの `let` バインディング | `move` クロージャが必要になり生成コードが複雑化。v6 候補 |
| WASM サポート | ビルドチェーン整備が別途必要。v6 候補 |
| 正規表現フィルタ (`where_regex`) | regex クレートへの依存を避けたい。オプション feature として v6 で検討 |

---

## 13. v5 完了の定義

以下がすべて満たされたとき v5.0 完了とする。

- [ ] Phase 5A: ディレクトリ再構成・バージョン 0.1.0・全 README 存在
- [ ] Phase 5B: `cargo test` 全件通過（未テスト演算子含む）
- [ ] Phase 5C: `cargo bench --no-run` 全件通過（v4 全演算子のベンチマーク）
- [ ] Phase 5D: `for_each`/`to_sorted_vec`/`take_last`/`skip_last`/`count_by`/`reduce`/`all_unique`/`none` 実装
- [ ] Phase 5E: `frequencies`/`flatten`/`position`/`find`/`index_of` 実装
- [ ] Phase 5F: `inner_join`/`left_join`/`cross_join`・rinq-syntax `join` 節 実装
- [ ] Phase 5G: `normalize`/`standardize`/`weighted_average`/`rate_of_change`/`validate_unique` 実装
- [ ] Phase 5H: `cargo publish --dry-run` 全クレートでエラーゼロ・全 examples 動作確認
- [ ] 最終: `cargo clippy --all-features -- -D warnings` ゼロ・`cargo doc` 警告ゼロ
