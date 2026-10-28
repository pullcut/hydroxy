use std::time::Duration;

use leptos::prelude::*;
use hydroxy_leptos::log_stream::{Entry, Level, LogStream};

const LINES: [(Level, &str); 8] = [
    (Level::Info, "GET /api/summaries 200 41ms"),
    (Level::Debug, "cache miss for key summaries:v3"),
    (Level::Info, "worker 2 picked up job import-tickets"),
    (Level::Warn, "retrying source fetch (attempt 2 of 3)"),
    (Level::Info, "indexed 128 documents in 1.2s"),
    (Level::Error, "connection reset while reading support-tickets.csv"),
    (Level::Debug, "flushing 14 pending writes"),
    (Level::Info, "POST /api/answers 201 312ms"),
];

pub fn demo() -> AnyView {
    // New lines arrive in small batches, like a tail of a real log.
    let (batch, set_batch) = signal(
        LINES.iter().take(5).enumerate().map(|(i, (l, t))| Entry::new(*l, format!("09:41:{:02}", 30 + i), *t)).collect::<Vec<_>>(),
    );
    let tick = StoredValue::new(5_usize);
    Effect::new(move |_| {
        let handle = set_interval_with_handle(
            move || {
                let n = tick.get_value();
                tick.set_value(n + 1);
                let (level, text) = LINES[n % LINES.len()];
                set_batch.set(vec![Entry::new(level, format!("09:{:02}:{:02}", 41 + (30 + n) / 60, (30 + n) % 60), format!("{text} #{n}"))]);
            },
            Duration::from_millis(900),
        )
        .ok();
        on_cleanup(move || {
            if let Some(handle) = handle {
                handle.clear();
            }
        });
    });

    view! {
        <div style="height: 380px; max-width: 760px;">
            <LogStream entries=batch max_lines=500 label="Worker log" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "log-stream",
    name: "Log stream",
    group: "Data",
    summary: "A live, monospace log. It follows the tail until you scroll away, holds new lines while paused, filters by level and text, and keeps only the newest lines so long logs stay cheap.",
    demo,
    leptos: include_str!("log_stream.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/log_stream.rs"),
    parts: &[
        &["LogStream", "root", "Toolbar over the viewport. data-state: following, scrolled or paused"],
        &["", "toolbar", "Level toggles, search, and the pause, follow, wrap and clear buttons"],
        &["", "level", "Toggle per level, coloured by severity. aria-pressed"],
        &["", "viewport", "The scrolling role=log region. data-wrap when wrapping"],
        &["", "line", "One line. data-level"],
        &["", "time / tag / text", "Timestamp, level tag and message"],
        &["", "empty", "Shown when no line passes the filters"],
    ],
    keys: &[
        &["Arrow keys / Page keys", "Scroll the focused log"],
        &["Escape", "In the filter field: clear it"],
        &["Enter / Space", "On a toolbar button: toggle it"],
    ],
    props: &[
        &["entries", "Vec<LogEntry>", "Each change appends these lines. Set it to the lines that just arrived"],
        &["max_lines", "usize", "Most lines kept; the oldest drop first. Default 1000"],
        &["paused", "Option<bool>", "Controlled pause. Pair with on_paused_change"],
        &["default_paused", "bool", "Initial pause when uncontrolled"],
        &["on_paused_change", "Callback<bool>", "Called when the pause button is pressed"],
        &["label / empty", "String", "Accessible name and empty text"],
    ],
};
