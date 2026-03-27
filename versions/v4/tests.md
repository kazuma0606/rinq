# RINQ v4.0 テスト仕様

**作成日**: 2026-03-26

---

## テスト方針

- 統合テストは `tests/` 以下にフェーズごとのファイルを作成する
- `rinq-derive` / `rinq-syntax` / `rinq-stats` はそれぞれのクレート内 `tests/` に配置する
- 各メソッドに doc test を 1〜2 例付ける（最低限: 基本動作・空コレクション）
- 新演算子はすべて `QueryBuilder` 本体への適用を対象とする（`MetricsQueryBuilder` は v4.1）
- v3 の型ステート制約（`Projected<U>` / `Initial` の制約）を引き継ぐ

---

## Phase 4A: DX 強化テスト

### ファイル: `tests/rinq_dx_tests.rs`

#### D1: 型エイリアス

```rust
// 型エイリアスを戻り値型に使った関数が定義できる
fn get_adults(users: Vec<User>) -> FilteredQuery<User> { ... }
fn make_range(n: i32) -> InitialQuery<i32> { ... }
fn get_names(users: Vec<User>) -> ProjectedQuery<User, String> { ... }

test_type_alias_in_function_signature
  // 型エイリアスを使った関数が正しくコンパイル・実行できることを確認

test_initial_query_from_range
  // InitialQuery<i32> を返す関数が QueryBuilder::range と同じ動作をする

test_filtered_query_chaining
  // FilteredQuery<User> を返す関数の戻り値に .order_by().collect() をチェーンできる

test_sorted_query_type_alias
  // SortedQuery<User> を戻り値型に使える
```

#### D3: `rinq_explain!`

```rust
test_rinq_explain_returns_correct_result
  // rinq_explain!(query.collect::<Vec<_>>()) が collect() と同じ結果を返す

test_rinq_explain_is_noop_in_release
  // cfg(not(debug_assertions)) 時に副作用がないことを確認（eprintln が呼ばれないこと）
  // → #[cfg(debug_assertions)] ブロックの存在を確認するコンパイルテスト
```

#### D4: `pred!`

```rust
test_pred_single_condition
  // pred!(age > 18) が |u| u.age > 18 と同じ動作をする

test_pred_with_string_field
  // pred!(name == "Alice") の動作確認

test_pred_and_chain
  // pred!(age > 18 && active == true) の動作確認
```

---

## Phase 4B: ライフサイクル改善テスト

### ファイル: `tests/rinq_lifecycle_tests.rs`

#### H1: `from_arc_cloned`

```rust
test_from_arc_cloned_basic
  // Arc<Vec<i32>> から QueryBuilder を構築し、collect() が元の Vec と同じ結果を返す

test_from_arc_cloned_empty
  // Arc<Vec<i32>>::new(vec![]) から空の QueryBuilder が作れる

test_from_arc_cloned_shared_arc
  // Arc を複数スレッドで共有しつつ QueryBuilder を構築できる
  // （Arc のクローンを別スレッドに渡してから QueryBuilder を構築）

test_from_arc_slice_cloned_basic
  // Arc<[i32]> から QueryBuilder を構築できる

test_from_arc_cloned_with_filter
  // from_arc_cloned().where_(...).collect() が期待通りに動作する
```

#### H2a: `tap_each`

```rust
test_tap_each_side_effect_per_element
  // atomic カウンタを使い、tap_each のクロージャが要素ごとに呼ばれることを確認

test_tap_each_does_not_consume_elements
  // tap_each 後に collect() すると全要素が取得できる

test_tap_each_lazy_evaluation
  // tap_each をチェーンしても collect() するまで副作用が実行されないことを確認
  // （collect() を呼ぶ前は副作用が 0 回）

test_tap_each_on_empty
  // 空コレクションに tap_each を適用しても副作用が呼ばれない

test_tap_each_chaining
  // where_().tap_each().order_by().collect() が正しく動作する
```

#### H2b: `tap_collect`

