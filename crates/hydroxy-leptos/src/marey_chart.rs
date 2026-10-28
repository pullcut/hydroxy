//! `<MareyChart>`: a train graph. Stations run down the side, spaced by distance, time
//! runs across, and each trip is a line through its stops. Geometry comes from
//! `hydroxy_charts::marey`.

use leptos::prelude::*;
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
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn MareyChart(
    /// The stations in line order, each at its distance from the start.
    #[prop(into)]
    stations: Signal<Vec<Station>>,
    /// The trips: `Trip::new(name, service, [(station index, seconds since midnight), ...])`.
    #[prop(into)]
    trips: Signal<Vec<Trip>>,
    /// What the chart shows, such as "Weekday morning departures". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the time labels and legend.
    #[prop(default = 320)]
    height: u16,
) -> impl IntoView {
    let count = move || trips.with(Vec::len);
    let layout = Memo::new(move |_| stations.with(|s| trips.with(|t| marey::layout(s, t, 8))));
    let active = RwSignal::new(None::<usize>);
    let pointer = RwSignal::new(None::<(f64, f64)>);

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let trip = trips.with(|t| t.get(i).cloned())?;
        let (x, y) = pointer.get().or_else(|| layout.with(|l| l.trips.get(i).and_then(|t| t.points.get(t.points.len() / 2).copied())))?;
        let colour = layout.with(|l| colour(l.trips[i].service));
        let rows = stations.with(|s| {
            trip.stops
                .iter()
                .filter_map(|&(k, t)| Some(TipRow::new(Some(colour.clone()), s.get(k)?.name.clone(), marey::clock(t))))
                .collect()
        });
        Some(Tip { x, y, title: format!("{} · {}", trip.name, trip.service), rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|p| layout.with(|l| marey::nearest_trip(&l.trips, p, ASPECT, REACH)));
        pointer.set(p.filter(|_| next.is_some()));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        list_key(&key, active.get_untracked(), count())
            .map(|next| {
                pointer.set(None);
                active.set(next);
            })
            .is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.stations.clone()));
    let labels = Signal::derive(move || layout.with(|l| l.times.clone()));
    let legend = Signal::derive(move || layout.with(|l| if l.services.len() < 2 { Vec::new() } else { l.services.iter().enumerate().map(|(i, s)| Key::new(s.clone(), colour(i))).collect() }));
    let head = Signal::derive(move || {
        let mut h = vec!["Trip".to_string(), "Service".to_string()];
        stations.with(|s| h.extend(s.iter().map(|s| s.name.clone())));
        h
    });
    let rows = Signal::derive(move || {
        let n = stations.with(Vec::len);
        trips.with(|t| {
            t.iter()
                .map(|t| {
                    let mut r = vec![t.name.clone(), t.service.clone()];
                    r.extend((0..n).map(|k| t.stops.iter().find(|s| s.0 == k).map_or_else(|| "–".to_string(), |s| marey::clock(s.1))));
                    r
                })
                .collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        let paths = l
            .trips
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                let is_active = move || (active.get() == Some(i)).then_some("");
                view! { <path data-part="trip" data-active=is_active d=t.path stroke=colour(t.service) style=format!("--i: {i};")></path> }
            })
            .collect_view();
        view! { <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{paths}</svg> }
    };
    let overlay = move || {
        let i = active.get()?;
        let (points, c) = layout.with(|l| l.trips.get(i).map(|t| (t.points.clone(), colour(t.service))))?;
        Some(
            points
                .into_iter()
                .map(|(x, y)| view! { <span data-part="stop" style=format!("left: {}; bottom: {}; background: {c};", pct(x), pct(y))></span> })
                .collect_view(),
        )
    };

    view! {
        <Plot
            scope="marey-chart"
            label
            hint="Use the left and right arrow keys to step through trips."
            height
            ticks
            labels
            annotations=Signal::derive(Vec::new)
            legend
            tip
            head
            rows
            on_key
            on_point
            marks
            overlay
        />
    }
}
