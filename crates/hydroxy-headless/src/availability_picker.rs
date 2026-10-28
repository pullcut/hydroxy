//! Availability picker connector. Anatomy: root, timezone, grid, corner, day,
//! time, cell, summary, live. Every part carries `data-scope="availability-picker"`
//! and its `data-part`.

pub use hydroxy_machines::availability_picker::*;
pub use hydroxy_machines::date::Date;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "availability-picker";

/// The id of the cell at a column and row.
#[must_use]
pub fn cell_id(root: &str, col: usize, row: usize) -> String {
    format!("{root}-{col}-{row}")
}

/// Whole-grid movement used for Home and End.
const FAR: i32 = 10_000;

/// Map a key on a cell to a machine event. `Some` means: prevent the default.
/// Shift with an arrow paints; Home and End jump along the row, Ctrl with them
/// to the first or last cell; `PageUp` and `PageDown` jump along the column.
#[must_use]
pub fn keydown(key: &str, shift: bool, ctrl: bool) -> Option<AvailabilityEvent> {
    let (dc, dr) = match key {
        "ArrowLeft" => (-1, 0),
        "ArrowRight" => (1, 0),
        "ArrowUp" => (0, -1),
        "ArrowDown" => (0, 1),
        "Home" => (-FAR, if ctrl { -FAR } else { 0 }),
        "End" => (FAR, if ctrl { FAR } else { 0 }),
        "PageUp" => (0, -FAR),
        "PageDown" => (0, FAR),
        " " | "Enter" => return Some(AvailabilityEvent::Toggle),
        "Escape" => return Some(AvailabilityEvent::Cancel),
        _ => return None,
    };
    Some(AvailabilityEvent::Move { dc, dr, paint: shift })
}

/// A time of day as `09:00`.
#[must_use]
pub fn time_label(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60 % 24, minute % 60)
}

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// A column heading such as `Mon 5`.
#[must_use]
pub fn day_label(date: Date) -> String {
    format!("{} {}", WEEKDAYS[usize::from(date.weekday()) % 7], date.day())
}

/// A cell's accessible name: `Mon 5, 09:00 to 09:30, available`.
#[must_use]
pub fn cell_label(slot: Slot, slot_minutes: u16, available: bool) -> String {
    format!(
        "{}, {} to {}, {}",
        day_label(slot.date),
        time_label(slot.minute),
        time_label(slot.minute + slot_minutes),
        if available { "available" } else { "unavailable" }
    )
}

/// One line per day for the summary: `Tue 6: 09:00 to 11:30, 14:00 to 15:00`.
#[must_use]
pub fn summary_lines(value: &std::collections::BTreeSet<Slot>, slot_minutes: u16) -> Vec<String> {
    let mut lines: Vec<(Date, Vec<String>)> = Vec::new();
    for (date, from, to) in ranges(value, slot_minutes) {
        let range = format!("{} to {}", time_label(from), time_label(to));
        match lines.last_mut() {
            Some((d, parts)) if *d == date => parts.push(range),
            _ => lines.push((date, vec![range])),
        }
    }
    lines.into_iter().map(|(d, parts)| format!("{}: {}", day_label(d), parts.join(", "))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_move_paint_and_toggle() {
        assert_eq!(keydown("ArrowDown", true, false), Some(AvailabilityEvent::Move { dc: 0, dr: 1, paint: true }));
        assert_eq!(keydown(" ", false, false), Some(AvailabilityEvent::Toggle));
        assert_eq!(keydown("x", false, false), None);
        assert_eq!(keydown("Home", false, true), Some(AvailabilityEvent::Move { dc: -FAR, dr: -FAR, paint: false }));
    }

    #[test]
    fn labels_and_summary() {
        let d = Date::new(2026, 1, 5).unwrap();
        assert_eq!(day_label(d), "Mon 5");
        let slot = Slot { date: d, minute: 540 };
        assert_eq!(cell_label(slot, 30, true), "Mon 5, 09:00 to 09:30, available");
        let value = [slot, Slot { date: d, minute: 570 }].into_iter().collect();
        assert_eq!(summary_lines(&value, 30), ["Mon 5: 09:00 to 10:00"]);
    }
}
