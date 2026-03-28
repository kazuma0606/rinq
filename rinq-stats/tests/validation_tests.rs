// rinq-stats/tests/validation_tests.rs
// Phase B4 integration tests for ValidationExt.

use rinq::QueryBuilder;
use rinq_stats::{ValidationError, ValidationExt};

// ── collect_validated — all pass ──────────────────────────────────────────────

#[test]
fn all_valid_returns_ok() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_validated();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn empty_source_returns_ok_empty() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_validated();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ── collect_validated — failures ─────────────────────────────────────────────

#[test]
fn single_failure_returns_err() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1, -2, 3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_validated();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 1);
    assert_eq!(errors[0].rule, "positive");
    assert_eq!(errors[0].message, "must be positive");
}

#[test]
fn multiple_failures_all_collected() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1, -2, 3, -4, 5])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].index, 1);
    assert_eq!(errors[1].index, 3);
}

#[test]
fn all_fail_returns_err_with_all() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![-1, -2, -3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 3);
}

// ── multiple rules ─────────────────────────────────────────────────────────

#[test]
fn two_rules_both_pass() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![2, 4, 6])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x % 2 == 0, "even", "must be even")
        .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn two_rules_one_fails_first_rule() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![2, -4, 6])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x % 2 == 0, "even", "must be even")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "positive");
    assert_eq!(errors[0].index, 1);
}

#[test]
fn two_rules_one_fails_second_rule() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![2, 3, 6])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x % 2 == 0, "even", "must be even")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "even");
    assert_eq!(errors[0].index, 1);
}

#[test]
fn same_item_fails_two_rules_both_reported() {
    // -3 is negative AND odd
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![2, -3, 6])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x % 2 == 0, "even", "must be even")
        .collect_validated();
    let errors = result.unwrap_err();
    // -3 violates both rules → 2 errors, both at index 1
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|e| e.index == 1));
    let rules: Vec<&str> = errors.iter().map(|e| e.rule.as_str()).collect();
    assert!(rules.contains(&"positive"));
    assert!(rules.contains(&"even"));
}

#[test]
fn multiple_items_multiple_rules_all_violations_collected() {
    // items: [1, -2, 3, -4]
    // rule1: positive → -2 (idx 1), -4 (idx 3) fail
    // rule2: < 10 → all pass
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1, -2, 3, -4])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x < 10, "small", "must be < 10")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2);
}

// ── ValidationError fields ─────────────────────────────────────────────────

#[test]
fn error_fields_are_correct() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![0])
        .validate(|x| *x > 0, "positive", "value must be greater than zero")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors[0].rule, "positive");
    assert_eq!(errors[0].message, "value must be greater than zero");
    assert_eq!(errors[0].index, 0);
}

#[test]
fn validation_error_display() {
    let err = ValidationError {
        rule: "positive".to_string(),
        message: "must be positive".to_string(),
        index: 5,
    };
    let s = format!("{err}");
    assert!(s.contains("positive"));
    assert!(s.contains("must be positive"));
    assert!(s.contains('5'));
}

#[test]
fn validation_error_clone_and_eq() {
    let err = ValidationError {
        rule: "r".to_string(),
        message: "m".to_string(),
        index: 0,
    };
    assert_eq!(err.clone(), err);
}

// ── collect_valid ──────────────────────────────────────────────────────────

#[test]
fn collect_valid_returns_passing_items() {
    let valid: Vec<i32> = QueryBuilder::from(vec![1, -2, 3, -4, 5])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_valid();
    assert_eq!(valid, vec![1, 3, 5]);
}

#[test]
fn collect_valid_all_pass() {
    let valid: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_valid();
    assert_eq!(valid, vec![1, 2, 3]);
}

#[test]
fn collect_valid_none_pass_returns_empty() {
    let valid: Vec<i32> = QueryBuilder::from(vec![-1, -2, -3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_valid();
    assert!(valid.is_empty());
}

#[test]
fn collect_valid_two_rules_all_must_pass() {
    // item must be > 0 AND even
    let valid: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, -2])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x % 2 == 0, "even", "must be even")
        .collect_valid();
    assert_eq!(valid, vec![2, 4]);
}

// ── collect_invalid ────────────────────────────────────────────────────────

