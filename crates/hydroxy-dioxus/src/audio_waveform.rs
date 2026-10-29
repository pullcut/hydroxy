//! `AudioWaveform`: a peak display with a playhead, a selectable range and
//! a loop toggle. The host owns the clock; clicks and keys become requests.
//!
//! Dragging uses a full-window layer while the pointer is held, as the media
//! timeline does, because Dioxus has no pointer capture.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::audio_waveform::{self as h, Edge};

pub use h::{Peaks, Range};
use hydroxy_headless::media_timeline::format_time;

/// Peaks drawn as bars, with a playhead and an optional selection.
///
/// Click or drag the track to seek; arrow keys step the playhead. When a
/// selection is present its edges get sliders and a loop button.
#[component]
pub fn AudioWaveform(
    /// Peak bins and their scale.
    peaks: Peaks,
    /// Length of the audio in seconds.
    #[props(default)]
    duration: ReadSignal<Option<f64>>,
    /// Playhead position in seconds.
    #[props(default)]
    current_time: ReadSignal<f64>,
    /// The selected span, if any.
    #[props(default)]
    selected_range: ReadSignal<Option<Range>>,
    /// Zoom factor, 1 or more. Default 1.
    #[props(default = 1.0)]
    zoom: f64,
    /// First visible second when zoomed.
    #[props(default)]
    viewport_start: f64,
    /// The selection is looping.
    #[props(default)]
    looping: bool,
    /// Accessible name of the track. Default "Audio waveform".
    label: Option<String>,
    /// The reader chose this time: seek here.
    on_seek: Option<EventHandler<f64>>,
    /// The reader moved a selection edge.
    on_range_change: Option<EventHandler<Range>>,
    /// The reader toggled looping; the value is the new state.
    on_loop_request: Option<EventHandler<bool>>,
) -> Element {
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    let p = h::project(Some(&peaks), duration(), current_time(), selected_range(), zoom, viewport_start);
    let mut track = use_signal(|| None::<Rc<MountedData>>);
    let mut rect = use_signal(|| (0.0_f64, 1.0_f64));
    let mut dragging = use_signal(|| false);

    let (view, total) = (p.viewport, p.duration);
    let time_at = move |x: f64| {
        let (left, width) = rect();
        h::time_at((x - left) / width.max(1.0), &view, total)
    };
    let seek = move |t: f64| {
        if let Some(handler) = on_seek {
            handler.call(t);
        }
    };
    let percent = hydroxy_charts::plot::pct;
    let issue = if p.issues.is_empty() {
        "false".to_string()
    } else {
        p.issues.iter().map(|i| i.as_str()).collect::<Vec<_>>().join(" ")
    };
    let range = p.range;
    let current = p.current;
    let edge = move |edge: Edge, value: String| {
        let (Some(range), Ok(v)) = (selected_range.peek().as_ref().copied(), value.parse::<f64>()) else { return };
        let candidate = match edge {
            Edge::Start => Range { start: v, end: range.end },
            Edge::End => Range { start: range.start, end: v },
        };
        if let Some(handler) = on_range_change {
            handler.call(h::step_edge(candidate, edge, 0.0, duration.peek().as_ref().copied()));
        }
    };

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-has-peaks": (!p.bars.is_empty()).to_string(),
            "data-issue": issue,
            div {
                "data-scope": h::SCOPE,
                "data-part": "track",
                role: "slider",
                tabindex: "0",
                "aria-label": label,
                "aria-valuemin": "0",
                "aria-valuemax": "{total.unwrap_or(0.0)}",
                "aria-valuenow": "{current.floor()}",
                "aria-valuetext": h::value_text(current, total),
                "data-dragging": dragging().then_some(""),
                onmounted: move |e| track.set(Some(e.data())),
                onpointerdown: move |e| {
                    e.prevent_default();
                    dragging.set(true);
                    let x = e.client_coordinates().x;
                    let Some(el) = track.peek().clone() else { return };
                    spawn(async move {
                        if let Ok(r) = el.get_client_rect().await {
                            rect.set((r.origin.x, r.size.width));
                        }
                        seek(time_at(x));
                    });
                },
                onkeydown: move |e| {
                    if let Some(t) = h::track_key(&e.key().to_string(), current, total, h::SEEK_STEP) {
                        e.prevent_default();
                        seek(t);
                    }
                },
                if let Some((from, to)) = p.range_fractions {
                    span { "data-scope": h::SCOPE, "data-part": "range", style: "left: {percent(from)}; width: {percent(to - from)};" }
                }
                span { "data-scope": h::SCOPE, "data-part": "bars", "aria-hidden": "true",
                    for bar in p.bars.iter() {
                        span {
                            key: "{bar.index}",
                            "data-scope": h::SCOPE,
                            "data-part": "bar",
                            style: "left: {percent(bar.left)}; width: {percent(bar.width)}; height: {bar.amplitude.max(0.04) * 100.0:.1}%;",
                        }
                    }
                }
                if let Some(f) = p.playhead {
                    span { "data-scope": h::SCOPE, "data-part": "playhead", "aria-hidden": "true", style: "left: {percent(f)};" }
                }
            }
            if dragging() {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| seek(time_at(e.client_coordinates().x)),
                    onpointerup: move |_| dragging.set(false),
                    onpointercancel: move |_| dragging.set(false),
                }
            }
            if let Some(r) = range {
                div { "data-scope": h::SCOPE, "data-part": "selection",
                    label {
                        span { "Selection start" }
                        input {
                            r#type: "range",
                            "data-scope": h::SCOPE,
                            "data-part": "range-start",
                            min: "0",
                            max: "{r.end}",
                            step: "{h::RANGE_STEP}",
                            value: "{r.start}",
                            "aria-valuetext": format_time(r.start),
                            oninput: move |e| edge(Edge::Start, e.value()),
                        }
                    }
                    label {
                        span { "Selection end" }
                        input {
                            r#type: "range",
                            "data-scope": h::SCOPE,
                            "data-part": "range-end",
                            min: "{r.start}",
                            max: "{total.unwrap_or(r.end)}",
                            step: "{h::RANGE_STEP}",
                            value: "{r.end}",
                            "aria-valuetext": format_time(r.end),
                            oninput: move |e| edge(Edge::End, e.value()),
                        }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "loop",
                        "aria-pressed": looping.to_string(),
                        onclick: move |_| {
                            if let Some(handler) = on_loop_request {
                                handler.call(!looping);
                            }
                        },
                        "Loop selection"
                    }
                }
            }
            p {
                "data-scope": h::SCOPE,
                "data-part": "status",
                "aria-live": "polite",
                "data-issue": (!p.issues.is_empty()).to_string(),
                {h::status_text(&p)}
            }
        }
    }
}
