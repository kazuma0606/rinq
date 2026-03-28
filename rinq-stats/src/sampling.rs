// rinq-stats/src/sampling.rs
// SamplingExt trait — reservoir and stratified sampling on QueryBuilder.

use rand::Rng;
use rinq::QueryBuilder;
use std::hash::Hash;

/// Sampling operations for `QueryBuilder`.
///
/// Import this trait to add sampling methods:
///
/// ```
/// use rinq::QueryBuilder;
/// use rinq_stats::SamplingExt;
/// use rand::SeedableRng;
///
/// let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
/// // (requires rand feature "small_rng")
/// let sample: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
///     .sample_n(&mut rng, 4)
///     .collect();
/// assert_eq!(sample.len(), 4);
/// ```
pub trait SamplingExt<T> {
    /// Sample approximately `fraction * N` elements using Vitter's Algorithm R
    /// (reservoir sampling). `fraction` is clamped to `[0.0, 1.0]`.
    ///
    /// The result count may vary slightly because the reservoir size is computed
    /// as `(fraction * N).ceil()`, where `N` is the total element count.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::SamplingExt;
    /// use rand::SeedableRng;
    ///
    /// let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    /// let data: Vec<i32> = (1..=100).collect();
    /// let sample: Vec<i32> = QueryBuilder::from(data)
    ///     .sample_fraction(&mut rng, 0.1)
    ///     .collect();
    /// assert!(sample.len() <= 10);
    /// ```
    fn sample_fraction<R: Rng>(self, rng: &mut R, fraction: f64)
    -> QueryBuilder<T, rinq::Filtered>;

    /// Sample exactly `k` elements (or all elements if `k ≥ N`) using
    /// Vitter's Algorithm R (reservoir sampling).
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::SamplingExt;
    /// use rand::SeedableRng;
    ///
    /// let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    /// let sample: Vec<i32> = QueryBuilder::from(vec![1, 2, 3, 4, 5])
    ///     .sample_n(&mut rng, 3)
    ///     .collect();
    /// assert_eq!(sample.len(), 3);
    /// ```
    fn sample_n<R: Rng>(self, rng: &mut R, k: usize) -> QueryBuilder<T, rinq::Filtered>;

    /// Stratified sampling: group by a key selector and take up to `k` elements
    /// per group using reservoir sampling.
    ///
    /// The output order is undefined (groups are merged after independent sampling).
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::SamplingExt;
    /// use rand::SeedableRng;
    ///
    /// let mut rng = rand::rngs::SmallRng::seed_from_u64(2);
    /// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let sample: Vec<i32> = QueryBuilder::from(data)
    ///     .stratified_sample(&mut rng, |x| x % 2, 2)  // max 2 per parity group
    ///     .collect();
    /// assert!(sample.len() <= 4); // 2 groups × 2 max
    /// ```
    fn stratified_sample<R, K, F>(
        self,
        rng: &mut R,
        key: F,
        k: usize,
    ) -> QueryBuilder<T, rinq::Filtered>
    where
        R: Rng,
        K: Eq + Hash,
        F: Fn(&T) -> K;

    /// Bootstrap sample (sampling with replacement): draw `n` elements uniformly
    /// at random from the population, returning a new `Vec` of length `n`.
    ///
    /// Returns an empty `Vec` if the source is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::SamplingExt;
    /// use rand::SeedableRng;
    ///
    /// let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
    /// let sample: Vec<i32> = QueryBuilder::from(vec![1, 2, 3])
    ///     .bootstrap_sample(&mut rng, 10);
    /// assert_eq!(sample.len(), 10);
    /// assert!(sample.iter().all(|x| [1, 2, 3].contains(x)));
    /// ```
    fn bootstrap_sample<R: Rng>(self, rng: &mut R, n: usize) -> Vec<T>
    where
        T: Clone;
}

// ── blanket impl for QueryBuilder<T, State> ──────────────────────────────────

impl<T: 'static, State: 'static> SamplingExt<T> for QueryBuilder<T, State> {
    fn sample_fraction<R: Rng>(
        self,
        rng: &mut R,
        fraction: f64,
    ) -> QueryBuilder<T, rinq::Filtered> {
        let items: Vec<T> = self.collect();
        let fraction = fraction.clamp(0.0, 1.0);
        let k = (fraction * items.len() as f64).ceil() as usize;
        let sampled = reservoir_sample(items, k, rng);
        QueryBuilder::from(sampled).where_(|_| true)
    }

    fn sample_n<R: Rng>(self, rng: &mut R, k: usize) -> QueryBuilder<T, rinq::Filtered> {
        let items: Vec<T> = self.collect();
        let sampled = reservoir_sample(items, k, rng);
        QueryBuilder::from(sampled).where_(|_| true)
    }

    fn stratified_sample<R, K, F>(
        self,
        rng: &mut R,
        key: F,
        k: usize,
    ) -> QueryBuilder<T, rinq::Filtered>
    where
        R: Rng,
        K: Eq + Hash,
        F: Fn(&T) -> K,
    {
        use std::collections::HashMap;
        let items: Vec<T> = self.collect();
        let mut groups: HashMap<K, Vec<T>> = HashMap::new();
        for item in items {
            let group_key = key(&item);
            groups.entry(group_key).or_default().push(item);
        }
        let mut result: Vec<T> = Vec::new();
        for (_, group) in groups {
            result.extend(reservoir_sample(group, k, rng));
        }
        QueryBuilder::from(result).where_(|_| true)
    }

    fn bootstrap_sample<R: Rng>(self, rng: &mut R, n: usize) -> Vec<T>
    where
        T: Clone,
    {
        let items: Vec<T> = self.collect();
        if items.is_empty() {
            return vec![];
        }
        (0..n)
            .map(|_| items[rng.gen_range(0..items.len())].clone())
            .collect()
    }
}

// ── Vitter's Algorithm R ──────────────────────────────────────────────────────

/// Reservoir sampling (Vitter Algorithm R): select `k` items from `source`
/// with uniform probability, in O(n) time and O(k) space.
fn reservoir_sample<T, R: Rng>(source: Vec<T>, k: usize, rng: &mut R) -> Vec<T> {
    if k == 0 || source.is_empty() {
        return vec![];
    }
    let mut reservoir: Vec<T> = Vec::with_capacity(k);
    let mut iter = source.into_iter();

    // Fill reservoir with first k items
    for _ in 0..k {
        match iter.next() {
            Some(item) => reservoir.push(item),
            None => return reservoir, // source smaller than k
        }
    }

    // For each remaining item, replace a random slot with decreasing probability
    for (i, item) in iter.enumerate() {
        let j = rng.gen_range(0..=(i + k));
        if j < k {
            reservoir[j] = item;
        }
    }
    reservoir
}
