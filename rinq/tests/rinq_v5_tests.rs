// tests/rinq_v5_tests.rs
// Integration tests for Phase 5B: untested operators, combinations, edge cases.

use rinq::{FilteredQuery, QueryBuilder};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

// ── 5B-1: tap_each ────────────────────────────────────────────────────────────

#[test]
fn tap_each_counts_all_elements() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .tap_each(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[test]
fn tap_each_empty_no_side_effect() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let result = QueryBuilder::<i32, _>::from(vec![])
        .tap_each(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        })
        .collect::<Vec<_>>();
    assert!(result.is_empty());
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn tap_each_after_where_sees_only_filtered_elements() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|&x| x % 2 == 0)
        .tap_each(move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![2, 4]);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn tap_each_does_not_modify_elements() {
    let result = QueryBuilder::from(vec![10, 20, 30])
        .tap_each(|_| { /* noop */ })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn tap_each_can_record_values() {
    let seen = Arc::new(Mutex::new(Vec::<i32>::new()));
    let s = seen.clone();
    let _result = QueryBuilder::from(vec![3, 1, 4])
        .tap_each(move |&x| {
            s.lock().unwrap().push(x);
        })
        .collect::<Vec<_>>();
    assert_eq!(*seen.lock().unwrap(), vec![3, 1, 4]);
}

// ── 5B-1: tap_collect ─────────────────────────────────────────────────────────

#[test]
fn tap_collect_receives_all_elements_as_slice() {
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
fn tap_collect_empty_still_calls_closure() {
    let called = Arc::new(AtomicBool::new(false));
    let c = called.clone();
    let result = QueryBuilder::<i32, _>::from(vec![])
        .tap_collect(move |items| {
            c.store(true, Ordering::SeqCst);
            assert!(items.is_empty());
        })
        .collect::<Vec<_>>();
    assert!(result.is_empty());
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn tap_collect_does_not_change_elements() {
    let result = QueryBuilder::from(vec![10, 20, 30])
        .tap_collect(|_| { /* noop */ })
        .collect::<Vec<_>>();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn tap_collect_slice_length_matches() {
    let length = Arc::new(AtomicUsize::new(0));
    let l = length.clone();
    QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|&x| x > 2)
        .tap_collect(move |items| {
            l.store(items.len(), Ordering::SeqCst);
        })
        .collect::<Vec<_>>();
    assert_eq!(length.load(Ordering::SeqCst), 3);
}

// ── 5B-1: pipe ────────────────────────────────────────────────────────────────

fn add_positive_filter(q: FilteredQuery<i32>) -> FilteredQuery<i32> {
    q.where_(|&x| x > 0)
}

fn add_even_filter(q: FilteredQuery<i32>) -> FilteredQuery<i32> {
    q.where_(|&x| x % 2 == 0)
}

#[test]
fn pipe_delegates_to_external_function() {
    let result = QueryBuilder::from(vec![-2, -1, 0, 1, 2, 3])
        .where_(|_| true)
        .pipe(add_positive_filter)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn pipe_chains_two_functions() {
    let result = QueryBuilder::from(vec![-2, -1, 0, 1, 2, 3, 4])
        .where_(|_| true)
        .pipe(add_positive_filter)
        .pipe(add_even_filter)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![2, 4]);
}

#[test]
fn pipe_identity_returns_same_elements() {
    let result = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .pipe(|q| q)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn pipe_with_closure() {
    let threshold = 3;
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|_| true)
        .pipe(move |q| q.where_(move |&x| x >= threshold))
        .collect::<Vec<_>>();
    assert_eq!(result, vec![3, 4, 5]);
}

// ── 5B-1: map (select alias) ──────────────────────────────────────────────────

#[test]
fn map_equals_select_on_primitives() {
    let via_map = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .map(|x| x * 2)
        .collect::<Vec<_>>();
    let via_select = QueryBuilder::from(vec![1, 2, 3])
        .where_(|_| true)
        .select(|x| x * 2)
        .collect::<Vec<_>>();
    assert_eq!(via_map, via_select);
}

#[test]
fn map_type_transformation() {
    let result = QueryBuilder::from(vec![1i32, 2, 3])
        .where_(|_| true)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    assert_eq!(result, vec!["1", "2", "3"]);
}

// ── 5B-1: collect_vec ─────────────────────────────────────────────────────────

#[test]
fn collect_vec_equals_collect() {
    let via_collect_vec = QueryBuilder::from(vec![1, 2, 3, 4])
        .where_(|&x| x > 1)
        .collect_vec();
    let via_collect: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4])
        .where_(|&x| x > 1)
        .collect();
    assert_eq!(via_collect_vec, via_collect);
}

#[test]
fn collect_vec_empty() {
    let result: Vec<i32> = QueryBuilder::from(vec![]).collect_vec();
    assert!(result.is_empty());
}

// ── 5B-3: pairwise edge cases (0 / 1 / 2 elements) ───────────────────────────

#[test]
fn pairwise_zero_elements_returns_empty() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![]).pairwise().collect();
    assert!(result.is_empty());
}

