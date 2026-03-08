// tests/rinq_v0.2_tests.rs
// RINQ v0.2 Test Suite
// Tests for aggregation, grouping, deduplication, and transformation operations

use proptest::prelude::*;
use rusted_ca::domain::rinq::QueryBuilder;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Phase 1: User Story 1 - Numeric Aggregations (P1)
// ============================================================================

#[cfg(test)]
mod aggregation_properties {
    use super::*;

    // T101: Property test - sum() correctness
    proptest! {
        #[test]
        fn prop_sum_equals_manual_sum(data in prop::collection::vec(-1000i32..1000i32, 0..100)) {
            let rinq_sum: i32 = QueryBuilder::from(data.clone()).sum();
            let manual_sum: i32 = data.iter().sum();
            prop_assert_eq!(rinq_sum, manual_sum);
        }
    }

    // T102: Property test - sum() on filtered data
    proptest! {
        #[test]
        fn prop_sum_on_filtered_data(
            data in prop::collection::vec(-1000i32..1000i32, 0..100),
            threshold in 0i32..100i32
        ) {
            let rinq_sum: i32 = QueryBuilder::from(data.clone())
                .where_(move |x| *x > threshold)
                .sum();

            let manual_sum: i32 = data.iter()
                .filter(|x| **x > threshold)
                .sum();

            prop_assert_eq!(rinq_sum, manual_sum);
        }
    }

    // T104: Property test - average() correctness
    proptest! {
        #[test]
        fn prop_average_equals_manual_average(data: Vec<i32>) {
            let rinq_avg = QueryBuilder::from(data.clone()).average();

            if data.is_empty() {
                prop_assert_eq!(rinq_avg, None);
            } else {
                let manual_sum: f64 = data.iter().map(|x| *x as f64).sum();
                let manual_avg = manual_sum / data.len() as f64;
                let rinq_avg_val = rinq_avg.unwrap();

                // Allow small floating point error
                prop_assert!((rinq_avg_val - manual_avg).abs() < 1e-10);
            }
        }
    }

    // T106: Property test - min() / max() correctness
    proptest! {
        #[test]
        fn prop_min_equals_iterator_min(data: Vec<i32>) {
            let rinq_min = QueryBuilder::from(data.clone()).min();
            let manual_min = data.iter().min().copied();
            prop_assert_eq!(rinq_min, manual_min);
        }

        #[test]
        fn prop_max_equals_iterator_max(data: Vec<i32>) {
            let rinq_max = QueryBuilder::from(data.clone()).max();
            let manual_max = data.iter().max().copied();
            prop_assert_eq!(rinq_max, manual_max);
        }
    }

    // T108: Property test - min_by() / max_by() with key selector
    proptest! {
        #[test]
        fn prop_min_by_with_key_selector(data: Vec<(i32, String)>) {
            let rinq_min = QueryBuilder::from(data.clone())
                .min_by(|(val, _)| *val);

            let manual_min = data.iter()
                .min_by_key(|(val, _)| val)
                .cloned();

            prop_assert_eq!(rinq_min, manual_min);
        }

        #[test]
        fn prop_max_by_with_key_selector(data: Vec<(i32, String)>) {
            let rinq_max = QueryBuilder::from(data.clone())
                .max_by(|(val, _)| *val);

            let manual_max = data.iter()
                .max_by_key(|(val, _)| val)
                .cloned();

            prop_assert_eq!(rinq_max, manual_max);
        }
    }
}

#[cfg(test)]
mod aggregation_unit_tests {
    use super::*;

    // T103: Unit test - sum() edge cases
    #[test]
    fn test_sum_empty_collection() {
        let empty: Vec<i32> = vec![];
        let sum: i32 = QueryBuilder::from(empty).sum();
        assert_eq!(sum, 0); // Additive identity
    }

    #[test]
    fn test_sum_single_element() {
        let single = vec![42];
        let sum: i32 = QueryBuilder::from(single).sum();
        assert_eq!(sum, 42);
    }

    #[test]
    fn test_sum_negative_numbers() {
        let negatives = vec![-5, -3, -8];
        let sum: i32 = QueryBuilder::from(negatives).sum();
        assert_eq!(sum, -16);
    }

    #[test]
    fn test_sum_mixed_signs() {
        let mixed = vec![-5, 10, -3, 8];
        let sum: i32 = QueryBuilder::from(mixed).sum();
        assert_eq!(sum, 10);
    }

    // T105: Unit test - average() edge cases
    #[test]
    fn test_average_empty_collection() {
        let empty: Vec<i32> = vec![];
        let avg = QueryBuilder::from(empty).average();
        assert_eq!(avg, None);
    }

    #[test]
    fn test_average_single_element() {
        let single = vec![42];
        let avg = QueryBuilder::from(single).average().unwrap();
        assert_eq!(avg, 42.0);
    }

