# Quick Start: RINQ v0.2

**Feature**: RINQ v0.2 - Aggregation and Transformation Extensions  
**Prerequisite**: RINQ v0.1 installed (see `src/domain/rinq/README.md`)

## Installation

Add dependency (if needed):

```toml
[dependencies]
num-traits = "0.2"  # For numeric operations
```

RINQ v0.2 is a backward-compatible extension of v0.1. All v0.1 features continue to work.

---

## 5-Minute Quick Start

### Example 1: Calculate Summary Statistics

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    let sales = vec![100, 250, 175, 300, 225, 400];
    
    // Calculate total revenue
    let total: i32 = QueryBuilder::from(sales.clone())
        .sum();
    println!("Total: ${}", total);  // Total: $1450
    
    // Calculate average sale
    let avg = QueryBuilder::from(sales.clone())
        .average()
        .unwrap();
    println!("Average: ${:.2}", avg);  // Average: $241.67
    
    // Find highest and lowest sales
    let max_sale = QueryBuilder::from(sales.clone())
        .max()
        .unwrap();
    let min_sale = QueryBuilder::from(sales.clone())
        .min()
        .unwrap();
    
    println!("Range: ${} - ${}", min_sale, max_sale);  
    // Range: $100 - $400
}
```

**Output**:
```
Total: $1450
Average: $241.67
Range: $100 - $400
```

---

### Example 2: Group and Aggregate Data

```rust
use rusted_ca::domain::rinq::QueryBuilder;

#[derive(Debug, Clone)]
struct Order {
    user_id: u32,
    amount: f64,
    category: String,
}

fn main() {
    let orders = vec![
        Order { user_id: 1, amount: 50.0, category: "Books".into() },
        Order { user_id: 2, amount: 30.0, category: "Electronics".into() },
        Order { user_id: 1, amount: 25.0, category: "Books".into() },
        Order { user_id: 2, amount: 100.0, category: "Electronics".into() },
    ];
    
    // Group orders by user and sum amounts
    let totals_by_user = QueryBuilder::from(orders.clone())
        .group_by_aggregate(
            |o| o.user_id,
            |group| group.iter().map(|o| o.amount).sum::<f64>()
        );
    
    println!("User Totals: {:?}", totals_by_user);
    // User Totals: {1: 75.0, 2: 130.0}
    
    // Group by category (just grouping, no aggregation)
    let by_category = QueryBuilder::from(orders)
        .group_by(|o| o.category.clone());
    
    for (category, orders) in by_category {
        println!("{}: {} orders", category, orders.len());
    }
    // Books: 2 orders
    // Electronics: 2 orders
}
```

---

### Example 3: Remove Duplicates

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    // Remove duplicate numbers
    let numbers = vec![1, 2, 2, 3, 3, 3, 4, 5, 5];
    let unique: Vec<i32> = QueryBuilder::from(numbers)
        .distinct()
        .collect();
    
    println!("Unique: {:?}", unique);  
    // Unique: [1, 2, 3, 4, 5]
}
```

**With Custom Keys**:
```rust
#[derive(Debug, Clone)]
struct User {
    id: u32,
    email: String,
    name: String,
}

fn main() {
    let users = vec![
        User { id: 1, email: "alice@example.com".into(), name: "Alice".into() },
        User { id: 2, email: "bob@example.com".into(), name: "Bob".into() },
        User { id: 3, email: "alice@example.com".into(), name: "Alice Duplicate".into() },
    ];
    
    // Remove users with duplicate emails (keep first occurrence)
    let unique_emails: Vec<User> = QueryBuilder::from(users)
        .distinct_by(|u| u.email.clone())
        .collect();
    
    println!("Unique users: {}", unique_emails.len());  
    // Unique users: 2 (alice@example.com kept first occurrence, bob kept)
}
```

---

