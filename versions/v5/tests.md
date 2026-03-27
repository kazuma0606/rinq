# RINQ v5.0 テスト計画

**作成日**: 2026-03-28

---

## 概要

本ドキュメントは v5.0 で追加・補完するテストの仕様を記述する。
既存テストの変更なし。新規テストのみ追記する。

---

## テストファイル一覧

| ファイル | 区分 | 内容 |
|---|---|---|
| `rinq/tests/rinq_v5_tests.rs` | 統合テスト | 5B・5D・5E・5F の新演算子 |
| `rinq-stats/tests/transform_tests.rs` | 統合テスト | 5G-1: normalize/standardize/weighted_average 等 |
| `rinq-stats/tests/timeseries_tests.rs` | 統合テスト | 5G-2 の追加テストを既存ファイルに追記 |
| `rinq-stats/tests/outlier_tests.rs` | 統合テスト | 5G-3 の追加テストを既存ファイルに追記 |
| `rinq-stats/tests/validation_tests.rs` | 統合テスト | 5G-4 の追加テストを既存ファイルに追記 |
| `rinq-syntax/tests/syntax_tests.rs` | 統合テスト | 5F: join 節のテストを既存ファイルに追記 |

---

## Phase 5B テスト仕様

### `rinq/tests/rinq_v5_tests.rs`

#### 5B-1: 未テスト演算子

```rust
// --- tap_each ---

#[test]
fn test_tap_each_counts_elements() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .tap_each(move |_| { c.fetch_add(1, Ordering::SeqCst); })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[test]
fn test_tap_each_empty_no_side_effect() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let _result = QueryBuilder::<i32, _>::from(vec![])
        .tap_each(move |_| { c.fetch_add(1, Ordering::SeqCst); })
        .collect::<Vec<_>>();
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn test_tap_each_in_chain() {
    // tap_each の前後で where_ が動作する
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|&x| x % 2 == 0)
        .tap_each(move |_| { c.fetch_add(1, Ordering::SeqCst); })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![2, 4]);
    assert_eq!(counter.load(Ordering::SeqCst), 2); // フィルタ後の要素数
}

// --- tap_collect ---

#[test]
fn test_tap_collect_receives_all_elements() {
    let captured = Arc::new(Mutex::new(Vec::<i32>::new()));
    let cap = captured.clone();
    let result = QueryBuilder::from(vec![1, 2, 3])
        .tap_collect(move |items| {
            *cap.lock().unwrap() = items.to_vec();
        })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3]);
    assert_eq!(*captured.lock().unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_tap_collect_empty() {
    let called = Arc::new(AtomicBool::new(false));
    let c = called.clone();
    let result = QueryBuilder::<i32, _>::from(vec![])
        .tap_collect(move |items| {
            c.store(true, Ordering::SeqCst);
            assert!(items.is_empty());
        })
        .collect::<Vec<_>>();
    assert!(result.is_empty());
    assert!(called.load(Ordering::SeqCst)); // 空でも呼ばれる
}

#[test]
fn test_tap_collect_does_not_change_elements() {
    // tap_collect は要素を変更しない
    let result = QueryBuilder::from(vec![10, 20, 30])
        .tap_collect(|_| {})
        .collect::<Vec<_>>();
    assert_eq!(result, vec![10, 20, 30]);
}

// --- pipe ---

fn add_filter(q: QueryBuilder<i32, rinq::Filtered>) -> QueryBuilder<i32, rinq::Filtered> {
    q.where_(|&x| x > 3)
}

#[test]
fn test_pipe_delegates_to_function() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|&x| x > 0) // Filtered state に遷移
        .pipe(add_filter)
        .collect_vec();
    assert_eq!(result, vec![4, 5]);
}

#[test]
fn test_pipe_identity() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .where_(|&x| x > 0)
        .pipe(|q| q)
        .collect_vec();
    assert_eq!(result, vec![1, 2, 3]);
}

// --- cycle ---

#[test]
fn test_cycle_repeats_elements() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .cycle()
        .take(7)
        .collect_vec();
    assert_eq!(result, vec![1, 2, 3, 1, 2, 3, 1]);
}

#[test]
fn test_cycle_empty_stays_empty() {
    let result = QueryBuilder::<i32, _>::from(vec![])
        .cycle()
        .take(10)
        .collect_vec();
    assert!(result.is_empty());
}

// --- step_by ---

#[test]
fn test_step_by_1_returns_all() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .step_by(1)
        .collect_vec();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_step_by_2_returns_every_other() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
        .step_by(2)
        .collect_vec();
    assert_eq!(result, vec![1, 3, 5]);
}

#[test]
fn test_step_by_3() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9])
        .step_by(3)
        .collect_vec();
    assert_eq!(result, vec![1, 4, 7]);
}

#[test]
#[should_panic]
fn test_step_by_0_panics() {
    let _: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .step_by(0)
        .collect_vec();
}

// --- map (select alias) ---

#[test]
fn test_map_equals_select() {
    let via_map = QueryBuilder::from(vec![1, 2, 3])
        .where_(|&x| x > 0)
        .map(|x| x * 2)
        .collect::<Vec<_>>();
    let via_select = QueryBuilder::from(vec![1, 2, 3])
        .where_(|&x| x > 0)
        .select(|x| x * 2)
        .collect::<Vec<_>>();
    assert_eq!(via_map, via_select);
}

// --- collect_vec ---

#[test]
fn test_collect_vec_equals_collect() {
    let via_collect_vec = QueryBuilder::from(vec![1, 2, 3])
        .where_(|&x| x > 1)
        .collect_vec();
    let via_collect: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .where_(|&x| x > 1)
        .collect();
    assert_eq!(via_collect_vec, via_collect);
}
```

