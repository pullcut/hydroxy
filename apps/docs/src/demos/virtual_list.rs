use leptos::prelude::*;
use hydroxy_leptos::button::{Button, ButtonSize, ButtonVariant};
use hydroxy_leptos::virtual_list::{Align, VirtualList};

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
    (MERCHANTS[h % MERCHANTS.len()], format!("{:02}:{:02}", minutes / 60, minutes % 60), format!("£{}.{:02}", pence / 100, pence % 100))
}

pub fn demo() -> AnyView {
    let rows = DAYS * GROUP;
    let headers: Vec<usize> = (0..DAYS).map(|d| d * GROUP).collect();
    let (jump, set_jump) = signal(None::<(usize, Align)>);
    let (picked, set_picked) = signal(None::<usize>);
    let summary = move || {
        picked.get().map_or_else(
            || "Nothing selected".to_string(),
            |i| {
                let (merchant, time, amount) = payment(i);
                format!("{merchant}, {amount} at {time} on {}", date(i / GROUP))
            },
        )
    };

    view! {
        <div style="display: grid; gap: 24px; max-width: 760px;">
            <div style="display: grid; gap: 10px;">
                <div style="display: flex; flex-wrap: wrap; align-items: center; gap: 8px;">
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=move |_| set_jump.set(Some((0, Align::Start)))>
                        "Today"
                    </Button>
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=move |_| set_jump.set(Some((365 * GROUP, Align::Start)))>
                        "A year ago"
                    </Button>
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=move |_| set_jump.set(Some((50_001, Align::Center)))>
                        "Row 50,001"
                    </Button>
                    <span style="font-size: 13px; color: var(--text-3);" aria-live="polite">{summary}</span>
                </div>
                <div style="height: 360px;">
                    <VirtualList
                        label="Card payments"
                        count=rows
                        headers=headers
                        row_height=40.0
                        header_height=32.0
                        scroll_to=jump
                        on_select=move |i: usize| set_picked.set(Some(i))
                        row=move |i: usize| {
                            if i.is_multiple_of(GROUP) {
                                view! { <span>{date(i / GROUP)}</span> }.into_any()
                            } else {
                                let (merchant, time, amount) = payment(i);
                                view! {
                                    <span style="width: 3.2em; color: var(--text-3); font-variant-numeric: tabular-nums;">{time}</span>
                                    <span style="flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis;">{merchant}</span>
                                    <span style="font-variant-numeric: tabular-nums;">{amount}</span>
                                }
                                .into_any()
                            }
                        }
                    />
                </div>
            </div>
            <div style="height: 320px;">
                <VirtualList
                    label="Team notes"
                    count=5000_usize
                    measure=true
                    row_height=64.0
                    row=move |i: usize| {
                        let h = mix(i + 7);
                        let note = NOTES[h % NOTES.len()];
                        let more = h.is_multiple_of(3).then(|| NOTES[(h / 3) % NOTES.len()]);
                        view! {
                            <div style="display: grid; gap: 2px; min-width: 0; padding: 4px 0;">
                                <strong style="font-size: 12.5px;">{PEOPLE[h % PEOPLE.len()]}</strong>
                                <span style="white-space: normal; color: var(--text-2);">{note}{more.map(|m| format!(" {m}"))}</span>
                            </div>
                        }
                        .into_any()
                    }
                />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "virtual-list",
    name: "Virtual list",
    group: "Data",
    summary: "A listbox that only renders the rows in view, so 100,000 rows scroll as smoothly as 100. Rows can be a fixed height or measured as they render, group headers stick until the next group pushes them off, and the keyboard moves through every row with the active one always announced.",
    demo,
    leptos: include_str!("virtual_list.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/virtual_list.rs"),
    parts: &[
        &["VirtualList", "root", "Frame around the list; carries the focus ring"],
        &["", "viewport", "The scrolling role=listbox. Holds focus; aria-activedescendant names the active row. data-measured when measuring"],
        &["", "spacer", "Sized to the whole list so the scrollbar is true"],
        &["", "group", "role=group box spanning one group's rows, labelled by its header"],
        &["", "group-label", "The header. position: sticky inside its group"],
        &["", "item", "role=option with aria-setsize and aria-posinset for the whole list. data-active, data-selected, data-index"],
        &["", "empty", "Shown when there are no rows"],
    ],
    keys: &[
        &["Arrow Down / Arrow Up", "Next or previous option, skipping headers"],
        &["Page Down / Page Up", "About a screen further or back"],
        &["Home / End", "First or last option"],
        &["Enter / Space", "Select the active option"],
    ],
    props: &[
        &["count", "usize", "Number of rows, headers included"],
        &["row", "Callback<usize, AnyView>", "Renders row index, option or header"],
        &["label", "String", "Accessible name for the listbox"],
        &["row_height", "f64", "Exact row height, or the first guess when measuring. Default 36"],
        &["header_height", "Option<f64>", "Header height. Defaults to row_height"],
        &["headers", "Vec<usize>", "Indexes of sticky group header rows"],
        &["measure", "bool", "Measure rows as they render, for content-sized rows"],
        &["overscan", "usize", "Rows kept rendered beyond each edge. Default 6"],
        &["default_selected", "Option<usize>", "Option selected at first"],
        &["on_select", "Callback<usize>", "Called when an option is chosen"],
        &["scroll_to", "Option<(usize, Align)>", "Set to scroll a row into view: Start, Center, End or Auto"],
        &["empty", "String", "Text when there are no rows"],
    ],
};
