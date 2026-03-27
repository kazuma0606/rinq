# RINQ v4.0 仕様書

**作成日**: 2026-03-25
**ステータス**: Draft

### 改訂履歴

| 日付 | 改訂内容 | 出典 |
|---|---|---|
| 2026-03-25 | 初版作成 | — |
| 2026-03-25 | D2 拡充 / D4+F1 シナジー明記 / E6 安全性 / H2 重要性 / G クレート分離戦略 | Gemini レビュー |
| 2026-03-26 | 技術的バグ修正・設計変更・拡張案追加（下記参照） | Codex・Gemini・Claude・作成者レビュー |

**2026-03-26 の主な修正点（レビュー反映）**:
- `E1 scan`: クロージャ型 `Fn` → `FnMut`（Fn では B を消費できない）
- `E6 unfold`: クロージャ型 `Fn` → `FnMut`（同上）、`unfold_bounded` を v4.0 に前倒し
- `D1`: `InitialQuery<T>` 型エイリアスの追加漏れを修正
- `D2`: `SummableElement` の二重管理問題を解消、`D2a` / `D2b` に分割
- `D3 rinq_explain!`: 遅延評価との矛盾を明記し設計選択肢を整理
- `H1 from_arc`: 名称変更（O(N) コピーの誤解防止）
- `H2 tap`: `tap_each`（lazy）/ `tap_collect`（eager）の 2 バリアントに分割
- `F1`: `Age.gt(18)` → `Age::gt(18)`（Rust 慣習との整合）
- 拡張案追加: `filter_map`、`map` alias、`IntoQuery` トレイト、`collect_vec`、`step_by`、`cycle`
- `Filtered` 状態の意味論を明文化
- 新演算子の `MetricsQueryBuilder` / `ParallelQueryBuilder` 波及方針を追記
- 国際化（英語化）方針を追記

---

## 概要

RINQ v4.0 は v3.0 で確立した並列・統計・バリデーション基盤を土台に、**Rust らしい DX（Developer Experience）の徹底強化**と**他の関数型言語から学んだ演算子の拡充**を軸とするリリースです。

### 3 本柱

1. **DX 強化** — 型エラーの改善、derive マクロ、デバッグツール
2. **演算子拡充** — Haskell / Elixir / Kotlin にある chainable 演算子の取り込み
3. **クレートエコシステム整備** — `rinq-derive`、`rinq-syntax` の段階的追加

### スコープ外（v4 では対象外）

- SQL 統合・DB 操作（別プロジェクト `oxide` の責務）
- 非同期イテレータ（`async-std` / `tokio` Stream との統合）— v5 候補
- WASM ターゲット — v5 候補
- `no_std` 対応 — 別クレート `rinq-nostd` として切り出す方向で検討中

---

## v3.0 からの位置づけ

```
v1.0  コアエンジン確立（QueryBuilder, 型ステートパターン, Queryable）
  ↓
v2.0  LINQ 差分の補完（flat_map, aggregate, 集合演算, 生成演算子 等）
  ↓
v3.0  Rust 独自拡張（並列, ウィンドウ関数, serde, rinq-stats 等）
  ↓
v4.0  DX 強化 + 関数型演算子拡充 + マクロエコシステム  ← 本文書
```

v4.0 は **破壊的変更なし** を原則とします。すべての新機能は新規メソッド・新規クレートの追加であり、v3.0 のコードはそのままビルドできます。

---

## クレート構成

```
rinq              コアクエリエンジン（v4 新規演算子 + 型エイリアス + DX 改善）
                    feature flags:
                      parallel   rayon による並列処理（v3 継続）
                      serde      JSON / serde_json 統合（v3 継続）
                      diagnostics  詳細エラーメッセージ（新規）

rinq-stats        統計演算・相関・サンプリング・バリデーション（v3 継続）

rinq-derive       derive マクロ（新規）
                    #[derive(Queryable)]  フィールドアクセサ自動生成

rinq-syntax       proc-macro クエリ構文（新規、実験的）
                    query! { from u in users where u.age > 18 select u.name }
```

---

## Phase D: DX 強化（`rinq` 本体）

### D1: 型エイリアス

型エラーメッセージ中の `QueryBuilder<T, rinq::core::state::Filtered>` を短縮する。

> **修正（Claude レビュー）**: 初版では `Initial` 状態のエイリアスが欠落していた。
> `unfold` / `range` / `repeat` の戻り値を関数シグネチャで表現できなかったため追加。

```rust
// src/lib.rs への追加
pub type InitialQuery<T>         = QueryBuilder<T, Initial>;   // ← 追加
pub type FilteredQuery<T>        = QueryBuilder<T, Filtered>;
pub type SortedQuery<T>          = QueryBuilder<T, Sorted>;
pub type ProjectedQuery<T, U>    = QueryBuilder<T, Projected<U>>;
```

**効果**:
- 関数シグネチャが読みやすくなる
- エラーメッセージに `FilteredQuery<User>` と表示される

```rust
// 型エイリアスを使った関数シグネチャ
fn get_adults(users: Vec<User>) -> FilteredQuery<User> {
    QueryBuilder::from(users).where_(|u| u.age > 18)
}

// InitialQuery: 生成演算子の戻り値を明示できる
fn make_range(n: i32) -> InitialQuery<i32> {
    QueryBuilder::range(0..n)
}

---

### D2: `#[diagnostic::on_unimplemented]` による型ステートエラー改善

Rust 1.78 stable で使える `#[diagnostic::on_unimplemented]` を型ステートトレイトに適用する。

> **Gemini フィードバック**: `State` だけでなく `T` のトレイト境界にも適用すると更に親切になる。
>
> **修正（Claude・Codex レビュー）**: 初版では `SummableElement` という独自トレイトを定義して
> `i32` / `i64` / `f64` に手動で impl していたが、現行の `sum()` が使う `std::iter::Sum` と
> 二重管理になる問題があった。`#[diagnostic]` は標準トレイトに直接適用できるため、
> `SummableElement` は不要。`D2a`（状態制約）と `D2b`（型境界）に分割して設計を整理する。

