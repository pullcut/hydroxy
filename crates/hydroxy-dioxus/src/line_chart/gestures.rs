//! Helpers the cartesian charts share: annotation markup, pointer fractions, wheel and
//! drag zoom, the axis gutter, and the controlled-view link.

use dioxus::html::geometry::ElementPoint;
use dioxus::prelude::*;
use hydroxy_charts::annotate::Placed;
use hydroxy_charts::zoom::{self, Axes, Gesture, View};
use std::cell::Cell;
use std::rc::Rc;

/// How long a zoom or pan must rest, in ms, before `on_view_change` fires.
const SETTLE: u32 = 120;

/// The placed annotations, as positioned `annotation` parts. Shared by the cartesian charts.
pub(crate) fn annotation_nodes(placed: &[Placed]) -> Element {
    rsx! {
        for (i , p) in placed.iter().enumerate() {
            div { key: "{i}", "data-part": "annotation", "data-shape": p.shape.attr(), style: "{p.style()}",
                if let Some(l) = &p.label {
                    span { "data-part": "annotation-label", "{l}" }
                }
            }
        }
    }
}

/// A point over an element of `size` (width, height), as fractions with y up.
pub(crate) fn plot_fractions(point: ElementPoint, size: (f64, f64)) -> Option<(f64, f64)> {
    (size.0 > 0.0 && size.1 > 0.0).then(|| (point.x / size.0, 1.0 - point.y / size.1))
}

/// Zoom `view` for a wheel or trackpad-pinch event, cancelling the page scroll when it zooms.
pub(crate) fn wheel_zoom(
    e: &Event<WheelData>,
    size: (f64, f64),
    view: View,
    axes: Axes,
    min_span: f64,
) -> Option<View> {
    let at = plot_fractions(e.data().element_coordinates(), size)?;
    let next = view.zoom(at, zoom::wheel_factor(e.data().delta().strip_units().y), axes, min_span);
    (next != view).then(|| {
        e.prevent_default();
        next
    })
}

/// Start a drag or pinch on a zoomable plot.
pub(crate) fn gesture_down(mut gesture: Signal<Gesture>, e: &Event<PointerData>, size: (f64, f64)) {
    let primary = e.data().trigger_button().is_none_or(|b| b == dioxus::html::input_data::MouseButton::Primary);
    if let (true, Some(at)) = (primary, plot_fractions(e.data().element_coordinates(), size)) {
        gesture.write().down(e.data().pointer_id(), at);
    }
}

/// Continue a drag or pinch: the new view, if it changed.
pub(crate) fn gesture_move(
    mut gesture: Signal<Gesture>,
    e: &Event<PointerData>,
    size: (f64, f64),
    view: View,
    axes: Axes,
    min_span: f64,
) -> Option<View> {
    let at = plot_fractions(e.data().element_coordinates(), size)?;
    gesture.write().moved(e.data().pointer_id(), at, view, axes, min_span)
}

pub(crate) use hydroxy_charts::line::windowed as cut;

/// Sizes the left gutter to the widest tick label.
pub(crate) fn gutter(labels: &[String], min: f64) -> String {
    format!("--ox-axis: {}px;", hydroxy_charts::plot::axis_width(labels.iter().map(String::as_str), min))
}

/// Wire a chart's internal view to a controlled `view` prop and an `on_view_change` handler.
///
/// `view` is the prop (`None` when uncontrolled); when it changes to something other than the
/// chart's own view, it is applied with `write`. `on_view_change` fires once the view has been
/// still for [`SETTLE`] ms and differs from the last one the parent saw, so a change that came
/// from the prop never echoes back. `read` is the tracked view, `peek` the same untracked.
/// A hook: call it unconditionally.
pub(crate) fn link_view(
    view: ReadSignal<Option<View>>,
    on_view_change: Option<EventHandler<View>>,
    read: impl Fn() -> View + Copy + 'static,
    peek: impl Fn() -> View + Copy + 'static,
    mut write: impl FnMut(View) + Copy + 'static,
) {
    let last = use_hook(|| Rc::new(Cell::new(peek())));
    let run = use_hook(|| Rc::new(Cell::new(0_u64)));
    let seen = Rc::clone(&last);
    use_effect(move || {
        if let Some(v) = view()
            && peek() != v
        {
            seen.set(v);
            write(v);
        }
    });
    let first = use_hook(|| Rc::new(Cell::new(true)));
    use_effect(move || {
        let _ = read();
        if first.replace(false) {
            return;
        }
        let Some(cb) = on_view_change else { return };
        let id = run.get().wrapping_add(1);
        run.set(id);
        let (last, run) = (Rc::clone(&last), Rc::clone(&run));
        spawn(async move {
            crate::js::sleep(SETTLE).await;
            let v = peek();
            if run.get() == id && last.get() != v {
                last.set(v);
                cb.call(v);
            }
        });
    });
}
