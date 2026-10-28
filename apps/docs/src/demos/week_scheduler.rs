use leptos::prelude::*;
use hydroxy_leptos::week_scheduler::*;

fn d(day: u8) -> Date {
    Date::new(2026, 11, day).expect("a real date")
}

fn at(h: u16, m: u16) -> u16 {
    h * 60 + m
}

fn week() -> Vec<WeekEvent> {
    let mut events = vec![
        WeekEvent::all_day("planning-week", "Q1 planning week", d(2)).with_tone("info"),
        WeekEvent::timed("sprint", "Sprint planning", d(2), at(10, 0), at(11, 30)).with_tone("info"),
        WeekEvent::timed("one-to-one", "1:1 with Priya", d(2), at(14, 0), at(14, 45)),
        WeekEvent::timed("design-review", "Design review: checkout", d(3), at(11, 0), at(12, 30)),
        WeekEvent::timed("northwind", "Customer call: Northwind", d(3), at(11, 30), at(12, 0)).with_tone("warning"),
        WeekEvent::timed("interview", "Interview: frontend engineer", d(3), at(15, 0), at(16, 0)).with_tone("success"),
        WeekEvent::timed("lunch-learn", "Lunch and learn: accessibility", d(4), at(12, 30), at(13, 30)).with_tone("success"),
        WeekEvent::timed("pairing", "Pairing with Sam", d(4), at(13, 0), at(14, 30)),
        WeekEvent::timed("vendor", "Vendor demo", d(4), at(13, 30), at(14, 0)).with_tone("warning"),
        WeekEvent::all_day("offsite", "Team offsite", d(5)),
        WeekEvent::all_day("freeze", "Release freeze", d(6)).with_tone("danger"),
        WeekEvent::timed("retro", "Sprint retro", d(6), at(16, 0), at(17, 0)).with_tone("info"),
    ];
    for day in 2..=6 {
        events.push(WeekEvent::timed(format!("standup-{day}"), "Standup", d(day), at(9, 30), at(9, 45)));
    }
    events
}

pub fn demo() -> AnyView {
    // Controlled: the owner holds the events. Today and now are fixed so the page is the same every day.
    let (events, set_events) = signal(week());
    let (last, set_last) = signal(String::from("Drag on empty time to add an event, or drag one to move it."));
    let next = StoredValue::new(1_u32);

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <WeekScheduler
                label="Product team calendar"
                events=events
                today=d(3)
                timezone="GMT"
                now=at(14, 25)
                on_create=move |p: Placement| {
                    let n = next.get_value();
                    next.set_value(n + 1);
                    let id = format!("new-{n}");
                    let event = if p.all_day {
                        WeekEvent::all_day(id, "New event", p.date)
                    } else {
                        WeekEvent::timed(id, "New event", p.date, p.start, p.end)
                    };
                    set_last.set(format!("Added: {}", placement_text(p)));
                    set_events.update(|list| list.push(event));
                }
                on_change=move |e: WeekEvent| {
                    set_last.set(format!("Moved: {}", event_label(&e)));
                    set_events.update(|list| apply_change(list, &e));
                }
                on_select=move |id: String| {
                    let picked = events.with_untracked(|list| list.iter().find(|e| e.id == id).map(event_label));
                    if let Some(text) = picked {
                        set_last.set(format!("Selected: {text}"));
                    }
                }
            />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">{last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "week-scheduler",
    name: "Week scheduler",
    group: "Data",
    summary: "Events on a time grid in day, work-week or week view. Drag on empty time to create an event, drag one to move it across times and days, or drag its bottom edge to resize it, all snapping to 15 minutes. Overlapping events sit side by side. Everything works from the keyboard too.",
    demo,
    leptos: include_str!("week_scheduler.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/week_scheduler.rs"),
    parts: &[
        &["WeekScheduler", "root", "role=region. data-view=day|work-week|week, data-dragging=create|move|resize during a drag"],
        &["(inside)", "toolbar, nav, prev, today, next", "Previous, Today and Next. The arrows are named \"Previous week\" or \"Next day\""],
        &["(inside)", "title", "The visible range, such as \"2 – 8 November 2026\""],
        &["(inside)", "views", "role=group of Day, Work week and Week buttons, aria-pressed"],
        &["(inside)", "header, zone, day, weekday, date", "The time zone in the corner, then one heading per day. data-today on today"],
        &["(inside)", "allday, allday-days, allday-cell, allday-event", "The all-day row. Click an empty cell to add an all-day event"],
        &["(inside)", "scroller, body, gutter, hour", "The scrolling grid and its hour labels. Opens at scroll_to_hour"],
        &["(inside)", "days, column", "The days layer that takes pointer drags, and one column per day (data-today, data-weekend)"],
        &["(inside)", "event, event-title, event-time, handle", "<div role=button>. data-tone, data-selected, data-dragging, data-short. The handle is the bottom edge"],
        &["(inside)", "ghost", "The new event being drawn"],
        &["(inside)", "now", "The line across today at the now minute"],
        &["(inside)", "slot", "<div role=button> that walks the grid with the keyboard; visible only with keyboard focus"],
        &["(inside)", "instructions, live", "Visually hidden help (aria-describedby) and a polite status region"],
    ],
    keys: &[
        &["Tab", "Move between the toolbar, all-day events, events and the slot"],
        &["Arrow Up / Down on an event", "Move it 15 minutes earlier or later"],
        &["Arrow Left / Right on an event", "Move it a day, within the visible days"],
        &["Shift + Arrow Up / Down on an event", "Change when it ends"],
        &["Enter / Space on an event", "Select it"],
        &["Arrow keys on the slot", "Move the slot; Shift with Up or Down changes its length"],
        &["Page Up / Page Down on the slot", "Move the slot an hour"],
        &["Enter / Space on the slot", "Create an event there"],
        &["Escape", "Cancel a drag"],
        &["Pointer drag", "Create, move or resize. A click on empty time makes a one-hour event. Touch scrolls the grid; drag events directly"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the region"],
        &["events", "Vec<WeekEvent>", "Required. The owner's events. WeekEvent::timed(id, title, date, start, end) or WeekEvent::all_day(id, title, date), .with_tone(..)"],
        &["today", "Date", "Required. The Today button's target and the highlighted column"],
        &["timezone", "String", "Required. Shown in the corner, such as \"GMT\""],
        &["default_date", "Option<Date>", "The date shown first. Default today"],
        &["default_view", "WeekView", "Day, WorkWeek or Week. Default Week"],
        &["week_start", "u8", "0 = Sunday. Default 1, Monday"],
        &["snap", "u16", "Minutes drags and arrow keys snap to. Default 15"],
        &["default_duration", "u16", "Minutes for an event made by a click or Enter. Default 60"],
        &["now", "Option<u16>", "Minutes after midnight for the now line. The component reads no clock"],
        &["scroll_to_hour", "u8", "Hour at the top when it opens. Default 8"],
        &["height", "u32", "Height of the scrolling grid in px. Default 560"],
        &["disabled", "bool", "Block creating, moving and resizing"],
        &["on_create", "Callback<Placement>", "A new event was drawn: date, start, end, all_day"],
        &["on_change", "Callback<WeekEvent>", "An event moved or was resized. Store it with apply_change"],
        &["on_select", "Callback<String>", "An event was clicked or Enter was pressed on it"],
        &["on_range_change", "Callback<(Date, Date)>", "The visible days changed, first to last"],
    ],
};