#### D2a: 型ステート制約の診断改善

```rust
// src/core/state_diagnostics.rs

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

> **Codex 提案**: 状態ごとのボイラープレートを削減するため、内部マクロ
> `define_state_constraint!` を用意してまとめて定義できるようにする。

#### D2b: 要素型 `T` のトレイト境界違反の診断改善

標準トレイト（`std::iter::Sum`、`Hash`、`Eq` 等）に対して `#[diagnostic]` を**直接**適用する。
独自トレイトを新たに定義・実装する必要はない。

```rust
// sum() の境界: T: Sum に対して診断を付与
// （Sum は std のトレイトだが #[diagnostic] は外部トレイトにも適用可能）

// distinct() / union() 等の境界: T: Hash + Eq
#[diagnostic::on_unimplemented(
    message = "`distinct()` requires T to implement `Hash + Eq`",
    label = "`{T}` does not implement `Hash` or `Eq`",
    note = "add `#[derive(Hash, PartialEq, Eq)]` to your struct"
)]
pub trait HashEqBound: Hash + Eq + private::Sealed {}
impl<T: Hash + Eq> HashEqBound for T {}
```

**現状のエラー（状態違反）**:
```
no method named `select` found for struct `QueryBuilder<User, Initial>`
```

**改善後のエラー（状態違反）**:
```
error: `select()` can only be used in Filtered state
  --> src/main.rs:5:10
   |
 5 |     .select(|u| u.name.clone())
   |      ^^^^^^ this QueryBuilder is in `Initial` state
   |
   = note: call `.where_()` or `.flat_map()` first
```

**改善後のエラー（型境界違反）**:
```
error: `distinct()` requires T to implement `Hash + Eq`
  --> src/main.rs:8:10
   |
 8 |     .distinct()
   |      ^^^^^^^^ `User` does not implement `Hash` or `Eq`
   |
   = note: add `#[derive(Hash, PartialEq, Eq)]` to your struct
```

---

### D3: `rinq_explain!` デバッグマクロ

`cfg(debug_assertions)` 時のみ有効。`release` ビルドでは完全な no-op になる。

> **修正（Claude・Codex レビュー）**: 初版の出力例「`where_(predicate) → 42 items`」は
> `QueryBuilder` の遅延評価と根本的に矛盾していた。`where_` ステップで件数を得るには
> その時点で一度 `Vec` に収集する必要があり、チェーンの lazy 性が失われる。
> 設計選択肢を整理し、仕様に明記する。

#### 設計選択肢（どちらを採用するかは実装フェーズで決定）

**Option A（推奨・v4.0）**: 総所要時間のみ計測、件数は表示しない

```rust
let result = rinq_explain!(
    QueryBuilder::from(users)
        .where_(|u| u.age > 18)
        .order_by(|u| u.name.clone())
        .collect::<Vec<_>>()
);
```

```
[rinq::explain] query completed: 100 → 42 items, total 0.4ms
```

実装は `macro_rules!` + `std::time::Instant` で完結する。遅延評価を損なわない。

**Option B（v4.1 候補）**: ステップ別件数を計測する診断モード

各ステップ後に意図的に中間 `Vec` を作る。`cfg(debug_assertions)` 時のみ有効で、
遅延評価を破壊することをドキュメントに明記する。`MetricsQueryBuilder` の転用も有効。

```
[rinq::explain] ─────────────────────────────────── (diagnostic mode: eager)
  from(Vec<User>)              100 items
    └─ where_(predicate)    →   42 items   0.1ms  ← ここで中間 Vec を作成
    └─ order_by(key)        →   42 items   0.3ms
    └─ collect()            →   42 items   total: 0.4ms
[rinq::explain] ─────────────────────────────────────────────────────────────
```

> **注意**: Option B は「遅延評価の QueryBuilder」の計測値と「実運用の遅延評価」の
> パフォーマンス特性が異なる。ベンチマーク目的での使用は避けること。

```rust
// v4.0: release では完全に透過
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

---

### D4: `pred!` クロージャ糖衣マクロ

`where_` / `take_while` 等の述語クロージャを簡潔に記述する。

```rust
use rinq::pred;

QueryBuilder::from(users)
    .where_(pred!(age > 18))
    .where_(pred!(active == true && name != ""))
    .collect()

// ↓ マクロ展開

QueryBuilder::from(users)
    .where_(|__it| __it.age > 18)
    .where_(|__it| __it.active == true && __it.name != "")
    .collect()
```

**制約と注意**:
- `__it` は内部変数名（衝突回避）
- ネストしたフィールドアクセス（`u.address.city`）もそのまま使える
- メソッド呼び出し（`u.name.is_empty()`）も可
- **フィールド名の存在チェックはマクロ展開後**になる（コンパイル後半のエラー）

#### `pred!` vs `user_fields` の使い分け（Gemini フィードバック）

> `pred!` と F1 の `user_fields` は競合ではなく**補完関係**にある。
> `user_fields::Age` のような構造体は IDE の補完（IntelliSense）が効くが、
> `pred!` はアドホックな述語に向いている。

| アプローチ | IDE 補完 | コンパイル時フィールド検証 | 記述量 | 向いているケース |
|---|---|---|---|---|
| `pred!(age > 18)` | なし（マクロ内） | 展開後 | 最小 | 一時的なフィルタ、プロトタイプ |
| `Age.gt(18)` | **あり** | コンパイル前半 | 中 | 本番コード、チーム開発 |
| クロージャ直書き | あり | コンパイル前半 | 最大 | 複雑なロジック、デバッグ時 |

**推奨**: 本番コードでは `user_fields` の型付きアクセサを優先し、`pred!` はプロトタイプや
`rinq_explain!` でのデバッグ用途に留めるとよい。この指針をドキュメントに明記する。

---

## Phase E: 演算子拡充

他言語の関数型パラダイムから取り込む chainable 演算子。すべて `QueryBuilder<T, State>` の blanket impl として追加し、型ステートは原則 `Filtered` を返す。

---

### E1: `scan` — 汎用累積（Haskell `scanl` / Kotlin `runningFold`）

