//! One timed event block in the day columns, plus the lookup both event views share.

use hydroxy_headless::week_scheduler::{self as h, Hit, Placed, WeekEvent};
use leptos::prelude::*;

use super::Ctx;

pub(super) type Found = Option<(WeekEvent, Placed)>;

pub(super) fn find(shown: Memo<Vec<WeekEvent>>, list: Memo<Vec<Placed>>, id: String) -> Memo<Found> {
    Memo::new(move |_| {
        list.with(|l| shown.with(|e| l.iter().find(|p| e[p.index].id == id).map(|p| (e[p.index].clone(), *p))))
    })
}

pub(super) fn event_view(
    ctx: Ctx,
    id: &str,
    shown: Memo<Vec<WeekEvent>>,
    placed: Memo<Vec<Placed>>,
    count: Memo<usize>,
) -> impl IntoView + use<> {
    let item = find(shown, placed, id.to_owned());
    let root = ctx.root.get_value();
    let selected = {
        let id = id.to_owned();
        Memo::new(move |_| ctx.state.with(|s| s.selected.as_deref() == Some(id.as_str())))
    };
    let dragged = {
        let id = id.to_owned();
        move || ctx.state.with(|s| s.drag.as_ref().is_some_and(|d| d.id.as_deref() == Some(id.as_str())))
    };
    let (moving, resizing, keyed) = (id.to_owned(), id.to_owned(), id.to_owned());
    let snap = ctx.input.with_untracked(|i| i.snap);

    view! {
        <div
            id=h::event_id(&root, id)
            role="button"
            tabindex="0"
            data-scope=h::SCOPE
            data-part="event"
            data-tone=move || item.with(|x| x.as_ref().and_then(|(e, _)| e.tone.clone()))
            data-selected=move || selected.get().then_some("")
            data-dragging=move || dragged().then_some("")
            data-short=move || item.with(|x| x.as_ref().is_some_and(|(e, _)| e.end.saturating_sub(e.start) < 45)).then_some("")
            aria-label=move || item.with(|x| x.as_ref().map(|(e, _)| h::event_label(e)))
            aria-pressed=move || selected.get().to_string()
            aria-describedby=h::event_help_id(&root)
            style=move || {
                item.with(|x| {
                    x.as_ref().map(|(e, p)| h::block_style(p.day, p.lane, p.lanes, count.get(), e.start, e.end))
                })
            }
            on:pointerdown=move |ev| ctx.down(&ev, Hit::Event(moving.clone()))
            on:keydown=move |ev| {
                if let Some(e) = h::event_keydown(&ev.key(), ev.shift_key(), &keyed, snap, ctx.dragging()) {
                    ev.prevent_default();
                    ctx.send(e);
                }
            }
        >
            <span data-scope=h::SCOPE data-part="event-title">
                {move || item.with(|x| x.as_ref().map(|(e, _)| e.title.clone()))}
            </span>
            <span data-scope=h::SCOPE data-part="event-time">
                {move || item.with(|x| x.as_ref().map(|(e, _)| h::time_range(e.start, e.end)))}
            </span>
            <span
                data-scope=h::SCOPE
                data-part="handle"
                aria-hidden="true"
                on:pointerdown=move |ev| ctx.down(&ev, Hit::Resize(resizing.clone()))
            ></span>
        </div>
    }
}