#[test]
fn collect_invalid_returns_failing_items_with_errors() {
    let invalid: Vec<(i32, Vec<ValidationError>)> = QueryBuilder::from(vec![1, -2, 3, -4])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_invalid();
    assert_eq!(invalid.len(), 2);
    assert_eq!(invalid[0].0, -2);
    assert_eq!(invalid[0].1[0].index, 1);
    assert_eq!(invalid[1].0, -4);
    assert_eq!(invalid[1].1[0].index, 3);
}

#[test]
fn collect_invalid_all_pass_returns_empty() {
    let invalid: Vec<(i32, Vec<ValidationError>)> = QueryBuilder::from(vec![1, 2, 3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .collect_invalid();
    assert!(invalid.is_empty());
}

#[test]
fn collect_invalid_item_with_multiple_rule_failures() {
    let invalid: Vec<(i32, Vec<ValidationError>)> = QueryBuilder::from(vec![-3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .validate(|x| *x % 2 == 0, "even", "must be even")
        .collect_invalid();
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0].1.len(), 2);
}

// ── integration with rinq operations ──────────────────────────────────────

#[test]
fn validate_after_filter() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6])
        .where_(|x| *x % 2 == 0) // keep evens: 2, 4, 6
        .validate(|x| *x < 5, "small", "must be < 5")
        .collect_validated();
    let errors = result.unwrap_err();
    // 6 fails (index 2 in the filtered stream)
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 2);
}

#[test]
fn validate_strings() {
    let result: Result<Vec<String>, _> = QueryBuilder::from(vec![
        "hello".to_string(),
        "".to_string(),
        "world".to_string(),
    ])
    .validate(|s| !s.is_empty(), "non_empty", "string must not be empty")
    .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 1);
}

#[test]
fn validate_struct_fields() {
    #[derive(Debug)]
    struct Record {
        name: String,
        value: f64,
    }
    let records = vec![
        Record {
            name: "a".to_string(),
            value: 1.0,
        },
        Record {
            name: "".to_string(),
            value: -1.0,
        },
        Record {
            name: "c".to_string(),
            value: 3.0,
        },
    ];
    let result: Result<Vec<Record>, _> = QueryBuilder::from(records)
        .validate(|r| !r.name.is_empty(), "name_required", "name is required")
        .validate(
            |r| r.value > 0.0,
            "positive_value",
            "value must be positive",
        )
        .collect_validated();
    let errors = result.unwrap_err();
    // index 1 fails both rules
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|e| e.index == 1));
}

#[test]
fn no_rules_returns_ok() {
    // A ValidationQueryBuilder with no rules: everything passes
    use rinq_stats::ValidationQueryBuilder;
    let vqb: rinq_stats::ValidationQueryBuilder<i32> = ValidationQueryBuilder::new(vec![1, 2, 3]);
    let result = vqb.collect_validated();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

// ── I3: validate_if ────────────────────────────────────────────────────────

#[test]
fn validate_if_condition_false_skips_rule() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![-1_i32, -2])
        .validate(|_| true, "dummy", "")
        .validate_if(false, |x| *x > 0, "positive", "must be positive")
        .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn validate_if_condition_true_applies_rule() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1_i32, -2, 3])
        .validate(|_| true, "dummy", "")
        .validate_if(true, |x| *x > 0, "positive", "must be positive")
        .collect_validated();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 1);
}

#[test]
fn validate_if_condition_false_collect_valid_unchanged() {
    let valid: Vec<i32> = QueryBuilder::from(vec![-1_i32, -2, -3])
        .validate(|_| true, "dummy", "")
        .validate_if(false, |x| *x > 0, "positive", "must be positive")
        .collect_valid();
    assert_eq!(valid, vec![-1, -2, -3]);
}

// ── I3: validate_with ──────────────────────────────────────────────────────

#[test]
fn validate_with_custom_message() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1_i32, -5, 3])
        .validate(|_| true, "dummy", "")
        .validate_with(|x| *x > 0, "positive", |x| format!("{x} is not positive"))
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "-5 is not positive");
    assert_eq!(errors[0].rule, "positive");
    assert_eq!(errors[0].index, 1);
}

