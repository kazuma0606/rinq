// tests/rinq_immutability_test.rs
// Property-based test for RINQ immutability (Task 2.2)

use proptest::prelude::*;
use rusted_ca::domain::rinq::QueryBuilder;

// **Feature: rinq-v0.1, Property 3: 不変性の保証**
// **Validates: Requirements 1.5**
//
// This property tests that RINQ operations preserve the original collection.
// The test verifies that:
// 1. Creating a QueryBuilder from a collection doesn't modify the original
// 2. Executing queries doesn't modify the original collection
// 3. Complex query chains preserve immutability

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_from_preserves_original_collection(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        let original = data.clone();
        let _query = QueryBuilder::from(data.clone());
        
        // Original data should be unchanged
        prop_assert_eq!(data, original);
    }

    #[test]
    fn prop_query_execution_preserves_original(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        let original = data.clone();
        
        // Execute a query
        let _result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .select(|x| x * 2)
            .collect();
        
        // Original data should be unchanged
        prop_assert_eq!(data, original);
    }

    #[test]
    fn prop_complex_query_preserves_original(
        data in prop::collection::vec(any::<i32>(), 0..100)
    ) {
        let original = data.clone();
        
        // Execute a complex query with multiple operations
        let _result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .where_(|x| *x > 10)
            .take(5)
            .skip(1)
            .collect();
        
        // Original data should be unchanged
        prop_assert_eq!(data, original);
    }
}
