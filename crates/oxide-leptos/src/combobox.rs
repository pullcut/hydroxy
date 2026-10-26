//! `<Combobox>`: a text input that filters a list of options. The list is a
//! `popover="manual"` anchored under the field; focus stays in the input and the
//! highlight is announced with `aria-activedescendant`.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::combobox::{self as h, ComboboxCommand, ComboboxEvent, ComboboxInput, ComboboxMachine};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::combobox::ComboboxOption;

/// Scroll `item` into view inside the scrollable `list` by the smallest amount.
/// The list must be the item's offset parent. Best-effort.
pub(crate) fn scroll_into_list(list_id: &str, item_id: &str) {
    let doc = document();
    let (Some(list), Some(item)) = (doc.get_element_by_id(list_id), doc.get_element_by_id(item_id)) else {
        return;
    };
    let item: web_sys::HtmlElement = item.unchecked_into();
    let (top, bottom) = (item.offset_top(), item.offset_top() + item.offset_height());
    let (scroll, height) = (list.scroll_top(), list.client_height());
    if top < scroll {
        list.set_scroll_top(top - 6);
    } else if bottom > scroll + height {
        list.set_scroll_top(bottom - height + 6);
    }
}

/// A filterable select. Type to filter, arrows to move, Enter to choose, Esc to
/// close (or clear when closed).
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn Combobox(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// The options, in display order.
    #[prop(into)]
    options: Signal<Vec<ComboboxOption>>,
    /// Controlled value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Placeholder text.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Form field name: submits the selected value through a hidden input.
    #[prop(optional, into)]
    name: Option<String>,
    /// Greyed out and not editable.
    #[prop(optional)]
    disabled: bool,
    /// Called with the chosen value, or `None` when cleared.
    #[prop(optional, into)]
    on_value_change: Option<Callback<Option<String>>>,
) -> impl IntoView {
    let input = Memo::new(move |_| ComboboxInput {
        options: options.get(),
        value: value.get(),
        default_value: default_value.clone(),
    });
    let state = RwSignal::new(input.with_untracked(ComboboxMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ComboboxMachine::sync(s, i)));
        if state.with_untracked(|s| *s != synced) {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let id = StoredValue::new(root.clone());
    // The popover follows the state; keep the highlighted option in view.
    Effect::new(move |_| {
        let (open, highlighted) = state.with(|s| (s.open, s.highlighted.clone()));
        let root = id.get_value();
        let list = h::listbox_id(&root);
        if open {
            show_popover(&list);
            if let Some(value) = highlighted {
                scroll_into_list(&list, &h::option_id(&root, &value));
            }
        } else {
            hide_popover(&list);
        }
    });

    let send = move |event: ComboboxEvent| {
        // Events can still arrive after unmount (blur on a removed element).
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| ComboboxMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if state.with_untracked(|s| *s != step.state) {
            state.set(step.state);
        }
        for ComboboxCommand::ValueChange(value) in step.commands {
            if let Some(cb) = on_value_change {
                cb.run(value);
            }
        }
    };

    let control = Memo::new(move |_| state.with(|s| h::control_attrs(&id.get_value(), s)));
    let listbox = Memo::new(move |_| state.with(|s| h::listbox_attrs(&id.get_value(), s)));

    let options_view = move || {
        let root = id.get_value();
        let shown: Vec<ComboboxOption> =
            input.with(|i| state.with(|s| h::visible(i, &s.query).into_iter().cloned().collect()));
        if shown.is_empty() {
            return view! { <div data-scope=h::SCOPE data-part="empty">"No matches"</div> }.into_any();
        }
        shown
            .into_iter()
            .map(|option| {
                let a = state.with(|s| h::option_attrs(&root, s, &option));
                let (hovered, clicked) = (option.value.clone(), option.value.clone());
                view! {
                    <div
                        id=a.id
                        role="option"
                        aria-selected=a.selected.to_string()
                        aria-disabled=a.disabled.then_some("true")
                        data-scope=h::SCOPE
                        data-part="option"
                        data-highlighted=a.highlighted.then_some("")
                        // Keep focus in the input so blur doesn't close the list first.
                        on:pointerdown=|ev| ev.prevent_default()
                        on:pointermove=move |_| send(ComboboxEvent::Highlight(hovered.clone()))
                        on:click=move |_| send(ComboboxEvent::Select(clicked.clone()))
                    >
                        <span data-scope=h::SCOPE data-part="option-label">{option.label}</span>
                        {a.selected.then(|| view! { <Icon name=IconName::Check size=14 /> })}
                    </div>
                }
            })
            .collect_view()
            .into_any()
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-disabled=disabled.then_some("")>
            <label id=h::label_id(&root) for=h::control_id(&root) data-scope=h::SCOPE data-part="label">
                {label}
            </label>
            <div
                data-scope=h::SCOPE
                data-part="field"
                data-state=move || listbox.get().data_state
                style=h::field_style(&root)
            >
                <input
                    id=move || control.get().id
                    type="text"
                    role="combobox"
                    autocomplete="off"
                    aria-autocomplete="list"
                    aria-expanded=move || control.get().aria_expanded.to_string()
                    aria-controls=move || control.get().aria_controls
                    aria-activedescendant=move || control.get().aria_activedescendant
                    placeholder=placeholder
                    disabled=disabled
                    data-scope=h::SCOPE
                    data-part="control"
                    prop:value=move || state.with(|s| s.query.clone())
                    on:input=move |ev| send(ComboboxEvent::Input(event_target_value(&ev)))
                    on:click=move |_| send(ComboboxEvent::Open)
                    on:blur=move |_| send(ComboboxEvent::Blur)
                    on:keydown=move |ev| {
                        let Some(event) = h::keydown(&ev.key()) else { return };
                        // An idle Esc (closed and empty) is left to an outer dialog.
                        let idle = state.with_untracked(|s| !s.open && s.value.is_none() && s.query.is_empty());
                        if event == ComboboxEvent::Escape {
                            if idle {
                                return;
                            }
                            ev.stop_propagation();
                        }
                        ev.prevent_default();
                        send(event);
                    }
                />
                <span data-scope=h::SCOPE data-part="chevron" aria-hidden="true">
                    <Icon name=IconName::ChevronDown size=18 />
                </span>
            </div>
            <div
                id=move || listbox.get().id
                role="listbox"
                popover="manual"
                aria-labelledby=move || listbox.get().aria_labelledby
                style=move || listbox.get().style
                data-overlay=""
                data-side=move || listbox.get().data_side
                data-state=move || listbox.get().data_state
                data-scope=h::SCOPE
                data-part="listbox"
            >
                {options_view}
            </div>
            {name
                .map(|name| {
                    view! {
                        <input
                            type="hidden"
                            name=name
                            prop:value=move || state.with(|s| s.value.clone().unwrap_or_default())
                        />
                    }
                })}
        </div>
    }
}
