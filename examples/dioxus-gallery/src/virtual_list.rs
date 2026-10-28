use dioxus::prelude::*;
use hydroxy_dioxus::button::{Button, ButtonSize, ButtonVariant};
use hydroxy_dioxus::virtual_list::{Align, VirtualList};

/// A date header, then 40 card payments.
const GROUP: usize = 41;
/// Days of history: 100,040 rows in all.
const DAYS: usize = 2440;
const MERCHANTS: [&str; 12] = [
    "Pret A Manger",
    "Transport for London",
    "Sainsbury's",
    "Waterstones",
    "Deliveroo",
    "Boots",
    "Spotify",
    "Uber",
    "Tesco Express",
    "Caffè Nero",
    "Amazon Marketplace",
    "The Gym Group",
];
const NOTES: [&str; 6] = [
    "Moved the release to Thursday so QA has a full day on the payments flow.",
    "Looks good to me.",
    "The import job retried three times overnight and then succeeded; I've added an alert so we hear about it next time rather than finding out from the dashboard.",
    "Can someone with access to the billing account check whether the October invoice went out?",
    "Shipped.",
    "I paired with Priya on the flaky search test. It was a timezone assumption in the fixture, not the code, so the fix is a one-liner in the test data.",
];
const PEOPLE: [&str; 5] = ["Amara Okafor", "Priya Shah", "Tom Fletcher", "Lena Brandt", "Jonah Reyes"];

fn mix(i: usize) -> usize {
    i.wrapping_mul(2_654_435_761) % 1_000_003
}

/// `Wed 28 Oct 2026`, `back` days before 28 October 2026.
fn date(back: usize) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    // Days since 1970-01-01, then the civil date (Howard Hinnant's algorithm).
    let days = 20_754 - back;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + usize::from(month <= 2);
    format!("{} {day} {} {year}", WEEKDAYS[(days + 4) % 7], MONTHS[month - 1])
}

/// Merchant, time and amount for payment row `i`.
fn payment(i: usize) -> (&'static str, String, String) {
    let h = mix(i);
    let minutes = 1380 - (i % GROUP) * 32 - h % 20;
    let pence = 120 + h % 9_400;
    (
        MERCHANTS[h % MERCHANTS.len()],
        format!("{:02}:{:02}", minutes / 60, minutes % 60),
        format!("£{}.{:02}", pence / 100, pence % 100),
    )
}

#[component]
pub fn VirtualListDemo() -> Element {
    let headers = use_hook(|| (0..DAYS).map(|d| d * GROUP).collect::<Vec<_>>());
    let mut jump = use_signal(|| None::<(usize, Align)>);
    let mut picked = use_signal(|| None::<usize>);
    let payments = use_callback(move |i: usize| {
        if i.is_multiple_of(GROUP) {
            return rsx! { span { "{date(i / GROUP)}" } };
        }
        let (merchant, time, amount) = payment(i);
        rsx! {
            span { style: "width: 3.2em; color: var(--text-3); font-variant-numeric: tabular-nums;", "{time}" }
            span { style: "flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis;", "{merchant}" }
            span { style: "font-variant-numeric: tabular-nums;", "{amount}" }
        }
    });
    let notes = use_callback(move |i: usize| {
        let h = mix(i + 7);
        let note = NOTES[h % NOTES.len()];
        let more = if h.is_multiple_of(3) { format!(" {}", NOTES[(h / 3) % NOTES.len()]) } else { String::new() };
        rsx! {
            div { style: "display: grid; gap: 2px; min-width: 0; padding: 4px 0;",
                strong { style: "font-size: 12.5px;", "{PEOPLE[h % PEOPLE.len()]}" }
                span { style: "white-space: normal; color: var(--text-2);", "{note}{more}" }
            }
        }
    });
    let summary = picked().map_or_else(
        || "Nothing selected".to_string(),
        |i| {
            let (merchant, time, amount) = payment(i);
            format!("{merchant}, {amount} at {time} on {}", date(i / GROUP))
        },
    );

    rsx! {
        div { style: "display: grid; gap: 24px; max-width: 760px;",
            div { style: "display: grid; gap: 10px;",
                div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 8px;",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        on_click: move |_| jump.set(Some((0, Align::Start))),
                        "Today"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        on_click: move |_| jump.set(Some((365 * GROUP, Align::Start))),
                        "A year ago"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        on_click: move |_| jump.set(Some((50_001, Align::Center))),
                        "Row 50,001"
                    }
                    span { style: "font-size: 13px; color: var(--text-3);", "aria-live": "polite", "{summary}" }
                }
                div { style: "height: 360px;",
                    VirtualList {
                        label: "Card payments",
                        count: DAYS * GROUP,
                        headers: headers.clone(),
                        row_height: 40.0,
                        header_height: 32.0,
                        scroll_to: jump,
                        on_select: move |i: usize| picked.set(Some(i)),
                        row: payments,
                    }
                }
            }
            div { style: "height: 320px;",
                VirtualList { label: "Team notes", count: 5000_usize, measure: true, row_height: 64.0, row: notes }
            }
        }
    }
}
