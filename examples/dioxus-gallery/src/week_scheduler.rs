use dioxus::prelude::*;
use oxide_dioxus::week_scheduler::{Date, Placement, WeekEvent, WeekScheduler, apply_change, event_label, placement_text};

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

#[component]
pub fn WeekSchedulerDemo() -> Element {
    // Controlled: the owner holds the events. Today and now are fixed so the page is the same every day.
    let mut events = use_signal(week);
    let mut last = use_signal(|| String::from("Drag on empty time to add an event, or drag one to move it."));
    let mut next = use_signal(|| 1_u32);

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            WeekScheduler {
                label: "Product team calendar",
                events: events(),
                today: d(3),
                timezone: "GMT",
                now: Some(at(14, 25)),
                on_create: move |p: Placement| {
                    let n = next();
                    next.set(n + 1);
                    let id = format!("new-{n}");
                    let event = if p.all_day {
                        WeekEvent::all_day(id, "New event", p.date)
                    } else {
                        WeekEvent::timed(id, "New event", p.date, p.start, p.end)
                    };
                    last.set(format!("Added: {}", placement_text(p)));
                    events.write().push(event);
                },
                on_change: move |e: WeekEvent| {
                    last.set(format!("Moved: {}", event_label(&e)));
                    apply_change(&mut events.write(), &e);
                },
                on_select: move |id: String| {
                    let picked = events.peek().iter().find(|e| e.id == id).map(event_label);
                    if let Some(text) = picked {
                        last.set(format!("Selected: {text}"));
                    }
                },
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "{last}" }
        }
    }
}