```rust
test_tap_collect_eager_materialization
  // atomic カウンタで tap_collect 呼び出し直後（collect 前）に副作用が実行されることを確認

test_tap_collect_receives_correct_count
  // where_ で絞った後の tap_collect が正しい件数を受け取ることを確認

test_tap_collect_does_not_lose_elements
  // tap_collect 後に collect() すると全要素が取得できる

test_tap_collect_on_empty
  // 空コレクションに tap_collect を適用すると &[] を受け取る
```

#### H2c: `pipe`

```rust
test_pipe_conditional_filter
  // if/else で異なる where_ を適用するパターン
  // flag=true → Active ユーザーのみ、flag=false → 全ユーザー

test_pipe_dynamic_sort
  // match で sort キーを切り替えるパターン
  // sort_key = "age" → age 順、sort_key = "name" → name 順

test_pipe_delegation_to_function
  // 外部関数（fn apply_filter(q: FilteredQuery<User>) -> FilteredQuery<User>）への委譲

test_pipe_identity
  // .pipe(|q| q) が元の結果と同じことを確認

test_pipe_type_change
  // pipe 内で型が変わる変換（Filtered → Sorted）が正しく動作する
```

---

## Phase 4C: クイックウィン演算子テスト

### ファイル: `tests/rinq_quick_wins_tests.rs`

#### J1: `filter_map`

```rust
test_filter_map_some_and_none
  // Some を返す要素のみ変換後の型で収集される

test_filter_map_all_some
  // 全要素が Some → 全件変換

test_filter_map_all_none
  // 全要素が None → 空の Vec

test_filter_map_empty_input
  // 空コレクション → 空の Vec

test_filter_map_type_conversion
  // &str → i32 の parse（失敗は除外）

test_filter_map_after_where
  // where_().filter_map() の組み合わせ

test_filter_map_chaining
  // filter_map 後に where_ / order_by をチェーン
```

#### J2: `map`

```rust
test_map_equals_select
  // .map(|x| x * 2).collect() と .select(|x| x * 2).collect() が同じ結果

test_map_type_conversion
  // i32 → String への変換

test_map_empty
  // 空コレクションに map を適用
```

#### J3: `IntoQuery`

```rust
test_into_query_vec
  // Vec<i32>.into_query().collect() が元の Vec と同じ

test_into_query_with_filter
  // Vec<User>.into_query().where_(...).collect()

test_into_query_empty
  // Vec::<i32>::new().into_query().collect() が空の Vec

test_into_query_interop_with_derive_queryable_from
  // #[derive(QueryableFrom)] を使った型で into_query() が動作する（Phase 4E と結合テスト）
```

#### J4: `collect_vec`

```rust
test_collect_vec_equals_collect
  // .collect_vec() と .collect::<Vec<T>>() が同じ結果

test_collect_vec_empty
  // 空コレクションに collect_vec

test_collect_vec_after_filter
  // where_().collect_vec()
```

#### J5: `step_by`

```rust
test_step_by_2
  // [1,2,3,4,5,6].step_by(2) → [1,3,5]

test_step_by_1
  // step_by(1) は全要素を返す

test_step_by_greater_than_len
  // step_by(100) で要素数が 1 未満 → 最初の 1 件のみ

test_step_by_empty
  // 空コレクションに step_by

test_step_by_zero_panics
  // step_by(0) でパニックすることを確認（#[should_panic]）

test_step_by_after_filter
  // where_().step_by(3).collect_vec()
```

#### J6: `cycle`

```rust
test_cycle_with_take
  // [A,B,C].cycle().take(7) → [A,B,C,A,B,C,A]

test_cycle_round_robin
  // .cycle().take(10) でラウンドロビン割り当てが正しく動作する

test_cycle_single_element
  // [42].cycle().take(5) → [42,42,42,42,42]

test_cycle_empty_input
  // [].cycle().take(5) → []（空コレクションのサイクルは即終了）

test_cycle_after_filter
  // where_().cycle().take(n).collect_vec()
```

---

## Phase 4D: 新演算子テスト

### ファイル: `tests/rinq_operators_tests.rs`

#### E1: `scan`