### Example 4: Batch Processing with Chunks

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Process in batches of 3
    let batches: Vec<Vec<i32>> = QueryBuilder::from(data)
        .chunk(3)
        .collect();
    
    println!("Batches: {:?}", batches);
    // Batches: [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10]]
    
    // Process each batch
    for (i, batch) in batches.iter().enumerate() {
        let batch_sum: i32 = batch.iter().sum();
        println!("Batch {}: sum = {}", i, batch_sum);
    }
    // Batch 0: sum = 6
    // Batch 1: sum = 15
    // Batch 2: sum = 24
    // Batch 3: sum = 10
}
```

---

### Example 5: Sliding Window Analysis

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    // Time series data (e.g., daily temperatures)
    let temperatures = vec![20, 22, 25, 23, 21, 19, 20, 22];
    
    // Calculate 3-day moving average
    let windows: Vec<Vec<i32>> = QueryBuilder::from(temperatures.clone())
        .window(3)
        .collect();
    
    for (i, window) in windows.iter().enumerate() {
        let avg: f64 = window.iter().sum::<i32>() as f64 / window.len() as f64;
        println!("Days {}-{}: avg = {:.1}°C", i, i+2, avg);
    }
    // Days 0-2: avg = 22.3°C
    // Days 1-3: avg = 23.3°C
    // Days 2-4: avg = 23.0°C
    // Days 3-5: avg = 21.0°C
    // Days 4-6: avg = 20.0°C
    // Days 5-7: avg = 20.3°C
}
```

---

### Example 6: Combining Collections

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    let ids = vec![1, 2, 3, 4];
    let names = vec!["Alice", "Bob", "Charlie", "David"];
    
    // Pair IDs with names
    let users: Vec<(i32, &str)> = QueryBuilder::from(ids)
        .zip(names)
        .collect();
    
    for (id, name) in users {
        println!("User {}: {}", id, name);
    }
    // User 1: Alice
    // User 2: Bob
    // User 3: Charlie
    // User 4: David
}
```

**With Index Tracking**:
```rust
fn main() {
    let names = vec!["Alice", "Bob", "Charlie"];
    
    // Add indices
    let indexed: Vec<(usize, &str)> = QueryBuilder::from(names)
        .enumerate()
        .collect();
    
    for (i, name) in indexed {
        println!("[{}] {}", i, name);
    }
    // [0] Alice
    // [1] Bob
    // [2] Charlie
}
```

---

### Example 7: Partition Data by Condition

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Split into evens and odds
    let (evens, odds) = QueryBuilder::from(numbers)
        .partition(|x| x % 2 == 0);
    
    println!("Evens: {:?}", evens);  // Evens: [2, 4, 6, 8, 10]
    println!("Odds: {:?}", odds);    // Odds: [1, 3, 5, 7, 9]
}
```

**With Filtering First**:
```rust
fn main() {
    let scores = vec![45, 67, 89, 92, 55, 73, 81, 39];
    
    // Filter passing scores, then partition into A grades and B grades
    let (a_grades, b_grades) = QueryBuilder::from(scores)
        .where_(|s| *s >= 60)  // Pass threshold
        .partition(|s| *s >= 80);  // A grade threshold
    
    println!("A Grades: {:?}", a_grades);  // A Grades: [89, 92, 81]
    println!("B Grades: {:?}", b_grades);  // B Grades: [67, 73]
}
```

---

## Chaining v0.1 and v0.2 Operations

### Example 8: Filter → Group → Aggregate

```rust
use rusted_ca::domain::rinq::QueryBuilder;

#[derive(Clone)]
struct Transaction {
    user_id: u32,
    amount: f64,
    status: String,
}

fn main() {
    let transactions = vec![
        Transaction { user_id: 1, amount: 100.0, status: "completed".into() },
        Transaction { user_id: 2, amount: 50.0, status: "pending".into() },
        Transaction { user_id: 1, amount: 75.0, status: "completed".into() },
        Transaction { user_id: 2, amount: 200.0, status: "completed".into() },
        Transaction { user_id: 1, amount: 25.0, status: "pending".into() },
    ];
    
    // Find completed transaction totals by user
    let completed_totals = QueryBuilder::from(transactions)
        .where_(|t| t.status == "completed")     // v0.1: Filter
        .group_by_aggregate(                      // v0.2: Group + Aggregate
            |t| t.user_id,
            |group| group.iter().map(|t| t.amount).sum::<f64>()
        );
    
    println!("Completed totals: {:?}", completed_totals);
    // Completed totals: {1: 175.0, 2: 200.0}
}
```

