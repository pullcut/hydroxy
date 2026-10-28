//! One timed event block on the day grid.

use dioxus::prelude::*;
use hydroxy_headless::week_scheduler::{self as h, Hit, Placed, WeekEvent, WeekSchedulerState};

use super::Ctx;

/// The block for one timed event.
#[allow(clippy::too_many_arguments, reason = "the render state the block reads")]
pub(super) fn event_block(
    ctx: Ctx,
    p: Placed,
    shown: &[WeekEvent],
    s: &WeekSchedulerState,
    root: &str,
    event_help: &str,
    n: usize,
    snap: u16,
) -> Element {
    let e = &shown[p.index];
    let (moving, resizing, keyed) = (e.id.clone(), e.id.clone(), e.id.clone());
    let selected = s.selected.as_deref() == Some(e.id.as_str());
    let dragged = s.drag.as_ref().is_some_and(|d| d.id.as_deref() == Some(e.id.as_str()));
    let (start, end) = (e.start, e.end);
    #[allow(clippy::cast_precision_loss, reason = "a handful of days and lanes")]
    let left = p.day as f64 + p.lane as f64 / p.lanes.max(1) as f64;
    rsx! {
        div {
            key: "{e.id}",
            id: h::event_id(root, &e.id),
            role: "button",
            tabindex: "0",
            "data-scope": h::SCOPE,
            "data-part": "event",
            "data-tone": e.tone.clone(),
            "data-selected": selected.then_some(""),
            "data-dragging": dragged.then_some(""),
            "data-short": (e.end.saturating_sub(e.start) < 45).then_some(""),
            "aria-label": h::event_label(e),
            "aria-pressed": selected.to_string(),
            "aria-describedby": event_help,
            style: h::block_style(p.day, p.lane, p.lanes, n, e.start, e.end),
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
            span { "data-scope": h::SCOPE, "data-part": "event-title", "{e.title}" }
            span { "data-scope": h::SCOPE, "data-part": "event-time", {h::time_range(e.start, e.end)} }
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