#[test]
fn pairwise_one_element_returns_empty() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![42]).pairwise().collect();
    assert!(result.is_empty());
}

#[test]
fn pairwise_two_elements_returns_one_pair() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2]).pairwise().collect();
    assert_eq!(result, vec![(1, 2)]);
}

#[test]
fn pairwise_three_elements_returns_two_pairs() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2, 3]).pairwise().collect();
    assert_eq!(result, vec![(1, 2), (2, 3)]);
}

// ── 5B-3: intersperse edge cases ──────────────────────────────────────────────

#[test]
fn intersperse_empty_returns_empty() {
    let result: Vec<i32> = QueryBuilder::from(vec![]).intersperse(0).collect();
    assert!(result.is_empty());
}

#[test]
fn intersperse_single_element_returns_unchanged() {
    let result: Vec<i32> = QueryBuilder::from(vec![42]).intersperse(0).collect();
    assert_eq!(result, vec![42]);
}

#[test]
fn intersperse_two_elements_has_one_separator() {
    let result: Vec<i32> = QueryBuilder::from(vec![1, 2]).intersperse(0).collect();
    assert_eq!(result, vec![1, 0, 2]);
}

// ── 5B-3: dedup_by edge cases ─────────────────────────────────────────────────

#[test]
fn dedup_by_all_same_key_returns_first() {
    let result = QueryBuilder::from(vec![5, 5, 5, 5])
        .dedup_by(|&x| x)
        .collect_vec();
    assert_eq!(result, vec![5]);
}

#[test]
fn dedup_by_all_different_returns_all() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4])
        .dedup_by(|&x| x)
        .collect_vec();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn dedup_by_tuple_key_groups_by_first_element() {
    let data = vec![(1, 'a'), (1, 'b'), (2, 'c'), (2, 'd'), (1, 'e')];
    let result = QueryBuilder::from(data).dedup_by(|&(g, _)| g).collect_vec();
    assert_eq!(result, vec![(1, 'a'), (2, 'c'), (1, 'e')]);
}

#[test]
fn dedup_by_string_prefix_key() {
    let data = vec!["apple", "avocado", "banana", "blueberry", "cherry"];
    let result = QueryBuilder::from(data)
        .dedup_by(|s| s.chars().next().unwrap())
        .collect_vec();
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
}

// ── 5B-3: unfold early termination via take ───────────────────────────────────

