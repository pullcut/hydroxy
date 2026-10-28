use std::time::Duration;

use leptos::prelude::*;
use oxide_leptos::button::*;
use oxide_leptos::input::Input;
use oxide_pacer::leptos::{use_async_queue, use_batcher, use_debounced, use_rate_limited_callback, use_throttled_callback};
use oxide_pacer::{AsyncQueue, Batcher, PacerState, RateLimiter, Throttler};

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

pub fn demo() -> AnyView {
    // Debounce and throttle the same keystrokes.
    let (text, set_text) = signal(String::new());
    let (keys, set_keys) = signal(0u32);
    let settled = use_debounced::<String>(text, ms(400));
    let (sampled, set_sampled) = signal(String::new());
    let throttled = use_throttled_callback(Throttler::new(ms(500)), move |v: String| set_sampled.set(v));

    // Three sends per five seconds.
    let (sent, set_sent) = signal(0u32);
    let (note, set_note) = signal(String::new());
    let limited = use_rate_limited_callback(RateLimiter::new(3, Duration::from_secs(5)), move |()| {
        set_sent.update(|n| *n += 1);
    });

    // Clicks leave in batches of five, or a second after the first.
    let (batches, set_batches) = signal(Vec::<String>::new());
    let clicks = use_batcher(Batcher::new().max_size(5).wait(Duration::from_secs(1)), move |batch: Vec<u32>| {
        set_batches.update(|b| b.insert(0, format!("{} click(s): {batch:?}", batch.len())));
    });
    let (count, set_count) = signal(0u32);

    // Two fake uploads at a time.
    let uploads = use_async_queue(AsyncQueue::new(2), |secs: u64| async move {
        oxide_leptos::async_validated_input::sleep(u32::try_from(secs * 700).unwrap_or(0)).await;
        Ok::<(), ()>(())
    });

    let row = "display: grid; gap: var(--space-3); padding: var(--space-5); border: 1px solid var(--border-1); border-radius: var(--radius-3);";
    let line = "margin: 0; font: var(--text-sm); color: var(--text-2);";

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 560px;">
            <section style=row>
                <Input
                    label="Type quickly"
                    placeholder="debounce vs throttle"
                    on_input=Callback::new(move |v: String| {
                        set_keys.update(|n| *n += 1);
                        set_text.set(v.clone());
                        throttled.call(v);
                    })
                />
                <p style=line>{move || format!("Keystrokes: {}", keys.get())}</p>
                <p style=line>{move || format!("Debounced (400ms quiet): \"{}\"", settled.get())}</p>
                <p style=line>{move || format!("Throttled (every 500ms): \"{}\" from {} runs", sampled.get(), throttled.state().get().executions)}</p>
            </section>

            <section style=row>
                <div>
                    <Button on_click=move |_| match limited.call(()) {
                        Ok(()) => set_note.set(String::new()),
                        Err(no) => set_note.set(format!("Limit of {} reached. Try again in {:.1}s.", no.limit, no.retry_after.as_secs_f64())),
                    }>"Send (3 per 5s)"</Button>
                </div>
                <p style=line>{move || format!("Sent {} · {} left · {} turned away", sent.get(), limited.state().get().remaining.unwrap_or(0), limited.state().get().rejections)}</p>
                <p style=line aria-live="polite">{note}</p>
            </section>

            <section style=row>
                <div style="display: flex; gap: var(--space-3);">
                    <Button variant=ButtonVariant::Secondary on_click=move |_| {
                        set_count.update(|n| *n += 1);
                        clicks.add(count.get_untracked());
                    }>"Click"</Button>
                    <Button variant=ButtonVariant::Ghost on_click=move |_| clicks.flush()>"Flush now"</Button>
                </div>
                <p style=line>{move || format!("Waiting: {}", clicks.state().get().pending)}</p>
                <ul style="margin: 0; padding-left: var(--space-6); font: var(--text-sm);">
                    {move || batches.get().into_iter().take(4).map(|b| view! { <li>{b}</li> }).collect_view()}
                </ul>
            </section>

            <section style=row>
                <div style="display: flex; gap: var(--space-3); flex-wrap: wrap;">
                    <Button variant=ButtonVariant::Secondary on_click=move |_| { uploads.push(2, 0); }>"Queue a job"</Button>
                    <Button variant=ButtonVariant::Secondary on_click=move |_| { uploads.push(1, 10); }>"Queue urgent"</Button>
                    <Button variant=ButtonVariant::Ghost on_click=move |_| {
                        if uploads.state().get_untracked().status == oxide_pacer::Status::Paused { uploads.resume() } else { uploads.pause() }
                    }>{move || if uploads.state().get().status == oxide_pacer::Status::Paused { "Resume" } else { "Pause" }}</Button>
                </div>
                <p style=line>{move || queue_line(uploads.state().get())}</p>
            </section>
        </div>
    }
    .into_any()
}

fn queue_line(s: PacerState) -> String {
    format!("{:?}: {} running, {} waiting, {} done", s.status, s.active, s.pending, s.executions)
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "pacer",
    name: "Pacer",
    group: "Utilities",
    summary: "Debounce, throttle, rate limit, batch and an async queue with priority and cancellation. A clock-free core you drive with timestamps, plus Leptos and Dioxus hooks that expose each controller's state as a signal.",
    demo,
    leptos: include_str!("pacer.rs"),
    dioxus: include_str!("../../../../crates/oxide-pacer/examples/dioxus.rs"),
    parts: &[],
    keys: &[],
    props: &[
        &["use_debounced", "(Signal<T>, Duration) -> Signal<T>", "Follows the source once it has been still for the wait"],
        &["use_debounced_callback", "(Debouncer<A>, Fn(A)) -> Debounced<A>", "call, flush, cancel, state. Edge::Leading / Trailing / Both and max_wait on the Debouncer"],
        &["use_throttled_callback", "(Throttler<A>, Fn(A)) -> Throttled<A>", "At most once per interval; both edges by default"],
        &["use_rate_limited_callback", "(RateLimiter, Fn(A)) -> RateLimited<A>", "call returns Err(Rejection { limit, retry_at, retry_after }). Window::Fixed or Sliding"],
        &["use_batcher", "(Batcher<T>, Fn(Vec<T>)) -> Batched<T>", "Hands over max_size items, or wait after the oldest"],
        &["use_async_queue", "(AsyncQueue<T>, Fn(T) -> Future<Result>) -> Queue<T>", "push(item, priority), cancel, pause, resume, clear. Concurrency on the AsyncQueue"],
        &["PacerState", "status, pending, active, executions, rejections, last_run, next_run, next_allowed, remaining", "What every state() signal holds"],
    ],
};
