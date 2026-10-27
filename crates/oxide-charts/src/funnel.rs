//! Funnel stages. Each stage is a trapezoid whose wide edge is its own value and
//! whose narrow edge is the next stage's, so the outline steps down continuously.
//! Polygons sit in a 0..100 viewBox, centred across the flow.

use crate::Orientation;

/// One stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage {
    /// Width of the entry edge as a fraction of the widest stage, 0..=1.
    pub top: f64,
    /// Width of the exit edge: the next stage's width, or `top` for the last.
    pub bottom: f64,
    /// Where the stage starts along the flow, 0..=1.
    pub start: f64,
    /// Where it ends, 0..=1.
    pub end: f64,
    /// Share of the previous stage that reached this one. `None` for the first,
    /// or after a zero stage.
    pub conversion: Option<f64>,
    /// Share of the first stage that reached this one.
    pub of_first: Option<f64>,
}

#[allow(clippy::cast_precision_loss, reason = "stage counts are small")]
/// Lay out `values` in order, leaving `gap` (a fraction of the flow) between stages.
#[must_use]
pub fn layout(values: &[f64], gap: f64) -> Vec<Stage> {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let max = clean.iter().copied().fold(0.0, f64::max);
    let width = |v: f64| if max > 0.0 { v / max } else { 0.0 };
    let n = clean.len();
    let gap = if n > 1 { gap.clamp(0.0, 0.5 / n as f64) } else { 0.0 };
    let step = if n > 0 { (1.0 + gap) / n as f64 } else { 1.0 };
    let ratio = |a: f64, b: f64| (b > 0.0).then(|| a / b);
    clean
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let top = width(v);
            Stage {
                top,
                bottom: clean.get(i + 1).map_or(top, |&next| width(next)),
                start: i as f64 * step,
                end: i as f64 * step + step - gap,
                conversion: if i == 0 { None } else { ratio(v, clean[i - 1]) },
                of_first: clean.first().and_then(|&f| ratio(v, f)),
            }
        })
        .collect()
}

/// SVG `points` for a stage's trapezoid in a 0..100 viewBox.
#[must_use]
pub fn polygon(stage: &Stage, orientation: Orientation) -> String {
    let (entry, exit) = (stage.start * 100.0, stage.end * 100.0);
    let (wide, narrow) = (stage.top * 50.0, stage.bottom * 50.0);
    let pts = [(50.0 - wide, entry), (50.0 + wide, entry), (50.0 + narrow, exit), (50.0 - narrow, exit)];
    pts.iter()
        .map(|&(across, along)| match orientation {
            Orientation::Vertical => format!("{across:.3},{along:.3}"),
            Orientation::Horizontal => format!("{along:.3},{across:.3}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Up and Down step through a vertical list the way Left and Right do, so
/// [`crate::nav::list_key`] handles either axis.
#[must_use]
pub fn arrow_alias(key: &str) -> &str {
    match key {
        "ArrowUp" => "ArrowLeft",
        "ArrowDown" => "ArrowRight",
        k => k,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_narrow_to_the_next_value_with_conversions() {
        let s = layout(&[1000.0, 400.0, 100.0], 0.0);
        assert_eq!((s[0].top, s[0].bottom), (1.0, 0.4));
        assert_eq!((s[2].top, s[2].bottom), (0.1, 0.1));
        assert_eq!(s[0].conversion, None);
        assert_eq!(s[1].conversion, Some(0.4));
        assert_eq!(s[2].conversion, Some(0.25));
        assert_eq!(s[2].of_first, Some(0.1));
        assert!((s[2].end - 1.0).abs() < 1e-9);
    }

    #[test]
    fn gaps_and_zero_stages() {
        let s = layout(&[10.0, 0.0, 5.0], 0.06);
        assert!(s[1].start - s[0].end > 0.05);
        assert!((s[2].end - 1.0).abs() < 1e-9);
        assert_eq!(s[2].conversion, None);
    }

    #[test]
    fn polygons_swap_axes() {
        let s = layout(&[10.0, 5.0], 0.0);
        assert_eq!(polygon(&s[0], Orientation::Vertical), "0.000,0.000 100.000,0.000 75.000,50.000 25.000,50.000");
        assert_eq!(polygon(&s[0], Orientation::Horizontal), "0.000,0.000 0.000,100.000 50.000,75.000 50.000,25.000");
        assert_eq!(arrow_alias("ArrowDown"), "ArrowRight");
    }
}
