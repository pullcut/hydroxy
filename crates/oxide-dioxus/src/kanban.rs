//! `KanbanBoard`: columns of cards, moved by drag and drop or by the keyboard
//! (Space to pick up, arrows to move, Space to drop, Esc to cancel).

use dioxus::prelude::*;
use oxide_headless::kanban::{
    self as h, KanbanCommand, KanbanEvent, KanbanInput, KanbanMachine, KanbanState, announcement_text,
};
use oxide_statechart::Machine;

use crate::focus::FocusMap;

pub use oxide_headless::kanban::{KanbanCard, KanbanColumn, KanbanMove, apply_move};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<KanbanState>,
    input: Memo<KanbanInput>,
    live: Signal<String>,
    cards: FocusMap,
    /// A card to focus when it mounts: a keyboard move can re-create it in another column.
    pending_focus: Signal<Option<String>>,
    on_move: Option<EventHandler<KanbanMove>>,
}

impl Ctx {
    fn send(mut self, event: KanbanEvent) {
        let step = KanbanMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                KanbanCommand::Move(mv) => {
                    if let Some(handler) = self.on_move {
                        handler.call(mv);
                    }
                }
                KanbanCommand::FocusCard(card) => {
                    self.cards.focus(&card);
                    self.pending_focus.set(Some(card));
                }
                KanbanCommand::Announce(a) => self.live.set(announcement_text(&a)),
            }
        }
    }
}

/// A board of columns. Cards move within and across columns.
#[component]
pub fn KanbanBoard(
    /// Controlled layout. Pair with `on_move` and [`apply_move`].
    #[props(default)]
    columns: ReadSignal<Option<Vec<KanbanColumn>>>,
    /// Initial layout when uncontrolled.
    #[props(default)]
    default_columns: Vec<KanbanColumn>,
    /// Block every move.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called when a card lands somewhere new.
    on_move: Option<EventHandler<KanbanMove>>,
    /// Renders a card's content. Default: its title.
    card: Option<Callback<KanbanCard, Element>>,
) -> Element {
    let input = use_memo(move || KanbanInput {
        columns: columns(),
        default_columns: default_columns.clone(),
        disabled: disabled(),
    });
    let mut state = use_signal(|| KanbanMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = KanbanMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx {
        state,
        input,
        live: Signal::new(String::new()),
        cards: FocusMap::new(),
        pending_focus: Signal::new(None),
        on_move,
    });

    let s = state.read();
    let i = input.read();
    let preview = s.preview();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-dragging": s.grab.is_some().then_some(""),
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: h::instructions_id(&root), {h::INSTRUCTIONS} }
            for column in preview.iter() {
                {
                    let header = h::header_id(&root, &column.id);
                    let (over, col_id) = (column.id.clone(), column.id.clone());
                    rsx! {
                        section { key: "{column.id}", "data-scope": h::SCOPE, "data-part": "column",
                            h3 { "data-scope": h::SCOPE, "data-part": "header", id: header.clone(),
                                span { "{column.title}" }
                                span { "data-scope": h::SCOPE, "data-part": "count", "{column.cards.len()}" }
                            }
                            ul {
                                role: "list",
                                "aria-labelledby": header,
                                "data-scope": h::SCOPE,
                                "data-part": "list",
                                "data-drop-end": h::drop_at_end(&s, &column.id).then_some(""),
                                ondragover: move |e| {
                                    e.prevent_default();
                                    ctx.send(KanbanEvent::DragOver { column: over.clone(), before: None });
                                },
                                ondrop: move |e| {
                                    e.prevent_default();
                                    ctx.send(KanbanEvent::Drop);
                                },
                                for k in column.cards.iter() {
                                    {
                                        let a = h::card_attrs(&root, &s, &i, &k.id);
                                        let (mounted, started, hovered, keyed) =
                                            (k.id.clone(), k.id.clone(), k.id.clone(), k.id.clone());
                                        let column = col_id.clone();
                                        let content = match card {
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
                                                "data-part": "card",
                                                "data-grabbed": a.grabbed.then_some(""),
                                                "data-drop-before": a.drop_before.then_some(""),
                                                onmounted: move |e| {
                                                    let el = e.data();
                                                    ctx.cards.register(mounted.clone(), el.clone());
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
                                                    ctx.send(KanbanEvent::DragStart(started.clone()));
                                                },
                                                ondragover: move |e| {
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    ctx.send(KanbanEvent::DragOver {
                                                        column: column.clone(),
                                                        before: Some(hovered.clone()),
                                                    });
                                                },
                                                ondragend: move |_| ctx.send(KanbanEvent::Cancel),
                                                onkeydown: move |e| {
                                                    let event = h::card_keydown(&e.key().to_string(), &keyed, &ctx.state.peek());
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