#[test]
fn unfold_infinite_generator_terminated_by_take() {
    let result = QueryBuilder::<u64, _>::unfold(0u64, |s| Some((s, s + 1)))
        .take(5)
        .collect_vec();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

#[test]
fn unfold_take_zero_returns_empty() {
    let result = QueryBuilder::<u64, _>::unfold(0u64, |s| Some((s, s + 1)))
        .take(0)
        .collect_vec();
    assert!(result.is_empty());
}

#[test]
fn unfold_take_then_where_pipeline() {
    // Even numbers from unfold
    let result = QueryBuilder::<u64, _>::unfold(0u64, |s| Some((s, s + 1)))
        .take(10)
        .where_(|&x| x % 2 == 0)
        .collect_vec();
    assert_eq!(result, vec![0, 2, 4, 6, 8]);
}

// ── 5B-2: rinq-derive + v4 operators ─────────────────────────────────────────

#[cfg(test)]
mod derive_integration {
    use rinq::QueryBuilder;
    use rinq_derive::Queryable;

    #[derive(Queryable, Clone, Debug, PartialEq)]
    struct Product {
        pub id: u32,
        pub price: f64,
        pub category: String,
    }

    fn sample_products() -> Vec<Product> {
        vec![
            Product {
                id: 1,
                price: 10.0,
                category: "A".into(),
            },
            Product {
                id: 2,
                price: 30.0,
                category: "B".into(),
            },
            Product {
                id: 3,
                price: 20.0,
                category: "A".into(),
            },
            Product {
                id: 4,
                price: 40.0,
                category: "B".into(),
            },
            Product {
                id: 5,
                price: 15.0,
                category: "A".into(),
            },
        ]
    }

    #[test]
    fn derive_queryable_with_pairwise() {
        let products = sample_products();
        // pairwise で連続する商品のペア
        let pairs: Vec<(Product, Product)> = QueryBuilder::from(products).pairwise().collect();
        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0].0.id, 1);
        assert_eq!(pairs[0].1.id, 2);
    }

    #[test]
    fn derive_queryable_with_scan_running_sum() {
        let products = sample_products();
        let running: Vec<f64> = QueryBuilder::from(products)
            .scan(0.0f64, |acc, p| acc + p.price)
            .collect();
        assert_eq!(running.len(), 5);
        assert!((running[0] - 10.0).abs() < 1e-10);
        assert!((running[1] - 40.0).abs() < 1e-10);
        assert!((running[4] - 115.0).abs() < 1e-10);
    }

    #[test]
    fn derive_queryable_with_zip_with() {
        let prices_a: Vec<f64> = vec![10.0, 20.0, 30.0];
        let prices_b: Vec<f64> = vec![1.0, 2.0, 3.0];
        let discounted = QueryBuilder::from(prices_a)
            .zip_with(prices_b, |a, b| a - b)
            .collect_vec();
        assert_eq!(discounted, vec![9.0, 18.0, 27.0]);
    }

    #[test]
    fn derive_queryable_field_accessor_with_order_by() {
        // f64 does not implement Ord; use integer id field for order_by
        let products = sample_products();
        let sorted = QueryBuilder::from(products)
            .order_by(Product::by_id)
            .collect_vec();
        assert_eq!(sorted[0].id, 1);
        assert_eq!(sorted[4].id, 5);
    }

    #[test]
    fn derive_queryable_predicate_with_tap_each() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let products = sample_products();
        let result = QueryBuilder::from(products)
            .where_(|p: &Product| p.price > 15.0)
            .tap_each(move |_| {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
            .collect_vec();
        // prices > 15: 30.0, 20.0, 40.0 = 3 items
        assert_eq!(result.len(), 3);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}

// ── 5B-2: rinq-syntax + rinq-derive ──────────────────────────────────────────

#[cfg(test)]
mod syntax_derive_integration {
    use rinq_derive::Queryable;
    use rinq_syntax::query;

    #[derive(Queryable, Clone, Debug, PartialEq)]
    struct Employee {
        pub name: String,
        pub age: u32,
        pub department: String,
        pub salary: f64,
    }

    fn employees() -> Vec<Employee> {
        vec![
            Employee {
                name: "Alice".into(),
                age: 30,
                department: "Eng".into(),
                salary: 90000.0,
            },
            Employee {
                name: "Bob".into(),
                age: 22,
                department: "HR".into(),
                salary: 50000.0,
            },
            Employee {
                name: "Carol".into(),
                age: 35,
                department: "Eng".into(),
                salary: 110000.0,
            },
            Employee {
                name: "Dave".into(),
                age: 28,
                department: "HR".into(),
                salary: 55000.0,
            },
            Employee {
                name: "Eve".into(),
                age: 40,
                department: "Eng".into(),
                salary: 120000.0,
            },
        ]
    }

    #[test]
    fn query_macro_with_derived_struct_filter_and_select() {
        let result = query! {
            from emp in employees()
            where emp.age >= 30
            select emp.name.clone()
        };
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"Alice".to_string()));
        assert!(result.contains(&"Carol".to_string()));
        assert!(result.contains(&"Eve".to_string()));
    }

    #[test]
    fn query_macro_order_by_with_derived_struct() {
        // f64 does not implement Ord; sort by age (u32) instead
        let result = query! {
            from emp in employees()
            where emp.department == "Eng"
            order_by emp.age
            select emp.name.clone()
        };
        // Eng employees sorted by age: Alice(30), Carol(35), Eve(40)
        assert_eq!(result, vec!["Alice", "Carol", "Eve"]);
    }

    #[test]
    fn query_macro_take_skip_with_derived_struct() {
        let result = query! {
            from emp in employees()
            order_by emp.age desc
            take 3
            select emp.name.clone()
        };
        assert_eq!(result.len(), 3);
        // Top 3 by age descending: Eve(40), Carol(35), Alice(30)
        assert_eq!(result[0], "Eve");
        assert_eq!(result[1], "Carol");
        assert_eq!(result[2], "Alice");
    }
}

