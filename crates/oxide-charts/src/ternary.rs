//! Ternary plots: three proportions that sum to a whole, drawn in an equilateral
//! triangle. A sits at the bottom left, B at the bottom right and C at the top.
//! Positions are fractions of the triangle's bounding box: x from the left, y from
//! the bottom. The box is `1 : HEIGHT` wide to tall.

/// Height of a unit-sided equilateral triangle.
pub const HEIGHT: f64 = 0.866_025_403_784_438_6;

/// A composition, in any units: it is normalised to shares.
#[derive(Debug, Clone, PartialEq)]
pub struct TernaryPoint {
    /// What the point is, such as a sample name.
    pub label: String,
    /// The first component.
    pub a: f64,
    /// The second component.
    pub b: f64,
    /// The third component.
    pub c: f64,
}

impl TernaryPoint {
    /// A labelled composition.
    #[must_use]
    pub fn new(label: impl Into<String>, a: f64, b: f64, c: f64) -> Self {
        Self { label: label.into(), a, b, c }
    }

    /// The three shares, summing to one. `None` when a component is negative or
    /// non-finite, or all are zero.
    #[must_use]
    pub fn shares(&self) -> Option<(f64, f64, f64)> {
        let ok = |v: f64| v.is_finite() && v >= 0.0;
        let sum = self.a + self.b + self.c;
        (ok(self.a) && ok(self.b) && ok(self.c) && sum > 0.0).then(|| (self.a / sum, self.b / sum, self.c / sum))
    }

    /// Position in the box, or `None` if [`Self::shares`] is.
    #[must_use]
    pub fn position(&self) -> Option<(f64, f64)> {
        self.shares().map(|(a, b, c)| project(a, b, c))
    }
}

/// A named set of points, drawn in one colour.
#[derive(Debug, Clone, PartialEq)]
pub struct TernaryGroup {
    /// Shown in the legend, tooltip and table.
    pub name: String,
    /// The compositions.
    pub points: Vec<TernaryPoint>,
}

impl TernaryGroup {
    /// A named group.
    #[must_use]
    pub fn new(name: impl Into<String>, points: impl Into<Vec<TernaryPoint>>) -> Self {
        Self { name: name.into(), points: points.into() }
    }
}

/// A drawable point: which group and point it is, and where it sits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dot {
    /// Index into the groups.
    pub group: usize,
    /// Index into that group's points.
    pub index: usize,
    /// Position in the box, x from the left and y from the bottom.
    pub at: (f64, f64),
}

/// Every point with a valid composition, group by group in input order.
#[must_use]
pub fn dots(groups: &[TernaryGroup]) -> Vec<Dot> {
    groups
        .iter()
        .enumerate()
        .flat_map(|(g, grp)| grp.points.iter().enumerate().filter_map(move |(i, p)| p.position().map(|at| Dot { group: g, index: i, at })))
        .collect()
}

/// Box position for shares that already sum to one.
#[must_use]
pub fn project(_a: f64, b: f64, c: f64) -> (f64, f64) {
    (b + c / 2.0, c)
}

/// A line of equal share for one component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLine {
    /// Which component: 0 for A, 1 for B, 2 for C.
    pub axis: usize,
    /// The share it marks.
    pub value: f64,
    /// The labelled end.
    pub from: (f64, f64),
    /// The other end.
    pub to: (f64, f64),
}

#[allow(clippy::cast_precision_loss, reason = "division counts are tiny")]
/// Gridlines at every `1 / divisions` step for each component, excluding the edges.
/// A is labelled on the left edge, B along the bottom and C on the right edge.
#[must_use]
pub fn grid(divisions: usize) -> Vec<GridLine> {
    let n = divisions.max(1);
    (1..n)
        .flat_map(|k| {
            let t = k as f64 / n as f64;
            let u = 1.0 - t;
            [
                GridLine { axis: 0, value: t, from: project(t, 0.0, u), to: project(t, u, 0.0) },
                GridLine { axis: 1, value: t, from: project(u, t, 0.0), to: project(0.0, t, u) },
                GridLine { axis: 2, value: t, from: project(0.0, u, t), to: project(u, 0.0, t) },
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_and_centre() {
        assert_eq!(project(1.0, 0.0, 0.0), (0.0, 0.0));
        assert_eq!(project(0.0, 1.0, 0.0), (1.0, 0.0));
        assert_eq!(project(0.0, 0.0, 1.0), (0.5, 1.0));
        let (x, y) = TernaryPoint::new("Even", 2.0, 2.0, 2.0).position().unwrap_or_default();
        assert!((x - 0.5).abs() < 1e-9 && (y - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn dots_skip_invalid_points() {
        let groups = [TernaryGroup::new("Soil", [TernaryPoint::new("Loam", 40.0, 40.0, 20.0), TernaryPoint::new("Bad", 0.0, 0.0, 0.0)])];
        let d = dots(&groups);
        assert_eq!(d.len(), 1);
        assert_eq!((d[0].group, d[0].index), (0, 0));
    }

    #[test]
    fn invalid_compositions_have_no_position() {
        assert_eq!(TernaryPoint::new("Zero", 0.0, 0.0, 0.0).position(), None);
        assert_eq!(TernaryPoint::new("Neg", -1.0, 2.0, 2.0).position(), None);
        assert_eq!(TernaryPoint::new("NaN", f64::NAN, 2.0, 2.0).position(), None);
    }

    #[test]
    fn gridlines_run_edge_to_edge() {
        let g = grid(5);
        assert_eq!(g.len(), 12);
        // 20% C: a horizontal line a fifth of the way up.
        let c = g.iter().find(|l| l.axis == 2 && (l.value - 0.2).abs() < 1e-9).copied();
        assert!(c.is_some_and(|l| (l.from.1 - 0.2).abs() < 1e-9 && (l.to.1 - 0.2).abs() < 1e-9));
        // A labels sit on the left edge, where x = y / 2.
        assert!(g.iter().filter(|l| l.axis == 0).all(|l| (l.from.0 - l.from.1 / 2.0).abs() < 1e-9));
    }
}
