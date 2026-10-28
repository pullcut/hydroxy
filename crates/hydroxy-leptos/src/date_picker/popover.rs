//! The popover shell that `<DatePicker>` and `<DateRangePicker>` both open.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::overlay::{hide_popover, show_popover};

/// Keep the native popover `content` in step with `open`. On open, focus the first element
/// matching `focus_on_open` inside it once the calendar has rendered.
pub(crate) fn follow_open(open: Memo<bool>, content: String, focus_on_open: &'static str) {
    Effect::new(move |_| {
        if open.get() {
            show_popover(&content);
            let content = content.clone();
            request_animation_frame(move || {
                let target =
                    document().get_element_by_id(&content).and_then(|c| c.query_selector(focus_on_open).ok().flatten());
                if let Some(el) = target {
                    let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
                }
            });
        } else {
            hide_popover(&content);
        }
    });
}

/// The popover `div` that holds the calendar.
#[component]
pub(crate) fn PickerContent(
    /// The `data-scope` value.
    scope: &'static str,
    /// The element id the trigger points at.
    id: String,
    /// The dialog's accessible name.
    label: String,
    /// Anchor styles.
    style: String,
    /// `data-state`: open or closed.
    #[prop(into)]
    data_state: Signal<&'static str>,
    /// Called for every key press inside the popover.
    #[prop(into)]
    on_keydown: Callback<web_sys::KeyboardEvent>,
    /// Called when the browser closed the popover (an outside click).
    #[prop(into)]
    on_dismiss: Callback<()>,
    /// The calendar.
    children: Children,
) -> impl IntoView {
    view! {
        <div
            id=id
            popover="auto"
            role="dialog"
            aria-label=label
            style=style
            data-overlay=""
            data-scope=scope
            data-part="content"
            data-state=move || data_state.get()
            on:keydown=move |ev| on_keydown.run(ev)
            on:toggle=move |ev| {
                let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                if !showing {
                    on_dismiss.run(());
                }
            }
        >
            {children()}
        </div>
    }
}
