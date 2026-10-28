use std::time::Duration;

use crate::{PacerState, Paced, Status};

/// Collects items and hands them over together, once `max_size` are waiting
/// or `wait` after the oldest arrived, whichever comes first.
///
/// With neither set, items only leave through [`flush`](Paced::flush).
///
/// ```
/// use std::time::Duration;
/// use hydroxy_pacer::{Batcher, Paced};
///
/// let ms = Duration::from_millis;
/// let mut log = Batcher::new().max_size(3).wait(ms(500));
/// assert_eq!(log.add(ms(0), "a"), None);
/// assert_eq!(log.add(ms(10), "b"), None);
/// assert_eq!(log.add(ms(20), "c"), Some(vec!["a", "b", "c"]));
/// log.add(ms(30), "d");
/// assert_eq!(log.tick(ms(530)), Some(vec!["d"]));
/// ```
#[derive(Clone, Debug)]
pub struct Batcher<T> {
    max_size: Option<usize>,
    wait: Option<Duration>,
    items: Vec<T>,
    due_at: Option<Duration>,
    executions: u64,
    last_run: Option<Duration>,
}

impl<T> Default for Batcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Batcher<T> {
    /// An empty batcher that only flushes on demand.
    #[must_use]
    pub const fn new() -> Self {
        Self { max_size: None, wait: None, items: Vec::new(), due_at: None, executions: 0, last_run: None }
    }

    /// Hand the batch over as soon as this many items wait (at least 1).
    #[must_use]
    pub fn max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size.max(1));
        self
    }

    /// Hand the batch over this long after its oldest item arrived.
    #[must_use]
    pub const fn wait(mut self, wait: Duration) -> Self {
        self.wait = Some(wait);
        self
    }

    /// Items waiting, oldest first.
    #[must_use]
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Add an item. Returns the full batch when this one fills it.
    /// Tick first if a deadline may have passed.
    pub fn add(&mut self, now: Duration, item: T) -> Option<Vec<T>> {
        if self.items.is_empty() {
            self.due_at = self.wait.map(|w| now + w);
        }
        self.items.push(item);
        if self.max_size.is_some_and(|m| self.items.len() >= m) {
            return self.flush(now);
        }
        None
    }
}

impl<T> Paced for Batcher<T> {
    type Output = Vec<T>;

    fn tick(&mut self, now: Duration) -> Option<Vec<T>> {
        if self.due_at.is_none_or(|d| d > now) {
            return None;
        }
        self.flush(now)
    }

    fn deadline(&self) -> Option<Duration> {
        self.due_at
    }

    fn flush(&mut self, now: Duration) -> Option<Vec<T>> {
        self.due_at = None;
        if self.items.is_empty() {
            return None;
        }
        self.executions += 1;
        self.last_run = Some(now);
        Some(std::mem::take(&mut self.items))
    }

    fn cancel(&mut self) {
        self.items.clear();
        self.due_at = None;
    }

    fn state(&self, _now: Duration) -> PacerState {
        PacerState {
            status: if self.items.is_empty() { Status::Idle } else { Status::Pending },
            pending: self.items.len(),
            executions: self.executions,
            last_run: self.last_run,
            next_run: self.due_at,
            ..PacerState::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn size_and_time_triggers() {
        let mut b = Batcher::new().max_size(2).wait(ms(100));
        assert_eq!(b.add(ms(0), 1), None);
        assert_eq!(b.deadline(), Some(ms(100)));
        assert_eq!(b.add(ms(5), 2), Some(vec![1, 2]));
        assert_eq!(b.deadline(), None);
        assert_eq!(b.add(ms(50), 3), None);
        assert_eq!(b.state(ms(50)).pending, 1);
        assert_eq!(b.tick(ms(149)), None);
        assert_eq!(b.tick(ms(150)), Some(vec![3]));
        assert_eq!(b.state(ms(150)).executions, 2);
    }

    #[test]
    fn manual_only() {
        let mut b = Batcher::new();
        b.add(ms(0), 'a');
        b.add(ms(1), 'b');
        assert_eq!(b.deadline(), None);
        assert_eq!(b.items(), ['a', 'b']);
        assert_eq!(b.flush(ms(2)), Some(vec!['a', 'b']));
        assert_eq!(b.flush(ms(3)), None);
        b.add(ms(4), 'c');
        b.cancel();
        assert_eq!(b.flush(ms(5)), None);
    }
}