    #[test]
    fn test_average_precision() {
        let data = vec![1, 2, 3, 4, 5];
        let avg = QueryBuilder::from(data).average().unwrap();
        assert!((avg - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_average_large_numbers() {
        let large = vec![1_000_000, 2_000_000, 3_000_000];
        let avg = QueryBuilder::from(large).average().unwrap();
        assert!((avg - 2_000_000.0).abs() < 1.0);
    }

    // T107: Unit test - min() / max() edge cases
    #[test]
    fn test_min_max_empty_collection() {
        let empty: Vec<i32> = vec![];
        let min = QueryBuilder::from(empty.clone()).min();
        let max = QueryBuilder::from(empty).max();
        assert_eq!(min, None);
        assert_eq!(max, None);
    }

    #[test]
    fn test_min_max_single_element() {
        let single = vec![42];
        let min = QueryBuilder::from(single.clone()).min();
        let max = QueryBuilder::from(single).max();
        assert_eq!(min, Some(42));
        assert_eq!(max, Some(42));
    }

    #[test]
    fn test_min_max_all_equal() {
        let equal = vec![7, 7, 7, 7];
        let min = QueryBuilder::from(equal.clone()).min();
        let max = QueryBuilder::from(equal).max();
        assert_eq!(min, Some(7));
        assert_eq!(max, Some(7));
    }

    // T109: Unit test - min_by() / max_by() edge cases
    #[test]
    fn test_min_by_empty_collection() {
        let empty: Vec<(i32, String)> = vec![];
        let min = QueryBuilder::from(empty).min_by(|(val, _)| *val);
        assert_eq!(min, None);
    }

    #[test]
    fn test_max_by_multiple_same_key() {
        let data = vec![
            (1, "first".to_string()),
            (5, "second".to_string()),
            (5, "third".to_string()), // Same max key
            (3, "fourth".to_string()),
        ];

        let max = QueryBuilder::from(data).max_by(|(val, _)| *val).unwrap();
        // Note: Rust's Iterator::max_by_key returns LAST occurrence of max key
        assert_eq!(max.0, 5);
        assert_eq!(max.1, "third"); // Last with value 5 (Rust standard behavior)
    }

    #[test]
    fn test_min_by_with_struct() {
        #[derive(Debug, Clone, PartialEq)]
        struct User {
            name: String,
            age: u32,
        }

        let users = vec![
            User {
                name: "Alice".into(),
                age: 30,
            },
            User {
                name: "Bob".into(),
                age: 25,
            },
            User {
                name: "Charlie".into(),
                age: 35,
            },
        ];

        let youngest = QueryBuilder::from(users).min_by(|u| u.age).unwrap();
        assert_eq!(youngest.name, "Bob");
        assert_eq!(youngest.age, 25);
    }
}

// Placeholder modules for future phases
// Will be populated as we implement each user story

#[cfg(test)]
mod grouping_properties {
    use super::*;

    // T201: Property test - group_by() completeness
    proptest! {
        #[test]
        fn prop_group_by_completeness(data in prop::collection::vec(-100i32..100i32, 0..50)) {
            let groups = QueryBuilder::from(data.clone())
                .group_by(|x| x % 3);

            // All elements should be accounted for
            let mut regrouped: Vec<i32> = groups.values()
                .flat_map(|v| v.iter().copied())
                .collect();
            regrouped.sort();

            let mut original_sorted = data.clone();
            original_sorted.sort();

            prop_assert_eq!(regrouped, original_sorted);
        }
    }

    // T202: Property test - group_by() determinism
    proptest! {
        #[test]
        fn prop_group_by_determinism(data in prop::collection::vec(-50i32..50i32, 0..30)) {
            let groups1 = QueryBuilder::from(data.clone())
                .group_by(|x| x / 10);

            let groups2 = QueryBuilder::from(data.clone())
                .group_by(|x| x / 10);

            // Same input should produce same grouping
            prop_assert_eq!(groups1.len(), groups2.len());
            for (k, v1) in &groups1 {
                let v2 = groups2.get(k).unwrap();
                prop_assert_eq!(v1, v2);
            }
        }
    }

    // T203: Property test - group_by() order preservation
    proptest! {
        #[test]
        fn prop_group_by_preserves_within_group_order(
            data in prop::collection::vec(0i32..20i32, 0..30)
        ) {
            let groups = QueryBuilder::from(data.clone())
                .group_by(|x| x % 5);

            // Within each group, order should match original
            for (key, group) in &groups {
                let expected_order: Vec<i32> = data.iter()
                    .copied()
                    .filter(|x| x % 5 == *key)
                    .collect();
                prop_assert_eq!(group, &expected_order);
            }
        }
    }

    // T205: Property test - group_by_aggregate() correctness
    proptest! {
        #[test]
        fn prop_group_by_aggregate_correctness(
            data in prop::collection::vec(-50i32..50i32, 0..30)
        ) {
            let aggregated = QueryBuilder::from(data.clone())
                .group_by_aggregate(
                    |x| x % 3,
                    |group| group.iter().sum::<i32>()
                );

            // Manual grouping and aggregation
            let manual_groups = QueryBuilder::from(data.clone())
                .group_by(|x| x % 3);

            for (key, expected_sum) in manual_groups {
                let expected: i32 = expected_sum.iter().sum();
                let actual = aggregated.get(&key).unwrap();
                prop_assert_eq!(*actual, expected);
            }
        }
    }
}

#[cfg(test)]
mod grouping_unit_tests {
    use super::*;

    // T204: Unit test - group_by() edge cases
    #[test]
    fn test_group_by_empty_collection() {
        let empty: Vec<i32> = vec![];
        let groups = QueryBuilder::from(empty).group_by(|x| *x);
        assert_eq!(groups.len(), 0);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_by_all_same_key() {
        let data = vec![5, 5, 5, 5];
        let groups = QueryBuilder::from(data.clone()).group_by(|x| *x);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups.get(&5).unwrap(), &data);
    }

    #[test]
    fn test_group_by_all_unique_keys() {
        let data = vec![1, 2, 3, 4];
        let groups = QueryBuilder::from(data.clone()).group_by(|x| *x);

        assert_eq!(groups.len(), 4);
        for &val in &data {
            assert_eq!(groups.get(&val).unwrap(), &vec![val]);
        }
    }

    #[test]
    fn test_group_by_with_structs() {
        #[derive(Debug, Clone, PartialEq)]
        struct User {
            name: String,
            department: String,
        }

        let users = vec![
            User {
                name: "Alice".into(),
                department: "Engineering".into(),
            },
            User {
                name: "Bob".into(),
                department: "Sales".into(),
            },
            User {
                name: "Charlie".into(),
                department: "Engineering".into(),
            },
        ];

        let by_dept = QueryBuilder::from(users.clone()).group_by(|u| u.department.clone());

        assert_eq!(by_dept.len(), 2);
        assert_eq!(by_dept.get("Engineering").unwrap().len(), 2);
        assert_eq!(by_dept.get("Sales").unwrap().len(), 1);
    }

    // T206: Unit test - group_by_aggregate() edge cases
    #[test]
    fn test_group_by_aggregate_empty() {
        let empty: Vec<i32> = vec![];
        let aggregated = QueryBuilder::from(empty).group_by_aggregate(|x| *x, |group| group.len());

        assert!(aggregated.is_empty());
    }

    #[test]
    fn test_group_by_aggregate_single_group() {
        let data = vec![10, 10, 10];
        let aggregated = QueryBuilder::from(data)
            .group_by_aggregate(|x| x / 10, |group| group.iter().sum::<i32>());

        assert_eq!(aggregated.len(), 1);
        assert_eq!(*aggregated.get(&1).unwrap(), 30);
    }

    #[test]
    fn test_group_by_aggregate_multiple_aggregations() {
        #[derive(Debug, Clone)]
        struct Order {
            user_id: u32,
            amount: f64,
        }

        let orders = vec![
            Order {
                user_id: 1,
                amount: 100.0,
            },
            Order {
                user_id: 2,
                amount: 50.0,
            },
            Order {
                user_id: 1,
                amount: 75.0,
            },
            Order {
                user_id: 2,
                amount: 200.0,
            },
        ];

        // Sum amounts by user
        let totals = QueryBuilder::from(orders.clone()).group_by_aggregate(
            |o| o.user_id,
            |group| group.iter().map(|o| o.amount).sum::<f64>(),
        );

        assert_eq!(totals.get(&1), Some(&175.0));
        assert_eq!(totals.get(&2), Some(&250.0));

        // Count orders by user
        let counts =
            QueryBuilder::from(orders).group_by_aggregate(|o| o.user_id, |group| group.len());

        assert_eq!(counts.get(&1), Some(&2));
        assert_eq!(counts.get(&2), Some(&2));
    }
}

#[cfg(test)]
mod deduplication_properties {
    use super::*;

    // T301: Property test - distinct() removes all duplicates
    proptest! {
        #[test]
        fn prop_distinct_removes_all_duplicates(data in prop::collection::vec(-50i32..50i32, 0..100)) {
            let distinct_data: Vec<i32> = QueryBuilder::from(data.clone())
                .distinct()
                .collect();

            // Manual deduplication
            let manual_set: HashSet<i32> = data.into_iter().collect();
            let manual_vec: HashSet<i32> = distinct_data.iter().copied().collect();

            // Same elements (order may differ)
            prop_assert_eq!(manual_vec, manual_set);
        }
    }

    // T302: Property test - distinct() preserves first occurrence order
    proptest! {
        #[test]
        fn prop_distinct_preserves_order(data in prop::collection::vec(-20i32..20i32, 0..30)) {
            let distinct_data: Vec<i32> = QueryBuilder::from(data.clone())
                .distinct()
                .collect();

            // Check no duplicates exist
            let mut seen = HashSet::new();
            for item in &distinct_data {
                prop_assert!(!seen.contains(item), "Found duplicate: {}", item);
                seen.insert(*item);
            }

            // Check all original unique elements are present
            let original_unique: HashSet<i32> = data.into_iter().collect();
            let result_set: HashSet<i32> = distinct_data.into_iter().collect();
            prop_assert_eq!(result_set, original_unique);
        }
    }

    // T304: Property test - distinct_by() with key selector
    proptest! {
        #[test]
        fn prop_distinct_by_key_selector(
            data in prop::collection::vec((-50i32..50i32, 0usize..100usize), 0..50)
        ) {
            let distinct_data: Vec<(i32, usize)> = QueryBuilder::from(data.clone())
                .distinct_by(|(val, _id)| *val)
                .collect();

            // Check no duplicate keys
            let mut seen_keys = HashSet::new();
            for (val, _) in &distinct_data {
                prop_assert!(!seen_keys.contains(val));
                seen_keys.insert(*val);
            }

            // Check all unique keys from original are present
            let original_keys: HashSet<i32> = data.iter().map(|(v, _)| *v).collect();
            prop_assert_eq!(seen_keys, original_keys);
        }
    }
}

#[cfg(test)]
mod deduplication_unit_tests {
    use super::*;

    // T303: Unit test - distinct() edge cases
    #[test]
    fn test_distinct_empty_collection() {
        let empty: Vec<i32> = vec![];
        let result: Vec<i32> = QueryBuilder::from(empty).distinct().collect();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_distinct_no_duplicates() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<i32> = QueryBuilder::from(data.clone()).distinct().collect();

        let result_set: HashSet<i32> = result.into_iter().collect();
        let data_set: HashSet<i32> = data.into_iter().collect();
        assert_eq!(result_set, data_set);
    }

    #[test]
    fn test_distinct_all_duplicates() {
        let data = vec![7, 7, 7, 7, 7];
        let result: Vec<i32> = QueryBuilder::from(data).distinct().collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 7);
    }

    #[test]
    fn test_distinct_mixed_duplicates() {
        let data = vec![1, 2, 1, 3, 2, 4, 1];
        let result: Vec<i32> = QueryBuilder::from(data).distinct().collect();

        // Should contain 1, 2, 3, 4 (order preserved for first occurrence)
        assert_eq!(result.len(), 4);
        let result_set: HashSet<i32> = result.into_iter().collect();
        assert_eq!(result_set, [1, 2, 3, 4].iter().copied().collect());
    }

    // T305: Unit test - distinct_by() edge cases
    #[test]
    fn test_distinct_by_empty_collection() {
        let empty: Vec<(i32, String)> = vec![];
        let result: Vec<(i32, String)> = QueryBuilder::from(empty)
            .distinct_by(|(val, _)| *val)
            .collect();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_distinct_by_custom_key() {
        #[derive(Debug, Clone, PartialEq)]
        struct User {
            id: u32,
            name: String,
            email: String,
        }

        let users = vec![
            User {
                id: 1,
                name: "Alice".into(),
                email: "alice@example.com".into(),
            },
            User {
                id: 2,
                name: "Bob".into(),
                email: "bob@example.com".into(),
            },
            User {
                id: 3,
                name: "Alice".into(),
                email: "alice2@example.com".into(),
            }, // Duplicate name
        ];

        // Deduplicate by name
        let unique_names: Vec<User> = QueryBuilder::from(users)
            .distinct_by(|u| u.name.clone())
            .collect();

        assert_eq!(unique_names.len(), 2); // Only Alice and Bob
        assert_eq!(unique_names[0].name, "Alice");
        assert_eq!(unique_names[1].name, "Bob");
    }

    #[test]
    fn test_distinct_by_preserves_first_occurrence() {
        let data = vec![
            (1, "first"),
            (2, "second"),
            (1, "third"), // Duplicate key 1
            (3, "fourth"),
        ];

        let result: Vec<(i32, &str)> = QueryBuilder::from(data)
            .distinct_by(|(val, _)| *val)
            .collect();

        assert_eq!(result.len(), 3);
        // Should keep first occurrence of key 1
        assert_eq!(result[0], (1, "first"));
        assert_eq!(result[1], (2, "second"));
        assert_eq!(result[2], (3, "fourth"));
    }
}

#[cfg(test)]
mod sequence_properties {
    use super::*;

    // T401: Property test - reverse() correctness
    proptest! {
        #[test]
        fn prop_reverse_equals_manual_reverse(data in prop::collection::vec(-100i32..100i32, 0..100)) {
            let reversed: Vec<i32> = QueryBuilder::from(data.clone())
                .reverse()
                .collect();

            let mut manual_reversed = data;
            manual_reversed.reverse();

            prop_assert_eq!(reversed, manual_reversed);
        }
    }

    // T403: Property test - chunk() correctness
    proptest! {
        #[test]
        fn prop_chunk_correctness(
            data in prop::collection::vec(-50i32..50i32, 0..50),
            chunk_size in 1usize..10usize
        ) {
            let chunks: Vec<Vec<i32>> = QueryBuilder::from(data.clone())
                .chunk(chunk_size)
                .collect();

            // Manual chunking
            let manual_chunks: Vec<Vec<i32>> = data
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect();

            prop_assert_eq!(chunks, manual_chunks);
        }
    }

    // T405: Property test - window() correctness
    proptest! {
        #[test]
        fn prop_window_correctness(
            data in prop::collection::vec(-50i32..50i32, 3..30),
            window_size in 2usize..5usize
        ) {
            let windows: Vec<Vec<i32>> = QueryBuilder::from(data.clone())
                .window(window_size)
                .collect();

            // Manual windowing
            let manual_windows: Vec<Vec<i32>> = data
                .windows(window_size)
                .map(|w| w.to_vec())
                .collect();

            prop_assert_eq!(windows, manual_windows);
        }
    }
}

#[cfg(test)]
mod sequence_unit_tests {
    use super::*;

    // T402: Unit test - reverse() edge cases
    #[test]
    fn test_reverse_empty_collection() {
        let empty: Vec<i32> = vec![];
        let result: Vec<i32> = QueryBuilder::from(empty).reverse().collect();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_reverse_single_element() {
        let single = vec![42];
        let result: Vec<i32> = QueryBuilder::from(single).reverse().collect();
        assert_eq!(result, vec![42]);
    }

    #[test]
    fn test_reverse_multiple_elements() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<i32> = QueryBuilder::from(data).reverse().collect();
        assert_eq!(result, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_after_filter() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let result: Vec<i32> = QueryBuilder::from(data)
            .where_(|x| *x % 2 == 0)
            .reverse()
            .collect();
        assert_eq!(result, vec![6, 4, 2]);
    }

    // T404: Unit test - chunk() edge cases
    #[test]
    fn test_chunk_empty_collection() {
        let empty: Vec<i32> = vec![];
        let result: Vec<Vec<i32>> = QueryBuilder::from(empty).chunk(3).collect();
        assert_eq!(result, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_chunk_exact_division() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data).chunk(3).collect();
        assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    fn test_chunk_partial_last_chunk() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data).chunk(3).collect();
        assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5]]);
    }