v3 の `running_sum` / `running_average` を一般化した演算子。

> **修正（Claude・Codex レビュー）**: 初版のクロージャ型 `Fn(B, T) -> B` は
> `B` を所有権ごと消費・返却するが、`Fn` は共有参照セマンティクスのため `B` を move-out できない。
> `FnMut` に変更する。また `std::iter::scan` のシグネチャは `FnMut(&mut B, T) -> Option<C>`
> と異なるため「ラップするだけ」は不正確 — 変換アダプタが必要。

```rust
pub fn scan<B, F>(self, seed: B, f: F) -> QueryBuilder<B, Filtered>
where
    B: Clone + 'static,
    F: FnMut(B, T) -> B + 'static,  // ← Fn → FnMut に修正
```

```rust
// 使用例
let running_product: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    .scan(1, |acc, x| acc * x)
    .collect();
// → [1, 2, 6, 24, 120]

// running_sum は scan で表現可能（後方互換で残す）
let running_sum = QueryBuilder::from(data).scan(0, |acc, x| acc + x);
```

**実装方針**: `std::iter::scan` は `FnMut(&mut B, T) -> Option<C>` なので、
所有権渡しシグネチャ `FnMut(B, T) -> B` へのアダプタを挟む必要がある。

---

### E2: `chunk_by` — 述語による連続グルーピング（Elixir `Enum.chunk_by` / Ruby `Enumerable#chunk`）

連続する要素を「変化点」でチャンクに分ける。`chunk(n)` のサイズ固定版とは別概念。

```rust
pub fn chunk_by<F, K>(self, key: F) -> QueryBuilder<Vec<T>, Filtered>
where
    F: Fn(&T) -> K + 'static,
    K: PartialEq + 'static,
    T: 'static,
```

```rust
// 使用例
let chunks: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 1, 2, 2, 3, 1, 1])
    .chunk_by(|x| *x)
    .collect();
// → [[1,1], [2,2], [3], [1,1]]

// ログの連続エラーをまとめる
let error_bursts: Vec<Vec<LogEntry>> = QueryBuilder::from(logs)
    .chunk_by(|log| log.level)
    .where_(|chunk| chunk[0].level == Level::Error)
    .collect();
```

---

### E3: `dedup` / `dedup_by` — 連続重複除去（Elixir `Enum.dedup`）

`distinct` は全体から重複除去するが、`dedup` は**連続した**重複のみ除去する。

```rust
pub fn dedup(self) -> QueryBuilder<T, Filtered>
where T: PartialEq + 'static

pub fn dedup_by<K, F>(self, key: F) -> QueryBuilder<T, Filtered>
where
    F: Fn(&T) -> K + 'static,
    K: PartialEq + 'static,
```

```rust
// 使用例
let deduped: Vec<i32> = QueryBuilder::from(vec![1, 1, 2, 2, 3, 1, 1])
    .dedup()
    .collect();
// → [1, 2, 3, 1]  ← 非連続の重複は残る（distinct との違い）

// キーによる連続重複除去
let deduped: Vec<Event> = QueryBuilder::from(events)
    .dedup_by(|e| e.kind.clone())
    .collect();
```

---

### E4: `zip_with` — Zip + 変換（Haskell `zipWith`）

`zip` は `(T, U)` タプルを返すが、`zip_with` は変換関数を同時に適用する。

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

```rust
// 使用例
let sums: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    .zip_with(vec![10, 20, 30], |a, b| a + b)
    .collect();
// → [11, 22, 33]

// 価格リストに税率を適用
let prices_with_tax: Vec<f64> = QueryBuilder::from(prices)
    .zip_with(tax_rates, |price, rate| price * (1.0 + rate))
    .collect();
```

---

### E5: `pairwise` — 隣接ペア（Python 3.10 `itertools.pairwise`）

`window(2)` の特殊ケースだが、型が `Vec<T>` ではなく `(T, T)` になり扱いやすい。

```rust
pub fn pairwise(self) -> QueryBuilder<(T, T), Filtered>
where T: Clone + 'static
```

```rust
// 使用例
let pairs: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2, 3, 4])
    .pairwise()
    .collect();
// → [(1,2), (2,3), (3,4)]

// 差分計算
let diffs: Vec<f64> = QueryBuilder::from(prices)
    .pairwise()
    .select(|(a, b)| b - a)
    .collect();
```

---

### E6: `unfold` — シードからシーケンス生成（Haskell `unfoldr`）

`range` / `repeat` より強力な生成演算子。状態を持ちながら有限シーケンスを生成する。

> **修正（Claude レビュー）**: 初版のクロージャ型 `Fn(S) -> Option<(T, S)>` は
> ループ内で `S` を毎回 move-out するため `Fn`（共有参照）では不可能。
> 内部に `Option<S>` を持ち `take()` で swap する実装が必要なため `FnMut` が正しい。
>
> **修正（作成者メモ）**: 無限シーケンスのユースケースは実務では限定的。
> `unfold_bounded` を v4.1 から v4.0 に前倒しし、こちらを推奨エントリポイントとする。
>
> **修正（Codex・Claude）**: `unfold` が `Initial` を返すと `where_` / `order_by` が
> 直接使えない（`Initial` には filtering/sorting API がない）。`Filtered` を返すよう変更する。

```rust
// 静的メソッドとして追加（Filtered を返す）
impl<T: 'static> QueryBuilder<T, Filtered> {
    pub fn unfold<S, F>(seed: S, f: F) -> Self
    where
        S: 'static,
        F: FnMut(S) -> Option<(T, S)> + 'static,  // ← Fn → FnMut に修正
```

```rust
// 使用例

// フィボナッチ数列（有限）
let fibs: Vec<u64> = QueryBuilder::unfold((0u64, 1u64), |(a, b)| {
    if a > 1000 { None } else { Some((a, (b, a + b))) }
})
.collect();
// → [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987]

// ページネーション結果の逐次取得
let all_users: Vec<User> = QueryBuilder::unfold(0usize, |page| {
    let results = fetch_page(page);
    if results.is_empty() { None }
    else { Some((results, page + 1)) }  // ← ここは flat_map と組み合わせる形に
})
.flat_map(|page_results| page_results.into_iter())
.collect();
```