```rust
test_scan_running_product
  // [1,2,3,4,5].scan(1, |acc, x| acc * x) → [1,2,6,24,120]

test_scan_running_sum
  // [1,2,3,4,5].scan(0, |acc, x| acc + x) → [1,3,6,10,15]（running_sum の再現）

test_scan_string_accumulation
  // ["a","b","c"].scan("".to_string(), |acc, x| acc + x) → ["a","ab","abc"]

test_scan_empty
  // 空コレクションに scan(seed, f) → 空の Vec

test_scan_single_element
  // [42].scan(0, |acc, x| acc + x) → [42]

test_scan_seed_only_matters_initially
  // seed=10, scan(10, |acc, x| acc + x) on [1,2,3] → [11,13,16]

test_scan_fnmut_state_mutation
  // クロージャが状態を持つケース（FnMut の確認）
  // カウンタや外部状態に依存するクロージャが正しく動作する

test_scan_after_where
  // where_().scan(seed, f).collect_vec()
```

#### E2: `chunk_by`

```rust
test_chunk_by_basic
  // [1,1,2,2,3,1,1].chunk_by(|x| *x) → [[1,1],[2,2],[3],[1,1]]

test_chunk_by_all_same
  // 全要素同一キー → [[1,1,1,1,1]] (単一チャンク)

test_chunk_by_all_different
  // 全要素異なるキー → [[1],[2],[3],[4],[5]] (各要素が単独チャンク)

test_chunk_by_empty
  // 空コレクション → 空の Vec<Vec<T>>

test_chunk_by_single_element
  // [42] → [[42]]

test_chunk_by_after_filter
  // where_().chunk_by(...).collect()

test_chunk_by_struct_field
  // ログレベルでチャンク分け（LogEntry.level フィールド）

test_chunk_by_then_where
  // chunk_by 後に where_ で特定のチャンクのみ抽出
  // （Error バーストのみ抽出パターン）
```

#### E3: `dedup` / `dedup_by`

```rust
test_dedup_consecutive_duplicates
  // [1,1,2,2,3,1,1].dedup() → [1,2,3,1]（非連続重複は残る）

test_dedup_vs_distinct
  // dedup は非連続重複を残し、distinct は残さないことを確認

test_dedup_all_same
  // 全同値 → [single_element]

test_dedup_all_different
  // 全異なる → 元のシーケンスと同じ

test_dedup_empty
  // 空コレクション → 空

test_dedup_single_element
  // [42] → [42]

test_dedup_by_key_function
  // dedup_by(|e| e.kind) でイベント種別の連続重複を除去

test_dedup_by_struct_field
  // 構造体のフィールドをキーにした連続重複除去

test_dedup_after_order_by
  // order_by().dedup() でソート後の連続重複を除去（実質 distinct と同等になるケース）
```

#### E4: `zip_with`

```rust
test_zip_with_sum
  // [1,2,3].zip_with([10,20,30], |a, b| a + b) → [11,22,33]

test_zip_with_type_conversion
  // zip_with で (i32, f64) → String に変換

test_zip_with_left_longer
  // 左が右より長い場合 → 右の長さに切り詰め

test_zip_with_right_longer
  // 右が左より長い場合 → 左の長さに切り詰め

test_zip_with_empty_left
  // 左が空 → 空の Vec

test_zip_with_empty_right
  // 右が空 → 空の Vec

test_zip_with_after_filter
  // where_().zip_with(..., f).collect()

test_zip_with_struct_transform
  // 2つのリストを対応付けて構造体に変換
```

#### E5: `pairwise`

```rust
test_pairwise_basic
  // [1,2,3,4].pairwise() → [(1,2),(2,3),(3,4)]

test_pairwise_empty
  // 空コレクション → 空の Vec<(T,T)>

test_pairwise_single_element
  // [42] → 空の Vec<(T,T)>（ペアを作れない）

test_pairwise_two_elements
  // [1,2] → [(1,2)]

test_pairwise_diff_calculation
  // pairwise 後に select で差分を計算
  // [1.0, 3.0, 6.0].pairwise().select(|(a,b)| b-a) → [2.0, 3.0]

test_pairwise_after_filter
  // where_().pairwise().collect()

test_pairwise_after_sort
  // order_by().pairwise() — ソート後の隣接ペアを取得
```

#### E6: `unfold` / `unfold_bounded`

