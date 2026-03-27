# RINQ v4.0 仕様 — レビューディスカッション

**作成**: 2026-03-26
**対象**: `versions/v4/spec.md`

---

## 1. 指摘事項（潜在的な問題・要検討点）

### D1: `InitialQuery<T>` が欠落している

型エイリアスに `FilteredQuery` / `SortedQuery` / `ProjectedQuery` はあるが、`Initial` 状態のエイリアスがない。`unfold` / `range` / `repeat` の戻り値を関数シグネチャで表現するときに困る。

```rust
// 現状、これを型注釈で書けない
fn make_range(n: i32) -> ??? {
    QueryBuilder::range(0, n)
}
// InitialQuery<i32> があれば書ける
```

**提案**: `pub type InitialQuery<T> = QueryBuilder<T, Initial>;` を D1 に追加する。

---

### D2: `SummableElement` のシールドトレイト実装が既存の `Sum` と二重管理になる

仕様では `SummableElement` を新たに定義し、`i32` / `i64` / `f64` 等に手動で `impl` している。しかし現行の `sum()` は `std::iter::Sum` トレイトを使っているはず。

- 将来 `u8` / `f32` / `isize` 等を追加するたびに両方のトレイトに impl が必要になる
- `SummableElement` と `Sum` の実装漏れが起きると「型エラーメッセージは出るが実装できない」という謎の状況になる

**提案**: `SummableElement` を別定義せず、既存の `T: Sum` トレイト境界に対して `#[diagnostic::on_unimplemented]` を直接適用する方向を検討する（Rust 1.78 の `#[diagnostic]` はコアトレイトにも適用できる）。

---

### D3: `rinq_explain!` の「ステップ別件数」は遅延評価と相容れない

出力例に `where_(predicate) → 42 items` とあるが、`QueryBuilder` は terminal 操作まで実行されない。ステップ別件数を得るには各ステップで中間的に `collect` する必要があり、**ベンチマーク値が実運用と乖離する**。

- `where_` + `order_by` を別々に計測するには `where_` 後に一度 Vec に落とすしかない
- これは「クエリチェーンの遅延評価」という核心的な設計と衝突する

**提案 A**: 件数は表示せず、総所要時間のみを計測する簡易版に留める。
**提案 B**: `cfg(debug_assertions)` 時のみ各ステップ後に中間 Vec を作る「診断モード」として、遅延評価を意図的に破壊する旨をドキュメントに明記する。

---

### E1: `scan` のクロージャ型が `std::iter::scan` と異なる

仕様の `scan` シグネチャ:

```rust
F: Fn(B, T) -> B  // B を consume して返す
```

`std::iter::scan` のシグネチャ:

```rust
F: FnMut(&mut B, T) -> Option<C>  // B を可変参照で受け取る、打ち切り可能
```

「`std::iter::scan` をラップするだけでほぼ実装できる」とあるが、シグネチャが異なるため直接ラップはできない。変換アダプタが必要になる。また、`Fn` (共有参照) では `B` を所有権ごと渡せないため、実際には `FnMut` が適切。

**提案**: シグネチャを `F: FnMut(B, T) -> B` に変更する。または `FnMut(&mut B, T)` として std との一貫性を保つ。

---

### E6: `unfold` のクロージャが `Fn` では動作しない

```rust
F: Fn(S) -> Option<(T, S)>
```

`S` を所有権ごと受け取って次の `S` を返す関数型だが、ループ内で毎回 `S` を move-out するには `Fn`（共有参照セマンティクス）では不可能。実際には内部に `Option<S>` を持たせ、`take()` で swap する実装になるため、**クロージャは `FnMut` にすべき**。

```rust
// 正しい実装イメージ
struct UnfoldIter<S, T, F> {
    state: Option<S>,  // None になったら終端
    f: F,
}
// next() では self.state.take() → f(s) → self.state = next_s
// → FnMut(S) -> Option<(T, S)> が適切
```

---

### H1: `from_arc` がクローンしてしまい Arc の意味がない

```rust
pub fn from_arc(source: Arc<Vec<T>>) -> Self {
    let items: Vec<T> = (*source).clone();  // ← Arc を使いながら全コピー
    Self::from(items)
}
```

`Arc` の目的は共有・参照カウントによるコピー回避なので、全クローンしてしまうと `Arc` を受け取る意味がほとんどない（所有権を持つ `Vec<T>` を渡すのと同等になる）。

**提案**: `QueryBuilder` の内部ストレージとして `Arc<Vec<T>>` を直接保持し、イテレーションをインデックスベース（`..items.len()` のインデックスループ）で実装するか、`Arc<Vec<T>>` から `into_iter` 相当のアダプタを作る設計を検討する。

---

### H2: `tap` の実装には中間コレクションが必要

