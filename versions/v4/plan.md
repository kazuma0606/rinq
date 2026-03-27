# RINQ v4.0 実装計画

**作成日**: 2026-03-26

---

## 目標

v3.0 で確立した並列・統計・バリデーション基盤を土台に、**Rust らしい DX の徹底強化**と**他の関数型言語から学んだ演算子の拡充**を行う。v3.0 の公開 API は一切変更しない。

---

## 設計方針

### 変更しないもの
- 既存の全公開 API（`QueryBuilder`, `MetricsQueryBuilder`, `ParallelQueryBuilder`, `rinq-stats` 全トレイト）
- 型ステートパターン（`Initial → Filtered → Sorted / Projected`）
- 既存の依存関係（`thiserror`, `num-traits`, `parking_lot`, `rayon`, `serde_json`）

### 追加するもの
- `rinq` 本体: 型エイリアス、診断属性、新演算子、ライフサイクル改善、マクロ
- `rinq-derive`: 新規クレート — `#[derive(Queryable)]` / `#[derive(QueryableFrom)]`
- `rinq-syntax`: 新規クレート（実験的）— `query!` マクロ
- `rinq-stats`: 時系列演算子・外れ値検出・`ValidationExt` 拡張

### 波及方針（新演算子と他ビルダー）
Phase E・J の新演算子（`scan`, `chunk_by`, `dedup`, `filter_map` 等）は **`QueryBuilder` のみ**が v4.0 の対象。
`MetricsQueryBuilder` / `ParallelQueryBuilder` への追随は **v4.1** で行う。

---

## フェーズ構成

```
Phase 4A: DX 強化（型エイリアス・診断・マクロ）
  ↓
Phase 4B: ライフサイクル設計改善（from_arc_cloned・tap・pipe）
  ↓
Phase 4C: クイックウィン演算子（J1〜J6）
  ↓
Phase 4D: 新演算子（E1〜E8）
  ↓
Phase 4E: rinq-derive クレート（F1・F2）
  ↓
Phase 4F: rinq-syntax クレート（G4 安定 API → G1〜G3 マクロ）
  ↓
Phase 4G: rinq-stats 拡張（I1〜I3）
  ↓
Phase 4H: ドキュメント・CHANGELOG・公開準備
```

各フェーズは `cargo test` 全件通過・`cargo clippy -- -D warnings` ゼロを確認してから完了とする。

---

## Phase 4A: DX 強化

**目的**: 型エラーメッセージの改善・デバッグマクロ・述語糖衣マクロを追加する。

### ファイル構成（変更・追加）

```
src/
  lib.rs                       — 型エイリアス追加、rinq_explain! / pred! re-export
  core/
    state.rs                   — Filtered 意味論のドキュメント更新
    state_diagnostics.rs       — 新規: #[diagnostic::on_unimplemented] トレイト群
  macros/
    mod.rs                     — 新規: rinq_explain! / pred! macro_rules! 定義
```

### D1: 型エイリアス

```rust
// src/lib.rs への追加
pub type InitialQuery<T>      = QueryBuilder<T, Initial>;
pub type FilteredQuery<T>     = QueryBuilder<T, Filtered>;
pub type SortedQuery<T>       = QueryBuilder<T, Sorted>;
pub type ProjectedQuery<T, U> = QueryBuilder<T, Projected<U>>;
```

### D2a: 型ステート制約の診断トレイト

```rust
// src/core/state_diagnostics.rs

// 必要なトレイト（Rust 1.78 stable）
#[diagnostic::on_unimplemented(
    message = "`select()` can only be used in Filtered state",
    label = "this QueryBuilder is in `Initial` state",
    note = "call `.where_()` or `.flat_map()` first"
)]
pub trait SupportsSelect: private::Sealed {}
impl SupportsSelect for Filtered {}

#[diagnostic::on_unimplemented(
    message = "`then_by()` can only be used in Sorted state",
    label = "this QueryBuilder is in `{State}` state",
    note = "call `.order_by()` first"
)]
pub trait SupportsThenBy: private::Sealed {}
impl SupportsThenBy for Sorted {}
```

内部マクロ `define_state_constraint!` でボイラープレートを削減する（Codex 提案）:

```rust
macro_rules! define_state_constraint {
    ($trait:ident, $state:ty, $method:literal, $note:literal) => {
        #[diagnostic::on_unimplemented(
            message = concat!("`", $method, "()` can only be used in ", stringify!($state), " state"),
            note = $note,
        )]
        pub(crate) trait $trait: private::Sealed {}
        impl $trait for $state {}
    };
}
```

対象メソッドと状態（最低限として以下を定義）:

| トレイト | 対象状態 | 対象メソッド | note |
|---|---|---|---|
| `SupportsSelect` | `Filtered` | `select` | "call `.where_()` or `.flat_map()` first" |
| `SupportsThenBy` | `Sorted` | `then_by` / `then_by_descending` | "call `.order_by()` first" |
| `SupportsOrderBy` | `Initial`, `Filtered` | `order_by` | — |

### D2b: 要素型 T のトレイト境界違反の診断

標準トレイト（`Hash + Eq` 等）に `#[diagnostic::on_unimplemented]` を適用する。
独自トレイトの新規実装は不要（標準トレイトに直接適用できる）。

```rust
// distinct() / union() 等: T: Hash + Eq
#[diagnostic::on_unimplemented(
    message = "`distinct()` requires T to implement `Hash + Eq`",
    label = "`{T}` does not implement `Hash` or `Eq`",
    note = "add `#[derive(Hash, PartialEq, Eq)]` to your struct"
)]
pub trait HashEqBound: Hash + Eq {}
impl<T: Hash + Eq> HashEqBound for T {}
```

### D3: `rinq_explain!` マクロ（Option A: 総時間のみ）

```rust
// src/macros/mod.rs
#[macro_export]
macro_rules! rinq_explain {
    ($expr:expr) => {{
        #[cfg(debug_assertions)]
        {
            let __t = std::time::Instant::now();
            let __result = $expr;
            eprintln!("[rinq::explain] {} items, {}ms",
                __result.len(), __t.elapsed().as_millis());
            __result
        }
        #[cfg(not(debug_assertions))]
        { $expr }
    }};
}
```

`release` ビルドでは完全な no-op。`cfg(debug_assertions)` 時のみ動作。

### D4: `pred!` クロージャ糖衣マクロ

```rust
// src/macros/mod.rs
#[macro_export]
macro_rules! pred {
    ($field:ident $op:tt $val:expr) => {
        |__it| __it.$field $op $val
    };
    ($field:ident $op:tt $val:expr && $($rest:tt)*) => {
        |__it| __it.$field $op $val && { let __it = __it; pred!($($rest)*)(__it) }
    };
}
```

---

## Phase 4B: ライフサイクル設計改善

**目的**: `Arc` ソース対応・チェーン中デバッグ・型ステート横断条件分岐を追加する。

### ファイル構成（変更）

```
src/core/builder/
  shared.rs      — tap_each, tap_collect, pipe, from_arc_cloned を追加
```

### H1: `from_arc_cloned` / `from_arc_slice_cloned`

```rust
// src/core/builder/shared.rs または initial.rs
impl<T: Clone + 'static> QueryBuilder<T, Initial> {
    /// Creates a QueryBuilder by cloning all elements from an Arc<Vec<T>>.
    /// This operation is O(N) — use only when the Arc is shared across threads.
    pub fn from_arc_cloned(source: Arc<Vec<T>>) -> Self {
        Self::from((*source).clone())
    }

    /// Creates a QueryBuilder by cloning all elements from an Arc<[T]>.
    /// This operation is O(N).
    pub fn from_arc_slice_cloned(source: Arc<[T]>) -> Self {
        Self::from(source.to_vec())
    }
}
```

### H2: `tap_each` / `tap_collect` / `pipe`

```rust
// src/core/builder/shared.rs — impl<T: 'static, State> QueryBuilder<T, State>

/// Lazily applies a side-effect function to each element without consuming the chain.
pub fn tap_each<F>(self, f: F) -> Self
where F: FnMut(&T) + 'static
{ /* inspect と同等: Box<dyn Iterator> に map で副作用を包む */ }

/// Eagerly collects all elements, applies a side-effect function, then re-wraps.
/// ⚠ Breaks lazy evaluation at this point.
pub fn tap_collect<F>(self, f: F) -> Self
where
    F: FnOnce(&[T]) + 'static,
    T: 'static,
{ /* collect → f(&items) → QueryBuilder::from(items) */ }

