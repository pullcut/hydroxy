//! Cron builder connector. Anatomy: root, controls, control, control-label, frequency,
//! interval, time, minute, weekdays, weekday, month-day, expression-field, expression,
//! error, description, runs-label, runs, run. Every part carries `data-scope="cron-builder"`
//! and its `data-part`.
//!
//! Controls are native `<select>` and `<input type="time">` elements. Weekdays are a
//! `group` of `<button aria-pressed>` toggles. The expression is a text `<input>`
//! described by the error and the description, which sits in a polite live region so the
//! schedule is read back as it changes. Upcoming runs are an `<ol>` of `<time>`.

pub use oxide_machines::cron_builder::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "cron-builder";

/// How many upcoming runs to list.
pub const RUN_COUNT: usize = 5;

/// Element ids for one builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The preset `<select>`.
    pub frequency: String,
    /// The interval `<select>`.
    pub interval: String,
    /// The time input or minute select.
    pub time: String,
    /// The weekday group's label.
    pub weekdays_label: String,
    /// The month-day `<select>`.
    pub month_day: String,
    /// The expression `<input>`.
    pub expression: String,
    /// The error.
    pub error: String,
    /// The description.
    pub description: String,
    /// The runs list's heading.
    pub runs_label: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids {
        frequency: format!("{root}-frequency"),
        interval: format!("{root}-interval"),
        time: format!("{root}-time"),
        weekdays_label: format!("{root}-weekdays-label"),
        month_day: format!("{root}-month-day"),
        expression: format!("{root}-expression"),
        error: format!("{root}-error"),
        description: format!("{root}-description"),
        runs_label: format!("{root}-runs-label"),
    }
}

/// Weekday toggles in display order, Monday first: (cron index, short label, full name).
pub const WEEKDAY_TOGGLES: [(u8, &str, &str); 7] = [
    (1, "Mon", "Monday"),
    (2, "Tue", "Tuesday"),
    (3, "Wed", "Wednesday"),
    (4, "Thu", "Thursday"),
    (5, "Fri", "Friday"),
    (6, "Sat", "Saturday"),
    (0, "Sun", "Sunday"),
];

/// Whether weekday `d` is on in the controls.
#[must_use]
pub const fn weekday_on(controls: &Controls, d: u8) -> bool {
    controls.weekdays & (1 << d) != 0
}

/// The time input's value, "09:30".
#[must_use]
pub fn time_value(controls: &Controls) -> String {
    format!("{:02}:{:02}", controls.hour, controls.minute)
}

/// Option text for a minute interval: "Every 15 minutes", "Every minute".
#[must_use]
pub fn minutes_label(n: u32) -> String {
    if n == 1 { "Every minute".into() } else { format!("Every {n} minutes") }
}

/// Option text for an hour interval.
#[must_use]
pub fn hours_label(n: u32) -> String {
    if n == 1 { "Every hour".into() } else { format!("Every {n} hours") }
}

/// Option text for a day of the month: "1st", "31st (skips shorter months)".
#[must_use]
pub fn month_day_label(d: u32) -> String {
    let suffix = match (d % 10, d % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    if d > 28 { format!("{d}{suffix} (skips shorter months)") } else { format!("{d}{suffix}") }
}

/// What the description region says.
#[must_use]
pub fn description(state: &CronBuilderState) -> String {
    match state.expr.parse::<Cron>() {
        Ok(c) => c.describe(),
        Err(_) => "Not a valid schedule".into(),
    }
}

/// The `aria-describedby` for the expression field.
#[must_use]
pub fn expression_described_by(ids: &Ids, has_error: bool) -> String {
    if has_error { format!("{} {}", ids.error, ids.description) } else { ids.description.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_read_naturally() {
        assert_eq!(minutes_label(1), "Every minute");
        assert_eq!(hours_label(6), "Every 6 hours");
        assert_eq!(month_day_label(2), "2nd");
        assert_eq!(month_day_label(12), "12th");
        assert_eq!(month_day_label(31), "31st (skips shorter months)");
        assert_eq!(time_value(&Controls::default()), "09:00");
    }

    #[test]
    fn description_follows_expression() {
        let s = CronBuilderState { expr: "0 9 * * 1-5".into(), ..Default::default() };
        assert_eq!(description(&s), "At 09:00 on weekdays");
        let s = CronBuilderState { expr: "nope".into(), ..Default::default() };
        assert_eq!(description(&s), "Not a valid schedule");
        assert_eq!(expression_described_by(&ids("c"), true), "c-error c-description");
    }
}