#### 5B-3: エッジケース補強

```rust
// --- pairwise 境界 ---

#[test]
fn test_pairwise_empty() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![])
        .pairwise()
        .collect();
    assert!(result.is_empty());
}

#[test]
fn test_pairwise_one_element() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![1])
        .pairwise()
        .collect();
    assert!(result.is_empty());
}

#[test]
fn test_pairwise_two_elements() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2])
        .pairwise()
        .collect();
    assert_eq!(result, vec![(1, 2)]);
}

// --- intersperse 境界 ---

#[test]
fn test_intersperse_empty() {
    let result: Vec<i32> = QueryBuilder::from(vec![])
        .intersperse(0)
        .collect();
    assert!(result.is_empty());
}

#[test]
fn test_intersperse_one_element() {
    let result: Vec<i32> = QueryBuilder::from(vec![42])
        .intersperse(0)
        .collect();
    assert_eq!(result, vec![42]);
}

// --- dedup_by ---

#[test]
fn test_dedup_by_all_same_returns_one() {
    let result = QueryBuilder::from(vec![5, 5, 5, 5])
        .dedup_by(|&x| x)
        .collect_vec();
    assert_eq!(result, vec![5]);
}

#[test]
fn test_dedup_by_all_different_returns_all() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4])
        .dedup_by(|&x| x)
        .collect_vec();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn test_dedup_by_tuple_key() {
    // (group, value) の group でデdup
    let data = vec![(1, 'a'), (1, 'b'), (2, 'c'), (2, 'd'), (1, 'e')];
    let result = QueryBuilder::from(data)
        .dedup_by(|&(g, _)| g)
        .collect_vec();
    assert_eq!(result, vec![(1, 'a'), (2, 'c'), (1, 'e')]);
}

// --- unfold early termination ---

#[test]
fn test_unfold_take_terminates() {
    let result = QueryBuilder::<u64, _>::unfold(0u64, |s| Some((s, s + 1)))
        .take(5)
        .collect_vec();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}
```

---

## Phase 5D テスト仕様

