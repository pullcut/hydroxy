//! `SortableList`: a vertical list reordered by drag and drop or by the
//! keyboard (Space to pick up, arrows to move, Space to drop, Esc to cancel).

use dioxus::prelude::*;
use hydroxy_headless::sortable_list::{
    self as h, SortItem, SortableCommand, SortableEvent, SortableInput, SortableMachine, SortableState,
    announcement_text,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;

pub use hydroxy_headless::sortable_list::{SortItem as Item, moved};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<SortableState>,
    input: Memo<SortableInput>,
    live: Signal<String>,
    items: FocusMap,
    /// An item to focus when it mounts: a keyboard move can re-create it.
    pending_focus: Signal<Option<String>>,
    on_reorder: Option<EventHandler<Vec<SortItem>>>,
}

impl Ctx {
    fn send(mut self, event: SortableEvent) {
        for command in drive::<SortableMachine>(self.state, self.input, event) {
            match command {
                SortableCommand::Reorder(items) => {
                    if let Some(handler) = self.on_reorder {
                        handler.call(items);
                    }
                }
                SortableCommand::FocusItem(item) => {
                    self.items.focus(&item);
                    self.pending_focus.set(Some(item));
                }
                SortableCommand::Announce(a) => self.live.set(announcement_text(&a)),
            }
        }
    }
}

/// A list whose items can be reordered.
#[component]
pub fn SortableList(
    /// Accessible name for the list.
    label: String,
    /// Controlled order. Pair with `on_reorder`.
    #[props(default)]
    items: ReadSignal<Option<Vec<SortItem>>>,
    /// Initial order when uncontrolled.
    #[props(default)]
    default_items: Vec<SortItem>,
    /// Block every move.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the full new order when an item lands somewhere new.
    on_reorder: Option<EventHandler<Vec<SortItem>>>,
    /// Renders an item's content. Default: its title.
    item: Option<Callback<SortItem, Element>>,
) -> Element {
    let input =
        use_memo(move || SortableInput { items: items(), default_items: default_items.clone(), disabled: disabled() });
    let mut state = use_signal(|| SortableMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = SortableMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx {
        state,
        input,
        live: Signal::new(String::new()),
        items: FocusMap::new(),
        pending_focus: Signal::new(None),
        on_reorder,
    });

    let s = state.read();
    let off = input.read().disabled;
    let preview = h::preview(&s);

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-dragging": s.grab.is_some().then_some(""),
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: h::instructions_id(&root), {h::INSTRUCTIONS} }
            ul {
                role: "list",
                "aria-label": label,
                "data-scope": h::SCOPE,
                "data-part": "list",
                "data-drop-end": h::drop_at_end(&s).then_some(""),
                ondragover: move |e| {
                    e.prevent_default();
                    ctx.send(SortableEvent::DragOver { column: h::COLUMN.into(), before: None });
                },
                ondrop: move |e| {
                    e.prevent_default();
                    ctx.send(SortableEvent::Drop);
                },
                for k in preview.iter() {
                    {
                        let a = h::item_attrs(&root, &s, off, &k.id);
                        let (mounted, started, hovered, keyed) =
                            (k.id.clone(), k.id.clone(), k.id.clone(), k.id.clone());
                        let content = match item {
                            Some(render) => render.call(k.clone()),
                            None => rsx! { "{k.title}" },
                        };
                        rsx! {
                            li {
                                key: "{k.id}",
                                role: "listitem",
                                id: a.id,
                                draggable: if a.movable { "true" } else { "false" },
                                tabindex: a.movable.then_some("0"),
                                "aria-describedby": a.aria_describedby,
                                "data-scope": h::SCOPE,
                                "data-part": "item",
                                "data-grabbed": a.grabbed.then_some(""),
                                "data-drop-before": a.drop_before.then_some(""),
                                onmounted: move |e| {
                                    let el = e.data();
                                    ctx.items.register(mounted.clone(), el.clone());
                                    let mut pending = ctx.pending_focus;
                                    if pending.peek().as_deref() == Some(mounted.as_str()) {
                                        pending.set(None);
                                        spawn(async move {
                                            let _ = el.set_focus(true).await;
                                        });
                                    }
                                },
                                ondragstart: move |e| {
                                    let dt = e.data_transfer();
                                    // Firefox only starts a drag that carries data.
                                    let _ = dt.set_data("text/plain", &started);
                                    dt.set_effect_allowed("move");
                                    ctx.send(SortableEvent::DragStart(started.clone()));
                                },
                                ondragover: move |e| {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    ctx.send(SortableEvent::DragOver {
                                        column: h::COLUMN.into(),
                                        before: Some(hovered.clone()),
                                    });
                                },
                                ondragend: move |_| ctx.send(SortableEvent::Cancel),
                                onkeydown: move |e| {
                                    let event = h::item_keydown(&e.key().to_string(), &keyed, &ctx.state.peek());
                                    if let Some(event) = event {
                                        e.prevent_default();
                                        ctx.send(event);
                                    }
                                },
                                {content}
                            }
                        }
                    }
                }
            }
            div {
                "data-scope": h::SCOPE,
                "data-part": "live",
                role: "status",
                "aria-live": "polite",
                "aria-atomic": "true",
                "{ctx.live}"
            }
        }
    }
}
