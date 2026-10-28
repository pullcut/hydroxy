//! Leptos hooks: each drives a core controller with `setTimeout` and exposes
//! its [`PacerState`] as a signal. Timers are cleared when the owner is
//! cleaned up; after that, calls do nothing.
//!
//! ```no_run
//! use std::time::Duration;
//! use leptos::prelude::*;
//! use hydroxy_pacer::{Debouncer, leptos::use_debounced_callback};
//!
//! #[component]
//! fn Search() -> impl IntoView {
//!     let search = use_debounced_callback(Debouncer::new(Duration::from_millis(300)), |q: String| {
//!         leptos::logging::log!("searching {q}");
//!     });
//!     view! { <input on:input=move |ev| { search.call(event_target_value(&ev)); } /> }
//! }
//! ```

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    AsyncQueue, Batcher, Debouncer, Decision, JobId, Paced, PacerState, RateLimiter, Rejection, Throttler, now,
};

type Callback<A> = StoredValue<Box<dyn Fn(A)>, LocalStorage>;

/// Owns a controller, its callback and its timer.
struct Engine<P: Paced + 'static> {
    pacer: StoredValue<P, LocalStorage>,
    run: Callback<P::Output>,
    timer: StoredValue<Option<TimeoutHandle>, LocalStorage>,
    state: RwSignal<PacerState>,
}

impl<P: Paced + 'static> Clone for Engine<P> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: Paced + 'static> Copy for Engine<P> {}

impl<P: Paced + 'static> Engine<P> {
    fn new(pacer: P, run: impl Fn(P::Output) + 'static) -> Self {
        let engine = Self {
            state: RwSignal::new(pacer.state(now())),
            pacer: StoredValue::new_local(pacer),
            run: StoredValue::new_local(Box::new(run)),
            timer: StoredValue::new_local(None),
        };
        // Timers only exist in the browser. On the server the owner can be dropped on another
        // thread, where touching these thread-local values would panic.
        if cfg!(target_arch = "wasm32") {
            on_cleanup(move || engine.clear_timer());
        }
        engine
    }

    fn clear_timer(self) {
        if let Some(handle) = self.timer.try_update_value(Option::take).flatten() {
            handle.clear();
        }
    }

    fn emit(self, out: Option<P::Output>) {
        if let Some(out) = out {
            self.run.try_with_value(|run| run(out));
        }
    }

    /// Fire anything overdue, apply `f`, then re-arm the timer and publish.
    fn with<R>(self, f: impl FnOnce(&mut P, Duration) -> R) -> Option<R> {
        let at = now();
        let due = self.pacer.try_update_value(|p| p.tick(at)).flatten();
        self.emit(due);
        let out = self.pacer.try_update_value(|p| f(p, at));
        self.sync();
        out
    }

    fn sync(self) {
        self.clear_timer();
        let at = now();
        let Some((state, deadline)) = self.pacer.try_with_value(|p| (p.state(at), p.deadline())) else { return };
        if self.state.try_get_untracked().is_some_and(|s| s != state) {
            self.state.try_set(state);
        }
        if let Some(deadline) = deadline {
            let handle = set_timeout_with_handle(
                move || {
                    self.with(|_, _| ());
                },
                deadline.saturating_sub(at),
            )
            .ok();
            self.timer.try_set_value(handle);
        }
    }

    fn decide(self, decision: Option<Decision<P::Output>>) -> Decision<()> {
        match decision {
            Some(Decision::Run(args)) => {
                self.emit(Some(args));
                Decision::Run(())
            }
            Some(Decision::Scheduled) => Decision::Scheduled,
            _ => Decision::Suppressed,
        }
    }
}

/// A paced callback returned by the `use_*_callback` hooks and [`use_batcher`].
pub struct Handle<P: Paced + 'static> {
    engine: Engine<P>,
}

impl<P: Paced + 'static> Clone for Handle<P> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: Paced + 'static> Copy for Handle<P> {}

impl<P: Paced + 'static> Handle<P> {
    /// The controller's state, kept current.
    #[must_use]
    pub fn state(&self) -> ReadSignal<PacerState> {
        self.engine.state.read_only()
    }

    /// Run pending work now.
    pub fn flush(&self) {
        let out = self.engine.with(P::flush).flatten();
        self.engine.emit(out);
    }

    /// Drop pending work.
    pub fn cancel(&self) {
        self.engine.with(|p, _| p.cancel());
    }
}

/// A debounced callback.
pub type Debounced<A> = Handle<Debouncer<A>>;
/// A throttled callback.
pub type Throttled<A> = Handle<Throttler<A>>;
/// A batching callback.
pub type Batched<T> = Handle<Batcher<T>>;

impl<A: 'static> Handle<Debouncer<A>> {
    /// Offer a call; `Run` means the callback already ran.
    pub fn call(&self, args: A) -> Decision<()> {
        self.engine.decide(self.engine.with(|p, at| p.call(at, args)))
    }
}

impl<A: 'static> Handle<Throttler<A>> {
    /// Offer a call; `Run` means the callback already ran.
    pub fn call(&self, args: A) -> Decision<()> {
        self.engine.decide(self.engine.with(|p, at| p.call(at, args)))
    }
}

impl<T: 'static> Handle<Batcher<T>> {
    /// Add an item; the callback runs with the batch once it is due.
    pub fn add(&self, item: T) {
        let out = self.engine.with(|p, at| p.add(at, item)).flatten();
        self.engine.emit(out);
    }
}

/// Debounce a callback. Build the [`Debouncer`] for edges and max wait.
pub fn use_debounced_callback<A: 'static>(debouncer: Debouncer<A>, callback: impl Fn(A) + 'static) -> Debounced<A> {
    Handle { engine: Engine::new(debouncer, callback) }
}

