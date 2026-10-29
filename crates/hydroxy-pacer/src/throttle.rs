use std::time::Duration;

use crate::{Decision, Edge, Paced, PacerState, Status};

/// Runs at most once per `interval`.
///
/// Both edges by default: a call when the interval has passed runs at once,
/// and the latest call inside the interval runs when it ends.
///
/// ```
/// use std::time::Duration;
/// use hydroxy_pacer::{Decision, Paced, Throttler};
///
/// let ms = Duration::from_millis;
/// let mut scroll = Throttler::new(ms(100));
/// assert_eq!(scroll.call(ms(0), 0), Decision::Run(0));
/// assert_eq!(scroll.call(ms(30), 30), Decision::Scheduled);
/// assert_eq!(scroll.call(ms(60), 60), Decision::Scheduled);
/// assert_eq!(scroll.tick(ms(100)), Some(60));
/// ```
#[derive(Clone, Debug)]
pub struct Throttler<A> {
    interval: Duration,
    edge: Edge,
    pending: Option<A>,
    due_at: Option<Duration>,
    executions: u64,
    last_run: Option<Duration>,
}

impl<A> Throttler<A> {
    /// A throttler running both edges at most once per `interval`.
    #[must_use]
    pub const fn new(interval: Duration) -> Self {
        Self { interval, edge: Edge::Both, pending: None, due_at: None, executions: 0, last_run: None }
    }

    /// Which edge runs. Default [`Edge::Both`].
    #[must_use]
    pub const fn edge(mut self, edge: Edge) -> Self {
        self.edge = edge;
        self
    }

    /// Offer a call. Tick first if a deadline may have passed.
    pub fn call(&mut self, now: Duration, args: A) -> Decision<A> {
        let Some(last) = self.last_run else {
            if self.edge.leading() {
                return self.run(now, args);
            }
            self.pending = Some(args);
            self.due_at.get_or_insert(now.saturating_add(self.interval));
            return Decision::Scheduled;
        };
        let next = last.saturating_add(self.interval);
        if now >= next {
            self.pending = None;
            self.due_at = None;
            return self.run(now, args);
        }
        if !self.edge.trailing() {
            return Decision::Suppressed;
        }
        self.pending = Some(args);
        self.due_at = Some(next);
        Decision::Scheduled
    }

    fn run(&mut self, now: Duration, args: A) -> Decision<A> {
        self.executions += 1;
        self.last_run = Some(now);
        Decision::Run(args)
    }
}

impl<A> Paced for Throttler<A> {
    type Output = A;

    fn tick(&mut self, now: Duration) -> Option<A> {
        if self.due_at.is_none_or(|d| d > now) {
            return None;
        }
        self.flush(now)
    }

    fn deadline(&self) -> Option<Duration> {
        self.due_at
    }

    fn flush(&mut self, now: Duration) -> Option<A> {
        self.due_at = None;
        let args = self.pending.take()?;
        match self.run(now, args) {
            Decision::Run(args) => Some(args),
            _ => None,
        }
    }

    fn cancel(&mut self) {
        self.pending = None;
        self.due_at = None;
    }

    fn state(&self, now: Duration) -> PacerState {
        let pending = usize::from(self.pending.is_some());
        PacerState {
            status: if pending > 0 { Status::Pending } else { Status::Idle },
            pending,
            executions: self.executions,
            last_run: self.last_run,
            next_run: self.due_at,
            next_allowed: Some(self.last_run.map_or(now, |l| l.saturating_add(self.interval).max(now))),
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
    fn huge_intervals_saturate() {
        let mut t: Throttler<i32> = Throttler::new(Duration::MAX);
        t.call(ms(1), 1);
        t.call(ms(2), 2);
        assert_eq!(t.state(ms(3)).next_allowed, Some(Duration::MAX));
    }

    #[test]
    fn leading_and_trailing() {
        let mut t = Throttler::new(ms(100));
        assert_eq!(t.call(ms(0), 1), Decision::Run(1));
        assert_eq!(t.call(ms(10), 2), Decision::Scheduled);
        assert_eq!(t.call(ms(20), 3), Decision::Scheduled);
        assert_eq!(t.state(ms(20)).next_allowed, Some(ms(100)));
        assert_eq!(t.tick(ms(99)), None);
        assert_eq!(t.tick(ms(100)), Some(3));
        // The trailing run starts a new interval.
        assert_eq!(t.call(ms(150), 4), Decision::Scheduled);
        assert_eq!(t.deadline(), Some(ms(200)));
        assert_eq!(t.call(ms(250), 5), Decision::Run(5), "a stale deadline is replaced by a direct run");
        assert_eq!(t.tick(ms(400)), None);
    }

    #[test]
    fn leading_only_drops_inside_the_interval() {
        let mut t = Throttler::new(ms(100)).edge(Edge::Leading);
        assert_eq!(t.call(ms(0), 1), Decision::Run(1));
        assert_eq!(t.call(ms(50), 2), Decision::Suppressed);
        assert_eq!(t.deadline(), None);
        assert_eq!(t.call(ms(100), 3), Decision::Run(3));
    }

    #[test]
    fn trailing_only_waits_for_the_first_interval() {
        let mut t = Throttler::new(ms(100)).edge(Edge::Trailing);
        assert_eq!(t.call(ms(0), 1), Decision::Scheduled);
        assert_eq!(t.call(ms(40), 2), Decision::Scheduled);
        assert_eq!(t.deadline(), Some(ms(100)));
        assert_eq!(t.tick(ms(100)), Some(2));
        assert_eq!(t.state(ms(100)).executions, 1);
    }

    #[test]
    fn flush_and_cancel() {
        let mut t = Throttler::new(ms(100));
        t.call(ms(0), 1);
        t.call(ms(10), 2);
        assert_eq!(t.flush(ms(20)), Some(2));
        t.call(ms(30), 3);
        t.cancel();
        assert_eq!(t.tick(ms(500)), None);
        assert_eq!(t.state(ms(500)).pending, 0);
    }
}