/// Passes the builder into a closure, allowing arbitrary transformation.
/// Primary use: conditional filtering or sorting without breaking the type chain.
pub fn pipe<F, T2, S2>(self, f: F) -> QueryBuilder<T2, S2>
where
    F: FnOnce(Self) -> QueryBuilder<T2, S2>,
    T2: 'static,
    S2: 'static,
{ f(self) }
```

---

## Phase 4C: クイックウィン演算子（Phase J）

**目的**: 1〜数行で実装できる高 DX 演算子を一括追加する。

### ファイル構成（変更）

```
src/core/builder/
  shared.rs      — map, collect_vec, step_by, cycle を追加
  filtered.rs    — filter_map を追加（Filtered 状態のみで意味のある操作）
src/lib.rs       — IntoQuery トレイトの定義・blanket impl
```

### J1: `filter_map`

```rust
// すべての状態共通 (shared.rs)
pub fn filter_map<U, F>(self, f: F) -> QueryBuilder<U, Filtered>
where
    F: Fn(T) -> Option<U> + 'static,
    U: 'static,
{ /* self.inner に filter_map アダプタを適用 */ }
```

### J2: `map`（`select` の alias）

```rust
// filtered.rs — select と同じ実装
pub fn map<U, F>(self, f: F) -> QueryBuilder<U, Projected<U>>
where
    F: Fn(T) -> U + 'static,
    U: 'static,
{ self.select(f) }
```

### J3: `IntoQuery` トレイト

```rust
// src/lib.rs
pub trait IntoQuery: Sized {
    type Item: 'static;
    fn into_query(self) -> QueryBuilder<Self::Item, Initial>;
}

impl<T: 'static> IntoQuery for Vec<T> {
    type Item = T;
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}
```

### J4: `collect_vec`

```rust
// shared.rs
pub fn collect_vec(self) -> Vec<T> {
    self.collect::<Vec<T>>()
}
```

### J5: `step_by`

```rust
// shared.rs
pub fn step_by(self, step: usize) -> QueryBuilder<T, Filtered>
{ /* self.into_iter().step_by(step) を Box<dyn Iterator> にラップ */ }
```

`step == 0` の場合 panic（`std::iter::StepBy` の仕様と同様）。

### J6: `cycle`

```rust
// shared.rs
pub fn cycle(self) -> QueryBuilder<T, Filtered>
where T: Clone + 'static
{ /* collect → into_iter().cycle() を Box<dyn Iterator> にラップ */ }
```

⚠ `take` と組み合わせないと無限ループになる。ドキュメントに `# Panics / Infinite loop` を明記。

---

## Phase 4D: 新演算子（Phase E）

**目的**: Haskell / Elixir / Kotlin から着想を得た chainable 演算子を追加する。

### ファイル構成（変更）

```
src/core/builder/
  functional.rs    — 新規: scan, chunk_by, dedup, dedup_by, zip_with,
                           pairwise, intersperse, min_max
  iterators.rs     — UnfoldIter / UnfoldBoundedIter を追加
  shared.rs        — unfold / unfold_bounded を静的メソッドとして追加
```

### E1: `scan`

```rust
// functional.rs — impl<T: 'static, State> QueryBuilder<T, State>
pub fn scan<B, F>(self, seed: B, f: F) -> QueryBuilder<B, Filtered>
where
    B: Clone + 'static,
    F: FnMut(B, T) -> B + 'static,
```

**実装上の注意**: `std::iter::scan` のシグネチャは `FnMut(&mut B, T) -> Option<C>` であり、
仕様の `FnMut(B, T) -> B` とは異なる。変換アダプタを挟む必要がある:

```rust
// 所有権渡しシグネチャへのアダプタ
let mut state = Some(seed);
let mut f = f;
let iter = self.into_iter().map(move |item| {
    let acc = state.take().unwrap();
    let next = f(acc, item);
    state = Some(next.clone());
    next
});
```

### E2: `chunk_by`

```rust
pub fn chunk_by<F, K>(self, key: F) -> QueryBuilder<Vec<T>, Filtered>
where
    F: Fn(&T) -> K + 'static,
    K: PartialEq + 'static,
    T: 'static,
```

**実装方針**: 全要素を走査し、`key` が変化した時点で新しいチャンクを開始する。
`iterators.rs` に `ChunkByIterator` を定義して再利用する。

### E3: `dedup` / `dedup_by`

