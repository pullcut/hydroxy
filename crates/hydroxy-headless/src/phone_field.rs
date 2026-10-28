//! Phone field connector. Anatomy, all under `data-scope="phone-field"`: root,
//! label, field (the anchor), trigger (country button: flag, dial code, chevron),
//! control (`type="tel"` input), picker (`popover="auto"`, `role="dialog"`),
//! search (`role="combobox"`), listbox, option (flag, name, dial), empty, hint,
//! error, hidden input.
//!
//! The picker follows the APG combobox pattern: focus stays in the search box and
//! the highlighted country is announced with `aria-activedescendant`.

pub use hydroxy_machines::listbox::Direction;
pub use hydroxy_machines::phone_field::*;

use crate::overlay::{self, Align, Placement, Side};

/// `data-scope` for every part.
pub const SCOPE: &str = "phone-field";

/// Below the field, lined up with its start edge.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

/// Pixels between field and picker.
pub const GAP_PX: u16 = 6;

/// Element ids for one phone field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The visible label.
    pub label: String,
    /// The number input.
    pub control: String,
    /// The country button.
    pub trigger: String,
    /// The picker popover.
    pub picker: String,
    /// The picker's search box.
    pub search: String,
    /// The country list.
    pub listbox: String,
    /// Help text under the field.
    pub hint: String,
    /// The error under the field.
    pub error: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids {
        label: format!("{root}-label"),
        control: format!("{root}-control"),
        trigger: format!("{root}-trigger"),
        picker: format!("{root}-picker"),
        search: format!("{root}-search"),
        listbox: format!("{root}-listbox"),
        hint: format!("{root}-hint"),
        error: format!("{root}-error"),
    }
}

/// The id of the option for country `iso`.
#[must_use]
pub fn option_id(root: &str, iso: &str) -> String {
    format!("{root}-country-{iso}")
}

/// `style` for the field: names the CSS anchor.
#[must_use]
pub fn field_style(root: &str) -> String {
    overlay::anchor_style(root)
}

/// `style` for the picker: anchored under the field, at least as wide as it.
#[must_use]
pub fn picker_style(root: &str) -> String {
    format!("{} min-width: anchor-size(width);", overlay::floating_style(root, PLACEMENT, GAP_PX))
}

/// `data-state` for the trigger and picker.
#[must_use]
pub const fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// The country button's accessible name, such as "Country: United Kingdom +44".
#[must_use]
pub fn trigger_label(state: &PhoneFieldState) -> String {
    let c = state.country();
    format!("Country: {} +{}", c.name, c.dial)
}

/// The search box's `aria-activedescendant` while the picker is open.
#[must_use]
pub fn active_descendant(root: &str, state: &PhoneFieldState) -> Option<String> {
    state.highlighted.as_deref().filter(|_| state.picker_open).map(|iso| option_id(root, iso))
}

/// Error text while the number is invalid.
#[must_use]
pub fn error_text(state: &PhoneFieldState) -> Option<String> {
    state.invalid().then(|| format!("Enter a valid {} number, like {}", state.country().name, placeholder(state)))
}

/// The number input's placeholder: the country's example, laid out.
#[must_use]
pub fn placeholder(state: &PhoneFieldState) -> String {
    state.country().placeholder()
}

/// The hidden input's value: E.164, empty when there is no number.
#[must_use]
pub fn form_value(state: &PhoneFieldState) -> String {
    state.e164().unwrap_or_default()
}

/// The polite live message after the picker list changes: how many countries match.
#[must_use]
pub fn result_count(query: &str) -> String {
    match visible(query).len() {
        0 => "No countries match".to_string(),
        1 => "1 country".to_string(),
        n => format!("{n} countries"),
    }
}

/// Keys on the country button. `Some` means: prevent the default.
#[must_use]
pub fn trigger_keydown(key: &str) -> Option<PhoneFieldEvent> {
    matches!(key, "ArrowDown" | "ArrowUp").then_some(PhoneFieldEvent::OpenPicker)
}

/// Keys in the picker's search box. `Some` means: prevent the default.
#[must_use]
pub fn search_keydown(key: &str) -> Option<PhoneFieldEvent> {
    match key {
        "ArrowDown" => Some(PhoneFieldEvent::Move(Direction::Next)),
        "ArrowUp" => Some(PhoneFieldEvent::Move(Direction::Prev)),
        "Enter" => Some(PhoneFieldEvent::Enter),
        "Escape" => Some(PhoneFieldEvent::ClosePicker { restore_focus: true }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Machine;

    #[test]
    fn labels_and_ids() {
        let s = PhoneFieldMachine::initial(&PhoneFieldInput { default_country: "GB".into(), ..PhoneFieldInput::default() });
        assert_eq!(trigger_label(&s), "Country: United Kingdom +44");
        assert_eq!(placeholder(&s), "7400 123456");
        assert_eq!(active_descendant("p", &s), None);
        let open = PhoneFieldState { picker_open: true, highlighted: Some("FR".into()), ..s };
        assert_eq!(active_descendant("p", &open).as_deref(), Some("p-country-FR"));
        assert!(picker_style("p").contains("position-anchor: --p;"));
    }

    #[test]
    fn keys() {
        assert_eq!(trigger_keydown("ArrowDown"), Some(PhoneFieldEvent::OpenPicker));
        assert_eq!(search_keydown("Escape"), Some(PhoneFieldEvent::ClosePicker { restore_focus: true }));
        assert_eq!(search_keydown("a"), None);
        assert_eq!(result_count("zzz"), "No countries match");
        assert_eq!(result_count("+44"), "1 country");
    }

    #[test]
    fn error_names_the_country() {
        let s = PhoneFieldState { country: "FR".into(), national: "06".into(), touched: true, ..PhoneFieldState::default() };
        assert_eq!(error_text(&s).as_deref(), Some("Enter a valid France number, like 6 12 34 56 78"));
        assert_eq!(form_value(&s), "+336");
    }
}
