// examples/rinq_basic_usage.rs
// Basic usage example for RINQ v2.0

use rinq::QueryBuilder;

fn main() {
    println!("=== RINQ v2.0 Basic Usage Examples ===\n");

    // Example 1: Basic filtering
    println!("Example 1: Filtering even numbers");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let even_numbers: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("Input: {:?}", numbers);
    println!("Even numbers: {:?}\n", even_numbers);

    // Example 2: Chaining filters
    println!("Example 2: Chaining multiple filters");
    let filtered: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .where_(|x| *x > 4)
        .collect();
    println!("Even numbers > 4: {:?}\n", filtered);

    // Example 3: Projection (select)
    println!("Example 3: Projection");
    let doubled: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .select(|x| x * 2)
        .collect();
    println!("Even numbers doubled: {:?}\n", doubled);

    // Example 4: Sorting
    println!("Example 4: Sorting (ascending and descending)");
    let unsorted = vec![5, 2, 8, 1, 9, 3];
    let sorted_asc: Vec<_> = QueryBuilder::from(unsorted.clone())
        .where_(|_| true)
        .order_by(|x| *x)
        .collect();
    let sorted_desc: Vec<_> = QueryBuilder::from(unsorted.clone())
        .order_by_descending(|x| *x)
        .collect();
    println!("Unsorted: {:?}", unsorted);
    println!("Ascending: {:?}", sorted_asc);
    println!("Descending: {:?}\n", sorted_desc);

    // Example 5: Pagination
    println!("Example 5: Pagination");
    let page: Vec<_> = QueryBuilder::from(numbers.clone())
        .where_(|_| true)
        .skip(3)
        .take(4)
        .collect();
    println!("Skip 3, take 4: {:?}\n", page);

    // Example 6: take_while / skip_while
    println!("Example 6: take_while / skip_while");
    let taken: Vec<_> = QueryBuilder::from(numbers.clone())
        .take_while(|x| *x < 6)
        .collect();
    let skipped: Vec<_> = QueryBuilder::from(numbers.clone())
        .skip_while(|x| *x < 6)
        .collect();
    println!("take_while < 6: {:?}", taken);
    println!("skip_while < 6: {:?}\n", skipped);

    // Example 7: flat_map
    println!("Example 7: flat_map");
    let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
    let flat: Vec<i32> = QueryBuilder::from(nested)
        .flat_map(|v| v)
        .collect();
    println!("Flattened: {:?}\n", flat);

    // Example 8: Scalar aggregations
    println!("Example 8: Aggregations");
    let count = QueryBuilder::from(numbers.clone()).count();
    let sum: i32 = QueryBuilder::from(numbers.clone()).sum();
    let avg = QueryBuilder::from(numbers.clone()).average().unwrap();
    let min = QueryBuilder::from(numbers.clone()).min();
    let max = QueryBuilder::from(numbers.clone()).max();
    println!("Count: {count}, Sum: {sum}, Avg: {avg}, Min: {min:?}, Max: {max:?}\n");

    // Example 9: aggregate / aggregate_no_seed
    println!("Example 9: aggregate / aggregate_no_seed");
    let product = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .aggregate(1, |acc, x| acc * x);
    let max_custom = QueryBuilder::from(vec![3, 1, 4, 1, 5])
        .aggregate_no_seed(|a, b| if a > b { a } else { b });
    println!("Product 1..5: {product}");
    println!("Max via aggregate_no_seed: {max_custom:?}\n");

    // Example 10: contains / first_or_default / single
    println!("Example 10: contains / first_or_default / single");
    let has_7 = QueryBuilder::from(numbers.clone()).contains(&7);
    let default: i32 = QueryBuilder::from(Vec::<i32>::new()).first_or_default();
    let only = QueryBuilder::from(vec![42]).single();
    println!("Contains 7: {has_7}");
    println!("first_or_default of empty: {default}");
    println!("single([42]): {only:?}\n");

    // Example 11: concat / union / intersect / except
    println!("Example 11: Set operations");
    let a = vec![1, 2, 3, 4];
    let b = vec![3, 4, 5, 6];
    let concatenated: Vec<_> = QueryBuilder::from(a.clone()).concat(b.clone()).collect();
    let mut union_result: Vec<_> = QueryBuilder::from(a.clone()).union(b.clone()).collect();
    let mut intersect_result: Vec<_> =
        QueryBuilder::from(a.clone()).intersect(b.clone()).collect();
    let mut except_result: Vec<_> = QueryBuilder::from(a.clone()).except(b.clone()).collect();
    union_result.sort();
    intersect_result.sort();
    except_result.sort();
    println!("a={a:?}, b={b:?}");
    println!("concat: {concatenated:?}");
    println!("union:  {union_result:?}");
    println!("intersect: {intersect_result:?}");
    println!("except: {except_result:?}\n");

    // Example 12: to_hashmap / to_lookup
    println!("Example 12: to_hashmap / to_lookup");
    let pairs = vec![(1, "one"), (2, "two"), (3, "three")];
    let map = QueryBuilder::from(pairs).to_hashmap(|(k, _)| *k).unwrap();
    println!("to_hashmap[2] = {:?}", map[&2]);
    let words = vec!["apple", "ant", "banana", "blueberry", "avocado"];
    let lookup = QueryBuilder::from(words).to_lookup(|w| w.chars().next().unwrap());
    println!("to_lookup['a'] = {:?}\n", lookup[&'a']);

    // Example 13: Generation operators
    println!("Example 13: Generation operators");
    let range: Vec<i32> = QueryBuilder::range(1..=5).collect();
    let repeated: Vec<i32> = QueryBuilder::repeat(0, 4).collect();
    let empty: Vec<i32> = QueryBuilder::empty().collect();
    println!("range(1..=5): {range:?}");
    println!("repeat(0, 4): {repeated:?}");
    println!("empty: {empty:?}\n");

    // Example 14: element_at / distinct / distinct_by
    println!("Example 14: element_at / distinct / distinct_by");
    let third = QueryBuilder::from(numbers.clone()).element_at(2);
    let dupes = vec![1, 2, 2, 3, 1, 4];
    let unique: Vec<_> = QueryBuilder::from(dupes).distinct().collect();
    println!("element_at(2) = {third:?}");
    println!("distinct: {unique:?}\n");

    // Example 15: Type state pattern demonstration
    println!("Example 15: Type state pattern ensures compile-time safety");
    let _q1 = QueryBuilder::from(numbers.clone()).where_(|x| x % 2 == 0);
    println!("✓ Initial -> Filtered (where_)");
    let _q2 = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .order_by(|x| *x);
    println!("✓ Filtered -> Sorted (order_by)");
    let _q3 = QueryBuilder::from(numbers.clone())
        .where_(|x| x % 2 == 0)
        .select(|x| x * 2);
    println!("✓ Filtered -> Projected (select)");
    println!("\nThe type system prevents invalid operations at compile time!");
}
