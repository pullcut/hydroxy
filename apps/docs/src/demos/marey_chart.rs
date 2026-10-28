use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::marey_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    // Miles from Reading. Fast trains call at Slough only; stopping trains call everywhere.
    let stations = vec![
        Station::new("Reading", 0.0),
        Station::new("Twyford", 5.0),
        Station::new("Maidenhead", 11.7),
        Station::new("Burnham", 13.3),
        Station::new("Slough", 17.5),
        Station::new("Langley", 19.8),
        Station::new("West Drayton", 22.7),
        Station::new("Hayes", 25.0),
        Station::new("Southall", 26.8),
        Station::new("Ealing Broadway", 30.2),
        Station::new("Paddington", 35.9),
    ];
    // Running time per mile and a one-minute dwell at each call.
    let run = |name: String, service: &str, depart: f64, calls: &[usize], secs_per_mile: f64| {
        let mut t = depart;
        let mut prev = stations[calls[0]].distance;
        let stops: Vec<(usize, f64)> = calls
            .iter()
            .map(|&s| {
                t += (stations[s].distance - prev).abs() * secs_per_mile;
                prev = stations[s].distance;
                let at = t;
                t += 60.0;
                (s, at)
            })
            .collect();
        Trip::new(name, service, stops)
    };
    let all: Vec<usize> = (0..stations.len()).collect();
    let up: Vec<usize> = all.iter().rev().copied().collect();
    let mut trips = Vec::new();
    for k in 0..8 {
        let depart = hm(6, 0) + f64::from(k) * 900.0;
        trips.push(if k % 2 == 0 {
            run(format!("1P{:02}", 10 + k), "Fast", depart, &[0, 4, 10], 45.0)
        } else {
            run(format!("2P{:02}", 10 + k), "Stopping", depart, &all, 95.0)
        });
        trips.push(run(format!("2R{:02}", 10 + k), "Stopping", depart + 420.0, &up, 95.0));
    }

    view! {
        <ChartFrame title="Morning service, Reading – Paddington" description="Each line is one train. Steeper lines are faster; flat steps are station stops.">
            <MareyChart stations trips label="Weekday departures between 06:00 and 08:00" height=340 />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "marey-chart",
    name: "Marey chart",
    group: "Time",
    summary: "A timetable as a picture: stations down the side spaced by distance, time across, and every trip a line through its stops. Speed reads as slope, waits as flat steps, and passing trains as crossings.",
    demo,
    leptos: include_str!("marey_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/marey_chart.rs"),
    parts: &[
        &["MareyChart", "root", "figure with a screen-reader table of every trip's calling times"],
        &["", "trip", "One trip's line; draws along its length, data-active when picked"],
        &["", "stop", "A call on the picked trip"],
        &["", "tooltip", "The trip, its service and each call"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next trip"],
        &["Home / End", "First or last trip"],
        &["Escape", "Clear the picked trip"],
    ],
    props: &[
        &["stations", "Vec<Station>", "Required. Station::new(name, distance), in line order"],
        &["trips", "Vec<Trip>", "Required. Trip::new(name, service, [(station index, seconds since midnight)])"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 320"],
    ],
};
