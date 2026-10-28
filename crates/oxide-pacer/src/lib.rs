//! Pacing controllers: debounce, throttle, rate limit, batch and an async
//! queue, each with observable [`PacerState`].
//!
//! The core is a set of plain state machines. Every method takes `now`, a
//! [`Duration`] since any fixed origin, so the core never reads a clock or owns
//! a timer: it reports the next [`Paced::deadline`] and you call
//! [`Paced::tick`] when it passes. That keeps it runtime-agnostic and makes
//! tests deterministic. Controllers never run your code either; they hand back
//! the arguments (or batch, or job) to run, like `oxide-statechart` commands.
//!
//! ```
//! use std::time::Duration;
//! use oxide_pacer::{Debouncer, Decision, Paced};
//!
//! let ms = Duration::from_millis;
//! let mut search = Debouncer::new(ms(300));
//! assert_eq!(search.call(ms(0), "o"), Decision::Scheduled);
//! assert_eq!(search.call(ms(100), "ox"), Decision::Scheduled);
//! assert_eq!(search.deadline(), Some(ms(400)));
//! assert_eq!(search.tick(ms(400)), Some("ox"));
//! ```
//!
//! The `leptos` and `dioxus` features add hooks that drive these with real
//! timers and expose state as signals.

use std::time::Duration;

mod batch;
mod debounce;
mod queue;
mod rate_limit;
mod throttle;

#[cfg(feature = "dioxus")]
pub mod dioxus;
#[cfg(feature = "leptos")]
pub mod leptos;

pub use batch::Batcher;
pub use debounce::Debouncer;
pub use queue::{AsyncQueue, JobId};
pub use rate_limit::{RateLimiter, Rejection, Window};
pub use throttle::Throttler;

/// What a controller is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    /// Nothing waiting or running.
    #[default]
    Idle,
    /// Work is waiting for its time.
    Pending,
    /// Queue jobs are in flight.
    Running,
    /// The queue is paused.
    Paused,
}

/// A snapshot of a controller, cheap to copy into a signal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PacerState {
    /// What the controller is doing.
    pub status: Status,
    /// Calls, items or jobs waiting.
    pub pending: usize,
    /// Queue jobs in flight.
    pub active: usize,
    /// Times the work ran (batches flushed, queue jobs that succeeded).
    pub executions: u64,
    /// Calls turned away by a rate limiter, or queue jobs that failed.
    pub rejections: u64,
    /// When the work last ran.
    pub last_run: Option<Duration>,
    /// When the pending work is due to run.
    pub next_run: Option<Duration>,
    /// Throttle and rate limit: the earliest time a call can run straight away.
    pub next_allowed: Option<Duration>,
    /// Rate limit: calls left in the current window.
    pub remaining: Option<u32>,
}

/// Which edge of a burst runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The first call of a burst.
    Leading,
    /// The last call of a burst, once it is over.
    Trailing,
    /// Both the first and the last.
    Both,
}

impl Edge {
    const fn leading(self) -> bool {
        matches!(self, Self::Leading | Self::Both)
    }
    const fn trailing(self) -> bool {
        matches!(self, Self::Trailing | Self::Both)
    }
}

/// What a debounced or throttled call did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision<T> {
    /// Run now with these arguments.
    Run(T),
    /// Kept; it runs at the next deadline unless a newer call replaces it.
    Scheduled,
    /// Dropped.
    Suppressed,
}

/// The time-driven side shared by every controller, which adapters drive.
pub trait Paced {
    /// What is due when the controller fires: arguments or a batch.
    type Output;
    /// Fire anything due at `now`.
    fn tick(&mut self, now: Duration) -> Option<Self::Output>;
    /// When [`tick`](Self::tick) next has something to do.
    fn deadline(&self) -> Option<Duration>;
    /// Run pending work now instead of waiting.
    fn flush(&mut self, now: Duration) -> Option<Self::Output>;
    /// Drop pending work.
    fn cancel(&mut self);
    /// A snapshot for display.
    fn state(&self, now: Duration) -> PacerState;
}

/// A monotonic clock reading for the `now` arguments: time since first use
/// natively, `Date.now()` in the browser.
#[must_use]
pub fn now() -> Duration {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        Duration::from_secs_f64(js_sys::Date::now() / 1000.0)
    }
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START.get_or_init(std::time::Instant::now).elapsed()
    }
}
