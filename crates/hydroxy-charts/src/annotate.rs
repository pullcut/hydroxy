//! Reference lines, bands and point labels drawn over a cartesian chart: a target,
//! a threshold, a release date. Charts place them with the same scales as their marks.

/// One annotation, in data units. `x` values are whatever the chart's x axis uses:
/// a category index for bar and line charts, or the numeric x for scatter and time charts.
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    /// A horizontal line across the plot at a value, such as a target.
    YLine {
        /// Where the line sits.
        value: f64,
        /// Text at the right end of the line.
        label: Option<String>,
    },
    /// A shaded horizontal band between two values, such as a normal range.
    YBand {
        /// One edge.
        from: f64,
        /// The other edge.
        to: f64,
        /// Text inside the band.
        label: Option<String>,
    },
    /// A vertical line at an x position, such as a release.
    XLine {
        /// Where the line sits.
        at: f64,
        /// Text at the top of the line.
        label: Option<String>,
    },
    /// A shaded vertical band between two x positions, such as an outage.
    XBand {
        /// One edge.
        from: f64,
        /// The other edge.
        to: f64,
        /// Text inside the band.
        label: Option<String>,
    },
    /// A labelled point, such as a record high.
    Point {
        /// Horizontal position.
        x: f64,
        /// Vertical position.
        y: f64,
        /// Text beside the dot.
        label: String,
    },
}

impl Annotation {
    /// A horizontal reference line.
    #[must_use]
    pub fn y_line(value: f64, label: impl Into<String>) -> Self {
        Self::YLine { value, label: Some(label.into()) }
    }

    /// A horizontal band.
    #[must_use]
    pub fn y_band(from: f64, to: f64, label: impl Into<String>) -> Self {
        Self::YBand { from, to, label: Some(label.into()) }
    }

    /// A vertical reference line.
    #[must_use]
    pub fn x_line(at: f64, label: impl Into<String>) -> Self {
        Self::XLine { at, label: Some(label.into()) }
    }

    /// A vertical band.
    #[must_use]
    pub fn x_band(from: f64, to: f64, label: impl Into<String>) -> Self {
        Self::XBand { from, to, label: Some(label.into()) }
    }

    /// A labelled point.
    #[must_use]
    pub fn point(x: f64, y: f64, label: impl Into<String>) -> Self {
        Self::Point { x, y, label: label.into() }
    }

    /// The label, if any.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::YLine { label, .. }
            | Self::YBand { label, .. }
            | Self::XLine { label, .. }
            | Self::XBand { label, .. } => label.as_deref(),
            Self::Point { label, .. } => Some(label),
        }
    }
}

/// The y values annotations need visible, so a chart can widen its value axis to fit them.
#[must_use]
pub fn y_extent(annotations: &[Annotation]) -> Option<(f64, f64)> {
    let values = annotations.iter().flat_map(|a| match a {
        Annotation::YLine { value, .. } => vec![*value],
        Annotation::YBand { from, to, .. } => vec![*from, *to],
        Annotation::Point { y, .. } => vec![*y],
        Annotation::XLine { .. } | Annotation::XBand { .. } => vec![],
    });
    values
        .filter(|v| v.is_finite())
        .fold(None, |acc, v| Some(acc.map_or((v, v), |(lo, hi): (f64, f64)| (lo.min(v), hi.max(v)))))
}

/// What an annotation looks like once placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// A full-width horizontal line at `y0`.
    HLine,
    /// A full-height vertical line at `x0`.
    VLine,
    /// A full-width band from `y0` to `y1`.
    HBand,
    /// A full-height band from `x0` to `x1`.
    VBand,
    /// A dot at (`x0`, `y0`).
    Point,
}

impl Shape {
    /// The `data-shape` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            Shape::HLine => "h-line",
            Shape::VLine => "v-line",
            Shape::HBand => "h-band",
            Shape::VBand => "v-band",
            Shape::Point => "point",
        }
    }
}

/// An annotation in plot fractions: x from the left, y from the bottom, both 0..=1.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// How to draw it.
    pub shape: Shape,
    /// Left edge or position.
    pub x0: f64,
    /// Right edge (bands only; equals `x0` otherwise).
    pub x1: f64,
    /// Bottom edge or position.
    pub y0: f64,
    /// Top edge (bands only; equals `y0` otherwise).
    pub y1: f64,
    /// Text to show, if any.
    pub label: Option<String>,
}

