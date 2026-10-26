//! Date range picker connector. Anatomy: root, label, field, trigger, content
//! (a popover holding two month grids), then per month: months, month, header,
//! previous, next, label, grid, weekday, week, cell, day. Every part carries
//! `data-scope="date-range-picker"` and its `data-part`.
//!
//! Like the date picker, the content is a native `popover="auto"` anchored to
//! the field. Each month is a `<table role="grid">` of 42 `<button>`s; days from
//! the neighbouring months are blank, so every date has exactly one button and
//! one id. Only the focused day is in the tab order. Labels are English.

pub use oxide_machines::date_range_picker::*;

use crate::calendar::{self, CalendarEvent, CalendarInput};
use crate::overlay::{self, Align, Placement, Side};
use oxide_machines::date::Date;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "date-range-picker";

/// Below the field, lined up with its start edge.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

/// Pixels between field and popover.
pub const GAP_PX: u16 = 6;

/// What opening focuses inside the content: the focused day.
pub const FOCUS_ON_OPEN: &str = "[data-part=\"day\"][tabindex=\"0\"]";

/// Text on the trigger while empty.
pub const PLACEHOLDER: &str = "Select dates";

/// Ids for the parts the adapter needs to find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The trigger button, named by the label.
    pub trigger: String,
    /// The popover.
    pub content: String,
}

/// Ids from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { trigger: format!("{root}-trigger"), content: format!("{root}-content") }
}

/// A day button's id (one per date, so focus commands find it).
#[must_use]
pub fn day_id(root: &str, date: Date) -> String {
    format!("{root}-{date}")
}

/// A month label's id, which names its grid.
#[must_use]
pub fn label_id(root: &str, index: usize) -> String {
    format!("{root}-label-{index}")
}

/// `style` for the field: names the CSS anchor.
#[must_use]
pub fn field_style(root: &str) -> String {
    overlay::anchor_style(root)
}

/// `style` for the content: anchored below the field, flipping when short of room.
#[must_use]
pub fn content_style(root: &str) -> String {
    overlay::floating_style(root, PLACEMENT, GAP_PX)
}

/// `data-state` for the trigger and content.
#[must_use]
pub fn data_state(state: &DateRangePickerState) -> &'static str {
    if state.open { "open" } else { "closed" }
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn short(date: Date, with_year: bool) -> String {
    let month = MONTHS[usize::from(date.month() - 1)];
    if with_year { format!("{} {month} {}", date.day(), date.year()) } else { format!("{} {month}", date.day()) }
}

/// "26 Oct – 2 Nov 2026", or with both years when they differ; `None` when empty.
#[must_use]
pub fn range_text(range: Option<DateRange>) -> Option<String> {
    let r = range?;
    let same_year = r.start.year() == r.end.year();
    Some(format!("{} – {}", short(r.start, !same_year), short(r.end, true)))
}

/// One cell of a month grid.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools, reason = "one flag per ARIA/data attribute")]
pub struct DayAttrs {
    /// The date.
    pub date: Date,
    /// `id`, absent for blank cells.
    pub id: Option<String>,
    /// `aria-label`.
    pub label: String,
    /// Belongs to a neighbouring month: blank, `disabled`, `aria-hidden`.
    pub outside: bool,
    /// The first day of the range on show (`data-range-start`).
    pub start: bool,
    /// The last day of the range on show (`data-range-end`).
    pub end: bool,
    /// Between the ends (`data-in-range`).
    pub between: bool,
    /// `aria-selected` on the cell: inside the range on show.
    pub selected: bool,
    /// The range on show is still being picked (`data-preview`).
    pub preview: bool,
    /// `aria-current="date"`, `data-today`.
    pub today: bool,
    /// `aria-disabled`, `data-unavailable`: focusable but not selectable.
    pub unavailable: bool,
    /// `disabled`: blank, or outside `min`/`max`, or the picker is disabled.
    pub disabled: bool,
    /// `tabindex`: 0 on the focused day, -1 elsewhere.
    pub tabindex: i8,
}

/// One month of the pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthAttrs {
    /// "October 2026".
    pub label: String,
    /// `id` of the label.
    pub label_id: String,
    /// The 42 cells, row by row.
    pub days: Vec<DayAttrs>,
}

/// Both months on show.
#[must_use]
pub fn months(root: &str, input: &DateRangePickerInput, state: &DateRangePickerState) -> [MonthAttrs; 2] {
    let rules = &input.rules;
    let preview = state.preview();
    let in_progress = state.anchor.is_some();
    std::array::from_fn(|index| {
        let first = state.month.add_months(i32::try_from(index).unwrap_or(0));
        let days = first
            .month_grid(rules.week_start)
            .into_iter()
            .map(|date| {
                let outside = date.month() != first.month();
                let disabled = outside || rules.disabled || !rules.in_range(date);
                let shown = preview.filter(|_| !outside);
                DayAttrs {
                    date,
                    id: (!outside).then(|| day_id(root, date)),
                    label: calendar::day_label(date),
                    outside,
                    start: shown.is_some_and(|r| r.start == date),
                    end: shown.is_some_and(|r| r.end == date),
                    between: shown.is_some_and(|r| r.contains(date) && r.start != date && r.end != date),
                    selected: shown.is_some_and(|r| r.contains(date)),
                    preview: in_progress,
                    today: !outside && date == rules.today,
                    unavailable: !disabled && !rules.is_selectable(date),
                    disabled,
                    tabindex: if !outside && date == state.focused { 0 } else { -1 },
                }
            })
            .collect();
        MonthAttrs { label: calendar::month_label(first), label_id: label_id(root, index), days }
    })
}

