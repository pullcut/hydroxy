//! `Toolbar`: a row of actions and toggles with a single tab stop. Arrow keys
//! move between items, Home and End jump to the ends. Items that don't fit
//! fold into a "More" menu, lowest `priority` first, keeping their order.
//!
//! The "More" menu runs on the menu machine and connector, so it looks and
//! behaves like [`Menu`](crate::menu::Menu); toggles show there as checkable items.

use std::collections::BTreeSet;

use dioxus::prelude::*;
use hydroxy_headless::menu::{self as hm, MenuCommand, MenuEntry, MenuEvent, MenuInput, MenuMachine, MenuState};
use hydroxy_headless::overlay::{self, Align, Placement, Side};
use hydroxy_headless::toolbar::{self as h, MORE, ToolbarCommand, ToolbarEvent, ToolbarInput, ToolbarMachine, ToolbarState};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::js;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::toolbar::{Orientation, ToolbarItem};

const MORE_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::End);

fn focus_id(id: &str) {
    js::run(&[("id", id)], "document.getElementById(id)?.focus();");
}

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<ToolbarState>,
    input: Memo<ToolbarInput>,
    on_action: Option<EventHandler<String>>,
    on_toggle: Option<EventHandler<(String, bool)>>,
}

impl Ctx {
    fn send(mut self, event: ToolbarEvent) {
        let step = ToolbarMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state
            && *self.state.peek() != next
        {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                ToolbarCommand::Action(v) => {
                    if let Some(handler) = self.on_action {
                        handler.call(v);
                    }
                }
                ToolbarCommand::Toggle(v, on) => {
                    if let Some(handler) = self.on_toggle {
                        handler.call((v, on));
                    }
                }
                ToolbarCommand::Focus(v) => focus_id(&h::item_id(&self.id.peek(), &v)),
            }
        }
    }
}

/// The "More" menu's machine, bridged to the toolbar's.
#[derive(Clone, Copy)]
struct More {
    bar: Ctx,
    state: Signal<MenuState>,
    input: Memo<MenuInput>,
}

impl More {
    fn send(mut self, event: MenuEvent) {
        let step = MenuMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state
            && *self.state.peek() != next
        {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                MenuCommand::OpenChange(_) => {}
                MenuCommand::Select(v) => self.bar.send(ToolbarEvent::Press(v)),
                MenuCommand::FocusTrigger => focus_id(&h::item_id(&self.bar.id.peek(), MORE)),
            }
        }
    }
}

