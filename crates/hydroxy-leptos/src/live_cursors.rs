//! `<LiveCursors>`: other people's pointers drawn over a shared surface. The host
//! owns the network: it passes positions in normalised coordinates (0–1 across
//! the surface) and gets the local pointer back the same way from `on_move`.
//! Moves glide between updates; a cursor that stops moving fades out.

use std::collections::HashSet;
use std::time::Duration;

use leptos::prelude::*;
use hydroxy_headless::live_cursors::{self as h, CursorTracker};

pub use hydroxy_headless::live_cursors::RemoteCursor;

/// A surface with live cursors over it.
#[component]
pub fn LiveCursors(
    /// Everyone else's latest pointer.
    #[prop(into)]
    cursors: Signal<Vec<RemoteCursor>>,
    /// Milliseconds of stillness before a cursor fades. Default 4000.
    #[prop(default = 4_000)]
    idle_ms: u32,
    /// Called with the local pointer in normalised coordinates as it moves over the
    /// surface, and with `None` when it leaves. Send these to the other people.
    #[prop(optional, into)]
    on_move: Option<Callback<Option<(f64, f64)>>>,
    /// The shared surface.
    children: Children,
) -> impl IntoView {
    let tracker = StoredValue::new(CursorTracker::default());
    let tick = RwSignal::new(0_u64);
    let ticks = h::idle_ticks(idle_ms);
    // The clock only advances; positions are compared on each read, so a still
    // cursor goes idle and a moved one wakes on the next tick at the latest.
    // Started from an effect, so it only runs in the browser.
    let handle = StoredValue::new(None::<IntervalHandle>);
    Effect::new(move |prev: Option<()>| {
        if prev.is_none()
            && let Ok(interval) = set_interval_with_handle(
                move || {
                    tracker.update_value(CursorTracker::advance);
                    tick.update(|t| *t += 1);
                },
                Duration::from_millis(h::TICK_MS.into()),
            )
        {
            handle.set_value(Some(interval));
        }
    });
    on_cleanup(move || {
        if let Some(interval) = handle.try_get_value().flatten() {
            interval.clear();
        }
    });
    let idle = Memo::new(move |_| {
        tick.track();
        cursors.with(|list| {
            tracker.update_value(|t| t.update(list.iter().map(|c| (c.id.as_str(), c.x, c.y))));
            tracker.with_value(|t| list.iter().filter(|c| t.is_idle(&c.id, ticks)).map(|c| c.id.clone()).collect::<HashSet<_>>())
        })
    });

    let report = move |ev: leptos::ev::PointerEvent| {
        let Some(cb) = on_move else { return };
        let Some(el) = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(t).ok()) else {
            return;
        };
        let r = el.get_bounding_client_rect();
        let at = h::normalise(f64::from(ev.client_x()) - r.left(), f64::from(ev.client_y()) - r.top(), r.width(), r.height());
        cb.run(at);
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            on:pointermove=report
            on:pointerleave=move |_| {
                if let Some(cb) = on_move {
                    cb.run(None);
                }
            }
        >
            <div data-scope=h::SCOPE data-part="content">{children()}</div>
            <div data-scope=h::SCOPE data-part="layer" aria-hidden="true">
                <For
                    each=move || cursors.with(|c| c.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                    key=String::clone
                    children=move |id| {
                        let me = Memo::new({
                            let id = id.clone();
                            move |_| cursors.with(|c| c.iter().find(|c| c.id == id).cloned())
                        });
                        let style = move || me.with(|c| c.as_ref().map(|c| h::position_style(c.x, c.y)));
                        let jewel = move || me.with(|c| c.as_ref().map(|c| c.tint().as_str()));
                        let name = move || me.with(|c| c.as_ref().map(|c| c.name.clone()));
                        let is_idle = move || idle.with(|i| i.contains(&id)).then_some("");
                        view! {
                            <div data-scope=h::SCOPE data-part="cursor" data-jewel=jewel data-idle=is_idle style=style>
                                <svg data-scope=h::SCOPE data-part="arrow" viewBox="0 0 16 16" width="16" height="16">
                                    <path d=h::ARROW_PATH />
                                </svg>
                                <span data-scope=h::SCOPE data-part="label">{name}</span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