```rust
test_unfold_bounded_fibonacci
  // unfold_bounded((0u64,1u64), 10, |(a,b)| Some((a,(b,a+b)))).collect()
  // → 最初の 10 個のフィボナッチ数

test_unfold_bounded_countdown
  // unfold_bounded(10usize, 100, |n| if n == 0 { None } else { Some((n, n-1)) })
  // → [10,9,8,...,1]（自然終了）

test_unfold_bounded_respects_max
  // 無限生成クロージャに max=5 を指定 → 5 件で停止

test_unfold_bounded_empty
  // 初回で None を返すクロージャ → 空の Vec

test_unfold_with_take
  // unfold で無限フィボナッチ → take(20) で最初の 20 件

test_unfold_lazy_first
  // unfold(...).first() は 1 回だけクロージャが呼ばれることを atomic カウンタで確認

test_unfold_none_on_first_call
  // unfold(seed, |_| None).collect() → 空の Vec

test_unfold_returns_filtered_state
  // unfold の戻り値に where_() / order_by() がチェーンできることをコンパイルで確認

test_unfold_fnmut_accumulation
  // FnMut の状態変化を確認（外部変数のミュータブル参照を使うクロージャ）
```

#### E7: `intersperse`

```rust
test_intersperse_basic
  // ["a","b","c"].intersperse(",") → ["a",",","b",",","c"]

test_intersperse_empty
  // 空コレクション → 空の Vec

test_intersperse_single_element
  // ["only"] → ["only"]（セパレータは挿入されない）

test_intersperse_two_elements
  // ["a","b"] → ["a",",","b"]（セパレータ 1 個）

test_intersperse_result_length
  // n 要素に intersperse → 2n-1 要素

test_intersperse_with_aggregate
  // intersperse(" ").aggregate(String::new(), |mut acc, s| { acc.push_str(&s); acc })
  // でスペース区切り文字列を生成
```

#### E8: `min_max`

```rust
test_min_max_basic
  // [3,1,4,1,5,9].min_max() → Some((1, 9))

test_min_max_empty
  // 空コレクション → None

test_min_max_single_element
  // [42].min_max() → Some((42, 42))

test_min_max_all_same
  // [5,5,5,5].min_max() → Some((5, 5))

test_min_max_two_elements
  // [10,3].min_max() → Some((3, 10))

test_min_max_equals_separate_min_max
  // .min_max() の結果が .min() と .max() の組み合わせと同じ

test_min_max_after_filter
  // where_().min_max() の動作確認

test_min_max_sorted_order
  // 結果は (min, max) の順（小さい方が先）
```

---

## Phase 4E: `rinq-derive` テスト

### ファイル: `rinq-derive/tests/derive_tests.rs`

#### F1: `#[derive(Queryable)]` — アクセサ関数

```rust
test_derive_queryable_by_field_accessors
  // User::by_age / User::by_name / User::by_active が正しく動作する

test_derive_queryable_order_by
  // QueryBuilder::from(users).order_by(User::by_age).collect() が年齢順になる

test_derive_queryable_group_by
  // group_by(User::by_department) が部署ごとに分類される

test_derive_queryable_skip_attribute
  // #[queryable(skip)] フィールドのアクセサが生成されないことを確認
  // （生成されていれば user_fields::InternalCode が存在するが、存在しないこと）

test_derive_queryable_rename_attribute
  // #[queryable(rename = "price_jpy")] で by_price_jpy が生成される

test_derive_queryable_key_attribute
  // #[queryable(key)] フィールドが適切にマークされる
```

#### F1: `#[derive(Queryable)]` — 型付き述語

```rust
test_derive_queryable_age_gt
  // Age::gt(18) がクロージャとして where_ に渡せる

test_derive_queryable_age_lt
  // Age::lt(65) の動作確認

test_derive_queryable_age_between
  // Age::between(20, 40) の動作確認

test_derive_queryable_active_is_true
  // Active::is_true() の動作確認

test_derive_queryable_active_is_false
  // Active::is_false() の動作確認

test_derive_queryable_name_contains
  // Name::contains("Alice") の動作確認

test_derive_queryable_name_starts_with
  // Name::starts_with("A") の動作確認

test_derive_queryable_combined_predicates
  // Age::gt(18) と Active::is_true() を組み合わせた where_ チェーン

test_derive_queryable_hygiene_user_variable
  // ユーザーが `user` という変数名を持つ構造体に derive しても衝突しない

test_derive_queryable_hygiene_it_variable
  // `__it` という変数名のフィールドがあっても衝突しない
```

