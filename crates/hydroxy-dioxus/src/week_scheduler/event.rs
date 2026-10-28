//! One timed event block on the day grid.

use dioxus::prelude::*;
use hydroxy_headless::week_scheduler::{self as h, Hit, Placed, WeekEvent, WeekSchedulerState};

use super::Ctx;

/// The block for one timed event.
#[allow(clippy::too_many_arguments, clippy::large_types_passed_by_value, reason = "Ctx is a Copy bundle of signal handles; the render state the block reads")]
pub(super) fn event_block(
    ctx: Ctx,
    placed: Placed,
    shown: &[WeekEvent],
    state: &WeekSchedulerState,
    root: &str,
    event_help: &str,
    days: usize,
    snap: u16,
) -> Element {
    let item = &shown[placed.index];
    let (moving, resizing, keyed) = (item.id.clone(), item.id.clone(), item.id.clone());
    let selected = state.selected.as_deref() == Some(item.id.as_str());
    let dragged = state.drag.as_ref().is_some_and(|d| d.id.as_deref() == Some(item.id.as_str()));
    let (start, end) = (item.start, item.end);
    #[allow(clippy::cast_precision_loss, reason = "a handful of days and lanes")]
    let left = placed.day as f64 + placed.lane as f64 / placed.lanes.max(1) as f64;
    rsx! {
        div {
            key: "{item.id}",
            id: h::event_id(root, &item.id),
            role: "button",
            tabindex: "0",
            "data-scope": h::SCOPE,
            "data-part": "event",
            "data-tone": item.tone.clone(),
            "data-selected": selected.then_some(""),
            "data-dragging": dragged.then_some(""),
            "data-short": (item.end.saturating_sub(item.start) < 45).then_some(""),
            "aria-label": h::event_label(item),
            "aria-pressed": selected.to_string(),
            "aria-describedby": event_help,
            style: h::block_style(placed.day, placed.lane, placed.lanes, days, item.start, item.end),
            onpointerdown: move |ev| {
                let at = ev.data().element_coordinates();
                let x = left * ctx.column() + at.x;
                ctx.down(&ev, Hit::Event(moving.clone()), (x, h::minute_y(start) + at.y));
            },
            onkeydown: move |ev| {
                let key = ev.key().to_string();
                if let Some(event) = h::event_keydown(&key, ev.modifiers().shift(), &keyed, snap, ctx.dragging()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            },
            span { "data-scope": h::SCOPE, "data-part": "event-title", "{item.title}" }
            span { "data-scope": h::SCOPE, "data-part": "event-time", {h::time_range(item.start, item.end)} }
            span {
                "data-scope": h::SCOPE,
                "data-part": "handle",
                "aria-hidden": "true",
                onpointerdown: move |ev| {
                    let x = left * ctx.column();
                    ctx.down(&ev, Hit::Resize(resizing.clone()), (x, h::minute_y(end)));
                },
            }
        }
    }
}