// ── 5B-2: large data smoke test ───────────────────────────────────────────────

#[test]
fn large_data_filter_sort_count() {
    let data: Vec<i32> = (0..100_000).collect();
    let count = QueryBuilder::from(data)
        .where_(|&x| x % 2 == 0)
        .order_by(|x| *x)
        .count();
    assert_eq!(count, 50_000);
}

#[test]
fn large_data_group_by() {
    let data: Vec<i32> = (0..10_000).collect();
    let groups = QueryBuilder::from(data).group_by(|&x| x % 10);
    assert_eq!(groups.len(), 10);
    for (_, v) in &groups {
        assert_eq!(v.len(), 1_000);
    }
}

// ── 5B-2: parallel feature combination ───────────────────────────────────────

#[cfg(feature = "parallel")]
mod parallel_tests {
    use rinq::ParallelQueryBuilder;

    #[test]
    fn parallel_filter_sum_large() {
        let data: Vec<i64> = (0..100_000_i64).collect();
        let result: i64 = ParallelQueryBuilder::from(data)
            .par_where(|&x| x % 2 == 0)
            .par_sum();
        let expected: i64 = (0..100_000_i64).filter(|x| x % 2 == 0).sum();
        assert_eq!(result, expected);
    }

    #[test]
    fn parallel_count_equals_sequential() {
        let data: Vec<i32> = (0..50_000).collect();
        let par_count = ParallelQueryBuilder::from(data.clone())
            .par_where(|&x| x % 3 == 0)
            .par_count();
        let seq_count = rinq::QueryBuilder::from(data)
            .where_(|&x| x % 3 == 0)
            .count();
        assert_eq!(par_count, seq_count);
    }
}

// ── 5B-2: serde feature combination ──────────────────────────────────────────

#[cfg(feature = "serde")]
mod serde_tests {
    use rinq::QueryBuilder;

    #[test]
    fn from_json_filter_collect() {
        let json = r#"[1, 2, 3, 4, 5, 6]"#;
        let result: Vec<i64> = QueryBuilder::from_json(json)
            .unwrap()
            .where_(|&x| x % 2 == 0)
            .collect();
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn from_json_empty_array() {
        let result: Vec<i64> = QueryBuilder::from_json("[]").unwrap().collect();
        assert!(result.is_empty());
    }
}

// ── 5B-2: MetricsQueryBuilder combination ────────────────────────────────────

mod metrics_tests {
    use rinq::{MetricsCollector, MetricsQueryBuilder, QueryBuilder};
    use std::sync::Arc;

    fn make_metrics_query(
        data: Vec<i32>,
        name: &str,
        collector: Arc<MetricsCollector>,
    ) -> MetricsQueryBuilder<i32, rinq::Initial> {
        MetricsQueryBuilder::new(QueryBuilder::from(data), collector, name.to_string())
    }

    #[test]
    fn metrics_records_count_terminal() {
        let collector = Arc::new(MetricsCollector::new());
        let result = make_metrics_query(vec![1, 2, 3, 4, 5], "even_count", collector.clone())
            .where_(|&x| x % 2 == 0)
            .count();
        assert_eq!(result, 2);
        assert_eq!(collector.get("query_even_count_count"), Some(1));
    }

    #[test]
    fn metrics_records_collect_terminal() {
        let collector = Arc::new(MetricsCollector::new());
        let _result: Vec<i32> = make_metrics_query(vec![1, 2, 3], "my_query", collector.clone())
            .where_(|_| true)
            .collect();
        assert_eq!(collector.get("query_my_query"), Some(1));
    }