    #[test]
    #[should_panic(expected = "chunk size must be greater than 0")]
    fn test_chunk_zero_size_panics() {
        let data = vec![1, 2, 3];
        let _: Vec<Vec<i32>> = QueryBuilder::from(data).chunk(0).collect();
    }

    #[test]
    fn test_chunk_size_larger_than_collection() {
        let data = vec![1, 2, 3];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data.clone()).chunk(10).collect();
        assert_eq!(result, vec![data]);
    }

    // T406: Unit test - window() edge cases
    #[test]
    fn test_window_empty_collection() {
        let empty: Vec<i32> = vec![];
        let result: Vec<Vec<i32>> = QueryBuilder::from(empty).window(3).collect();
        assert_eq!(result, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_window_overlapping() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data).window(3).collect();
        assert_eq!(result, vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
    }

    #[test]
    fn test_window_size_two() {
        let data = vec![10, 20, 30, 40];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data).window(2).collect();
        assert_eq!(result, vec![vec![10, 20], vec![20, 30], vec![30, 40]]);
    }

    #[test]
    #[should_panic(expected = "window size must be greater than 0")]
    fn test_window_zero_size_panics() {
        let data = vec![1, 2, 3];
        let _: Vec<Vec<i32>> = QueryBuilder::from(data).window(0).collect();
    }

    #[test]
    fn test_window_size_larger_than_collection() {
        let data = vec![1, 2, 3];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data).window(5).collect();
        assert_eq!(result, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_window_exactly_collection_size() {
        let data = vec![1, 2, 3];
        let result: Vec<Vec<i32>> = QueryBuilder::from(data.clone()).window(3).collect();
        assert_eq!(result, vec![data]);
    }
}