#### `unfold` の安全性設計（Gemini フィードバック、Claude・Codex レビューで強化）

**`unfold_bounded` を v4.0 に前倒し（Claude・Codex）**

`unfold` の安全性問題は v4.0 でユーザーが踏み得るため、`unfold_bounded` を同梱し
こちらを推奨エントリポイントとする。生の `unfold` は上級者向けとして後に掲載する。

```rust
// 推奨: 上限付き版
pub fn unfold_bounded<S, F>(seed: S, max: usize, f: F) -> QueryBuilder<T, Filtered>
where
    S: 'static,
    F: FnMut(S) -> Option<(T, S)> + 'static,

// 上級者向け: 上限なし版（take と組み合わせること）
pub fn unfold<S, F>(seed: S, f: F) -> QueryBuilder<T, Filtered>
where
    S: 'static,
    F: FnMut(S) -> Option<(T, S)> + 'static,
```

**対策: `size_hint` の実装**

```rust
struct UnfoldIter<S, T, F> {
    state: Option<S>,
    f: F,
    _phantom: PhantomData<T>,
}

impl<S, T, F> Iterator for UnfoldIter<S, T, F>
where F: FnMut(S) -> Option<(T, S)>
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let s = self.state.take()?;    // Option<S> から take() で所有権を取得
        match (self.f)(s) {
            Some((item, next_s)) => { self.state = Some(next_s); Some(item) }
            None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)  // 上限不明 = 無限の可能性あり
    }
}
```

**用途例（Codex 指摘: 危険性だけでなく lazy の利点も示す）**

```rust
// 純粋な数列生成（有限）— None で自然終了
let fibs: Vec<u64> = QueryBuilder::unfold_bounded((0u64, 1u64), 20, |(a, b)| {
    Some((a, (b, a + b)))
}).collect();

// take を組み合わせた無限生成（生の unfold）
let first_20 = QueryBuilder::unfold((0u64, 1u64), |(a, b)| Some((a, (b, a + b))))
    .take(20)
    .collect::<Vec<_>>();

// lazy 評価との相性: first() は 1 件生成で停止する
let first_fib = QueryBuilder::unfold((0u64, 1u64), |(a, b)| Some((a, (b, a + b))))
    .first();  // → Some(0)  — 1 回だけクロージャが呼ばれる

// debug_assertions 時: Gemini 提案によるランタイムカウンタ
// unfold が 1_000_000 回を超えた場合に warn! を出す（環境変数で設定可能）
```

> **Gemini 提案**: `debug_assertions` 有効時のみ内部カウンタを持たせ、
> 上限（デフォルト 1,000,000 件）を超えた場合に `log::warn!` を出す安全装置を追加する。

---

### E7: `intersperse` — セパレータ挿入（Haskell `intersperse`）

要素間にセパレータを挿入する。

```rust
pub fn intersperse(self, sep: T) -> QueryBuilder<T, Filtered>
where T: Clone + 'static
```

```rust
// 使用例
let csv_row: Vec<String> = QueryBuilder::from(vec!["Alice".to_string(), "30".to_string(), "Tokyo".to_string()])
    .intersperse(",".to_string())
    .collect();
// → ["Alice", ",", "30", ",", "Tokyo"]

// セパレータを挿入して結合
let joined: String = QueryBuilder::from(words)
    .intersperse(" ".to_string())
    .aggregate(String::new(), |mut acc, s| { acc.push_str(&s); acc });
```

---

### E8: `min_max` — 一度の走査で最小値・最大値（Elixir `Enum.min_max`）

```rust
pub fn min_max(self) -> Option<(T, T)>
where T: Ord + Clone + 'static
```

```rust
let (min, max) = QueryBuilder::from(vec![3, 1, 4, 1, 5, 9])
    .min_max()
    .unwrap();
// min = 1, max = 9
```

`min()` + `max()` の 2 回走査より効率的（1 回の走査で完結）。

---

## Phase F: `rinq-derive` クレート（新規）

### 設計方針

- `rinq` 本体への依存はなし（`rinq` からオプショナルで `rinq-derive` を re-export）
- `proc-macro` クレートとして独立
- 生成コードはすべて安定版 Rust のみ使用

---

### F1: `#[derive(Queryable)]` — フィールドアクセサ自動生成

```toml
[dependencies]
rinq-derive = "4"
```

```rust
use rinq_derive::Queryable;

#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub active: bool,
    pub department: String,
}
```

**生成物 1: フィールドアクセサ関数**（`order_by` / `group_by` 向け）

```rust
// マクロが生成
impl User {
    pub fn by_name(u: &User)       -> &str   { &u.name }
    pub fn by_age(u: &User)        -> u32    { u.age }
    pub fn by_active(u: &User)     -> bool   { u.active }
    pub fn by_department(u: &User) -> &str   { &u.department }
}

// 使用例
QueryBuilder::from(users)
    .order_by(User::by_age)          // fn ポインタをそのまま渡せる
    .group_by(User::by_department)
    .collect()
```

**生成物 2: 型付きフィールド述語**（`where_` 向け）

> **修正（Claude レビュー）**: 初版の `Age.gt(18)`（インスタンスメソッド）は
> ゼロサイズ構造体のインスタンスを作って `.` で呼ぶ形で Rust の慣習と合わない。
> `Age::gt(18)`（関連関数）を採用する。

