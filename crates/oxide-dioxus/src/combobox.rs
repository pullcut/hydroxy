//! `Combobox`: a text input that filters a list of options. The list is a
//! `popover="manual"` anchored under the field; focus stays in the input and the
//! highlight is announced with `aria-activedescendant`.

use dioxus::prelude::*;
use oxide_headless::combobox::{self as h, ComboboxCommand, ComboboxEvent, ComboboxInput, ComboboxMachine};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::combobox::ComboboxOption;

/// Scroll the element with this id into view inside its scrolling list, by the
/// smallest amount. Runs through `document::eval`, like the overlay helpers.
pub(crate) fn scroll_into_list(item_id: &str) {
    let _ = document::eval(&format!("document.getElementById({item_id:?})?.scrollIntoView({{ block: 'nearest' }});"));
}

/// A filterable select. Type to filter, arrows to move, Enter to choose, Esc to
/// close (or clear when closed).
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn Combobox(
    /// Visible label.
    label: String,
    /// The options, in display order.
    options: ReadSignal<Vec<ComboboxOption>>,
    /// Controlled value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<String>,
    /// Placeholder text.
    #[props(default)]
    placeholder: Option<String>,
    /// Form field name: submits the selected value through a hidden input.
    #[props(default)]
    name: Option<String>,
    /// Greyed out and not editable.
    #[props(default)]
    disabled: bool,
    /// Called with the chosen value, or `None` when cleared.
    on_value_change: Option<EventHandler<Option<String>>>,
) -> Element {
    let input =
        use_memo(move || ComboboxInput { options: options(), value: value(), default_value: default_value.clone() });
    let mut state = use_signal(|| ComboboxMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ComboboxMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    // The popover follows the state; keep the highlighted option in view.
    {
        let root = root.clone();
        use_effect(move || {
            let (open, highlighted) = {
                let s = state.read();
                (s.open, s.highlighted.clone())
            };
            let list = h::listbox_id(&root);
            if open {
                show_popover(&list);
                if let Some(value) = highlighted {
                    scroll_into_list(&h::option_id(&root, &value));
                }
            } else {
                hide_popover(&list);
            }
        });
    }

    let send = move |event: ComboboxEvent| {
        let step = ComboboxMachine::transition(&state.peek(), event, &input.peek());
        let mut state = state;
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for ComboboxCommand::ValueChange(value) in step.commands {
            if let Some(handler) = on_value_change {
                handler.call(value);
            }
        }
    };

    let s = state.read();
    let control = h::control_attrs(&root, &s);
    let listbox = h::listbox_attrs(&root, &s);
    let query = s.query.clone();
    let selected = s.value.clone().unwrap_or_default();
    let shown: Vec<(ComboboxOption, h::OptionAttrs)> =
        h::visible(&input.read(), &s.query).into_iter().map(|o| (o.clone(), h::option_attrs(&root, &s, o))).collect();
    drop(s);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-disabled": disabled.then_some(""),
            label {
                id: h::label_id(&root),
                r#for: h::control_id(&root),
                "data-scope": h::SCOPE,
                "data-part": "label",
                {label}
            }
            div {
                "data-scope": h::SCOPE,
                "data-part": "field",
                "data-state": listbox.data_state,
                style: h::field_style(&root),
                input {
                    id: control.id,
                    r#type: "text",
                    role: "combobox",
                    autocomplete: "off",
                    "aria-autocomplete": "list",
                    "aria-expanded": control.aria_expanded.to_string(),
                    "aria-controls": control.aria_controls,
                    "aria-activedescendant": control.aria_activedescendant,
                    placeholder,
                    disabled,
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    value: query,
                    oninput: move |e| send(ComboboxEvent::Input(e.value())),
                    onclick: move |_| send(ComboboxEvent::Open),
                    onblur: move |_| send(ComboboxEvent::Blur),
                    onkeydown: move |e| {
                        let Some(event) = h::keydown(&e.key().to_string()) else { return };
                        // An idle Esc (closed and empty) is left to an outer dialog.
                        if event == ComboboxEvent::Escape {
                            let idle = {
                                let s = state.peek();
                                !s.open && s.value.is_none() && s.query.is_empty()
                            };
                            if idle {
                                return;
                            }
                            e.stop_propagation();
                        }
                        e.prevent_default();
                        send(event);
                    },
                }
                span { "data-scope": h::SCOPE, "data-part": "chevron", "aria-hidden": "true",
                    Icon { name: IconName::ChevronDown, size: 18 }
                }
            }
            div {
                id: listbox.id,
                role: "listbox",
                popover: "manual",
                "aria-labelledby": listbox.aria_labelledby,
                style: listbox.style,
                "data-overlay": "",
                "data-side": listbox.data_side,
                "data-state": listbox.data_state,
                "data-scope": h::SCOPE,
                "data-part": "listbox",
                if shown.is_empty() {
                    div { "data-scope": h::SCOPE, "data-part": "empty", "No matches" }
                }
                for (option, a) in shown {
                    div {
                        key: "{option.value}",
                        id: a.id,
                        role: "option",
                        "aria-selected": a.selected.to_string(),
                        "aria-disabled": a.disabled.then_some("true"),
                        "data-scope": h::SCOPE,
                        "data-part": "option",
                        "data-highlighted": a.highlighted.then_some(""),
                        // Keep focus in the input so blur doesn't close the list first.
                        onpointerdown: move |e| e.prevent_default(),
                        onpointermove: {
                            let value = option.value.clone();
                            move |_| send(ComboboxEvent::Highlight(value.clone()))
                        },
                        onclick: {
                            let value = option.value.clone();
                            move |_| send(ComboboxEvent::Select(value.clone()))
                        },
                        span { "data-scope": h::SCOPE, "data-part": "option-label", {option.label} }
                        if a.selected {
                            Icon { name: IconName::Check, size: 14 }
                        }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: selected }
            }
        }
    }
}
