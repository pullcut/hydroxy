//! `BarRaceChart`: the top entries ranked frame by frame, playing through time. Bars glide
//! past each other and their values count. Geometry comes from `hydroxy_charts::race`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{self, colour};
use hydroxy_charts::race;
use hydroxy_headless::carousel::{PAUSE_MARKUP, PLAY_MARKUP};
use std::cell::Cell;
use std::rc::Rc;

pub use hydroxy_charts::line::Series;

use crate::plot_kit::{formatter, pct};

/// How often the race moves while playing, in ms. Bars and values tween between ticks.
const TICK_MS: u32 = 80;

/// The playhead and its clock. The clock runs through eval, so it works on web and desktop.
#[derive(Clone, Copy)]
struct Clock {
    position: Signal<f64>,
    playing: Signal<bool>,
    reduced: Signal<bool>,
    /// Bumped on every start, so an older loop sees it is stale and stops.
    run: Signal<u32>,
    frame_ms: u32,
}

impl Clock {
    fn start(mut self, end: f64) {
        if *self.position.peek() >= end {
            self.position.set(0.0);
        }
        let run = self.run.peek().wrapping_add(1);
        self.run.set(run);
        self.playing.set(true);
        let jump = *self.reduced.peek();
        let (every, step) =
            if jump { (self.frame_ms, 1.0) } else { (TICK_MS, f64::from(TICK_MS) / f64::from(self.frame_ms.max(1))) };
        spawn(async move {
            loop {
                let js = format!("await new Promise(r => setTimeout(r, {every})); return true;");
                let _ = document::eval(&js).join::<bool>().await;
                if *self.run.peek() != run || !*self.playing.peek() {
                    break;
                }
                let at = *self.position.peek();
                let next = if jump { (at + step).floor() } else { at + step }.min(end);
                self.position.set(next);
                if next >= end {
                    self.playing.set(false);
                    break;
                }
            }
        });
    }

    fn seek(mut self, at: f64) {
        self.playing.set(false);
        self.position.set(at);
    }
}

/// The whole frame the playhead is nearest to, as `race::race` reports it.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "frame positions are small and clamped"
)]
fn frame_at(position: f64, last: usize) -> usize {
    (position.clamp(0.0, last as f64).round() as usize).min(last)
}

