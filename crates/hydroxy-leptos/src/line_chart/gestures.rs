//! Helpers the cartesian charts share: annotation markup, pointer fractions, wheel and
//! drag zoom, and the axis gutter.

use hydroxy_charts::annotate::Placed;
use hydroxy_charts::zoom::{self, Axes, Gesture, View};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use std::time::Duration;

/// How long a zoom or pan must rest before `on_view_change` fires.
const SETTLE: Duration = Duration::from_millis(120);

/// The placed annotations, as positioned `annotation` parts. Shared by the cartesian charts.
pub(crate) fn annotation_views(placed: Vec<Placed>) -> impl IntoView {
    placed
        .into_iter()
        .map(|p| {
            let style = p.style();
            view! {
                <div data-part="annotation" data-shape=p.shape.attr() style=style>
                    {p.label.map(|l| view! { <span data-part="annotation-label">{l}</span> })}
                </div>
            }
        })
        .collect_view()
}

/// The pointer's position over the event's current target, as fractions with y up.
pub(crate) fn plot_fractions(ev: &web_sys::MouseEvent) -> Option<(f64, f64)> {
    let el = ev.current_target()?.dyn_into::<web_sys::HtmlElement>().ok()?;
    let (w, h) = (f64::from(el.offset_width().max(1)), f64::from(el.offset_height().max(1)));
    Some((f64::from(ev.offset_x()) / w, 1.0 - f64::from(ev.offset_y()) / h))
}

/// Zoom `view` for a wheel or trackpad-pinch event, cancelling the page scroll when it zooms.
pub(crate) fn wheel_zoom(ev: &web_sys::WheelEvent, view: View, axes: Axes, min_span: f64) -> Option<View> {
    let at = plot_fractions(ev)?;
    let lines = if ev.delta_mode() == 1 { 16.0 } else { 1.0 };
    let next = view.zoom(at, zoom::wheel_factor(ev.delta_y() * lines), axes, min_span);
    (next != view).then(|| {
        ev.prevent_default();
        next
    })
}

/// Start a drag or pinch on a zoomable plot.
pub(crate) fn gesture_down(gesture: StoredValue<Gesture>, ev: &web_sys::PointerEvent) {
    if ev.button() > 0 {
        return;
    }
    if let Some(at) = plot_fractions(ev) {
        if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        gesture.update_value(|g| g.down(ev.pointer_id(), at));
    }
}

/// Continue a drag or pinch: the new view, if it changed.
pub(crate) fn gesture_move(
    gesture: StoredValue<Gesture>,
    ev: &web_sys::PointerEvent,
    view: View,
    axes: Axes,
    min_span: f64,
) -> Option<View> {
    let at = plot_fractions(ev)?;
    let mut g = gesture.get_value();
    let next = g.moved(ev.pointer_id(), at, view, axes, min_span);
    gesture.set_value(g);
    next
}

pub(crate) use hydroxy_charts::line::windowed as cut;

/// Sizes the left gutter to the widest tick label.
pub(crate) fn gutter(labels: &[String], min: f64) -> String {
    format!("--ox-axis: {}px;", hydroxy_charts::plot::axis_width(labels.iter().map(String::as_str), min))
}

/// Wire a chart's internal view to a controlled `view` prop and an `on_view_change` callback.
///
/// `view` returns the prop's value (`None` when uncontrolled); when it changes to something
/// other than `read`, it is applied with `write`. `on_view_change` fires once the view has been
/// still for [`SETTLE`] and differs from the last one the parent saw, so a change that came from
/// the prop never echoes back. `read` must be cheap and may be tracked.
pub(crate) fn link_view(
    view: impl Fn() -> Option<View> + 'static,
    on_view_change: Option<Callback<View>>,
    read: impl Fn() -> View + Copy + 'static,
    write: impl Fn(View) + Copy + 'static,
) {
    let last = StoredValue::new_local(untrack(read));
    Effect::new(move |_| {
        if let Some(v) = view()
            && untrack(read) != v
        {
            last.set_value(v);
            write(v);
        }
    });
    let Some(cb) = on_view_change else { return };
    let timer = StoredValue::new_local(None::<TimeoutHandle>);
    Effect::new(move |first: Option<()>| {
        let _ = read();
        if first.is_none() {
            return;
        }
        if let Some(h) = timer.get_value() {
            h.clear();
        }
        let handle = set_timeout_with_handle(
            move || {
                // The chart may have unmounted since the timer was armed.
                let Some(seen) = last.try_get_value() else { return };
                let v = untrack(read);
                if seen != v {
                    last.set_value(v);
                    cb.run(v);
                }
            },
            SETTLE,
        )
        .ok();
        timer.set_value(handle);
    });
}
