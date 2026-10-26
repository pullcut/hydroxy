//! Combobox connector. Anatomy: root, label, field (anchor), control (the
//! `role="combobox"` input), trigger (chevron), listbox, option, empty. Every
//! part carries `data-scope="combobox"` and its `data-part`.

pub use oxide_machines::combobox::*;

use crate::overlay::{self, Align, Placement, Side};

/// The `data-scope` value on every combobox part.
pub const SCOPE: &str = "combobox";

/// Below the field, lined up with its start edge.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

/// Pixels between field and list.
pub const GAP_PX: u16 = 4;

/// Attributes for the text input (`role="combobox"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlAttrs {
    /// `id`, referenced by the label's `for`.
    pub id: String,
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls`: the listbox's id.
    pub aria_controls: String,
    /// `aria-activedescendant`: the highlighted option's id while open.
    pub aria_activedescendant: Option<String>,
}

/// Attributes for the floating list (`popover="manual"`, `role="listbox"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListboxAttrs {
    /// `id`, referenced by the input.
    pub id: String,
    /// `aria-labelledby`: the label's id.
    pub aria_labelledby: String,
    /// `style`: anchored below the field, at least as wide as it.
    pub style: String,
    /// `data-side`.
    pub data_side: &'static str,
    /// `data-state`: `"open"` or `"closed"`.
    pub data_state: &'static str,
}

/// Attributes for an option (`role="option"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionAttrs {
    /// `id`, referenced by `aria-activedescendant`.
    pub id: String,
    /// `aria-selected`.
    pub selected: bool,
    /// `aria-disabled`.
    pub disabled: bool,
    /// Emit `data-highlighted` when true.
    pub highlighted: bool,
}

/// The input's id.
#[must_use]
pub fn control_id(root: &str) -> String {
    format!("{root}-control")
}

/// The label's id.
#[must_use]
pub fn label_id(root: &str) -> String {
    format!("{root}-label")
}

/// The listbox's id.
#[must_use]
pub fn listbox_id(root: &str) -> String {
    format!("{root}-listbox")
}

/// The id of option `value`.
#[must_use]
pub fn option_id(root: &str, value: &str) -> String {
    format!("{root}-option-{value}")
}

/// `data-state` for open state.
#[must_use]
pub const fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Inline style for the field: names the CSS anchor.
#[must_use]
pub fn field_style(root: &str) -> String {
    overlay::anchor_style(root)
}

/// Attributes for the input.
#[must_use]
pub fn control_attrs(root: &str, state: &ComboboxState) -> ControlAttrs {
    ControlAttrs {
        id: control_id(root),
        aria_expanded: state.open,
        aria_controls: listbox_id(root),
        aria_activedescendant: state.highlighted.as_deref().filter(|_| state.open).map(|v| option_id(root, v)),
    }
}

/// Attributes for the list.
#[must_use]
pub fn listbox_attrs(root: &str, state: &ComboboxState) -> ListboxAttrs {
    ListboxAttrs {
        id: listbox_id(root),
        aria_labelledby: label_id(root),
        style: format!("{} min-width: anchor-size(width);", overlay::floating_style(root, PLACEMENT, GAP_PX)),
        data_side: PLACEMENT.side_str(),
        data_state: data_state(state.open),
    }
}

/// Attributes for `option`.
#[must_use]
pub fn option_attrs(root: &str, state: &ComboboxState, option: &ComboboxOption) -> OptionAttrs {
    OptionAttrs {
        id: option_id(root, &option.value),
        selected: state.value.as_deref() == Some(option.value.as_str()),
        disabled: option.disabled,
        highlighted: state.highlighted.as_deref() == Some(option.value.as_str()),
    }
}

/// Keys on the input. `Some` means: prevent the default.
#[must_use]
pub fn keydown(key: &str) -> Option<ComboboxEvent> {
    match key {
        "ArrowDown" => Some(ComboboxEvent::Move(Direction::Next)),
        "ArrowUp" => Some(ComboboxEvent::Move(Direction::Prev)),
        "Enter" => Some(ComboboxEvent::Enter),
        "Escape" => Some(ComboboxEvent::Escape),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activedescendant_only_while_open() {
        let mut state = ComboboxState { highlighted: Some("a".into()), ..ComboboxState::default() };
        assert_eq!(control_attrs("c", &state).aria_activedescendant, None);
        state.open = true;
        let open = state;
        assert_eq!(control_attrs("c", &open).aria_activedescendant.as_deref(), Some("c-option-a"));
        assert!(listbox_attrs("c", &open).style.contains("anchor-size(width)"));
    }
}
