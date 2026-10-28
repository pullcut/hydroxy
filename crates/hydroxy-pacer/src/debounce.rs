use std::time::Duration;

use crate::{Decision, Edge, PacerState, Paced, Status};

/// Runs once a burst of calls goes quiet for `wait`.
///
/// Trailing by default: the last call's arguments run after the quiet time.
/// With [`Edge::Leading`] the first call runs at once and the rest of the
/// burst is dropped; [`Edge::Both`] does both. [`max_wait`](Self::max_wait)
/// caps how long a kept call can be put off by a steady stream of calls.
///
/// ```
/// use std::time::Duration;
/// use hydroxy_pacer::{Debouncer, Decision, Edge, Paced};
///
/// let ms = Duration::from_millis;
/// let mut save = Debouncer::new(ms(100)).edge(Edge::Both);
/// assert_eq!(save.call(ms(0), 1), Decision::Run(1));
/// assert_eq!(save.call(ms(50), 2), Decision::Scheduled);
/// assert_eq!(save.tick(ms(150)), Some(2));
/// ```
#[derive(Clone, Debug)]
pub struct Debouncer<A> {
    wait: Duration,
    max_wait: Option<Duration>,
    edge: Edge,
    pending: Option<A>,
    /// Some while a burst is open.
    quiet_at: Option<Duration>,
    max_at: Option<Duration>,
    executions: u64,
    last_run: Option<Duration>,
}

impl<A> Debouncer<A> {
    /// A trailing debouncer with this quiet time.
    #[must_use]
    pub const fn new(wait: Duration) -> Self {
        Self { wait, max_wait: None, edge: Edge::Trailing, pending: None, quiet_at: None, max_at: None, executions: 0, last_run: None }
    }

    /// Which edge of a burst runs. Default [`Edge::Trailing`].
    #[must_use]
    pub const fn edge(mut self, edge: Edge) -> Self {
        self.edge = edge;
        self
    }

    /// Run a kept call no later than this long after it was first kept, even
    /// if calls keep coming. Values below `wait` act as `wait`.
    #[must_use]
    pub fn max_wait(mut self, max_wait: Duration) -> Self {
        self.max_wait = Some(max_wait.max(self.wait));
        self
    }

    /// The quiet time.
    #[must_use]
    pub const fn wait(&self) -> Duration {
        self.wait
    }

    /// Change the quiet time. Applies from the next call.
    pub const fn set_wait(&mut self, wait: Duration) {
        self.wait = wait;
    }

    /// Offer a call. Tick first if a deadline may have passed.
    pub fn call(&mut self, now: Duration, args: A) -> Decision<A> {
        let opening = self.quiet_at.is_none();
        self.quiet_at = Some(now + self.wait);
        if opening && self.edge.leading() {
            return self.run(now, args);
        }
        if !self.edge.trailing() {
            return Decision::Suppressed;
        }
        if self.pending.is_none() && self.max_at.is_none() {
            self.max_at = self.max_wait.map(|m| now + m);
        }
        self.pending = Some(args);
        Decision::Scheduled
    }

    fn run(&mut self, now: Duration, args: A) -> Decision<A> {
        self.executions += 1;
        self.last_run = Some(now);
        Decision::Run(args)
    }

    fn finish_burst(&mut self) {
        self.quiet_at = None;
        self.max_at = None;
    }
}

impl<A> Paced for Debouncer<A> {
    type Output = A;

    fn tick(&mut self, now: Duration) -> Option<A> {
        if self.deadline().is_none_or(|d| d > now) {
            return None;
        }
        self.flush(now)
    }

    fn deadline(&self) -> Option<Duration> {
        match (self.quiet_at, self.max_at) {
            (Some(q), Some(m)) => Some(q.min(m)),
            (q, m) => q.or(m),
        }
    }

    fn flush(&mut self, now: Duration) -> Option<A> {
        self.finish_burst();
        let args = self.pending.take()?;
        match self.run(now, args) {
            Decision::Run(args) => Some(args),
            _ => None,
        }
    }

    fn cancel(&mut self) {
        self.pending = None;
        self.finish_burst();
    }

    fn state(&self, _now: Duration) -> PacerState {
        let pending = usize::from(self.pending.is_some());
        PacerState {
            status: if pending > 0 { Status::Pending } else { Status::Idle },
            pending,
            executions: self.executions,
            last_run: self.last_run,
            next_run: if pending > 0 { self.deadline() } else { None },
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
    fn trailing_runs_last_call_after_quiet() {
        let mut d = Debouncer::new(ms(100));
        assert_eq!(d.call(ms(0), 1), Decision::Scheduled);
        assert_eq!(d.call(ms(60), 2), Decision::Scheduled);
        assert_eq!(d.tick(ms(100)), None);
        assert_eq!(d.state(ms(100)).next_run, Some(ms(160)));
        assert_eq!(d.tick(ms(160)), Some(2));
        assert_eq!(d.deadline(), None);
        let s = d.state(ms(160));
        assert_eq!((s.status, s.executions, s.last_run), (Status::Idle, 1, Some(ms(160))));
    }

    #[test]
    fn leading_runs_first_and_drops_the_rest() {
        let mut d = Debouncer::new(ms(100)).edge(Edge::Leading);
        assert_eq!(d.call(ms(0), 1), Decision::Run(1));
        assert_eq!(d.call(ms(50), 2), Decision::Suppressed);
        // The quiet window was extended by the dropped call.
        assert_eq!(d.tick(ms(100)), None);
        assert_eq!(d.tick(ms(150)), None);
        assert_eq!(d.deadline(), None);
        assert_eq!(d.call(ms(200), 3), Decision::Run(3));
    }

    #[test]
    fn both_edges() {
        let mut d = Debouncer::new(ms(100)).edge(Edge::Both);
        assert_eq!(d.call(ms(0), 1), Decision::Run(1));
        assert_eq!(d.tick(ms(100)), None, "a lone leading call has no trailing run");
        assert_eq!(d.call(ms(200), 2), Decision::Run(2));
        assert_eq!(d.call(ms(250), 3), Decision::Scheduled);
        assert_eq!(d.tick(ms(350)), Some(3));
    }

    #[test]
    fn max_wait_caps_a_steady_stream() {
        let mut d = Debouncer::new(ms(100)).max_wait(ms(250));
        for t in [0, 80, 160, 240] {
            assert_eq!(d.tick(ms(t)), None);
            d.call(ms(t), t);
        }
        assert_eq!(d.deadline(), Some(ms(250)));
        assert_eq!(d.tick(ms(250)), Some(240));
        // A new burst starts its own max wait.
        d.call(ms(300), 300);
        assert_eq!(d.deadline(), Some(ms(400)));
    }

    #[test]
    fn flush_and_cancel() {
        let mut d = Debouncer::new(ms(100));
        d.call(ms(0), 1);
        assert_eq!(d.flush(ms(10)), Some(1));
        assert_eq!(d.flush(ms(10)), None);
        d.call(ms(20), 2);
        d.cancel();
        assert_eq!(d.tick(ms(500)), None);
        assert_eq!(d.state(ms(500)).executions, 1);
    }
}