    #[test]
    fn metrics_multiple_queries_accumulate() {
        let collector = Arc::new(MetricsCollector::new());
        for _ in 0..5 {
            let _: Vec<i32> = make_metrics_query(vec![1, 2, 3], "repeated", collector.clone())
                .where_(|_| true)
                .collect();
        }
        assert_eq!(collector.get("query_repeated"), Some(5));
    }
}

// ── Phase 5F: JOIN operations ─────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct User {
    id: u32,
    name: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct Order {
    user_id: u32,
    amount: u32,
}

#[test]
fn inner_join_basic_matching() {
    let users = vec![
        User {
            id: 1,
            name: "Alice",
        },
        User { id: 2, name: "Bob" },
    ];
    let orders = vec![
        Order {
            user_id: 1,
            amount: 100,
        },
        Order {
            user_id: 1,
            amount: 200,
        },
    ];

    let mut result: Vec<(&str, u32)> = QueryBuilder::from(users)
        .inner_join(orders, |u| u.id, |o| o.user_id)
        .select(|(u, o)| (u.name, o.amount))
        .collect();
    result.sort_by_key(|&(_, a)| a);
    assert_eq!(result, vec![("Alice", 100), ("Alice", 200)]);
}

#[test]
fn inner_join_no_match_excluded() {
    let users = vec![
        User {
            id: 1,
            name: "Alice",
        },
        User { id: 2, name: "Bob" },
    ];
    let orders = vec![Order {
        user_id: 1,
        amount: 50,
    }];

    let result: Vec<(&str, u32)> = QueryBuilder::from(users)
        .inner_join(orders, |u| u.id, |o| o.user_id)
        .select(|(u, o)| (u.name, o.amount))
        .collect();
    // Bob (id=2) has no matching order → excluded
    assert_eq!(result, vec![("Alice", 50)]);
}

#[test]
fn inner_join_empty_right_returns_empty() {
    let users: Vec<User> = vec![User {
        id: 1,
        name: "Alice",
    }];
    let orders: Vec<Order> = vec![];

    let result: Vec<(&str, u32)> = QueryBuilder::from(users)
        .inner_join(orders, |u| u.id, |o| o.user_id)
        .select(|(u, o)| (u.name, o.amount))
        .collect();
    assert!(result.is_empty());
}

#[test]
fn left_join_all_matched() {
    let users = vec![User {
        id: 1,
        name: "Alice",
    }];
    let orders = vec![Order {
        user_id: 1,
        amount: 99,
    }];

    let result: Vec<(&str, Option<u32>)> = QueryBuilder::from(users)
        .left_join(orders, |u| u.id, |o| o.user_id)
        .select(|(u, o)| (u.name, o.map(|x| x.amount)))
        .collect();
    assert_eq!(result, vec![("Alice", Some(99))]);
}

#[test]
fn left_join_unmatched_is_none() {
    let users = vec![
        User {
            id: 1,
            name: "Alice",
        },
        User { id: 2, name: "Bob" },
    ];
    let orders = vec![Order {
        user_id: 1,
        amount: 50,
    }];

    let result: Vec<(&str, Option<u32>)> = QueryBuilder::from(users)
        .left_join(orders, |u| u.id, |o| o.user_id)
        .select(|(u, o)| (u.name, o.map(|x| x.amount)))
        .collect();
    assert_eq!(result[0], ("Alice", Some(50)));
    assert_eq!(result[1], ("Bob", None));
}

#[test]
fn cross_join_cartesian_product() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2])
        .cross_join(vec![10, 20, 30])
        .collect();
    assert_eq!(result.len(), 6);
    assert_eq!(result[0], (1, 10));
    assert_eq!(result[1], (1, 20));
    assert_eq!(result[2], (1, 30));
    assert_eq!(result[3], (2, 10));
    assert_eq!(result[4], (2, 20));
    assert_eq!(result[5], (2, 30));
}

#[test]
fn cross_join_empty_right_returns_empty() {
    let result: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 2])
        .cross_join(Vec::<i32>::new())
        .collect();
    assert!(result.is_empty());
}

// ── join + query! macro ───────────────────────────────────────────────────────

mod join_syntax_tests {
    use rinq_syntax::query;

    #[derive(Clone, Debug, PartialEq)]
    struct Dept {
        id: u32,
        name: &'static str,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Emp {
        dept_id: u32,
        name: &'static str,
    }

    #[test]
    fn query_macro_inner_join() {
        let depts = vec![
            Dept {
                id: 1,
                name: "Engineering",
            },
            Dept {
                id: 2,
                name: "Marketing",
            },
        ];
        let emps = vec![
            Emp {
                dept_id: 1,
                name: "Alice",
            },
            Emp {
                dept_id: 1,
                name: "Bob",
            },
            Emp {
                dept_id: 2,
                name: "Carol",
            },
        ];

        let mut result: Vec<(&str, &str)> = query! {
            from dept in depts
            join emp in emps on dept.id == emp.dept_id
            select (dept.name, emp.name)
        };
        result.sort();
        assert_eq!(
            result,
            vec![
                ("Engineering", "Alice"),
                ("Engineering", "Bob"),
                ("Marketing", "Carol"),
            ]
        );
    }