```rust
// マクロが生成（別モジュールに配置して名前衝突を回避）
pub mod user_fields {
    use super::User;
    pub struct Age;
    impl Age {
        pub fn gt(n: u32)                -> impl Fn(&User) -> bool { move |u| u.age > n }
        pub fn lt(n: u32)                -> impl Fn(&User) -> bool { move |u| u.age < n }
        pub fn eq(n: u32)                -> impl Fn(&User) -> bool { move |u| u.age == n }
        pub fn between(lo: u32, hi: u32) -> impl Fn(&User) -> bool {
            move |u| u.age >= lo && u.age <= hi
        }
    }
    pub struct Active;
    impl Active {
        pub fn is_true()  -> impl Fn(&User) -> bool { |u| u.active }
        pub fn is_false() -> impl Fn(&User) -> bool { |u| !u.active }
    }
    pub struct Name;
    impl Name {
        pub fn eq(s: &str)          -> impl Fn(&User) -> bool { let s = s.to_owned(); move |u| u.name == s }
        pub fn contains(s: &str)    -> impl Fn(&User) -> bool { let s = s.to_owned(); move |u| u.name.contains(&*s) }
        pub fn starts_with(s: &str) -> impl Fn(&User) -> bool { let s = s.to_owned(); move |u| u.name.starts_with(&*s) }
        pub fn is_empty()           -> impl Fn(&User) -> bool { |u| u.name.is_empty() }
    }
}

// 使用例（Age::gt — 関連関数スタイル）
use user_fields::*;

QueryBuilder::from(users)
    .where_(Age::gt(18))        // ← Age.gt(18) から修正
    .where_(Active::is_true())
    .where_(Name::contains("Alice"))
    .order_by(User::by_age)
    .collect()
```

> **Codex 指摘**: `order_by` 向けアクセサ（`&str` を返す `by_name`）と
> `group_by` 向けアクセサ（所有 key が必要）は設計を分ける必要がある。
> `group_by` は内部で `HashMap<K, Vec<T>>` に所有 key を格納するため、
> `&str` を返すアクセサはそのまま使えない。
> **実装フェーズで生成パターンを 2 種類に分けること**。

**属性によるカスタマイズ**:

```rust
#[derive(Queryable)]
pub struct Product {
    pub id: u64,

    #[queryable(skip)]          // アクセサを生成しない
    pub internal_code: String,

    #[queryable(rename = "price_jpy")]  // 生成される関数名を変更
    pub price: f64,

    #[queryable(key)]           // これをデフォルトの sort/group キーとしてマーク
    pub category: String,
}
```

---

### F2: `#[derive(QueryableFrom)]` — コレクション型への `Queryable` 実装

カスタムコレクション型から直接 `QueryBuilder` を作れるようにする。

```rust
#[derive(QueryableFrom)]
pub struct UserList(Vec<User>);

// 生成物
impl From<UserList> for QueryBuilder<User, Initial> {
    fn from(list: UserList) -> Self {
        QueryBuilder::from(list.0)
    }
}

// 使用例
let result = UserList(users).into_query()
    .where_(|u| u.age > 18)
    .collect();
```

---

## Phase G: `rinq-syntax` クレート（新規・実験的）

### 設計方針

- `rinq` 本体とは完全に独立したオプショナルクレート
- `query!` マクロで C# LINQ に近い構文を提供
- 単一 `from` のシンプルなケースを v4.0 の対象とし、JOIN（複数 `from`）は v4.1 以降

```toml
[dependencies]
rinq-syntax = { version = "4", features = ["rinq"] }
```

---

### G1: `query!` マクロ — 基本構文

```rust
use rinq_syntax::query;

// 基本形
let adults: Vec<&User> = query! {
    from user in &users
    where user.age > 18
    order_by user.last_name
    select user
};

// 複数条件
let result: Vec<String> = query! {
    from user in users
    where user.age > 18
    where user.active
    order_by user.last_name, user.first_name
    select user.name.clone()
};

// let バインディング
let result: Vec<String> = query! {
    from user in users
    where user.age > 18
    let display = format!("{} ({})", user.name, user.age)
    select display
};
```

**展開後のコード**:

```rust
// query! { from user in users where user.age > 18 order_by user.name select user.name.clone() }
// ↓
QueryBuilder::from(users)
    .where_(|user| user.age > 18)
    .order_by(|user| user.name.clone())
    .select(|user| user.name.clone())
    .collect()
```

---

### G2: サポートする節（v4.0 スコープ）

| 節 | 対応する QueryBuilder メソッド | 備考 |
|---|---|---|
| `from x in source` | `QueryBuilder::from(source)` | 必須、最初に 1 つだけ |
| `where predicate` | `.where_(|x| predicate)` | 複数可 |
| `order_by key` | `.order_by(|x| key)` | `,` 区切りで複数キー |
| `order_by_desc key` | `.order_by_descending(|x| key)` | |
| `select expr` | `.select(|x| expr).collect()` | 省略可（省略時は `collect()` のみ） |
| `take n` | `.take(n)` | |
| `skip n` | `.skip(n)` | |
| `let name = expr` | マクロ内 let バインディング | クロージャ内に展開 |

**v4.0 スコープ外**（v4.1 以降）:
- `from x in a, from y in b` — JOIN / クロス積
- `group by` — グループ化
- `into` — サブクエリへの継続

---

### G3: エラーメッセージの方針

proc-macro のエラーは展開後コードを指してしまう問題を `proc_macro::Span` で緩和する。

```
error: `order_by` の後に `where` は使えません
  --> src/main.rs:3:5
   |
3  |     where user.age > 18   ← ここ
   |     ^^^^^
   |
   = note: `where` は `from` の直後に記述してください
```

---

### G4: `rinq-syntax` のクレート分離戦略（Gemini フィードバック）

> `rinq-syntax` が `rinq` 本体のプライベートな構造体やメソッドに依存しすぎると、
> 本体のアップデートでマクロが壊れやすくなる。
> マクロ用の「安定した公開インターフェース」を整備しておくとメンテナンス性が向上する。

#### `rinq::query_api` — マクロ用安定インターフェース

`rinq-syntax` は `rinq` の内部構造に直接依存せず、専用の安定 API 層を通じてのみアクセスする。

```
rinq 本体
  ├── src/core/builder/   ← 内部実装（変更自由）
  ├── src/lib.rs          ← 通常の公開 API
  └── src/query_api.rs    ← マクロ専用の安定インターフェース（新設）
        QueryBuilder::__from_macro(source)
        QueryBuilder::__where_macro(pred)
        QueryBuilder::__order_by_macro(key)
        QueryBuilder::__select_macro(proj)
        QueryBuilder::__collect_macro()
```

