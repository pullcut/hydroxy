//! `<KanbanBoard>`: columns of cards, moved by native drag and drop or by the
//! keyboard (Space to pick up, arrows to move, Space to drop, Esc to cancel).

use leptos::either::Either;
use leptos::prelude::*;

use crate::render_fn::RenderFn;
use oxide_headless::kanban::{
    self as h, KanbanCommand, KanbanEvent, KanbanInput, KanbanMachine, KanbanState, announcement_text,
};
use oxide_statechart::Machine;

use crate::focus::focus_id;

pub use oxide_headless::kanban::{KanbanCard, KanbanColumn, KanbanMove, apply_move};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<KanbanState>,
    input: Memo<KanbanInput>,
    live: RwSignal<String>,
    on_move: Option<Callback<KanbanMove>>,
}

impl Ctx {
    fn send(self, event: KanbanEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| KanbanMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                KanbanCommand::Move(mv) => {
                    if let Some(cb) = self.on_move {
                        cb.run(mv);
                    }
                }
                KanbanCommand::FocusCard(card) => {
                    // The card may re-render in another column; focus once the DOM has caught up.
                    let target = h::card_id(&self.id.get_value(), &card);
                    set_timeout(move || focus_id(&target), std::time::Duration::ZERO);
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
    #[prop(optional, into)]
    columns: MaybeProp<Vec<KanbanColumn>>,
    /// Initial layout when uncontrolled.
    #[prop(optional, into)]
    default_columns: Vec<KanbanColumn>,
    /// Block every move.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called when a card lands somewhere new.
    #[prop(optional, into)]
    on_move: Option<Callback<KanbanMove>>,
    /// Renders a card's content. Default: its title.
    #[prop(optional, into)]
    card: Option<RenderFn<KanbanCard>>,
) -> impl IntoView {
    let input = Memo::new(move |_| KanbanInput {
        columns: columns.get(),
        default_columns: default_columns.clone(),
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(KanbanMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| KanbanMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx {
        id: StoredValue::new(crate::id::next_id(h::SCOPE)),
        state,
        input,
        live: RwSignal::new(String::new()),
        on_move,
    };
    let preview = Memo::new(move |_| state.with(KanbanState::preview));
    let column_ids = Memo::new(move |_| preview.with(|cs| cs.iter().map(|c| c.id.clone()).collect::<Vec<_>>()));
    let root = ctx.id.get_value();

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-dragging=move || state.with(|s| s.grab.is_some()).then_some("")
        >
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root)>{h::INSTRUCTIONS}</p>
            <For
                each=move || column_ids.get()
                key=String::clone
                children=move |column| column_view(ctx, preview, column, card.clone())
            />
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || ctx.live.get()}
            </div>
        </div>
    }
}

fn column_view(
    ctx: Ctx,
    preview: Memo<Vec<KanbanColumn>>,
    column: String,
    render: Option<RenderFn<KanbanCard>>,
) -> impl IntoView {
    let col = {
        let column = column.clone();
        Memo::new(move |_| preview.with(|cs| cs.iter().find(|c| c.id == column).cloned()))
    };
    let header = h::header_id(&ctx.id.get_value(), &column);
    let at_end = {
        let column = column.clone();
        move || ctx.state.with(|s| h::drop_at_end(s, &column))
    };
    let over = column.clone();
    view! {
        <section data-scope=h::SCOPE data-part="column">
            <h3 data-scope=h::SCOPE data-part="header" id=header.clone()>
                <span>{move || col.with(|c| c.as_ref().map(|c| c.title.clone()))}</span>
                <span data-scope=h::SCOPE data-part="count">
                    {move || col.with(|c| c.as_ref().map_or(0, |c| c.cards.len()))}
                </span>
            </h3>
            <ul
                role="list"
                aria-labelledby=header
                data-scope=h::SCOPE
                data-part="list"
                data-drop-end=move || at_end().then_some("")
                on:dragover=move |ev| {
                    ev.prevent_default();
                    ctx.send(KanbanEvent::DragOver { column: over.clone(), before: None });
                }
                on:drop=move |ev| {
                    ev.prevent_default();
                    ctx.send(KanbanEvent::Drop);
                }
            >
                <For
                    each=move || col.with(|c| c.as_ref().map(|c| c.cards.clone()).unwrap_or_default())
                    key=|k| (k.id.clone(), k.title.clone())
                    children=move |k| card_view(ctx, column.clone(), k, render.clone())
                />
            </ul>
        </section>
    }
}

fn card_view(ctx: Ctx, column: String, card: KanbanCard, render: Option<RenderFn<KanbanCard>>) -> impl IntoView {
    let id = card.id.clone();
    let attrs = {
        let id = id.clone();
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::card_attrs(&ctx.id.get_value(), s, i, &id))))
    };
    let (started, over, keyed) = (id.clone(), id.clone(), id);
    let content = match render {
        Some(f) => Either::Left(f.run(card)),
        None => Either::Right(card.title),
    };
    view! {
        <li
            role="listitem"
            id=move || attrs.get().id
            draggable=move || if attrs.get().movable { "true" } else { "false" }
            tabindex=move || attrs.get().movable.then_some(0)
            aria-describedby=move || attrs.get().aria_describedby
            data-scope=h::SCOPE
            data-part="card"
            data-grabbed=move || attrs.get().grabbed.then_some("")
            data-drop-before=move || attrs.get().drop_before.then_some("")
            on:dragstart=move |ev| {
                if let Some(dt) = ev.data_transfer() {
                    // Firefox only starts a drag that carries data.
                    let _ = dt.set_data("text/plain", &started);
                    dt.set_effect_allowed("move");
                }
                ctx.send(KanbanEvent::DragStart(started.clone()));
            }
            on:dragover=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                ctx.send(KanbanEvent::DragOver { column: column.clone(), before: Some(over.clone()) });
            }
            on:dragend=move |_| ctx.send(KanbanEvent::Cancel)
            on:keydown=move |ev| {
                let event = ctx.state.with_untracked(|s| h::card_keydown(&ev.key(), &keyed, s));
                if let Some(event) = event {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {content}
        </li>
    }
}