```rust
// --- for_each ---

#[test]
fn test_for_each_visits_all_elements() {
    let sum = Arc::new(AtomicI32::new(0));
    let s = sum.clone();
    QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .for_each(move |x| { s.fetch_add(x, Ordering::SeqCst); });
    assert_eq!(sum.load(Ordering::SeqCst), 15);
}

// --- to_sorted_vec ---

#[test]
fn test_to_sorted_vec_equals_order_by_collect() {
    let via_short = QueryBuilder::from(vec![3, 1, 4, 1, 5])
        .where_(|_| true)
        .to_sorted_vec(|&x| x);
    let via_long: Vec<i32> = QueryBuilder::from(vec![3, 1, 4, 1, 5])
        .where_(|_| true)
        .order_by(|x| *x)
        .collect();
    assert_eq!(via_short, via_long);
}

// --- take_last / skip_last ---

#[test]
fn test_take_last_3() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|_| true)
        .take_last(3);
    assert_eq!(result, vec![3, 4, 5]);
}

#[test]
fn test_take_last_0_returns_empty() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .take_last(0);
    assert!(result.is_empty());
}

#[test]
fn test_take_last_exceeds_len_returns_all() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .take_last(100);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_skip_last_2() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|_| true)
        .skip_last(2);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_skip_last_0_returns_all() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .skip_last(0);
    assert_eq!(result, vec![1, 2, 3]);
}

// --- count_by ---

#[test]
fn test_count_by_even_numbers() {
    let count = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
        .where_(|_| true)
        .count_by(|&x| x % 2 == 0);
    assert_eq!(count, 3);
}

#[test]
fn test_count_by_no_match_returns_zero() {
    let count = QueryBuilder::from(vec![1, 3, 5])
        .where_(|_| true)
        .count_by(|&x| x % 2 == 0);
    assert_eq!(count, 0);
}

// --- reduce ---

#[test]
fn test_reduce_sum() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|_| true)
        .reduce(|a, b| a + b);
    assert_eq!(result, Some(15));
}

#[test]
fn test_reduce_empty_returns_none() {
    let result = QueryBuilder::<i32, _>::from(vec![])
        .where_(|_| true)
        .reduce(|a, b| a + b);
    assert_eq!(result, None);
}

// --- all_unique ---

#[test]
fn test_all_unique_true() {
    assert!(QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|_| true)
        .all_unique());
}

#[test]
fn test_all_unique_false_with_duplicate() {
    assert!(!QueryBuilder::from(vec![1, 2, 3, 2, 5])
        .where_(|_| true)
        .all_unique());
}

// --- none ---

#[test]
fn test_none_when_no_match() {
    assert!(QueryBuilder::from(vec![1, 3, 5])
        .where_(|_| true)
        .none(|&x| x % 2 == 0));
}

#[test]
fn test_none_false_when_match_exists() {
    assert!(!QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .none(|&x| x % 2 == 0));
}
```

---

## Phase 5E テスト仕様

```rust
// --- frequencies ---

#[test]
fn test_frequencies_counts_occurrences() {
    let freq = QueryBuilder::from(vec!["a", "b", "a", "c", "a", "b"])
        .where_(|_| true)
        .frequencies();
    assert_eq!(freq[&"a"], 3);
    assert_eq!(freq[&"b"], 2);
    assert_eq!(freq[&"c"], 1);
}

#[test]
fn test_frequencies_empty() {
    let freq = QueryBuilder::<&str, _>::from(vec![])
        .where_(|_| true)
        .frequencies();
    assert!(freq.is_empty());
}

// --- flatten ---

#[test]
fn test_flatten_nested_vecs() {
    let result: Vec<i32> = QueryBuilder::from(vec![vec![1, 2], vec![3], vec![4, 5]])
        .where_(|_| true)
        .flatten()
        .collect();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_flatten_empty_inner_vecs() {
    let result: Vec<i32> = QueryBuilder::from(vec![vec![], vec![1], vec![], vec![2, 3]])
        .where_(|_| true)
        .flatten()
        .collect();
    assert_eq!(result, vec![1, 2, 3]);
}

// --- position ---

#[test]
fn test_position_found() {
    let pos = QueryBuilder::from(vec![10, 20, 30, 40])
        .where_(|_| true)
        .position(|&x| x == 30);
    assert_eq!(pos, Some(2));
}

#[test]
fn test_position_not_found() {
    let pos = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .position(|&x| x == 99);
    assert_eq!(pos, None);
}

// --- find ---

#[test]
fn test_find_equals_first() {
    let via_find = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|_| true)
        .find(|&&x| x > 3);
    // first は where_ + first_or_default パターンを想定
    assert_eq!(via_find, Some(4));
}

// --- index_of ---

#[test]
fn test_index_of_found() {
    let idx = QueryBuilder::from(vec![10, 20, 30, 40])
        .where_(|_| true)
        .index_of(&30);
    assert_eq!(idx, Some(2));
}

#[test]
fn test_index_of_not_found() {
    let idx = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .index_of(&99);
    assert_eq!(idx, None);
}

// --- tee ---

#[test]
fn test_tee_produces_two_equal_vecs() {
    let (a, b) = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .tee();
    assert_eq!(a, b);
}

#[test]
fn test_tee_independent_clones() {
    let (mut a, b) = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .tee();
    a.push(99);
    assert_eq!(b, vec![1, 2, 3]); // b は影響を受けない
}
```

---

## Phase 5F テスト仕様

