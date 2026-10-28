//! The floating toolbar that quotes the current text selection.

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::{
    self as h, SelectionToolbarCommand, SelectionToolbarEvent, SelectionToolbarMachine,
};
use hydroxy_statechart::Machine;

use super::Selection;
use crate::drive::drive;

const WATCH_SELECTION: &str = "
if (window.__hydroxySelection) document.removeEventListener('selectionchange', window.__hydroxySelection);
window.__hydroxySelection = () => {
  const s = window.getSelection();
  let out = '';
  if (s && !s.isCollapsed && s.rangeCount) {
    const r = s.getRangeAt(0);
    const n = r.commonAncestorContainer;
    const el = n.nodeType === 1 ? n : n.parentElement;
    const m = el && el.closest('[data-message-id]');
    if (m) {
      const b = r.getBoundingClientRect();
      out = [m.dataset.messageId, b.left, b.top, s.toString()].join('\\u0001');
    }
  }
  dioxus.send(out);
};
document.addEventListener('selectionchange', window.__hydroxySelection);
await new Promise(() => {});
";

fn parse_selection(raw: &str) -> Option<Selection> {
    let mut it = raw.splitn(4, '\u{1}');
    Some(Selection {
        message_id: it.next()?.to_string(),
        left: it.next()?.parse().ok()?,
        top: it.next()?.parse().ok()?,
        text: it.next()?.to_string(),
    })
}

/// A Quote button that floats over text selected inside a message (anything under a
/// `[data-message-id]` element). Rendered by `AssistantThread` when `on_quote` is set.
#[component]
pub fn AssistantSelectionToolbar(
    /// Called with the selection when Quote is pressed. The page selection is then cleared.
    on_quote: EventHandler<Selection>,
) -> Element {
    let state = use_signal(|| SelectionToolbarMachine::initial(&()));
    let unit = use_signal(|| ());
    let send = move |event: SelectionToolbarEvent| {
        for SelectionToolbarCommand::Quote(selection) in drive::<SelectionToolbarMachine>(state, unit, event) {
            on_quote.call(selection);
            document::eval("window.getSelection()?.removeAllRanges();");
        }
    };
    use_hook(|| {
        spawn(async move {
            let mut watcher = document::eval(WATCH_SELECTION);
            while let Ok(raw) = watcher.recv::<String>().await {
                send(parse_selection(&raw).map_or(SelectionToolbarEvent::Cleared, SelectionToolbarEvent::Selected));
            }
        });
    });
    use_drop(|| {
        document::eval(
            "if (window.__hydroxySelection) { document.removeEventListener('selectionchange', window.__hydroxySelection); window.__hydroxySelection = null; }",
        );
    });

    let selection = state.read().selection.clone();
    rsx! {
        if let Some(s) = selection {
            div {
                "data-scope": h::SCOPE_SELECTION_TOOLBAR,
                "data-part": "root",
                role: "toolbar",
                "aria-label": "Selection",
                style: "left: {s.left.max(8.0)}px; top: {s.top}px;",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_SELECTION_TOOLBAR,
                    "data-part": "quote",
                    // Keep the page selection alive while the button is pressed.
                    onmousedown: move |e| e.prevent_default(),
                    onclick: move |_| send(SelectionToolbarEvent::Quote),
                    "Quote"
                }
            }
        }
    }
}
