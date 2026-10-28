use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::marey_chart::{MareyChart, Station, Trip, hm};

#[component]
pub fn MareyChartDemo() -> Element {
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

    rsx! {
        ChartFrame { title: "Morning service, Reading – Paddington", description: "Each line is one train. Steeper lines are faster; flat steps are station stops.",
            MareyChart { stations, trips, label: "Weekday departures between 06:00 and 08:00", height: 340 }
        }
    }
}
