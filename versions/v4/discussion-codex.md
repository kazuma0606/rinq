# RINQ v4.0 仕様レビュー（Codex）

対象: `versions/v4/spec.md`  
レビュー日: 2026-03-26

## 総評

v4 の方向性は妥当です。特に `D1`、`D2`、`H2` は、既存の type-state 設計が持つ「学習コスト」と「条件分岐の書きにくさ」を直接下げるため、DX 向上の打ち手として筋が良いです。  
一方で、現行実装は `Initial` / `Filtered` / `Sorted` に似た API が分散し、さらに `MetricsQueryBuilder` 側にも同系統の実装が重複しています。そのため v4 で演算子を増やすほど、**仕様上は小さく見える変更でも実装面の波及が大きい**点は明示しておくべきです。

## 指摘事項

### 1. `D2` は「no method found」改善と「trait bound 未充足」改善を分けて設計したほうがよい

仕様では `#[diagnostic::on_unimplemented]` を型ステート制約と要素型 `T` の両方に適用する案になっていますが、この 2 つは性質が違います。

- 状態違反:
  `select()` が `Initial` に存在しない、`then_by()` が `Sorted` 以外に存在しない
- 型境界違反:
  `sum()` に必要な `Sum`、`distinct()` に必要な `Hash + Eq` など

現行実装では、前者は「メソッド未定義」で表現され、後者は「メソッドはあるが trait bound を満たさない」で表現されています。  
したがって `D2` を一括りにせず、以下の 2 段階に分けたほうが実装計画が安定します。

- `D2a`: 状態制約専用 trait を導入して診断改善
- `D2b`: 数値演算・集合演算用の補助 trait を導入して境界違反を改善

特に `sum()` は現行コードで `T: Sum`、`average()` は `T: ToPrimitive` を使っているため、仕様中の `Into<f64>` 例は現実装とずれています。ここは実装寄りに揃えたほうがよいです。

### 2. `D3 rinq_explain!` はマクロだけではステップごとの情報を十分に取れない

仕様の出力例は魅力的ですが、単に最終式を `macro_rules!` で包むだけでは、

- `where_`
- `order_by`
- `collect`

ごとの件数や所要時間を自動で分解して観測することは難しいです。  
現行の `QueryBuilder` はパイプラインの AST を保持せず、その場で `Iterator` を積み上げているためです。

実現方法は少なくとも次のどちらかです。

- `debug_assertions` 時のみ内部に簡易プラン情報を持つ
- `MetricsQueryBuilder` をデバッグ用途に転用し、`rinq_explain!` は薄い糖衣にする

`macro_rules!` 単独で完結すると書くと期待値が上がりすぎるので、仕様では「内部計測フック追加を含む可能性がある」と書いておくほうが安全です。

### 3. `D4 pred!` は構文境界を明確化しないと利用者が迷う

`pred!(age > 18)` のような最短形は便利ですが、どこまで許容するかを決めないと曖昧さが残ります。

特に以下は先に定義したいです。

- フィールド参照だけを糖衣化するのか
- ローカル変数の capture を許すのか
- メソッド呼び出しはどこまで許すのか
- `&&` / `||` / `!` / `match` のような式をどこまで受けるのか

`pred!` は「汎用式マクロ」に寄せるほど実装が重くなり、逆に「単純なフィールド式専用」に寄せるほど使い勝手が落ちます。  
仕様上は次のように絞るのが現実的です。

- v4.0 ではフィールドアクセスと比較演算、論理演算まで
- より複雑な式は通常クロージャにフォールバック

### 4. Phase E の演算子追加は `MetricsQueryBuilder` と `ParallelQueryBuilder` への波及方針が必要

現行コードではコア演算子の多くが:

- `QueryBuilder<Initial>`
- `QueryBuilder<Filtered>`
- `QueryBuilder<Sorted>`
- `MetricsQueryBuilder<...>`

