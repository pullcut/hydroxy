//! `<Toolbar>`: a row of actions and toggles with a single tab stop. Arrow keys
//! move between items, Home and End jump to the ends. Items that don't fit
//! fold into a "More" menu, lowest `priority` first, keeping their order.
//!
//! The "More" menu runs on the menu machine and connector, so it looks and
//! behaves like [`Menu`](crate::menu::Menu); toggles show there as checkable items.

use std::collections::BTreeSet;

use hydroxy_headless::menu::{self as hm, MenuCommand, MenuEntry, MenuEvent, MenuInput, MenuMachine, MenuState};
use hydroxy_headless::overlay::{self, Align, Placement, Side};
use hydroxy_headless::toolbar::{
    self as h, MORE, ToolbarCommand, ToolbarEvent, ToolbarInput, ToolbarMachine, ToolbarState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::js;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::toolbar::{Orientation, ToolbarItem};

const MORE_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::End);

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<ToolbarState>,
    input: Memo<ToolbarInput>,
    on_action: Option<Callback<String>>,
    on_toggle: Option<Callback<(String, bool)>>,
}

impl Ctx {
    fn send(self, event: ToolbarEvent) {
        for command in drive::<ToolbarMachine, _>(self.state, self.input, event) {
            match command {
                ToolbarCommand::Action(v) => {
                    if let Some(cb) = self.on_action {
                        cb.run(v);
                    }
                }
                ToolbarCommand::Toggle(v, on) => {
                    if let Some(cb) = self.on_toggle {
                        cb.run((v, on));
                    }
                }
                ToolbarCommand::Focus(v) => focus_id(&h::item_id(&self.id.get_value(), &v)),
            }
        }
    }
}

/// The "More" menu's machine, bridged to the toolbar's.
#[derive(Clone, Copy)]
struct More {
    bar: Ctx,
    root: StoredValue<String>,
    state: RwSignal<MenuState>,
    input: Memo<MenuInput>,
}

impl More {
    fn send(self, event: MenuEvent) {
        for command in drive::<MenuMachine, _>(self.state, self.input, event) {
            match command {
                MenuCommand::OpenChange(_) => {}
                MenuCommand::Select(v) => self.bar.send(ToolbarEvent::Press(v)),
                MenuCommand::FocusTrigger => focus_id(&h::item_id(&self.bar.id.get_value(), MORE)),
            }
        }
    }
}

