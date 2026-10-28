//! `LiveCursors`: other people's pointers drawn over a shared surface. The host
//! owns the network: it passes positions in normalised coordinates (0–1 across
//! the surface) and gets the local pointer back the same way from `on_move`.
//! Moves glide between updates; a cursor that stops moving fades out.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use oxide_headless::live_cursors::{self as h, CursorTracker};

pub use oxide_headless::live_cursors::RemoteCursor;

/// A surface with live cursors over it.
#[component]
pub fn LiveCursors(
    /// Everyone else's latest pointer.
    cursors: ReadSignal<Vec<RemoteCursor>>,
    /// Milliseconds of stillness before a cursor fades. Default 4000.
    #[props(default = 4_000)]
    idle_ms: u32,
    /// Called with the local pointer in normalised coordinates as it moves over the
    /// surface, and with `None` when it leaves. Send these to the other people.
    on_move: Option<EventHandler<Option<(f64, f64)>>>,
    /// The shared surface.
    children: Element,
) -> Element {
    let tracker = use_hook(|| Rc::new(RefCell::new(CursorTracker::default())));
    let mut tick = use_signal(|| 0_u64);
    // The clock runs through eval, so it works on web and desktop alike.
    let clock = tracker.clone();
    use_future(move || {
        let clock = clock.clone();
        async move {
            loop {
                let wait = format!("await new Promise(r => setTimeout(r, {})); return true;", h::TICK_MS);
                if document::eval(&wait).join::<bool>().await.is_err() {
                    break;
                }
                clock.borrow_mut().advance();
                tick += 1;
            }
        }
    });
    // Positions are compared on every render (each tick re-renders), so moved
    // cursors wake and still ones go idle.
    let _ = tick();
    let list = cursors.read().clone();
    tracker.borrow_mut().update(list.iter().map(|c| (c.id.as_str(), c.x, c.y)));
    let ticks = h::idle_ticks(idle_ms);
    let t = tracker.borrow();

    // The surface's box, fetched when the pointer comes in, to normalise its moves.
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let rect = use_signal(|| None::<(f64, f64, f64, f64)>);
    let measure = move || {
        if let Some(el) = mounted.peek().clone() {
            spawn(async move {
                let mut rect = rect;
                if let Ok(r) = el.get_client_rect().await {
                    rect.set(Some((r.origin.x, r.origin.y, r.size.width, r.size.height)));
                }
            });
        }
    };

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            onmounted: move |e| mounted.set(Some(e.data())),
            onpointerenter: move |_| measure(),
            onpointermove: move |e| {
                let Some(handler) = on_move else { return };
                let Some((left, top, w, hgt)) = *rect.peek() else { return };
                let p = e.client_coordinates();
                handler.call(h::normalise(p.x - left, p.y - top, w, hgt));
            },
            onpointerleave: move |_| {
                if let Some(handler) = on_move {
                    handler.call(None);
                }
            },
            div { "data-scope": h::SCOPE, "data-part": "content", {children} }
            div { "data-scope": h::SCOPE, "data-part": "layer", "aria-hidden": "true",
                for c in list {
                    div {
                        key: "{c.id}",
                        "data-scope": h::SCOPE,
                        "data-part": "cursor",
                        "data-jewel": c.tint().as_str(),
                        "data-idle": t.is_idle(&c.id, ticks).then_some(""),
                        style: h::position_style(c.x, c.y),
                        svg {
                            "data-scope": h::SCOPE,
                            "data-part": "arrow",
                            view_box: "0 0 16 16",
                            width: "16",
                            height: "16",
                            path { d: h::ARROW_PATH }
                        }
                        span { "data-scope": h::SCOPE, "data-part": "label", "{c.name}" }
                    }
                }
            }
        }
    }
}
