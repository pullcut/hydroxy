//! Date field connector. Anatomy: root, label, field (the group), segment (one
//! per day/month/year, a `role="spinbutton"` text input), separator, hidden
//! input. Every part carries `data-scope="date-field"` and its `data-part`.
//!
//! Segments are text inputs so they take focus, tab order and mobile keyboards
//! for free; the machine owns their text and the keys never insert characters.
//! Labels are English.

pub use oxide_machines::date_field::*;
use oxide_machines::segments::{self, SegmentKey};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "date-field";

/// The text between segments.
pub const SEPARATOR: &str = "/";

/// A segment's id, from the root id.
#[must_use]
pub fn segment_id(root: &str, segment: DateSegment) -> String {
    let name = match segment {
        DateSegment::Day => "day",
        DateSegment::Month => "month",
        DateSegment::Year => "year",
    };
    format!("{root}-{name}")
}

/// The label's id, which names the group.
#[must_use]
pub fn label_id(root: &str) -> String {
    format!("{root}-label")
}

/// Attributes for one segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentAttrs {
    /// `id`.
    pub id: String,
    /// `aria-label`: "Day", "Month" or "Year".
    pub label: &'static str,
    /// What the input shows: padded digits, or dashes while empty.
    pub text: String,
    /// `aria-valuenow`, absent while empty.
    pub valuenow: Option<u32>,
    /// `aria-valuemin`.
    pub valuemin: u32,
    /// `aria-valuemax`.
    pub valuemax: u32,
    /// `aria-valuetext`: the digits, or "Empty".
    pub valuetext: String,
    /// `size`: digits that fit.
    pub width: u8,
}

/// Attributes for `segment`.
#[must_use]
pub fn segment_attrs(root: &str, state: &DateFieldState, segment: DateSegment) -> SegmentAttrs {
    let spec = state.spec(segment);
    let value = state.get(segment);
    SegmentAttrs {
        id: segment_id(root, segment),
        label: match segment {
            DateSegment::Day => "Day",
            DateSegment::Month => "Month",
            DateSegment::Year => "Year",
        },
        text: state.text(segment),
        valuenow: value,
        valuemin: spec.min,
        valuemax: spec.max,
        valuetext: value.map_or_else(|| "Empty".into(), |_| state.text(segment)),
        width: spec.width,
    }
}

/// The hidden input's value: `YYYY-MM-DD`, empty while there is no allowed date.
#[must_use]
pub fn form_value(state: &DateFieldState, input: &DateFieldInput) -> String {
    state.value(input).map(|d| d.to_string()).unwrap_or_default()
}

/// What a key in a segment asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Send this event to the machine.
    Send(DateFieldEvent),
    /// Move focus to this segment.
    Focus(DateSegment),
    /// Swallow the key without effect.
    Ignore,
}

/// Map a `KeyboardEvent.key` on `segment`. `Some` means prevent the default.
#[must_use]
pub fn keydown(key: &str, modified: bool, segment: DateSegment, order: DateOrder) -> Option<Action> {
    let all = order.segments();
    let at = all.iter().position(|s| *s == segment)?;
    Some(match segments::key(key, modified)? {
        SegmentKey::Digit(digit) => Action::Send(DateFieldEvent::Digit { segment, digit }),
        SegmentKey::Step { up } => Action::Send(DateFieldEvent::Step { segment, up }),
        SegmentKey::Jump { to_max } => Action::Send(DateFieldEvent::Jump { segment, to_max }),
        SegmentKey::Clear => Action::Send(DateFieldEvent::Clear(segment)),
        SegmentKey::Prev => at.checked_sub(1).map_or(Action::Ignore, |i| Action::Focus(all[i])),
        SegmentKey::Next => all.get(at + 1).map_or(Action::Ignore, |s| Action::Focus(*s)),
        SegmentKey::Ignore => Action::Ignore,
    })
}

/// A digit typed through an input event (soft keyboards that report no key).
#[must_use]
pub fn input_digit(data: &str) -> Option<u32> {
    data.chars().rev().find_map(|c| c.to_digit(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs_describe_a_segment() {
        let state = DateFieldState { day: Some(5), ..Default::default() };
        let a = segment_attrs("f", &state, DateSegment::Day);
        assert_eq!((a.id.as_str(), a.text.as_str(), a.valuenow, a.valuemax), ("f-day", "05", Some(5), 31));
        let a = segment_attrs("f", &state, DateSegment::Year);
        assert_eq!((a.text.as_str(), a.valuetext.as_str(), a.valuenow), ("––––", "Empty", None));
    }

    #[test]
    fn keys_map_to_actions() {
        use DateSegment::{Day, Month, Year};
        let dmy = DateOrder::Dmy;
        assert_eq!(keydown("3", false, Day, dmy), Some(Action::Send(DateFieldEvent::Digit { segment: Day, digit: 3 })));
        assert_eq!(keydown("ArrowRight", false, Day, dmy), Some(Action::Focus(Month)));
        assert_eq!(keydown("ArrowLeft", false, Day, dmy), Some(Action::Ignore));
        assert_eq!(keydown("ArrowLeft", false, Year, DateOrder::Ymd), Some(Action::Ignore));
        assert_eq!(keydown("ArrowLeft", false, Year, dmy), Some(Action::Focus(Month)));
        assert_eq!(keydown("x", false, Day, dmy), Some(Action::Ignore));
        assert_eq!(keydown("Tab", false, Day, dmy), None);
        assert_eq!(input_digit("4"), Some(4));
        assert_eq!(input_digit("x"), None);
    }

    #[test]
    fn form_value_is_iso() {
        let input = DateFieldInput::default();
        let state = DateFieldState { day: Some(4), month: Some(3), year: Some(2026), ..Default::default() };
        assert_eq!(form_value(&state, &input), "2026-03-04");
        assert_eq!(form_value(&DateFieldState::default(), &input), "");
    }
}
