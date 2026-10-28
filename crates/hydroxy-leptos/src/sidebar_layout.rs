//! `<SidebarLayout>`: an app shell with a collapsible sidebar, a top bar and
//! the main region. On narrow screens the sidebar becomes a native popover drawer.

use hydroxy_headless::sidebar_layout::{
    self as h, CollapsibleCommand, CollapsibleEvent, CollapsibleInput, CollapsibleMachine,
};
use hydroxy_statechart::Machine;
use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::drive::drive;
use crate::overlay::hide_popover;

pub use hydroxy_headless::sidebar_layout::CollapseMode;

/// App shell. Children are the main content. Ctrl/Cmd+B toggles the sidebar.
#[component]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
pub fn SidebarLayout(
    /// Sidebar content: brand, nav items, footer.
    #[prop(into)]
    sidebar: ViewFn,
    /// Top bar content, after the toggle.
    #[prop(optional, into)]
    header: Option<ViewFn>,
    /// Accessible name of the sidebar's `<nav>`. Default "Main".
    #[prop(optional, into, default = "Main".into())]
    label: String,
    /// Controlled expanded state on wide screens. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial expanded state when uncontrolled. Default `true`.
    #[prop(default = true)]
    default_open: bool,
    /// Called with the new expanded state.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// What a closed sidebar becomes: an icon rail (default) or nothing.
    #[prop(optional)]
    collapse_to: CollapseMode,
    /// The main content.
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| CollapsibleInput { open: open.get(), default_open, disabled: false });
    let state = RwSignal::new(input.with_untracked(CollapsibleMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CollapsibleMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let toggle = move || {
        for command in drive::<CollapsibleMachine, _>(state, input, CollapsibleEvent::Toggle) {
            let CollapsibleCommand::OpenChange(open) = command;
            if let Some(cb) = on_open_change {
                cb.run(open);
            }
        }
    };

    let shortcut = window_event_listener(ev::keydown, move |ev| {
        if h::is_toggle_shortcut(&ev.key(), ev.ctrl_key(), ev.meta_key()) {
            ev.prevent_default();
            toggle();
        }
    });
    on_cleanup(move || shortcut.remove());

    let sidebar_id = StoredValue::new(h::sidebar_id(&crate::id::next_id(h::SCOPE)));
    let data_state = move || h::data_state(state.get().open.value, collapse_to);
    // The drawer stays open after in-app navigation unless we close it.
    let close_on_link = move |ev: ev::MouseEvent| {
        let on_link = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("a").ok().flatten())
            .is_some();
        if on_link {
            hide_popover(&sidebar_id.read_value());
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-state=data_state>
            <nav
                id=sidebar_id.get_value()
                popover="auto"
                aria-label=label
                data-scope=h::SCOPE
                data-part="sidebar"
                data-state=data_state
                on:click=close_on_link
            >
                {sidebar.run()}
            </nav>
            <div data-scope=h::SCOPE data-part="body">
                <header data-scope=h::SCOPE data-part="header">
                    <button
                        type="button"
                        aria-label=h::TOGGLE_LABEL
                        aria-expanded=move || state.get().open.value.to_string()
                        aria-controls=sidebar_id.get_value()
                        aria-keyshortcuts="Control+B Meta+B"
                        title="Toggle sidebar (Ctrl/⌘ B)"
                        data-scope=h::SCOPE
                        data-part="toggle"
                        on:click=move |_| toggle()
                    >
                        <PanelGlyph />
                    </button>
                    <button
                        type="button"
                        aria-label=h::MOBILE_TOGGLE_LABEL
                        popovertarget=sidebar_id.get_value()
                        data-scope=h::SCOPE
                        data-part="mobile-toggle"
                    >
                        <PanelGlyph />
                    </button>
                    {header.map(|h| h.run())}
                </header>
                <main data-scope=h::SCOPE data-part="main">{children()}</main>
            </div>
        </div>
    }
}

/// A panel-left icon.
#[component]
fn PanelGlyph() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <rect x="2" y="2.5" width="12" height="11" rx="2" />
            <path d="M6 2.5v11" />
        </svg>
    }
}
