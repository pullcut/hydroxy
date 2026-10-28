//! `MediaTimeline`: a scrubber over a native `<audio>` or `<video>` element.
//!
//! Dioxus has no typed handle for media elements, so a small script run through
//! `document::eval` mirrors the element's clock into the machine and applies
//! seek, play and pause. That works on web and in a desktop or mobile webview.
//! While dragging, a fixed full-window layer catches pointer moves.

use std::rc::Rc;

use dioxus::prelude::*;
use oxide_headless::media_timeline::{
    self as h, MediaTimelineCommand, MediaTimelineEvent, MediaTimelineInput, MediaTimelineMachine,
};
use oxide_statechart::Machine;

pub use h::{Marker, MediaKind};

/// Waits for the element, then reports `[time, duration, buffered end, playing]` on
/// every media event and applies `[command, value]` messages.
const SCRIPT: &str = r"
const el = await new Promise((resolve) => {
  const look = () => { const e = document.getElementById(__ID__); e ? resolve(e) : requestAnimationFrame(look); };
  look();
});
const post = () => {
  let buffered = 0;
  try { if (el.buffered.length) buffered = el.buffered.end(el.buffered.length - 1); } catch (_) {}
  dioxus.send([el.currentTime, Number.isFinite(el.duration) ? el.duration : 0, buffered, !el.paused && !el.ended]);
};
['timeupdate', 'durationchange', 'loadedmetadata', 'progress', 'play', 'pause', 'ended'].forEach((n) => el.addEventListener(n, post));
post();
while (true) {
  const [command, value] = await dioxus.recv();
  if (command === 'seek') el.currentTime = value;
  else if (command === 'play') el.play().catch(() => {});
  else if (command === 'pause') el.pause();
}
";