```rust
pub fn dedup(self) -> QueryBuilder<T, Filtered>
where T: PartialEq + 'static

pub fn dedup_by<K, F>(self, key: F) -> QueryBuilder<T, Filtered>
where
    F: Fn(&T) -> K + 'static,
    K: PartialEq + 'static,
```

**実装方針**: 前の要素のキー値を `Option<K>` として保持し、変化した場合のみ emit する。

### E4: `zip_with`

```rust
pub fn zip_with<U, V, F>(
    self,
    other: impl IntoIterator<Item = U> + 'static,
    f: F,
) -> QueryBuilder<V, Filtered>
where
    F: Fn(T, U) -> V + 'static,
    V: 'static,
    U: 'static,
```

**実装方針**: `self.into_iter().zip(other.into_iter()).map(|(a, b)| f(a, b))`

### E5: `pairwise`

```rust
pub fn pairwise(self) -> QueryBuilder<(T, T), Filtered>
where T: Clone + 'static
```

**実装方針**: `window(2)` の特殊ケースとして実装してもよいが、`(T, T)` 型を直接返すため
専用イテレータの方が効率的。前の要素を `Option<T>` として保持するステートフルアダプタ。

### E6: `unfold` / `unfold_bounded`

```rust
// iterators.rs
struct UnfoldIter<S, T, F> {
    state: Option<S>,
    f: F,
    _phantom: PhantomData<T>,
}

impl<S, T, F: FnMut(S) -> Option<(T, S)>> Iterator for UnfoldIter<S, T, F> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let s = self.state.take()?;
        match (self.f)(s) {
            Some((item, next_s)) => { self.state = Some(next_s); Some(item) }
            None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
}
```

```rust
// shared.rs — 静的メソッドとして Filtered を返す
impl<T: 'static> QueryBuilder<T, Filtered> {
    /// Recommended entry point — generates at most `max` elements.
    pub fn unfold_bounded<S, F>(seed: S, max: usize, f: F) -> Self
    where
        S: 'static,
        F: FnMut(S) -> Option<(T, S)> + 'static,

    /// Advanced: no upper bound. Always combine with `.take(n)`.
    /// In debug builds, emits `log::warn!` after 1,000,000 iterations (configurable).
    pub fn unfold<S, F>(seed: S, f: F) -> Self
    where
        S: 'static,
        F: FnMut(S) -> Option<(T, S)> + 'static,
}
```

`debug_assertions` 時: 内部カウンタで 1,000,000 件を超えた場合に `log::warn!` を出す（Gemini 提案）。

### E7: `intersperse`

```rust
pub fn intersperse(self, sep: T) -> QueryBuilder<T, Filtered>
where T: Clone + 'static
```

**実装方針**: 要素間に `sep` を挿入するステートフルアダプタ。先頭要素の前には挿入しない。

### E8: `min_max`

```rust
pub fn min_max(self) -> Option<(T, T)>
where T: Ord + Clone + 'static
```

**実装方針**: 単一走査で min / max を同時に追跡。`(T, T)` の初期値は `first().clone()`。

---

## Phase 4E: `rinq-derive` クレート（新規）

**目的**: `#[derive(Queryable)]` でフィールドアクセサと型付き述語を自動生成する。

### ファイル構成（新規）

```
rinq-derive/
  Cargo.toml      — [lib] proc-macro = true
  src/
    lib.rs        — derive(Queryable) / derive(QueryableFrom) のエントリポイント
    queryable.rs  — #[derive(Queryable)] の展開ロジック
    from.rs       — #[derive(QueryableFrom)] の展開ロジック
```

### `rinq-derive/Cargo.toml`

```toml
[package]
name    = "rinq-derive"
version = "4.0.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn     = { version = "2", features = ["full"] }
quote   = "1"
proc-macro2 = "1"
```

### F1: `#[derive(Queryable)]`

**生成物 1: フィールドアクセサ関数**

```rust
// マクロが生成（impl User ブロックへの追加）
pub fn by_name(u: &User)       -> &str  { &u.name }
pub fn by_age(u: &User)        -> u32   { u.age }
pub fn by_active(u: &User)     -> bool  { u.active }
pub fn by_department(u: &User) -> &str  { &u.department }
```

**生成物 2: 型付き述語構造体**

