//! Order book depth: cumulative bid and ask size as step areas either side of the
//! spread, on a price axis centred on the mid price. Paths are in a 0..100 viewBox, y down.

use std::fmt::Write;

use crate::annotate::{self, Annotation};
use crate::probe::{self, Cell, Pick, Readout, Row, Stop};
use crate::scale::{ScaleKind, ValueScale};

/// Colour slot for bids (`--chart-4`).
pub const BID: usize = 3;
/// Colour slot for asks (`--chart-2`).
pub const ASK: usize = 1;

/// One price level of the book.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    /// Price.
    pub price: f64,
    /// Size resting at that price.
    pub size: f64,
}

impl Level {
    /// A level.
    #[must_use]
    pub const fn new(price: f64, size: f64) -> Self {
        Self { price, size }
    }
}

/// One side's step area.
#[derive(Debug, Clone, PartialEq)]
pub struct Side {
    /// Outline along the top of the steps.
    pub line: String,
    /// The filled steps.
    pub area: String,
}

/// Areas plus the shared readout.
#[derive(Debug, Clone, PartialEq)]
pub struct DepthLayout {
    /// Bids, stepping down and left from the best bid.
    pub bids: Side,
    /// Asks, stepping up and right from the best ask.
    pub asks: Side,
    /// The mid price and its position across the plot, when both sides have levels.
    pub mid: Option<(f64, f64)>,
    /// Ticks, stops, legend, notes and table.
    pub readout: Readout,
}

/// Levels from the best price outwards (bids high to low, asks low to high) with the
/// running total. Non-finite or non-positive sizes are dropped.
#[must_use]
pub fn cumulative(levels: &[Level], bids: bool) -> Vec<(Level, f64)> {
    let mut v: Vec<Level> =
        levels.iter().copied().filter(|l| l.price.is_finite() && l.size.is_finite() && l.size > 0.0).collect();
    v.sort_by(|a, b| if bids { b.price.total_cmp(&a.price) } else { a.price.total_cmp(&b.price) });
    let mut total = 0.0;
    v.into_iter()
        .map(|l| {
            total += l.size;
            (l, total)
        })
        .collect()
}

fn steps(side: &[(Level, f64)], edge: f64, x: impl Fn(f64) -> f64, y: impl Fn(f64) -> f64) -> Side {
    let Some(&(first, _)) = side.first() else { return Side { line: String::new(), area: String::new() } };
    let mut line = format!("M{:.3} {:.3}", x(first.price), y(0.0));
    let mut prev = None;
    for &(l, total) in side {
        if let Some(p) = prev {
            let _ = write!(line, "L{:.3} {:.3}", x(l.price), y(p));
        }
        let _ = write!(line, "L{:.3} {:.3}", x(l.price), y(total));
        prev = Some(total);
    }
    let _ = write!(line, "L{:.3} {:.3}", edge, y(prev.unwrap_or(0.0)));
    let area = format!("{line}L{edge:.3} {:.3}Z", y(0.0));
    Side { line, area }
}

