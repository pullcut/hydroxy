//! Open-high-low-close ticks: a vertical range per period with the open marked to the
//! left and the close to the right. Geometry is [`CandleGeom`], on any value scale.

use crate::annotate::{self, Annotation};
use crate::candle::{Candle, CandleGeom};
use crate::line;
use crate::probe::{self, Cell, Pick, Readout, Row, Stop, Tick};
use crate::scale::{ScaleKind, ValueScale};

/// Colour slot for periods that closed at or above their open (`--chart-4`).
pub const UP: usize = 3;
/// Colour slot for periods that closed below their open (`--chart-2`).
pub const DOWN: usize = 1;

/// Ticks plus the shared readout.
#[derive(Debug, Clone, PartialEq)]
pub struct OhlcLayout {
    /// One per period; `None` when a price is not finite.
    pub candles: Vec<Option<CandleGeom>>,
    /// Ticks, stops, legend, notes and table.
    pub readout: Readout,
}

fn valid(c: &Candle) -> bool {
    [c.open, c.high, c.low, c.close].iter().all(|v| v.is_finite())
}

/// Lay out `candles` (oldest first) with one `labels` entry each, on a price axis that
/// fits the data and any annotations.
#[must_use]
pub fn layout(candles: &[Candle], labels: &[String], scale: ScaleKind, annotations: &[Annotation]) -> OhlcLayout {
    let n = candles.len().max(1);
    let lo = candles.iter().filter(|c| valid(c)).map(|c| c.low.min(c.open).min(c.close)).fold(f64::INFINITY, f64::min);
    let hi = candles.iter().filter(|c| valid(c)).map(|c| c.high.max(c.open).max(c.close)).fold(f64::NEG_INFINITY, f64::max);
    let (lo, hi) = probe::widen(if lo.is_finite() { (lo, hi) } else { (0.0, 1.0) }, annotate::y_extent(annotations));
    let sy = ValueScale::new(scale, lo, if hi > lo { hi } else { lo + 1.0 }, 5);
    let f = |v: f64| sy.map(v).clamp(0.0, 1.0);
    let label = |i: usize| labels.get(i).cloned().unwrap_or_else(|| format!("Period {}", i + 1));
    let has_volume = candles.iter().any(|c| c.volume.is_some_and(f64::is_finite));
    let geoms: Vec<Option<CandleGeom>> = candles
        .iter()
        .map(|c| {
            valid(c).then(|| CandleGeom {
                open: f(c.open),
                close: f(c.close),
                high: f(c.high.max(c.open).max(c.close)),
                low: f(c.low.min(c.open).min(c.close)),
                up: c.close >= c.open,
                volume: 0.0,
            })
        })
        .collect();
    let volume = |c: &Candle| c.volume.filter(|v| v.is_finite()).map_or_else(|| Cell::Text("–".into()), Cell::Compact);
    let stops = candles
        .iter()
        .zip(&geoms)
        .enumerate()
        .filter_map(|(i, (c, g))| {
            let g = (*g)?;
            let mut rows = vec![
                Row::new("Open", Cell::Value(c.open)),
                Row::new("High", Cell::Value(c.high)),
                Row::new("Low", Cell::Value(c.low)),
                Row::new("Close", Cell::Value(c.close)),
            ];
            if has_volume {
                rows.push(Row::new("Volume", volume(c)));
            }
            Some(Stop { x: probe::band(i, n), y: g.close, title: Cell::Text(label(i)), colour: Some(if g.up { UP } else { DOWN }), rows })
        })
        .collect();
    let rows = candles
        .iter()
        .enumerate()
        .map(|(i, c)| vec![Cell::Text(label(i)), Cell::Value(c.open), Cell::Value(c.high), Cell::Value(c.low), Cell::Value(c.close), volume(c)])
        .collect();
    #[allow(clippy::cast_precision_loss, reason = "period counts are small")]
    let notes = annotate::place(annotations, |i| (i + 0.5) / n as f64, |v| sy.map(v));
    OhlcLayout {
        candles: geoms,
        readout: Readout {
            x_ticks: line::label_indices(candles.len(), 6).into_iter().map(|i| Tick { fraction: probe::band(i, n), label: Cell::Text(label(i)) }).collect(),
            y_ticks: probe::value_ticks(sy, 5, Cell::Value),
            x_bands: true,
            stops,
            pick: Pick::X,
            legend: vec![("Closed up".into(), UP), ("Closed down".into(), DOWN)],
            notes,
            head: ["Period", "Open", "High", "Low", "Close", "Volume"].map(String::from).to_vec(),
            rows,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_follow_prices_and_skip_bad_periods() {
        let candles = [Candle::new(100.0, 110.0, 95.0, 108.0), Candle::new(f64::NAN, 1.0, 1.0, 1.0), Candle::new(108.0, 109.0, 98.0, 99.0)];
        let l = layout(&candles, &["Mon".into(), "Tue".into(), "Wed".into()], ScaleKind::Linear, &[]);
        assert!(l.candles[1].is_none());
        assert_eq!(l.readout.stops.len(), 2);
        let a = l.candles[0].unwrap_or_else(|| unreachable!());
        assert!(a.up && a.low < a.open && a.open < a.close && a.close < a.high);
        assert_eq!(l.readout.stops[1].colour, Some(DOWN));
        assert_eq!(l.readout.rows.len(), 3);
    }

    #[test]
    fn log_scale_and_annotations_widen_the_axis() {
        let l = layout(&[Candle::new(10.0, 12.0, 9.0, 11.0)], &[], ScaleKind::Log, &[Annotation::y_line(500.0, "Target")]);
        assert_eq!(l.readout.notes.len(), 1);
        assert!(l.readout.y_ticks.iter().any(|t| t.label == Cell::Value(1000.0)));
        assert_eq!(l.readout.x_ticks[0].label, Cell::Text("Period 1".into()));
    }
}
