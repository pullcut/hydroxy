//! Raincloud plots: per group a half violin (the cloud), a slim box plot, and every raw
//! observation jittered beside it (the rain), on one value axis. Densities come from
//! [`crate::density`], box statistics from [`crate::boxplot`].

#![allow(clippy::many_single_char_names, reason = "layout maths reads best as n, b, f, v")]

use std::fmt::Write as _;

use crate::Orientation;
use crate::annotate::{self, Annotation};
use crate::boxplot::{self, Group, Stats};
use crate::density;
use crate::probe::{self, Cell, Pick, Readout, Row, Stop, Tick};
use crate::scale::{ScaleKind, ValueScale};
use crate::scatter::Dot;

/// Points sampled along each cloud.
const SAMPLES: usize = 64;
/// Across each group's band: where the rain starts and ends, the box centre, and the
/// cloud's baseline and peak, as shares of the band.
const RAIN: (f64, f64) = (0.1, 0.36);
const BOX: f64 = 0.44;
const CLOUD: (f64, f64) = (0.5, 0.94);

/// One group's box plot along the value axis, as plot fractions.
#[derive(Debug, Clone, PartialEq)]
pub struct Boxed {
    /// The numbers behind it.
    pub stats: Stats,
    /// Across the value axis: the box's centre line.
    pub cross: f64,
    /// Lower whisker end.
    pub low: f64,
    /// First quartile.
    pub q1: f64,
    /// Median.
    pub median: f64,
    /// Third quartile.
    pub q3: f64,
    /// Upper whisker end.
    pub high: f64,
}

/// One group's cloud.
#[derive(Debug, Clone, PartialEq)]
pub struct Cloud {
    /// Group index.
    pub group: usize,
    /// Colour slot.
    pub colour: usize,
    /// Filled half violin, in a 0..100 viewBox with y down.
    pub area: String,
    /// Its outer edge, open, in the same viewBox.
    pub line: String,
}

/// Everything a raincloud plot needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct RainLayout {
    /// Which way the value axis runs: up (vertical) or across (horizontal).
    pub orientation: Orientation,
    /// One per group with values.
    pub clouds: Vec<Cloud>,
    /// One per input group; `None` for a group with no finite values.
    pub boxes: Vec<Option<Boxed>>,
    /// Every finite observation, jittered across its band. `x` and `y` are plot fractions.
    pub dots: Vec<Dot>,
    /// Ticks, stops, legend, notes and table.
    pub readout: Readout,
}

