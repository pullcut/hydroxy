//! `RasterCanvas`: paints a chart's pixel buffer onto a `<canvas>` at device resolution,
//! in the current theme's colours. It talks to the page through `eval`, so it works on web
//! and desktop alike, and repaints when its inputs change, when it is resized, and when a
//! `data-theme` attribute changes anywhere on the page. While a zoomable view moves, the
//! last frame is stretched with a CSS transform and repainted once it settles.

use dioxus::prelude::*;
use oxide_charts::raster::{Raster, Rgba};
use oxide_charts::zoom::View;

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

/// Receives a CSS transform, applies it, then waits a moment and returns, so Rust can
/// repaint once the view has stopped moving.
const STRETCH: &str = r"
const t = await dioxus.recv();
const canvas = document.getElementById(CANVAS_ID);
if (!canvas) return;
canvas.style.transform = t;
await new Promise((r) => setTimeout(r, 120));
return 0;
";

/// Waits for "start" (the canvas is mounted), then pings on every resize or theme change
/// until any further message arrives.
const WATCH: &str = r#"
if ((await dioxus.recv()) !== "start") return;
const canvas = document.getElementById(CANVAS_ID);
if (!canvas) return;
const ping = () => dioxus.send(0);
const resize = new ResizeObserver(ping);
resize.observe(canvas);
const theme = new MutationObserver(ping);
theme.observe(document.documentElement, { attributes: true, subtree: true, attributeFilter: ["data-theme"] });
await dioxus.recv();
resize.disconnect();
theme.disconnect();
"#;

/// Resolves the CSS colours it is sent (anything `color` accepts) against the canvas by
/// painting them onto a scratch canvas and reading them back, and measures the canvas in
/// device pixels. Returns `[colours, width, height, dpr]`, or null before it is mounted.
const MEASURE: &str = r#"
const css = await dioxus.recv();
const canvas = document.getElementById(CANVAS_ID);
if (!canvas) return null;
const probe = document.createElement("canvas");
probe.width = Math.max(css.length, 1);
probe.height = 1;
const ctx = probe.getContext("2d", { willReadFrequently: true });
const computed = getComputedStyle(canvas);
css.forEach((c, i) => {
  canvas.style.color = c;
  ctx.fillStyle = computed.color;
  ctx.fillRect(i, 0, 1, 1);
});
canvas.style.removeProperty("color");
const px = ctx.getImageData(0, 0, probe.width, 1).data;
const dpr = Math.max(window.devicePixelRatio || 1, 1);
return [
  css.map((_, i) => Array.from(px.subarray(i * 4, i * 4 + 4))),
  Math.round(canvas.clientWidth * dpr),
  Math.round(canvas.clientHeight * dpr),
  dpr,
];
"#;

/// Receives `[width, height, base64 rgba]` and puts the pixels on the canvas.
const PAINT: &str = r#"
const [w, h, data] = await dioxus.recv();
const canvas = document.getElementById(CANVAS_ID);
if (!canvas) return;
const bin = atob(data);
const bytes = new Uint8ClampedArray(bin.length);
for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
canvas.width = w;
canvas.height = h;
canvas.style.transform = "";
canvas.getContext("2d").putImageData(new ImageData(bytes, w, h), 0, 0);
"#;

/// A script with the canvas id filled in. `{id:?}` quotes it as a JS string; ids from
/// `next_id` are plain ASCII.
fn script(js: &str, id: &str) -> String {
    js.replace("CANVAS_ID", &format!("{id:?}"))
}

/// Standard base64 with padding, for handing pixels to `atob`.
fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            out.push(if i <= chunk.len() { char::from(ABC[(n >> (18 - 6 * i) & 63) as usize]) } else { '=' });
        }
    }
    out
}

/// A canvas filling its positioned parent, drawn by `draw`. Any signal or memo `draw`
/// reads triggers a repaint.
#[component]
pub(crate) fn RasterCanvas(
    /// CSS colours to resolve for each paint, such as `var(--chart-1)`.
    colours: ReadSignal<Vec<String>>,
    /// Draws the frame. It should read the view from the [`Frame`], not a signal, so view
    /// changes go through the cheap stretch-then-settle path.
    draw: Callback<Frame, Raster>,
    /// The zoomed window, for charts that zoom.
    #[props(default)]
    view: Option<ReadSignal<View>>,
) -> Element {
    let id = use_hook(|| oxide_headless::next_id("raster-canvas"));
    let mut tick = use_signal(|| 0u32);
    let mut frame = use_signal(|| None::<Frame>);
    let mut painted = use_signal(|| View::FULL);
    let watch = use_hook(|| document::eval(&script(WATCH, &id)));
    use_future(move || async move {
        let mut watch = watch;
        while watch.recv::<u8>().await.is_ok() {
            tick.with_mut(|t| *t = t.wrapping_add(1));
        }
    });
    use_drop(move || {
        let _ = watch.send("stop");
    });
    // Measure on mount, on resize or theme change, and when the colours change.
    let measure_id = id.clone();
    use_effect(move || {
        tick.read();
        let css = colours();
        let js = document::eval(&script(MEASURE, &measure_id));
        spawn(async move {
            let _ = js.send(css);
            if let Ok(Some((colours, width, height, dpr))) = js.join::<Option<(Vec<Rgba>, usize, usize, f64)>>().await {
                let view = view.map_or(View::FULL, |v| *v.peek());
                frame.set(Some(Frame { colours, width, height, dpr, view }));
            }
        });
    });
    // Paint after each measure, and whenever what `draw` reads changes.
    let paint_id = id.clone();
    use_effect(move || {
        let Some(f) = frame() else { return };
        if f.width == 0 || f.height == 0 {
            return;
        }
        let at = f.view;
        let raster = draw.call(f);
        painted.set(at);
        let _ = document::eval(&script(PAINT, &paint_id)).send((raster.width, raster.height, base64(&raster.rgba)));
    });
    // A moving view stretches the last frame now and repaints once it has been still a moment.
    let mut moves = use_signal(|| 0u32);
    let stretch_id = id.clone();
    use_effect(move || {
        let Some(at) = view.map(|v| v()) else { return };
        let from = *painted.peek();
        if at == from {
            return;
        }
        let this = moves.peek().wrapping_add(1);
        moves.set(this);
        let js = document::eval(&script(STRETCH, &stretch_id));
        let _ = js.send(oxide_charts::zoom::css_stretch(from, at));
        spawn(async move {
            if js.join::<u8>().await.is_ok() && *moves.peek() == this {
                tick.with_mut(|t| *t = t.wrapping_add(1));
            }
        });
    });

    rsx! {
        div { "data-part": "canvas-clip", "aria-hidden": "true",
        canvas {
            id: "{id}",
            "data-part": "canvas",
            onmounted: move |_| {
                let _ = watch.send("start");
            },
        }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::base64;

    #[test]
    fn base64_pads_short_tails() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"M"), "TQ==");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(&[0xff, 0xfe, 0xfd, 0x00]), "//79AA==");
    }
}
