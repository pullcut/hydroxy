//! Renko bricks: a new fixed-size brick each time price moves a full brick from the last
//! one, ignoring time. Ported from hecto's `pricesToRenkoBricks`.

use crate::annotate::{self, Annotation};
use crate::probe::{self, Cell, Pick, Readout, Row, Stop};
use crate::scale::{ScaleKind, ValueScale, nice_step};

/// Colour slot for rising bricks (`--chart-4`).
pub const UP: usize = 3;
/// Colour slot for falling bricks (`--chart-2`).
pub const DOWN: usize = 1;

/// One brick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brick {
    /// Price the brick starts from.
    pub open: f64,
    /// Price it ends at, one brick size away.
    pub close: f64,
    /// Whether it rises.
    pub up: bool,
    /// Index of the price that completed it.
    pub source: usize,
}

/// A brick in plot fractions (0 = bottom).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrickGeom {
    /// Lower edge.
    pub bottom: f64,
    /// Upper edge.
    pub top: f64,
    /// Whether it rises.
    pub up: bool,
}

/// Bricks plus the shared readout.
#[derive(Debug, Clone, PartialEq)]
pub struct RenkoLayout {
    /// The brick size used.
    pub size: f64,
    /// One per brick, left to right.
    pub bricks: Vec<BrickGeom>,
    /// Ticks, stops, legend, notes and table.
    pub readout: Readout,
}

/// Bricks for `prices` with bricks of `size`. Non-finite prices are skipped.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "brick counts are small")]
#[must_use]
pub fn bricks(prices: &[f64], size: f64) -> Vec<Brick> {
    let size = size.abs().max(f64::EPSILON);
    let mut finite = prices.iter().copied().enumerate().filter(|(_, p)| p.is_finite());
    let Some((_, mut anchor)) = finite.next() else { return Vec::new() };
    let mut out = Vec::new();
    for (source, price) in finite {
        let delta = price - anchor;
        if delta.abs() < size {
            continue;
        }
        let up = delta > 0.0;
        for _ in 0..(delta.abs() / size).floor() as usize {
            let close = anchor + if up { size } else { -size };
            out.push(Brick { open: anchor, close, up, source });
            anchor = close;
        }
    }
    out
}

/// A brick size of about a twentieth of the price range, rounded to 1, 2 or 5 × 10ⁿ.
#[must_use]
pub fn auto_size(prices: &[f64]) -> f64 {
    let (lo, hi) = prices.iter().filter(|p| p.is_finite()).fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &p| (lo.min(p), hi.max(p)));
    if hi > lo { nice_step((hi - lo) / 20.0) } else { 1.0 }
}

/// Lay out bricks for `prices` (with one optional label each) on a price axis.
/// `size` defaults to [`auto_size`].
#[must_use]
pub fn layout(prices: &[f64], labels: &[String], size: Option<f64>, annotations: &[Annotation]) -> RenkoLayout {
    let size = size.filter(|s| s.is_finite() && *s > 0.0).unwrap_or_else(|| auto_size(prices));
    let all = bricks(prices, size);
    let n = all.len().max(1);
    let lo = all.iter().map(|b| b.open.min(b.close)).fold(f64::INFINITY, f64::min);
    let hi = all.iter().map(|b| b.open.max(b.close)).fold(f64::NEG_INFINITY, f64::max);
    let (lo, hi) = probe::widen(if lo.is_finite() { (lo, hi) } else { (0.0, 1.0) }, annotate::y_extent(annotations));
    let sy = ValueScale::new(ScaleKind::Linear, lo, if hi > lo { hi } else { lo + 1.0 }, 5);
    let label = |b: &Brick| labels.get(b.source).cloned().unwrap_or_else(|| format!("Price {}", b.source + 1));
    let geoms = all.iter().map(|b| BrickGeom { bottom: sy.map(b.open.min(b.close)), top: sy.map(b.open.max(b.close)), up: b.up }).collect();
    let stops = all
        .iter()
        .enumerate()
        .map(|(i, b)| Stop {
            x: probe::band(i, n),
            y: sy.map(b.close),
            title: Cell::Text(format!("Brick {} · {}", i + 1, label(b))),
            colour: Some(if b.up { UP } else { DOWN }),
            rows: vec![Row::new("Open", Cell::Value(b.open)), Row::new("Close", Cell::Value(b.close))],
        })
        .collect();
    let rows = all
        .iter()
        .enumerate()
        .map(|(i, b)| vec![Cell::Text(format!("Brick {}", i + 1)), Cell::Text(label(b)), Cell::Value(b.open), Cell::Value(b.close)])
        .collect();
    #[allow(clippy::cast_precision_loss, reason = "brick counts are small")]
    let notes = annotate::place(annotations, |i| (i + 0.5) / n as f64, |v| sy.map(v));
    RenkoLayout {
        size,
        bricks: geoms,
        readout: Readout {
            y_ticks: probe::value_ticks(sy, 5),
            stops,
            pick: Pick::X,
            legend: vec![("Rising".into(), UP), ("Falling".into(), DOWN)],
            notes,
            head: ["Brick", "Completed at", "Open", "Close"].map(String::from).to_vec(),
            rows,
            ..Readout::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bricks_need_a_full_move() {
        let b = bricks(&[100.0, 101.0, 104.5, f64::NAN, 103.0, 99.0], 2.0);
        // 100 → 104.5 makes two up bricks (to 104); 103 is not a full brick back; 99 makes two down.
        assert_eq!(b.iter().map(|b| (b.close, b.up)).collect::<Vec<_>>(), vec![(102.0, true), (104.0, true), (102.0, false), (100.0, false)]);
        assert_eq!(b[0].source, 2);
        assert_eq!(b[3].source, 5);
        assert_eq!(bricks(&[1.0], 1.0), vec![]);
    }

    #[test]
    fn auto_size_rounds_nicely() {
        assert!((auto_size(&[100.0, 140.0]) - 2.0).abs() < 1e-12);
        assert!((auto_size(&[5.0]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn layout_stacks_bricks_on_the_price_axis() {
        let l = layout(&[10.0, 14.0, 11.0], &["Mon".into(), "Tue".into(), "Wed".into()], Some(1.0), &[]);
        assert_eq!(l.bricks.len(), 7);
        assert!(l.bricks[0].top <= l.bricks[3].bottom + 1e-9);
        assert_eq!(l.readout.stops[0].title, Cell::Text("Brick 1 · Tue".into()));
        assert_eq!(l.readout.stops[6].colour, Some(DOWN));
    }
}