```rust
pub fn tap<F>(self, f: F) -> Self
where F: FnOnce(&[T])
```

`&[T]` を渡すには全要素をメモリに展開する必要がある。`tap` をチェーン途中に置くと**そこで一度 Vec に収集されてしまい**、遅延評価の利点が失われる。

`inspect`（要素ごとの副作用）は遅延評価のままだが、`tap`（コレクション全体の副作用）は本質的に eager になる。

**提案**: 仕様に「`tap` は呼び出し時点で全要素を実体化する（eager 操作）」と明記し、ユーザーに想定外のコスト増を警告する。または `FnMut(&T)` シグネチャの `tap_each`（= `inspect` と同等）に限定し、コレクション全体を取る `tap` は削除する。

---

### F1: `Age.gt(18)` は Rust 的に不自然（インスタンス不要なのに `.` 呼び出し）

```rust
.where_(Age.gt(18))
```

`Age` はフィールドレスのゼロサイズ構造体。インスタンスを作らず `Age::gt(18)` （関連関数）の方が Rust の慣習に合っている。`.gt()` はメソッドに見えるが、実質的にはファクトリ関数。

**提案**: 関連関数 `Age::gt(18)` を採用するか、または `age::gt(18)` のようなモジュール関数スタイルを検討する。ドキュメントで「`.` は意図的な設計選択である」旨を明記するか、採用理由を spec に追記する。

---

### G1: `query!` が `select` を省略したとき返り値型が曖昧

仕様の G2 テーブルに「`select` 省略可（省略時は `collect()` のみ）」とある。しかし省略時にどの型で `collect` するかは文脈依存になり、`query! { from u in users where u.age > 18 }` の型が `Vec<User>` なのか `Vec<&User>` なのかがマクロ展開前には不明。

**提案**: `select` を省略可能にする場合、要素型 `T` を明示的に型注釈で要求するか、`select` を必須にするかを仕様に明記する。

---

### 制約 2 と `unfold` の矛盾

仕様の「制約 2: Initial 状態に select は存在しない」セクションに `unfold` が明記されているが、E6 の使用例では `unfold` 直後に `flat_map` を挟んで `Filtered` に遷移している。

`unfold` が `QueryBuilder<T, Initial>` を返すなら、`where_` / `order_by` も直接使えない（`Initial` 状態は filtering/sorting API を持たない）という使い勝手の悪さがある。

**提案**: `unfold` は `QueryBuilder<T, Initial>` でなく `QueryBuilder<T, Filtered>` を返すよう設計変更を検討する。生成演算子（range / repeat / empty も同様）に `Filtered` を返させることで「生成 → フィルタ → ソート → 収集」の自然なチェーンが可能になる。

---

## 2. 拡張案

### 拡張案 A: `filter_map` — `Option` を返す変換フィルタ

Rust の `Iterator::filter_map` に相当する演算子。現在は `flat_map` で代替できるが、`Option` 専用の明示的な API があると意図が伝わりやすい。

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

`flat_map(|x| f(x).into_iter())` との違い: 意図が明確、`Option` 以外への誤適用を型で防ぐ。

---

### 拡張案 B: `try_collect` — `Result` / `Option` の一括収集

`Iterator::collect::<Result<Vec<T>, E>>()` はすでに `std` で使えるが、`QueryBuilder` のターミナル操作として明示的に提供すると API の一貫性が高まる。

```rust
pub fn try_collect<U, E>(self) -> Result<Vec<U>, E>
where T: Into<Result<U, E>>

// または
pub fn collect_results<U, E>(self) -> Result<Vec<U>, E>
where T: Into<Result<U, E>>
```

```rust
// 使用例: 全要素が Ok なら Vec を返す、一つでも Err なら Err を返す
let results: Result<Vec<i32>, _> = QueryBuilder::from(raw_data)
    .select(|s: String| s.parse::<i32>())
    .try_collect();
```

---

### 拡張案 C: `step_by` — N ステップごとの要素取得

`std::iter::StepBy` に対応する薄いラッパー。ダウンサンプリングや定期サンプリングに有用。

```rust
pub fn step_by(self, step: usize) -> QueryBuilder<T, Filtered>
```

```rust
// 使用例: 偶数インデックスの要素のみ
let every_other: Vec<i32> = QueryBuilder::from(0..10)
    .step_by(2)
    .collect();
// → [0, 2, 4, 6, 8]

// 時系列データの 1/10 ダウンサンプリング
let sampled = QueryBuilder::from(sensor_readings).step_by(10).collect();
```

---

### 拡張案 D: `cycle` — 無限繰り返し

`std::iter::Cycle` のラッパー。`unfold` より意図が明確なユースケース向け。`take` と組み合わせることを必須とする（`unfold` 同様に安全性注意を付記）。

