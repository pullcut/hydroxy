//! `NavigationMenu` / `NavigationMenuItem` / `NavigationMenuLink` /
//! `NavigationMenuPanelLink`. A site nav: top-level links and triggers that
//! open flyout panels. Panels are native `popover="auto"`, anchored under their
//! trigger. Hover opens a panel after a short rest, and a close delay lets the
//! pointer cross the gap to the panel.

use dioxus::prelude::*;
use hydroxy_headless::navigation_menu::{
    self as h, NavigationMenuCommand, NavigationMenuEvent, NavigationMenuInput, NavigationMenuMachine,
    NavigationMenuState,
};
use hydroxy_statechart::Machine;

use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::navigation_menu::NavigationMenuEntry;
pub use hydroxy_headless::overlay::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<NavigationMenuState>,
    input: Memo<NavigationMenuInput>,
    /// Triggers and links, by item value.
    focus: FocusMap,
    on_open_change: Option<EventHandler<Option<String>>>,
}

impl Ctx {
    fn send(mut self, event: NavigationMenuEvent) {
        let step = NavigationMenuMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state
            && *self.state.peek() != next
        {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                NavigationMenuCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
                // The clock runs through eval, so it works on web and desktop alike.
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                NavigationMenuCommand::Wait { id, ms } => {
                    spawn(async move {
                        let js = format!("await new Promise(r => setTimeout(r, {ms})); return true;");
                        let _ = document::eval(&js).join::<bool>().await;
                        self.send(NavigationMenuEvent::Elapsed(id));
                    });
                }
                NavigationMenuCommand::FocusItem(value) => self.focus.focus(&value),
                NavigationMenuCommand::FocusPanel(value) => {
                    // The panel is shown by an effect after this runs, so wait a frame.
                    let panel = h::content_id(&self.id.peek(), &value);
                    let js = format!(
                        "await new Promise(r => requestAnimationFrame(r)); document.getElementById({panel:?})?.querySelector('a, button')?.focus(); return true;"
                    );
                    spawn(async move {
                        let _ = document::eval(&js).join::<bool>().await;
                    });
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("NavigationMenu parts must be rendered inside NavigationMenu")
}

/// Root: a `<nav>` holding the list of items. Owns the machine and the hover intent.
#[component]
pub fn NavigationMenu(
    /// The top-level items, in display order. Must match the `value`s of the parts.
    items: ReadSignal<Vec<NavigationMenuEntry>>,
    /// Accessible name for the nav.
    #[props(default = "Main".to_string())]
    label: String,
    /// Controlled open panel (`Some(None)` is none open). Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<Option<String>>>,
    /// Initial open panel when uncontrolled.
    default_open: Option<String>,
    /// Milliseconds the pointer rests on a trigger before its panel opens.
    #[props(default = 150)]
    open_delay_ms: u32,
    /// Milliseconds after the pointer leaves before the panel closes.
    #[props(default = 250)]
    close_delay_ms: u32,
    /// Called with the open panel's value, or `None`, whenever it changes or is requested.
    on_open_change: Option<EventHandler<Option<String>>>,
    children: Element,
) -> Element {
    let input = use_memo(move || NavigationMenuInput {
        items: items(),
        open: open(),
        default_open: default_open.clone(),
        open_delay_ms,
        close_delay_ms,
    });
    let mut state = use_signal(|| NavigationMenuMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = NavigationMenuMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let ctx = use_context_provider(|| Ctx {
        id: CopyValue::new(hydroxy_headless::next_id(h::SCOPE)),
        state,
        input,
        focus: FocusMap::new(),
        on_open_change,
    });
    let nav_id = ctx.id.read().clone();
    rsx! {
        nav {
            id: "{nav_id}",
            "aria-label": "{label}",
            "data-scope": h::SCOPE,
            "data-part": "root",
            onpointerleave: move |_| ctx.send(NavigationMenuEvent::Leave),
            onfocusout: move |_| {
                // Focus left the nav (and its panels): close. Wait a frame for the new focus to land.
                let js = format!(
                    "await new Promise(r => requestAnimationFrame(r)); return !document.getElementById({:?})?.contains(document.activeElement);",
                    ctx.id.peek().clone()
                );
                spawn(async move {
                    if let Ok(true) = document::eval(&js).join::<bool>().await {
                        ctx.send(NavigationMenuEvent::Close);
                    }
                });
            },
            ul { "data-scope": h::SCOPE, "data-part": "list", {children} }
        }
    }
}

/// A top-level item with a flyout panel. Children are the panel's content,
/// usually `NavigationMenuPanelLink`s.
#[component]
pub fn NavigationMenuItem(
    /// Unique key. Must match a panel entry in the nav's `items`.
    value: String,
    /// Text of the trigger.
    label: String,
    /// Where the panel sits relative to its trigger. Default below, start-aligned.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let trigger = h::trigger_attrs(&ctx.id.read(), &ctx.state.read(), &value);
    let content = h::content_attrs(&ctx.id.read(), &ctx.state.read(), &value, placement);
    let item_id = h::item_id(&ctx.id.read(), &value);
    // The popover follows the state, whoever changed it.
    let panel = content.id.clone();
    let watched = value.clone();
    use_effect(move || {
        let open = ctx.state.read().open.as_deref() == Some(watched.as_str());
        if open { show_popover(&panel) } else { hide_popover(&panel) }
    });
    let (mounted, clicked, entered, keyed, dismissed, js_panel) =
        (value.clone(), value.clone(), value.clone(), value.clone(), value, content.id.clone());
    rsx! {
        li { "data-scope": h::SCOPE, "data-part": "item",
            button {
                r#type: "button",
                id: item_id,
                "aria-expanded": trigger.aria_expanded.to_string(),
                "aria-controls": trigger.aria_controls.clone(),
                "popovertarget": trigger.aria_controls,
                style: trigger.style,
                "data-scope": h::SCOPE,
                "data-part": "trigger",
                "data-state": trigger.data_state,
                onmounted: move |e| ctx.focus.register(mounted.clone(), e.data()),
                onclick: move |e| {
                    // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                    e.prevent_default();
                    ctx.send(NavigationMenuEvent::Toggle(clicked.clone()));
                },
                onpointerenter: move |_| ctx.send(NavigationMenuEvent::Enter(entered.clone())),
                onkeydown: move |e| {
                    if let Some(event) = h::item_keydown(&e.key().to_string(), &keyed, true) {
                        e.prevent_default();
                        ctx.send(event);
                    }
                },
                "{label}"
                span { "data-scope": h::SCOPE, "data-part": "chevron", Icon { name: IconName::ChevronDown, size: 14 } }
            }
            div {
                id: content.id,
                "popover": "auto",
                "aria-labelledby": content.aria_labelledby,
                style: content.style,
                "data-overlay": "",
                "data-side": content.data_side,
                "data-scope": h::SCOPE,
                "data-part": "content",
                "data-state": content.data_state,
                onpointerenter: move |_| ctx.send(NavigationMenuEvent::PanelEnter),
                ontoggle: move |_| {
                    // The browser may have closed it (Esc, outside press, or another panel opened): ask, then catch up.
                    let js = format!("return document.getElementById({js_panel:?})?.matches(':popover-open') ?? false");
                    let value = dismissed.clone();
                    spawn(async move {
                        if let Ok(false) = document::eval(&js).join::<bool>().await {
                            ctx.send(NavigationMenuEvent::Dismiss(value));
                        }
                    });
                },
                {children}
            }
        }
    }
}

/// A top-level link with no panel.
#[component]
pub fn NavigationMenuLink(
    /// Unique key. Must match a link entry in the nav's `items`.
    value: String,
    /// Where the link goes.
    href: String,
    /// This link is the current page.
    #[props(default)]
    current: bool,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let id = h::item_id(&ctx.id.read(), &value);
    let (mounted, entered, keyed) = (value.clone(), value.clone(), value);
    rsx! {
        li { "data-scope": h::SCOPE, "data-part": "item",
            a {
                id,
                href,
                "aria-current": current.then_some("page"),
                "data-scope": h::SCOPE,
                "data-part": "link",
                onmounted: move |e| ctx.focus.register(mounted.clone(), e.data()),
                onpointerenter: move |_| ctx.send(NavigationMenuEvent::Enter(entered.clone())),
                onkeydown: move |e| {
                    if let Some(event) = h::item_keydown(&e.key().to_string(), &keyed, false) {
                        e.prevent_default();
                        ctx.send(event);
                    }
                },
                {children}
            }
        }
    }
}

/// A link inside a panel: a title with an optional description under it.
#[component]
pub fn NavigationMenuPanelLink(
    /// Where the link goes.
    href: String,
    /// Second line under the title.
    description: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        a { href, "data-scope": h::SCOPE, "data-part": "panel-link",
            span { "data-scope": h::SCOPE, "data-part": "panel-link-title", {children} }
            if let Some(description) = description {
                span { "data-scope": h::SCOPE, "data-part": "panel-link-description", "{description}" }
            }
        }
    }
}
