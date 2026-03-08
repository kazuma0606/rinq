// tests/rinq_integration_tests.rs
// Integration tests for RINQ with rusted-ca architecture

use rusted_ca::domain::rinq::{MetricsQueryBuilder, QueryBuilder};
use rusted_ca::shared::metrics::collector::MetricsCollector;
use std::sync::Arc;

// **Task 13.2: Integration test for metrics collection**
// **Validates: Requirements 11.2**

#[test]
fn test_metrics_query_builder_basic() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![1, 2, 3, 4, 5];

    let result: Vec<_> = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "basic_query".to_string(),
    )
    .where_(|x| *x > 2)
    .collect();

    assert_eq!(result, vec![3, 4, 5]);

    // Verify metrics were recorded
    let count = metrics.get("query_basic_query");
    assert_eq!(count, Some(1));
}

#[test]
fn test_metrics_query_builder_complex_chain() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];

    let result: Vec<_> = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "complex_query".to_string(),
    )
    .where_(|x| *x > 3)
    .order_by(|x| *x)
    .take(4)
    .collect();

    assert_eq!(result, vec![4, 5, 6, 7]);

    // Verify metrics were recorded
    let count = metrics.get("query_complex_query");
    assert_eq!(count, Some(1));
}

#[test]
fn test_metrics_query_builder_count_operation() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let count = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "count_query".to_string(),
    )
    .where_(|x| *x % 2 == 0)
    .count();

    assert_eq!(count, 5);

    // Verify metrics were recorded with _count suffix
    let metric_count = metrics.get("query_count_query_count");
    assert_eq!(metric_count, Some(1));
}

#[test]
fn test_metrics_query_builder_first_operation() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![10, 20, 30, 40, 50];

    let first = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "first_query".to_string(),
    )
    .where_(|x| *x > 15)
    .first();

    assert_eq!(first, Some(20));

    // Verify metrics were recorded with _first suffix
    let metric_count = metrics.get("query_first_query_first");
    assert_eq!(metric_count, Some(1));
}

#[test]
fn test_metrics_query_builder_last_operation() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![10, 20, 30, 40, 50];

    let last = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "last_query".to_string(),
    )
    .where_(|x| *x < 45)
    .last();

    assert_eq!(last, Some(40));

    // Verify metrics were recorded with _last suffix
    let metric_count = metrics.get("query_last_query_last");
    assert_eq!(metric_count, Some(1));
}

#[test]
fn test_metrics_query_builder_any_operation() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![1, 2, 3, 4, 5];

    let has_even = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "any_query".to_string(),
    )
    .any(|x| *x % 2 == 0);

    assert_eq!(has_even, true);

    // Verify metrics were recorded with _any suffix
    let metric_count = metrics.get("query_any_query_any");
    assert_eq!(metric_count, Some(1));
}

#[test]
fn test_metrics_query_builder_all_operation() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![2, 4, 6, 8, 10];

    let all_even = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "all_query".to_string(),
    )
    .all(|x| *x % 2 == 0);

    assert_eq!(all_even, true);

    // Verify metrics were recorded with _all suffix
    let metric_count = metrics.get("query_all_query_all");
    assert_eq!(metric_count, Some(1));
}

#[test]
fn test_metrics_query_builder_multiple_queries() {
    let metrics = Arc::new(MetricsCollector::new());

    // First query
    let data1 = vec![1, 2, 3, 4, 5];
    let result1: Vec<_> = MetricsQueryBuilder::new(
        QueryBuilder::from(data1),
        metrics.clone(),
        "query_1".to_string(),
    )
    .where_(|x| *x > 2)
    .collect();

    assert_eq!(result1, vec![3, 4, 5]);

    // Second query with same name
    let data2 = vec![10, 20, 30];
    let result2: Vec<_> = MetricsQueryBuilder::new(
        QueryBuilder::from(data2),
        metrics.clone(),
        "query_1".to_string(),
    )
    .where_(|x| *x < 25)
    .collect();

    assert_eq!(result2, vec![10, 20]);

    // Verify metrics were incremented correctly
    let count = metrics.get("query_query_1");
    assert_eq!(count, Some(2));
}

#[test]
fn test_metrics_collector_integration() {
    let metrics = Arc::new(MetricsCollector::new());

    // Record some metrics manually
    metrics.increment("test_metric");
    metrics.increment("test_metric");
    metrics.increment("another_metric");

    assert_eq!(metrics.get("test_metric"), Some(2));
    assert_eq!(metrics.get("another_metric"), Some(1));
    assert_eq!(metrics.get("nonexistent"), None);
}

// **Task 13.3: Unit test for error type compatibility**
// **Validates: Requirements 11.3**

#[test]
fn test_rinq_error_converts_to_application_error() {
    use rusted_ca::domain::rinq::RinqDomainError;
    use rusted_ca::shared::error::application_error::ApplicationError;

    let rinq_error = RinqDomainError::ExecutionError {
        message: "Query failed".to_string(),
    };

    let app_error: ApplicationError = rinq_error.into();

    // Should be wrapped in Domain variant
    match app_error {
        ApplicationError::Domain(domain_err) => {
            assert!(domain_err.to_string().contains("Query failed"));
        }
        _ => panic!("Expected ApplicationError::Domain"),
    }
}

#[test]
fn test_rinq_error_preserves_message() {
    use rusted_ca::domain::rinq::RinqDomainError;
    use rusted_ca::shared::error::application_error::ApplicationError;

    let test_cases = vec![
        RinqDomainError::InvalidQuery {
            message: "Invalid predicate".to_string(),
        },
        RinqDomainError::ExecutionError {
            message: "Execution failed".to_string(),
        },
        RinqDomainError::TypeMismatch {
            expected: "i32".to_string(),
            actual: "String".to_string(),
        },
    ];

    for rinq_error in test_cases {
        let original_message = rinq_error.to_string();
        let app_error: ApplicationError = rinq_error.into();

        // Message should be preserved in conversion
        assert!(app_error.to_string().contains(&original_message));
    }
}

#[test]
fn test_metrics_with_sorted_query() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![5, 2, 8, 1, 9];

    let result: Vec<_> = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "sorted_query".to_string(),
    )
    .order_by(|x| *x)
    .take(3)
    .collect();

    assert_eq!(result, vec![1, 2, 5]);

    // Verify metrics were recorded
    let count = metrics.get("query_sorted_query");
    assert_eq!(count, Some(1));
}

#[test]
fn test_metrics_with_projection() {
    let metrics = Arc::new(MetricsCollector::new());
    let data = vec![1, 2, 3, 4, 5];

    let result: Vec<_> = MetricsQueryBuilder::new(
        QueryBuilder::from(data),
        metrics.clone(),
        "projection_query".to_string(),
    )
    .where_(|x| *x > 2)
    .select(|x| x * 2)
    .collect();

    assert_eq!(result, vec![6, 8, 10]);

    // Verify metrics were recorded
    let count = metrics.get("query_projection_query");
    assert_eq!(count, Some(1));
}
