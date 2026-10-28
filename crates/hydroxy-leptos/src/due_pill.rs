//! `<DuePill>`: a day-month date that turns citrine when due soon and garnet when overdue.

use hydroxy_headless::due::{due, parse_ymd, today as today_utc};
use leptos::either::Either;
use leptos::prelude::*;

/// A due date. Dates are `YYYY-MM-DD`.
#[component]
pub fn DuePill(
    /// The due date. None, or anything that isn't a date, shows `empty`.
    #[prop(optional, into)]
    date: MaybeProp<String>,
    /// Compare against this date instead of today (UTC), for stable demos and tests.
    #[prop(optional, into)]
    today: Option<String>,
    /// Within this many days counts as soon.
    #[prop(default = 3)]
    soon_days: i64,
    /// Shown when there's no date.
    #[prop(default = "–".into(), into)]
    empty: String,
) -> impl IntoView {
    let now = today.and_then(|t| parse_ymd(&t)).unwrap_or_else(today_utc);
    let info = move || date.get().and_then(|d| due(&d, now, soon_days));
    move || match info() {
        Some(d) => Either::Left(view! {
            <span data-scope="due-pill" data-state=d.state.as_str() title=d.title.clone() aria-label=d.title>
                {d.label}
            </span>
        }),
        None => Either::Right(view! { <span data-scope="due-pill" data-state="none">{empty.clone()}</span> }),
    }
}
