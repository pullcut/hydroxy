//! One all-day event chip in the all-day row.

use hydroxy_headless::week_scheduler::{self as h, Placed, WeekEvent, WeekSchedulerEvent};
use leptos::prelude::*;

use super::Ctx;
use super::event::find;

pub(super) fn all_day_view(
    ctx: Ctx,
    id: &str,
    shown: Memo<Vec<WeekEvent>>,
    rows: Memo<Vec<Placed>>,
    count: Memo<usize>,
) -> impl IntoView + use<> {
    let item = find(shown, rows, id.to_owned());
    let root = ctx.root.get_value();
    let selected = {
        let id = id.to_owned();
        Memo::new(move |_| ctx.state.with(|s| s.selected.as_deref() == Some(id.as_str())))
    };
    let (clicked, keyed) = (id.to_owned(), id.to_owned());
    let snap = ctx.input.with_untracked(|i| i.snap);

    view! {
        <div
            id=h::event_id(&root, id)
            role="button"
            tabindex="0"
            data-scope=h::SCOPE
            data-part="allday-event"
            data-tone=move || item.with(|x| x.as_ref().and_then(|(e, _)| e.tone.clone()))
            data-selected=move || selected.get().then_some("")
            aria-label=move || item.with(|x| x.as_ref().map(|(e, _)| h::event_label(e)))
            aria-pressed=move || selected.get().to_string()
            aria-describedby=h::event_help_id(&root)
            style=move || item.with(|x| x.as_ref().map(|(_, p)| h::all_day_style(p.day, p.lane, count.get())))
            on:click=move |ev| {
                ev.stop_propagation();
                ctx.send(WeekSchedulerEvent::Select(clicked.clone()));
            }
            on:keydown=move |ev| {
                if let Some(e) = h::event_keydown(&ev.key(), ev.shift_key(), &keyed, snap, false) {
                    ev.prevent_default();
                    ctx.send(e);
                }
            }
        >
            {move || item.with(|x| x.as_ref().map(|(e, _)| e.title.clone()))}
        </div>
    }
}