```rust
// src/query_api.rs（rinq 本体側）
// #[doc(hidden)] で一般ユーザーには非表示にしつつ、pub で外部クレートから呼べる

#[doc(hidden)]
pub mod __macro_support {
    use crate::{QueryBuilder, Filtered, Sorted, Initial};

    /// rinq-syntax が展開するコードのエントリポイント。
    /// 内部実装が変わっても、このシグネチャは semver で保護される。
    pub fn from<T: 'static>(source: Vec<T>) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(source)
    }
    // ... where_, order_by, select, collect
}
```

```rust
// rinq-syntax 側の展開結果（query! マクロの出力）
rinq::__macro_support::from(users)
    .__where_(|user| user.age > 18)
    .__order_by(|user| user.name.clone())
    .__collect()
```

#### バージョン互換性の方針

| 層 | 変更の自由度 | semver 保護 |
|---|---|---|
| `src/core/builder/` 内部実装 | 自由 | なし |
| `src/lib.rs` 公開 API | 破壊的変更は semver major | あり |
| `src/query_api.rs` マクロ用 | 変更には `#[deprecated]` 移行期間 | **強く保護** |

この設計により `rinq` 本体の内部リファクタリングが `rinq-syntax` を壊さなくなる。

---

## Phase H: ライフタイム・設計改善

### H1: `Arc` ソースのサポート

関数の外にクエリを返すユースケースで `Arc<Vec<T>>` を受け取れる構築関数を追加。

> **修正（Claude・Codex・Gemini 三者一致）**: 初版の `from_arc` は内部で `clone()` していたため
> Arc を受け取る意味がなかった（所有権を持つ `Vec<T>` を渡すのと同等）。
> 名称を `from_arc_cloned` に変更して O(N) コピーを明示する。
> ゼロコピー版は `QueryBuilder` の内部ストレージ設計の変更を要するため v5 候補。

```rust
impl<T: Clone + 'static> QueryBuilder<T, Initial> {
    /// Arc の内容を全コピーして QueryBuilder を構築する。O(N) のコピーを伴う。
    /// ゼロコピーが必要な場合は将来の `from_arc_shared` を参照（v5 候補）。
    pub fn from_arc_cloned(source: Arc<Vec<T>>) -> Self {
        let items: Vec<T> = (*source).clone();  // O(N) — ドキュメントに明記
        Self::from(items)
    }

    /// Arc<[T]> の内容を全コピーして QueryBuilder を構築する。O(N) のコピーを伴う。
    pub fn from_arc_slice_cloned(source: Arc<[T]>) -> Self {
        Self::from(source.to_vec())
    }
}

// 使用例
fn build_adult_query(users: Arc<Vec<User>>) -> FilteredQuery<User> {
    QueryBuilder::from_arc_cloned(users)  // ← O(N) コピーが起きることが名前から明らか
        .where_(|u| u.age > 18)
}
```

---

### H2: `tap` / `pipe` — チェーン中の副作用と条件分岐

> **Gemini フィードバック**: `pipe` は地味に見えるが、実戦で最も重宝するメソッドになる。
> Rust の型ステートパターンは「if 文でフィルタを足したり引いたりする」のが苦手だが、
> `pipe` があればチェーンを壊さず動的なクエリ構築が可能になる。
> Elixir の `|>` やマクロに近い柔軟性を型安全に提供できる、非常にクレバーな設計。

#### 型ステートパターンの条件分岐問題

型ステートパターンの最大の弱点は、分岐によって返り値の型が変わるとコンパイルが通らない点にある。

```rust
// これはコンパイルエラー
// then ブランチは FilteredQuery<User>、else ブランチは型が合わない
let q = if only_active {
    QueryBuilder::from(users).where_(|u| u.active)   // FilteredQuery<User>
} else {
    QueryBuilder::from(users)                        // QueryBuilder<User, Initial> ← 型が違う！
};
```

`pipe` はこの問題をクロージャで包むことで解決する：

```rust
// pipe を使えば型が統一される
let q: FilteredQuery<User> = QueryBuilder::from(users)
    .pipe(|q| {
        if only_active { q.where_(|u| u.active) }
        else           { q.where_(|_| true)      }  // 両ブランチとも FilteredQuery<User>
    });

// さらにチェーンを続けられる
let result = q.order_by(|u| u.name.clone()).collect();
```

**`tap_each` / `tap_collect`**: 副作用系メソッド

> **修正（Claude・Codex レビュー）**: 初版の `tap<F: FnOnce(&[T])>` は `&[T]` を渡すために
> 呼び出し時点で全要素を collect する必要があり、遅延評価が失われる設計だった。
> 用途別に 2 バリアントに分割する。

```rust
/// lazy — 各要素を通過させながら副作用を実行（= inspect のコレクション全体版）
pub fn tap_each<F>(self, f: F) -> Self
where F: FnMut(&T) + 'static

/// eager — 呼び出し時点で全要素を収集し、スライスとして副作用関数に渡す
/// ⚠ チェーン途中に置くと遅延評価が破壊される
pub fn tap_collect<F>(self, f: F) -> Self
where
    F: FnOnce(&[T]) + 'static,
    T: 'static,
```

```rust
// tap_each: lazy のまま要素ログ
QueryBuilder::from(users)
    .where_(|u| u.age > 18)
    .tap_each(|u| log::debug!("user: {}", u.name))
    .order_by(|u| u.name.clone())
    .collect()

// tap_collect: 中間集計などに使う（eager 化を明示的に選択）
QueryBuilder::from(users)
    .where_(|u| u.age > 18)
    .tap_collect(|items| log::debug!("after filter: {} items", items.len()))
    .order_by(|u| u.name.clone())
    .collect()
```

| メソッド | 評価タイミング | 主な用途 |
|---|---|---|
| `tap_each` | lazy（要素ごと） | 要素ログ、デバッグカウント |
| `tap_collect` | eager（呼び出し時点で全収集） | バッチログ、中間集計 |

**`pipe`**: 任意の変換を挿入（型ステートを跨いだ条件分岐に対応）

```rust
pub fn pipe<F, T2, S2>(self, f: F) -> QueryBuilder<T2, S2>
where
    F: FnOnce(Self) -> QueryBuilder<T2, S2>,
    T2: 'static,
    S2: 'static,
```

