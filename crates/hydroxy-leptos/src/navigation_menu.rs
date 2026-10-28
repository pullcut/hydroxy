//! `<NavigationMenu>` / `<NavigationMenuItem>` / `<NavigationMenuLink>` /
//! `<NavigationMenuPanelLink>`. A site nav: top-level links and triggers that
//! open flyout panels. Panels are native `popover="auto"`, anchored under their
//! trigger. Hover opens a panel after a short rest, and a close delay lets the
//! pointer cross the gap to the panel.

use std::time::Duration;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use hydroxy_headless::navigation_menu::{
    self as h, NavigationMenuCommand, NavigationMenuEvent, NavigationMenuInput,
    NavigationMenuMachine, NavigationMenuState,
};
use hydroxy_statechart::Machine;

use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::navigation_menu::NavigationMenuEntry;
pub use hydroxy_headless::overlay::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<NavigationMenuState>,
    input: Memo<NavigationMenuInput>,
    on_open_change: Option<Callback<Option<String>>>,
}

impl Ctx {
    fn send(self, event: NavigationMenuEvent) {
        // Events can still arrive after unmount (a timer, or blur on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| NavigationMenuMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && self.state.with_untracked(|s| *s != next)
        {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                NavigationMenuCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                NavigationMenuCommand::Wait { id, ms } => {
                    set_timeout(move || self.send(NavigationMenuEvent::Elapsed(id)), Duration::from_millis(ms.into()));
                }
                NavigationMenuCommand::FocusItem(value) => focus_id(&h::item_id(&self.id.get_value(), &value)),
                NavigationMenuCommand::FocusPanel(value) => focus_first_link(&h::content_id(&self.id.get_value(), &value)),
            }
        }
    }
}

/// Focus the first link in the panel, once it is showing.
fn focus_first_link(panel_id: &str) {
    let panel = panel_id.to_string();
    // The panel is shown by an effect after this runs, so wait a frame.
    request_animation_frame(move || {
        let first = document().get_element_by_id(&panel).and_then(|el| el.query_selector("a, button").ok().flatten());
        if let Some(el) = first {
            let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
        }
    });
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("NavigationMenu parts must be rendered inside <NavigationMenu>")
}

/// Root: a `<nav>` holding the list of items. Owns the machine and the hover intent.
#[component]
pub fn NavigationMenu(
    /// The top-level items, in display order. Must match the `value`s of the parts.
    #[prop(into)]
    items: Signal<Vec<NavigationMenuEntry>>,
    /// Accessible name for the nav.
    #[prop(default = "Main".into(), into)]
    label: String,
    /// Controlled open panel (`Some(None)` is none open). Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<Option<String>>,
    /// Initial open panel when uncontrolled.
    #[prop(optional, into)]
    default_open: Option<String>,
    /// Milliseconds the pointer rests on a trigger before its panel opens.
    #[prop(default = 150)]
    open_delay_ms: u32,
    /// Milliseconds after the pointer leaves before the panel closes.
    #[prop(default = 250)]
    close_delay_ms: u32,
    /// Called with the open panel's value, or `None`, whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<Option<String>>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| NavigationMenuInput {
        items: items.get(),
        open: open.get(),
        default_open: default_open.clone(),
        open_delay_ms,
        close_delay_ms,
    });
    let state = RwSignal::new(input.with_untracked(NavigationMenuMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| NavigationMenuMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let ctx = Ctx { id, state, input, on_open_change };
    provide_context(ctx);
    view! {
        <nav
            aria-label=label
            data-scope=h::SCOPE
            data-part="root"
            on:pointerleave=move |_| ctx.send(NavigationMenuEvent::Leave)
            on:focusout=move |ev| {
                // Focus left the nav (and its panels): close.
                let inside = ev
                    .related_target()
                    .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                    .zip(ev.current_target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()))
                    .is_some_and(|(related, nav)| nav.contains(Some(&related)));
                if !inside {
                    ctx.send(NavigationMenuEvent::Close);
                }
            }
        >
            <ul data-scope=h::SCOPE data-part="list">{children()}</ul>
        </nav>
    }
}