    #[test]
    fn query_macro_left_join() {
        let depts = vec![
            Dept {
                id: 1,
                name: "Engineering",
            },
            Dept { id: 3, name: "HR" }, // no matching emp
        ];
        let emps = vec![Emp {
            dept_id: 1,
            name: "Alice",
        }];

        let result: Vec<(&str, Option<&str>)> = query! {
            from dept in depts
            left join emp in emps on dept.id == emp.dept_id
            select (dept.name, emp.map(|e| e.name))
        };
        assert_eq!(result[0], ("Engineering", Some("Alice")));
        assert_eq!(result[1], ("HR", None));
    }

    #[test]
    fn query_macro_join_with_where() {
        let depts = vec![
            Dept {
                id: 1,
                name: "Engineering",
            },
            Dept {
                id: 2,
                name: "Marketing",
            },
        ];
        let emps = vec![
            Emp {
                dept_id: 1,
                name: "Alice",
            },
            Emp {
                dept_id: 2,
                name: "Bob",
            },
        ];

        let result: Vec<(&str, &str)> = query! {
            from dept in depts
            join emp in emps on dept.id == emp.dept_id
            where dept.id == 1
            select (dept.name, emp.name)
        };
        assert_eq!(result, vec![("Engineering", "Alice")]);
    }
}

// ── Phase 5E: query enrichment ────────────────────────────────────────────────

#[test]
fn frequencies_counts_occurrences() {
    let freq = QueryBuilder::from(vec!["a", "b", "a", "c", "b", "a"]).frequencies();
    assert_eq!(freq[&"a"], 3);
    assert_eq!(freq[&"b"], 2);
    assert_eq!(freq[&"c"], 1);
}

#[test]
fn frequencies_empty_returns_empty_map() {
    let freq = QueryBuilder::from(Vec::<i32>::new()).frequencies();
    assert!(freq.is_empty());
}

#[test]
fn frequencies_all_unique() {
    let freq = QueryBuilder::from(vec![1, 2, 3]).frequencies();
    assert!(freq.values().all(|&v| v == 1));
}

#[test]
fn flatten_nested_vecs() {
    let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
    let flat: Vec<i32> = QueryBuilder::from(nested).flatten().collect();
    assert_eq!(flat, vec![1, 2, 3, 4, 5]);
}

#[test]
fn flatten_empty_outer() {
    let nested: Vec<Vec<i32>> = vec![];
    let flat: Vec<i32> = QueryBuilder::from(nested).flatten().collect();
    assert_eq!(flat, Vec::<i32>::new());
}

#[test]
fn flatten_with_empty_inner() {
    let nested = vec![vec![], vec![1, 2], vec![]];
    let flat: Vec<i32> = QueryBuilder::from(nested).flatten().collect();
    assert_eq!(flat, vec![1, 2]);
}

#[test]
fn position_finds_first_matching_index() {
    let pos = QueryBuilder::from(vec![10, 20, 30, 40]).position(|x| *x == 30);
    assert_eq!(pos, Some(2));
}

#[test]
fn position_returns_none_when_not_found() {
    let pos = QueryBuilder::from(vec![10, 20, 30]).position(|x| *x == 99);
    assert_eq!(pos, None);
}

#[test]
fn position_returns_first_match_index() {
    // 20 appears at index 1 and 3; position returns the first (1)
    let pos = QueryBuilder::from(vec![10, 20, 30, 20]).position(|x| *x == 20);
    assert_eq!(pos, Some(1));
}

#[test]
fn find_returns_first_matching_element() {
    let found = QueryBuilder::from(vec![1, 2, 3, 4, 5]).find(|x| *x > 3);
    assert_eq!(found, Some(4));
}

#[test]
fn find_returns_none_when_not_found() {
    let not_found = QueryBuilder::from(vec![1, 2, 3]).find(|x| *x > 10);
    assert_eq!(not_found, None);
}

#[test]
fn find_is_equivalent_to_where_first() {
    let data = vec![1, 2, 3, 4, 5];
    let via_find = QueryBuilder::from(data.clone()).find(|x| *x % 2 == 0);
    let via_where_first: Option<i32> = QueryBuilder::from(data).where_(|x| x % 2 == 0).first();
    assert_eq!(via_find, via_where_first);
}

#[test]
fn index_of_returns_first_occurrence() {
    let idx = QueryBuilder::from(vec![10, 20, 30, 20]).index_of(&20);
    assert_eq!(idx, Some(1));
}

