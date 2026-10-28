//! Marey (train graph) charts: stations down the side, spaced by distance, time across,
//! and each trip a line through its stops. Steep lines are fast, flat runs are dwells.

use std::fmt::Write as _;

use crate::plot::Tick;
use crate::scale;

/// A station on the line, at its distance from the start.
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    /// Shown on the axis, tooltip and table.
    pub name: String,
    /// Distance along the line, in any unit (km, miles). Sets the spacing.
    pub distance: f64,
}

impl Station {
    /// A station `distance` along the line.
    #[must_use]
    pub fn new(name: impl Into<String>, distance: f64) -> Self {
        Self { name: name.into(), distance }
    }
}

/// One timetabled trip: the stations it calls at, with times.
#[derive(Debug, Clone, PartialEq)]
pub struct Trip {
    /// Headcode or trip id, such as "1A05".
    pub name: String,
    /// Service pattern, such as "Express". Trips sharing one share a colour.
    pub service: String,
    /// `(station index, seconds since midnight)` in calling order.
    pub stops: Vec<(usize, f64)>,
}

impl Trip {
    /// A trip calling at `stops`, each `(station index, seconds since midnight)`.
    #[must_use]
    pub fn new(name: impl Into<String>, service: impl Into<String>, stops: impl IntoIterator<Item = (usize, f64)>) -> Self {
        Self { name: name.into(), service: service.into(), stops: stops.into_iter().collect() }
    }
}

/// Seconds since midnight for `h:m`, for building timetables.
#[must_use]
pub fn hm(h: u32, m: u32) -> f64 {
    f64::from(h * 3600 + m * 60)
}

/// Seconds since midnight as "06:42".
#[allow(clippy::cast_possible_truncation, reason = "times of day are small")]
#[must_use]
pub fn clock(secs: f64) -> String {
    let m = (secs / 60.0).round() as i64;
    format!("{:02}:{:02}", m.div_euclid(60).rem_euclid(24), m.rem_euclid(60))
}

/// One trip, laid out.
#[derive(Debug, Clone, PartialEq)]
pub struct MareyTrip {
    /// Each valid stop as `(x, y)` fractions of the plot, y from the bottom.
    pub points: Vec<(f64, f64)>,
    /// SVG `d` in the 100×100 drawing box.
    pub path: String,
    /// Index into [`MareyLayout::services`], for the colour.
    pub service: usize,
}

/// Everything a Marey chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct MareyLayout {
    /// One per trip, in input order.
    pub trips: Vec<MareyTrip>,
    /// Station positions as axis ticks (first station at the top).
    pub stations: Vec<Tick>,
    /// Distinct service names in order of first appearance.
    pub services: Vec<String>,
    /// Time ticks along the bottom.
    pub times: Vec<Tick>,
}

/// Lay out a train graph with about `tick_count` time ticks.
#[must_use]
pub fn layout(stations: &[Station], trips: &[Trip], tick_count: usize) -> MareyLayout {
    let (mut d0, mut d1) = (f64::INFINITY, f64::NEG_INFINITY);
    for s in stations.iter().filter(|s| s.distance.is_finite()) {
        d0 = d0.min(s.distance);
        d1 = d1.max(s.distance);
    }
    let (mut t0, mut t1) = (f64::INFINITY, f64::NEG_INFINITY);
    for &(_, t) in trips.iter().flat_map(|t| &t.stops).filter(|(s, t)| *s < stations.len() && t.is_finite()) {
        t0 = t0.min(t);
        t1 = t1.max(t);
    }
    if !t0.is_finite() {
        (t0, t1) = (0.0, 3600.0);
    }
    if t1 <= t0 {
        t1 = t0 + 3600.0;
    }
    let y = |d: f64| if d1 > d0 { 1.0 - (d - d0) / (d1 - d0) } else { 0.5 };
    let x = |t: f64| (t - t0) / (t1 - t0);

    let mut services: Vec<String> = Vec::new();
    let trips = trips
        .iter()
        .map(|trip| {
            let service = services.iter().position(|s| *s == trip.service).unwrap_or_else(|| {
                services.push(trip.service.clone());
                services.len() - 1
            });
            let points: Vec<(f64, f64)> = trip
                .stops
                .iter()
                .filter_map(|&(s, t)| stations.get(s).filter(|s| s.distance.is_finite() && t.is_finite()).map(|s| (x(t), y(s.distance))))
                .collect();
            let mut path = String::new();
            for (i, (px, py)) in points.iter().enumerate() {
                let _ = write!(path, "{}{:.3} {:.3}", if i == 0 { 'M' } else { 'L' }, px * 100.0, (1.0 - py) * 100.0);
            }
            if points.len() == 1 {
                path.push_str("h0");
            }
            MareyTrip { points, path, service }
        })
        .collect();
    let stations = stations.iter().filter(|s| s.distance.is_finite()).map(|s| Tick { at: y(s.distance), text: s.name.clone(), baseline: false }).collect();
    let times = scale::time_ticks(t0, t1, tick_count).into_iter().map(|t| Tick { at: x(t.at), text: clock(t.at), baseline: false }).collect();
    MareyLayout { trips, stations, services, times }
}