に重複実装されています。  
v4 で `scan`、`chunk_by`、`dedup`、`zip_with`、`pairwise`、`intersperse`、`min_max` を増やすと、仕様上の 1 機能が複数箇所の追加作業に増幅されます。

そのため仕様に以下の明記があるとよいです。

- v4.0 の新演算子はまず `QueryBuilder` 本体のみを対象とする
- `MetricsQueryBuilder` 追随は同リリース内の後半、または v4.1 に分離する
- `ParallelQueryBuilder` は一部演算子のみ追随対象とする

これを書かないと「全部に揃って当然」という期待になり、工数見積もりがぶれます。

### 5. `E1 scan` の戻り状態は `Filtered` 固定でよいか再確認したほうがよい

仕様では `scan` が `QueryBuilder<B, Filtered>` を返します。  
現行の状態機械では「`select` を使うには `Filtered` に入る必要がある」ため、実用上は自然です。

ただし意味論としては `scan` は「フィルタ」ではなく「変換」です。  
この設計を採るなら、仕様に以下の意図を明文化したほうがよいです。

- `Filtered` は論理的な filter 状態ではなく「射影可能・連鎖可能な一般状態」
- `Projected<U>` は終端に近い特殊状態

ここが曖昧だと、将来 `map` 相当や `pairwise` 追加時に毎回「なぜ Filtered なのか」が再燃します。

### 6. `E2 chunk_by` と `E3 dedup_by` の key で `Clone` を避けられるか検討余地がある

仕様では `K: PartialEq` にしていて比較主体の設計ですが、実装方式によっては毎回 key を生成・保持する必要があります。  
特に `chunk_by` は「前回キーとの比較」が必要なので、次のどちらで行くかを決めたほうがよいです。

- `K: PartialEq + Clone`
- `F: Fn(&T) -> K` を毎回評価し、前回値だけ保持する

後者でも実装できますが、重い key だとコスト説明が必要です。

### 7. `E5 pairwise` は `Clone` 必須よりも borrowing 版の余地を残しておきたい

`pairwise(self) -> QueryBuilder<(T, T), Filtered>` で `T: Clone` は分かりやすいです。  
ただし大きい構造体では clone コストが目立ちます。

v4.0 では所有版だけでよいとしても、仕様メモとして以下を残す価値があります。

- v4.1 候補: `pairwise_ref()` または `window2_ref()` のような参照版

現行が `Box<dyn Iterator<Item = T>>` 中心なので参照版は難しいですが、将来の方向としては有益です。

### 8. `E6 unfold` は「危険」だけでなく「遅延実行」であることも前面に出すべき

安全性の話は十分ありますが、`unfold` の本質は:

- lazy に生成される
- `take` / `first` / `any` などと非常に相性が良い

という点です。  
今の記述だと「危ない演算子」という印象が先に立つので、用途面も補ったほうがよいです。

また、`fetch_page(page)` の例は同期 I/O をクロージャに埋め込むため、利用者に「重い副作用を Iterator に混ぜる」感覚を与えます。  
この例は悪くないですが、ドキュメント上は以下を併記するとバランスが取れます。

- 純粋な数列生成例
- ページングや cursor 走査の副作用例

### 9. `F1 derive(Queryable)` は生成 API の命名衝突を最初に整理したほうがよい

例えば `User::by_name`、`user_fields::Name` のような生成物は分かりやすい一方で、既存メソッドや型名と衝突しやすいです。  
少なくとも次を仕様に入れたほうがよいです。

- 生成先モジュールのデフォルト名
- 衝突時の rename 規則
- tuple struct / generic struct / lifetime 付き struct の扱い
- private field を生成対象に含めるか

また `group_by(User::by_department)` は `&str` を返していても成立しますが、返り値ライフタイムと key 所有権の説明が必要です。  
現行 `group_by` は `K: Eq + Hash` を所有して `HashMap<K, Vec<T>>` に入れるので、`&str` を返す設計はそのままだと噛み合わない可能性があります。