/// Throttle a callback. Build the [`Throttler`] to pick edges.
pub fn use_throttled_callback<A: 'static>(throttler: Throttler<A>, callback: impl Fn(A) + 'static) -> Throttled<A> {
    Handle { engine: Engine::new(throttler, callback) }
}

/// Collect items and run the callback with each batch.
pub fn use_batcher<T: 'static>(batcher: Batcher<T>, callback: impl Fn(Vec<T>) + 'static) -> Batched<T> {
    Handle { engine: Engine::new(batcher, callback) }
}

/// A signal that follows `source` once it has been still for `wait`.
#[must_use]
pub fn use_debounced<T>(source: impl Into<Signal<T>>, wait: Duration) -> Signal<T>
where
    T: Clone + Send + Sync + 'static,
{
    let source = source.into();
    let out = RwSignal::new(source.get_untracked());
    let debounced = use_debounced_callback(Debouncer::new(wait), move |v| {
        out.try_set(v);
    });
    Effect::watch(
        move || source.get(),
        move |v, _, _| {
            debounced.call(v.clone());
        },
        false,
    );
    out.into()
}

/// A rate-limited callback from [`use_rate_limited_callback`].
pub struct RateLimited<A: 'static> {
    engine: Engine<RateLimiter>,
    callback: Callback<A>,
}

impl<A: 'static> Clone for RateLimited<A> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<A: 'static> Copy for RateLimited<A> {}

impl<A: 'static> RateLimited<A> {
    /// Run the callback if the limit allows.
    ///
    /// # Errors
    ///
    /// A [`Rejection`] saying when to retry when the window is full.
    pub fn call(&self, args: A) -> Result<(), Rejection> {
        match self.engine.with(RateLimiter::try_acquire) {
            Some(Ok(())) => {
                self.callback.try_with_value(|run| run(args));
                Ok(())
            }
            Some(Err(rejection)) => Err(rejection),
            None => Ok(()),
        }
    }

    /// The limiter's state, kept current as the window moves.
    #[must_use]
    pub fn state(&self) -> ReadSignal<PacerState> {
        self.engine.state.read_only()
    }

    /// Forget past calls, freeing every slot.
    pub fn reset(&self) {
        self.engine.with(|p, _| p.cancel());
    }
}

/// Let at most the limiter's quota of calls through.
pub fn use_rate_limited_callback<A: 'static>(limiter: RateLimiter, callback: impl Fn(A) + 'static) -> RateLimited<A> {
    RateLimited {
        engine: Engine::new(limiter, |never: Infallible| match never {}),
        callback: StoredValue::new_local(Box::new(callback)),
    }
}

type Worker<T> = Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = bool>>>>;

/// An async job queue from [`use_async_queue`].
pub struct Queue<T: 'static> {
    inner: StoredValue<AsyncQueue<T>, LocalStorage>,
    worker: StoredValue<Worker<T>, LocalStorage>,
    state: RwSignal<PacerState>,
}

impl<T: 'static> Clone for Queue<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for Queue<T> {}

impl<T: 'static> Queue<T> {
    /// Queue a job. Higher `priority` runs sooner.
    pub fn push(&self, item: T, priority: i32) -> Option<JobId> {
        let id = self.inner.try_update_value(|q| q.push(item, priority));
        self.pump();
        id
    }

    /// Cancel a job: waiting ones are dropped, running ones have their result ignored.
    #[allow(clippy::must_use_candidate, reason = "cancelling is the point; the flag is extra")]
    pub fn cancel(&self, id: JobId) -> bool {
        let done = self.inner.try_update_value(|q| q.cancel(id)).unwrap_or(false);
        self.publish();
        done
    }

    /// Drop every waiting job.
    pub fn clear(&self) {
        self.inner.try_update_value(AsyncQueue::clear);
        self.publish();
    }

    /// Stop starting jobs.
    pub fn pause(&self) {
        self.inner.try_update_value(AsyncQueue::pause);
        self.publish();
    }

    /// Start jobs again.
    pub fn resume(&self) {
        self.inner.try_update_value(AsyncQueue::resume);
        self.pump();
    }

    /// The queue's state, kept current.
    #[must_use]
    pub fn state(&self) -> ReadSignal<PacerState> {
        self.state.read_only()
    }

    fn publish(self) {
        if let Some(state) = self.inner.try_with_value(AsyncQueue::state) {
            self.state.try_set(state);
        }
    }

    fn pump(self) {
        while let Some(Some((id, item))) = self.inner.try_update_value(|q| q.next(now())) {
            let Some(job) = self.worker.try_with_value(|w| w(item)) else { break };
            spawn_local(async move {
                let ok = job.await;
                if self.inner.try_update_value(|q| q.finish(id, now(), ok)).is_some() {
                    self.pump();
                }
            });
        }
        self.publish();
    }
}

/// Run `worker` over queued jobs, by priority, up to the queue's concurrency.
/// A job counts as an execution when its future returns `Ok`, else a rejection.
pub fn use_async_queue<T, R, E, Fut>(queue: AsyncQueue<T>, worker: impl Fn(T) -> Fut + 'static) -> Queue<T>
where
    T: 'static,
    Fut: Future<Output = Result<R, E>> + 'static,
{
    let worker: Worker<T> = Box::new(move |item| {
        let job = worker(item);
        Box::pin(async move { job.await.is_ok() })
    });
    Queue {
        state: RwSignal::new(queue.state()),
        inner: StoredValue::new_local(queue),
        worker: StoredValue::new_local(worker),
    }
}
