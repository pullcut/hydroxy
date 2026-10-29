//! `<SortableList>`: a vertical list reordered by native drag and drop or by
//! the keyboard (Space to pick up, arrows to move, Space to drop, Esc to cancel).

use leptos::either::Either;
use leptos::prelude::*;

use crate::drive::drive;
use crate::render_fn::RenderFn;
use hydroxy_headless::sortable_list::{
    self as h, SortItem, SortableCommand, SortableEvent, SortableInput, SortableMachine, SortableState,
    announcement_text,
};
use hydroxy_statechart::Machine;

use crate::focus::focus_soon;

pub use hydroxy_headless::sortable_list::{SortItem as Item, moved};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<SortableState>,
    input: Memo<SortableInput>,
    live: RwSignal<String>,
    on_reorder: Option<Callback<Vec<SortItem>>>,
}

impl Ctx {
    fn send(self, event: SortableEvent) {
        for command in drive::<SortableMachine, _>(self.state, self.input, event) {
            match command {
                SortableCommand::Reorder(items) => {
                    if let Some(cb) = self.on_reorder {
                        cb.run(items);
                    }
                }
                SortableCommand::FocusItem(item) => {
                    // The item may re-render in its new place; focus once the DOM has caught up.
                    let target = h::item_id(&self.id.get_value(), &item);
                    focus_soon(&target);
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
    #[prop(into)]
    label: String,
    /// Controlled order. Pair with `on_reorder`.
    #[prop(optional, into)]
    items: MaybeProp<Vec<SortItem>>,
    /// Initial order when uncontrolled.
    #[prop(optional, into)]
    default_items: Vec<SortItem>,
    /// Block every move.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called with the full new order when an item lands somewhere new.
    #[prop(optional, into)]
    on_reorder: Option<Callback<Vec<SortItem>>>,
    /// Renders an item's content. Default: its title.
    #[prop(optional, into)]
    item: Option<RenderFn<SortItem>>,
) -> impl IntoView {
    let input = Memo::new(move |_| SortableInput {
        items: items.get(),
        default_items: default_items.clone(),
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(SortableMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| SortableMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx {
        id: StoredValue::new(crate::id::next_id(h::SCOPE)),
        state,
        input,
        live: RwSignal::new(String::new()),
        on_reorder,
    };
    let preview = Memo::new(move |_| state.with(h::preview));
    let root = ctx.id.get_value();

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-dragging=move || state.with(|s| s.grab.is_some()).then_some("")
        >
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root)>{h::INSTRUCTIONS}</p>
            <ul
                role="list"
                aria-label=label
                data-scope=h::SCOPE
                data-part="list"
                data-drop-end=move || state.with(h::drop_at_end).then_some("")
                on:dragover=move |ev| {
                    ev.prevent_default();
                    ctx.send(SortableEvent::DragOver { column: h::COLUMN.into(), before: None });
                }
                on:drop=move |ev| {
                    ev.prevent_default();
                    ctx.send(SortableEvent::Drop);
                }
            >
                <For
                    each=move || preview.get()
                    key=|k| (k.id.clone(), k.title.clone())
                    children=move |k| item_view(ctx, k, item.clone())
                />
            </ul>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || ctx.live.get()}
            </div>
        </div>
    }
}

fn item_view(ctx: Ctx, item: SortItem, render: Option<RenderFn<SortItem>>) -> impl IntoView {
    let id = item.id.clone();
    let attrs = {
        let id = id.clone();
        Memo::new(move |_| {
            let disabled = ctx.input.with(|i| i.disabled);
            ctx.state.with(|s| h::item_attrs(&ctx.id.get_value(), s, disabled, &id))
        })
    };
    let (started, over, keyed) = (id.clone(), id.clone(), id);
    let content = match render {
        Some(f) => Either::Left(f.run(item)),
        None => Either::Right(item.title),
    };
    view! {
        <li
            role="listitem"
            id=move || attrs.with(|x| x.id.clone())
            draggable=move || if attrs.with(|x| x.movable) { "true" } else { "false" }
            tabindex=move || attrs.with(|x| x.movable).then_some(0)
            aria-describedby=move || attrs.with(|x| x.aria_describedby.clone())
            data-scope=h::SCOPE
            data-part="item"
            data-grabbed=move || attrs.with(|x| x.grabbed).then_some("")
            data-drop-before=move || attrs.with(|x| x.drop_before).then_some("")
            on:dragstart=move |ev| {
                if let Some(dt) = ev.data_transfer() {
                    // Firefox only starts a drag that carries data.
                    let _ = dt.set_data("text/plain", &started);
                    dt.set_effect_allowed("move");
                }
                ctx.send(SortableEvent::DragStart(started.clone()));
            }
            on:dragover=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                ctx.send(SortableEvent::DragOver { column: h::COLUMN.into(), before: Some(over.clone()) });
            }
            on:dragend=move |_| ctx.send(SortableEvent::Cancel)
            on:keydown=move |ev| {
                let event = ctx.state.with_untracked(|s| h::item_keydown(&ev.key(), &keyed, s));
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