impl Placed {
    /// CSS that positions it inside a `position: relative` plot, using percentages.
    #[must_use]
    pub fn style(&self) -> String {
        let p = |f: f64| format!("{:.3}%", f * 100.0);
        match self.shape {
            Shape::HLine => format!("bottom: {};", p(self.y0)),
            Shape::VLine => format!("left: {};", p(self.x0)),
            Shape::HBand => format!("bottom: {}; height: {};", p(self.y0), p(self.y1 - self.y0)),
            Shape::VBand => format!("left: {}; width: {};", p(self.x0), p(self.x1 - self.x0)),
            Shape::Point => format!("left: {}; bottom: {};", p(self.x0), p(self.y0)),
        }
    }
}

/// Place annotations with the chart's own mappings from data to plot fractions.
/// Anything entirely outside the plot is dropped; bands are clipped to it.
pub fn place(annotations: &[Annotation], x: impl Fn(f64) -> f64, y: impl Fn(f64) -> f64) -> Vec<Placed> {
    let inside = |f: f64| (-1e-9..=1.0 + 1e-9).contains(&f);
    let span = |a: f64, b: f64| {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        (hi >= 0.0 && lo <= 1.0).then(|| (lo.max(0.0), hi.min(1.0)))
    };
    annotations
        .iter()
        .filter_map(|a| {
            let label = a.label().map(str::to_string);
            let placed = |shape, x0, x1, y0, y1| Placed { shape, x0, x1, y0, y1, label: label.clone() };
            match a {
                Annotation::YLine { value, .. } => {
                    let f = y(*value);
                    inside(f).then(|| placed(Shape::HLine, 0.0, 1.0, f, f))
                }
                Annotation::XLine { at, .. } => {
                    let f = x(*at);
                    inside(f).then(|| placed(Shape::VLine, f, f, 0.0, 1.0))
                }
                Annotation::YBand { from, to, .. } => {
                    span(y(*from), y(*to)).map(|(lo, hi)| placed(Shape::HBand, 0.0, 1.0, lo, hi))
                }
                Annotation::XBand { from, to, .. } => {
                    span(x(*from), x(*to)).map(|(lo, hi)| placed(Shape::VBand, lo, hi, 0.0, 1.0))
                }
                Annotation::Point { x: px, y: py, .. } => {
                    let (fx, fy) = (x(*px), y(*py));
                    (inside(fx) && inside(fy)).then(|| placed(Shape::Point, fx, fx, fy, fy))
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_lines_bands_and_points() {
        let notes = [
            Annotation::y_line(50.0, "Target"),
            Annotation::x_band(1.0, 2.0, "Outage"),
            Annotation::point(3.0, 80.0, "Peak"),
            Annotation::y_line(500.0, "Off the chart"),
        ];
        let placed = place(&notes, |x| x / 4.0, |y| y / 100.0);
        assert_eq!(placed.len(), 3);
        assert_eq!(placed[0].shape, Shape::HLine);
        assert!((placed[0].y0 - 0.5).abs() < 1e-9);
        assert_eq!((placed[1].x0, placed[1].x1), (0.25, 0.5));
        assert_eq!(placed[2].label.as_deref(), Some("Peak"));
        assert_eq!(placed[0].style(), "bottom: 50.000%;");
    }

    #[test]
    fn bands_clip_to_the_plot() {
        let placed = place(&[Annotation::y_band(-10.0, 30.0, "Low")], |x| x, |y| y / 100.0);
        assert_eq!((placed[0].y0, placed[0].y1), (0.0, 0.3));
    }

    #[test]
    fn extent_covers_y_values_only() {
        let notes = [
            Annotation::y_line(120.0, "Target"),
            Annotation::x_line(3.0, "Launch"),
            Annotation::point(1.0, -5.0, "Dip"),
        ];
        assert_eq!(y_extent(&notes), Some((-5.0, 120.0)));
        assert_eq!(y_extent(&[Annotation::x_line(1.0, "x")]), None);
    }
}