/// A ranking over time: the `top` largest entries as horizontal bars, re-ranked every
/// frame. Play, pause and scrub with the controls (Left/Right on the scrubber step a frame,
/// Home/End jump). Under reduced motion it plays frame by frame without tweening, and
/// never starts by itself.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn BarRaceChart(
    /// One label per frame, such as years.
    frames: ReadSignal<Vec<String>>,
    /// The entries, one value per frame. NaN counts as zero. Six or fewer get their own
    /// colours; more share one, so colour never repeats for different entries.
    series: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Most spoken languages". Used as the accessible name.
    label: String,
    /// How many bars show at once.
    #[props(default = 10)]
    top: usize,
    /// How long each frame lasts while playing, in ms.
    #[props(default = 1200)]
    frame_ms: u32,
    /// Start playing once mounted (never under reduced motion).
    #[props(default)]
    autoplay: bool,
    /// Row height in px.
    #[props(default = 30)]
    row_height: u16,
    /// Formats values for the labels, axis and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// The frame to show, to control the race from outside. Changing it moves the race there
    /// without calling `on_frame_change`. Default: the chart keeps its own position.
    frame: Option<ReadSignal<usize>>,
    /// Called with the frame the race lands on when it plays, pauses or is scrubbed.
    on_frame_change: Option<EventHandler<usize>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let labels = frames();
    let entries = series();
    let count = labels.len();
    #[allow(clippy::cast_precision_loss, reason = "frame counts are small")]
    let end = count.saturating_sub(1) as f64;
    let clock = Clock {
        position: use_signal(|| 0.0),
        playing: use_signal(|| false),
        reduced: use_signal(|| false),
        run: use_signal(|| 0),
        frame_ms,
    };
    let mut reduced = clock.reduced;
    use_future(move || async move {
        let r = document::eval("return matchMedia('(prefers-reduced-motion: reduce)').matches;")
            .join::<bool>()
            .await
            .unwrap_or(true);
        reduced.set(r);
        if autoplay && !r && end > 0.0 {
            clock.start(end);
        }
    });
    let mut playing = clock.playing;
    let at = (clock.position)();
    let is_playing = playing();
    let state = race::race(&entries, count, at, 4);
    let frame_now = state.frame;
    // A controlled `frame` moves the race there.
    let mut position = clock.position;
    use_effect(move || {
        if let Some(f) = frame {
            let last = frames.peek().len().saturating_sub(1);
            let f = f().min(last);
            if frame_at(*position.peek(), last) != f {
                #[allow(clippy::cast_precision_loss, reason = "frame counts are small")]
                position.set(f as f64);
            }
        }
    });
    // Tell `on_frame_change` when the frame changes, but not when the change came from `frame`.
    let landed = use_memo(use_reactive((&frame_now,), move |(f,)| f));
    let seen = use_hook(|| Rc::new(Cell::new(None::<usize>)));
    use_effect(move || {
        let f = landed();
        let prev = seen.replace(Some(f));
        let controlled = frame.map(|s| *s.peek());
        if let (Some(prev), Some(cb)) = (prev, on_frame_change)
            && prev != f
            && controlled != Some(f)
        {
            cb.call(f);
        }
    });
    // The hovered row. Its datum is read when the row changes, so the race ticking is not a new hover.
    let mut hovered = use_signal(|| None::<usize>);
    let build = move |i: usize| -> Option<EventDatum> {
        let all = series.peek();
        let s = all.get(i)?;
        let f = frame_at(*position.peek(), frames.peek().len().saturating_sub(1));
        let value = race::values_at(&all, frames.peek().len(), *position.peek()).get(i).copied().unwrap_or(0.0);
        Some(EventDatum { series: i, ..EventDatum::category(f, s.name.clone(), value) })
    };
    let hovered_now = if on_hover.is_some() || on_select.is_some() { hovered() } else { None };
    let datum: ReadSignal<Option<EventDatum>> =
        use_memo(use_reactive((&hovered_now,), move |(h,)| h.and_then(build))).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let frame_label = labels.get(state.frame).cloned().unwrap_or_default();
    let many = entries.len() > 6;
    let announce = if is_playing {
        String::new()
    } else {
        let mut ranked: Vec<(usize, usize)> = state.bars.iter().enumerate().map(|(i, b)| (b.rank, i)).collect();
        ranked.sort_unstable();
        let leaders = ranked
            .iter()
            .take(3)
            .map(|&(r, i)| format!("{}. {} {}", r + 1, entries[i].name, fmt(state.bars[i].value)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{frame_label}: {leaders}")
    };
    let played = pct(if end > 0.0 { at / end } else { 0.0 });
    let span =
        format!("{} – {}", labels.first().cloned().unwrap_or_default(), labels.last().cloned().unwrap_or_default());

    rsx! {
        figure {
            "data-scope": "bar-race-chart",
            "data-part": "root",
            "data-playing": is_playing.then_some(""),
            role: "group",
            "aria-label": "{label}",
            style: "--rows: {top}; --row: {row_height}px;",
            div { "data-part": "plot",
                div { "data-part": "grid", "aria-hidden": "true",
                    // Keyed by value, so as the axis rescales gridlines slide and new ones fade in.
                    for (v , f) in state.ticks.iter() {
                        div { key: "{v}", "data-part": "gridline", "data-baseline": (*v == 0.0).then_some(""), style: "--p: {pct(*f)};",
                            span { "data-part": "tick", {fmt(*v)} }
                        }
                    }
                }
                div { "data-part": "bars", "aria-hidden": "true",
                    for (i , (s , b)) in entries.iter().zip(&state.bars).enumerate() {
                        div {
                            key: "{i}",
                            "data-part": "row",
                            "data-out": (b.rank >= top).then_some(""),
                            style: format!("--rank: {}; --f: {:.4}; --c: {};", b.rank.min(top), b.fraction, colour(if many { 0 } else { i })),
                            onpointerenter: {
                                let shown = b.rank < top;
                                move |_| hovered.set(shown.then_some(i))
                            },
                            onpointerleave: move |_| hovered.set(None),
                            onclick: {
                                let shown = b.rank < top;
                                move |_| {
                                    hovered.set(shown.then_some(i));
                                    if shown && let (Some(cb), Some(d)) = (on_select, build(i)) {
                                        cb.call(d);
                                    }
                                }
                            },
                            span { "data-part": "name", title: "{s.name}", "{s.name}" }
                            span { "data-part": "track",
                                span { "data-part": "bar" }
                                span { "data-part": "value", {fmt(b.value)} }
                            }
                        }
                    }
                }
                span { "data-part": "frame", "aria-hidden": "true", "{frame_label}" }
            }
            div { "data-part": "controls",
                button {
                    r#type: "button",
                    "data-part": "play",
                    "aria-label": if is_playing { "Pause" } else { "Play" },
                    disabled: count < 2,
                    dangerous_inner_html: if is_playing { PAUSE_MARKUP } else { PLAY_MARKUP },
                    onclick: move |_| {
                        if playing() {
                            playing.set(false);
                        } else {
                            clock.start(end);
                        }
                    },
                }
                input {
                    r#type: "range",
                    "data-part": "scrubber",
                    min: "0",
                    max: "{end}",
                    step: "any",
                    "aria-label": "Frame",
                    "aria-valuetext": "{frame_label}",
                    disabled: count < 2,
                    value: "{at:.3}",
                    style: "--ox-played: {played};",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            clock.seek(v.clamp(0.0, end));
                        }
                    },
                    onkeydown: move |e| {
                        let at = *clock.position.peek();
                        let Some(next) = race::scrub_key(&e.key().to_string(), at, end) else {
                            return;
                        };
                        e.prevent_default();
                        clock.seek(next);
                    },
                }
                span { "data-part": "frame-range", "aria-hidden": "true", "{span}" }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announce}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col" }
                        for f in labels.iter() {
                            th { scope: "col", "{f}" }
                        }
                    }
                }
                tbody {
                    for s in entries.iter() {
                        tr {
                            th { scope: "row", "{s.name}" }
                            for k in 0..count {
                                td { {plot::cell(s.values.get(k).copied(), fmt)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