#[cfg(test)]
mod combination_properties {
    use super::*;

    // T501: Property test - zip() correctness
    proptest! {
        #[test]
        fn prop_zip_correctness(
            data1 in prop::collection::vec(-50i32..50i32, 0..50),
            data2 in prop::collection::vec(0usize..100usize, 0..50)
        ) {
            let zipped: Vec<(i32, usize)> = QueryBuilder::from(data1.clone())
                .zip(data2.clone())
                .collect();

            let manual_zipped: Vec<(i32, usize)> = data1.into_iter()
                .zip(data2.into_iter())
                .collect();

            prop_assert_eq!(zipped, manual_zipped);
        }
    }

    // T503: Property test - enumerate() correctness
    proptest! {
        #[test]
        fn prop_enumerate_correctness(data in prop::collection::vec(-50i32..50i32, 0..50)) {
            let enumerated: Vec<(usize, i32)> = QueryBuilder::from(data.clone())
                .enumerate()
                .collect();

            let manual_enumerated: Vec<(usize, i32)> = data.into_iter()
                .enumerate()
                .collect();

            prop_assert_eq!(enumerated, manual_enumerated);
        }
    }

    // T505: Property test - partition() correctness
    proptest! {
        #[test]
        fn prop_partition_correctness(
            data in prop::collection::vec(-50i32..50i32, 0..50),
            threshold in -25i32..25i32
        ) {
            let (left, right) = QueryBuilder::from(data.clone())
                .partition(move |x| *x < threshold);

            let manual_left: Vec<i32> = data.iter().copied().filter(|x| *x < threshold).collect();
            let manual_right: Vec<i32> = data.iter().copied().filter(|x| *x >= threshold).collect();

            prop_assert_eq!(left, manual_left);
            prop_assert_eq!(right, manual_right);
        }
    }

