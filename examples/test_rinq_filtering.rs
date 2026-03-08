// Example to test RINQ filtering functionality
// This can be run independently to verify the implementation

use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    println!("Testing RINQ Filtering Functionality\n");

    // Test 1: Basic where_() filtering
    println!("Test 1: Basic where_() filtering");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("  Input: {:?}", data);
    println!("  Filter: even numbers");
    println!("  Result: {:?}", result);
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
    println!("  ✓ PASSED\n");

    // Test 2: Chained where_() calls
    println!("Test 2: Chained where_() calls");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| *x % 2 == 0) // even numbers
        .where_(|x| *x > 5) // greater than 5
        .where_(|x| *x < 15) // less than 15
        .collect();
    println!("  Input: {:?}", data);
    println!("  Filters: even AND > 5 AND < 15");
    println!("  Result: {:?}", result);
    assert_eq!(result, vec![6, 8, 10, 12, 14]);
    println!("  ✓ PASSED\n");

    // Test 3: All elements satisfy predicate
    println!("Test 3: All elements satisfy predicate");
    let data = vec![2, 4, 6, 8, 10];
    let result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("  Input: {:?}", data);
    println!("  Filter: even numbers");
    println!("  Result: {:?}", result);
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
    assert!(result.iter().all(|x| x % 2 == 0));
    println!("  ✓ PASSED\n");

    // Test 4: No elements satisfy predicate
    println!("Test 4: No elements satisfy predicate");
    let data = vec![1, 3, 5, 7, 9];
    let result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("  Input: {:?}", data);
    println!("  Filter: even numbers");
    println!("  Result: {:?}", result);
    assert_eq!(result, Vec::<i32>::new());
    println!("  ✓ PASSED\n");

    // Test 5: Empty collection
    println!("Test 5: Empty collection");
    let data: Vec<i32> = vec![];
    let result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("  Input: {:?}", data);
    println!("  Filter: even numbers");
    println!("  Result: {:?}", result);
    assert_eq!(result, Vec::<i32>::new());
    println!("  ✓ PASSED\n");

    // Test 6: Order independence of chained filters
    println!("Test 6: Order independence of chained filters");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result1: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| *x % 2 == 0)
        .where_(|x| *x > 5)
        .collect();
    let result2: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| *x > 5)
        .where_(|x| *x % 2 == 0)
        .collect();
    println!("  Input: {:?}", data);
    println!("  Result1 (even then > 5): {:?}", result1);
    println!("  Result2 (> 5 then even): {:?}", result2);
    assert_eq!(result1, result2);
    println!("  ✓ PASSED\n");

    // Test 7: Immutability - original data unchanged
    println!("Test 7: Immutability - original data unchanged");
    let data = vec![1, 2, 3, 4, 5];
    let original = data.clone();
    let _result: Vec<_> = QueryBuilder::from(data.clone())
        .where_(|x| x % 2 == 0)
        .collect();
    println!("  Original: {:?}", original);
    println!("  After query: {:?}", data);
    assert_eq!(data, original);
    println!("  ✓ PASSED\n");

    println!("All tests passed! ✓");
}
