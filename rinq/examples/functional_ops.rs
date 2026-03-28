// rinq/examples/functional_ops.rs
// Demonstrates functional operations: scan, pairwise, zip_with, unfold_bounded, intersperse, dedup.

use rinq::QueryBuilder;

fn main() {
    println!("=== RINQ Functional Operations ===\n");

    // scan — running product
    let product: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
        .scan(1, |acc, x| acc * x)
        .collect();
    println!("Running product: {:?}", product);

    // pairwise — returns consecutive (T, T) pairs, then compute diffs
    let pairs: Vec<(i32, i32)> = QueryBuilder::from(vec![1, 3, 6, 10, 15])
        .pairwise()
        .collect();
    let diffs: Vec<i32> = pairs.iter().map(|(a, b)| b - a).collect();
    println!("Pairwise diffs:  {:?}", diffs);

    // zip_with — plain Vec as the right side
    let a = vec![1_i32, 2, 3];
    let b = vec![10_i32, 20, 30];
    let sums: Vec<i32> = QueryBuilder::from(a).zip_with(b, |x, y| x + y).collect();
    println!("Zip sum:         {:?}", sums);

    // unfold_bounded — Fibonacci (state = (a, b), emit a)
    let fib: Vec<u64> =
        QueryBuilder::unfold_bounded((0_u64, 1_u64), |(a, b)| Some((a, (b, a + b))), 8).collect();
    println!("Fibonacci(8):    {:?}", fib);

    // intersperse
    let words: Vec<&str> = QueryBuilder::from(vec!["hello", "world", "rinq"])
        .intersperse("-")
        .collect();
    println!("Intersperse:     {:?}", words);

    // dedup
    let deduped: Vec<i32> = QueryBuilder::from(vec![1, 1, 2, 3, 3, 3, 4])
        .dedup()
        .collect();
    println!("Dedup:           {:?}", deduped);
}