```rust
pub mod user_fields {
    pub struct Age;
    impl Age {
        pub fn gt(n: u32)                -> impl Fn(&User) -> bool { move |u| u.age > n }
        pub fn lt(n: u32)                -> impl Fn(&User) -> bool { move |u| u.age < n }
        pub fn eq(n: u32)                -> impl Fn(&User) -> bool { move |u| u.age == n }
        pub fn between(lo: u32, hi: u32) -> impl Fn(&User) -> bool {
            move |u| u.age >= lo && u.age <= hi
        }
    }
    // ... Name, Active, Department 等
}
```

**Codex 指摘への対応**: `order_by` 向けアクセサ（`&str` 返し）と `group_by` 向けアクセサ
（所有キー返し）は生成パターンを分ける。`by_*` は `&T` 参照版、`owned_by_*` は所有版とする
（命名は実装フェーズで決定）。

**属性によるカスタマイズ**:

| 属性 | 効果 |
|---|---|
| `#[queryable(skip)]` | このフィールドのアクセサを生成しない |
| `#[queryable(rename = "name")]` | 生成される `by_*` 関数名を上書き |
| `#[queryable(key)]` | デフォルト sort/group キーとしてマーク |

**Macro hygiene**: `proc-macro2` の `Span::mixed_site()` を使い、展開変数がユーザーコードと衝突しないようにする（Gemini G4 指摘）。

### F2: `#[derive(QueryableFrom)]`

```rust
// 生成物
impl From<UserList> for QueryBuilder<User, Initial> {
    fn from(list: UserList) -> Self {
        QueryBuilder::from(list.0)
    }
}
```

`IntoQuery` トレイトと組み合わせることで `UserList(users).into_query()` の記法が使える。

---

## Phase 4F: `rinq-syntax` クレート（新規・実験的）

**目的**: `query!` マクロで C# LINQ に近い記法を提供する。

### ファイル構成（新規）

```
rinq/src/
  query_api.rs    — __macro_support モジュール（マクロ専用安定 API）

rinq-syntax/
  Cargo.toml
  src/
    lib.rs        — query! マクロのエントリポイント
    parser.rs     — from/where/order_by/select 節のパーサー
    codegen.rs    — __macro_support 呼び出しへの変換
```

### G4: `rinq::__macro_support`（先に rinq 本体側に追加）

```rust
// src/query_api.rs
#[doc(hidden)]
pub mod __macro_support {
    use crate::{QueryBuilder, Filtered, Sorted, Initial};

    pub fn from<T: 'static>(source: Vec<T>) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(source)
    }
    // where_, order_by, select, collect も同様に定義
}
```

`rinq-syntax` はこの安定インターフェースのみを呼び出し、内部実装には依存しない。
バージョン互換性方針: `__macro_support` の変更には `#[deprecated]` 移行期間を設ける。

### G1〜G3: `query!` マクロ

**v4.0 でサポートする節**:

| 節 | 展開後 |
|---|---|
| `from x in source` | `__macro_support::from(source)` |
| `where predicate` | `.__where_(|x| predicate)` |
| `order_by key` | `.__order_by(|x| key)` |
| `order_by_desc key` | `.__order_by_descending(|x| key)` |
| `select expr` | `.__select(|x| expr).__collect()` |
| `take n` | `.__take(n)` |
| `skip n` | `.__skip(n)` |
| `let name = expr` | クロージャ内 let バインディングに展開 |

**エラーメッセージ**: `proc_macro::Span` を用いて展開前のソース位置を指すようにする。

---

## Phase 4G: `rinq-stats` 拡張

**目的**: 時系列・外れ値・`ValidationExt` を拡張する。

### ファイル構成（変更・追加）

```
rinq-stats/src/
  timeseries.rs    — 新規: exponential_moving_average, bollinger_bands
  outliers.rs      — 新規: remove_outliers_zscore, remove_outliers_iqr
  validation.rs    — 既存: validate_if, validate_with を追加
```

### I1: 時系列演算子

```rust
// timeseries.rs
pub trait TimeSeriesExt<T>: Sized {
    fn exponential_moving_average(self, alpha: f64) -> QueryBuilder<f64, Filtered>
    where T: Into<f64> + 'static;

    fn bollinger_bands(self, window: usize, sigma: f64)
        -> QueryBuilder<(f64, f64, f64), Filtered>
    where T: Into<f64> + 'static;
}

impl<T: 'static, S: TypeState> TimeSeriesExt<T> for QueryBuilder<T, S> { ... }
```

