//! Dioxus hooks: each drives a core controller with a timer task and exposes
//! its [`PacerState`] as a signal. Timers wait with `setTimeout` through
//! `document::eval`, like the rest of hydroxy's Dioxus code, and are cancelled
//! when the component is dropped.
//!
//! ```no_run
//! use std::time::Duration;
//! use dioxus::prelude::*;
//! use hydroxy_pacer::{Debouncer, dioxus::use_debounced_callback};
//!
//! #[component]
//! fn Search() -> Element {
//!     let search = use_debounced_callback(Debouncer::new(Duration::from_millis(300)), |q: String| {
//!         println!("searching {q}");
//!     });
//!     rsx! { input { oninput: move |ev| { search.call(ev.value()); } } }
//! }
//! ```

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use dioxus::core::Task;
use dioxus::prelude::*;

use crate::{AsyncQueue, Batcher, Debouncer, Decision, JobId, PacerState, Paced, RateLimiter, Rejection, Throttler, now};

async fn sleep(for_: Duration) {
    let ms = for_.as_millis();
    let _ = document::eval(&format!("await new Promise(r => setTimeout(r, {ms})); return true;")).await;
}

/// Owns a controller, its callback and its timer task.
struct Engine<P: Paced + 'static> {
    pacer: CopyValue<P>,
    run: CopyValue<Box<dyn FnMut(P::Output)>>,
    timer: CopyValue<Option<Task>>,
    state: Signal<PacerState>,
}

impl<P: Paced + 'static> Clone for Engine<P> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: Paced + 'static> Copy for Engine<P> {}

impl<P: Paced + 'static> Engine<P> {
    fn use_new(pacer: impl FnOnce() -> P, run: impl FnMut(P::Output) + 'static) -> Self {
        let engine = use_hook(move || {
            let pacer = pacer();
            let run: Box<dyn FnMut(P::Output)> = Box::new(run);
            Self {
                state: Signal::new(pacer.state(now())),
                pacer: CopyValue::new(pacer),
                run: CopyValue::new(run),
                timer: CopyValue::new(None),
            }
        });
        use_drop(move || engine.clear_timer());
        engine
    }

    fn clear_timer(self) {
        let mut timer = self.timer;
        if let Some(task) = timer.try_write().ok().and_then(|mut t| t.take()) {
            task.cancel();
        }
    }

    fn emit(self, out: Option<P::Output>) {
        let mut run = self.run;
        if let (Some(out), Ok(mut run)) = (out, run.try_write()) {
            run(out);
        }
    }

    /// Fire anything overdue, apply `f`, then re-arm the timer and publish.
    fn with<R>(self, f: impl FnOnce(&mut P, Duration) -> R) -> Option<R> {
        let at = now();
        let mut pacer = self.pacer;
        let due = pacer.try_write().ok().and_then(|mut p| p.tick(at));
        self.emit(due);
        let out = pacer.try_write().ok().map(|mut p| f(&mut p, at));
        self.sync();
        out
    }

    fn sync(self) {
        self.clear_timer();
        let at = now();
        let Some((state, deadline)) = self.pacer.try_read().ok().map(|p| (p.state(at), p.deadline())) else { return };
        let mut signal = self.state;
        if signal.try_peek().is_ok_and(|s| *s != state) {
            signal.set(state);
        }
        if let Some(deadline) = deadline {
            let task = spawn(async move {
                sleep(deadline.saturating_sub(at)).await;
                // This task is the timer; forget it before `with` cancels it.
                let mut timer = self.timer;
                if let Ok(mut t) = timer.try_write() {
                    t.take();
                }
                self.with(|_, _| ());
            });
            let mut timer = self.timer;
            if let Ok(mut t) = timer.try_write() {
                *t = Some(task);
            }
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

impl<P: Paced + 'static> PartialEq for Handle<P> {
    fn eq(&self, other: &Self) -> bool {
        self.engine.state == other.engine.state
    }
}

impl<P: Paced + 'static> Handle<P> {
    /// The controller's state, kept current.
    #[must_use]
    pub fn state(&self) -> ReadSignal<PacerState> {
        self.engine.state.into()
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

/// Debounce a callback. The [`Debouncer`] is built once, on first render.
pub fn use_debounced_callback<A: 'static>(debouncer: Debouncer<A>, callback: impl FnMut(A) + 'static) -> Debounced<A> {
    Handle { engine: Engine::use_new(|| debouncer, callback) }
}

/// Throttle a callback. The [`Throttler`] is built once, on first render.
pub fn use_throttled_callback<A: 'static>(throttler: Throttler<A>, callback: impl FnMut(A) + 'static) -> Throttled<A> {
    Handle { engine: Engine::use_new(|| throttler, callback) }
}