#### F2: `#[derive(QueryableFrom)]`

```rust
test_derive_queryable_from_basic
  // UserList(users).into_query().where_(...).collect() が動作する

test_derive_queryable_from_empty
  // 空の UserList から into_query() を作れる
```

---

## Phase 4F: `rinq-syntax` テスト

### ファイル: `rinq-syntax/tests/syntax_tests.rs`

#### G1: 基本構文

```rust
test_query_from_where_select
  // query! { from u in users where u.age > 18 select u.name.clone() }
  // が期待通りのデータを返す

test_query_from_only
  // select 省略形: query! { from x in numbers } が全件 collect

test_query_multiple_where
  // 複数 where 節が .where_() チェーンに展開される

test_query_order_by
  // query! { from u in users order_by u.last_name select u } が正しく並ぶ

test_query_order_by_desc
  // order_by_desc が order_by_descending に展開される

test_query_take_skip
  // query! { from x in numbers take 5 skip 2 select x } の動作確認

test_query_let_binding
  // let display = format!(...) のバインディングが正しく展開される
```

#### G4: `__macro_support` 安定 API

```rust
test_macro_support_from
  // rinq::__macro_support::from(data) が QueryBuilder::from(data) と同じ結果

test_macro_support_stability
  // __macro_support のシグネチャ変更時に既存コードが警告を受けることを確認
  // （semver 保護のスモークテスト）
```

---

## Phase 4G: `rinq-stats` 拡張テスト

### ファイル: `rinq-stats/tests/timeseries_tests.rs`

#### I1: 時系列演算子

```rust
test_ema_alpha_1_equals_raw
  // alpha=1.0 の EMA は元の値そのもの（現在値のみを反映）

test_ema_alpha_half
  // alpha=0.5 の EMA を既知値と比較して精度を確認

test_ema_empty
  // 空コレクション → 空の Vec

test_ema_single_element
  // [42.0].ema(0.5) → [42.0]（初期値は最初の要素）

test_bollinger_bands_center_is_moving_average
  // 中央バンドが移動平均と一致することを確認

test_bollinger_bands_band_width
  // 上下バンドの幅が sigma × 標準偏差と一致することを確認

test_bollinger_bands_empty
  // 空コレクション → 空の Vec

test_bollinger_bands_window_greater_than_len
  // window > len → 空または先頭から window が満たされるまで None を返す（仕様を決定して記録）
```

### ファイル: `rinq-stats/tests/outlier_tests.rs`

#### I2: 外れ値検出

```rust
test_remove_outliers_zscore_basic
  // 正規分布データに明らかな外れ値を混入させ、z-score=3.0 で除去できることを確認

test_remove_outliers_zscore_no_outliers
  // 外れ値なしのデータ → 全件返す

test_remove_outliers_zscore_empty
  // 空コレクション → 空の Vec

test_remove_outliers_zscore_threshold_zero
  // threshold=0.0 → 平均以外は全部除去（極端なケース）

test_remove_outliers_iqr_basic
  // IQR 法で外れ値を除去できることを確認

test_remove_outliers_iqr_symmetric
  // 対称分布で IQR 法が正しく動作

test_remove_outliers_iqr_empty
  // 空コレクション → 空の Vec
```

### ファイル: `rinq-stats/tests/validation_tests.rs`（追記）

#### I3: `ValidationExt` 拡張

```rust
test_validate_if_condition_false_skips_validation
  // condition が false の場合、バリデーションルールが実行されない

test_validate_if_condition_true_executes_validation
  // condition が true の場合、バリデーションルールが正常に実行される

test_validate_if_conditional_dependency
  // discount > 0 の場合のみ price > discount を検証するパターン

test_validate_with_ok
  // validate_with クロージャが Ok(()) を返す → エラーなし

test_validate_with_err
  // validate_with クロージャが Err(...) を返す → ValidationError に変換

test_validate_with_custom_error_display
  // カスタムエラー型の Display が ValidationError.message に反映される

test_validate_if_and_validate_chaining
  // validate_if と validate を連鎖させて複合検証
```