/// Lay out a book. The price axis is centred on the mid price so both sides get room.
#[must_use]
pub fn layout(bids: &[Level], asks: &[Level], annotations: &[Annotation]) -> DepthLayout {
    let (b, a) = (cumulative(bids, true), cumulative(asks, false));
    let prices = || b.iter().chain(&a).map(|(l, _)| l.price);
    let (lo, hi) = prices().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| (lo.min(p), hi.max(p)));
    let (lo, hi) = if lo.is_finite() { (lo, hi) } else { (0.0, 1.0) };
    let mid = b.first().zip(a.first()).map(|((bb, _), (ba, _))| bb.price.midpoint(ba.price));
    let (lo, hi) = mid.map_or((lo, hi), |m| {
        let half = (m - lo).max(hi - m);
        (m - half, m + half)
    });
    let (lo, hi) = probe::widen((lo, hi), probe::x_extent(annotations));
    let sx = ValueScale::new(ScaleKind::Linear, lo, if hi > lo { hi } else { lo + 1.0 }, 4);
    let top = b.iter().chain(&a).map(|(_, t)| *t).fold(0.0, f64::max);
    let (top, _) = probe::widen((top, top), annotate::y_extent(annotations));
    let sy = ValueScale::new(ScaleKind::Linear, 0.0, if top > 0.0 { top } else { 1.0 }, 5);
    let vx = |p: f64| sx.map(p) * 100.0;
    let vy = |t: f64| 100.0 - sy.map(t) * 100.0;

    let mut levels: Vec<(Level, f64, bool)> =
        b.iter().map(|&(l, t)| (l, t, true)).chain(a.iter().map(|&(l, t)| (l, t, false))).collect();
    levels.sort_by(|x, y| x.0.price.total_cmp(&y.0.price));
    let stops = levels
        .iter()
        .map(|&(l, total, bid)| Stop {
            x: sx.map(l.price),
            y: sy.map(total),
            title: Cell::Value(l.price),
            colour: Some(if bid { BID } else { ASK }),
            rows: vec![
                Row::new(if bid { "Bid depth" } else { "Ask depth" }, Cell::Compact(total)),
                Row::new("Size", Cell::Compact(l.size)),
            ],
        })
        .collect();
    let rows = levels
        .iter()
        .map(|&(l, total, bid)| {
            vec![
                Cell::Value(l.price),
                Cell::Text(if bid { "Bid" } else { "Ask" }.into()),
                Cell::Text(l.size.to_string()),
                Cell::Text(total.to_string()),
            ]
        })
        .collect();
    DepthLayout {
        bids: steps(&b, 0.0, vx, vy),
        asks: steps(&a, 100.0, vx, vy),
        mid: mid.map(|m| (m, sx.map(m))),
        readout: Readout {
            x_ticks: probe::value_ticks(sx, 4, Cell::Value),
            y_ticks: probe::value_ticks(sy, 5, Cell::Compact),
            stops,
            pick: Pick::X,
            legend: vec![("Bids".into(), BID), ("Asks".into(), ASK)],
            notes: annotate::place(annotations, |p| sx.map(p), |t| sy.map(t)),
            head: ["Price", "Side", "Size", "Cumulative"].map(String::from).to_vec(),
            rows,
            ..Readout::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> (Vec<Level>, Vec<Level>) {
        (
            vec![Level::new(99.0, 5.0), Level::new(100.0, 2.0), Level::new(98.0, 0.0)],
            vec![Level::new(101.0, 1.0), Level::new(103.0, 4.0)],
        )
    }

    #[test]
    fn totals_run_outward_from_the_spread() {
        let (bids, asks) = book();
        let b = cumulative(&bids, true);
        assert_eq!(b.iter().map(|x| x.1).collect::<Vec<_>>(), vec![2.0, 7.0]);
        assert_eq!(cumulative(&asks, false)[1].1, 5.0);
    }

    #[test]
    fn centred_on_the_mid_with_steps_to_the_edges() {
        let (bids, asks) = book();
        let l = layout(&bids, &asks, &[]);
        let (m, f) = l.mid.unwrap_or_default();
        assert!((m - 100.5).abs() < 1e-9);
        assert!((0.3..0.7).contains(&f));
        assert!(l.bids.area.starts_with('M') && l.bids.area.ends_with('Z'));
        assert!(l.asks.line.contains("L100.000"));
        assert_eq!(l.readout.stops.len(), 4);
        assert!(l.readout.stops.windows(2).all(|w| w[0].x <= w[1].x));
    }

    #[test]
    fn one_sided_and_empty_books() {
        let l = layout(&[Level::new(10.0, 1.0)], &[], &[]);
        assert!(l.mid.is_none() && l.asks.area.is_empty());
        assert_eq!(layout(&[], &[], &[]).readout.stops, vec![]);
    }
}
