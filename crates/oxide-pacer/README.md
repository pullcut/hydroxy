# oxide-pacer

Debounce, throttle, rate limit, batch and an async queue, each with observable
state (pending, executions, last run, next allowed time) and flush/cancel.
Ported from `@hecto/pacer`.

The core has no framework, runtime or clock. Every method takes `now` as a
`Duration` since any origin, controllers report their next `deadline()`, and you
call `tick(now)` when it passes. They hand back what to run instead of running
it, so tests are plain assertions with made-up timestamps.

```rust
use std::time::Duration;
use oxide_pacer::{Debouncer, Decision, Edge, Paced};

let ms = Duration::from_millis;
let mut search = Debouncer::new(ms(300)).edge(Edge::Trailing).max_wait(ms(1000));
assert_eq!(search.call(ms(0), "o"), Decision::Scheduled);
assert_eq!(search.call(ms(100), "ox"), Decision::Scheduled);
assert_eq!(search.tick(ms(400)), Some("ox"));
```

| Controller | Builder | Call |
| --- | --- | --- |
| `Debouncer<A>` | `new(wait)`, `edge`, `max_wait` | `call(now, args) -> Decision<A>` |
| `Throttler<A>` | `new(interval)`, `edge` | `call(now, args) -> Decision<A>` |
| `RateLimiter` | `new(limit, window)`, `window(Window::Sliding)` | `try_acquire(now) -> Result<(), Rejection>` |
| `Batcher<T>` | `new()`, `max_size`, `wait` | `add(now, item) -> Option<Vec<T>>` |
| `AsyncQueue<T>` | `new(concurrency)` | `push(item, priority)`, `next(now)`, `finish(id, now, ok)`, `cancel(id)`, `pause`, `resume` |

All but the queue implement `Paced` (`tick`, `deadline`, `flush`, `cancel`,
`state`). `oxide_pacer::now()` reads a monotonic clock natively and `Date.now()`
in the browser.

## Leptos and Dioxus

Enable the `leptos` or `dioxus` feature for hooks that drive the controllers
with real timers and expose `state()` as a signal:

```rust,ignore
use oxide_pacer::leptos::*;

let query = use_debounced(text, Duration::from_millis(300));
let save = use_debounced_callback(Debouncer::new(ms(500)).max_wait(ms(2000)), move |doc| persist(doc));
let scroll = use_throttled_callback(Throttler::new(ms(100)), move |y| set_y(y));
let send = use_rate_limited_callback(RateLimiter::new(3, Duration::from_secs(5)), move |msg| post(msg));
let log = use_batcher(Batcher::new().max_size(20).wait(ms(1000)), move |batch| ship(batch));
let uploads = use_async_queue(AsyncQueue::new(2), |file| async move { upload(file).await });
```

Leptos uses `setTimeout` and clears it on cleanup. Dioxus waits through
`document::eval`, like the rest of oxide's Dioxus code, and cancels on drop.
Both compile for the server, where no timers run.