---

### Example 9: Distinct → Sort → Paginate

```rust
use rusted_ca::domain::rinq::QueryBuilder;

fn main() {
    let tags = vec![
        "rust", "async", "rust", "web", "async", 
        "database", "rust", "web", "api"
    ];
    
    // Get unique tags, sort alphabetically, take top 5
    let top_tags: Vec<&str> = QueryBuilder::from(tags)
        .distinct()              // v0.2: Remove duplicates
        .order_by(|t| *t)        // v0.1: Sort alphabetically
        .take(5)                 // v0.1: Paginate
        .collect();
    
    println!("Top tags: {:?}", top_tags);
    // Top tags: ["api", "async", "database", "rust", "web"]
}
```

---

### Example 10: Project → Enumerate → Chunk

```rust
use rusted_ca::domain::rinq::QueryBuilder;

#[derive(Clone)]
struct Product {
    name: String,
    price: f64,
}

fn main() {
    let products = vec![
        Product { name: "Laptop".into(), price: 1200.0 },
        Product { name: "Mouse".into(), price: 25.0 },
        Product { name: "Keyboard".into(), price: 75.0 },
        Product { name: "Monitor".into(), price: 300.0 },
        Product { name: "Headphones".into(), price: 150.0 },
    ];
    
    // Extract prices, add indices, group into pages of 2
    let price_pages: Vec<Vec<(usize, f64)>> = QueryBuilder::from(products)
        .select(|p| p.price)        // v0.1: Project to prices
        .enumerate()                // v0.2: Add indices
        .chunk(2)                   // v0.2: Paginate
        .collect();
    
    for (page_num, page) in price_pages.iter().enumerate() {
        println!("Page {}: {:?}", page_num + 1, page);
    }
    // Page 1: [(0, 1200.0), (1, 25.0)]
    // Page 2: [(2, 75.0), (3, 300.0)]
    // Page 3: [(4, 150.0)]
}
```

---

## Common Patterns

### Pattern 1: Data Analysis Pipeline

**Use Case**: Calculate statistics on filtered data

```rust
let high_value_stats = QueryBuilder::from(orders)
    .where_(|o| o.amount > 1000.0)
    .select(|o| o.amount)
    .collect();  // Could also use sum(), average(), etc.

let total: f64 = QueryBuilder::from(orders.clone())
    .where_(|o| o.amount > 1000.0)
    .select(|o| o.amount)
    .sum();

let average = QueryBuilder::from(orders)
    .where_(|o| o.amount > 1000.0)
    .select(|o| o.amount)
    .average();
```

---

### Pattern 2: Grouping and Reporting

**Use Case**: Create category-based summaries

```rust
// Group users by department, count per department
let dept_counts = QueryBuilder::from(users.clone())
    .group_by_aggregate(
        |u| u.department.clone(),
        |group| group.len()
    );

// Group orders by status, sum amounts
let status_totals = QueryBuilder::from(orders)
    .group_by_aggregate(
        |o| o.status.clone(),
        |group| group.iter().map(|o| o.amount).sum::<f64>()
    );
```

---

### Pattern 3: Data Cleaning Pipeline

**Use Case**: Filter, deduplicate, sort

```rust
let clean_data = QueryBuilder::from(raw_data)
    .where_(|x| x.is_valid())     // v0.1: Remove invalid
    .distinct_by(|x| x.key())     // v0.2: Remove duplicates by key
    .order_by(|x| x.timestamp)    // v0.1: Sort chronologically
    .collect();
```

