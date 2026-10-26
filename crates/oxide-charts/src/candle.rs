//! Candlestick (OHLC) geometry with an optional volume strip.

use crate::bar::Gridline;
use crate::scale::{Linear, ticks};

/// One period of price data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    /// Opening price.
    pub open: f64,
    /// Highest price.
    pub high: f64,
    /// Lowest price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Traded volume, if known.
    pub volume: Option<f64>,
}

impl Candle {
    /// A candle without volume.
    #[must_use]
    pub const fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self { open, high, low, close, volume: None }
    }

    /// The same candle with a volume.
    #[must_use]
    pub const fn with_volume(mut self, volume: f64) -> Self {
        self.volume = Some(volume);
        self
    }

    fn valid(self) -> bool {
        [self.open, self.high, self.low, self.close].iter().all(|v| v.is_finite())
    }
}

/// One candle in plot fractions (0 = bottom).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandleGeom {
    /// Open height.
    pub open: f64,
    /// Close height.
    pub close: f64,
    /// Top of the wick.
    pub high: f64,
    /// Bottom of the wick.
    pub low: f64,
    /// Closed at or above its open.
    pub up: bool,
    /// Volume bar height within the volume strip; 0 when absent.
    pub volume: f64,
}

/// Everything a candlestick chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct CandleLayout {
    /// One per candle; `None` when a price is not finite.
    pub candles: Vec<Option<CandleGeom>>,
    /// Price gridlines from the bottom up.
    pub gridlines: Vec<Gridline>,
    /// Bottom of the price axis.
    pub min: f64,
    /// Top of the price axis.
    pub max: f64,
    /// Whether any candle has a volume.
    pub has_volume: bool,
    /// The largest volume, the top of the strip.
    pub volume_max: f64,
}

/// Lay out `candles` on a price axis that fits the data (zero is not forced in).
/// Wicks are widened to cover open and close if the data disagrees.
#[must_use]
pub fn layout(candles: &[Candle], tick_count: usize) -> CandleLayout {
    let valid = || candles.iter().filter(|c| c.valid());
    let lo = valid().map(|c| c.low.min(c.open).min(c.close)).fold(f64::INFINITY, f64::min);
    let hi = valid().map(|c| c.high.max(c.open).max(c.close)).fold(f64::NEG_INFINITY, f64::max);
    let (lo, hi) = if lo.is_finite() { (lo, hi) } else { (0.0, 1.0) };
    let tick_values = ticks(lo, if hi > lo { hi } else { lo + 1.0 }, tick_count);
    let (min, max) = (tick_values.first().copied().unwrap_or(lo), tick_values.last().copied().unwrap_or(hi));
    let scale = Linear::new((min, max), (0.0, 1.0));
    let f = |v: f64| scale.map(v).clamp(0.0, 1.0);
    let volume = |c: &Candle| c.volume.filter(|v| v.is_finite() && *v > 0.0).unwrap_or(0.0);
    let volume_max = candles.iter().map(volume).fold(0.0, f64::max);
    CandleLayout {
        candles: candles
            .iter()
            .map(|c| {
                c.valid().then(|| CandleGeom {
                    open: f(c.open),
                    close: f(c.close),
                    high: f(c.high.max(c.open).max(c.close)),
                    low: f(c.low.min(c.open).min(c.close)),
                    up: c.close >= c.open,
                    volume: if volume_max > 0.0 { volume(c) / volume_max } else { 0.0 },
                })
            })
            .collect(),
        gridlines: tick_values.iter().map(|&value| Gridline { value, fraction: scale.map(value) }).collect(),
        min,
        max,
        has_volume: volume_max > 0.0,
        volume_max,
    }
}

/// Short volume text: 950, 12.5k, 3.4M.
#[must_use]
pub fn compact(v: f64) -> String {
    let (n, unit) = match v.abs() {
        a if a >= 1e9 => (v / 1e9, "B"),
        a if a >= 1e6 => (v / 1e6, "M"),
        a if a >= 1e3 => (v / 1e3, "k"),
        _ => return crate::format_value(v),
    };
    let text = format!("{n:.1}");
    format!("{}{unit}", text.strip_suffix(".0").unwrap_or(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_and_down_candles_with_a_price_axis_that_skips_zero() {
        let l = layout(&[Candle::new(100.0, 110.0, 95.0, 108.0), Candle::new(108.0, 109.0, 98.0, 99.0)], 5);
        let (a, b) = (l.candles[0].unwrap_or_else(|| unreachable!()), l.candles[1].unwrap_or_else(|| unreachable!()));
        assert!(a.up && !b.up);
        assert!(l.min > 0.0 && l.min <= 95.0 && l.max >= 110.0);
        assert!(a.low < a.open && a.open < a.close && a.close < a.high);
        assert!(!l.has_volume);
    }

    #[test]
    fn volume_scales_to_the_largest_and_bad_candles_are_gaps() {
        let l = layout(
            &[Candle::new(1.0, 2.0, 0.5, 1.5).with_volume(50.0), Candle::new(f64::NAN, 2.0, 1.0, 1.0), Candle::new(1.0, 2.0, 1.0, 2.0).with_volume(200.0)],
            5,
        );
        assert!(l.has_volume && (l.volume_max - 200.0).abs() < 1e-9);
        assert!(l.candles[1].is_none());
        assert!((l.candles[0].map_or(0.0, |c| c.volume) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn volumes_read_compactly() {
        assert_eq!(compact(950.0), "950");
        assert_eq!(compact(12_500.0), "12.5k");
        assert_eq!(compact(3_000_000.0), "3M");
    }

    #[test]
    fn wicks_cover_the_body() {
        let l = layout(&[Candle::new(10.0, 9.0, 11.0, 12.0)], 5);
        let c = l.candles[0].unwrap_or_else(|| unreachable!());
        assert!(c.high >= c.close && c.low <= c.open);
    }
}