/// A repeatable spread in 0..1 for observation `i` of group `g` (a splitmix64 hash).
#[allow(clippy::cast_precision_loss, reason = "53 bits of mantissa are plenty")]
#[must_use]
pub fn jitter(g: usize, i: usize) -> f64 {
    let mut z = ((g as u64) << 32 ^ i as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Lay out one raincloud per group. `Vertical` puts groups along the bottom with values
/// rising; `Horizontal` stacks groups from the top with values running across. Each cloud
/// is trimmed to its data and scaled to its own peak. `bandwidth` overrides Silverman's rule.
#[allow(clippy::too_many_lines, reason = "one pass per group building every mark")]
#[allow(clippy::cast_precision_loss, reason = "group counts are small")]
#[must_use]
pub fn raincloud(
    groups: &[Group],
    orientation: Orientation,
    bandwidth: Option<f64>,
    annotations: &[Annotation],
) -> RainLayout {
    let n = groups.len().max(1);
    let b = 1.0 / n as f64;
    let vertical = orientation == Orientation::Vertical;
    // Bands run left to right, or top to bottom: `start` is the band's left or bottom edge.
    let start = |g: usize| if vertical { g as f64 * b } else { 1.0 - (g + 1) as f64 * b };
    let across = |g: usize, share: f64| start(g) + share * b;
    let extra = if vertical { annotate::y_extent(annotations) } else { probe::x_extent(annotations) };
    let (lo, hi) = probe::widen(probe::extent(groups).unwrap_or((0.0, 1.0)), extra);
    let sv = ValueScale::new(ScaleKind::Linear, lo, if hi > lo { hi } else { lo + 1.0 }, 6);
    let f = |v: f64| sv.map(v).clamp(0.0, 1.0);
    // (value fraction, cross fraction) to (x, y) plot fractions from the bottom left.
    let xy = |v: f64, c: f64| if vertical { (c, v) } else { (v, c) };
    let svg = |(x, y): (f64, f64)| (x * 100.0, 100.0 - y * 100.0);
    let mut clouds = Vec::new();
    let mut boxes = Vec::with_capacity(groups.len());
    let mut dots = Vec::new();
    let mut stops = Vec::with_capacity(groups.len());
    for (g, group) in groups.iter().enumerate() {
        let colour = probe::slot(g, groups.len());
        let mut v: Vec<f64> = group.values.iter().copied().filter(|v| v.is_finite()).collect();
        v.sort_by(f64::total_cmp);
        let stop_at = |value: f64| if vertical { (probe::band(g, n), value) } else { (value, start(g)) };
        let Some(stats) = boxplot::summarize(&v) else {
            boxes.push(None);
            let (x, y) = stop_at(0.0);
            stops.push(Stop {
                x,
                y,
                title: Cell::Text(group.label.clone()),
                colour: Some(colour),
                rows: vec![Row::new("Count", Cell::Count(0))],
            });
            continue;
        };
        // The cloud: density along the data range, rising from the baseline.
        let h = bandwidth.filter(|h| h.is_finite() && *h > 0.0).unwrap_or_else(|| density::silverman(&v));
        let ys = density::grid(v[0], v[v.len() - 1], SAMPLES);
        let ds = density::kde(&v, h, &ys);
        let peak = ds.iter().copied().fold(0.0, f64::max);
        let edge: Vec<(f64, f64)> = ys
            .iter()
            .zip(&ds)
            .map(|(&y, &d)| {
                svg(xy(f(y), across(g, CLOUD.0 + (CLOUD.1 - CLOUD.0) * if peak > 0.0 { d / peak } else { 0.0 })))
            })
            .collect();
        let mut line = String::new();
        for (i, (x, y)) in edge.iter().enumerate() {
            let _ = write!(line, "{}{x:.2} {y:.2}", if i == 0 { "M" } else { "L" });
        }
        let (b0, b1) = (svg(xy(f(v[0]), across(g, CLOUD.0))), svg(xy(f(v[v.len() - 1]), across(g, CLOUD.0))));
        let area = format!("M{:.2} {:.2}L{}L{:.2} {:.2}Z", b0.0, b0.1, &line[1..], b1.0, b1.1);
        clouds.push(Cloud { group: g, colour, area, line });
        // The rain, in input order so the jitter stays put as values change.
        let mut k = 0;
        for &value in &group.values {
            if !value.is_finite() {
                continue;
            }
            let (x, y) = xy(f(value), across(g, RAIN.0 + (RAIN.1 - RAIN.0) * jitter(g, k)));
            dots.push(Dot { group: colour, index: k, x, y, size: None });
            k += 1;
        }
        let rows = vec![
            Row::new("Count", Cell::Count(stats.n)),
            Row::new("Q1", Cell::Value(stats.q1)),
            Row::new("Median", Cell::Value(stats.median)),
            Row::new("Q3", Cell::Value(stats.q3)),
        ];
        let (x, y) = stop_at(f(stats.median));
        stops.push(Stop { x, y, title: Cell::Text(group.label.clone()), colour: Some(colour), rows });
        boxes.push(Some(Boxed {
            cross: across(g, BOX),
            low: f(stats.low),
            q1: f(stats.q1),
            median: f(stats.median),
            q3: f(stats.q3),
            high: f(stats.high),
            stats,
        }));
    }
    let value_ticks = probe::value_ticks(sv, 6, Cell::Value);
    let labels: Vec<Tick> = groups
        .iter()
        .enumerate()
        .map(|(g, grp)| Tick {
            fraction: if vertical { probe::band(g, n) } else { start(g) + b / 2.0 },
            label: Cell::Text(grp.label.clone()),
        })
        .collect();
    let notes = if vertical {
        annotate::place(annotations, |i| probe::band(0, n) + i * b, |v| sv.map(v))
    } else {
        annotate::place(annotations, |v| sv.map(v), |i| start(0) + b / 2.0 - i * b)
    };
    let (head, rows) = probe::observations(groups);
    let (x_ticks, y_ticks) = if vertical { (labels, value_ticks) } else { (value_ticks, labels) };
    RainLayout {
        orientation,
        clouds,
        boxes,
        dots,
        readout: Readout {
            x_ticks,
            y_ticks,
            x_bands: vertical,
            stops,
            pick: if vertical { Pick::X } else { Pick::Row },
            legend: probe::legend(groups.iter().map(|g| g.label.clone())),
            notes,
            head,
            rows,
        },
    }
}

impl Boxed {
    /// Inline style placing a span from value fraction `a` to `b` along the value axis on
    /// the box's centre line, with `--m` the share of the way from `a` to the median (where
    /// it opens from).
    #[must_use]
    pub fn span(&self, orientation: Orientation, a: f64, b: f64) -> String {
        let (a, b) = (a.min(b), a.max(b));
        let m = if b > a { ((self.median - a) / (b - a)).clamp(0.0, 1.0) } else { 0.5 };
        let (c, a, len) = (self.cross * 100.0, a * 100.0, (b - a) * 100.0);
        match orientation {
            Orientation::Vertical => {
                format!("left: {c:.3}%; bottom: {a:.3}%; height: {len:.3}%; --m: {:.1}%;", m * 100.0)
            }
            Orientation::Horizontal => {
                format!("bottom: {c:.3}%; left: {a:.3}%; width: {len:.3}%; --m: {:.1}%;", m * 100.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: [f64; 10] = [2.0, 3.0, 3.5, 4.0, 4.2, 4.8, 5.0, 5.5, 6.0, 9.0];

    #[test]
    fn jitter_is_repeatable_and_spread() {
        assert_eq!(jitter(1, 7), jitter(1, 7));
        assert_ne!(jitter(1, 7), jitter(2, 7));
        let xs: Vec<f64> = (0..1000).map(|i| jitter(0, i)).collect();
        assert!(xs.iter().all(|x| (0.0..1.0).contains(x)));
        let mean = xs.iter().sum::<f64>() / 1000.0;
        assert!((mean - 0.5).abs() < 0.05, "{mean}");
    }

    #[test]
    fn vertical_rain_falls_left_of_the_box_and_cloud_rises_right() {
        let l = raincloud(&[Group::new("a", SAMPLE), Group::new("b", SAMPLE)], Orientation::Vertical, None, &[]);
        assert_eq!(l.dots.len(), 20);
        let bx = l.boxes[0].as_ref().map_or(0.0, |b| b.cross);
        assert!(l.dots.iter().filter(|d| d.group == 0).all(|d| d.x < bx && d.x >= 0.0));
        assert!(l.dots.iter().filter(|d| d.group == 1).all(|d| d.x > 0.5));
        assert!(l.clouds[0].area.ends_with('Z') && l.clouds[0].line.starts_with('M'));
        assert_eq!(l.readout.pick, Pick::X);
        assert!(l.readout.x_bands);
    }

    #[test]
    fn horizontal_stacks_groups_from_the_top() {
        let l = raincloud(&[Group::new("a", SAMPLE), Group::new("b", SAMPLE)], Orientation::Horizontal, None, &[]);
        let (a, b) = (l.boxes[0].as_ref().map_or(0.0, |b| b.cross), l.boxes[1].as_ref().map_or(0.0, |b| b.cross));
        assert!(a > 0.5 && b < 0.5);
        assert_eq!(probe::nearest(&l.readout.stops, 0.3, 0.9, l.readout.pick), Some(0));
        assert_eq!(probe::nearest(&l.readout.stops, 0.3, 0.1, l.readout.pick), Some(1));
        assert!(l.readout.y_ticks[0].fraction > l.readout.y_ticks[1].fraction);
    }

    #[test]
    fn spans_open_from_the_median_and_empty_groups_skip() {
        let l = raincloud(&[Group::new("a", SAMPLE), Group::new("none", [f64::NAN])], Orientation::Vertical, None, &[]);
        assert!(l.boxes[1].is_none());
        assert_eq!(l.clouds.len(), 1);
        let bx = l.boxes[0].clone().unwrap_or_else(|| unreachable!());
        assert!(bx.q1 < bx.median && bx.median < bx.q3);
        let style = bx.span(Orientation::Horizontal, bx.q1, bx.q3);
        assert!(style.starts_with("bottom: ") && style.contains("--m: "));
        assert_eq!(l.readout.rows.len(), 10);
    }
}
