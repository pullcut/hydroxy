//! The quote-selection toolbar.

use hydroxy_headless::assistant_thread::{
    self as h, SelectionToolbarCommand, SelectionToolbarEvent, SelectionToolbarMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, closure::Closure};

use super::Selection;
use crate::drive::drive;
use crate::local::Local;

fn read_selection() -> Option<Selection> {
    let sel = window().get_selection().ok()??;
    if sel.is_collapsed() || sel.range_count() == 0 {
        return None;
    }
    let range = sel.get_range_at(0).ok()?;
    let node = range.common_ancestor_container().ok()?;
    let el = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element())?;
    let message = el.closest("[data-message-id]").ok()??;
    let rect = range.get_bounding_client_rect();
    Some(Selection {
        message_id: message.get_attribute("data-message-id")?,
        text: String::from(sel.to_string()),
        left: rect.left(),
        top: rect.top(),
    })
}

/// A Quote button that floats over text selected inside a message (anything under a
/// `[data-message-id]` element). Rendered by [`AssistantThread`](super::AssistantThread) when `on_quote` is set.
#[component]
pub fn AssistantSelectionToolbar(
    /// Called with the selection when Quote is pressed. The page selection is then cleared.
    on_quote: Callback<Selection>,
) -> impl IntoView {
    let state = RwSignal::new(SelectionToolbarMachine::initial(&()));
    let input = Signal::stored(());
    let send = move |event: SelectionToolbarEvent| {
        for SelectionToolbarCommand::Quote(selection) in drive::<SelectionToolbarMachine, _>(state, input, event) {
            on_quote.run(selection);
            if let Ok(Some(sel)) = window().get_selection() {
                let _ = sel.remove_all_ranges();
            }
        }
    };

    Effect::new(move |_| {
        let listener = Closure::<dyn Fn()>::new(move || {
            send(read_selection().map_or(SelectionToolbarEvent::Cleared, SelectionToolbarEvent::Selected));
        });
        let doc = document();
        let _ = doc.add_event_listener_with_callback("selectionchange", listener.as_ref().unchecked_ref());
        let held = Local::new(listener);
        on_cleanup(move || {
            held.with(|l| {
                let _ = document().remove_event_listener_with_callback("selectionchange", l.as_ref().unchecked_ref());
            });
        });
    });

    view! {
        {move || {
            state
                .with(|s| s.selection.clone())
                .map(|s| {
                    view! {
                        <div
                            data-scope=h::SCOPE_SELECTION_TOOLBAR
                            data-part="root"
                            role="toolbar"
                            aria-label="Selection"
                            style=format!("left: {}px; top: {}px;", s.left.max(8.0), s.top)
                        >
                            <button
                                type="button"
                                data-scope=h::SCOPE_SELECTION_TOOLBAR
                                data-part="quote"
                                // Keep the page selection alive while the button is pressed.
                                on:mousedown=move |ev| ev.prevent_default()
                                on:click=move |_| send(SelectionToolbarEvent::Quote)
                            >
                                "Quote"
                            </button>
                        </div>
                    }
                })
        }}
    }
}
