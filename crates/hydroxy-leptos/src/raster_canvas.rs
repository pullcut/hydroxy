//! `<RasterCanvas>`: paints a chart's pixel buffer onto a `<canvas>` at device resolution,
//! in the current theme's colours. It repaints when its inputs change, when it is resized,
//! and when a `data-theme` attribute changes anywhere on the page. While a zoomable view
//! moves, the last frame is stretched with a CSS transform and repainted once it settles.

use std::time::Duration;

use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::{Clamped, JsCast};
use hydroxy_charts::raster::{Raster, Rgba};
use hydroxy_charts::zoom::View;
use hydroxy_pacer::Debouncer;
use hydroxy_pacer::leptos::use_debounced_callback;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MutationObserver, MutationObserverInit, ResizeObserver};

/// What a draw callback gets: resolved colours and the size to draw at.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Frame {
    /// The `colours` prop, resolved to RGBA in the current theme.
    pub colours: Vec<Rgba>,
    /// Width in device pixels.
    pub width: usize,
    /// Height in device pixels.
    pub height: usize,
    /// Device pixels per CSS pixel.
    pub dpr: f64,
    /// The view to draw (full extent when the canvas has no `view`).
    pub view: View,
}

/// How long the view must stay still before a full repaint.
const SETTLE: Duration = Duration::from_millis(120);


/// The observers and the closures they call, kept alive together.
struct Watch {
    resize: ResizeObserver,
    theme: MutationObserver,
    _on_resize: Closure<dyn FnMut()>,
    _on_theme: Closure<dyn FnMut()>,
}

fn watch(canvas: &HtmlCanvasElement, tick: RwSignal<u32>) -> Option<Watch> {
    let bump = move || {
        tick.try_update(|t| *t = t.wrapping_add(1));
    };
    let on_resize = Closure::<dyn FnMut()>::new(bump);
    let on_theme = Closure::<dyn FnMut()>::new(bump);
    let resize = ResizeObserver::new(on_resize.as_ref().unchecked_ref()).ok()?;
    resize.observe(canvas);
    let theme = MutationObserver::new(on_theme.as_ref().unchecked_ref()).ok()?;
    let options = MutationObserverInit::new();
    options.set_attributes(true);
    options.set_subtree(true);
    options.set_attribute_filter(&web_sys::js_sys::Array::of1(&"data-theme".into()));
    theme.observe_with_options(&document().document_element()?.into(), &options).ok()?;
    Some(Watch { resize, theme, _on_resize: on_resize, _on_theme: on_theme })
}

/// Resolve CSS colours (anything `color` accepts, `var()` and `color-mix()` included) to
/// RGBA by painting a strip of them onto the canvas's top row and reading it back.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss, reason = "small counts")]
fn resolve(canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d, css: &[String]) -> Vec<Rgba> {
    let Ok(Some(computed)) = window().get_computed_style(canvas) else { return Vec::new() };
    let style = web_sys::HtmlElement::style(canvas);
    let width = css.len().max(1) as u32;
    if canvas.width() < width {
        canvas.set_width(width);
    }
    ctx.clear_rect(0.0, 0.0, f64::from(width), 1.0);
    for (i, c) in css.iter().enumerate() {
        let _ = style.set_property("color", c);
        ctx.set_fill_style_str(&computed.get_property_value("color").unwrap_or_default());
        ctx.fill_rect(i as f64, 0.0, 1.0, 1.0);
    }
    let _ = style.remove_property("color");
    let Ok(data) = ctx.get_image_data(0.0, 0.0, f64::from(width), 1.0) else { return Vec::new() };
    data.data().as_chunks::<4>().0.iter().take(css.len()).copied().collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "canvas sizes are small and positive")]
fn paint(canvas: &HtmlCanvasElement, css: &[String], view: View, draw: impl FnOnce(Frame) -> Raster) {
    let Some(ctx) = canvas.get_context("2d").ok().flatten().and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok()) else { return };
    let dpr = window().device_pixel_ratio().max(1.0);
    let width = (f64::from(canvas.client_width()) * dpr).round() as usize;
    let height = (f64::from(canvas.client_height()) * dpr).round() as usize;
    if width == 0 || height == 0 {
        return;
    }
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    let colours = resolve(canvas, &ctx, css);
    let raster = draw(Frame { colours, width, height, dpr, view });
    if let Ok(image) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&raster.rgba), raster.width as u32, raster.height as u32) {
        let _ = ctx.put_image_data(&image, 0.0, 0.0);
    }
}

/// A canvas filling its positioned parent, drawn by `draw`. Any signal `draw` reads
/// triggers a repaint.
#[component]
pub(crate) fn RasterCanvas(
    /// CSS colours to resolve for each paint, such as `var(--chart-1)`.
    #[prop(into)]
    colours: Signal<Vec<String>>,
    /// Draws the frame. It should read the view from the [`Frame`], not a signal, so view
    /// changes go through the cheap stretch-then-settle path.
    draw: Callback<Frame, Raster>,
    /// The zoomed window, for charts that zoom.
    #[prop(optional, into)]
    view: Option<Signal<View>>,
) -> impl IntoView {
    let node = NodeRef::<leptos::html::Canvas>::new();
    let tick = RwSignal::new(0u32);
    let watching = StoredValue::new_local(None::<Watch>);
    let painted = StoredValue::new(View::FULL);
    let current = move || view.map_or(View::FULL, |v| v.get_untracked());
    Effect::new(move |_| {
        tick.track();
        let Some(canvas) = node.get() else { return };
        if watching.with_value(Option::is_none) {
            watching.set_value(watch(&canvas, tick));
        }
        let css = colours.get();
        let at = current();
        paint(&canvas, &css, at, |frame| draw.run(frame));
        painted.set_value(at);
        let _ = web_sys::HtmlElement::style(&canvas).remove_property("transform");
    });
    // A moving view stretches the last frame now and repaints once it has been still a moment.
    let repaint = use_debounced_callback(Debouncer::new(SETTLE), move |()| {
        tick.try_update(|t| *t = t.wrapping_add(1));
    });
    Effect::new(move |_| {
        let Some(at) = view.map(|v| v.get()) else { return };
        let Some(canvas) = node.get_untracked() else { return };
        if at == painted.get_value() {
            return;
        }
        let _ = web_sys::HtmlElement::style(&canvas).set_property("transform", &hydroxy_charts::zoom::css_stretch(painted.get_value(), at));
        repaint.call(());
    });
    on_cleanup(move || {
        if let Some(w) = watching.try_update_value(Option::take).flatten() {
            w.resize.disconnect();
            w.theme.disconnect();
        }
    });
    view! {
        <div data-part="canvas-clip" aria-hidden="true">
            <canvas node_ref=node data-part="canvas"></canvas>
        </div>
    }
}