#[test]
fn index_of_returns_none_when_absent() {
    let none = QueryBuilder::from(vec![1, 2, 3]).index_of(&99);
    assert_eq!(none, None);
}

#[test]
fn nth_is_alias_for_element_at() {
    let data = vec![10, 20, 30, 40];
    assert_eq!(QueryBuilder::from(data.clone()).nth(2), Some(30));
    assert_eq!(QueryBuilder::from(data.clone()).element_at(2), Some(30));
    assert_eq!(QueryBuilder::from(data.clone()).nth(10), None);
}

#[test]
fn batch_splits_into_fixed_size_groups() {
    let batches: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4, 5]).batch(2).collect();
    assert_eq!(batches, vec![vec![1, 2], vec![3, 4], vec![5]]);
}

#[test]
fn batch_equal_division() {
    let batches: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2, 3, 4]).batch(2).collect();
    assert_eq!(batches, vec![vec![1, 2], vec![3, 4]]);
}

#[test]
fn batch_larger_than_len() {
    let batches: Vec<Vec<i32>> = QueryBuilder::from(vec![1, 2]).batch(10).collect();
    assert_eq!(batches, vec![vec![1, 2]]);
}

#[test]
fn exactly_one_is_alias_for_single() {
    assert_eq!(QueryBuilder::from(vec![42]).exactly_one(), Ok(42));
    assert!(QueryBuilder::from(vec![1, 2]).exactly_one().is_err());
    assert!(QueryBuilder::from(Vec::<i32>::new()).exactly_one().is_err());
}

#[test]
fn tee_produces_two_identical_vecs() {
    let (a, b) = QueryBuilder::from(vec![1, 2, 3]).tee();
    assert_eq!(a, vec![1, 2, 3]);
    assert_eq!(b, vec![1, 2, 3]);
}

#[test]
fn tee_vecs_are_independent() {
    let (mut a, b) = QueryBuilder::from(vec![1, 2, 3]).tee();
    a.push(99);
    // b is unchanged
    assert_eq!(b, vec![1, 2, 3]);
    assert_eq!(a, vec![1, 2, 3, 99]);
}

#[test]
fn tee_empty_collection() {
    let (a, b) = QueryBuilder::from(Vec::<i32>::new()).tee();
    assert!(a.is_empty());
    assert!(b.is_empty());
}

// ── Phase 5D: terminal operator enhancements ─────────────────────────────────

#[test]
fn for_each_applies_to_all_elements() {
    let mut sum = 0i32;
    QueryBuilder::from(vec![1, 2, 3, 4, 5]).for_each(|x| sum += x);
    assert_eq!(sum, 15);
}

#[test]
fn for_each_empty_collection() {
    let mut called = false;
    QueryBuilder::from(Vec::<i32>::new()).for_each(|_| called = true);
    assert!(!called);
}

#[test]
fn for_each_after_where_filter() {
    let mut collected = Vec::new();
    QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .where_(|x| x % 2 == 0)
        .for_each(|x| collected.push(x));
    assert_eq!(collected, vec![2, 4]);
}

#[test]
fn to_sorted_vec_ascending() {
    let result = QueryBuilder::from(vec![3, 1, 4, 1, 5, 9, 2]).to_sorted_vec(|x| *x);
    assert_eq!(result, vec![1, 1, 2, 3, 4, 5, 9]);
}

#[test]
fn to_sorted_vec_matches_order_by_collect() {
    let data = vec![5, 3, 8, 1, 9, 2, 7];
    let via_sorted = QueryBuilder::from(data.clone()).to_sorted_vec(|x| *x);
    let via_order_by: Vec<i32> = QueryBuilder::from(data).order_by(|x| *x).collect();
    assert_eq!(via_sorted, via_order_by);
}

#[test]
fn to_sorted_vec_desc_descending() {
    let result = QueryBuilder::from(vec![3, 1, 4, 1, 5]).to_sorted_vec_desc(|x| *x);
    assert_eq!(result, vec![5, 4, 3, 1, 1]);
}

#[test]
fn to_sorted_vec_empty() {
    let result = QueryBuilder::from(Vec::<i32>::new()).to_sorted_vec(|x| *x);
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn take_last_returns_last_n() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5]).take_last(3);
    assert_eq!(result, vec![3, 4, 5]);
}

#[test]
fn take_last_more_than_len_returns_all() {
    let result = QueryBuilder::from(vec![1, 2]).take_last(10);
    assert_eq!(result, vec![1, 2]);
}

