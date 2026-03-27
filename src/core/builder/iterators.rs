// src/core/builder/iterators.rs
// Custom iterator adapters: ChunkIterator, WindowIterator, MovingAverageIterator

use std::collections::VecDeque;

/// Iterator adapter for chunking elements into fixed-size vectors
pub(crate) struct ChunkIterator<I> {
    pub(crate) inner: I,
    pub(crate) chunk_size: usize,
}

impl<I, T> Iterator for ChunkIterator<I>
where
    I: Iterator<Item = T>,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.chunk_size);
        for _ in 0..self.chunk_size {
            match self.inner.next() {
                Some(item) => chunk.push(item),
                None => break,
            }
        }
        if chunk.is_empty() { None } else { Some(chunk) }
    }
}

/// Iterator adapter for computing a sliding-window average.
///
/// Produces `None` for the first `window - 1` elements (incomplete window),
/// then `Some(avg)` for each subsequent position.
pub(crate) struct MovingAverageIterator {
    values: std::vec::IntoIter<f64>,
    window: usize,
    buffer: VecDeque<f64>,
    sum: f64,
}

impl MovingAverageIterator {
    pub(crate) fn new(values: Vec<f64>, window: usize) -> Self {
        debug_assert!(window >= 1);
        Self {
            values: values.into_iter(),
            window,
            buffer: VecDeque::with_capacity(window),
            sum: 0.0,
        }
    }
}

impl Iterator for MovingAverageIterator {
    type Item = Option<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.values.next()?;
        self.buffer.push_back(val);
        self.sum += val;
        if self.buffer.len() < self.window {
            Some(None)
        } else {
            let avg = self.sum / self.window as f64;
            self.sum -= self.buffer.pop_front().unwrap_or(0.0);
            Some(Some(avg))
        }
    }
}

/// Iterator adapter that groups consecutive elements sharing the same key.
///
/// Produces `Vec<T>` chunks where all elements in a chunk have the same key
/// value.  A new chunk starts whenever the key changes.
pub(crate) struct ChunkByIterator<T, K, F>
where
    F: FnMut(&T) -> K,
    K: PartialEq,
{
    pub(crate) inner: Box<dyn Iterator<Item = T>>,
    pub(crate) key_fn: F,
    pub(crate) buffered: Option<T>,
    pub(crate) done: bool,
}

impl<T: 'static, K: PartialEq, F: FnMut(&T) -> K> Iterator for ChunkByIterator<T, K, F> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Get the first element of the new chunk (either buffered or from iterator)
        let first = match self.buffered.take() {
            Some(item) => item,
            None => self.inner.next()?,
        };

        let mut chunk = vec![];
        let chunk_key = (self.key_fn)(&first);
        chunk.push(first);

        loop {
            match self.inner.next() {
                None => {
                    self.done = true;
                    break;
                }
                Some(next_item) => {
                    if (self.key_fn)(&next_item) == chunk_key {
                        chunk.push(next_item);
                    } else {
                        // Key changed: buffer this item for the next chunk
                        self.buffered = Some(next_item);
                        break;
                    }
                }
            }
        }

        Some(chunk)
    }
}

/// Iterator adapter for `unfold`: generates values from a seed using a closure
/// that returns `Option<(yield_value, new_seed)>`.
pub(crate) struct UnfoldIter<S, T, F>
where
    F: FnMut(S) -> Option<(T, S)>,
{
    pub(crate) state: Option<S>,
    pub(crate) f: F,
}

impl<S, T, F> Iterator for UnfoldIter<S, T, F>
where
    F: FnMut(S) -> Option<(T, S)>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let state = self.state.take()?;
        match (self.f)(state) {
            Some((value, next_state)) => {
                self.state = Some(next_state);
                Some(value)
            }
            None => None,
        }
    }
}

/// Iterator adapter for `unfold_bounded`: same as `UnfoldIter` but caps at
/// `max` elements and emits a debug warning when the limit is reached.
pub(crate) struct UnfoldBoundedIter<S, T, F>
where
    F: FnMut(S) -> Option<(T, S)>,
{
    pub(crate) inner: UnfoldIter<S, T, F>,
    pub(crate) remaining: usize,
}

impl<S, T, F> Iterator for UnfoldBoundedIter<S, T, F>
where
    F: FnMut(S) -> Option<(T, S)>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.remaining == 0 {
            #[cfg(debug_assertions)]
            eprintln!("[rinq] unfold_bounded: limit reached");
            return None;
        }
        self.remaining -= 1;
        self.inner.next()
    }
}

/// Iterator adapter for sliding windows over elements
pub(crate) struct WindowIterator<T> {
    pub(crate) buffer: VecDeque<T>,
    pub(crate) inner: Box<dyn Iterator<Item = T>>,
    pub(crate) window_size: usize,
    pub(crate) finished: bool,
}

impl<T: Clone> Iterator for WindowIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Fill initial buffer
        while self.buffer.len() < self.window_size {
            match self.inner.next() {
                Some(item) => self.buffer.push_back(item),
                None => {
                    self.finished = true;
                    return None;
                }
            }
        }

        // Create window from current buffer
        let window: Vec<T> = self.buffer.iter().cloned().collect();

        // Slide the window
        self.buffer.pop_front();
        match self.inner.next() {
            Some(item) => self.buffer.push_back(item),
            None => self.finished = true,
        }

        Some(window)
    }
}
