//! Pictograms (isotype charts): a count drawn as a row of repeated icons, each standing
//! for a fixed number of units. The last icon is cut to the remainder, so 3.4 icons'
//! worth draws three whole icons and four tenths of a fourth.

/// The shape repeated in a pictogram, drawn in a 24×24 box.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PictogramIcon {
    /// A person: head and shoulders.
    #[default]
    Person,
    /// A rounded square.
    Square,
    /// A circle.
    Circle,
    /// Any SVG path `d` drawn in a 24×24 box, filled.
    Custom(String),
}

impl PictogramIcon {
    /// The icon's SVG path `d`, in a 24×24 box.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Person => "M12 2.5a4.25 4.25 0 1 1 0 8.5a4.25 4.25 0 0 1 0-8.5zM4 20.5c0-4.1 3.3-7.5 7.4-7.5h1.2c4.1 0 7.4 3.4 7.4 7.5v.5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1z",
            Self::Square => "M6.5 3h11A3.5 3.5 0 0 1 21 6.5v11a3.5 3.5 0 0 1-3.5 3.5h-11A3.5 3.5 0 0 1 3 17.5v-11A3.5 3.5 0 0 1 6.5 3z",
            Self::Circle => "M12 3a9 9 0 1 1 0 18a9 9 0 0 1 0-18z",
            Self::Custom(d) => d,
        }
    }
}

/// The smallest 1, 2 or 5 × 10ⁿ unit that draws `max` in at most `most` icons.
#[must_use]
pub fn auto_unit(max: f64, most: usize) -> f64 {
    #[allow(clippy::cast_precision_loss, reason = "icon counts are small")]
    let raw = max / most.max(1) as f64;
    if !raw.is_finite() || raw <= 0.0 {
        return 1.0;
    }
    let power = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0].into_iter().map(|f| f * power).find(|u| *u >= raw * (1.0 - 1e-9)).unwrap_or(10.0 * power)
}

/// The unit each icon stands for: `unit` when given, raised if needed so no row draws
/// more than `most` icons; otherwise the [`auto_unit`] for the largest value.
#[allow(clippy::cast_precision_loss, reason = "icon counts are small")]
#[must_use]
pub fn unit_for(values: &[f64], unit: Option<f64>, most: usize) -> f64 {
    let max = values.iter().copied().filter(|v| v.is_finite()).fold(0.0, f64::max);
    let floor = auto_unit(max, most);
    unit.filter(|u| u.is_finite() && *u > 0.0).map_or(floor, |u| if max / u > most as f64 { floor } else { u })
}

/// How full each icon in a row is, 0..=1: whole icons, then the remainder.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "icon counts are small and non-negative")]
#[must_use]
pub fn icons(value: f64, unit: f64) -> Vec<f64> {
    if !value.is_finite() || value <= 0.0 || unit <= 0.0 {
        return Vec::new();
    }
    let count = value / unit;
    let whole = (count + 1e-9).floor();
    let mut out = vec![1.0; whole as usize];
    let rest = count - whole;
    if rest > 1e-6 {
        out.push(rest);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_cut_the_last_one() {
        let row = icons(3.4, 1.0);
        assert_eq!(row.len(), 4);
        assert!((row[3] - 0.4).abs() < 1e-9);
        assert_eq!(icons(3.0, 1.0), [1.0; 3]);
        assert_eq!(icons(0.3, 1.0).len(), 1);
        assert!(icons(f64::NAN, 1.0).is_empty() && icons(-2.0, 1.0).is_empty());
        // Floating error does not add a sliver icon.
        assert_eq!(icons(0.3, 0.1).len(), 3);
    }

    #[test]
    fn units_stay_nice_and_bounded() {
        assert!((auto_unit(4_200.0, 20) - 500.0).abs() < 1e-9);
        assert!((auto_unit(20.0, 20) - 1.0).abs() < 1e-9);
        assert!((auto_unit(0.0, 20) - 1.0).abs() < 1e-9);
        assert!((unit_for(&[4_200.0, 900.0], Some(1_000.0), 20) - 1_000.0).abs() < 1e-9);
        assert!((unit_for(&[4_200.0], Some(1.0), 20) - 500.0).abs() < 1e-9, "a tiny unit is raised");
        assert_eq!(PictogramIcon::Custom("M0 0h24v24H0z".into()).path(), "M0 0h24v24H0z");
    }
}
