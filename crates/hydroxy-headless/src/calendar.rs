//! Calendar connector. Anatomy: root, header, previous, label, next, grid,
//! weekday, week, cell, day. Every part carries `data-scope="calendar"` and its
//! `data-part`.
//!
//! The grid is a native `<table role="grid">` of 42 `<button>`s, so click, Enter
//! and Space come for free; only the focused day is in the tab order. Cells keep
//! their elements across months (only attributes change), so focus survives.
//! Unavailable days stay focusable (`aria-disabled`); days outside the month or
//! `min`/`max` are `disabled`. Labels are English.

pub use hydroxy_machines::calendar::*;
pub use hydroxy_machines::date::{Date, ParseDateError};

/// The `data-scope` value on every calendar part.
pub const SCOPE: &str = "calendar";

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];
const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

/// "October 2026": the visible month, for the live label.
#[must_use]
pub fn month_label(date: Date) -> String {
    format!("{} {}", MONTHS[usize::from(date.month() - 1)], date.year())
}

/// "Monday 26 October 2026": a day's accessible name.
#[must_use]
pub fn day_label(date: Date) -> String {
    format!(
        "{} {} {} {}",
        WEEKDAYS[usize::from(date.weekday())],
        date.day(),
        MONTHS[usize::from(date.month() - 1)],
        date.year()
    )
}

/// Column headings from `week_start`: (short "Mo", full "Monday") pairs.
#[must_use]
pub fn weekdays(week_start: u8) -> [(&'static str, &'static str); 7] {
    std::array::from_fn(|i| {
        let name = WEEKDAYS[(usize::from(week_start) + i) % 7];
        (&name[..2], name)
    })
}

/// The month label's id, which names the grid.
#[must_use]
pub fn label_id(root: &str) -> String {
    format!("{root}-label")
}

/// A day button's id.
#[must_use]
pub fn day_id(root: &str, date: Date) -> String {
    format!("{root}-{date}")
}

/// Today's date in the browser's time zone (UTC off the web).
#[must_use]
pub fn today() -> Date {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        let now = js_sys::Date::new_0();
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, reason = "calendar fields are small")]
        if let Some(date) = Date::new(now.get_full_year() as i32, now.get_month() as u8 + 1, now.get_date() as u8) {
            return date;
        }
    }
    Date::from_days(crate::due::today())
}

/// One cell of the six-week grid.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools, reason = "one flag per ARIA/data attribute")]
pub struct DayAttrs {
    /// The date.
    pub date: Date,
    /// `id` of the day button.
    pub id: String,
    /// `aria-label`.
    pub label: String,
    /// Outside the visible month: muted, `disabled` and `aria-hidden`.
    pub outside: bool,
    /// `aria-selected` on the cell, `data-selected` on the button.
    pub selected: bool,
    /// `aria-current="date"`, `data-today`.
    pub today: bool,
    /// `aria-disabled`, `data-unavailable`: focusable but not selectable.
    pub unavailable: bool,
    /// `disabled`: outside the month or `min`/`max`, or the whole calendar is disabled.
    pub disabled: bool,
    /// `tabindex`: 0 on the focused day, -1 elsewhere.
    pub tabindex: i8,
}

/// The 42 cells of the visible month, row by row.
#[must_use]
pub fn days(root: &str, input: &CalendarInput, state: &CalendarState) -> Vec<DayAttrs> {
    state
        .focused
        .month_grid(input.week_start)
        .into_iter()
        .map(|date| {
            let outside = date.month() != state.focused.month();
            let disabled = outside || input.disabled || !input.in_range(date);
            DayAttrs {
                date,
                id: day_id(root, date),
                label: day_label(date),
                outside,
                selected: state.value == Some(date),
                today: date == input.today,
                unavailable: !disabled && !input.is_selectable(date),
                disabled,
                tabindex: if date == state.focused { 0 } else { -1 },
            }
        })
        .collect()
}

/// Whether the previous (-1) or next (+1) month button works.
#[must_use]
pub fn can_shift(input: &CalendarInput, state: &CalendarState, months: i32) -> bool {
    !input.disabled && input.can_shift(state.focused, months)
}

/// Keys on a focused day. Enter and Space are left to the native button click.
#[must_use]
pub fn day_keydown(key: &str, shift: bool) -> Option<CalendarEvent> {
    let by = match key {
        "ArrowLeft" => CalendarMove::Days(-1),
        "ArrowRight" => CalendarMove::Days(1),
        "ArrowUp" => CalendarMove::Days(-7),
        "ArrowDown" => CalendarMove::Days(7),
        "PageUp" if shift => CalendarMove::Years(-1),
        "PageDown" if shift => CalendarMove::Years(1),
        "PageUp" => CalendarMove::Months(-1),
        "PageDown" => CalendarMove::Months(1),
        "Home" => CalendarMove::WeekStart,
        "End" => CalendarMove::WeekEnd,
        _ => return None,
    };
    Some(CalendarEvent::Move(by))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    #[test]
    fn labels() {
        assert_eq!(month_label(d("2026-10-26")), "October 2026");
        assert_eq!(day_label(d("2026-10-26")), "Monday 26 October 2026");
        assert_eq!(weekdays(1)[0], ("Mo", "Monday"));
        assert_eq!(weekdays(1)[6], ("Su", "Sunday"));
    }

    #[test]
    fn day_cells_mark_state() {
        let input = CalendarInput {
            today: d("2026-10-26"),
            week_start: 1,
            unavailable: vec![d("2026-10-03")],
            max: Some(d("2026-10-30")),
            ..Default::default()
        };
        let state = CalendarState { value: Some(d("2026-10-02")), focused: d("2026-10-02"), controlled: false };
        let cells = days("c", &input, &state);
        assert_eq!(cells.len(), 42);
        let at = |s: &str| cells.iter().find(|c| c.date == d(s)).unwrap();
        assert!(at("2026-09-28").outside);
        assert!(at("2026-10-02").selected && at("2026-10-02").tabindex == 0);
        assert_eq!(cells.iter().filter(|c| c.tabindex == 0).count(), 1);
        assert!(at("2026-10-26").today);
        assert!(at("2026-10-03").unavailable && !at("2026-10-03").disabled);
        assert!(at("2026-10-31").disabled);
        assert_eq!(at("2026-10-02").id, "c-2026-10-02");
        assert!(!can_shift(&input, &state, 1));
        assert!(can_shift(&input, &state, -1));
    }

    #[test]
    fn keys() {
        assert_eq!(day_keydown("ArrowUp", false), Some(CalendarEvent::Move(CalendarMove::Days(-7))));
        assert_eq!(day_keydown("PageDown", true), Some(CalendarEvent::Move(CalendarMove::Years(1))));
        assert_eq!(day_keydown("PageDown", false), Some(CalendarEvent::Move(CalendarMove::Months(1))));
        assert_eq!(day_keydown("Enter", false), None);
    }
}