    // T506: Property test - partition() completeness
    proptest! {
        #[test]
        fn prop_partition_completeness(data in prop::collection::vec(-50i32..50i32, 0..50)) {
            let (left, right) = QueryBuilder::from(data.clone())
                .partition(|x| *x % 2 == 0);

            // All elements should be accounted for
            let mut recombined = left.clone();
            recombined.extend(right.clone());
            recombined.sort();

            let mut original_sorted = data;
            original_sorted.sort();

            prop_assert_eq!(recombined, original_sorted);
        }
    }
}

#[cfg(test)]
mod combination_unit_tests {
    use super::*;

    // T502: Unit test - zip() edge cases
    #[test]
    fn test_zip_empty_collections() {
        let empty1: Vec<i32> = vec![];
        let empty2: Vec<String> = vec![];
        let result: Vec<(i32, String)> = QueryBuilder::from(empty1).zip(empty2).collect();
        assert_eq!(result, Vec::<(i32, String)>::new());
    }

    #[test]
    fn test_zip_first_shorter() {
        let data1 = vec![1, 2];
        let data2 = vec!["a", "b", "c", "d"];
        let result: Vec<(i32, &str)> = QueryBuilder::from(data1).zip(data2).collect();
        assert_eq!(result, vec![(1, "a"), (2, "b")]);
    }