```rust
// --- inner_join ---

#[test]
fn test_inner_join_all_match() {
    let orders = vec![(1, "apple"), (2, "banana")];
    let customers = vec![(1, "Alice"), (2, "Bob")];
    let mut result = QueryBuilder::from(orders)
        .where_(|_| true)
        .inner_join(
            QueryBuilder::from(customers).where_(|_| true),
            |&(id, _)| id,
            |&(id, _)| id,
        )
        .collect_vec();
    result.sort_by_key(|((id, _), _)| *id);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0 .0, 1);
}

#[test]
fn test_inner_join_partial_match() {
    let left = vec![(1, "a"), (2, "b"), (3, "c")];
    let right = vec![(1, "x"), (3, "z")];
    let result = QueryBuilder::from(left)
        .where_(|_| true)
        .inner_join(
            QueryBuilder::from(right).where_(|_| true),
            |&(id, _)| id,
            |&(id, _)| id,
        )
        .collect_vec();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_inner_join_right_empty() {
    let left = vec![(1, "a")];
    let right: Vec<(i32, &str)> = vec![];
    let result = QueryBuilder::from(left)
        .where_(|_| true)
        .inner_join(
            QueryBuilder::from(right).where_(|_| true),
            |&(id, _)| id,
            |&(id, _)| id,
        )
        .collect_vec();
    assert!(result.is_empty());
}

// --- left_join ---

#[test]
fn test_left_join_all_match() {
    let left = vec![(1, "a"), (2, "b")];
    let right = vec![(1, "x"), (2, "y")];
    let result = QueryBuilder::from(left)
        .where_(|_| true)
        .left_join(
            QueryBuilder::from(right).where_(|_| true),
            |&(id, _)| id,
            |&(id, _)| id,
        )
        .collect_vec();
    assert!(result.iter().all(|(_, r)| r.is_some()));
}

#[test]
fn test_left_join_partial_match() {
    let left = vec![(1, "a"), (2, "b"), (3, "c")];
    let right = vec![(1, "x")];
    let result = QueryBuilder::from(left)
        .where_(|_| true)
        .left_join(
            QueryBuilder::from(right).where_(|_| true),
            |&(id, _)| id,
            |&(id, _)| id,
        )
        .collect_vec();
    assert_eq!(result.len(), 3);
    let nones = result.iter().filter(|(_, r)| r.is_none()).count();
    assert_eq!(nones, 2);
}

// --- cross_join ---

#[test]
fn test_cross_join_produces_cartesian_product() {
    let result = QueryBuilder::from(vec![1, 2])
        .where_(|_| true)
        .cross_join(QueryBuilder::from(vec!["a", "b", "c"]).where_(|_| true))
        .collect_vec();
    assert_eq!(result.len(), 6); // 2 × 3
}

#[test]
fn test_cross_join_one_empty() {
    let result = QueryBuilder::from(vec![1, 2])
        .where_(|_| true)
        .cross_join(QueryBuilder::<i32, _>::from(vec![]).where_(|_| true))
        .collect_vec();
    assert!(result.is_empty());
}

// --- rinq-syntax join 節 ---
// rinq-syntax/tests/syntax_tests.rs に追記

#[test]
fn test_query_macro_join_expands_to_inner_join() {
    // このテストは rinq-syntax 側で query! マクロが inner_join に展開されることを確認する
    // 展開後の Vec<(_, _)> の件数と内容を確認
}
```

---

## Phase 5G テスト仕様

### `rinq-stats/tests/transform_tests.rs`

