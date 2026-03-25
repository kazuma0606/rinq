# RINQ v2.0 テスト追加タスク

チェックボックスの凡例: `[ ]` 未着手 / `[~]` 進行中 / `[x]` 完了

各タスク完了後は `cargo test` / `cargo clippy -- -D warnings` でグリーンを確認すること。

---

## T1: `then_by_descending` — テストゼロ（最優先）

現状: doc test・統合テスト・property test いずれもなし。

### doc test
- [x] `src/core/builder/sorted.rs` の `then_by_descending` に doc test を追加
  - 基本例: `order_by` → `then_by_descending` で 2 キーソート降順

### 統合テスト（`tests/core_tests.rs`）
- [x] `test_then_by_descending_basic` — 主キー昇順・副キー降順で正しく並ぶことを確認
- [x] `test_then_by_descending_all_equal_primary` — 主キーが全て同値のとき副キー降順になることを確認
- [x] `test_then_by_descending_single_element` — 要素数 1 で問題なく動作することを確認

### テスト確認
- [x] `cargo test then_by_descending` 全件通過

---

## T2: `take_while` / `skip_while` — 統合テストなし

現状: doc test のみ。3つの状態（Initial / Filtered / Sorted）すべてをカバーするテストがない。

### 統合テスト（`tests/core_tests.rs`）

#### `take_while`
- [x] `test_take_while_basic` — 条件が途中で偽になる基本ケース
- [x] `test_take_while_all_match` — 全要素が条件を満たす（全件返る）
- [x] `test_take_while_none_match` — 最初の要素から条件が偽（空を返す）
- [x] `test_take_while_empty_collection` — 空コレクションへの適用
- [x] `test_take_while_after_filter` — `where_` → `take_while` の連鎖（Filtered 状態）
- [x] `test_take_while_after_sort` — `order_by` → `take_while` の連鎖（Sorted 状態）

#### `skip_while`
- [x] `test_skip_while_basic` — 条件が途中で偽になる基本ケース
- [x] `test_skip_while_all_match` — 全要素が条件を満たす（空を返す）
- [x] `test_skip_while_none_match` — 最初の要素から条件が偽（全件返る）
- [x] `test_skip_while_empty_collection` — 空コレクションへの適用
- [x] `test_skip_while_after_filter` — Filtered 状態での動作
- [x] `test_skip_while_after_sort` — Sorted 状態での動作

### テスト確認
- [x] `cargo test take_while` 全件通過
- [x] `cargo test skip_while` 全件通過

---

## T3: `repeat` / `empty` — 統合テストなし

現状: doc test のみ。生成演算子として単体で使われることが多いため独立したテストが必要。

### 統合テスト（`tests/core_tests.rs`）

#### `repeat`
- [x] `test_repeat_basic` — 指定した値が N 回繰り返されることを確認
- [x] `test_repeat_zero_count` — count=0 のとき空になることを確認
- [x] `test_repeat_count_one` — count=1 のとき要素が 1 件のみ

#### `empty`
- [x] `test_empty_collects_to_empty_vec` — `collect::<Vec<_>>()` が空ベクタを返すことを確認
- [x] `test_empty_count_is_zero` — `count()` が 0 を返すことを確認
- [x] `test_empty_chained` — `empty()` → `where_` → `collect` でも空のままであることを確認

### テスト確認
- [x] `cargo test test_repeat` 全件通過
- [x] `cargo test test_empty` 全件通過

---

## T4: `flat_map` — 統合テストが 1 件のみ

現状: 基本的なハッピーパスのみ。エッジケースがカバーされていない。

### 統合テスト（`tests/core_tests.rs`）
- [x] `test_flat_map_empty_outer` — 外側が空コレクションのとき空を返す
- [x] `test_flat_map_empty_inner` — 内側が全て空のとき空を返す
- [x] `test_flat_map_mixed_empty_and_nonempty` — 空・非空が混在するネスト
- [x] `test_flat_map_type_transformation` — `T → U`（型変換あり）のケース
- [x] `test_flat_map_after_filter` — Filtered 状態からの `flat_map`
- [x] `test_flat_map_preserves_order` — 出力順序が外側→内側の順を保つことを確認

### テスト確認
- [x] `cargo test flat_map` 全件通過

---

## T5: `select` / `inspect` / `any` / `all` — doc test なし

現状: 統合テスト・property test はあるが、doc test がない。

### doc test 追加（各ソースファイル内）
- [x] `src/core/builder/filtered.rs` の `select` に doc test を追加
  - 例: `vec![1,2,3].where_(...).select(|x| x * 2).collect()` → `[2,4,6]`
- [x] `src/core/builder/initial.rs` の `inspect` に doc test を追加
  - 例: デバッグ出力を確認する基本例
