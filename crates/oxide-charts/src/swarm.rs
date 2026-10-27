//! Strip plots and beeswarms: one row per group, values along x. A strip plot spreads
//! dots with a fixed jitter; a beeswarm packs them so none overlap.

use crate::annotate::{self, Annotation};
use crate::boxplot::Group;
use crate::probe::{self, Cell, Pick, Readout, Row, Stop, Tick};
use crate::scale::{ScaleKind, ValueScale};

/// How dots sit in their row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arrange {
    /// Spread by a deterministic jitter, `0..=1` of the row height.
    Strip(f64),
    /// Packed without overlap. Dot diameter as a fraction of plot width and height.
    Swarm {
        /// Diameter across.
        dx: f64,
        /// Diameter up.
        dy: f64,
    },
}

/// One dot in plot fractions (y from the bottom).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dot {
    /// Group index, also its colour slot before [`probe::slot`].
    pub group: usize,
    /// From the left, 0..=1.
    pub x: f64,
    /// From the bottom, 0..=1.
    pub y: f64,
}

/// Dots plus the shared readout. `dots[i]` matches `readout.stops[i]`.
#[derive(Debug, Clone, PartialEq)]
pub struct SwarmLayout {
    /// The dots, group by group, left to right.
    pub dots: Vec<Dot>,
    /// Ticks, stops, legend, notes and table.
    pub readout: Readout,
}

#[allow(clippy::cast_precision_loss, reason = "indices are small")]
/// A repeatable jitter in `-0.5..0.5` for the `i`th dot: the golden-ratio sequence,
/// which spreads evenly without a random number generator.
#[must_use]
pub fn jitter(i: usize) -> f64 {
    ((i as f64 + 1.0) * 0.618_033_988_749_895).fract() - 0.5
}

/// Vertical offsets for dots at `xs` (in diameters), each as near zero as it can sit
/// without touching an earlier dot, limited to `±max`. Crowded rows overlap at the limit.
// checks every earlier neighbour within one diameter; fine for hundreds of dots.
#[must_use]
pub fn pack(xs: &[f64], max: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..xs.len()).collect();
    order.sort_by(|&a, &b| xs[a].total_cmp(&xs[b]));
    let mut placed: Vec<(f64, f64)> = Vec::new();
    let mut out = vec![0.0; xs.len()];
    for i in order {
        let x = xs[i];
        let near: Vec<(f64, f64)> = placed.iter().rev().take_while(|p| x - p.0 < 1.0).copied().collect();
        let mut candidates = vec![0.0];
        for &(px, py) in &near {
            let h = (1.0 - (x - px).powi(2)).max(0.0).sqrt();
            candidates.extend([py + h, py - h]);
        }
        candidates.sort_by(|a: &f64, b: &f64| a.abs().total_cmp(&b.abs()));
        let free = |c: f64| near.iter().all(|&(px, py)| (x - px).hypot(c - py) >= 1.0 - 1e-9);
        let y = candidates.iter().copied().find(|&c| free(c)).unwrap_or(0.0).clamp(-max, max);
        placed.push((x, y));
        out[i] = y;
    }
    out
}

/// Lay out `groups` as rows, top to bottom, on a shared value axis.
#[allow(clippy::cast_precision_loss, reason = "group counts are small")]
#[must_use]
pub fn layout(groups: &[Group], scale: ScaleKind, arrange: Arrange, annotations: &[Annotation]) -> SwarmLayout {
    let n = groups.len().max(1);
    let (lo, hi) = probe::widen(probe::extent(groups).unwrap_or((0.0, 1.0)), probe::x_extent(annotations));
    let sx = ValueScale::new(scale, lo, if hi > lo { hi } else { lo + 1.0 }, 6);
    let row = |g: usize| 1.0 - probe::band(g, n);
    let mut dots = Vec::new();
    let mut stops = Vec::new();
    for (g, group) in groups.iter().enumerate() {
        let mut values: Vec<f64> = group.values.iter().copied().filter(|v| v.is_finite()).collect();
        values.sort_by(f64::total_cmp);
        let xs: Vec<f64> = values.iter().map(|&v| sx.map(v).clamp(0.0, 1.0)).collect();
        let ys: Vec<f64> = match arrange {
            Arrange::Strip(amount) => (0..xs.len()).map(|i| row(g) + jitter(i) * amount.clamp(0.0, 1.0) / n as f64).collect(),
            Arrange::Swarm { dx, dy } => {
                let max = ((0.5 / n as f64) / dy.max(1e-6) - 0.5).max(0.0);
                pack(&xs.iter().map(|x| x / dx.max(1e-6)).collect::<Vec<_>>(), max).into_iter().map(|o| row(g) + o * dy).collect()
            }
        };
        let colour = probe::slot(g, groups.len());
        for ((&v, &x), y) in values.iter().zip(&xs).zip(ys) {
            dots.push(Dot { group: g, x, y });
            stops.push(Stop { x, y, title: Cell::Text(group.label.clone()), colour: Some(colour), rows: vec![Row::new("Value", Cell::Value(v))] });
        }
    }
    let (head, rows) = probe::observations(groups);
    let notes = annotate::place(annotations, |v| sx.map(v), |r| 1.0 - (r.max(0.0) + 0.5) / n as f64);
    let readout = Readout {
        x_ticks: probe::value_ticks(sx, 6),
        y_ticks: groups.iter().enumerate().map(|(g, grp)| Tick { fraction: row(g), label: Cell::Text(grp.label.clone()) }).collect(),
        stops,
        pick: Pick::Near,
        legend: probe::legend(groups.iter().map(|g| g.label.clone())),
        notes,
        head,
        rows,
        ..Readout::default()
    };
    SwarmLayout { dots, readout }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_is_repeatable_and_bounded() {
        assert_eq!(jitter(3), jitter(3));
        assert!((0..100).all(|i| (-0.5..0.5).contains(&jitter(i))));
    }

    #[test]
    fn packed_dots_never_touch() {
        let xs = [0.0, 0.2, 0.4, 0.5, 3.0, 0.1];
        let ys = pack(&xs, 10.0);
        for i in 0..xs.len() {
            for j in 0..i {
                assert!((xs[i] - xs[j]).hypot(ys[i] - ys[j]) >= 1.0 - 1e-6, "{i} and {j} overlap");
            }
        }
        assert!(ys[4].abs() < 1e-12, "a lone dot stays on the centre line");
    }

    #[test]
    fn packing_respects_the_limit() {
        let ys = pack(&[0.0; 8], 1.5);
        assert!(ys.iter().all(|y| y.abs() <= 1.5));
    }

    #[test]
    fn rows_run_top_to_bottom_and_stops_match_dots() {
        let groups = [Group::new("a", [1.0, 2.0, f64::NAN]), Group::new("b", [3.0])];
        let l = layout(&groups, ScaleKind::Linear, Arrange::Strip(0.0), &[]);
        assert_eq!(l.dots.len(), 3);
        assert_eq!(l.readout.stops.len(), 3);
        assert!(l.dots[0].y > l.dots[2].y);
        assert!((l.dots[0].y - 0.75).abs() < 1e-9);
        assert_eq!(l.readout.legend.len(), 2);
        let swarm = layout(&groups, ScaleKind::Log, Arrange::Swarm { dx: 0.02, dy: 0.05 }, &[Annotation::x_line(100.0, "Cap")]);
        assert!(swarm.readout.notes.len() == 1 && swarm.readout.x_ticks.last().is_some_and(|t| t.fraction > 0.99));
    }
}