/// A play button, a scrubber with chapter markers, and a native media element
/// underneath. Click or drag the track to seek; the thumb takes arrow keys.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn MediaTimeline(
    /// URL of the audio or video file.
    src: String,
    /// Audio (default) or video. Video shows the picture above the controls.
    #[props(default)]
    kind: MediaKind,
    /// Accessible name for the scrubber, such as "Episode 12".
    label: String,
    /// Chapters or points of interest, as ticks on the track.
    #[props(default)]
    markers: Vec<Marker>,
    /// Seconds an arrow key moves. Default 5.
    #[props(default = 5.0)]
    step: f64,
    /// Block seeking and playback controls.
    #[props(default)]
    disabled: ReadSignal<bool>,
) -> Element {
    let id = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let input = use_memo(move || MediaTimelineInput { step, disabled: disabled() });
    let mut state = use_signal(|| MediaTimelineMachine::initial(&input.peek()));
    let eval = use_hook({
        let id = id.clone();
        move || document::eval(&SCRIPT.replace("__ID__", &format!("{id:?}")))
    });
    let mut track = use_signal(|| None::<Rc<MountedData>>);
    // Track position, read when the pointer arrives and reused while it moves.
    let mut rect = use_signal(|| (0.0_f64, 1.0_f64));
    let mut pressed = use_signal(|| false);

    let mut send = move |event: MediaTimelineEvent| {
        let step = MediaTimelineMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let _ = match command {
                MediaTimelineCommand::Seek(t) => eval.send(("seek", t)),
                MediaTimelineCommand::Play => eval.send(("play", 0.0)),
                MediaTimelineCommand::Pause => eval.send(("pause", 0.0)),
            };
        }
    };
    use_future(move || async move {
        let mut eval = eval;
        while let Ok((time, duration, buffered, playing)) = eval.recv::<(f64, f64, f64, bool)>().await {
            send(MediaTimelineEvent::Duration(duration));
            send(MediaTimelineEvent::Time(time));
            send(MediaTimelineEvent::Playing(playing));
            send(MediaTimelineEvent::Buffered(buffered));
        }
    });
    let time_at = move |x: f64| {
        let (left, width) = rect();
        h::time_at((x - left) / width.max(1.0), state.peek().duration)
    };
    let measure = move || {
        let Some(el) = track.peek().clone() else { return };
        spawn(async move {
            if let Ok(r) = el.get_client_rect().await {
                rect.set((r.origin.x, r.size.width));
            }
        });
    };
    let mut release = move || {
        pressed.set(false);
        send(MediaTimelineEvent::ScrubEnd);
    };

    let s = *state.read();
    let is_disabled = disabled();
    let percent = |seconds: f64| format!("{:.3}%", s.fraction(seconds) * 100.0);
    let chapter = h::chapter_at(&markers, s.current).map(|i| markers[i].label.clone());
    let kind_name = match kind {
        MediaKind::Audio => "audio",
        MediaKind::Video => "video",
    };

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-kind": kind_name,
            "data-state": if s.playing { "playing" } else { "paused" },
            "data-disabled": is_disabled.then_some(""),
            if kind == MediaKind::Video {
                video { id: "{id}", "data-scope": h::SCOPE, "data-part": "media", src: "{src}", preload: "metadata", playsinline: true }
            } else {
                audio { id: "{id}", "data-scope": h::SCOPE, "data-part": "media", src: "{src}", preload: "metadata" }
            }
            div { "data-scope": h::SCOPE, "data-part": "controls",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "play",
                    "aria-label": h::play_label(s.playing),
                    disabled: is_disabled,
                    onclick: move |_| send(MediaTimelineEvent::TogglePlay),
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.75",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "aria-hidden": "true",
                        width: "18",
                        height: "18",
                        dangerous_inner_html: if s.playing { h::PAUSE } else { h::PLAY },
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "current", {h::format_time(s.current)} }
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "track",
                    "data-scrubbing": s.scrubbing.then_some(""),
                    onmounted: move |e| track.set(Some(e.data())),
                    onpointerenter: move |_| measure(),
                    onpointerdown: move |e| {
                        if input.peek().disabled {
                            return;
                        }
                        e.prevent_default();
                        pressed.set(true);
                        let x = e.client_coordinates().x;
                        let Some(el) = track.peek().clone() else { return };
                        spawn(async move {
                            if let Ok(r) = el.get_client_rect().await {
                                rect.set((r.origin.x, r.size.width));
                            }
                            send(MediaTimelineEvent::ScrubStart(time_at(x)));
                            if !pressed() {
                                send(MediaTimelineEvent::ScrubEnd);
                            }
                        });
                    },
                    onpointermove: move |e| send(MediaTimelineEvent::Hover(Some(time_at(e.client_coordinates().x)))),
                    onpointerleave: move |_| send(MediaTimelineEvent::Hover(None)),
                    onpointerup: move |_| release(),
                    span { "data-scope": h::SCOPE, "data-part": "buffered", style: "width: {percent(s.buffered)};" }
                    span { "data-scope": h::SCOPE, "data-part": "played", style: "width: {percent(s.current)};" }
                    for (i , marker) in markers.iter().enumerate() {
                        span {
                            key: "{i}",
                            "data-scope": h::SCOPE,
                            "data-part": "marker",
                            title: "{marker.label}",
                            style: "left: {percent(marker.time)};",
                        }
                    }
                    if let Some(t) = s.hover {
                        span { "data-scope": h::SCOPE, "data-part": "preview", style: "left: {percent(t)};", {h::format_time(t)} }
                    }
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "thumb",
                        role: "slider",
                        tabindex: if is_disabled { "-1" } else { "0" },
                        "aria-label": "{label}",
                        "aria-valuemin": "0",
                        "aria-valuemax": "{s.duration}",
                        "aria-valuenow": "{s.current.floor()}",
                        "aria-valuetext": h::value_text(s.current, s.duration),
                        "aria-disabled": is_disabled.then_some("true"),
                        style: "left: {percent(s.current)};",
                        onkeydown: move |e| {
                            let shift = e.modifiers().shift();
                            if let Some(event) = h::keydown(&e.key().to_string(), shift, step) {
                                e.prevent_default();
                                send(event);
                            }
                        },
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "duration", {h::format_time(s.duration)} }
            }
            if !markers.is_empty() {
                p { "data-scope": h::SCOPE, "data-part": "chapter",
                    if let Some(name) = chapter {
                        "{name}"
                    }
                }
            }
            if s.scrubbing {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| send(MediaTimelineEvent::ScrubMove(time_at(e.client_coordinates().x))),
                    onpointerup: move |_| release(),
                    onpointercancel: move |_| release(),
                }
            }
        }
    }
}