ここは次のどちらかに寄せると明瞭です。

- `group_by` 向けアクセサは所有 key を返す
- `order_by` 向けと `group_by` 向けで生成アクセサを分ける

### 10. `F2 QueryableFrom` は便益が小さい割に学習面のノイズになりやすい

`UserList(Vec<User>)` から直接 `into_query()` できるのは便利ですが、優先順位表どおり低優先で問題ありません。  
現状の `QueryBuilder::from(list.0)` で十分なケースが多く、v4.0 の訴求点としては弱めです。

もし入れるなら `derive(Queryable)` と同梱ではなく、明確に後回しにしたほうがよいです。

### 11. `G1 query!` は「表面上シンプル、保守は重い」ので MVP をさらに絞ってよい

`query!` は訴求力が高いですが、proc-macro の保守負荷に対して、コア演算子拡張の直接利益が薄いです。  
特に以下が重くなります。

- span ベースの診断調整
- `order_by a, b` の複数キー展開
- `let` 節のスコープ規則
- `where` と `select` の順序エラー診断

そのため MVP はさらに絞ってよいです。

- v4.0: `from` / `where` / `select` / 単一 `order_by`
- `let` と複数キー `order_by` は v4.1

仕様の安定 API 層案自体は良いですが、そこまでやるなら `query!` の文法面は意図的に小さくすべきです。

### 12. `H1 from_arc` は利便性よりコピーコストの説明が必要

`from_arc(Arc<Vec<T>>)` は名前から共有参照ベースの安価な取り込みを想像しやすいですが、仕様では `clone()` / `to_vec()` によって全件コピーします。  
これは悪くありませんが、メソッド名だけ見ると誤解が起きやすいです。

次のどちらかを推奨します。

- 名前を `from_arc_cloned` に寄せる
- 仕様に「ゼロコピーではない」と太字で明記する

### 13. `H2 tap` は `&[T]` 固定だと iterator パイプラインと相性が悪い可能性がある

仕様の `tap<F>(self, f: F) where F: FnOnce(&[T])` は、全件を一度 materialize する前提に見えます。  
現行の `QueryBuilder` は多くのケースで lazy iterator を積んでいるため、`tap` が `&[T]` を渡すには内部で collect が必要です。

それにより:

- `tap` を挟むだけで eager 化する
- 大規模データでメモリ特性が変わる

という設計変化が起きます。  
用途がログ・デバッグ中心なら、まずは次の形のほうが現行設計に合っています。

```rust
pub fn tap<F>(self, f: F) -> Self
where
    F: FnOnce(),
```

または

```rust
pub fn tap_collect<F>(self, f: F) -> Self
where
    F: FnOnce(&[T]),
    T: Clone,
```

つまり `tap` と `collect-view` を分けたほうがよいです。

### 14. `pipe` は v4 の中核候補だが、ドキュメントに「同型分岐」と「異型分岐」を分けて書くべき

`pipe` は非常に良いです。  
ただし実際の利用は次の 2 種に分かれます。

- 同じ `T, State` のまま条件分岐する
- 別の `T2, S2` に変換する

この 2 つは ergonomics も認知負荷も違います。  
まずはドキュメント例を「同型分岐中心」にしたほうが導入しやすいです。異型分岐は強力ですが、最初から前面に出すと抽象度が上がります。

## 追加で入れたい拡張案

### A1. `map` の導入を再検討する

現行の `select` は LINQ 由来で理解できますが、Rust 利用者には `map` のほうが直感的です。  
v4 では破壊的変更なし方針なので、次の形が自然です。

- `select` を維持
- `map` を `select` の alias として追加

これだけで Rust ユーザーの初見コストが下がります。

### A2. `filter_map` は優先度を上げる価値がある

