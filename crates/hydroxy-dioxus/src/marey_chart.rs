//! `MareyChart`: a train graph. Stations run down the side, spaced by distance, time runs
//! across, and each trip is a line through its stops. Geometry comes from `hydroxy_charts::marey`.

use dioxus::prelude::*;
use hydroxy_charts::marey;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{Key, Tip, TipRow, colour};

pub use hydroxy_charts::marey::{Station, Trip, hm};

use crate::plot_kit::{Plot, pct};

/// Plot width over height assumed for hit-testing; the plot stretches.
const ASPECT: f64 = 2.4;
/// How close (as a share of the plot height) the pointer must be to pick a trip.
const REACH: f64 = 0.05;

/// A timetable as a picture: steep lines are fast running, flat runs are dwells, and
/// lines that cross are trains passing. Trips are coloured by service. Hover or touch a
/// line to pick out its trip and read its calls; focus the plot and use Left/Right to step
/// through trips, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn MareyChart(
    /// The stations in line order, each at its distance from the start.
    stations: ReadSignal<Vec<Station>>,
    /// The trips: `Trip::new(name, service, [(station index, seconds since midnight), ...])`.
    trips: ReadSignal<Vec<Trip>>,
    /// What the chart shows, such as "Weekday morning departures". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the time labels and legend.
    #[props(default = 320)]
    height: u16,
) -> Element {
    let line = stations();
    let all = trips();
    let n = all.len();
    let layout = marey::layout(&line, &all, 8);
    let mut active = use_signal(|| None::<usize>);
    let mut pointer = use_signal(|| None::<(f64, f64)>);
    let current = active().filter(|&i| i < n);

    let tip = current.and_then(|i| {
        let trip = &all[i];
        let placed = &layout.trips[i];
        let (x, y) = pointer().or_else(|| placed.points.get(placed.points.len() / 2).copied())?;
        let c = colour(placed.service);
        let rows = trip
            .stops
            .iter()
            .filter_map(|&(k, t)| Some(TipRow::new(Some(c.clone()), line.get(k)?.name.clone(), marey::clock(t))))
            .collect();
        Some(Tip { x, y, title: format!("{} · {}", trip.name, trip.service), rows })
    });
    let legend: Vec<Key> = if layout.services.len() < 2 {
        Vec::new()
    } else {
        layout.services.iter().enumerate().map(|(i, s)| Key::new(s.clone(), colour(i))).collect()
    };
    let mut head = vec!["Trip".to_string(), "Service".to_string()];
    head.extend(line.iter().map(|s| s.name.clone()));
    let rows: Vec<Vec<String>> = all
        .iter()
        .map(|t| {
            let mut r = vec![t.name.clone(), t.service.clone()];
            r.extend(
                (0..line.len())
                    .map(|k| t.stops.iter().find(|s| s.0 == k).map_or_else(|| "–".to_string(), |s| marey::clock(s.1))),
            );
            r
        })
        .collect();
    let placed = layout.trips.clone();

    rsx! {
        Plot {
            scope: "marey-chart",
            label,
            hint: "Use the left and right arrow keys to step through trips.",
            height,
            ticks: layout.stations.clone(),
            labels: layout.times.clone(),
            annotations: Vec::new(),
            legend,
            tip,
            head,
            rows,
            on_key: move |key: String| {
                list_key(&key, active(), n)
                    .map(|next| {
                        pointer.set(None);
                        active.set(next);
                    })
                    .is_some()
            },
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|p| marey::nearest_trip(&placed, p, ASPECT, REACH));
                pointer.set(p.filter(|_| next.is_some()));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    for (i , t) in layout.trips.iter().enumerate() {
                        path {
                            key: "{i}",
                            "data-part": "trip",
                            "data-active": (current == Some(i)).then_some(""),
                            d: "{t.path}",
                            stroke: colour(t.service),
                            style: "--i: {i};",
                        }
                    }
                }
            },
            overlay: current.map(|i| {
                let t = &layout.trips[i];
                let c = colour(t.service);
                rsx! {
                    for (k , (x , y)) in t.points.iter().enumerate() {
                        span { key: "{k}", "data-part": "stop", style: "left: {pct(*x)}; bottom: {pct(*y)}; background: {c};" }
                    }
                }
            }),
        }
    }
}
