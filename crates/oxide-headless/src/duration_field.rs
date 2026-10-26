//! Duration field connector. Anatomy: root, label, field (the group), segment
//! (hours, minutes, seconds: a `role="spinbutton"` text input), separator, hidden
//! input. Every part carries `data-scope="duration-field"` and its `data-part`.
//! Labels are English.

pub use oxide_machines::duration_field::*;
use oxide_machines::segments::{self, SegmentKey};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "duration-field";

/// The text between segments.
pub const SEPARATOR: &str = ":";

/// A segment's id, from the root id.
#[must_use]
pub fn segment_id(root: &str, segment: DurationSegment) -> String {
    let name = match segment {
        DurationSegment::Hours => "hours",
        DurationSegment::Minutes => "minutes",
        DurationSegment::Seconds => "seconds",
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
    /// `aria-label`: "Hours", "Minutes" or "Seconds".
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
}

/// Attributes for `segment`.
#[must_use]
pub fn segment_attrs(root: &str, state: &DurationFieldState, segment: DurationSegment) -> SegmentAttrs {
    let spec = segment.spec();
    let value = state.get(segment);
    SegmentAttrs {
        id: segment_id(root, segment),
        label: match segment {
            DurationSegment::Hours => "Hours",
            DurationSegment::Minutes => "Minutes",
            DurationSegment::Seconds => "Seconds",
        },
        text: state.text(segment),
        valuenow: value,
        valuemin: spec.min,
        valuemax: spec.max,
        valuetext: value.map_or_else(|| "Empty".into(), |_| state.text(segment)),
    }
}

/// The hidden input's value: total seconds, empty while the field is empty.
#[must_use]
pub fn form_value(state: &DurationFieldState) -> String {
    state.value().map(|s| s.to_string()).unwrap_or_default()
}

/// What a key in a segment asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Send this event to the machine.
    Send(DurationFieldEvent),
    /// Move focus to this segment.
    Focus(DurationSegment),
    /// Swallow the key without effect.
    Ignore,
}

/// Map a `KeyboardEvent.key` on `segment`. `Some` means prevent the default.
#[must_use]
pub fn keydown(key: &str, modified: bool, segment: DurationSegment, input: &DurationFieldInput) -> Option<Action> {
    let all = input.segments();
    let at = all.iter().position(|s| *s == segment)?;
    Some(match segments::key(key, modified)? {
        SegmentKey::Digit(digit) => Action::Send(DurationFieldEvent::Digit { segment, digit }),
        SegmentKey::Step { up } => Action::Send(DurationFieldEvent::Step { segment, up }),
        SegmentKey::Jump { to_max } => Action::Send(DurationFieldEvent::Jump { segment, to_max }),
        SegmentKey::Clear => Action::Send(DurationFieldEvent::Clear(segment)),
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
    use DurationSegment::{Hours, Minutes, Seconds};

    #[test]
    fn attrs_and_form_value() {
        let state = DurationFieldState { minutes: Some(5), ..Default::default() };
        let a = segment_attrs("d", &state, Minutes);
        assert_eq!((a.id.as_str(), a.text.as_str(), a.valuemax), ("d-minutes", "05", 59));
        assert_eq!(segment_attrs("d", &state, Hours).valuetext, "Empty");
        assert_eq!(form_value(&state), "300");
        assert_eq!(form_value(&DurationFieldState::default()), "");
    }

    #[test]
    fn arrows_stay_among_visible_segments() {
        let input = DurationFieldInput::default();
        assert_eq!(keydown("ArrowRight", false, Minutes, &input), Some(Action::Ignore));
        let shown = DurationFieldInput { show_seconds: true, ..input };
        assert_eq!(keydown("ArrowRight", false, Minutes, &shown), Some(Action::Focus(Seconds)));
        assert_eq!(keydown("5", false, Hours, &shown), Some(Action::Send(DurationFieldEvent::Digit { segment: Hours, digit: 5 })));
        assert_eq!(keydown("ArrowUp", false, Seconds, &shown), Some(Action::Send(DurationFieldEvent::Step { segment: Seconds, up: true })));
    }
}
