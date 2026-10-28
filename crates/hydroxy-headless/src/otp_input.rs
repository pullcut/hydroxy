//! One-time-code connector. Anatomy: root (`role="group"`), label, cells (one
//! native text input per character), hidden input. Every part carries
//! `data-scope="otp-input"` and its `data-part`; cells add `data-state`
//! (`filled` or `empty`) and `data-index`.
//!
//! Only the first cell has `autocomplete="one-time-code"`, so SMS and password
//! manager autofill land there and the machine spreads the code across cells.

pub use hydroxy_machines::otp_input::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "otp-input";

/// Element ids for one code input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The visible label, which names the group.
    pub label: String,
    root: String,
}

impl Ids {
    /// The id of cell `index`.
    #[must_use]
    pub fn cell(&self, index: usize) -> String {
        format!("{}-cell-{index}", self.root)
    }
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { label: format!("{root}-label"), root: root.to_string() }
}

/// Attributes for cell `index`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellAttrs {
    /// `"one-time-code"` on the first cell, `"off"` elsewhere.
    pub autocomplete: &'static str,
    /// `"numeric"` or `"text"`, for the on-screen keyboard.
    pub inputmode: &'static str,
    /// Such as "Digit 2 of 6".
    pub aria_label: String,
    /// `"filled"` or `"empty"`.
    pub data_state: &'static str,
    /// The cell's character, or empty.
    pub value: String,
    /// Roving tabindex: only the focus cell is tabbable, so Tab leaves the group.
    pub tabindex: i32,
}

/// Attributes for cell `index`.
#[must_use]
pub fn cell_attrs(state: &OtpInputState, input: &OtpInputInput, index: usize) -> CellAttrs {
    let ch = state.char_at(index);
    let noun = match input.mode {
        OtpMode::Numeric => "Digit",
        OtpMode::Alphanumeric => "Character",
    };
    CellAttrs {
        autocomplete: if index == 0 { "one-time-code" } else { "off" },
        inputmode: match input.mode {
            OtpMode::Numeric => "numeric",
            OtpMode::Alphanumeric => "text",
        },
        aria_label: format!("{noun} {} of {}", index + 1, input.length.max(1)),
        data_state: if ch.is_some() { "filled" } else { "empty" },
        value: ch.map(String::from).unwrap_or_default(),
        tabindex: if index == state.focus { 0 } else { -1 },
    }
}

/// `data-state` for the root: `"complete"` once every cell is filled.
#[must_use]
pub fn root_state(state: &OtpInputState, input: &OtpInputInput) -> &'static str {
    if state.value.len() >= input.length.max(1) { "complete" } else { "incomplete" }
}

/// Map a `KeyboardEvent.key` on cell `index` to a machine event.
/// `Some` means: prevent the default.
#[must_use]
pub fn keydown(key: &str, index: usize) -> Option<OtpInputEvent> {
    match key {
        "Backspace" => Some(OtpInputEvent::Backspace(index)),
        "Delete" => Some(OtpInputEvent::Delete(index)),
        "ArrowLeft" => Some(OtpInputEvent::Move { forward: false }),
        "ArrowRight" => Some(OtpInputEvent::Move { forward: true }),
        "Home" => Some(OtpInputEvent::First),
        "End" => Some(OtpInputEvent::Last),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_first_cell_autofills() {
        let input = OtpInputInput { length: 4, ..OtpInputInput::default() };
        let state = OtpInputState { value: "1".into(), focus: 1, controlled: false };
        let first = cell_attrs(&state, &input, 0);
        assert_eq!(
            (first.autocomplete, first.data_state, first.value.as_str(), first.tabindex),
            ("one-time-code", "filled", "1", -1)
        );
        let second = cell_attrs(&state, &input, 1);
        assert_eq!((second.autocomplete, second.aria_label.as_str(), second.tabindex), ("off", "Digit 2 of 4", 0));
        assert_eq!(root_state(&state, &input), "incomplete");
        assert_eq!(ids("x").cell(2), "x-cell-2");
    }

    #[test]
    fn keys_map_with_the_cell_index() {
        assert_eq!(keydown("Backspace", 2), Some(OtpInputEvent::Backspace(2)));
        assert_eq!(keydown("ArrowRight", 0), Some(OtpInputEvent::Move { forward: true }));
        assert_eq!(keydown("7", 0), None);
    }
}