/// Collect items and run the callback with each batch.
pub fn use_batcher<T: 'static>(batcher: Batcher<T>, callback: impl FnMut(Vec<T>) + 'static) -> Batched<T> {
    Handle { engine: Engine::use_new(|| batcher, callback) }
}

/// A signal that follows `source` once it has been still for `wait`.
#[must_use]
pub fn use_debounced<T: Clone + PartialEq + 'static>(source: ReadSignal<T>, wait: Duration) -> ReadSignal<T> {
    let mut out = use_signal(|| source.cloned());
    let debounced = use_debounced_callback(Debouncer::new(wait), move |v: T| out.set(v));
    use_effect(move || {
        let v = source.cloned();
        if *out.peek() != v {
            debounced.call(v);
        }
    });
    out.into()
}

/// A rate-limited callback from [`use_rate_limited_callback`].
pub struct RateLimited<A: 'static> {
    engine: Engine<RateLimiter>,
    callback: CopyValue<Box<dyn FnMut(A)>>,
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
                let mut callback = self.callback;
                if let Ok(mut run) = callback.try_write() {
                    run(args);
                }
                Ok(())
            }
            Some(Err(rejection)) => Err(rejection),
            None => Ok(()),
        }
    }

    /// The limiter's state, kept current as the window moves.
    #[must_use]
    pub fn state(&self) -> ReadSignal<PacerState> {
        self.engine.state.into()
    }

    /// Forget past calls, freeing every slot.
    pub fn reset(&self) {
        self.engine.with(|p, _| p.cancel());
    }
}

/// Let at most the limiter's quota of calls through.
pub fn use_rate_limited_callback<A: 'static>(limiter: RateLimiter, callback: impl FnMut(A) + 'static) -> RateLimited<A> {
    let engine = Engine::use_new(|| limiter, |never: Infallible| match never {});
    let callback = use_hook(move || CopyValue::new(Box::new(callback) as Box<dyn FnMut(A)>));
    RateLimited { engine, callback }
}

type Worker<T> = Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = bool>>>>;

/// An async job queue from [`use_async_queue`].
pub struct Queue<T: 'static> {
    inner: CopyValue<AsyncQueue<T>>,
    worker: CopyValue<Worker<T>>,
    state: Signal<PacerState>,
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
        let mut queue = self.inner;
        let id = queue.try_write().ok().map(|mut q| q.push(item, priority));
        self.pump();
        id
    }

    /// Cancel a job: waiting ones are dropped, running ones have their result ignored.
    #[allow(clippy::must_use_candidate, reason = "cancelling is the point; the flag is extra")]
    pub fn cancel(&self, id: JobId) -> bool {
        let mut queue = self.inner;
        let done = queue.try_write().is_ok_and(|mut q| q.cancel(id));
        self.publish();
        done
    }

    /// Drop every waiting job.
    pub fn clear(&self) {
        self.update(AsyncQueue::clear);
        self.publish();
    }

    /// Stop starting jobs.
    pub fn pause(&self) {
        self.update(AsyncQueue::pause);
        self.publish();
    }

    /// Start jobs again.
    pub fn resume(&self) {
        self.update(AsyncQueue::resume);
        self.pump();
    }

    /// The queue's state, kept current.
    #[must_use]
    pub fn state(&self) -> ReadSignal<PacerState> {
        self.state.into()
    }

    fn update(self, f: impl FnOnce(&mut AsyncQueue<T>)) {
        let mut queue = self.inner;
        if let Ok(mut q) = queue.try_write() {
            f(&mut q);
        }
    }

    fn publish(self) {
        let Some(state) = self.inner.try_read().ok().map(|q| q.state()) else { return };
        let mut signal = self.state;
        if signal.try_peek().is_ok_and(|s| *s != state) {
            signal.set(state);
        }
    }

    fn pump(self) {
        let mut queue = self.inner;
        loop {
            let Some((id, item)) = queue.try_write().ok().and_then(|mut q| q.next(now())) else { break };
            let Some(job) = self.worker.try_read().ok().map(|w| w(item)) else { break };
            spawn(async move {
                let ok = job.await;
                if queue.try_write().is_ok_and(|mut q| {
                    q.finish(id, now(), ok);
                    true
                }) {
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
    use_hook(move || {
        let worker: Worker<T> = Box::new(move |item| {
            let job = worker(item);
            Box::pin(async move { job.await.is_ok() })
        });
        Queue { state: Signal::new(queue.state()), inner: CopyValue::new(queue), worker: CopyValue::new(worker) }
    })
}