/// A toolbar.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Toolbar(
    /// What the toolbar is for, such as "Text formatting". Its accessible name.
    #[prop(into)]
    label: String,
    /// The items, in order. Group them with [`ToolbarItem::group`].
    #[prop(into)]
    items: Signal<Vec<ToolbarItem>>,
    /// Row or column. Only rows overflow into "More".
    #[prop(optional)]
    orientation: Orientation,
    /// Controlled pressed toggles. Pair with `on_toggle`.
    #[prop(optional, into)]
    pressed: MaybeProp<BTreeSet<String>>,
    /// Toggles pressed at first, when uncontrolled.
    #[prop(optional, into)]
    default_pressed: Vec<String>,
    /// Called with an action's value.
    #[prop(optional, into)]
    on_action: Option<Callback<String>>,
    /// Called with a toggle's value and its new state.
    #[prop(optional, into)]
    on_toggle: Option<Callback<(String, bool)>>,
) -> impl IntoView {
    let default_pressed: BTreeSet<String> = default_pressed.into_iter().collect();
    let input = Memo::new(move |_| ToolbarInput {
        items: h::entries(&items.get()),
        pressed: pressed.get(),
        default_pressed: default_pressed.clone(),
    });
    let state = RwSignal::new(input.with_untracked(ToolbarMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ToolbarMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let ctx = Ctx { id, state, input, on_action, on_toggle };

    // Measure only rows: a column never overflows.
    let node = NodeRef::<leptos::html::Div>::new();
    let watching = StoredValue::new_local(None::<js::Watch>);
    Effect::new(move |_| {
        if orientation != Orientation::Horizontal || node.get().is_none() || watching.with_value(Option::is_some) {
            return;
        }
        let w = js::watch(h::MEASURE_JS, &id.get_value(), move |report| {
            if let Some(event) = h::parse_measure(&report) {
                ctx.send(event);
            }
        });
        js::keep(watching, w);
    });

    // The "More" menu lists hidden items in toolbar order.
    let menu_input = Memo::new(move |_| {
        let hidden = state.with(|s| s.hidden.clone());
        let entries = items.with(|list| {
            list.iter()
                .filter(|i| hidden.contains(&i.value))
                .map(|i| MenuEntry { value: i.value.clone(), disabled: i.disabled })
                .collect()
        });
        MenuInput { items: entries, open: None, default_open: false }
    });
    let menu_root = StoredValue::new(h::more_id(&id.get_value()));
    let more = More { bar: ctx, root: menu_root, state: RwSignal::new(MenuState::default()), input: menu_input };
    Effect::new(move |_| {
        let (open, highlighted) = more.state.with(|s| (s.open, s.highlighted.clone()));
        let root = menu_root.get_value();
        if open {
            show_popover(&hm::content_id(&root));
            if let Some(v) = highlighted {
                focus_id(&hm::item_id(&root, &v));
            }
        } else {
            hide_popover(&hm::content_id(&root));
        }
    });

    let tabindex = move |v: &str| input.with(|i| state.with(|s| h::tabindex(s, i, v)));
    let render = move || {
        let list = items.get();
        h::groups(&list)
            .into_iter()
            .enumerate()
            .map(|(gi, (_, run))| {
                let values: Vec<String> = run.iter().map(|&i| list[i].value.clone()).collect();
                let before: Vec<String> = list.iter().take(run[0]).map(|i| i.value.clone()).collect();
                // A separator shows only between groups that both have something showing.
                let show_sep = move || {
                    state.with(|s| {
                        values.iter().any(|v| !s.hidden.contains(v)) && before.iter().any(|v| !s.hidden.contains(v))
                    })
                };
                let buttons = run.into_iter().map(|i| item_view(ctx, i, list[i].clone(), tabindex)).collect_view();
                view! {
                    {(gi > 0).then(|| view! {
                        <div
                            role="separator"
                            aria-orientation=if orientation == Orientation::Horizontal { "vertical" } else { "horizontal" }
                            data-scope=h::SCOPE
                            data-part="separator"
                            hidden=move || !show_sep()
                        ></div>
                    })}
                    <div role="group" data-scope=h::SCOPE data-part="group">{buttons}</div>
                }
            })
            .collect_view()
    };
    let groups_count = move || items.with(|l| h::groups(l).len());

    view! {
        <div
            node_ref=node
            id=id.get_value()
            role="toolbar"
            aria-label=label
            aria-orientation=orientation.as_str()
            data-orientation=orientation.as_str()
            data-groups=groups_count
            data-scope=h::SCOPE
            data-part="root"
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key(), orientation) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {render}
            {more_view(more, tabindex, items)}
        </div>
    }
}

fn item_view(
    ctx: Ctx,
    index: usize,
    item: ToolbarItem,
    tabindex: impl Fn(&str) -> &'static str + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let root = ctx.id.get_value();
    let value = StoredValue::new(item.value.clone());
    let pressed = move || item.toggle.then(|| ctx.state.with(|s| s.pressed.contains(&value.get_value())).to_string());
    let hidden = move || ctx.state.with(|s| s.hidden.contains(&value.get_value()));
    let icon_only = item.icon.is_some();
    let text = item.label.clone();
    view! {
        <button
            type="button"
            id=h::item_id(&root, &item.value)
            data-scope=h::SCOPE
            data-part="item"
            data-index=index
            data-value=item.value.clone()
            data-icon-only=icon_only.then_some("")
            tabindex=move || value.with_value(|v| tabindex(v))
            aria-pressed=pressed
            aria-label=icon_only.then_some(item.label.clone())
            title=item.label
            disabled=item.disabled
            hidden=hidden
            on:click=move |_| ctx.send(ToolbarEvent::Press(value.get_value()))
            on:focus=move |_| ctx.send(ToolbarEvent::Focus(value.get_value()))
        >
            {match item.icon {
                Some(icon) => view! { <Icon name=icon size=16 /> }.into_any(),
                None => view! { <span data-scope=h::SCOPE data-part="item-label">{text}</span> }.into_any(),
            }}
        </button>
    }
}

fn more_view(
    more: More,
    tabindex: impl Fn(&str) -> &'static str + Copy + Send + Sync + 'static,
    items: Signal<Vec<ToolbarItem>>,
) -> impl IntoView {
    let bar = more.bar;
    let trigger = h::item_id(&bar.id.get_value(), MORE);
    let root = more.root.get_value();
    let content = hm::content_id(&root);
    let any_hidden = move || bar.state.with(|s| !s.hidden.is_empty());
    let open = move || more.state.with(|s| s.open);
    let hidden_items = move || {
        let hidden = bar.state.with(|s| s.hidden.clone());
        items.get().into_iter().filter(|i| hidden.contains(&i.value)).collect::<Vec<_>>()
    };
    view! {
        <button
            type="button"
            id=trigger.clone()
            data-scope=h::SCOPE
            data-part="more"
            data-state=move || if open() { "open" } else { "closed" }
            tabindex=move || tabindex(MORE)
            aria-label="More"
            title="More"
            aria-haspopup="menu"
            aria-expanded=move || open().to_string()
            aria-controls=content.clone()
            popovertarget=content.clone()
            style=overlay::anchor_style(&root)
            hidden=move || !any_hidden()
            on:click=move |ev| {
                ev.prevent_default();
                more.send(MenuEvent::Toggle);
            }
            on:focus=move |_| bar.send(ToolbarEvent::Focus(MORE.to_string()))
            on:keydown=move |ev| {
                if let Some(event) = hm::trigger_keydown(&ev.key()) {
                    ev.prevent_default();
                    ev.stop_propagation();
                    more.send(event);
                }
            }
        >
            <Icon name=IconName::Ellipsis size=16 />
        </button>
        <div
            id=content
            role="menu"
            popover="auto"
            aria-labelledby=trigger
            style=overlay::floating_style(&root, MORE_PLACEMENT, hm::GAP_PX)
            data-overlay=""
            data-side=MORE_PLACEMENT.side_str()
            data-scope=hm::SCOPE
            data-part="content"
            data-state=move || if open() { "open" } else { "closed" }
            on:keydown=move |ev| {
                ev.stop_propagation();
                if let Some((event, prevent)) = hm::content_keydown(&ev.key()) {
                    if prevent {
                        ev.prevent_default();
                    }
                    more.send(event);
                }
            }
            on:toggle=move |ev| {
                let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                if !showing {
                    more.send(MenuEvent::Close { restore_focus: false });
                }
            }
        >
            <For each=hidden_items key=|i| i.value.clone() let:item>
                {menu_item(more, item)}
            </For>
        </div>
    }
}

fn menu_item(more: More, item: ToolbarItem) -> impl IntoView {
    let root = more.root.get_value();
    let value = StoredValue::new(item.value.clone());
    let highlighted = move || more.state.with(|s| s.highlighted.as_deref() == Some(value.get_value().as_str()));
    let checked = move || more.bar.state.with(|s| s.pressed.contains(&value.get_value()));
    let toggle = item.toggle;
    view! {
        <button
            type="button"
            id=hm::item_id(&root, &item.value)
            role=if toggle { "menuitemcheckbox" } else { "menuitem" }
            aria-checked=move || toggle.then(|| checked().to_string())
            tabindex="-1"
            disabled=item.disabled
            data-scope=hm::SCOPE
            data-part="item"
            data-highlighted=move || highlighted().then_some("")
            on:click=move |_| more.send(MenuEvent::Select(value.get_value()))
            on:pointermove=move |_| more.send(MenuEvent::Highlight(value.get_value()))
            on:focus=move |_| more.send(MenuEvent::Highlight(value.get_value()))
        >
            {item.icon.map(|i| view! { <Icon name=i size=14 /> })}
            <span data-scope=hm::SCOPE data-part="item-label">{item.label}</span>
            {move || match (toggle && checked(), item.shortcut.clone()) {
                (true, _) => Some(view! { <span data-scope=hm::SCOPE data-part="item-hint"><Icon name=IconName::Check size=14 /></span> }.into_any()),
                (false, Some(s)) => Some(view! { <span data-scope=hm::SCOPE data-part="item-hint">{s}</span> }.into_any()),
                (false, None) => None,
            }}
        </button>
    }
}