```rust
// 実用例 1: 条件付きフィルタ
let q = QueryBuilder::from(users)
    .pipe(|q| if only_active { q.where_(|u| u.active) } else { q.where_(|_| true) });

// 実用例 2: 動的ソート条件
let q = QueryBuilder::from(users)
    .where_(|u| u.age > 18)
    .pipe(|q| match sort_key.as_str() {
        "age"  => q.order_by(|u| u.age),
        "name" => q.order_by(|u| u.name.clone()),
        _      => q.order_by(|u| u.id),
    });

// 実用例 3: 外部関数への委譲
fn apply_tenant_filter(q: FilteredQuery<User>, tenant_id: u32) -> FilteredQuery<User> {
    q.where_(move |u| u.tenant_id == tenant_id)
}

let q = QueryBuilder::from(users)
    .where_(|u| u.age > 18)
    .pipe(|q| apply_tenant_filter(q, current_tenant));
```

**`tap` と `pipe` の違い**:

| | 型の変化 | 主な用途 |
|---|---|---|
| `tap` | なし（同じ型を返す） | ログ、デバッグ、計測 |
| `pipe` | あり（任意の型へ変換可） | 条件分岐、動的クエリ構築、委譲 |

---

## Phase I: `rinq-stats` 拡張（v4 追加分）

### I1: 時系列演算子

```rust
// 指数移動平均（EMA）
pub fn exponential_moving_average(self, alpha: f64) -> QueryBuilder<f64, Filtered>
where T: Into<f64> + 'static

// ボリンジャーバンド（上限・下限・中央）
pub fn bollinger_bands(self, window: usize, sigma: f64)
    -> QueryBuilder<(f64, f64, f64), Filtered>
where T: Into<f64> + 'static
```

### I2: 外れ値検出

```rust
pub fn remove_outliers_zscore(self, threshold: f64) -> QueryBuilder<T, Filtered>
where T: Into<f64> + Clone + 'static

pub fn remove_outliers_iqr(self) -> QueryBuilder<T, Filtered>
where T: Into<f64> + Clone + 'static
```

### I3: `ValidationExt` 拡張

v3 で実装した `ValidationExt` に条件付き検証とメッセージテンプレートを追加。

```rust
// 依存条件付き検証（他フィールドへの参照）
.validate_if(|r| r.discount > 0.0, |r| r.price > r.discount, "discount", "割引は価格を超えられません")

// カスタムエラー型サポート
.validate_with(|r| {
    if r.price <= 0.0 {
        Err(MyError::InvalidPrice(r.price))
    } else {
        Ok(())
    }
})
```

---

## Phase J: 追加演算子（レビュー拡張案から採用）

複数のレビュアーが推薦した演算子を追加する。

---

### J1: `filter_map` — `Option` を返す変換フィルタ（Codex・Claude 両方推薦）

Rust の `Iterator::filter_map` に相当。`flat_map` で代替できるが、`Option` 専用の明示的な API があると意図が伝わりやすい。

```rust
pub fn filter_map<U, F>(self, f: F) -> QueryBuilder<U, Filtered>
where
    F: Fn(T) -> Option<U> + 'static,
    U: 'static,
```

```rust
// 使用例: 文字列を数値にパース、失敗は除外
let numbers: Vec<i32> = QueryBuilder::from(vec!["1", "two", "3", "four"])
    .filter_map(|s| s.parse::<i32>().ok())
    .collect();
// → [1, 3]
```

---

### J2: `map` — `select` の Rust イディオム alias（Codex 推薦）

`select` は LINQ 由来で意図は明確だが、Rust ユーザーには `map` の方が直感的。
破壊的変更なしに追加できる。

```rust
// map を select の alias として追加（select は維持）
pub fn map<U, F>(self, f: F) -> QueryBuilder<U, Projected<U>>
where
    F: Fn(T) -> U + 'static,
    U: 'static,
{ self.select(f) }
```

---

### J3: `IntoQuery` トレイト（Claude 推薦）

`IntoIterator` と対になる最もイディオマティックな設計。コレクション型に `.into_query()` を生やす。

```rust
pub trait IntoQuery: Sized {
    type Item: 'static;
    fn into_query(self) -> QueryBuilder<Self::Item, Initial>;
}

// 標準コレクションへの blanket impl
impl<T: 'static> IntoQuery for Vec<T> {
    type Item = T;
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}

// 使用例
let result = users.into_query()
    .where_(|u| u.age > 18)
    .collect::<Vec<_>>();
```

`F2 (#[derive(QueryableFrom)])` と補完関係にある。

---

### J4: `collect_vec` — 型注釈不要の収集（Codex 推薦）

```rust
pub fn collect_vec(self) -> Vec<T>
{ self.collect::<Vec<T>>() }
```

`collect::<Vec<_>>()` の型注釈が不要になる。サンプルコードが読みやすくなる。

---

### J5: `step_by` — N ステップごとの要素取得（Claude 推薦）

`std::iter::StepBy` の薄いラッパー。ダウンサンプリングに有用。

```rust
pub fn step_by(self, step: usize) -> QueryBuilder<T, Filtered>
```

```rust
// 時系列データの 1/10 ダウンサンプリング
let sampled = QueryBuilder::from(sensor_readings).step_by(10).collect_vec();
```

---

### J6: `cycle` — 無限繰り返し（Claude 推薦）

`std::iter::Cycle` のラッパー。`unfold` より意図が明確なユースケース向け。
`take` との組み合わせを必須とし、`unfold` 同様に安全性注意を付記する。

```rust
pub fn cycle(self) -> QueryBuilder<T, Filtered>
where T: Clone + 'static
```

```rust
// ラウンドロビン的な割り当て
let assignments: Vec<&str> = QueryBuilder::from(vec!["A", "B", "C"])
    .cycle()
    .take(10)
    .collect_vec();
// → ["A", "B", "C", "A", "B", "C", "A", "B", "C", "A"]
```

---

## 設計上の注意事項（v3 からの引き継ぎ + v4 追記）