---

## Phase 4H: 最終確認テスト

### ファイル: `tests/rinq_v4_integration_tests.rs`

複数フェーズの新機能を組み合わせた End-to-End シナリオ。

```rust
test_full_pipeline_with_new_operators
  // scan + where_ + pairwise + select の組み合わせパイプライン

test_pipe_with_filter_map_and_scan
  // pipe を使った条件付きフィルタ → filter_map → scan の組み合わせ

test_into_query_with_chunk_by_and_dedup
  // Vec.into_query().dedup().chunk_by(...).collect()

test_derive_queryable_with_filter_map
  // #[derive(Queryable)] した User に filter_map で変換

test_unfold_bounded_with_collect_vec
  // unfold_bounded(...).where_(...).collect_vec()

test_type_aliases_in_complex_pipeline
  // 型エイリアスを関数シグネチャに使いつつ複数フェーズをまたぐパイプライン

test_tap_each_and_tap_collect_combined
  // tap_each でロギング + tap_collect で中間カウント の組み合わせ

test_rinq_syntax_query_with_derived_predicates
  // query! マクロと #[derive(Queryable)] の生成した predicates の組み合わせ
```

---

## Doc Test 一覧

各メソッドに付ける最低限の doc test。実装時に対応する `.rs` ファイルの `///` コメント内に追加する。

| メソッド | 基本例 | 空入力例 |
|---|---|---|
| `from_arc_cloned` | `Arc<Vec<i32>>` から FilteredQuery | — |
| `tap_each` | `log::debug!` で要素ログ | — |
| `tap_collect` | 件数ログ | — |
| `pipe` | 条件付き `where_` | — |
| `filter_map` | &str → i32 parse | 全 None → 空 Vec |
| `map` | `i32` → `String` | — |
| `collect_vec` | `.where_(...).collect_vec()` | — |
| `step_by` | 1/10 ダウンサンプリング | — |
| `cycle` | take と組み合わせ | — |
| `scan` | 累積積 | 空 → 空 |
| `chunk_by` | `[1,1,2,2,3]` → チャンク | 空 → 空 |
| `dedup` | `[1,1,2,2,3,1]` → `[1,2,3,1]` | 空 → 空 |
| `dedup_by` | イベント種別でデデュップ | — |
| `zip_with` | 要素ごとの加算 | — |
| `pairwise` | `[1,2,3,4]` → `[(1,2),(2,3),(3,4)]` | 空 → 空 |
| `unfold_bounded` | フィボナッチ数列 | 初回 None → 空 |
| `unfold` | take と組み合わせ | — |
| `intersperse` | CSV 行生成 | 空 → 空 |
| `min_max` | `[3,1,5]` → `Some((1,5))` | 空 → None |
| `IntoQuery::into_query` | `users.into_query().where_(...)` | — |

---

## 注意事項

### `Projected<U>` 状態の制約

`map` / `select` 後は `Projected<U>` 状態になり `collect()` 以外の操作はできない。
**結合テスト（E5 `pairwise`）**では以下に注意:

```rust
// OK: pairwise 後に select (pairwise は Filtered を返す)
.pairwise().select(|(a, b)| b - a)

// NG: select 後に pairwise は不可（Projected<U> 状態）
.select(|x| x * 2).pairwise()
```

### `unfold` / `cycle` の無限ループ

以下の組み合わせは `collect()` でハングするため、doc test に含めない:

```rust
// NG: take がないと無限ループ
QueryBuilder::unfold(0, |n| Some((n, n + 1))).collect()
QueryBuilder::from(vec![1, 2, 3]).cycle().collect()
```

### `scan` の `FnMut` 確認

`scan` のクロージャは `FnMut` であるため、外部変数をキャプチャしてミュータブルな操作が可能。
テストでは `FnMut` 固有の動作（カウンタのインクリメント等）を含めることで `Fn` との差を明確にする。