`flat_map` は既にありますが、`Option` ベースの射影は `filter_map` が最も自然です。

```rust
QueryBuilder::from(rows)
    .filter_map(|r| r.email.clone())
    .collect()
```

これは `pred!` や `derive(Queryable)` より小コストで、実務上の使用頻度はかなり高いです。

### A3. `find_map` / `first_map` も相性が良い

`unfold` や `pipe` ほど派手ではありませんが、早期終了系のユーティリティとして有用です。

```rust
pub fn find_map<U, F>(self, f: F) -> Option<U>
where
    F: FnMut(T) -> Option<U>
```

現行の iterator ベース設計とよく噛み合います。

### A4. `inspect_count` または軽量メトリクス API

`rinq_explain!` が重い場合の代替として、もっと小さい観測 API があるとよいです。

例:

```rust
QueryBuilder::from(users)
    .where_(...)
    .inspect_count("after where")
    .order_by(...)
```

または `MetricsCollector` を使わずに、debug build 専用で件数だけ記録する軽量版でもよいです。

### A5. `group_adjacent_by` を `chunk_by` の別名候補として検討

`chunk_by` は他言語経験者には伝わりますが、初見だと「サイズベースの chunk の亜種」に見えがちです。  
ドキュメント用の別名、または説明語として `group_adjacent_by` を併記すると誤解が減ります。

### A6. `collect_vec()` の追加

`collect::<Vec<_>>()` は Rust では普通ですが、ライブラリ UX としては次があるとわずかに楽です。

```rust
pub fn collect_vec(self) -> Vec<T>
```

小さい追加ですが、サンプルコードが読みやすくなります。

### A7. `try_scan` は将来拡張として相性が良い

v3 で `TryQueryBuilder` が既にあるため、v4.1 以降で

- `scan`
- `try_scan`

の並びは自然です。  
失敗しうる状態遷移やストリーム処理に備えるなら、先に名前だけ設計メモへ置いておく価値があります。

## 実装優先順位の再提案

仕様の優先度表は大筋妥当ですが、現行コードの重複実装を踏まえると次の順で進めるのが安全です。

1. `D1` 型エイリアス
2. `H2 pipe`
3. `E1 scan`
4. `E3 dedup / dedup_by`
5. `E5 pairwise`
6. `E8 min_max`
7. `D2` 診断改善
8. `E2 chunk_by`
9. `H1 from_arc` または命名見直し版
10. `D3 rinq_explain!`
11. `D4 pred!`
12. `F1 derive(Queryable)`
13. `G1 query!`

理由は単純で、前半ほど:

- コア価値が高い
- API 面積が小さい
- 現行実装に馴染みやすい
- 新クレート導入を伴わない

からです。

## 仕様文書に追記したい短い注記

最後に、`spec.md` 自体へ入れておくと有効な短い注記を列挙します。

- `Filtered` は「フィルタ済み」ではなく「継続操作可能状態」という意味合いを含む
- v4 新演算子の `MetricsQueryBuilder` / `ParallelQueryBuilder` 追随範囲は別途定義する
- `from_arc` 系はゼロコピーではない
- `rinq_explain!` は内部計測フック追加を伴う可能性がある
- `derive(Queryable)` の生成アクセサは `order_by` 用と `group_by` 用で返り値設計を分ける可能性がある
- `query!` は MVP を小さく保ち、複数 `order_by` と `let` は後続フェーズでもよい

## 結論

v4 の核は `D1`、`D2`、`H2` と軽量な演算子追加です。  
`derive` と `syntax` は魅力的ですが、コアの ergonomics 改善が先です。

特に `pipe` は、現行の type-state 設計の弱点を最小の API 追加で補えるため、v4 の中心機能として扱う価値があります。  
逆に `rinq_explain!`、`derive(Queryable)`、`query!` は見栄えが良い反面、内部設計や保守コストの見積もりを慎重にしたほうがよいです。