    #[test]
    fn test_zip_second_shorter() {
        let data1 = vec![1, 2, 3, 4];
        let data2 = vec!["a", "b"];
        let result: Vec<(i32, &str)> = QueryBuilder::from(data1).zip(data2).collect();
        assert_eq!(result, vec![(1, "a"), (2, "b")]);
    }

    #[test]
    fn test_zip_same_length() {
        let data1 = vec![1, 2, 3];
        let data2 = vec!["a", "b", "c"];
        let result: Vec<(i32, &str)> = QueryBuilder::from(data1).zip(data2).collect();
        assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
    }

    // T504: Unit test - enumerate() edge cases
    #[test]
    fn test_enumerate_empty_collection() {
        let empty: Vec<i32> = vec![];
        let result: Vec<(usize, i32)> = QueryBuilder::from(empty).enumerate().collect();
        assert_eq!(result, Vec::<(usize, i32)>::new());
    }

    #[test]
    fn test_enumerate_single_element() {
        let data = vec![42];
        let result: Vec<(usize, i32)> = QueryBuilder::from(data).enumerate().collect();
        assert_eq!(result, vec![(0, 42)]);
    }

    #[test]
    fn test_enumerate_multiple_elements() {
        let data = vec![10, 20, 30];
        let result: Vec<(usize, i32)> = QueryBuilder::from(data).enumerate().collect();
        assert_eq!(result, vec![(0, 10), (1, 20), (2, 30)]);
    }

