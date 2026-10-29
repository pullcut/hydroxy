use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Duration;

use crate::{Paced, PacerState, Status};

/// How a [`RateLimiter`] counts its window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Window {
    /// Starts at the first call and resets in full once it has elapsed.
    #[default]
    Fixed,
    /// Always the last `window` of time; each call frees up `window` after it ran.
    Sliding,
}

/// Why a rate-limited call was turned away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rejection {
    /// Calls allowed per window.
    pub limit: u32,
    /// When a call will next be let through.
    pub retry_at: Duration,
    /// How long from the rejected call until then.
    pub retry_after: Duration,
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rate limit of {} reached; retry after {:?}", self.limit, self.retry_after)
    }
}

impl std::error::Error for Rejection {}

/// Lets at most `limit` calls through per `window`, and turns the rest away
/// with a [`Rejection`] saying when to try again.
///
/// ```
/// use std::time::Duration;
/// use hydroxy_pacer::{RateLimiter, Window};
///
/// let ms = Duration::from_millis;
/// let mut api = RateLimiter::new(2, ms(1000)).window(Window::Sliding);
/// assert!(api.try_acquire(ms(0)).is_ok());
/// assert!(api.try_acquire(ms(400)).is_ok());
/// let no = api.try_acquire(ms(500)).unwrap_err();
/// assert_eq!(no.retry_at, ms(1000));
/// assert!(api.try_acquire(ms(1000)).is_ok());
/// ```
#[derive(Clone, Debug)]
pub struct RateLimiter {
    limit: u32,
    window: Duration,
    kind: Window,
    starts: VecDeque<Duration>,
    window_start: Option<Duration>,
    executions: u64,
    rejections: u64,
    last_run: Option<Duration>,
}

impl RateLimiter {
    /// `limit` calls per `window` (a limit of 0 acts as 1), counted in a
    /// [`Window::Fixed`] window.
    #[must_use]
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit: limit.max(1),
            window,
            kind: Window::Fixed,
            starts: VecDeque::new(),
            window_start: None,
            executions: 0,
            rejections: 0,
            last_run: None,
        }
    }

    /// How the window is counted. Default [`Window::Fixed`].
    #[must_use]
    pub const fn window(mut self, kind: Window) -> Self {
        self.kind = kind;
        self
    }

    /// Take a slot if one is free at `now`.
    ///
    /// # Errors
    ///
    /// A [`Rejection`] when the window is full.
    pub fn try_acquire(&mut self, now: Duration) -> Result<(), Rejection> {
        self.prune(now);
        if self.used(now) >= self.limit {
            self.rejections += 1;
            let retry_at = self.retry_at(now);
            return Err(Rejection { limit: self.limit, retry_at, retry_after: retry_at.saturating_sub(now) });
        }
        self.window_start.get_or_insert(now);
        self.starts.push_back(now);
        self.executions += 1;
        self.last_run = Some(now);
        Ok(())
    }

    /// Calls that can still go through at `now`.
    #[must_use]
    pub fn remaining(&self, now: Duration) -> u32 {
        self.limit.saturating_sub(self.used(now))
    }

    /// The earliest time a call is let through: `now` if a slot is free.
    #[must_use]
    pub fn retry_at(&self, now: Duration) -> Duration {
        if self.remaining(now) > 0 {
            return now;
        }
        match self.kind {
            Window::Fixed => self.window_start.map_or(now, |s| s.saturating_add(self.window)),
            Window::Sliding => self.visible(now).next().map_or(now, |&t| t.saturating_add(self.window)),
        }
    }

    fn visible(&self, now: Duration) -> impl Iterator<Item = &Duration> {
        let fixed_open = self.window_start.filter(|&s| now < s.saturating_add(self.window));
        self.starts.iter().filter(move |&&t| match self.kind {
            Window::Fixed => fixed_open.is_some_and(|s| t >= s),
            Window::Sliding => t.saturating_add(self.window) > now,
        })
    }

    #[allow(clippy::cast_possible_truncation, reason = "never more starts than the u32 limit")]
    fn used(&self, now: Duration) -> u32 {
        self.visible(now).count() as u32
    }

    fn prune(&mut self, now: Duration) {
        match self.kind {
            Window::Fixed => {
                if self.window_start.is_some_and(|s| now >= s.saturating_add(self.window)) {
                    self.window_start = None;
                    self.starts.clear();
                }
            }
            Window::Sliding => {
                while self.starts.front().is_some_and(|&t| t.saturating_add(self.window) <= now) {
                    self.starts.pop_front();
                }
            }
        }
    }
}

