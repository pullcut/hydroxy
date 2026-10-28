use dioxus::prelude::*;
use hydroxy_dioxus::log_stream::{Entry, Level, LogStream};

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

#[component]
pub fn LogStreamDemo() -> Element {
    // New lines arrive in small batches, like a tail of a real log.
    let mut batch = use_signal(|| {
        LINES
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, (l, t))| Entry::new(*l, format!("09:41:{:02}", 30 + i), *t))
            .collect::<Vec<_>>()
    });
    use_future(move || async move {
        let mut n = 5_usize;
        loop {
            let _ = document::eval("await new Promise((r) => setTimeout(r, 900));").await;
            let (level, text) = LINES[n % LINES.len()];
            batch.set(vec![Entry::new(
                level,
                format!("09:{:02}:{:02}", 41 + (30 + n) / 60, (30 + n) % 60),
                format!("{text} #{n}"),
            )]);
            n += 1;
        }
    });

    rsx! {
        div { style: "height: 380px; max-width: 760px;",
            LogStream { entries: batch, max_lines: 500, label: "Worker log" }
        }
    }
}
