use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Duration;

use crate::{PacerState, Status};

/// Identifies a job in an [`AsyncQueue`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(pub u64);

struct Waiting<T> {
    priority: i32,
    id: JobId,
    item: T,
}

// Highest priority first, then oldest (lowest id) first.
impl<T> Ord for Waiting<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).then_with(|| other.id.cmp(&self.id))
    }
}
impl<T> PartialOrd for Waiting<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> PartialEq for Waiting<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Waiting<T> {}

/// Bookkeeping for async jobs run with a concurrency limit, by priority
/// (ties first in, first out), with pause and cancellation.
///
/// The queue does not run anything: [`next`](Self::next) hands out a job
/// when a slot is free, and you report back with [`finish`](Self::finish).
/// Cancelling a running job keeps its slot until it finishes, and `finish`
/// then tells you to ignore its result.
///
/// ```
/// use std::time::Duration;
/// use hydroxy_pacer::AsyncQueue;
///
/// let t = Duration::ZERO;
/// let mut uploads = AsyncQueue::new(1);
/// let a = uploads.push("a.png", 0);
/// let b = uploads.push("b.png", 5);
/// assert_eq!(uploads.next(t), Some((b, "b.png")));
/// assert_eq!(uploads.next(t), None, "one at a time");
/// assert!(uploads.finish(b, t, true));
/// assert_eq!(uploads.next(t), Some((a, "a.png")));
/// ```
pub struct AsyncQueue<T> {
    concurrency: usize,
    waiting: BinaryHeap<Waiting<T>>,
    active: Vec<(JobId, bool)>,
    paused: bool,
    next_id: u64,
    executions: u64,
    rejections: u64,
    last_run: Option<Duration>,
}

impl<T> std::fmt::Debug for AsyncQueue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncQueue").finish_non_exhaustive()
    }
}

impl<T> AsyncQueue<T> {
    /// A queue running up to `concurrency` jobs at once (at least 1).
    #[must_use]
    pub fn new(concurrency: usize) -> Self {
        Self {
            concurrency: concurrency.max(1),
            waiting: BinaryHeap::new(),
            active: Vec::new(),
            paused: false,
            next_id: 0,
            executions: 0,
            rejections: 0,
            last_run: None,
        }
    }

    /// Change how many jobs may run at once. Running jobs are not stopped.
    pub fn set_concurrency(&mut self, concurrency: usize) {
        self.concurrency = concurrency.max(1);
    }

    /// Queue a job. Higher `priority` runs sooner.
    pub fn push(&mut self, item: T, priority: i32) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;
        self.waiting.push(Waiting { priority, id, item });
        id
    }

    /// Start the next job if a slot is free and the queue is not paused.
    pub fn next(&mut self, now: Duration) -> Option<(JobId, T)> {
        if self.paused || self.active.len() >= self.concurrency {
            return None;
        }
        let job = self.waiting.pop()?;
        self.active.push((job.id, false));
        self.last_run = Some(now);
        Some((job.id, job.item))
    }

    /// Report a started job as done. Returns `false` if it was cancelled
    /// while running, so its result should be ignored.
    pub fn finish(&mut self, id: JobId, _now: Duration, ok: bool) -> bool {
        let Some(at) = self.active.iter().position(|&(a, _)| a == id) else { return false };
        let (_, cancelled) = self.active.swap_remove(at);
        if cancelled {
            return false;
        }
        if ok {
            self.executions += 1;
        } else {
            self.rejections += 1;
        }
        true
    }

    /// Cancel a job. A waiting job is dropped; a running one keeps its slot
    /// until it finishes. Returns `false` for unknown or finished jobs.
    pub fn cancel(&mut self, id: JobId) -> bool {
        if let Some(job) = self.active.iter_mut().find(|(a, _)| *a == id) {
            job.1 = true;
            return true;
        }
        let before = self.waiting.len();
        self.waiting.retain(|w| w.id != id);
        self.waiting.len() != before
    }

    /// Drop every waiting job.
    pub fn clear(&mut self) {
        self.waiting.clear();
    }

    /// Stop handing out jobs. Running ones carry on.
    pub const fn pause(&mut self) {
        self.paused = true;
    }

    /// Hand out jobs again.
    pub const fn resume(&mut self) {
        self.paused = false;
    }

    /// Whether the job is waiting or running (and not cancelled).
    #[must_use]
    pub fn contains(&self, id: JobId) -> bool {
        self.active.iter().any(|&(a, c)| a == id && !c) || self.waiting.iter().any(|w| w.id == id)
    }

    /// A snapshot for display.
    #[must_use]
    pub fn state(&self) -> PacerState {
        let status = if self.paused {
            Status::Paused
        } else if !self.active.is_empty() {
            Status::Running
        } else if self.waiting.is_empty() {
            Status::Idle
        } else {
            Status::Pending
        };
        PacerState {
            status,
            pending: self.waiting.len(),
            active: self.active.len(),
            executions: self.executions,
            rejections: self.rejections,
            last_run: self.last_run,
            ..PacerState::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Duration = Duration::ZERO;

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn priority_then_fifo_with_concurrency() {
        let mut q = AsyncQueue::new(2);
        let a = q.push('a', 0);
        let b = q.push('b', 1);
        let c = q.push('c', 0);
        assert_eq!(q.next(T), Some((b, 'b')));
        assert_eq!(q.next(T), Some((a, 'a')));
        assert_eq!(q.next(T), None);
        assert_eq!(q.state().status, Status::Running);
        assert!(q.finish(a, T, false));
        assert_eq!(q.next(T), Some((c, 'c')));
        q.finish(b, T, true);
        q.finish(c, T, true);
        let s = q.state();
        assert_eq!((s.status, s.executions, s.rejections), (Status::Idle, 2, 1));
    }

    #[test]
    fn cancel_waiting_and_running() {
        let mut q = AsyncQueue::new(1);
        let a = q.push(1, 0);
        let b = q.push(2, 0);
        assert_eq!(q.next(T), Some((a, 1)));
        assert!(q.cancel(b));
        assert!(q.cancel(a));
        assert!(!q.contains(a));
        assert_eq!(q.next(T), None, "a cancelled running job holds its slot");
        assert!(!q.finish(a, T, true), "its result is ignored");
        assert_eq!(q.state().executions, 0);
        assert!(!q.cancel(a));
    }

    #[test]
    fn pause_and_resume() {
        let mut q = AsyncQueue::new(1);
        q.push((), 0);
        q.pause();
        assert_eq!(q.state().status, Status::Paused);
        assert!(q.next(T).is_none());
        q.resume();
        assert!(q.next(T).is_some());
        q.push((), 0);
        q.clear();
        assert_eq!(q.state().pending, 0);
    }
}