/// Rate limiters never hold work; ticking just lets the window move on so
/// adapters can refresh their state when a slot frees up.
impl Paced for RateLimiter {
    type Output = Infallible;

    fn tick(&mut self, now: Duration) -> Option<Infallible> {
        self.prune(now);
        None
    }

    fn deadline(&self) -> Option<Duration> {
        match self.kind {
            Window::Fixed => self.window_start.map(|s| s.saturating_add(self.window)),
            Window::Sliding => self.starts.front().map(|&t| t.saturating_add(self.window)),
        }
    }

    fn flush(&mut self, _now: Duration) -> Option<Infallible> {
        None
    }

    /// Forget past calls, freeing every slot.
    fn cancel(&mut self) {
        self.starts.clear();
        self.window_start = None;
    }

    fn state(&self, now: Duration) -> PacerState {
        PacerState {
            status: Status::Idle,
            executions: self.executions,
            rejections: self.rejections,
            last_run: self.last_run,
            next_allowed: Some(self.retry_at(now)),
            remaining: Some(self.remaining(now)),
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
    fn huge_windows_saturate_instead_of_overflowing() {
        for kind in [Window::Fixed, Window::Sliding] {
            let mut r = RateLimiter::new(1, Duration::MAX).window(kind);
            assert!(r.try_acquire(ms(1)).is_ok());
            let no = r.try_acquire(ms(2)).unwrap_err();
            assert_eq!(no.retry_at, Duration::MAX);
            assert_eq!(r.deadline(), Some(Duration::MAX));
            assert!(no.to_string().starts_with("rate limit of 1 reached"));
        }
    }

    #[test]
    fn fixed_window_resets_in_full() {
        let mut r = RateLimiter::new(2, ms(100));
        assert!(r.try_acquire(ms(10)).is_ok());
        assert!(r.try_acquire(ms(50)).is_ok());
        assert_eq!(r.remaining(ms(60)), 0);
        let no = r.try_acquire(ms(60)).unwrap_err();
        assert_eq!((no.limit, no.retry_at, no.retry_after), (2, ms(110), ms(50)));
        assert_eq!(r.deadline(), Some(ms(110)));
        assert_eq!(r.remaining(ms(110)), 2);
        assert!(r.try_acquire(ms(115)).is_ok());
        assert_eq!(r.retry_at(ms(116)), ms(116));
        let s = r.state(ms(116));
        assert_eq!((s.executions, s.rejections, s.remaining), (3, 1, Some(1)));
    }

    #[test]
    fn sliding_window_frees_one_slot_at_a_time() {
        let mut r = RateLimiter::new(2, ms(100)).window(Window::Sliding);
        r.try_acquire(ms(0)).unwrap();
        r.try_acquire(ms(60)).unwrap();
        assert_eq!(r.try_acquire(ms(80)).unwrap_err().retry_at, ms(100));
        assert_eq!(r.remaining(ms(100)), 1);
        r.try_acquire(ms(100)).unwrap();
        assert_eq!(r.try_acquire(ms(120)).unwrap_err().retry_at, ms(160));
        // Ticking moves the deadline past now.
        assert_eq!(r.tick(ms(160)), None);
        assert_eq!(r.deadline(), Some(ms(200)));
    }

    #[test]
    fn cancel_resets() {
        let mut r = RateLimiter::new(1, ms(100));
        r.try_acquire(ms(0)).unwrap();
        assert!(r.try_acquire(ms(1)).is_err());
        r.cancel();
        assert!(r.try_acquire(ms(2)).is_ok());
    }
}
