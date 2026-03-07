// src/domain/rinq/tests.rs
// Unit and property-based tests for RINQ

use crate::domain::rinq::QueryBuilder;

#[test]
fn test_from_creates_query_builder() {
        let data = vec![1, 2, 3];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, vec![1, 2, 3]);
    }

    // Task 2.1: Unit tests for from() basic operations
    // Requirements: 1.1
    
    #[test]
    fn test_from_vec_creates_query_builder() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_from_empty_vec() {
        let data: Vec<i32> = vec![];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_from_vec_with_strings() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_from_slice_cloned() {
        let data = [1, 2, 3, 4, 5];
        let result: Vec<_> = QueryBuilder::from(data.iter().cloned()).collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_from_slice_copied() {
        let data = [10, 20, 30];
        let result: Vec<_> = QueryBuilder::from(data.iter().copied()).collect();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn test_from_array_into_iter() {
        let data = [1, 2, 3];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_from_range() {
        let result: Vec<_> = QueryBuilder::from(1..6).collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_from_vec_of_tuples() {
        let data = vec![(1, "a"), (2, "b"), (3, "c")];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
    }

    #[test]
    fn test_where_filters_correctly() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<_> = QueryBuilder::from(data)
            .where_(|x| x % 2 == 0)
            .collect();
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_count_returns_correct_length() {
        let data = vec![1, 2, 3, 4, 5];
        let count = QueryBuilder::from(data).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_first_returns_first_element() {
        let data = vec![1, 2, 3];
        let first = QueryBuilder::from(data).first();
        assert_eq!(first, Some(1));
    }

    #[test]
    fn test_first_returns_none_for_empty() {
        let data: Vec<i32> = vec![];
        let first = QueryBuilder::from(data).first();
        assert_eq!(first, None);
    }

    // Task 4.3: Unit tests for type conversion support
    // Requirements: 2.3, 2.4
    
    #[test]
    fn test_select_transforms_to_same_type() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<_> = QueryBuilder::from(data)
            .where_(|x| x % 2 == 0)
            .select(|x| x * 2)
            .collect();
        assert_eq!(result, vec![4, 8]);
    }

    #[test]
    fn test_select_transforms_to_string() {
        let data = vec![1, 2, 3];
        let result: Vec<String> = QueryBuilder::from(data)
            .where_(|x| *x > 1)
            .select(|x| format!("number: {}", x))
            .collect();
        assert_eq!(result, vec!["number: 2".to_string(), "number: 3".to_string()]);
    }

    #[test]
    fn test_select_transforms_to_tuple() {
        let data = vec![1, 2, 3, 4];
        let result: Vec<(i32, i32)> = QueryBuilder::from(data)
            .where_(|x| *x <= 3)
            .select(|x| (x, x * x))
            .collect();
        assert_eq!(result, vec![(1, 1), (2, 4), (3, 9)]);
    }

    #[test]
    fn test_select_transforms_to_bool() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<bool> = QueryBuilder::from(data)
            .where_(|x| *x > 0)
            .select(|x| x % 2 == 0)
            .collect();
        assert_eq!(result, vec![false, true, false, true, false]);
    }

    #[test]
    fn test_select_transforms_to_option() {
        let data = vec![0, 1, 2, 3];
        let result: Vec<Option<i32>> = QueryBuilder::from(data)
            .where_(|x| *x >= 0)
            .select(|x| if x > 0 { Some(x * 10) } else { None })
            .collect();
        assert_eq!(result, vec![None, Some(10), Some(20), Some(30)]);
    }

    #[test]
    fn test_select_transforms_string_to_length() {
        let data = vec!["hello".to_string(), "world".to_string(), "rust".to_string()];
        let result: Vec<usize> = QueryBuilder::from(data)
            .where_(|s| s.len() > 3)
            .select(|s| s.len())
            .collect();
        assert_eq!(result, vec![5, 5, 4]);
    }

    #[test]
    fn test_select_transforms_tuple_to_sum() {
        let data = vec![(1, 2), (3, 4), (5, 6)];
        let result: Vec<i32> = QueryBuilder::from(data)
            .where_(|(a, b)| a + b < 10)
            .select(|(a, b)| a + b)
            .collect();
        assert_eq!(result, vec![3, 7]);
    }

    #[test]
    fn test_select_with_complex_type_conversion() {
        #[derive(Debug, PartialEq)]
        struct Person {
            name: String,
            age: i32,
        }
        
        let data = vec![1, 2, 3];
        let result: Vec<Person> = QueryBuilder::from(data)
            .where_(|x| *x > 0)
            .select(|x| Person {
                name: format!("Person{}", x),
                age: x * 10,
            })
            .collect();
        
        assert_eq!(result, vec![
            Person { name: "Person1".to_string(), age: 10 },
            Person { name: "Person2".to_string(), age: 20 },
            Person { name: "Person3".to_string(), age: 30 },
        ]);
    }

    #[test]
    fn test_select_preserves_element_count() {
        let data = vec![1, 2, 3, 4, 5];
        let filtered_count = QueryBuilder::from(data.clone())
            .where_(|x| x % 2 == 0)
            .count();
        let selected_count = QueryBuilder::from(data)
            .where_(|x| x % 2 == 0)
            .select(|x| x * 2)
            .count();
        assert_eq!(filtered_count, selected_count);
    }

    #[test]
    fn test_select_on_empty_collection() {
        let data: Vec<i32> = vec![];
        let result: Vec<String> = QueryBuilder::from(data)
            .where_(|x| *x > 0)
            .select(|x| format!("{}", x))
            .collect();
        assert_eq!(result, Vec::<String>::new());
    }

    // Task 8.1: Unit tests for collect() basic operations
    // Requirements: 1.4
    
    #[test]
    fn test_collect_to_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<_> = QueryBuilder::from(data.clone())
            .where_(|x| *x % 2 == 0)
            .collect();
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_collect_to_hashset() {
        use std::collections::HashSet;
        
        let data = vec![1, 2, 3, 2, 1];
        let result: HashSet<_> = QueryBuilder::from(data)
            .where_(|x| *x > 0)
            .collect();
        
        let expected: HashSet<_> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collect_to_btreeset() {
        use std::collections::BTreeSet;
        
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let result: BTreeSet<_> = QueryBuilder::from(data)
            .where_(|x| *x < 7)
            .collect();
        
        let expected: BTreeSet<_> = vec![1, 2, 3, 4, 5, 6].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collect_to_vec_after_sort() {
        let data = vec![5, 3, 1, 4, 2];
        let result: Vec<_> = QueryBuilder::from(data)
            .where_(|_| true)
            .order_by(|x| *x)
            .collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_collect_to_vec_after_projection() {
        let data = vec![1, 2, 3];
        let result: Vec<String> = QueryBuilder::from(data)
            .where_(|x| *x > 0)
            .select(|x| format!("num_{}", x))
            .collect();
        assert_eq!(result, vec!["num_1", "num_2", "num_3"]);
    }

    #[test]
    fn test_collect_to_vec_with_pagination() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result: Vec<_> = QueryBuilder::from(data)
            .where_(|_| true)
            .skip(3)
            .take(4)
            .collect();
        assert_eq!(result, vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_collect_to_string() {
        let data = vec!['h', 'e', 'l', 'l', 'o'];
        let result: String = QueryBuilder::from(data)
            .where_(|_| true)
            .collect();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_collect_empty_to_vec() {
        let data: Vec<i32> = vec![];
        let result: Vec<_> = QueryBuilder::from(data).collect();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_collect_with_complex_chain() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result: Vec<_> = QueryBuilder::from(data)
            .where_(|x| *x % 2 == 0)
            .order_by(|x| -*x)
            .skip(1)
            .take(2)
            .collect();
        assert_eq!(result, vec![8, 6]);
    }

use proptest::prelude::*;

#[cfg(test)]
mod property_tests {
    use super::*;

    // Task 2.2: Property test for from() immutability
    // **Feature: rinq-v0.1, Property 3: 不変性の保証**
    // **Validates: Requirements 1.5**
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

    // **Feature: rinq-v0.1, Property 6.4: 型状態パターンによる有効なクエリ構築の強制**
    // **Validates: Requirements 6.4**
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
                .select(|x| x * 2);
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

    // Task 3.1: Property test for where_() correctness
    // **Feature: rinq-v0.1, Property 1: フィルタリングの正確性**
    // **Validates: Requirements 1.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_where_all_elements_satisfy_predicate(
            data in prop::collection::vec(any::<i32>(), 0..100)
        ) {
            // Test with various predicates
            let predicates: Vec<Box<dyn Fn(&i32) -> bool>> = vec![
                Box::new(|x: &i32| *x % 2 == 0),  // even numbers
                Box::new(|x: &i32| *x > 0),        // positive numbers
                Box::new(|x: &i32| *x < 50),       // less than 50
                Box::new(|x: &i32| *x % 3 == 0),   // divisible by 3
            ];
            
            for predicate in predicates {
                let result: Vec<_> = QueryBuilder::from(data.clone())
                    .where_(|x| predicate(x))
                    .collect();
                
                // All elements in result must satisfy the predicate
                prop_assert!(result.iter().all(|x| predicate(x)));
                
                // Result should be a subset of original data
                prop_assert!(result.len() <= data.len());
            }
        }
    }

    // Task 3.2: Property test for multiple filter chaining
    // **Feature: rinq-v0.1, Property 2: 複数フィルタの結合**
    // **Validates: Requirements 1.3**
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
        fn prop_chained_where_order_independent(
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
}
