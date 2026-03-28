// rinq/examples/parallel_example.rs
// Demonstrates ParallelQueryBuilder (requires the "parallel" feature).

fn main() {
    #[cfg(feature = "parallel")]
    {
        use rinq::ParallelQueryBuilder;

        println!("=== RINQ Parallel Operations ===\n");

        let data: Vec<i32> = (1..=1_000_000).collect();

        let sum: i32 = ParallelQueryBuilder::from(data.clone())
            .par_where(|x| x % 2 == 0)
            .par_sum();

        println!("Sum of even values in 1..=1_000_000: {sum}");

        let count = ParallelQueryBuilder::from(data)
            .par_where(|x| *x % 3 == 0)
            .par_count();

        println!("Count of multiples of 3: {count}");
    }

    #[cfg(not(feature = "parallel"))]
    {
        println!("Run with --features parallel to see this example.");
        println!("  cargo run --example parallel_example --features parallel");
    }
}