- [x] `src/core/builder/shared.rs` の `any` に doc test を追加
  - 例: 条件を満たす要素がある場合・ない場合
- [x] `src/core/builder/shared.rs` の `all` に doc test を追加
  - 例: 全要素が条件を満たす場合・満たさない場合

### テスト確認
- [x] `cargo test --doc` 全件通過

---

## T6: property test 追加（M2〜M4 新規オペレータ）

現状: M2〜M4 で追加した全オペレータに property test がない。
追加先: `tests/rinq_property_tests.rs`

> **注意**: proptest クロージャ内では飽和演算（`saturating_add` 等）を使用してオーバーフローを防ぐこと。

### M2 オペレータ
- [x] `prop_take_while_subset_of_original` — `take_while` の結果は元コレクションの prefix であることを検証
- [x] `prop_skip_while_concat_take_while` — `skip_while(p)` と `take_while(p)` を concat すると元コレクションと等価
- [x] `prop_contains_iff_in_vec` — `contains(&v)` の結果が `iter().any(|x| x == v)` と一致する
- [x] `prop_first_or_default_never_panics` — 空・非空を問わず `first_or_default()` がパニックしない
- [x] `prop_single_returns_err_on_multiple` — 要素数 2 以上のとき `single()` が常に `Err` を返す
- [x] `prop_single_or_default_ok_on_empty` — 空のとき `single_or_default()` が `Ok(T::default())` を返す

### M3 オペレータ
- [x] `prop_aggregate_matches_fold` — `aggregate(seed, f)` の結果が手動 `fold` と一致する
- [x] `prop_concat_length` — `concat(other)` の件数が `self.count() + other.len()` と等しい
- [x] `prop_union_idempotent` — `union(same_collection)` は `distinct()` と等価
- [x] `prop_intersect_subset` — `intersect` の結果が self・other の両方のサブセットであることを検証
- [x] `prop_except_disjoint` — `except(other)` の結果と other が共通要素を持たないことを検証
- [x] `prop_union_intersect_except_partition` — `union = intersect + except(A) + except(B)` の集合恒等式
- [x] `prop_element_at_matches_nth` — `element_at(i)` が `collect()[i]` と等しい（範囲内）

### M4 オペレータ
- [x] `prop_range_length` — `QueryBuilder::range(0..n).count() == n`
- [x] `prop_repeat_all_equal` — `repeat(v, n)` の全要素が `v` に等しい
- [x] `prop_empty_count_zero` — `empty::<T>().count() == 0`

### テスト確認
- [x] `cargo test --test rinq_property_tests` 全件通過

---

## T7: property test 追加（v0.2 既存オペレータの補強）

現状: v0.2 で追加された多くのオペレータに property test がない。

### 数値集計
- [x] `prop_sum_equals_iter_sum` — `QueryBuilder::from(v).sum()` と `v.iter().sum()` が等しい
- [x] `prop_min_leq_all_elements` — `min()` の結果がコレクション内のすべての要素以下
- [x] `prop_max_geq_all_elements` — `max()` の結果がコレクション内のすべての要素以上

### 集合演算
- [x] `prop_distinct_no_duplicates` — `distinct()` の結果に重複がない
- [x] `prop_distinct_by_key_unique` — `distinct_by(key)` の結果でキーが一意
- [x] `prop_reverse_involution` — `reverse().reverse()` が元のコレクションと等価

### チャンク / ウィンドウ
- [x] `prop_chunk_total_elements` — `chunk(n)` の全チャンクの要素数の合計が元の件数と等しい
- [x] `prop_window_count` — サイズ `w`、長さ `n` のコレクションに対して `window(w).count() == n - w + 1`（`n >= w` のとき）

### グループ化
- [x] `prop_group_by_covers_all` — `group_by` の全グループの要素数の合計が元の件数と等しい
- [x] `prop_partition_sum_equals_total` — `partition` の両側の件数合計が元の件数と等しい

### テスト確認
- [x] `cargo test --test rinq_property_tests` 全件通過

---

## 全体完了チェック

- [x] T1 完了（`then_by_descending` テストゼロ解消）
- [x] T2 完了（`take_while` / `skip_while` 統合テスト）
- [x] T3 完了（`repeat` / `empty` 統合テスト）
- [x] T4 完了（`flat_map` エッジケース）
- [x] T5 完了（`select` / `inspect` / `any` / `all` doc test）
- [x] T6 完了（M2〜M4 property tests）
- [x] T7 完了（v0.2 property tests 補強）
- [x] `cargo test` 全件通過（テスト数を記録: 350件）
- [x] `cargo test --doc` 全件通過
- [x] `cargo clippy -- -D warnings` ゼロ
