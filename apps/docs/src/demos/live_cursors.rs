use std::time::Duration;

use hydroxy_leptos::live_cursors::{LiveCursors, RemoteCursor};
use leptos::prelude::*;

/// Where a teammate's pointer is at time `t`: a slow loop over the board, as if
/// the positions were arriving over the network.
fn path(seed: f64, t: f64) -> (f64, f64) {
    (0.5 + 0.36 * (t * 0.7 + seed).sin(), 0.5 + 0.3 * (t * 1.1 + seed * 2.0).cos())
}

pub fn demo() -> AnyView {
    let t = RwSignal::new(0.0_f64);
    // Sana pauses half the time, so her cursor fades while it sits still.
    // Started from an effect, so the clock only runs in the browser.
    let handle = StoredValue::new(None::<IntervalHandle>);
    Effect::new(move |prev: Option<()>| {
        if prev.is_none() {
            handle
                .set_value(set_interval_with_handle(move || t.update(|t| *t += 0.25), Duration::from_millis(250)).ok());
        }
    });
    on_cleanup(move || {
        if let Some(h) = handle.try_get_value().flatten() {
            h.clear();
        }
    });
    let mine = RwSignal::new(None::<(f64, f64)>);
    let cursors = Signal::derive(move || {
        let now = t.get();
        let (jx, jy) = path(0.0, now);
        let (px, py) = path(2.1, now * 0.8);
        let still = (now / 6.0).floor() % 2.0 == 0.0;
        let (sx, sy) = if still { (0.78, 0.22) } else { path(4.2, now) };
        vec![
            RemoteCursor::new("u-jonas", "Jonas Berg", jx, jy),
            RemoteCursor::new("u-priya", "Priya Raman", px, py),
            RemoteCursor::new("u-sana", "Sana Haddad", sx, sy),
        ]
    });

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <LiveCursors cursors=cursors idle_ms=2_000 on_move=move |p| mine.set(p)>
                <div style="height: 280px; border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-sunken); display: grid; place-items: center; color: var(--text-3); font: var(--text-sm);">
                    "Launch retro whiteboard"
                </div>
            </LiveCursors>
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                "Your pointer, as sent to others: "
                {move || mine.get().map_or_else(|| "off the board".to_string(), |(x, y)| format!("{x:.2}, {y:.2}"))}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "live-cursors",
    name: "Live cursors",
    group: "Activity",
    summary: "Other people's pointers over a shared surface. You bring the network; positions glide between updates and fade when still.",
    demo,
    leptos: include_str!("live_cursors.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/live_cursors.rs"),
    parts: &[
        &["LiveCursors", "root", "The surface, position: relative. Reports your pointer through on_move"],
        &["LiveCursors", "content", "Your children"],
        &["LiveCursors", "layer", "aria-hidden overlay holding the cursors; pair with a presence bar for who's here"],
        &["LiveCursors", "cursor", "One person. data-jewel tints it; data-idle once still"],
        &["LiveCursors", "arrow", "The pointer glyph"],
        &["LiveCursors", "label", "Name pill beside the arrow"],
    ],
    keys: &[],
    props: &[
        &["cursors", "Vec<RemoteCursor>", "Required. Id, name, x and y in 0..1, optional jewel"],
        &["idle_ms", "u32", "Stillness before a cursor fades. Default 4000"],
        &["on_move", "Callback<Option<(f64, f64)>>", "Your pointer in 0..1 coordinates; None when it leaves"],
        &["children", "Children", "The shared surface"],
    ],
    events: &[],
};