/// A toolbar.
#[component]
pub fn Toolbar(
    /// What the toolbar is for, such as "Text formatting". Its accessible name.
    label: String,
    /// The items, in order. Group them with [`ToolbarItem::group`].
    items: ReadSignal<Vec<ToolbarItem>>,
    /// Row or column. Only rows overflow into "More".
    #[props(default)]
    orientation: Orientation,
    /// Controlled pressed toggles. Pair with `on_toggle`.
    #[props(default)]
    pressed: ReadSignal<Option<BTreeSet<String>>>,
    /// Toggles pressed at first, when uncontrolled.
    #[props(default)]
    default_pressed: Vec<String>,
    /// Called with an action's value.
    on_action: Option<EventHandler<String>>,
    /// Called with a toggle's value and its new state.
    on_toggle: Option<EventHandler<(String, bool)>>,
) -> Element {
    let default_pressed: BTreeSet<String> = default_pressed.into_iter().collect();
    let input = use_memo(move || ToolbarInput {
        items: h::entries(&items.read()),
        pressed: pressed(),
        default_pressed: default_pressed.clone(),
    });
    let mut state = use_signal(|| ToolbarMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ToolbarMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let id = use_hook(|| CopyValue::new(hydroxy_headless::next_id(h::SCOPE)));
    let ctx = use_hook(|| Ctx { id, state, input, on_action, on_toggle });
    let root = id.peek().clone();
    let watch = js::use_watch(h::MEASURE_JS, root.clone(), move |report| {
        if let Some(event) = h::parse_measure(&report) {
            ctx.send(event);
        }
    });

    let menu_input = use_memo(move || {
        let hidden = state.read().hidden.clone();
        let entries = items
            .read()
            .iter()
            .filter(|i| hidden.contains(&i.value))
            .map(|i| MenuEntry { value: i.value.clone(), disabled: i.disabled })
            .collect();
        MenuInput { items: entries, open: None, default_open: false }
    });
    let menu_root = use_hook(|| CopyValue::new(format!("{root}-more")));
    let menu_state = use_signal(MenuState::default);
    let more = More { bar: ctx, state: menu_state, input: menu_input };
    use_effect(move || {
        let s = menu_state.read().clone();
        let r = menu_root.peek().clone();
        if s.open {
            show_popover(&hm::content_id(&r));
            if let Some(v) = s.highlighted {
                focus_id(&hm::item_id(&r, &v));
            }
        } else {
            hide_popover(&hm::content_id(&r));
        }
    });

    let list = items();
    let s = state.read().clone();
    let inp = input.read().clone();
    let groups = h::groups(&list);
    let group_count = groups.len();
    let horizontal = orientation == Orientation::Horizontal;
    let sep_orientation = if horizontal { "vertical" } else { "horizontal" };

    let mr = menu_root.peek().clone();
    let trigger = h::item_id(&root, MORE);
    let content = hm::content_id(&mr);
    let toggle_id = content.clone();
    let open = menu_state.read().open;
    let highlighted = menu_state.read().highlighted.clone();
    let hidden_items: Vec<&ToolbarItem> = list.iter().filter(|i| s.hidden.contains(&i.value)).collect();

    rsx! {
        div {
            id: "{root}",
            role: "toolbar",
            "aria-label": label,
            "aria-orientation": orientation.as_str(),
            "data-orientation": orientation.as_str(),
            "data-groups": "{group_count}",
            "data-scope": h::SCOPE,
            "data-part": "root",
            onmounted: move |_| {
                if horizontal {
                    let _ = watch.send("start");
                }
            },
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string(), orientation) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            for (gi, (_, run)) in groups.into_iter().enumerate() {
                if gi > 0 {
                    div {
                        role: "separator",
                        "aria-orientation": sep_orientation,
                        "data-scope": h::SCOPE,
                        "data-part": "separator",
                        hidden: !(run.iter().any(|&i| !s.hidden.contains(&list[i].value))
                            && list[..run[0]].iter().any(|it| !s.hidden.contains(&it.value))),
                    }
                }
                div { role: "group", "data-scope": h::SCOPE, "data-part": "group",
                    for i in run.iter().copied() {
                        {item_view(ctx, i, &list[i], &s, &inp, &root)}
                    }
                }
            }
            button {
                r#type: "button",
                id: trigger.clone(),
                "data-scope": h::SCOPE,
                "data-part": "more",
                "data-state": if open { "open" } else { "closed" },
                tabindex: h::tabindex(&s, &inp, MORE),
                "aria-label": "More",
                title: "More",
                "aria-haspopup": "menu",
                "aria-expanded": open.to_string(),
                "aria-controls": content.clone(),
                "popovertarget": content.clone(),
                style: overlay::anchor_style(&mr),
                hidden: s.hidden.is_empty(),
                onclick: move |e| {
                    e.prevent_default();
                    more.send(MenuEvent::Toggle);
                },
                onfocus: move |_| ctx.send(ToolbarEvent::Focus(MORE.to_string())),
                onkeydown: move |e| {
                    if let Some(event) = hm::trigger_keydown(&e.key().to_string()) {
                        e.prevent_default();
                        e.stop_propagation();
                        more.send(event);
                    }
                },
                Icon { name: IconName::Ellipsis, size: 16 }
            }
            div {
                id: content.clone(),
                role: "menu",
                "popover": "auto",
                "aria-labelledby": trigger,
                style: overlay::floating_style(&mr, MORE_PLACEMENT, hm::GAP_PX),
                "data-overlay": "",
                "data-side": MORE_PLACEMENT.side_str(),
                "data-scope": hm::SCOPE,
                "data-part": "content",
                "data-state": if open { "open" } else { "closed" },
                onkeydown: move |e| {
                    e.stop_propagation();
                    if let Some((event, prevent)) = hm::content_keydown(&e.key().to_string()) {
                        if prevent {
                            e.prevent_default();
                        }
                        more.send(event);
                    }
                },
                ontoggle: move |_| {
                    let js = format!("return document.getElementById({toggle_id:?})?.matches(':popover-open') ?? false");
                    spawn(async move {
                        if let Ok(false) = document::eval(&js).join::<bool>().await {
                            more.send(MenuEvent::Close { restore_focus: false });
                        }
                    });
                },
                for item in hidden_items {
                    {menu_item(&more, item, &s, highlighted.as_deref(), &mr)}
                }
            }
        }
    }
}