---

### Pattern 4: Batch Processing

**Use Case**: Process large datasets in chunks

```rust
let data = load_large_dataset();  // 10,000+ items

let chunks: Vec<Vec<Record>> = QueryBuilder::from(data)
    .where_(|r| r.needs_processing())
    .chunk(100)  // Process 100 at a time
    .collect();

for (i, chunk) in chunks.iter().enumerate() {
    process_batch(i, chunk);
}
```

---

### Pattern 5: Time Series Analysis

**Use Case**: Calculate moving averages

```rust
let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0];

let moving_avg_3day: Vec<f64> = QueryBuilder::from(prices)
    .window(3)
    .select(|window| {
        let sum: f64 = window.iter().sum();
        sum / window.len() as f64
    })
    .collect();

println!("3-day moving averages: {:?}", moving_avg_3day);
// [101.0, 102.0, 103.0, 104.0]
```

---

### Pattern 6: Data Correlation

**Use Case**: Combine related datasets

```rust
let user_ids = vec![1, 2, 3, 4];
let usernames = vec!["alice", "bob", "charlie", "david"];
let scores = vec![95, 87, 92, 88];

// Create user records from three parallel arrays
let users: Vec<(u32, &str, i32)> = QueryBuilder::from(user_ids)
    .zip(usernames)
    .zip(scores)
    .select(|((id, name), score)| (id, name, score))
    .collect();
```

---

### Pattern 7: Find Extremes with Context

**Use Case**: Find min/max elements based on a field

```rust
#[derive(Clone, Debug)]
struct Employee {
    name: String,
    age: u32,
    salary: f64,
}

fn main() {
    let employees = vec![
        Employee { name: "Alice".into(), age: 30, salary: 80_000.0 },
        Employee { name: "Bob".into(), age: 25, salary: 65_000.0 },
        Employee { name: "Charlie".into(), age: 35, salary: 95_000.0 },
    ];
    
    // Find youngest employee
    let youngest = QueryBuilder::from(employees.clone())
        .min_by(|e| e.age)
        .unwrap();
    println!("Youngest: {} (age {})", youngest.name, youngest.age);
    // Youngest: Bob (age 25)
    
    // Find highest paid employee
    let highest_paid = QueryBuilder::from(employees)
        .max_by(|e| e.salary as u64)  // Cast to u64 for Ord
        .unwrap();
    println!("Highest paid: {} (${:.2})", highest_paid.name, highest_paid.salary);
    // Highest paid: Charlie ($95000.00)
}
```

---

### Pattern 8: Binary Classification

**Use Case**: Split data into two categories

```rust
let scores = vec![45, 89, 67, 92, 55, 73, 38, 81];

let (passing, failing) = QueryBuilder::from(scores)
    .partition(|s| *s >= 60);

println!("Passing: {:?}", passing);  // [89, 67, 92, 73, 81]
println!("Failing: {:?}", failing);  // [45, 55, 38]

// Calculate statistics on each partition
let passing_avg: f64 = passing.iter().sum::<i32>() as f64 / passing.len() as f64;
let failing_avg: f64 = failing.iter().sum::<i32>() as f64 / failing.len() as f64;

println!("Passing avg: {:.1}", passing_avg);  // 80.4
println!("Failing avg: {:.1}", failing_avg);  // 46.0
```

---

## Integration with Metrics

### Example 9: Observability with MetricsQueryBuilder

```rust
use rusted_ca::domain::rinq::MetricsQueryBuilder;
use rusted_ca::shared::metrics::MetricsCollector;

fn main() {
    let metrics = MetricsCollector::new();
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Query with metrics recording
    let result = MetricsQueryBuilder::from(data.clone(), metrics.clone())
        .where_(|x| *x % 2 == 0)
        .sum();
    
    println!("Sum: {}", result);  // Sum: 30
    
    // Check recorded metrics
    let sum_count = metrics.get_count("rinq.operations.sum");
    println!("Sum operations: {}", sum_count);  // Sum operations: 1
    
    // Metrics also recorded for timing
    // metrics.get_query_time("rinq.sum") -> Duration
}
```