```rust
// --- normalize ---

#[test]
fn test_normalize_range_is_0_to_1() {
    use rinq_stats::NormalizeExt;
    let result = QueryBuilder::from(vec![0.0, 5.0, 10.0])
        .where_(|_| true)
        .normalize();
    assert!((result[0] - 0.0).abs() < 1e-10);
    assert!((result[1] - 0.5).abs() < 1e-10);
    assert!((result[2] - 1.0).abs() < 1e-10);
}

#[test]
fn test_normalize_all_same_returns_zeros() {
    use rinq_stats::NormalizeExt;
    let result = QueryBuilder::from(vec![3.0, 3.0, 3.0])
        .where_(|_| true)
        .normalize();
    assert!(result.iter().all(|&x| x == 0.0));
}

// --- standardize ---

#[test]
fn test_standardize_mean_is_zero() {
    use rinq_stats::NormalizeExt;
    let result = QueryBuilder::from(vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .where_(|_| true)
        .standardize();
    let mean: f64 = result.iter().sum::<f64>() / result.len() as f64;
    assert!(mean.abs() < 1e-10);
}

#[test]
fn test_standardize_all_same_returns_zeros() {
    use rinq_stats::NormalizeExt;
    let result = QueryBuilder::from(vec![7.0, 7.0, 7.0])
        .where_(|_| true)
        .standardize();
    assert!(result.iter().all(|&x| x == 0.0));
}

// --- weighted_average ---

#[test]
fn test_weighted_average_uniform_weights() {
    use rinq_stats::NormalizeExt;
    // 均等な重みは通常の平均と一致する
    let data = vec![(1.0f64, 1.0f64), (2.0, 1.0), (3.0, 1.0)];
    let result = QueryBuilder::from(data)
        .where_(|_| true)
        .weighted_average(|&(v, w)| (v, w));
    assert!((result - 2.0).abs() < 1e-10);
}

// --- percentile_filter ---

#[test]
fn test_percentile_filter_removes_extremes() {
    use rinq_stats::NormalizeExt;
    let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let result = QueryBuilder::from(data)
        .where_(|_| true)
        .percentile_filter(10.0, 90.0)
        .collect_vec();
    assert!(!result.contains(&1.0));
    assert!(!result.contains(&100.0));
}
```

### `rinq-stats/tests/validation_tests.rs` 追記

```rust
// --- validate_range ---

#[test]
fn test_validate_range_valid() {
    let vq = ValidationQueryBuilder::new(vec![5u32, 10, 15])
        .validate_range(|&x| x as f64, 0.0, 20.0, "in_range");
    let (valid, errors) = vq.collect();
    assert_eq!(valid.len(), 3);
    assert!(errors.is_empty());
}

#[test]
fn test_validate_range_invalid() {
    let vq = ValidationQueryBuilder::new(vec![5u32, 25, 15])
        .validate_range(|&x| x as f64, 0.0, 20.0, "in_range");
    let (_, errors) = vq.collect();
    assert_eq!(errors.len(), 1);
}

// --- validate_unique ---

#[test]
fn test_validate_unique_all_unique() {
    let vq = ValidationQueryBuilder::new(vec![1u32, 2, 3, 4])
        .validate_unique(|&x| x, "unique_id");
    let (valid, errors) = vq.collect();
    assert_eq!(valid.len(), 4);
    assert!(errors.is_empty());
}

#[test]
fn test_validate_unique_with_duplicate() {
    let vq = ValidationQueryBuilder::new(vec![1u32, 2, 1, 3])
        .validate_unique(|&x| x, "unique_id");
    let (_, errors) = vq.collect();
    assert!(!errors.is_empty());
}

// --- validate_non_empty ---

#[test]
fn test_validate_non_empty_passes_with_data() {
    let vq = ValidationQueryBuilder::new(vec![1u32, 2, 3])
        .validate_non_empty("not_empty");
    let (valid, errors) = vq.collect();
    assert_eq!(valid.len(), 3);
    assert!(errors.is_empty());
}

// --- report ---

#[test]
fn test_report_returns_string_list() {
    let vq = ValidationQueryBuilder::new(vec![1i32, -1, 2])
        .validate(|&x| x > 0, "positive", "must be positive");
    let report = vq.report();
    assert_eq!(report.len(), 1);
    assert!(report[0].contains("positive"));
}
```

---

## テスト命名規則

| 規則 | 例 |
|---|---|
| `test_<演算子名>_<期待動作>` | `test_inner_join_all_match` |
| エラー系は `_returns_error` / `_returns_none` / `_returns_empty` | `test_reduce_empty_returns_none` |
| パニック系は `_panics` + `#[should_panic]` | `test_step_by_0_panics` |
| Alias 等価確認は `_equals_<original>` | `test_map_equals_select` |

## テスト作成ガイドライン

1. **Arrange-Act-Assert** 構造を維持する
2. `proptest` は v5 では使わない（実装速度優先）。必要なら v6 で追加
3. 数値比較は浮動小数点を扱う場合 `approx::assert_abs_diff_eq!` または `(a - b).abs() < 1e-10` を使う
4. `Arc<AtomicXxx>` / `Arc<Mutex<Vec<_>>>` を使った副作用確認は既存パターンを踏襲する
5. `collect_vec()` は `collect::<Vec<_>>()` より簡潔なので積極的に使う
6. `where_(|_| true)` は `Filtered` 状態に遷移するための定型句として使う（Initial 状態に存在しないメソッドを呼ぶ際に必要）
