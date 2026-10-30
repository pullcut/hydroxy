use hydroxy_leptos::due_pill::DuePill;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // A fixed "today" keeps the example stable; leave it out to use the real date.
    let tasks = [
        ("Send the Q3 summary", Some("2026-10-24")),
        ("Review flagged answers", Some("2026-10-27")),
        ("Plan the November interviews", Some("2026-11-12")),
        ("Archive old drafts", None),
    ];
    view! {
        <ul style="list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4); max-width: 360px;">
            {tasks
                .into_iter()
                .map(|(task, date)| {
                    view! {
                        <li style="display: flex; align-items: center; justify-content: space-between; gap: 10px; font: var(--text-sm);">
                            {task}
                            <DuePill date=date.map(String::from) today="2026-10-26" />
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "due-pill",
    name: "Due pill",
    group: "Display",
    summary: "A day-month date that turns citrine when it's due within three days and garnet once it's overdue.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("due_pill.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/due_pill.rs"),
    parts: &[&["DuePill", "—", "span[data-scope=due-pill][data-state=later|soon|overdue|none]"]],
    keys: &[],
    props: &[
        &["date", "Option<String>", "YYYY-MM-DD. Missing or invalid shows `empty`"],
        &["today", "String", "Compare against this date. Default: today in UTC"],
        &["soon_days", "i64", "Within this many days counts as soon. Default 3"],
        &["empty", "String", "Shown without a date. Default –"],
    ],
    events: &[],
};
