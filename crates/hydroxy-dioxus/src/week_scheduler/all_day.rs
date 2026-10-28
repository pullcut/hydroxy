//! One all-day event chip in the all-day row.

use dioxus::prelude::*;
use hydroxy_headless::week_scheduler::{self as h, Placed, WeekEvent, WeekSchedulerEvent, WeekSchedulerState};

use super::Ctx;

/// The chip for one all-day event.
#[allow(clippy::too_many_arguments, clippy::large_types_passed_by_value, reason = "Ctx is a Copy bundle of signal handles; the render state the chip reads")]
pub(super) fn all_day_chip(
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
    let (clicked, keyed) = (e.id.clone(), e.id.clone());
    let selected = s.selected.as_deref() == Some(e.id.as_str());
    rsx! {
        div {
            key: "{e.id}",
            id: h::event_id(root, &e.id),
            role: "button",
            tabindex: "0",
            "data-scope": h::SCOPE,
            "data-part": "allday-event",
            "data-tone": e.tone.clone(),
            "data-selected": selected.then_some(""),
            "aria-label": h::event_label(e),
            "aria-pressed": selected.to_string(),
            "aria-describedby": event_help,
            style: h::all_day_style(p.day, p.lane, n),
            onclick: move |ev| {
                ev.stop_propagation();
                ctx.send(WeekSchedulerEvent::Select(clicked.clone()));
            },
            onkeydown: move |ev| {
                let key = ev.key().to_string();
                if let Some(event) = h::event_keydown(&key, ev.modifiers().shift(), &keyed, snap, false) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            },
            "{e.title}"
        }
    }
}
