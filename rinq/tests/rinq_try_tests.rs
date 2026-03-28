// tests/rinq_try_tests.rs
// Phase A3 integration tests for the failure-tolerant pipeline.
// Run with: cargo test --test rinq_try_tests

use rinq::{QueryBuilder, RinqError};

// ── try_select + collect_partitioned ─────────────────────────────────────────

#[test]
fn try_select_all_ok() {
    let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec!["1", "2", "3"])
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert_eq!(ok, vec![1, 2, 3]);
    assert!(err.is_empty());
}

#[test]
fn try_select_some_fail() {
    let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec!["1", "two", "3", "four", "5"])
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert_eq!(ok, vec![1, 3, 5]);
    assert_eq!(err.len(), 2);
}

#[test]
fn try_select_all_fail() {
    let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec!["a", "b", "c"])
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert!(ok.is_empty());
    assert_eq!(err.len(), 3);
}

#[test]
fn try_select_empty_collection() {
    let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec![""; 0])
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert!(ok.is_empty());
    assert!(err.is_empty());
}

#[test]
fn try_select_type_transformation() {
    // i32 → f64, always succeeds
    let (ok, err): (Vec<f64>, Vec<String>) = QueryBuilder::from(vec![1, 2, 3])
        .try_select(|x| -> Result<f64, String> { Ok(x as f64 * 1.5) })
        .collect_partitioned();
    assert_eq!(ok, vec![1.5, 3.0, 4.5]);
    assert!(err.is_empty());
}

#[test]
fn try_select_preserves_ok_order() {
    // The Ok values must appear in the same order as the original input.
    let (ok, _): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec!["10", "bad", "30", "bad2", "50"])
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert_eq!(ok, vec![10, 30, 50]);
}

// ── try_select + collect_results ─────────────────────────────────────────────

#[test]
fn try_select_collect_results_all_ok() {
    let result = QueryBuilder::from(vec!["1", "2", "3"])
        .try_select(|s| s.parse::<i32>())
        .collect_results();
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn try_select_collect_results_first_err_stops() {
    // "1" succeeds, "oops" fails — processing should stop there.
    // We verify the result is Err (exact stop point is an impl detail,
    // but the first encountered Err must be returned).
    let result = QueryBuilder::from(vec!["1", "oops", "3"])
        .try_select(|s| s.parse::<i32>())
        .collect_results();
    assert!(result.is_err());
}

#[test]
fn try_select_collect_results_empty() {
    let result = QueryBuilder::from(vec![""; 0])
        .try_select(|s| s.parse::<i32>())
        .collect_results();
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[test]
fn try_select_with_rinq_error() {
    // Mapping errors into RinqError (as shown in spec)
    let result = QueryBuilder::from(vec!["1", "bad"])
        .try_select(|s| {
            s.parse::<i32>().map_err(|e| RinqError::ExecutionError {
                message: e.to_string(),
            })
        })
        .collect_results();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RinqError::ExecutionError { .. }
    ));
}

// ── collect_results short-circuit verification ───────────────────────────────

#[test]
fn collect_results_stops_at_first_error() {
    // Use a counter to verify that items after the first Err are not processed.
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let count = Arc::new(AtomicUsize::new(0));
    let count2 = Arc::clone(&count);

    let result = QueryBuilder::from(vec!["1", "BAD", "3", "4", "5"])
        .try_select(move |s| {
            count2.fetch_add(1, Ordering::SeqCst);
            s.parse::<i32>()
        })
        .collect_results();

    assert!(result.is_err());
    // At most 2 items should have been processed (up to and including "BAD").
    assert!(count.load(Ordering::SeqCst) <= 2);
}

// ── try_where_ ───────────────────────────────────────────────────────────────

#[test]
fn try_where_all_ok_keep() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4])
        .try_where_(|x| Ok::<bool, String>(*x % 2 == 0))
        .collect_results()
        .unwrap();
    assert_eq!(result, vec![2, 4]);
}

#[test]
fn try_where_all_ok_drop() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .try_where_(|_| Ok::<bool, String>(false))
        .collect_results()
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn try_where_err_propagates() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .try_where_(|x| {
            if *x == 2 {
                Err("validation failed".to_string())
            } else {
                Ok(*x != 2)
            }
        })
        .collect_results();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "validation failed");
}

#[test]
fn try_where_partitioned_with_errors() {
    // Ok(true) → keep, Ok(false) → drop, Err → error vec
    let (ok, err): (Vec<i32>, Vec<String>) = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .try_where_(|x| {
            if *x == 3 {
                Err(format!("rejected: {x}"))
            } else {
                Ok(*x % 2 == 0)
            }
        })
        .collect_partitioned();
    assert_eq!(ok, vec![2, 4]); // 1,3,5 dropped or erred; 2,4 kept
    assert_eq!(err, vec!["rejected: 3"]);
}

#[test]
fn try_where_empty_collection() {
    let result = QueryBuilder::from(vec![0i32; 0])
        .try_where_(|_| Ok::<bool, String>(true))
        .collect_results()
        .unwrap();
    assert!(result.is_empty());
}

// ── after QueryBuilder filter / sort ─────────────────────────────────────────

#[test]
fn try_select_after_where_() {
    let (ok, _): (Vec<i32>, Vec<_>) = QueryBuilder::from(vec!["1", "2", "3", "4", "5"])
        .where_(|s| s.len() == 1)
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    let mut ok = ok;
    ok.sort();
    assert_eq!(ok, vec![1, 2, 3, 4, 5]);
}

#[test]
fn try_select_after_order_by() {
    let result = QueryBuilder::from(vec!["30", "10", "20"])
        .where_(|_| true)
        .order_by(|s| s.parse::<i32>().unwrap_or(0))
        .try_select(|s| s.parse::<i32>())
        .collect_results()
        .unwrap();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn try_where_after_where_() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
        .where_(|x| *x > 2)
        .try_where_(|x| {
            if *x > 10 {
                Err("out of range")
            } else {
                Ok(*x % 2 == 0)
            }
        })
        .collect_results()
        .unwrap();
    assert_eq!(result, vec![4, 6]);
}

// ── integration: mix select + collect flavours ────────────────────────────────

#[test]
fn try_select_collect_partitioned_error_count() {
    // 100 strings, every 10th is invalid
    let strings: Vec<String> = (1..=100)
        .map(|i| {
            if i % 10 == 0 {
                "x".to_string()
            } else {
                i.to_string()
            }
        })
        .collect();
    let (ok, err): (Vec<i32>, Vec<_>) = QueryBuilder::from(strings)
        .try_select(|s| s.parse::<i32>())
        .collect_partitioned();
    assert_eq!(ok.len(), 90);
    assert_eq!(err.len(), 10);
}