### `Filtered` 状態の意味論（Codex レビューで明文化）

> `scan` / `pairwise` / `unfold` 等が `Filtered` を返すのは「フィルタした」からではない。
> 仕様全体でこの解釈を統一する。

**`Filtered` は「論理的なフィルタ済み状態」ではなく「射影・連鎖可能な一般的な中間状態」を意味する。**

| 状態 | 意味 |
|---|---|
| `Initial` | 生成直後。`where_`/`order_by` 等への入口 |
| `Filtered` | 連鎖可能な中間状態。`select`/`order_by` 等に進める |
| `Sorted` | ソート済み。`then_by` / 終端操作のみ |
| `Projected<U>` | 射影済み。`collect()` のみ |

この解釈を `src/core/state.rs` のドキュメントコメントに明記する。

---

### 新演算子の `MetricsQueryBuilder` / `ParallelQueryBuilder` 波及方針（Codex レビュー）

v4 で追加する Phase E・J の演算子（`scan`, `chunk_by`, `dedup`, `pairwise`, `filter_map`, `step_by`, `cycle` 等）は、現行コードで状態別に重複実装されている。

**方針**:
- v4.0 の新演算子は **`QueryBuilder` 本体のみ**を対象とする
- `MetricsQueryBuilder` への追随は **v4.1** に分離する
- `ParallelQueryBuilder` は性質が合う演算子（`dedup`, `filter_map` 等）のみ v4.1 で追随する

この方針を明記しないと「全ビルダーに揃って当然」という期待になり、工数見積もりがぶれる。

---

### 国際化（英語化）方針（作成者メモ）

- コードコメントは**英語**で記述する（リリース前にすべて英語化）
- エラーメッセージは英語を基本とする
- 将来的な i18n 対応として `settings.json` に `{ "language": "ja" }` が設定された場合に
  日本語メッセージを提供する仕組みを検討する（v5 以降の課題）

---

以下は v3 実装中に判明した制約で、v4 の実装・ドキュメント作成時にも適用される。

### 制約 1: `Projected<U>` では `collect` 以外の操作は使えない

- `select` の後は `Projected<U>` 状態になり、`enumerate` / `where_` 等は使えない
- `enumerate` は `select` より前に置く

### 制約 2: `Initial` 状態に `select` は存在しない

- `range` / `repeat` / `empty` / `unfold` の直後に `select` は使えない
- `flat_map` で `Filtered` に遷移してから `select` を使う

### 制約 3: `QueryBuilder::empty()` にターボフィッシュは使えない

- `empty::<T>()` の構文はコンパイルエラー
- 型は変数の型注釈または使用文脈の推論で解決する

### 制約 4: `Box<dyn Iterator>` の性質

- `T: 'static` が必要（参照を含む型はそのまま渡せない）
- ヒープ確保は発生する（`Box` の性質上）
- `no_std` 環境には対応していない
- 完全ゼロヒープが必要なら `rinq-nostd`（別クレート）を検討する

---

## 優先順位サマリ

> `★` はレビューフィードバックにより重要性が上昇・修正されたもの。

| Phase | 内容 | 実装コスト | DX 効果 | 優先度 | 変更点 |
|---|---|---|---|---|---|
| D1 | 型エイリアス（`InitialQuery` 追加 ★） | 数行 | 高 | **最高** | `InitialQuery<T>` 追加 |
| D2 | `#[diagnostic]` D2a/D2b 分割 ★ | 小〜中 | 非常に高 | **最高** | `SummableElement` 廃止 |
| H2 | `tap_each`/`tap_collect` + `pipe` ★ | 中 | 非常に高 | **最高** | `tap` を 2 分割 |
| J1 | `filter_map` ★ | 小 | 高 | **最高** | 新規追加（複数推薦） |
| E1 | `scan`（`FnMut` 修正 ★） | 小〜中 | 中 | 高 | クロージャ型修正 |
| E2 | `chunk_by` | 中 | 高 | 高 | — |
| E3 | `dedup` / `dedup_by` | 小 | 中 | 高 | — |
| E5 | `pairwise` | 小 | 中 | 高 | — |
| D3 | `rinq_explain!`（設計明確化 ★） | 中 | 高（開発時） | 高 | Option A/B 整理 |
| F1 | `#[derive(Queryable)]`（`Age::gt` 修正 ★） | 大（新クレート） | 非常に高 | 高 | 呼び出し構文修正 |
| J2 | `map` alias ★ | 数行 | 中 | 高 | 新規追加 |
| J3 | `IntoQuery` トレイト ★ | 小〜中 | 高 | 高 | 新規追加 |
| J4 | `collect_vec` ★ | 数行 | 中 | 高 | 新規追加 |
| D4 | `pred!`（F1 と補完関係） | 小〜中 | 中 | 中 | — |
| E4 | `zip_with` | 小 | 中 | 中 | — |
| E6 | `unfold` + `unfold_bounded` 前倒し ★ | 中 | 中 | 中 | `FnMut` / `Filtered` 修正 |
| E7 | `intersperse` | 小 | 低〜中 | 中 | — |
| E8 | `min_max` | 小 | 低〜中 | 中 | — |
| J5 | `step_by` ★ | 小 | 中 | 中 | 新規追加 |
| J6 | `cycle` ★ | 小 | 中 | 中 | 新規追加 |
| H1 | `from_arc_cloned`（命名修正 ★） | 小 | 中 | 中 | 名称変更 |
| I1/I2 | rinq-stats 時系列・外れ値 | 中 | 中（専門用途） | 中 | — |
| I3 | ValidationExt 拡張 | 小〜中 | 中 | 中 | — |
| G1 | `query!` マクロ + 安定 API 層 | 大（新クレート） | 高（訴求力） | 低〜中 | MVP 絞り込み検討 |
| F2 | `QueryableFrom` | 小 | 低〜中 | 低 | — |

---

## 破壊的変更の方針

v4.0 では一切の破壊的変更を行わない。

将来的に破壊的変更が必要になった場合は v5.0 として別バージョンに分離し、`rinq-compat` クレートで移行パスを提供する。