/// A top-level item with a flyout panel. Children are the panel's content,
/// usually `NavigationMenuPanelLink`s.
#[component]
pub fn NavigationMenuItem(
    /// Unique key. Must match a panel entry in the nav's `items`.
    #[prop(into)]
    value: String,
    /// Text of the trigger.
    #[prop(into)]
    label: String,
    /// Where the panel sits relative to its trigger. Default below, start-aligned.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let value = StoredValue::new(value);
    let trigger = Memo::new(move |_| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s, &value.get_value())));
    let content =
        Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, &value.get_value(), placement)));
    // The popover follows the state, whoever changed it.
    let is_open = Memo::new(move |_| ctx.state.with(|s| s.open.as_deref() == Some(value.get_value().as_str())));
    Effect::new(move |_| {
        let panel = h::content_id(&ctx.id.get_value(), &value.get_value());
        if is_open.get() { show_popover(&panel) } else { hide_popover(&panel) }
    });
    view! {
        <li data-scope=h::SCOPE data-part="item">
            <button
                type="button"
                id=move || h::item_id(&ctx.id.get_value(), &value.get_value())
                aria-expanded=move || trigger.get().aria_expanded.to_string()
                aria-controls=move || trigger.get().aria_controls
                popovertarget=move || trigger.get().aria_controls
                style=move || trigger.get().style
                data-scope=h::SCOPE
                data-part="trigger"
                data-state=move || trigger.get().data_state
                on:click=move |ev| {
                    // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                    ev.prevent_default();
                    ctx.send(NavigationMenuEvent::Toggle(value.get_value()));
                }
                on:pointerenter=move |_| ctx.send(NavigationMenuEvent::Enter(value.get_value()))
                on:keydown=move |ev| {
                    if let Some(event) = h::item_keydown(&ev.key(), &value.get_value(), true) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
            >
                {label}
                <span data-scope=h::SCOPE data-part="chevron"><Icon name=IconName::ChevronDown size=14 /></span>
            </button>
            <div
                id=move || content.get().id
                popover="auto"
                aria-labelledby=move || content.get().aria_labelledby
                style=move || content.get().style
                data-overlay=""
                data-side=move || content.get().data_side
                data-scope=h::SCOPE
                data-part="content"
                data-state=move || content.get().data_state
                on:pointerenter=move |_| ctx.send(NavigationMenuEvent::PanelEnter)
                on:toggle=move |ev| {
                    // The browser closed it (Esc, outside press, or another panel opened): catch the machine up.
                    let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                    if !showing {
                        ctx.send(NavigationMenuEvent::Dismiss(value.get_value()));
                    }
                }
            >
                {children()}
            </div>
        </li>
    }
}

/// A top-level link with no panel.
#[component]
pub fn NavigationMenuLink(
    /// Unique key. Must match a link entry in the nav's `items`.
    #[prop(into)]
    value: String,
    /// Where the link goes.
    #[prop(into)]
    href: String,
    /// This link is the current page.
    #[prop(optional)]
    current: bool,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let value = StoredValue::new(value);
    view! {
        <li data-scope=h::SCOPE data-part="item">
            <a
                id=move || h::item_id(&ctx.id.get_value(), &value.get_value())
                href=href
                aria-current=current.then_some("page")
                data-scope=h::SCOPE
                data-part="link"
                on:pointerenter=move |_| ctx.send(NavigationMenuEvent::Enter(value.get_value()))
                on:keydown=move |ev| {
                    if let Some(event) = h::item_keydown(&ev.key(), &value.get_value(), false) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
            >
                {children()}
            </a>
        </li>
    }
}

/// A link inside a panel: a title with an optional description under it.
#[component]
pub fn NavigationMenuPanelLink(
    /// Where the link goes.
    #[prop(into)]
    href: String,
    /// Second line under the title.
    #[prop(optional, into)]
    description: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a href=href data-scope=h::SCOPE data-part="panel-link">
            <span data-scope=h::SCOPE data-part="panel-link-title">{children()}</span>
            {description.map(|d| view! { <span data-scope=h::SCOPE data-part="panel-link-description">{d}</span> })}
        </a>
    }
}