fn item_view(ctx: Ctx, index: usize, item: &ToolbarItem, s: &ToolbarState, inp: &ToolbarInput, root: &str) -> Element {
    let icon_only = item.icon.is_some();
    let pressed = item.toggle.then(|| s.pressed.contains(&item.value).to_string());
    let (press, focus) = (item.value.clone(), item.value.clone());
    rsx! {
        button {
            key: "{item.value}",
            r#type: "button",
            id: h::item_id(root, &item.value),
            "data-scope": h::SCOPE,
            "data-part": "item",
            "data-index": "{index}",
            "data-value": item.value.clone(),
            "data-icon-only": icon_only.then_some(""),
            tabindex: h::tabindex(s, inp, &item.value),
            "aria-pressed": pressed,
            "aria-label": icon_only.then(|| item.label.clone()),
            title: item.label.clone(),
            disabled: item.disabled,
            hidden: s.hidden.contains(&item.value),
            onclick: move |_| ctx.send(ToolbarEvent::Press(press.clone())),
            onfocus: move |_| ctx.send(ToolbarEvent::Focus(focus.clone())),
            if let Some(icon) = item.icon {
                Icon { name: icon, size: 16 }
            } else {
                span { "data-scope": h::SCOPE, "data-part": "item-label", "{item.label}" }
            }
        }
    }
}

fn menu_item(more: &More, item: &ToolbarItem, s: &ToolbarState, highlighted: Option<&str>, root: &str) -> Element {
    let more = *more;
    let checked = s.pressed.contains(&item.value);
    let toggle = item.toggle;
    let (select, hover, focus) = (item.value.clone(), item.value.clone(), item.value.clone());
    rsx! {
        button {
            key: "{item.value}",
            r#type: "button",
            id: hm::item_id(root, &item.value),
            role: if toggle { "menuitemcheckbox" } else { "menuitem" },
            "aria-checked": toggle.then(|| checked.to_string()),
            tabindex: "-1",
            disabled: item.disabled,
            "data-scope": hm::SCOPE,
            "data-part": "item",
            "data-highlighted": (highlighted == Some(item.value.as_str())).then_some(""),
            onclick: move |_| more.send(MenuEvent::Select(select.clone())),
            onpointermove: move |_| more.send(MenuEvent::Highlight(hover.clone())),
            onfocus: move |_| more.send(MenuEvent::Highlight(focus.clone())),
            if let Some(i) = item.icon { Icon { name: i, size: 14 } }
            span { "data-scope": hm::SCOPE, "data-part": "item-label", "{item.label}" }
            if toggle && checked {
                span { "data-scope": hm::SCOPE, "data-part": "item-hint", Icon { name: IconName::Check, size: 14 } }
            } else if let Some(sc) = item.shortcut.clone() {
                span { "data-scope": hm::SCOPE, "data-part": "item-hint", "{sc}" }
            }
        }
    }
}