### I2: 外れ値検出

```rust
// outliers.rs
pub trait OutlierExt<T>: Sized {
    fn remove_outliers_zscore(self, threshold: f64) -> QueryBuilder<T, Filtered>
    where T: Into<f64> + Clone + 'static;

    fn remove_outliers_iqr(self) -> QueryBuilder<T, Filtered>
    where T: Into<f64> + Clone + 'static;
}
```

### I3: `ValidationExt` 拡張

```rust
// validation.rs への追加メソッド
// 依存条件付き検証
fn validate_if<P, F>(self, condition: P, rule: F, rule_name: &str, message: &str)
    -> ValidationQueryBuilder<T>
where
    P: Fn(&T) -> bool + 'static,
    F: Fn(&T) -> bool + 'static;

// カスタムエラー型サポート
fn validate_with<E, F>(self, f: F) -> ValidationQueryBuilder<T>
where
    F: Fn(&T) -> Result<(), E> + 'static,
    E: std::fmt::Display + 'static;
```

---

## Phase 4H: ドキュメント・公開準備

**目的**: 全クレートの docs.rs 整備と crates.io 公開に向けたメタデータ整備。

### 作業

- `state.rs` の `Filtered` 意味論説明を英語コメントで更新
- 全新規公開 API に英語 `///` コメント（要約・`# Examples`・`# Panics`）
- `Cargo.toml`（`rinq-derive` / `rinq-syntax`）のメタデータ追加
- `README.md` に v4 の新機能セクション追加（型エイリアス・新演算子・`rinq-derive`）
- `CHANGELOG.md` に v4.0 エントリを追加
- `cargo publish --dry-run` 全クレートで通過確認

### リリース判断基準

- `cargo test` 全件通過（`rinq` + `rinq-stats` + `rinq-derive`）
- `cargo test --all-features` 全件通過
- `cargo test --doc` 全件通過
- `cargo doc --no-deps --all-features` エラーなし
- `cargo clippy --all-features -- -D warnings` ゼロ
- `cargo bench --no-run` 通過

---

## リスク・注意事項

### `scan` の `FnMut` アダプタ

`std::iter::scan` は `FnMut(&mut B, T) -> Option<C>` で、仕様の `FnMut(B, T) -> B` とは異なる。
`Option<B>` で状態を包み `take()` で所有権を受け渡す変換アダプタが必要。実装後に単体テストで
`B` の drop タイミングを確認すること。

### `unfold` の無限ループ対策

`unfold` は終了条件がない場合に `collect()` がハングする。ドキュメントに明記し、
`unfold_bounded` を推奨エントリポイントとして先頭に掲載する。
`debug_assertions` 時のランタイム警告は `log` クレートを使用する。

### `rinq-derive` の proc-macro Hygiene

`proc-macro2` の `Span::mixed_site()` を用いて、生成変数名がユーザーコードと衝突しないよう
厳密にテストする（`user`, `__it` 等の変数名衝突ケースを明示的にテストする）。

### `rinq-syntax` の実験的ステータス

`rinq-syntax` は `Cargo.toml` に `# [experimental]` を明記し、semver 保護を `0.4.x` 扱いとする。
v4.0 では `query!` の基本形（単一 `from`）のみをサポートし、JOIN / `group by` は v4.1 以降。

### `group_by` 向けアクセサのライフタイム問題

`by_name(u: &User) -> &str` は `order_by` では動作するが、`group_by` は `HashMap<K, Vec<T>>`
に所有キーを格納するため `&str` では不可。`owned_by_name` や別途所有版アクセサを生成する。
実装フェーズで命名規則を決定し、spec.md と同期させること。

### `tap_collect` の eager 化への注意

`tap_collect` はチェーン中に全要素を収集するため、遅延評価が壊れる。
ドキュメントに `⚠ Eagerly collects all elements` を明記し、`tap_each` との使い分けを示す。

### v3.0 からの型ステート制約の引き継ぎ

以下の制約は v4 でも変わらず適用される（新演算子の doc test 作成時に遵守すること）:

- `Projected<U>` 状態では `collect` 以外の操作は使えない
- `Initial` 状態に `select` / `map` は存在しない
- `QueryBuilder::empty()` にターボフィッシュは使えない
- `unfold` / `cycle` は `take` なしで `collect` するとハング