/// The trip whose line passes nearest `p` (plot fractions, y from the bottom), if any is
/// within `within`. `aspect` is the plot's width over its height, so distances are even.
#[must_use]
pub fn nearest_trip(trips: &[MareyTrip], p: (f64, f64), aspect: f64, within: f64) -> Option<usize> {
    let a = if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 };
    let scaled = |(x, y): (f64, f64)| (x * a, y);
    let p = scaled(p);
    let mut best = None;
    let mut best_d = within * within;
    for (i, trip) in trips.iter().enumerate() {
        let pts: Vec<(f64, f64)> = trip.points.iter().copied().map(scaled).collect();
        let d = match pts.as_slice() {
            [] => continue,
            [only] => dist2(p, *only),
            _ => pts.windows(2).map(|w| segment_dist2(p, w[0], w[1])).fold(f64::INFINITY, f64::min),
        };
        if d <= best_d {
            best_d = d;
            best = Some(i);
        }
    }
    best
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

fn segment_dist2(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0) };
    dist2(p, (a.0 + t * dx, a.1 + t * dy))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Vec<Station> {
        vec![Station::new("A", 0.0), Station::new("B", 30.0), Station::new("C", 100.0)]
    }

    #[test]
    fn stations_spaced_by_distance_first_at_top() {
        let l = layout(&line(), &[], 5);
        assert_eq!(l.stations.len(), 3);
        assert!((l.stations[0].at - 1.0).abs() < 1e-12);
        assert!((l.stations[1].at - 0.7).abs() < 1e-12);
        assert!(l.stations[2].at.abs() < 1e-12);
    }

    #[test]
    fn trips_run_from_first_to_last_time() {
        let trips = [Trip::new("1", "Fast", [(0, hm(6, 0)), (2, hm(7, 0))]), Trip::new("2", "Slow", [(2, hm(6, 30)), (1, hm(7, 30)), (9, hm(8, 0))])];
        let l = layout(&line(), &trips, 4);
        assert_eq!(l.services, vec!["Fast", "Slow"]);
        // 06:00 to 07:30 spans the plot, so 07:00 sits two thirds across.
        assert_eq!(l.trips[0].points[0], (0.0, 1.0));
        assert!((l.trips[0].points[1].0 - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(l.trips[1].points.len(), 2, "unknown stations are skipped");
        assert!(l.trips[0].path.starts_with("M0.000 0.000L66.667 100.000"));
        assert!(l.times.iter().all(|t| (0.0..=1.0).contains(&t.at)));
        assert_eq!(l.times[0].text, "06:00");
    }

    #[test]
    fn picks_the_trip_under_the_pointer() {
        let trips = [Trip::new("1", "S", [(0, 0.0), (2, 100.0)]), Trip::new("2", "S", [(0, 50.0), (2, 150.0)])];
        let l = layout(&line(), &trips, 4);
        // Trip 1 runs from (0, 1) to (2/3, 0); its midpoint is (1/3, 0.5).
        assert_eq!(nearest_trip(&l.trips, (0.34, 0.5), 1.0, 0.05), Some(0));
        assert_eq!(nearest_trip(&l.trips, (0.67, 0.5), 1.0, 0.05), Some(1));
        assert_eq!(nearest_trip(&l.trips, (0.0, 0.0), 1.0, 0.05), None);
        assert_eq!(clock(hm(25, 5)), "01:05");
    }
}
