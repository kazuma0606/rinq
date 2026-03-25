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
