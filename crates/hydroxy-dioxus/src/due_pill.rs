//! `DuePill`: a day-month date that turns citrine when due soon and garnet when overdue.

use dioxus::prelude::*;
use hydroxy_headless::due::{due, parse_ymd, today as today_utc};

/// A due date. Dates are `YYYY-MM-DD`.
#[component]
pub fn DuePill(
    /// The due date. None, or anything that isn't a date, shows `empty`.
    #[props(default)]
    date: ReadSignal<Option<String>>,
    /// Compare against this date instead of today (UTC), for stable demos and tests.
    today: Option<String>,
    /// Within this many days counts as soon.
    #[props(default = 3)]
    soon_days: i64,
    /// Shown when there's no date.
    #[props(default = "–".to_string())]
    empty: String,
) -> Element {
    let now = today.and_then(|t| parse_ymd(&t)).unwrap_or_else(today_utc);
    match date().and_then(|d| due(&d, now, soon_days)) {
        Some(d) => rsx! {
            span {
                "data-scope": "due-pill",
                "data-state": d.state.as_str(),
                title: "{d.title}",
                "aria-label": "{d.title}",
                "{d.label}"
            }
        },
        None => rsx! { span { "data-scope": "due-pill", "data-state": "none", "{empty}" } },
    }
}
