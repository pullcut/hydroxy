//! The pacer hooks in Dioxus: the same playground as the docs page.

use std::time::Duration;

use dioxus::prelude::*;
use oxide_pacer::dioxus::{use_batcher, use_debounced, use_rate_limited_callback, use_throttled_callback};
use oxide_pacer::{Batcher, RateLimiter, Throttler};

#[component]
fn Pacer() -> Element {
    let mut text = use_signal(String::new);
    let settled = use_debounced(text.into(), Duration::from_millis(400));
    let mut sampled = use_signal(String::new);
    let throttled = use_throttled_callback(Throttler::new(Duration::from_millis(500)), move |v: String| sampled.set(v));

    let mut sent = use_signal(|| 0u32);
    let mut note = use_signal(String::new);
    let send = use_rate_limited_callback(RateLimiter::new(3, Duration::from_secs(5)), move |()| sent += 1);

    let mut batches = use_signal(Vec::<String>::new);
    let mut count = use_signal(|| 0u32);
    let clicks = use_batcher(Batcher::new().max_size(5).wait(Duration::from_secs(1)), move |batch: Vec<u32>| {
        batches.write().insert(0, format!("{} click(s): {batch:?}", batch.len()));
    });

    rsx! {
        input {
            placeholder: "debounce vs throttle",
            oninput: move |ev| {
                text.set(ev.value());
                throttled.call(ev.value());
            },
        }
        p { "Debounced: \"{settled}\"" }
        p { "Throttled: \"{sampled}\" from {throttled.state().read().executions} runs" }

        button {
            onclick: move |_| match send.call(()) {
                Ok(()) => note.set(String::new()),
                Err(no) => note.set(format!("Try again in {:.1}s.", no.retry_after.as_secs_f64())),
            },
            "Send (3 per 5s)"
        }
        p { "Sent {sent} · {send.state().read().remaining.unwrap_or(0)} left · {note}" }

        button {
            onclick: move |_| {
                count += 1;
                clicks.add(count());
            },
            "Click"
        }
        button { onclick: move |_| clicks.flush(), "Flush now" }
        p { "Waiting: {clicks.state().read().pending}" }
        for b in batches.iter().take(4) {
            p { "{b}" }
        }
    }
}

fn main() {
    let _ = Pacer;
}