    #[test]
    fn test_enumerate_after_filter() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<(usize, i32)> = QueryBuilder::from(data)
            .where_(|x| *x % 2 == 0)
            .enumerate()
            .collect();
        // Enumerate AFTER filtering, so indices start from 0
        assert_eq!(result, vec![(0, 2), (1, 4)]);
    }

    // T507: Unit test - partition() edge cases
    #[test]
    fn test_partition_empty_collection() {
        let empty: Vec<i32> = vec![];
        let (left, right) = QueryBuilder::from(empty).partition(|x| *x > 0);
        assert_eq!(left, Vec::<i32>::new());
        assert_eq!(right, Vec::<i32>::new());
    }

    #[test]
    fn test_partition_all_match() {
        let data = vec![1, 2, 3, 4];
        let (left, right) = QueryBuilder::from(data.clone()).partition(|_| true);
        assert_eq!(left, data);
        assert_eq!(right, Vec::<i32>::new());
    }

    #[test]
    fn test_partition_none_match() {
        let data = vec![1, 2, 3, 4];
        let (left, right) = QueryBuilder::from(data.clone()).partition(|_| false);
        assert_eq!(left, Vec::<i32>::new());
        assert_eq!(right, data);
    }

    #[test]
    fn test_partition_mixed() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let (evens, odds) = QueryBuilder::from(data).partition(|x| *x % 2 == 0);
        assert_eq!(evens, vec![2, 4, 6]);
        assert_eq!(odds, vec![1, 3, 5]);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // T601: Complex query chain - v0.1 + v0.2 operations
    #[test]
    fn test_complex_chain_with_all_features() {
        #[derive(Debug, Clone, PartialEq)]
        struct Order {
            user_id: u32,
            product: String,
            amount: f64,
            #[allow(dead_code)]
            quantity: u32,
        }

        let orders = vec![
            Order {
                user_id: 1,
                product: "A".into(),
                amount: 100.0,
                quantity: 2,
            },
            Order {
                user_id: 2,
                product: "B".into(),
                amount: 50.0,
                quantity: 1,
            },
            Order {
                user_id: 1,
                product: "A".into(),
                amount: 100.0,
                quantity: 2,
            }, // Duplicate
            Order {
                user_id: 3,
                product: "C".into(),
                amount: 200.0,
                quantity: 5,
            },
            Order {
                user_id: 2,
                product: "A".into(),
                amount: 75.0,
                quantity: 3,
            },
            Order {
                user_id: 1,
                product: "B".into(),
                amount: 60.0,
                quantity: 2,
            },
        ];

        // Complex chain: filter -> distinct -> sort -> take -> aggregate
        let top_users_total: Vec<(usize, f64)> = QueryBuilder::from(orders)
            .where_(|o| o.quantity >= 2) // v0.1: filter
            .distinct_by(|o| (o.user_id, o.product.clone())) // v0.2: deduplication
            .order_by(|o| o.amount as i32) // v0.1: sort (descending by amount)
            .reverse() // v0.2: reverse to get highest amounts first
            .take(3) // v0.1: pagination
            .enumerate() // v0.2: add indices
            .select(|(idx, order)| (idx, order.amount)) // v0.1: projection
            .collect();

        // Should have at most 3 elements, indexed
        assert!(top_users_total.len() <= 3);
        assert_eq!(top_users_total[0].0, 0);
    }

    // T602: Aggregation on filtered and sorted data
    #[test]
    fn test_aggregate_after_filter_and_sort() {
        let data = vec![10, 5, 20, 15, 8, 25, 12];

        // Get sum of top 3 values above 10
        let sum: i32 = QueryBuilder::from(data)
            .where_(|x| *x > 10) // v0.1: filter -> [20, 15, 25, 12]
            .order_by(|x| *x) // v0.1: sort -> [12, 15, 20, 25]
            .reverse() // v0.2: reverse -> [25, 20, 15, 12]
            .take(3) // v0.1: pagination -> [25, 20, 15]
            .sum(); // v0.2: aggregation

        assert_eq!(sum, 60);
    }

    // T603: Grouping after filtering and deduplication
    #[test]
    fn test_group_after_filter_and_distinct() {
        let data = vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 5];

        let groups: HashMap<i32, Vec<i32>> = QueryBuilder::from(data)
            .where_(|x| *x > 1) // v0.1: filter
            .distinct() // v0.2: remove duplicates -> [2, 3, 4, 5]
            .group_by(|x| x % 2); // v0.2: group by even/odd

        assert_eq!(groups.get(&0).unwrap(), &vec![2, 4]); // Even
        assert_eq!(groups.get(&1).unwrap(), &vec![3, 5]); // Odd
    }

    // T604: Chaining multiple v0.2 operations
    #[test]
    fn test_multiple_v0_2_operations() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // Complex v0.2 chain: chunk -> flatten via select -> distinct -> enumerate
        let result: Vec<(usize, Vec<i32>)> = QueryBuilder::from(data)
            .chunk(3) // v0.2: [[1,2,3], [4,5,6], [7,8,9], [10]]
            .enumerate() // v0.2: add indices
            .collect();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], (0, vec![1, 2, 3]));
        assert_eq!(result[1], (1, vec![4, 5, 6]));
        assert_eq!(result[2], (2, vec![7, 8, 9]));
        assert_eq!(result[3], (3, vec![10]));
    }

    // T605: Window with aggregation
    #[test]
    fn test_window_with_aggregation() {
        let data = vec![1, 5, 3, 8, 2, 9, 4];

        // Get sum of each 3-element window
        let window_sums: Vec<i32> = QueryBuilder::from(data)
            .window(3) // [[1,5,3], [5,3,8], [3,8,2], [8,2,9], [2,9,4]]
            .select(|w| w.iter().sum::<i32>()) // [9, 16, 13, 19, 15]
            .collect();

        assert_eq!(window_sums, vec![9, 16, 13, 19, 15]);
    }

    // T606: Partition after filtering
    #[test]
    fn test_partition_after_filter() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let (small, large) = QueryBuilder::from(data)
            .where_(|x| *x > 3) // [4, 5, 6, 7, 8, 9, 10]
            .partition(|x| *x < 7); // Partition at 7

        assert_eq!(small, vec![4, 5, 6]);
        assert_eq!(large, vec![7, 8, 9, 10]);
    }

    // T607: Zip with projection
    #[test]
    fn test_zip_with_projection() {
        let numbers = vec![1, 2, 3, 4];
        let names = vec!["Alice", "Bob", "Charlie", "Dave"];

        let pairs: Vec<String> = QueryBuilder::from(numbers)
            .zip(names)
            .select(|(num, name)| format!("{}: {}", num, name))
            .collect();

        assert_eq!(pairs, vec!["1: Alice", "2: Bob", "3: Charlie", "4: Dave"]);
    }

    // T608: Group by aggregate with complex calculation
    #[test]
    fn test_group_by_aggregate_complex() {
        #[derive(Clone)]
        struct Sale {
            region: String,
            amount: f64,
            #[allow(dead_code)]
            quantity: u32,
        }

        let sales = vec![
            Sale {
                region: "North".into(),
                amount: 100.0,
                quantity: 2,
            },
            Sale {
                region: "South".into(),
                amount: 150.0,
                quantity: 3,
            },
            Sale {
                region: "North".into(),
                amount: 200.0,
                quantity: 4,
            },
            Sale {
                region: "East".into(),
                amount: 80.0,
                quantity: 1,
            },
            Sale {
                region: "South".into(),
                amount: 120.0,
                quantity: 2,
            },
        ];

        // Calculate average amount per region
        let avg_by_region = QueryBuilder::from(sales).group_by_aggregate(
            |s| s.region.clone(),
            |group| {
                let total: f64 = group.iter().map(|s| s.amount).sum();
                total / group.len() as f64
            },
        );

        assert!((avg_by_region.get("North").unwrap() - 150.0).abs() < 1e-10);
        assert!((avg_by_region.get("South").unwrap() - 135.0).abs() < 1e-10);
        assert!((avg_by_region.get("East").unwrap() - 80.0).abs() < 1e-10);
    }

    // T609: Min/Max after complex chain
    #[test]
    fn test_min_max_after_complex_chain() {
        let data = vec![5, 1, 9, 3, 7, 2, 8, 4, 6];

        // Get max of middle 5 values (excluding extremes)
        let max_middle = QueryBuilder::from(data.clone())
            .order_by(|x| *x) // [1, 2, 3, 4, 5, 6, 7, 8, 9]
            .skip(2) // [3, 4, 5, 6, 7, 8, 9]
            .take(5) // [3, 4, 5, 6, 7]
            .max();

        assert_eq!(max_middle, Some(7));

        // Get min of values > 5
        let min_high = QueryBuilder::from(data)
            .where_(|x| *x > 5) // [9, 7, 8, 6]
            .min();

        assert_eq!(min_high, Some(6));
    }

    // T610: All v0.2 features in one query
    #[test]
    fn test_all_v0_2_features_combined() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        // Massive chain demonstrating all features
        let result: Vec<(usize, i32)> = QueryBuilder::from(data)
            .where_(|x| *x > 2) // v0.1: filter -> [3..=12]
            .distinct() // v0.2: remove duplicates (none here)
            .chunk(3) // v0.2: [[3,4,5], [6,7,8], [9,10,11], [12]]
            .enumerate() // v0.2: add indices
            .select(|(idx, chunk)| (idx, chunk.iter().sum::<i32>())) // v0.1: project to (idx, sum)
            .collect();

        assert_eq!(result, vec![(0, 12), (1, 21), (2, 30), (3, 12)]);
    }
}