#[test]
fn validate_with_all_pass() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1_i32, 2, 3])
        .validate(|_| true, "dummy", "")
        .validate_with(|x| *x > 0, "positive", |x| format!("{x} is not positive"))
        .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn validate_with_multiple_failures_correct_messages() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![-1_i32, -2])
        .validate(|_| true, "dummy", "")
        .validate_with(|x| *x > 0, "check", |x| format!("value={x}"))
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors[0].message, "value=-1");
    assert_eq!(errors[1].message, "value=-2");
}

// ── validate_range ────────────────────────────────────────────────────────────

#[test]
fn validate_range_all_in_range_ok() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![5_i32, 50, 99])
        .validate(|_| true, "dummy", "")
        .validate_range(|x| *x, 0, 100, "in_range")
        .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn validate_range_out_of_range_returns_error() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![5_i32, 150, 30])
        .validate(|_| true, "dummy", "")
        .validate_range(|x| *x, 0, 100, "in_range")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "in_range");
    assert_eq!(errors[0].index, 1);
}

#[test]
fn validate_range_boundary_values_pass() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![0_i32, 100])
        .validate(|_| true, "dummy", "")
        .validate_range(|x| *x, 0, 100, "bounds")
        .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn validate_range_error_message_includes_value_and_bounds() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![200_i32])
        .validate(|_| true, "dummy", "")
        .validate_range(|x| *x, 0, 100, "in_range")
        .collect_validated();
    let errors = result.unwrap_err();
    assert!(errors[0].message.contains("200"));
    assert!(errors[0].message.contains("0"));
    assert!(errors[0].message.contains("100"));
}

// ── validate_unique ───────────────────────────────────────────────────────────

#[test]
fn validate_unique_no_duplicates_ok() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1_i32, 2, 3, 4])
        .validate(|_| true, "dummy", "")
        .validate_unique(|x| *x, "unique_id")
        .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn validate_unique_duplicate_flagged() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1_i32, 2, 1, 3])
        .validate(|_| true, "dummy", "")
        .validate_unique(|x| *x, "unique_id")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 2);
    assert_eq!(errors[0].rule, "unique_id");
}

#[test]
fn validate_unique_multiple_duplicates() {
    let result: Result<Vec<i32>, _> = QueryBuilder::from(vec![1_i32, 1, 1])
        .validate(|_| true, "dummy", "")
        .validate_unique(|x| *x, "unique_id")
        .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2); // index 1 and 2 are duplicates
}

// ── validate_non_empty ────────────────────────────────────────────────────────

#[test]
fn validate_non_empty_all_non_empty_ok() {
    let result: Result<Vec<String>, _> =
        QueryBuilder::from(vec!["hello".to_owned(), "world".to_owned()])
            .validate(|_| true, "dummy", "")
            .validate_non_empty(|s| s.as_str(), "non_empty")
            .collect_validated();
    assert!(result.is_ok());
}

#[test]
fn validate_non_empty_empty_string_flagged() {
    let result: Result<Vec<String>, _> =
        QueryBuilder::from(vec!["ok".to_owned(), "".to_owned(), "fine".to_owned()])
            .validate(|_| true, "dummy", "")
            .validate_non_empty(|s| s.as_str(), "non_empty")
            .collect_validated();
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].index, 1);
    assert_eq!(errors[0].rule, "non_empty");
}

// ── report ────────────────────────────────────────────────────────────────────

#[test]
fn report_no_violations_returns_empty_vec() {
    let report = QueryBuilder::from(vec![1_i32, 2, 3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .report();
    assert!(report.is_empty());
}

#[test]
fn report_violations_returns_formatted_strings() {
    let report = QueryBuilder::from(vec![1_i32, -2, 3])
        .validate(|x| *x > 0, "positive", "must be positive")
        .report();
    assert_eq!(report.len(), 1);
    assert!(report[0].contains("positive"));
    assert!(report[0].contains("index 1"));
}

#[test]
fn report_format_matches_display_impl() {
    // Format: "[rule] message (index N)"
    let report = QueryBuilder::from(vec![-1_i32])
        .validate(|x| *x > 0, "positive", "must be positive")
        .report();
    assert_eq!(report[0], "[positive] must be positive (index 0)");
}