```rust
pub fn cycle(self) -> QueryBuilder<T, Filtered>
where T: Clone + 'static
```

```rust
// 使用例: ラウンドロビン的な割り当て
let assignments: Vec<&str> = QueryBuilder::from(vec!["A", "B", "C"])
    .cycle()
    .take(10)
    .collect();
// → ["A", "B", "C", "A", "B", "C", "A", "B", "C", "A"]
```

---

### 拡張案 E: `IntoQuery` トレイト — コレクションからのクエリ構築を統一

現在のエントリポイントは `QueryBuilder::from(collection)` のみ。標準の `IntoIterator` に倣い、`.into_query()` をコレクション型に生やすトレイトを提供することで、よりイディオマティックな記述が可能になる。

```rust
pub trait IntoQuery: Sized {
    type Item;
    fn into_query(self) -> QueryBuilder<Self::Item, Initial>;
}

// 標準コレクションへの blanket impl
impl<T: 'static> IntoQuery for Vec<T> {
    type Item = T;
    fn into_query(self) -> QueryBuilder<T, Initial> {
        QueryBuilder::from(self)
    }
}
```

```rust
// 現在
let result = QueryBuilder::from(users).where_(|u| u.age > 18).collect();

// IntoQuery トレイトあり
let result = users.into_query().where_(|u| u.age > 18).collect();
```

`F2 (#[derive(QueryableFrom)])` とも補完関係になる。

---

### 拡張案 F: `order_by_multiple` — 複数キーの一括指定

現在の多段ソートは `order_by` + `then_by` + `then_by` のチェーンが必要。タプルまたはスライスで複数キーを一括指定できるバリアントがあると、動的ソートキー構築（H2 `pipe` の実用例にもある）が楽になる。

```rust
// イメージ（実装は要検討）
QueryBuilder::from(users)
    .order_by_keys(&[SortKey::Asc(User::by_department), SortKey::Desc(User::by_age)])
    .collect()
```

現在の `pipe` + `match` パターンの代替として、仕様の H2 の「動的ソート条件」ユースケースに直接対応できる。

---

### 拡張案 G: `unfold_bounded` の v4.0 への前倒し

仕様では「v4.1 候補」とされているが、`unfold` の安全性問題は v4.0 でユーザーが踏み得るため、同時リリースが望ましい。最大件数を型レベルで持つ必要はなく、ランタイムの `max: usize` パラメータで十分。

```rust
pub fn unfold_bounded<S, F>(seed: S, max: usize, f: F) -> Self
where
    S: 'static,
    F: FnMut(S) -> Option<(T, S)> + 'static,
```

`unfold` と `take(max)` の合成に過ぎないが、「上限なしで `unfold` を使う」ことへのバリアになる。ドキュメントで `unfold_bounded` を先に紹介し、生の `unfold` は上級者向けとして後に掲載する構成にすると安全。

---

### 拡張案 H: `tap` の設計を 2 バリアントに分割

H2 の `tap` の設計問題（指摘事項参照）への解決案として、シグネチャ別に 2 つに分割する：

| メソッド | シグネチャ | 評価タイミング | 主な用途 |
|---|---|---|---|
| `tap_each` | `FnMut(&T)` | lazy（要素ごと） | 要素ログ、デバッグカウント |
| `tap_collect` | `FnOnce(&[T])` | eager（呼び出し時点で全収集） | バッチログ、中間集計 |

`tap` という名前は `tap_collect` に割り当て、`inspect` を `tap_each` として rename するか別名提供する形も選択肢の一つ。

---

## 3. まとめ

### 最優先で対応を推奨する指摘

| 優先度 | 指摘 | 理由 |
|---|---|---|
| 必須 | E6: `unfold` の `Fn` → `FnMut` | 現行のまま実装不可能 |
| 必須 | D3: `rinq_explain!` の設計明確化 | 遅延評価と矛盾しており、ユーザーの期待を裏切る可能性が高い |
| 高 | H2: `tap` の eager 性の明記 | サイレントなパフォーマンスコスト |
| 高 | H1: `from_arc` の再設計 | Arc を使う意味がない実装になっている |
| 中 | D1: `InitialQuery<T>` の追加 | 小変更で利便性が上がる |
| 中 | F1: `Age.gt` vs `Age::gt` の議論 | Rust の慣習との整合 |

### 最も可能性を感じる拡張案

| 拡張案 | 理由 |
|---|---|
| E: `IntoQuery` トレイト | `IntoIterator` と対になる、最もイディオマティックな設計 |
| A: `filter_map` | 実用頻度が高く、`flat_map` との使い分けが明確 |
| G: `unfold_bounded` の前倒し | 安全性問題の解決として v4.0 同梱が妥当 |
| H: `tap` 2 バリアント分割 | 既存の設計問題を解消しつつ API を豊かにできる |