---

## Performance Notes

### Zero-Cost Operations

These operations compile to the same machine code as hand-written loops:

- `sum()` - delegates to `Iterator::sum()`
- `min()` / `max()` - delegates to `Iterator::min()` / `max()`
- `enumerate()` - delegates to `Iterator::enumerate()`
- `zip()` - delegates to `Iterator::zip()`

**Benchmark Validation**: See `benches/rinq_v0.2_benchmarks.rs` for proof.

---

### Operations with Overhead

These operations have necessary overhead:

| Operation | Overhead | Reason |
|-----------|----------|--------|
| `average()` | O(n) space | Must collect to count elements |
| `group_by()` | O(n) space + HashMap overhead | Must build HashMap |
| `distinct()` | O(k) space | HashSet for seen elements (k = unique count) |
| `reverse()` | O(n) space | Must collect to reverse |
| `window()` | O(n * window_size) clones | Overlapping windows require cloning |

**Mitigation**: Benchmarks verify overhead is minimal and necessary. Users can choose alternative approaches if cost is prohibitive for their use case.

---

## Edge Case Handling

### Empty Collections

```rust
let empty: Vec<i32> = vec![];

let sum = QueryBuilder::from(empty.clone()).sum();
// sum = 0 (additive identity)

let avg = QueryBuilder::from(empty.clone()).average();
// avg = None

let min = QueryBuilder::from(empty.clone()).min();
// min = None

let groups = QueryBuilder::from(empty.clone()).group_by(|x| *x);
// groups = HashMap::new() (empty)

let chunks = QueryBuilder::from(empty.clone()).chunk(5).collect();
// chunks = Vec::new() (empty)
```

---

### Single Element

```rust
let single = vec![42];

let sum = QueryBuilder::from(single.clone()).sum();
// sum = 42

let avg = QueryBuilder::from(single.clone()).average();
// avg = Some(42.0)

let chunks = QueryBuilder::from(single.clone()).chunk(5).collect();
// chunks = [[42]] (single chunk)
```

---

### Boundary Conditions

```rust
// Chunk size larger than collection
let small = vec![1, 2];
let chunks = QueryBuilder::from(small).chunk(10).collect();
// chunks = [[1, 2]]

// Window size larger than collection
let small = vec![1, 2];
let windows = QueryBuilder::from(small).window(5).collect();
// windows = [] (not enough elements)

// Zip with different lengths
let short = vec![1, 2];
let long = vec!['a', 'b', 'c', 'd'];
let pairs = QueryBuilder::from(short).zip(long).collect();
// pairs = [(1, 'a'), (2, 'b')] (stops at shortest)
```

---

## API Cheat Sheet

### Aggregations (Terminal)

| Method | Trait Bounds | Returns | Description |
|--------|-------------|---------|-------------|
| `.sum()` | `T: Sum` | `T` | Sum all elements |
| `.average()` | `T: ToPrimitive` | `Option<f64>` | Arithmetic mean |
| `.min()` | `T: Ord` | `Option<T>` | Minimum element |
| `.max()` | `T: Ord` | `Option<T>` | Maximum element |
| `.min_by(f)` | `K: Ord` | `Option<T>` | Min by key selector |
| `.max_by(f)` | `K: Ord` | `Option<T>` | Max by key selector |

### Grouping (Terminal)

| Method | Trait Bounds | Returns | Description |
|--------|-------------|---------|-------------|
| `.group_by(f)` | `K: Eq + Hash` | `HashMap<K, Vec<T>>` | Group by key |
| `.group_by_aggregate(fk, fa)` | `K: Eq + Hash` | `HashMap<K, R>` | Group + aggregate |

### Transformations (Return QueryBuilder)

