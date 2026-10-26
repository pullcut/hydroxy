//! `<SearchField>`: a search input with a leading icon and a clear button.

use leptos::html;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};
pub use oxide_headless::input::Size;

const SCOPE: &str = "search-field";

/// Search input. The clear button and Esc empty it and keep focus in the field.
#[component]
pub fn SearchField(
    /// Accessible name. Not shown; the icon and placeholder carry the meaning visually.
    #[prop(into, default = "Search".into())]
    label: String,
    /// Controlled value. Pair with `on_input`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Called with the new text on every keystroke, and with `""` on clear.
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,
    /// Placeholder text.
    #[prop(into, default = "Search".into())]
    placeholder: String,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// `Md` for toolbars, `Lg` for page-level search.
    #[prop(optional)]
    size: Size,
) -> impl IntoView {
    let local = RwSignal::new(default_value);
    let text = move || value.get().unwrap_or_else(|| local.get());
    let control = NodeRef::<html::Input>::new();
    let set = move |v: String| {
        local.set(v.clone());
        if let Some(cb) = on_input {
            cb.run(v);
        }
    };
    let clear = move || {
        set(String::new());
        if let Some(el) = control.get_untracked() {
            let _ = el.focus();
        }
    };
    let icon_size = if size == Size::Lg { 18 } else { 14 };

    view! {
        <div
            data-scope=SCOPE
            data-part="root"
            data-size=size.as_str()
            data-disabled=move || disabled.get().unwrap_or(false).then_some("")
        >
            <span data-scope=SCOPE data-part="icon">
                <Icon name=IconName::Search size=icon_size />
            </span>
            <input
                node_ref=control
                data-scope=SCOPE
                data-part="control"
                type="search"
                aria-label=label
                placeholder=placeholder
                name=name
                autocomplete="off"
                disabled=move || disabled.get().unwrap_or(false)
                prop:value=text
                on:input=move |ev| set(event_target_value(&ev))
                on:keydown=move |ev| {
                    // With text: Esc clears. Empty: let Esc reach a parent (a dialog, say).
                    if ev.key() == "Escape" && !text().is_empty() {
                        ev.prevent_default();
                        ev.stop_propagation();
                        clear();
                    }
                }
            />
            <button
                type="button"
                data-scope=SCOPE
                data-part="clear"
                aria-label="Clear search"
                hidden=move || text().is_empty() || disabled.get().unwrap_or(false)
                // Keep focus in the field when clicked.
                on:pointerdown=|ev| ev.prevent_default()
                on:click=move |_| clear()
            >
                <Icon name=IconName::X size=13 />
            </button>
        </div>
    }
}
