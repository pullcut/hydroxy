//! `<BarRaceChart>`: the top entries ranked frame by frame, playing through time. Bars
//! glide past each other and their values count. Geometry comes from `hydroxy_charts::race`.

use std::time::Duration;

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot;
use hydroxy_charts::race;
use hydroxy_headless::carousel::{PAUSE_MARKUP, PLAY_MARKUP};
use leptos::prelude::*;

pub use hydroxy_charts::line::Series;

use crate::callbacks::{select_active, watch_hover};
use crate::js::prefers_reduced_motion;
use crate::plot_kit::{colour, formatter, pct};

/// How often the race moves while playing, in ms. Bars and values tween between ticks.
const TICK_MS: u32 = 80;

/// A ranking over time: the `top` largest entries as horizontal bars, re-ranked every
/// frame. Play, pause and scrub with the controls (Left/Right on the scrubber step a frame,
/// Home/End jump). Under reduced motion it plays frame by frame without tweening, and
/// never starts by itself.
#[component]
pub fn BarRaceChart(
    /// One label per frame, such as years.
    #[prop(into)]
    frames: Signal<Vec<String>>,
    /// The entries, one value per frame. NaN counts as zero. Six or fewer get their own
    /// colours; more share one, so colour never repeats for different entries.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// What the chart shows, such as "Most spoken languages". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// How many bars show at once.
    #[prop(default = 10)]
    top: usize,
    /// How long each frame lasts while playing, in ms.
    #[prop(default = 1200)]
    frame_ms: u32,
    /// Start playing once mounted (never under reduced motion).
    #[prop(optional)]
    autoplay: bool,
    /// Row height in px.
    #[prop(default = 30)]
    row_height: u16,
    /// Formats values for the labels, axis and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// The frame to show, to control the race from outside. Changing it moves the race there
    /// without calling `on_frame_change`. Default: the chart keeps its own position.
    #[prop(optional, into)]
    frame: MaybeProp<usize>,
    /// Called with the frame the race lands on when it plays, pauses or is scrubbed.
    #[prop(optional, into)]
    on_frame_change: Option<Callback<usize>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || frames.with(Vec::len);
    let last = move || count().saturating_sub(1);
    let position = RwSignal::new(0.0_f64);
    let playing = RwSignal::new(false);
    let reduced = RwSignal::new(false);
    #[allow(clippy::cast_precision_loss, reason = "frame counts are small")]
    let end = move || last() as f64;
    let state = Memo::new(move |_| series.with(|s| race::race(s, count(), position.get(), 4)));
    let frame_now = Memo::new(move |_| state.with(|s| s.frame));
    // A controlled `frame` moves the race; the change callback skips frames that came from it.
    Effect::new(move |_| {
        if let Some(f) = frame.get() {
            let f = f.min(last());
            if untrack(|| frame_now.get()) != f {
                #[allow(clippy::cast_precision_loss, reason = "frame counts are small")]
                position.set(f as f64);
            }
        }
    });
    if let Some(cb) = on_frame_change {
        Effect::new(move |prev: Option<usize>| {
            let f = frame_now.get();
            if prev.is_some_and(|p| p != f) && frame.get_untracked() != Some(f) {
                untrack(|| cb.run(f));
            }
            f
        });
    }
    // The hovered row; its value is read untracked so the race ticking is not a new hover.
    let hovered = RwSignal::new(None::<usize>);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = hovered.get()?;
            series.with(|all| {
                let name = all.get(i)?.name.clone();
                let value = state.with_untracked(|s| s.bars.get(i).map(|b| b.value))?;
                Some(EventDatum { series: i, ..EventDatum::category(frame_now.get_untracked(), name, value) })
            })
        })
    });
    watch_hover(datum, on_hover);
    let frame_label = move || frames.with(|f| f.get(state.with(|s| s.frame)).cloned().unwrap_or_default());

    // The clock: ticks while playing, cleared on pause and unmount.
    let handle = StoredValue::new(None::<IntervalHandle>);
    let halt = move || {
        if let Some(h) = handle.try_get_value().flatten() {
            h.clear();
            handle.set_value(None);
        }
    };
    Effect::new(move |_| {
        halt();
        if !playing.get() {
            return;
        }
        let jump = reduced.get_untracked();
        let (every, step) =
            if jump { (frame_ms, 1.0) } else { (TICK_MS, f64::from(TICK_MS) / f64::from(frame_ms.max(1))) };
        let tick = move || {
            let next = if jump { (position.get_untracked() + step).floor() } else { position.get_untracked() + step }
                .min(end());
            position.set(next);
            if next >= end() {
                playing.set(false);
            }
        };
        if let Ok(h) = set_interval_with_handle(tick, Duration::from_millis(every.into())) {
            handle.set_value(Some(h));
        }
    });
    on_cleanup(halt);
    Effect::new(move |prev: Option<()>| {
        if prev.is_none() {
            reduced.set(prefers_reduced_motion());
            if autoplay && !reduced.get_untracked() && count() > 1 {
                playing.set(true);
            }
        }
    });
    let toggle = move |_| {
        if !playing.get_untracked() && position.get_untracked() >= end() {
            position.set(0.0);
        }
        playing.update(|p| *p = !*p);
    };
    let scrub = move |ev: leptos::ev::Event| {
        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
            playing.set(false);
            position.set(v.clamp(0.0, end()));
        }
    };
    let scrub_key = move |ev: leptos::ev::KeyboardEvent| {
        let at = position.get_untracked();
        let Some(next) = race::scrub_key(&ev.key(), at, end()) else {
            return;
        };
        ev.prevent_default();
        playing.set(false);
        position.set(next);
    };

    let rows = move || {
        series.with(|all| {
            let many = all.len() > 6;
            all.iter()
                .enumerate()
                .map(|(i, s)| {
                    let name = s.name.clone();
                    let bar = move || state.with(|st| st.bars.get(i).copied());
                    let style = move || {
                        let b = bar().unwrap_or(race::RaceBar { value: 0.0, rank: top, fraction: 0.0 });
                        format!(
                            "--rank: {}; --f: {:.4}; --c: {};",
                            b.rank.min(top),
                            b.fraction,
                            colour(if many { 0 } else { i })
                        )
                    };
                    let shown = move || bar().is_some_and(|b| b.rank < top);
                    let out = move || bar().is_none_or(|b| b.rank >= top).then_some("");
                    let value = move || bar().map(|b| fmt(b.value)).unwrap_or_default();
                    view! {
                        <div
                            data-part="row"
                            data-out=out
                            style=style
                            on:pointerenter=move |_| hovered.set(shown().then_some(i))
                            on:pointerleave=move |_| hovered.set(None)
                            on:click=move |_| {
                                hovered.set(shown().then_some(i));
                                select_active(datum, on_select);
                            }
                        >
                            <span data-part="name" title=name>{name.clone()}</span>
                            <span data-part="track">
                                <span data-part="bar"></span>
                                <span data-part="value">{value}</span>
                            </span>
                        </div>
                    }
                })
                .collect_view()
        })
    };
    // Gridlines are keyed by value, so as the axis rescales they slide rather than redraw,
    // and a new tick fades in where it appears.
    let grid = move || {
        view! {
            <For each=move || state.with(|s| s.ticks.iter().map(|t| t.0.to_bits()).collect::<Vec<u64>>()) key=|bits| *bits let:bits>
                {
                    let v = f64::from_bits(bits);
                    let at = move || state.with(|s| s.ticks.iter().find(|t| t.0.to_bits() == bits).map_or(1.0, |t| t.1));
                    view! {
                        <div data-part="gridline" data-baseline=(v == 0.0).then_some("") style=move || format!("--p: {};", pct(at()))>
                            <span data-part="tick">{fmt(v)}</span>
                        </div>
                    }
                }
            </For>
        }
    };
    // Announce the standings when the race stops, not on every tick.
    let announce = move || {
        if playing.get() {
            return String::new();
        }
        let leaders = series.with(|all| {
            state.with(|s| {
                let mut ranked: Vec<(usize, usize)> = s.bars.iter().enumerate().map(|(i, b)| (b.rank, i)).collect();
                ranked.sort_unstable();
                ranked
                    .iter()
                    .take(3)
                    .map(|&(r, i)| format!("{}. {} {}", r + 1, all[i].name, fmt(s.bars[i].value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        });
        format!("{}: {leaders}", frame_label())
    };
    let head = move || frames.get().into_iter().map(|f| view! { <th scope="col">{f}</th> }).collect_view();
    let table = move || {
        series
            .get()
            .into_iter()
            .map(|s| {
                let n = count();
                view! {
                    <tr>
                        <th scope="row">{s.name}</th>
                        {(0..n).map(|k| view! { <td>{plot::cell(s.values.get(k).copied(), &fmt)}</td> }).collect_view()}
                    </tr>
                }
            })
            .collect_view()
    };
    let caption = label.clone();

    view! {
        <figure
            data-scope="bar-race-chart"
            data-part="root"
            data-playing=move || playing.get().then_some("")
            role="group"
            aria-label=label
            style=format!("--rows: {top}; --row: {row_height}px;")
        >
            <div data-part="plot">
                <div data-part="grid" aria-hidden="true">{grid}</div>
                <div data-part="bars" aria-hidden="true">{rows}</div>
                <span data-part="frame" aria-hidden="true">{frame_label}</span>
            </div>
            <div data-part="controls">
                <button
                    type="button"
                    data-part="play"
                    aria-label=move || if playing.get() { "Pause" } else { "Play" }
                    disabled=move || count() < 2
                    on:click=toggle
                    inner_html=move || if playing.get() { PAUSE_MARKUP } else { PLAY_MARKUP }
                ></button>
                <input
                    type="range"
                    data-part="scrubber"
                    min="0"
                    max=move || last().to_string()
                    step="any"
                    aria-label="Frame"
                    aria-valuetext=frame_label
                    disabled=move || count() < 2
                    prop:value=move || format!("{:.3}", position.get())
                    style=move || format!("--ox-played: {};", pct(if end() > 0.0 { position.get() / end() } else { 0.0 }))
                    on:input=scrub
                    on:keydown=scrub_key
                />
                <span data-part="frame-range" aria-hidden="true">
                    {move || frames.with(|f| format!("{} – {}", f.first().cloned().unwrap_or_default(), f.last().cloned().unwrap_or_default()))}
                </span>
            </div>
            <span data-part="sr-live" aria-live="polite">{announce}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col"></th>
                        {head}
                    </tr>
                </thead>
                <tbody>{table}</tbody>
            </table>
        </figure>
    }
}