| Method | Trait Bounds | Returns | Description |
|--------|-------------|---------|-------------|
| `.distinct()` | `T: Eq + Hash + Clone` | `QueryBuilder<T, Filtered>` | Remove duplicates |
| `.distinct_by(f)` | `K: Eq + Hash` | `QueryBuilder<T, Filtered>` | Remove by key |
| `.reverse()` | - | `QueryBuilder<T, Filtered>` | Reverse order |
| `.enumerate()` | - | `QueryBuilder<(usize, T), Filtered>` | Add indices |
| `.zip(other)` | `U: 'static` | `QueryBuilder<(T, U), Filtered>` | Pair elements |
| `.chunk(n)` | - | `QueryBuilder<Vec<T>, Filtered>` | Fixed-size chunks |
| `.window(n)` | `T: Clone` | `QueryBuilder<Vec<T>, Filtered>` | Sliding windows |

### Splitting (Terminal)

| Method | Trait Bounds | Returns | Description |
|--------|-------------|---------|-------------|
| `.partition(pred)` | - | `(Vec<T>, Vec<T>)` | Split by predicate |

---

## Next Steps

1. Explore full API documentation: See `src/domain/rinq/README.md` after implementation
2. Run examples: `cargo run --example rinq_v0.2_demo` (after implementation)
3. Run tests: `cargo test rinq_v0.2`
4. Run benchmarks: `cargo bench rinq_v0.2`

---

## Troubleshooting

### Compile Error: "method not found"

If you see errors like `method 'sum' not found for type 'QueryBuilder<SomeType, Initial>'`:

**Cause**: Missing trait bound on element type.

**Solution**: Ensure your type implements the required trait:
- `sum()` requires `T: Sum`
- `average()` requires `T: ToPrimitive`
- `distinct()` requires `T: Eq + Hash + Clone`

**Example**:
```rust
// Error: MyStruct doesn't implement Sum
let sum = QueryBuilder::from(vec![MyStruct::new()]).sum();  // ❌

// Fix: Use a field that does
let sum = QueryBuilder::from(vec![MyStruct::new()])
    .select(|s| s.numeric_field)
    .sum();  // ✅
```

---

### Compile Error: "mismatched types"

If you see state-related type errors:

**Cause**: Trying to call a method on an incompatible state.

**Solution**: Check the state transition table in data-model.md.

**Example**:
```rust
// Error: then_by() only available on Sorted state
let q = QueryBuilder::from(data)
    .where_(|x| x > 5)    // Returns Filtered
    .then_by(|x| *x);     // ❌ then_by() needs Sorted

// Fix: Use order_by() first
let q = QueryBuilder::from(data)
    .where_(|x| x > 5)
    .order_by(|x| *x)     // Filtered → Sorted
    .then_by(|x| x.id);   // ✅ Now valid
```

---

### Runtime Panic: "chunk size must be greater than 0"

**Cause**: Passing 0 to `.chunk(0)`.

**Solution**: Ensure chunk size is at least 1.

```rust
// Error
let chunks = QueryBuilder::from(data).chunk(0);  // ❌ Panics

// Fix
let chunks = QueryBuilder::from(data).chunk(1);  // ✅ Valid
```

---

### Performance Issue: window() is slow

**Cause**: `window()` clones elements for overlapping windows.

**Solution**: 
- Accept the clone overhead if window semantics are needed
- Alternative: Use standard library `slice::windows()` if you have a Vec and don't need RINQ's fluent interface
- Consider `chunk()` instead if non-overlapping partitions are acceptable

```rust
// RINQ approach (clones elements)
let windows = QueryBuilder::from(data).window(3).collect();

// std library approach (zero-copy slices, but no fluent chaining)
let data_vec = data.clone();
let windows: Vec<&[i32]> = data_vec.windows(3).collect();
```

---

**Quick Start Status**: ✅ **COMPLETE**  
**Next**: Run `/speckit.tasks` to break down into implementable tasks
