//! `<PhoneField>`: a country picker and a phone number that groups as you type,
//! reported in E.164.

use leptos::prelude::*;
use hydroxy_headless::phone_field::{
    self as h, Country, Ids, PhoneFieldCommand, PhoneFieldEvent, PhoneFieldInput, PhoneFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::combobox::scroll_into_list;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

/// What the number reports on every change: E.164 and whether it looks valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneValue {
    /// `+` dial code and number, such as `+447400123456`. `None` when empty.
    pub e164: Option<String>,
    /// Plausible length for the chosen country.
    pub valid: bool,
}

/// An international phone number. The country button opens a searchable list
/// (name, code or dial code); typing groups the number in the country's style; a
/// number typed or pasted with `+` picks its country. Validity shows after blur.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn PhoneField(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled E.164 value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial E.164 value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// ISO code of the starting country when there is no value. Default US.
    #[prop(default = "US".into(), into)]
    default_country: String,
    /// Block changes.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Focusable but not editable.
    #[prop(optional)]
    read_only: bool,
    /// Native `required` on the number.
    #[prop(optional)]
    required: bool,
    /// Help text under the field. Replaced by the error while invalid.
    #[prop(optional, into)]
    hint: Option<String>,
    /// Form field name. Submits E.164.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with E.164 and validity whenever the number changes.
    #[prop(optional, into)]
    on_value_change: Option<Callback<PhoneValue>>,
    /// Called with the ISO code when the country changes.
    #[prop(optional, into)]
    on_country_change: Option<Callback<String>>,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let ids = StoredValue::new(h::ids(&root));
    let rid = StoredValue::new(root.clone());
    let input = Memo::new(move |_| PhoneFieldInput {
        value: value.get(),
        default_value: default_value.clone(),
        default_country: default_country.clone(),
        disabled: disabled.get().unwrap_or(false),
        read_only,
    });
    let state = RwSignal::new(input.with_untracked(PhoneFieldMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| PhoneFieldMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let control = NodeRef::<leptos::html::Input>::new();
    let send = move |event: PhoneFieldEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| PhoneFieldMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && state.with_untracked(|s| *s != next)
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                PhoneFieldCommand::ValueChange { e164, valid } => {
                    if let Some(cb) = on_value_change {
                        cb.run(PhoneValue { e164, valid });
                    }
                }
                PhoneFieldCommand::CountryChange(iso) => {
                    if let Some(cb) = on_country_change {
                        cb.run(iso);
                    }
                }
                PhoneFieldCommand::FocusNumber => focus_id(&ids.with_value(|i| i.control.clone())),
                PhoneFieldCommand::FocusTrigger => focus_id(&ids.with_value(|i| i.trigger.clone())),
            }
        }
    };
    // Write the regrouped number back with the caret on the same digit. Digits and
    // ASCII literals only, so char indices are UTF-16 offsets.
    let place_caret = move || {
        let Some(el) = control.get_untracked() else { return };
        let (text, caret) = state.with_untracked(|s| (s.text.clone(), s.caret));
        el.set_value(&text);
        let at = u32::try_from(caret).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(at, at);
    };

    let open = Memo::new(move |_| state.with(|s| s.picker_open));
    // The native popover follows the state; opening focuses the search box.
    Effect::new(move |_| {
        let Ids { picker, search, .. } = ids.get_value();
        if open.get() {
            show_popover(&picker);
            request_animation_frame(move || focus_id(&search));
        } else {
            hide_popover(&picker);
        }
    });
    // Keep the highlighted country in view.
    Effect::new(move |_| {
        if let Some(iso) = state.with(|s| s.highlighted.clone().filter(|_| s.picker_open)) {
            let list = ids.with_value(|i| i.listbox.clone());
            scroll_into_list(&list, &h::option_id(&rid.get_value(), &iso));
        }
    });

    let country = Memo::new(move |_| state.with(|s| *s.country()));
    let shown = Memo::new(move |_| state.with(|s| h::visible(&s.query)));
    let is_disabled = move || disabled.get().unwrap_or(false);
    let error = Memo::new(move |_| state.with(h::error_text));
    let data_state = move || h::data_state(open.get());
    let ids_now = ids.get_value();
    let (hint_id, error_id) = (ids_now.hint.clone(), ids_now.error.clone());
    let has_hint = hint.is_some();
    let described = move || match (error.with(Option::is_some), has_hint) {
        (true, _) => Some(ids.with_value(|i| i.error.clone())),
        (false, true) => Some(ids.with_value(|i| i.hint.clone())),
        (false, false) => None,
    };

    let option_view = move |c: Country| {
        let iso = c.iso;
        let selected = move || state.with(|s| s.country == iso);
        let highlighted = move || state.with(|s| s.highlighted.as_deref() == Some(iso));
        view! {
            <div
                id=h::option_id(&rid.get_value(), iso)
                role="option"
                aria-selected=move || selected().to_string()
                data-scope=h::SCOPE
                data-part="option"
                data-highlighted=move || highlighted().then_some("")
                // Keep focus in the search box.
                on:pointerdown=|ev| ev.prevent_default()
                on:pointermove=move |_| send(PhoneFieldEvent::Highlight(iso.to_string()))
                on:click=move |_| send(PhoneFieldEvent::Select(iso.to_string()))
            >
                <span data-scope=h::SCOPE data-part="flag" aria-hidden="true">{h::flag(iso)}</span>
                <span data-scope=h::SCOPE data-part="option-name">{c.name}</span>
                <span data-scope=h::SCOPE data-part="option-dial">{format!("+{}", c.dial)}</span>
                {move || selected().then(|| view! { <Icon name=IconName::Check size=14 /> })}
            </div>
        }
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || is_disabled().then_some("")
            data-readonly=read_only.then_some("")
            data-invalid=move || error.with(Option::is_some).then_some("")
        >
            <label id=ids_now.label.clone() data-scope=h::SCOPE data-part="label" for=ids_now.control.clone()>
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field" style=h::field_style(&root)>
                <button
                    type="button"
                    id=ids_now.trigger.clone()
                    data-scope=h::SCOPE
                    data-part="trigger"
                    aria-label=move || state.with(h::trigger_label)
                    aria-haspopup="dialog"
                    aria-expanded=move || open.get().to_string()
                    aria-controls=ids_now.picker.clone()
                    popovertarget=ids_now.picker.clone()
                    data-state=data_state
                    disabled=move || is_disabled() || read_only
                    on:click=move |ev| {
                        // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                        ev.prevent_default();
                        send(PhoneFieldEvent::TogglePicker);
                    }
                    on:keydown=move |ev| {
                        if let Some(event) = h::trigger_keydown(&ev.key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                >
                    <span data-scope=h::SCOPE data-part="flag" aria-hidden="true">{move || h::flag(country.get().iso)}</span>
                    <span data-scope=h::SCOPE data-part="dial" aria-hidden="true">{move || format!("+{}", country.get().dial)}</span>
                    <Icon name=IconName::ChevronDown size=14 />
                </button>
                <input
                    node_ref=control
                    type="tel"
                    id=ids_now.control.clone()
                    data-scope=h::SCOPE
                    data-part="control"
                    inputmode="tel"
                    autocomplete="tel-national"
                    placeholder=move || state.with(h::placeholder)
                    required=required
                    disabled=is_disabled
                    readonly=read_only
                    aria-invalid=move || error.with(Option::is_some).then_some("true")
                    aria-describedby=described
                    prop:value=move || state.with(|s| s.text.clone())
                    on:input=move |_| {
                        let Some(el) = control.get_untracked() else { return };
                        let text = el.value();
                        let caret = el.selection_start().ok().flatten().map_or(usize::MAX, |n| n as usize);
                        send(PhoneFieldEvent::SetText { text, caret });
                        place_caret();
                    }
                    on:blur=move |_| send(PhoneFieldEvent::Blur)
                />
            </div>
            <div
                id=ids_now.picker.clone()
                popover="auto"
                role="dialog"
                aria-label="Choose country"
                style=h::picker_style(&root)
                data-overlay=""
                data-scope=h::SCOPE
                data-part="picker"
                data-state=data_state
                on:toggle=move |ev| {
                    // The browser closed it (outside click): catch the machine up.
                    let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                    if !showing {
                        send(PhoneFieldEvent::ClosePicker { restore_focus: false });
                    }
                }
            >
                <div data-scope=h::SCOPE data-part="search-field">
                    <Icon name=IconName::Search size=14 />
                    <input
                        type="text"
                        id=ids_now.search.clone()
                        data-scope=h::SCOPE
                        data-part="search"
                        role="combobox"
                        aria-label="Search countries"
                        aria-autocomplete="list"
                        aria-expanded=move || open.get().to_string()
                        aria-controls=ids_now.listbox.clone()
                        aria-activedescendant=move || state.with(|s| h::active_descendant(&rid.get_value(), s))
                        autocomplete="off"
                        spellcheck="false"
                        placeholder="Country or code"
                        prop:value=move || state.with(|s| s.query.clone())
                        on:input=move |ev| send(PhoneFieldEvent::Query(event_target_value(&ev)))
                        on:keydown=move |ev| {
                            if let Some(event) = h::search_keydown(&ev.key()) {
                                ev.prevent_default();
                                if matches!(event, PhoneFieldEvent::ClosePicker { .. }) {
                                    ev.stop_propagation();
                                }
                                send(event);
                            }
                        }
                    />
                </div>
                <div
                    id=ids_now.listbox.clone()
                    role="listbox"
                    aria-label="Countries"
                    data-scope=h::SCOPE
                    data-part="listbox"
                >
                    <Show
                        when=move || shown.with(|v| !v.is_empty())
                        fallback=|| view! { <div data-scope=h::SCOPE data-part="empty">"No countries match"</div> }
                    >
                        <For each=move || shown.get().into_iter().copied() key=|c| c.iso children=option_view />
                    </Show>
                </div>
                <span data-scope=h::SCOPE data-part="status" role="status" aria-live="polite">
                    {move || state.with(|s| s.picker_open.then(|| h::result_count(&s.query)))}
                </span>
            </div>
            {move || match error.get() {
                Some(e) => Some(view! { <p id=error_id.clone() data-scope=h::SCOPE data-part="error">{e}</p> }.into_any()),
                None => hint.clone().map(|t| view! { <p id=hint_id.clone() data-scope=h::SCOPE data-part="hint">{t}</p> }.into_any()),
            }}
            {name.map(|name| view! { <input type="hidden" name=name disabled=is_disabled prop:value=move || state.with(h::form_value) /> })}
        </div>
    }
}