/// Whether the previous (-1) or next (+1) month button works.
#[must_use]
pub fn can_shift(input: &DateRangePickerInput, state: &DateRangePickerState, direction: i32) -> bool {
    let rules: &CalendarInput = &input.rules;
    let from = if direction < 0 { state.month } else { state.month.add_months(1) };
    !rules.disabled && rules.can_shift(from, direction)
}

/// Keys on a focused day, shared with the calendar. Enter and Space click.
#[must_use]
pub fn day_keydown(key: &str, shift: bool) -> Option<DateRangePickerEvent> {
    match calendar::day_keydown(key, shift)? {
        CalendarEvent::Move(by) => Some(DateRangePickerEvent::Move(by)),
        _ => None,
    }
}

/// Keys inside the open popover. Escape is handled here so focus goes back to the trigger.
#[must_use]
pub fn content_keydown(key: &str) -> Option<DateRangePickerEvent> {
    (key == "Escape").then_some(DateRangePickerEvent::Close { restore_focus: true })
}

/// The hidden inputs' values: start and end as `YYYY-MM-DD`, empty while there is no range.
#[must_use]
pub fn form_values(state: &DateRangePickerState) -> [String; 2] {
    state.value.map_or([String::new(), String::new()], |r| [r.start.to_string(), r.end.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_machines::calendar::CalendarMove;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    #[test]
    fn range_reads_naturally() {
        assert_eq!(range_text(None), None);
        assert_eq!(range_text(Some(DateRange::new(d("2026-10-26"), d("2026-11-02")))).unwrap(), "26 Oct – 2 Nov 2026");
        assert_eq!(range_text(Some(DateRange::new(d("2026-12-30"), d("2027-01-02")))).unwrap(), "30 Dec 2026 – 2 Jan 2027");
    }

    #[test]
    fn two_months_mark_the_range() {
        let input = DateRangePickerInput {
            rules: CalendarInput { today: d("2026-10-26"), week_start: 1, ..Default::default() },
            ..Default::default()
        };
        let state = DateRangePickerState {
            value: Some(DateRange::new(d("2026-10-30"), d("2026-11-03"))),
            focused: d("2026-10-30"),
            month: d("2026-10-01"),
            ..Default::default()
        };
        let [oct, nov] = months("r", &input, &state);
        assert_eq!((oct.label.as_str(), nov.label.as_str()), ("October 2026", "November 2026"));
        let at = |m: &MonthAttrs, s: &str| m.days.iter().find(|c| c.date == d(s) && !c.outside).unwrap().clone();
        assert!(at(&oct, "2026-10-30").start && at(&oct, "2026-10-30").tabindex == 0);
        assert!(at(&oct, "2026-10-31").between && at(&nov, "2026-11-02").between);
        assert!(at(&nov, "2026-11-03").end && at(&nov, "2026-11-03").selected);
        assert!(!at(&nov, "2026-11-04").selected);
        assert!(at(&oct, "2026-10-26").today);
        // One button per date across both grids.
        let ids: Vec<_> = oct.days.iter().chain(&nov.days).filter_map(|c| c.id.clone()).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(ids.len(), unique.len());
        // Blank cells are inert, and exactly one cell is in the tab order.
        assert!(oct.days.iter().filter(|c| c.outside).all(|c| c.disabled && c.id.is_none()));
        assert_eq!(oct.days.iter().chain(&nov.days).filter(|c| c.tabindex == 0).count(), 1);
    }

    #[test]
    fn in_progress_range_follows_the_hover() {
        let input = DateRangePickerInput::default();
        let state = DateRangePickerState {
            anchor: Some(d("2026-10-10")),
            hover: Some(d("2026-10-08")),
            focused: d("2026-10-08"),
            month: d("2026-10-01"),
            ..Default::default()
        };
        let [oct, _] = months("r", &input, &state);
        let cell = |s: &str| oct.days.iter().find(|c| c.date == d(s) && !c.outside).unwrap();
        assert!(cell("2026-10-08").start && cell("2026-10-10").end && cell("2026-10-09").between);
        assert!(cell("2026-10-09").preview);
    }

    #[test]
    fn shift_buttons_respect_bounds_and_keys_map() {
        let mut input = DateRangePickerInput::default();
        input.rules.max = Some(d("2026-11-30"));
        let state = DateRangePickerState { month: d("2026-10-01"), ..Default::default() };
        assert!(can_shift(&input, &state, -1));
        assert!(!can_shift(&input, &state, 1));
        assert_eq!(day_keydown("PageDown", false), Some(DateRangePickerEvent::Move(CalendarMove::Months(1))));
        assert_eq!(day_keydown("Enter", false), None);
        assert_eq!(content_keydown("Escape"), Some(DateRangePickerEvent::Close { restore_focus: true }));
        assert_eq!(form_values(&DateRangePickerState::default()), [String::new(), String::new()]);
    }
}
