//! `SidebarLayout`: an app shell with a collapsible sidebar, a top bar and
//! the main region. On narrow screens the sidebar becomes a native popover drawer.

use dioxus::prelude::*;
use hydroxy_headless::sidebar_layout::{
    self as h, CollapsibleCommand, CollapsibleEvent, CollapsibleInput, CollapsibleMachine, CollapsibleState,
};
use hydroxy_statechart::Machine;

pub use hydroxy_headless::sidebar_layout::CollapseMode;

fn toggle(mut state: Signal<CollapsibleState>, input: Memo<CollapsibleInput>, on_open_change: Option<EventHandler<bool>>) {
    let step = CollapsibleMachine::transition(&state.peek(), CollapsibleEvent::Toggle, &input.peek());
    if let Some(next) = step.state {
        state.set(next);
    }
    for command in step.commands {
        let CollapsibleCommand::OpenChange(open) = command;
        if let Some(handler) = on_open_change {
            handler.call(open);
        }
    }
}

/// Window-level Ctrl/Cmd+B listener (mirrors `is_toggle_shortcut`), plus closing
/// the drawer when a link inside it is followed. Both run in the webview.
// the window listener outlives the component; it only sends into a closed channel after unmount.
fn wiring_script(sidebar_id: &str) -> String {
    format!(
        "const s = document.getElementById({sidebar_id:?}); \
         if (s) s.addEventListener('click', e => {{ if (e.target.closest('a') && s.matches(':popover-open')) s.hidePopover(); }}); \
         window.addEventListener('keydown', e => {{ if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {{ e.preventDefault(); dioxus.send(true); }} }});"
    )
}

/// App shell. Children are the main content. Ctrl/Cmd+B toggles the sidebar.
#[component]
pub fn SidebarLayout(
    /// Sidebar content: brand, nav items, footer.
    sidebar: Element,
    /// Top bar content, after the toggle.
    header: Option<Element>,
    /// Accessible name of the sidebar's `<nav>`. Default "Main".
    #[props(default = "Main".to_string())]
    label: String,
    /// Controlled expanded state on wide screens. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial expanded state when uncontrolled. Default `true`.
    #[props(default = true)]
    default_open: bool,
    /// Called with the new expanded state.
    on_open_change: Option<EventHandler<bool>>,
    /// What a closed sidebar becomes: an icon rail (default) or nothing.
    #[props(default)]
    collapse_to: CollapseMode,
    /// The main content.
    children: Element,
) -> Element {
    let input = use_memo(move || CollapsibleInput { open: open(), default_open, disabled: false });
    let mut state = use_signal(|| CollapsibleMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CollapsibleMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let sidebar_id = use_hook(|| h::sidebar_id(&hydroxy_headless::next_id(h::SCOPE)));
    let script = wiring_script(&sidebar_id);
    use_future(move || {
        let script = script.clone();
        async move {
            let mut wiring = document::eval(&script);
            while wiring.recv::<bool>().await.is_ok() {
                toggle(state, input, on_open_change);
            }
        }
    });

    let open_now = state.read().open;
    let data_state = h::data_state(open_now, collapse_to);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-state": data_state,
            nav {
                id: sidebar_id.clone(),
                "popover": "auto",
                "aria-label": label,
                "data-scope": h::SCOPE,
                "data-part": "sidebar",
                "data-state": data_state,
                {sidebar}
            }
            div { "data-scope": h::SCOPE, "data-part": "body",
                header { "data-scope": h::SCOPE, "data-part": "header",
                    button {
                        r#type: "button",
                        "aria-label": h::TOGGLE_LABEL,
                        "aria-expanded": open_now.to_string(),
                        "aria-controls": sidebar_id.clone(),
                        "aria-keyshortcuts": "Control+B Meta+B",
                        title: "Toggle sidebar (Ctrl/⌘ B)",
                        "data-scope": h::SCOPE,
                        "data-part": "toggle",
                        onclick: move |_| toggle(state, input, on_open_change),
                        PanelGlyph {}
                    }
                    button {
                        r#type: "button",
                        "aria-label": h::MOBILE_TOGGLE_LABEL,
                        "popovertarget": sidebar_id.clone(),
                        "data-scope": h::SCOPE,
                        "data-part": "mobile-toggle",
                        PanelGlyph {}
                    }
                    {header}
                }
                main { "data-scope": h::SCOPE, "data-part": "main", {children} }
            }
        }
    }
}

/// A panel-left icon.
#[component]
fn PanelGlyph() -> Element {
    rsx! {
        svg { view_box: "0 0 16 16", width: "16", height: "16", "aria-hidden": "true",
            rect { x: "2", y: "2.5", width: "12", height: "11", rx: "2" }
            path { d: "M6 2.5v11" }
        }
    }
}
