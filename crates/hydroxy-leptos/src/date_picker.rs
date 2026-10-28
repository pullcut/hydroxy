//! `<DatePicker>`: a text field that takes a typed `YYYY-MM-DD` date, and a
//! button that opens a [`Calendar`](crate::calendar::Calendar) in a native
//! `popover="auto"` anchored under the field.

use hydroxy_headless::calendar::CalendarInput;
use hydroxy_headless::date_picker::{
    self as h, DatePickerCommand, DatePickerEvent, DatePickerInput, DatePickerMachine, DatePickerState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::calendar::{calendar_view, use_today};
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::calendar::Date;

/// A date field with a calendar popover.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DatePicker(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<Date>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_value: Option<Date>,
    /// Earliest selectable date.
    #[prop(optional, into)]
    min: MaybeProp<Date>,
    /// Latest selectable date.
    #[prop(optional, into)]
    max: MaybeProp<Date>,
    /// Days that can't be picked.
    #[prop(optional, into)]
    unavailable: MaybeProp<Vec<Date>>,
    /// First day of the week, 0 = Sunday … 6 = Saturday. Default Monday.
    #[prop(default = 1)]
    week_start: u8,
    /// Today's date. Default: the browser's local date.
    #[prop(optional)]
    today: Option<Date>,
    /// Block editing and the calendar.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name: submits the date as `YYYY-MM-DD` through a hidden input.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the new date, or `None` when the field is cleared.
    #[prop(optional, into)]
    on_change: Option<Callback<Option<Date>>>,
) -> impl IntoView {
    let today = use_today(today);
    let rules = Memo::new(move |_| CalendarInput {
        today: today.get(),
        week_start,
        min: min.get(),
        max: max.get(),
        unavailable: unavailable.get().unwrap_or_default(),
        disabled: disabled.get().unwrap_or(false),
        ..CalendarInput::default()
    });
    let input = Memo::new(move |_| DatePickerInput { value: value.get(), default_value, rules: rules.get() });
    let state = RwSignal::new(input.with_untracked(DatePickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DatePickerMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let ids = StoredValue::new(h::ids(&root));
    let send = move |event: DatePickerEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| DatePickerMachine::transition(s, event, i)))
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
                DatePickerCommand::ValueChange(date) => {
                    if let Some(cb) = on_change {
                        cb.run(date);
                    }
                }
                DatePickerCommand::FocusTrigger => focus_id(&ids.with_value(|i| i.trigger.clone())),
            }
        }
    };

    let open = Memo::new(move |_| state.with(|s| s.open));
    // The native popover follows the state; on open, focus the calendar's day.
    Effect::new(move |_| {
        let content = ids.with_value(|i| i.content.clone());
        if open.get() {
            show_popover(&content);
            request_animation_frame(move || {
                let target = document()
                    .get_element_by_id(&content)
                    .and_then(|c| c.query_selector(h::FOCUS_ON_OPEN).ok().flatten());
                if let Some(el) = target {
                    let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
                }
            });
        } else {
            hide_popover(&content);
        }
    });

    // The calendar always shows the picker's value.
    let calendar_input = Memo::new(move |_| CalendarInput { value: state.with(|s| s.value), ..rules.get() });
    let is_disabled = move || rules.with(|r| r.disabled);
    let data_state = move || state.with(h::data_state);
    let ids_now = ids.get_value();
    let trigger_label = format!("Choose {}", label.to_lowercase());
    let (content_label, calendar_label) = (label.clone(), label.clone());

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || is_disabled().then_some("")
            data-invalid=move || state.with(DatePickerState::invalid).then_some("")
        >
            <label data-scope=h::SCOPE data-part="label" for=ids_now.input.clone()>
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field" style=h::field_style(&root)>
                <input
                    type="text"
                    data-scope=h::SCOPE
                    data-part="input"
                    id=ids_now.input
                    inputmode="numeric"
                    autocomplete="off"
                    placeholder=h::PLACEHOLDER
                    disabled=is_disabled
                    aria-invalid=move || state.with(DatePickerState::invalid).then_some("true")
                    prop:value=move || state.with(|s| s.text.clone())
                    on:input=move |ev| send(DatePickerEvent::Input(event_target_value(&ev)))
                    on:blur=move |_| send(DatePickerEvent::Commit)
                    on:keydown=move |ev| {
                        if let Some(event) = h::input_keydown(&ev.key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                />
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="trigger"
                    id=ids_now.trigger
                    aria-label=trigger_label
                    aria-haspopup="dialog"
                    aria-expanded=move || open.get().to_string()
                    aria-controls=ids_now.content.clone()
                    popovertarget=ids_now.content.clone()
                    data-state=data_state
                    disabled=is_disabled
                    on:click=move |ev| {
                        // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                        ev.prevent_default();
                        send(DatePickerEvent::Toggle);
                    }
                >
                    <Icon name=IconName::ChevronDown />
                </button>
            </div>
            {name.map(|name| {
                view! {
                    <input
                        type="hidden"
                        name=name
                        prop:value=move || state.with(|s| s.value.map(|d| d.to_string()).unwrap_or_default())
                    />
                }
            })}
            <div
                id=ids_now.content
                popover="auto"
                role="dialog"
                aria-label=content_label
                style=h::content_style(&root)
                data-overlay=""
                data-scope=h::SCOPE
                data-part="content"
                data-state=data_state
                on:keydown=move |ev| {
                    if let Some(event) = h::content_keydown(&ev.key()) {
                        ev.prevent_default();
                        send(event);
                    }
                }
                on:toggle=move |ev| {
                    // The browser closed it (outside click): catch the machine up.
                    let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                    if !showing {
                        send(DatePickerEvent::Close { restore_focus: false });
                    }
                }
            >
                // Mounted only while open, so each opening starts on the value (or today).
                <Show when=move || open.get()>
                    {calendar_view(
                        calendar_input,
                        true,
                        calendar_label.clone(),
                        Some(Callback::new(move |date| send(DatePickerEvent::Select(date)))),
                    )}
                </Show>
            </div>
        </div>
    }
}
