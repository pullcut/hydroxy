//! The overview strip under a zoomable [`LineChart`](super::LineChart).

use hydroxy_charts::brush::{Brush, Drag, Grab};
use leptos::prelude::*;

use crate::plot_kit::colour;

fn strip_fraction(ev: &leptos::ev::PointerEvent) -> Option<f64> {
    let el =
        ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())?;
    Some((f64::from(ev.offset_x()) / f64::from(el.offset_width().max(1))).clamp(0.0, 1.0))
}

/// The overview strip under a zoomable [`LineChart`]: a faint copy of every line over the
/// full range, with a draggable window and two keyboard-operable edges.
#[component]
pub(super) fn BrushStrip(
    window: RwSignal<Brush>,
    overview: Memo<Vec<String>>,
    labels: Signal<Vec<String>>,
    range: Memo<std::ops::Range<usize>>,
    label: String,
) -> impl IntoView {
    let drag = StoredValue::new(None::<Drag>);
    let len = move || labels.with(Vec::len);
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let step = move || 1.0 / len().saturating_sub(1).max(1) as f64;
    let on_down = move |ev: leptos::ev::PointerEvent| {
        let Some(f) = strip_fraction(&ev) else { return };
        if let Some(el) =
            ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())
        {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        let (next, d) = window.get_untracked().begin(f, 0.025);
        window.set(next);
        drag.set_value(Some(d));
    };
    let on_move = move |ev: leptos::ev::PointerEvent| {
        if let (Some(d), Some(f)) = (drag.get_value(), strip_fraction(&ev)) {
            window.set(d.apply(f, step()));
        }
    };
    let end_drag = move |_| drag.set_value(None);
    let handle_label = label.clone();
    let handle = move |grab: Grab, name: &'static str| {
        let index = move || if grab == Grab::Start { range.get().start } else { range.get().end.saturating_sub(1) };
        view! {
            <div
                data-part="brush-handle"
                data-edge=name
                role="slider"
                tabindex="0"
                aria-label=format!("{handle_label} range {name}")
                aria-valuemin="0"
                aria-valuemax=move || len().saturating_sub(1).to_string()
                aria-valuenow=move || index().to_string()
                aria-valuetext=move || labels.with(|l| l.get(index()).cloned().unwrap_or_default())
                style=move || {
                    let b = window.get();
                    format!("left: {:.3}%;", if grab == Grab::Start { b.start } else { b.end } * 100.0)
                }
                on:keydown=move |ev| {
                    if let Some(next) = window.get_untracked().key(grab, &ev.key(), ev.shift_key(), step(), step()) {
                        ev.prevent_default();
                        window.set(next);
                    }
                }
            ></div>
        }
    };
    view! {
        <div
            data-part="brush"
            role="group"
            aria-label=label + ": zoom range"
            on:pointerdown=on_down
            on:pointermove=on_move
            on:pointerup=end_drag
            on:pointercancel=end_drag
        >
            <svg data-part="brush-marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || {
                    overview
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, d)| view! { <path data-part="brush-line" d=d stroke=colour(i)></path> })
                        .collect_view()
                }}
            </svg>
            <div
                data-part="brush-window"
                style=move || {
                    let b = window.get();
                    format!("left: {:.3}%; width: {:.3}%;", b.start * 100.0, b.span() * 100.0)
                }
            ></div>
            {handle(Grab::Start, "start")}
            {handle(Grab::End, "end")}
        </div>
    }
}
