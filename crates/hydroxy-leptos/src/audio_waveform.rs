//! `<AudioWaveform>`: a peak display with a playhead, a selectable range and
//! a loop toggle. The host owns the clock; clicks and keys become requests.

use hydroxy_headless::audio_waveform::{self as h, Edge};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

pub use h::{Peaks, Range};

/// Peaks drawn as bars, with a playhead and an optional selection.
///
/// Click or drag the track to seek; arrow keys step the playhead. When a
/// selection is present its edges get sliders and a loop button.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "one view! tree; props are owned by the macro"
)]
pub fn AudioWaveform(
    /// Peak bins and their scale.
    peaks: Peaks,
    /// Length of the audio in seconds.
    #[prop(into)]
    duration: Signal<Option<f64>>,
    /// Playhead position in seconds.
    #[prop(optional, into)]
    current_time: Signal<f64>,
    /// The selected span, if any.
    #[prop(optional, into)]
    selected_range: Signal<Option<Range>>,
    /// Zoom factor, 1 or more. Default 1.
    #[prop(optional, into)]
    zoom: MaybeProp<f64>,
    /// First visible second when zoomed.
    #[prop(optional, into)]
    viewport_start: MaybeProp<f64>,
    /// The selection is looping.
    #[prop(optional, into)]
    looping: Signal<bool>,
    /// Accessible name of the track. Default "Audio waveform".
    #[prop(optional, into)]
    label: Option<String>,
    /// The reader chose this time: seek here.
    #[prop(optional, into)]
    on_seek: Option<Callback<f64>>,
    /// The reader moved a selection edge.
    #[prop(optional, into)]
    on_range_change: Option<Callback<Range>>,
    /// The reader toggled looping; the value is the new state.
    #[prop(optional, into)]
    on_loop_request: Option<Callback<bool>>,
) -> impl IntoView {
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    let peaks = StoredValue::new(peaks);
    let projection = Memo::new(move |_| {
        peaks.with_value(|p| {
            h::project(
                Some(p),
                duration.get(),
                current_time.get(),
                selected_range.get(),
                zoom.get().unwrap_or(1.0),
                viewport_start.get().unwrap_or(0.0),
            )
        })
    });
    let dragging = RwSignal::new(false);

    let seek_at = move |ev: &web_sys::PointerEvent| {
        let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let rect = el.get_bounding_client_rect();
        let fraction = (f64::from(ev.client_x()) - rect.left()) / rect.width().max(1.0);
        let time = projection.with_untracked(|p| h::time_at(fraction, &p.viewport, p.duration));
        if let Some(cb) = on_seek {
            cb.run(time);
        }
    };
    let edge_input = move |edge: Edge, value: String| {
        let (Some(range), Ok(v)) = (selected_range.get_untracked(), value.parse::<f64>()) else { return };
        let moved = match edge {
            Edge::Start => h::step_edge(Range { start: v, end: range.end }, Edge::Start, 0.0, duration.get_untracked()),
            Edge::End => h::step_edge(Range { start: range.start, end: v }, Edge::End, 0.0, duration.get_untracked()),
        };
        if let Some(cb) = on_range_change {
            cb.run(moved);
        }
    };
    let percent = hydroxy_charts::plot::pct;
    let issue = move || {
        projection.with(|p| {
            if p.issues.is_empty() {
                "false".to_string()
            } else {
                p.issues.iter().map(|i| i.as_str()).collect::<Vec<_>>().join(" ")
            }
        })
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-has-peaks=move || projection.with(|p| !p.bars.is_empty()).to_string() data-issue=issue>
            <div
                data-scope=h::SCOPE
                data-part="track"
                role="slider"
                tabindex="0"
                aria-label=label
                aria-valuemin="0"
                aria-valuemax=move || projection.with(|p| p.duration.unwrap_or(0.0)).to_string()
                aria-valuenow=move || projection.with(|p| p.current.floor()).to_string()
                aria-valuetext=move || projection.with(|p| h::value_text(p.current, p.duration))
                data-dragging=move || dragging.get().then_some("")
                on:pointerdown=move |ev| {
                    if ev.button() != 0 {
                        return;
                    }
                    if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                        let _ = el.set_pointer_capture(ev.pointer_id());
                    }
                    dragging.set(true);
                    seek_at(&ev);
                }
                on:pointermove=move |ev| {
                    if dragging.get_untracked() {
                        seek_at(&ev);
                    }
                }
                on:pointerup=move |_| dragging.set(false)
                on:pointercancel=move |_| dragging.set(false)
                on:keydown=move |ev| {
                    let (current, d) = projection.with_untracked(|p| (p.current, p.duration));
                    if let Some(time) = h::track_key(&ev.key(), current, d, h::SEEK_STEP) {
                        ev.prevent_default();
                        if let Some(cb) = on_seek {
                            cb.run(time);
                        }
                    }
                }
            >
                {move || {
                    projection
                        .with(|p| p.range_fractions)
                        .map(|(from, to)| {
                            view! {
                                <span
                                    data-scope=h::SCOPE
                                    data-part="range"
                                    style:left=percent(from)
                                    style:width=percent(to - from)
                                ></span>
                            }
                        })
                }}
                <span data-scope=h::SCOPE data-part="bars" aria-hidden="true">
                    {move || {
                        projection
                            .with(|p| p.bars.clone())
                            .into_iter()
                            .map(|b| {
                                view! {
                                    <span
                                        data-scope=h::SCOPE
                                        data-part="bar"
                                        style:left=percent(b.left)
                                        style:width=percent(b.width)
                                        style:height=format!("{:.1}%", (b.amplitude.max(0.04)) * 100.0)
                                    ></span>
                                }
                            })
                            .collect_view()
                    }}
                </span>
                {move || {
                    projection
                        .with(|p| p.playhead)
                        .map(|f| view! { <span data-scope=h::SCOPE data-part="playhead" aria-hidden="true" style:left=percent(f)></span> })
                }}
            </div>
            {move || {
                selected_range
                    .get()
                    .map(|range| {
                        let d = duration.get();
                        view! {
                            <div data-scope=h::SCOPE data-part="selection">
                                <label>
                                    <span>"Selection start"</span>
                                    <input
                                        type="range"
                                        data-scope=h::SCOPE
                                        data-part="range-start"
                                        min="0"
                                        max=range.end.to_string()
                                        step=h::RANGE_STEP.to_string()
                                        prop:value=range.start.to_string()
                                        aria-valuetext=hydroxy_headless::media_timeline::format_time(range.start)
                                        on:input=move |ev| edge_input(Edge::Start, event_target_value(&ev))
                                    />
                                </label>
                                <label>
                                    <span>"Selection end"</span>
                                    <input
                                        type="range"
                                        data-scope=h::SCOPE
                                        data-part="range-end"
                                        min=range.start.to_string()
                                        max=d.unwrap_or(range.end).to_string()
                                        step=h::RANGE_STEP.to_string()
                                        prop:value=range.end.to_string()
                                        aria-valuetext=hydroxy_headless::media_timeline::format_time(range.end)
                                        on:input=move |ev| edge_input(Edge::End, event_target_value(&ev))
                                    />
                                </label>
                                <button
                                    type="button"
                                    data-scope=h::SCOPE
                                    data-part="loop"
                                    aria-pressed=move || looping.get().to_string()
                                    on:click=move |_| {
                                        if let Some(cb) = on_loop_request {
                                            cb.run(!looping.get_untracked());
                                        }
                                    }
                                >
                                    "Loop selection"
                                </button>
                            </div>
                        }
                    })
            }}
            <p data-scope=h::SCOPE data-part="status" aria-live="polite" data-issue=move || projection.with(|p| !p.issues.is_empty()).to_string()>
                {move || projection.with(h::status_text)}
            </p>
        </div>
    }
}
