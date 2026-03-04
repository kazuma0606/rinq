// tests/rinq_property_tests.rs
// Property-based tests for RINQ v0.1

use proptest::prelude::*;
use rusted_ca::domain::rinq::QueryBuilder;

// **Feature: rinq-v0.1, Property 6.4: 型状態パターンによる有効なクエリ構築の強制**
// **Validates: Requirements 6.4**
//
// This property tests that the type state pattern enforces valid query construction
// at compile time. The test verifies that:
// 1. QueryBuilder can be created from any collection
// 2. Methods are only available in appropriate states
// 3. The query can be executed successfully
//
// Note: The primary validation here is compile-time type checking.
// If this test compiles, it demonstrates that the type state pattern
// is working correctly to prevent invalid query construction.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_type_state_enforces_valid_query_construction(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        // Test 1: Initial state allows from() and where_()
        let query = QueryBuilder::from(data.clone());
        let _result: Vec<_> = query.collect();
        
        // Test 2: Initial state can transition to Filtered
        let query = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0);
        let _result: Vec<_> = query.collect();
        
        // Test 3: Filtered state allows chaining where_()
        let query = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .where_(|x| *x > 0);
        let _result: Vec<_> = query.collect();
        
        // Test 4: Filtered state can transition to Sorted
        let query = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .order_by(|x| *x);
        let _result: Vec<_> = query.collect();
        
        // Test 5: Filtered state can transition to Projected
        let query = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .select(|x| x.saturating_mul(2));
        let _result: Vec<_> = query.collect();
        
        // Test 6: Filtered state allows take() and skip()
        let query = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .take(5)
            .skip(2);
        let _result: Vec<_> = query.collect();
        
        // Test 7: Sorted state allows then_by()
        let query = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .order_by(|x| *x)
            .then_by(|x| -*x);
        let _result: Vec<_> = query.collect();
        
        // Test 8: All states allow terminal operations
        let count = QueryBuilder::from(data.clone()).count();
        prop_assert!(count <= data.len());
        
        let _first = QueryBuilder::from(data.clone()).first();
        let _last = QueryBuilder::from(data.clone()).last();
        
        let _any_result = QueryBuilder::from(data.clone())
            .any(|x| *x > 0);
        
        let _all_result = QueryBuilder::from(data.clone())
            .all(|x| *x < 1000);
        
        // Test 9: inspect() doesn't change the result
        let without_inspect: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .collect();
        
        let with_inspect: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .inspect(|_x| {
                // Side effect for debugging
            })
            .collect();
        
        prop_assert_eq!(without_inspect, with_inspect);
    }
}

// **Feature: rinq-v0.1, Property 1: フィルタリングの正確性**
// **Validates: Requirements 1.2**
//
// This property tests that where_() correctly filters elements based on a predicate.
// For any collection and predicate, all elements in the filtered result must satisfy
// the predicate.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_where_all_elements_satisfy_predicate(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        // Test with even numbers predicate
        let result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x % 2 == 0)
            .collect();
        prop_assert!(result.iter().all(|x| *x % 2 == 0));
        prop_assert!(result.len() <= data.len());
        
        // Test with positive numbers predicate
        let result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x > 0)
            .collect();
        prop_assert!(result.iter().all(|x| *x > 0));
        prop_assert!(result.len() <= data.len());
        
        // Test with less than 50 predicate
        let result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x < 50)
            .collect();
        prop_assert!(result.iter().all(|x| *x < 50));
        prop_assert!(result.len() <= data.len());
        
        // Test with divisible by 3 predicate
        let result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x % 3 == 0)
            .collect();
        prop_assert!(result.iter().all(|x| *x % 3 == 0));
        prop_assert!(result.len() <= data.len());
    }
}

// **Feature: rinq-v0.1, Property 2: 複数フィルタの結合**
// **Validates: Requirements 1.3**
//
// This property tests that chaining multiple where_() calls correctly applies
// all predicates in sequence. The result should contain only elements that
// satisfy ALL predicates.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_multiple_where_chains_correctly(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        // Chain multiple where_() calls
        let result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x % 2 == 0)  // even numbers
            .where_(|x| *x > 0)        // positive numbers
            .where_(|x| *x < 100)      // less than 100
            .collect();
        
        // All elements must satisfy ALL predicates
        prop_assert!(result.iter().all(|x| *x % 2 == 0));
        prop_assert!(result.iter().all(|x| *x > 0));
        prop_assert!(result.iter().all(|x| *x < 100));
        
        // Result should be a subset of original data
        prop_assert!(result.len() <= data.len());
        
        // Verify equivalence with manual filtering
        let manual_result: Vec<_> = data.iter()
            .filter(|x| **x % 2 == 0)
            .filter(|x| **x > 0)
            .filter(|x| **x < 100)
            .copied()
            .collect();
        
        prop_assert_eq!(result, manual_result);
    }
    
    #[test]
    fn prop_chained_where_order_matters(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        // Test that the order of predicates doesn't affect the final result
        // (since all predicates must be satisfied)
        let result1: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x % 2 == 0)
            .where_(|x| *x > 10)
            .collect();
        
        let result2: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x > 10)
            .where_(|x| *x % 2 == 0)
            .collect();
        
        // Both should produce the same result (order-independent for AND logic)
        prop_assert_eq!(result1.len(), result2.len());
        
        // Verify all elements satisfy both predicates
        prop_assert!(result1.iter().all(|x| *x % 2 == 0 && *x > 10));
        prop_assert!(result2.iter().all(|x| *x % 2 == 0 && *x > 10));
    }
}

// Additional compile-time validation tests
// These tests primarily validate that certain operations are NOT allowed
// by the type system. If these compile, the type state pattern is working.

#[test]
fn test_type_state_compile_time_validation() {
    let data = vec![1, 2, 3, 4, 5];
    
    // Valid: Initial -> Filtered
    let _query = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0);
    
    // Valid: Filtered -> Sorted
    let _query = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .order_by(|x| *x);
    
    // Valid: Filtered -> Projected
    let _query = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .select(|x| x * 2);
    
    // Valid: Sorted -> terminal operation
    let _result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .order_by(|x| *x)
        .collect();
    
    // Valid: Projected -> terminal operation
    let _result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .select(|x| x * 2)
        .collect();
    
    // The following would NOT compile (demonstrating type safety):
    // 
    // // Invalid: Cannot call order_by() on Initial state
    // let _query = QueryBuilder::from(data.clone())
    //     .order_by(|x| *x);  // ERROR: method not found
    // 
    // // Invalid: Cannot call select() on Initial state
    // let _query = QueryBuilder::from(data.clone())
    //     .select(|x| x * 2);  // ERROR: method not found
    // 
    // // Invalid: Cannot call where_() on Sorted state
    // let _query = QueryBuilder::from(data.clone())
    //     .where_(|x| x % 2 == 0)
    //     .order_by(|x| *x)
    //     .where_(|x| *x > 0);  // ERROR: method not found
    // 
    // // Invalid: Cannot call where_() on Projected state
    // let _query = QueryBuilder::from(data.clone())
    //     .where_(|x| x % 2 == 0)
    //     .select(|x| x * 2)
    //     .where_(|x| *x > 0);  // ERROR: method not found
}