#[test]
fn take_last_zero_returns_empty() {
    let result = QueryBuilder::from(vec![1, 2, 3]).take_last(0);
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn skip_last_removes_last_n() {
    let result = QueryBuilder::from(vec![1, 2, 3, 4, 5]).skip_last(2);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn skip_last_more_than_len_returns_empty() {
    let result = QueryBuilder::from(vec![1, 2]).skip_last(5);
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn skip_last_zero_returns_all() {
    let result = QueryBuilder::from(vec![1, 2, 3]).skip_last(0);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn count_by_counts_matching_elements() {
    let count = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6]).count_by(|x| x % 2 == 0);
    assert_eq!(count, 3);
}

#[test]
fn count_by_none_match() {
    let count = QueryBuilder::from(vec![1, 3, 5]).count_by(|x| x % 2 == 0);
    assert_eq!(count, 0);
}

#[test]
fn count_by_all_match() {
    let count = QueryBuilder::from(vec![2, 4, 6]).count_by(|x| x % 2 == 0);
    assert_eq!(count, 3);
}

#[test]
fn sum_by_field_extraction() {
    #[derive(Clone)]
    struct Item {
        value: i32,
    }
    let total = QueryBuilder::from(vec![
        Item { value: 10 },
        Item { value: 20 },
        Item { value: 30 },
    ])
    .sum_by(|i| i.value);
    assert_eq!(total, 60_i32);
}

#[test]
fn sum_by_empty_is_default() {
    let total: i32 = QueryBuilder::from(Vec::<i32>::new()).sum_by(|x| x);
    assert_eq!(total, 0);
}

#[test]
fn average_by_basic() {
    let avg = QueryBuilder::from(vec![1.0_f64, 2.0, 3.0]).average_by(|x| *x);
    assert_eq!(avg, Some(2.0));
}

#[test]
fn average_by_empty_is_none() {
    let avg = QueryBuilder::from(Vec::<f64>::new()).average_by(|x| *x);
    assert_eq!(avg, None);
}

#[test]
fn average_by_field_extraction() {
    #[derive(Clone)]
    struct Score {
        points: f64,
    }
    let avg = QueryBuilder::from(vec![
        Score { points: 80.0 },
        Score { points: 90.0 },
        Score { points: 100.0 },
    ])
    .average_by(|s| s.points);
    assert_eq!(avg, Some(90.0));
}

#[test]
fn reduce_is_alias_for_aggregate_no_seed() {
    let max_reduce =
        QueryBuilder::from(vec![3, 1, 4, 1, 5]).reduce(|a, b| if a > b { a } else { b });
    let max_agg =
        QueryBuilder::from(vec![3, 1, 4, 1, 5]).aggregate_no_seed(|a, b| if a > b { a } else { b });
    assert_eq!(max_reduce, max_agg);
}

#[test]
fn reduce_empty_returns_none() {
    let result = QueryBuilder::from(Vec::<i32>::new()).reduce(|a, b| a + b);
    assert_eq!(result, None);
}

#[test]
fn all_unique_no_duplicates() {
    assert!(QueryBuilder::from(vec![1, 2, 3, 4, 5]).all_unique());
}

#[test]
fn all_unique_with_duplicates() {
    assert!(!QueryBuilder::from(vec![1, 2, 2, 3]).all_unique());
}

#[test]
fn all_unique_empty_is_true() {
    assert!(QueryBuilder::from(Vec::<i32>::new()).all_unique());
}

#[test]
fn all_unique_strings() {
    assert!(QueryBuilder::from(vec!["a", "b", "c"]).all_unique());
    assert!(!QueryBuilder::from(vec!["a", "b", "a"]).all_unique());
}

#[test]
fn none_when_no_element_matches() {
    assert!(QueryBuilder::from(vec![1, 2, 3]).none(|x| *x > 10));
}

#[test]
fn none_when_some_element_matches() {
    assert!(!QueryBuilder::from(vec![1, 2, 3]).none(|x| *x > 2));
}

#[test]
fn none_empty_collection_is_true() {
    assert!(QueryBuilder::from(Vec::<i32>::new()).none(|x| *x > 0));
}

#[test]
fn none_is_negation_of_any() {
    let data = vec![1, 2, 3, 4, 5];
    let pred = |x: &i32| *x > 3;
    let any_result = QueryBuilder::from(data.clone()).any(pred);
    let none_result = QueryBuilder::from(data).none(pred);
    assert_eq!(none_result, !any_result);
}
