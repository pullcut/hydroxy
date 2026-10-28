//! Week scheduler connector. Anatomy: root, toolbar, nav, title, views, header,
//! zone, day, allday, allday-cell, allday-event, scroller, body, gutter, hour,
//! days, column, event, event-title, event-time, handle, ghost, now, slot,
//! instructions, live. Every part carries `data-scope="week-scheduler"` and its
//! `data-part`; the root carries `data-view`.
//!
//! Events are focusable `role="button"`s laid out in one layer over the day
//! columns, so an event keeps its element (and focus) when it moves to another
//! day. The slot is one focusable `role="button"` that walks the grid with the
//! arrow keys; Enter creates an event there.

pub use oxide_machines::date::Date;
pub use oxide_machines::week_scheduler::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "week-scheduler";

/// Read by screen readers on every event, through `aria-describedby`.
pub const EVENT_HELP: &str = "Arrow Up and Down move the event by the snap step, Left and Right by a day. Shift with Up or Down changes when it ends. Enter selects it.";

/// Read by screen readers on the slot, through `aria-describedby`.
pub const SLOT_HELP: &str = "Arrow keys move the slot, Up and Down by the snap step, Left and Right by a day. Shift with Up or Down changes its length. Page Up and Page Down move an hour. Enter creates an event.";

/// An event's DOM id.
#[must_use]
pub fn event_id(root: &str, event: &str) -> String {
    format!("{root}-event-{event}")
}

/// The scroller's DOM id.
#[must_use]
pub fn scroller_id(root: &str) -> String {
    format!("{root}-scroller")
}

/// The slot's DOM id.
#[must_use]
pub fn slot_id(root: &str) -> String {
    format!("{root}-slot")
}

/// The event instructions' DOM id.
#[must_use]
pub fn event_help_id(root: &str) -> String {
    format!("{root}-event-help")
}

/// The slot instructions' DOM id.
#[must_use]
pub fn slot_help_id(root: &str) -> String {
    format!("{root}-slot-help")
}

/// Custom properties the stylesheet reads: the hour height and the day count.
#[must_use]
pub fn root_style(days: usize) -> String {
    format!("--ws-hour: {HOUR_HEIGHT}px; --ws-days: {days};")
}

/// The `scrollTop` that shows `hour` at the top of the grid.
#[must_use]
pub fn scroll_top_for(hour: u8) -> f64 {
    f64::from(hour.min(23)) * HOUR_HEIGHT
}

/// The `scrollTop` that keeps the slot `[start, end)` in a viewport `height` px tall
/// that is scrolled to `scroll`, or `None` when it is already in view.
#[must_use]
pub fn reveal_scroll(start: u16, end: u16, scroll: f64, height: f64) -> Option<f64> {
    let (top, bottom) = (minute_y(start), minute_y(end));
    if top < scroll {
        Some(top)
    } else if bottom > scroll + height {
        Some(bottom - height)
    } else {
        None
    }
}

/// Map a key on an event to a machine event. `Some` means: prevent the default.
#[must_use]
pub fn event_keydown(key: &str, shift: bool, id: &str, snap: u16, dragging: bool) -> Option<WeekSchedulerEvent> {
    let step = i32::from(snap.clamp(1, 60));
    let nudge = |days, minutes| WeekSchedulerEvent::Nudge { id: id.to_string(), days, minutes };
    Some(match key {
        "ArrowUp" if shift => WeekSchedulerEvent::Stretch { id: id.to_string(), minutes: -step },
        "ArrowDown" if shift => WeekSchedulerEvent::Stretch { id: id.to_string(), minutes: step },
        "ArrowUp" => nudge(0, -step),
        "ArrowDown" => nudge(0, step),
        "ArrowLeft" => nudge(-1, 0),
        "ArrowRight" => nudge(1, 0),
        "Enter" | " " => WeekSchedulerEvent::Select(id.to_string()),
        "Escape" if dragging => WeekSchedulerEvent::Cancel,
        _ => return None,
    })
}

/// Map a key on the slot to a machine event. `Some` means: prevent the default.
#[must_use]
pub fn slot_keydown(key: &str, shift: bool, snap: u16) -> Option<WeekSchedulerEvent> {
    let step = i32::from(snap.clamp(1, 60));
    let mv = |days, minutes| WeekSchedulerEvent::CursorMove { days, minutes };
    Some(match key {
        "ArrowUp" if shift => WeekSchedulerEvent::CursorStretch(-step),
        "ArrowDown" if shift => WeekSchedulerEvent::CursorStretch(step),
        "ArrowUp" => mv(0, -step),
        "ArrowDown" => mv(0, step),
        "ArrowLeft" => mv(-1, 0),
        "ArrowRight" => mv(1, 0),
        "PageUp" => mv(0, -60),
        "PageDown" => mv(0, 60),
        "Enter" | " " => WeekSchedulerEvent::CursorCreate,
        _ => return None,
    })
}

/// The previous or next button's accessible name: `"Previous week"`.
#[must_use]
pub fn nav_label(forward: bool, view: WeekView) -> String {
    format!("{} {}", if forward { "Next" } else { "Previous" }, view.unit())
}

/// A day column heading's parts: weekday and day of the month, `("Tue", 3)`.
#[must_use]
pub fn day_heading(d: Date) -> (&'static str, u8) {
    (weekday_short(d), d.day())
}

/// Hour labels down the gutter, `01:00` to `23:00` (midnight is the top edge).
#[must_use]
pub fn hour_labels() -> Vec<(u16, String)> {
    (1..24).map(|h| (h * 60, clock(h * 60))).collect()
}

/// The slot's accessible name: `"Tuesday 3 November, 09:00 to 10:00, empty slot"`.
#[must_use]
pub fn slot_label(cursor: Placement) -> String {
    format!("{}, empty slot", placement_text(cursor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_map_to_events() {
        assert_eq!(
            event_keydown("ArrowDown", false, "a", 15, false),
            Some(WeekSchedulerEvent::Nudge { id: "a".into(), days: 0, minutes: 15 })
        );
        assert_eq!(
            event_keydown("ArrowUp", true, "a", 15, false),
            Some(WeekSchedulerEvent::Stretch { id: "a".into(), minutes: -15 })
        );
        assert_eq!(event_keydown("Escape", false, "a", 15, false), None);
        assert_eq!(event_keydown("Escape", false, "a", 15, true), Some(WeekSchedulerEvent::Cancel));
        assert_eq!(slot_keydown("ArrowLeft", false, 15), Some(WeekSchedulerEvent::CursorMove { days: -1, minutes: 0 }));
        assert_eq!(slot_keydown("ArrowDown", true, 30), Some(WeekSchedulerEvent::CursorStretch(30)));
        assert_eq!(slot_keydown("Enter", false, 15), Some(WeekSchedulerEvent::CursorCreate));
        assert_eq!(slot_keydown("x", false, 15), None);
    }

    #[test]
    fn scrolling_and_labels() {
        assert!((scroll_top_for(8) - 384.0).abs() < 1e-9);
        assert_eq!(reveal_scroll(600, 660, 0.0, 400.0), Some(528.0 - 400.0));
        assert_eq!(reveal_scroll(600, 660, 400.0, 400.0), None);
        assert_eq!(reveal_scroll(60, 120, 400.0, 400.0), Some(48.0));
        assert_eq!(nav_label(true, WeekView::Day), "Next day");
        assert_eq!(nav_label(false, WeekView::WorkWeek), "Previous week");
        assert_eq!(hour_labels().len(), 23);
        assert_eq!(day_heading(Date::new(2026, 11, 3).unwrap()), ("Tue", 3));
    }
}
