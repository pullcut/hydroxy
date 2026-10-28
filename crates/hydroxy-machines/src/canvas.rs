//! Shared maths for infinite canvases (`node_editor`, `whiteboard`): points,
//! rectangles, a pan/zoom viewport and two-finger pinch tracking.
//!
//! Screen coordinates are pixels from the canvas element's top-left corner.
//! World coordinates are the scene's own units: `screen = world * zoom + pan`.

/// A point or a vector.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    /// Horizontal, growing right.
    pub x: f64,
    /// Vertical, growing down.
    pub y: f64,
}

impl Point {
    /// A point at `(x, y)`.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// `self + other`.
    #[must_use]
    pub fn plus(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }

    /// `self - other`.
    #[must_use]
    pub fn minus(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }

    /// `self * k`.
    #[must_use]
    pub fn scale(self, k: f64) -> Self {
        Self::new(self.x * k, self.y * k)
    }

    /// Distance to `o`.
    #[must_use]
    pub fn dist(self, o: Self) -> f64 {
        (self.x - o.x).hypot(self.y - o.y)
    }

    /// Halfway to `o`.
    #[must_use]
    pub fn mid(self, o: Self) -> Self {
        Self::new(f64::midpoint(self.x, o.x), f64::midpoint(self.y, o.y))
    }

    /// Rotated by `angle` radians (clockwise on screen) around `pivot`.
    #[must_use]
    pub fn rotate(self, pivot: Self, angle: f64) -> Self {
        if angle == 0.0 {
            return self;
        }
        let (s, c) = angle.sin_cos();
        let d = self.minus(pivot);
        Self::new(pivot.x + d.x * c - d.y * s, pivot.y + d.x * s + d.y * c)
    }
}

/// An axis-aligned rectangle. `w` and `h` are never negative.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl Rect {
    /// A rectangle at `(x, y)`, `w` by `h`.
    #[must_use]
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// The rectangle spanned by two corners, in any order.
    #[must_use]
    pub fn from_corners(a: Point, b: Point) -> Self {
        Self::new(a.x.min(b.x), a.y.min(b.y), (a.x - b.x).abs(), (a.y - b.y).abs())
    }

    /// The smallest rectangle around every point. `None` for no points.
    #[must_use]
    pub fn around(points: impl IntoIterator<Item = Point>) -> Option<Self> {
        let mut it = points.into_iter();
        let first = it.next()?;
        let (mut lo, mut hi) = (first, first);
        for p in it {
            lo = Point::new(lo.x.min(p.x), lo.y.min(p.y));
            hi = Point::new(hi.x.max(p.x), hi.y.max(p.y));
        }
        Some(Self::from_corners(lo, hi))
    }

    /// The centre.
    #[must_use]
    pub fn center(self) -> Point {
        Point::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Right edge.
    #[must_use]
    pub fn right(self) -> f64 {
        self.x + self.w
    }

    /// Bottom edge.
    #[must_use]
    pub fn bottom(self) -> f64 {
        self.y + self.h
    }

    /// The four corners, clockwise from top-left.
    #[must_use]
    pub fn corners(self) -> [Point; 4] {
        [
            Point::new(self.x, self.y),
            Point::new(self.right(), self.y),
            Point::new(self.right(), self.bottom()),
            Point::new(self.x, self.bottom()),
        ]
    }

    /// Whether `p` is inside or on the edge.
    #[must_use]
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }

    /// Whether `o` lies wholly inside.
    #[must_use]
    pub fn contains_rect(self, o: Self) -> bool {
        o.x >= self.x && o.y >= self.y && o.right() <= self.right() && o.bottom() <= self.bottom()
    }

    /// Whether the two overlap.
    #[must_use]
    pub fn intersects(self, o: Self) -> bool {
        self.x <= o.right() && o.x <= self.right() && self.y <= o.bottom() && o.y <= self.bottom()
    }

    /// The smallest rectangle around both.
    #[must_use]
    pub fn union(self, o: Self) -> Self {
        Self::around([
            Point::new(self.x, self.y),
            Point::new(self.right(), self.bottom()),
            Point::new(o.x, o.y),
            Point::new(o.right(), o.bottom()),
        ])
        .unwrap_or(self)
    }

    /// Grown by `by` on every side.
    #[must_use]
    pub fn inflate(self, by: f64) -> Self {
        Self::new(self.x - by, self.y - by, self.w + by * 2.0, self.h + by * 2.0)
    }
}

/// Distance from `p` to the segment `a`–`b`.
#[must_use]
pub fn segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let ab = b.minus(a);
    let len2 = ab.x * ab.x + ab.y * ab.y;
    if len2 <= f64::EPSILON {
        return p.dist(a);
    }
    let t = (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / len2).clamp(0.0, 1.0);
    p.dist(a.plus(ab.scale(t)))
}

/// Smallest and largest zoom a viewport allows.
pub const ZOOM_RANGE: (f64, f64) = (0.1, 4.0);

/// The visible window onto the world: `screen = world * zoom + (x, y)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Horizontal pan in screen pixels.
    pub x: f64,
    /// Vertical pan in screen pixels.
    pub y: f64,
    /// Scale, 1 = 100%.
    pub zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, zoom: 1.0 }
    }
}

impl Viewport {
    /// A screen point in world coordinates.
    #[must_use]
    pub fn to_world(self, p: Point) -> Point {
        Point::new((p.x - self.x) / self.zoom, (p.y - self.y) / self.zoom)
    }

    /// A world point in screen coordinates.
    #[must_use]
    pub fn to_screen(self, p: Point) -> Point {
        Point::new(p.x * self.zoom + self.x, p.y * self.zoom + self.y)
    }

    /// The world rectangle a `size` screen shows.
    #[must_use]
    pub fn visible(self, size: (f64, f64)) -> Rect {
        let a = self.to_world(Point::default());
        Rect::new(a.x, a.y, size.0 / self.zoom, size.1 / self.zoom)
    }

    /// Moved by a screen delta.
    #[must_use]
    pub fn pan_by(self, dx: f64, dy: f64) -> Self {
        Self { x: self.x + dx, y: self.y + dy, ..self }
    }

    /// Zoomed by `factor`, keeping the world point under screen point `at` still.
    #[must_use]
    pub fn zoom_at(self, at: Point, factor: f64) -> Self {
        let zoom = (self.zoom * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        let world = self.to_world(at);
        Self { x: at.x - world.x * zoom, y: at.y - world.y * zoom, zoom }
    }

    /// The viewport that frames `bounds` in a `size` screen with `padding` pixels
    /// around it, never zooming in past 100%.
    #[must_use]
    pub fn fit(bounds: Rect, size: (f64, f64), padding: f64) -> Self {
        let (w, h) = ((size.0 - padding * 2.0).max(1.0), (size.1 - padding * 2.0).max(1.0));
        let zoom = (w / bounds.w.max(1.0)).min(h / bounds.h.max(1.0)).clamp(ZOOM_RANGE.0, 1.0);
        let c = bounds.center();
        Self { x: size.0 / 2.0 - c.x * zoom, y: size.1 / 2.0 - c.y * zoom, zoom }
    }

    /// Centred on world point `p` at the current zoom.
    #[must_use]
    pub fn center_on(self, p: Point, size: (f64, f64)) -> Self {
        Self { x: size.0 / 2.0 - p.x * self.zoom, y: size.1 / 2.0 - p.y * self.zoom, ..self }
    }

    /// The CSS transform for the world layer.
    #[must_use]
    pub fn css(self) -> String {
        format!("transform: translate({:.2}px, {:.2}px) scale({:.4});", self.x, self.y, self.zoom)
    }

    /// Zoom as a whole percentage, for labels.
    #[must_use]
    pub fn percent(self) -> i64 {
        #[allow(clippy::cast_possible_truncation, reason = "zoom is clamped to a small range")]
        let p = (self.zoom * 100.0).round() as i64;
        p
    }
}

/// Pointers held down on a canvas, so two of them make a pinch.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pointers(Vec<(i32, Point)>);

impl Pointers {
    /// A pointer went down at screen point `at`. Returns how many are down.
    pub fn down(&mut self, id: i32, at: Point) -> usize {
        self.0.retain(|p| p.0 != id);
        self.0.push((id, at));
        self.0.len()
    }

    /// A pointer lifted. Returns how many are still down.
    pub fn up(&mut self, id: i32) -> usize {
        self.0.retain(|p| p.0 != id);
        self.0.len()
    }

    /// How many are down.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether none are down.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Pointer `id` moved to `at`. With two down, the viewport after the pinch.
    pub fn pinch(&mut self, id: i32, at: Point, view: Viewport) -> Option<Viewport> {
        let before: Vec<Point> = self.0.iter().take(2).map(|p| p.1).collect();
        let slot = self.0.iter().position(|p| p.0 == id)?;
        self.0[slot].1 = at;
        let after: Vec<Point> = self.0.iter().take(2).map(|p| p.1).collect();
        let (&[b0, b1], &[a0, a1]) = (before.as_slice(), after.as_slice()) else { return None };
        let d0 = b0.dist(b1);
        let factor = if d0 > 1e-6 { a0.dist(a1) / d0 } else { 1.0 };
        let (mb, ma) = (b0.mid(b1), a0.mid(a1));
        Some(view.pan_by(ma.x - mb.x, ma.y - mb.y).zoom_at(ma, factor))
    }
}

#[cfg(test)]
pub(crate) fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_anchor_still() {
        let v = Viewport { x: 30.0, y: -10.0, zoom: 1.0 };
        let at = Point::new(200.0, 120.0);
        let before = v.to_world(at);
        let z = v.zoom_at(at, 2.0);
        assert!(close(z.zoom, 2.0));
        let after = z.to_world(at);
        assert!(close(before.x, after.x) && close(before.y, after.y));
        assert!(close(v.zoom_at(at, 1000.0).zoom, ZOOM_RANGE.1));
    }

    #[test]
    fn fit_centres_and_caps_at_full_size() {
        let v = Viewport::fit(Rect::new(0.0, 0.0, 100.0, 50.0), (400.0, 300.0), 20.0);
        assert!(close(v.zoom, 1.0));
        let c = v.to_screen(Point::new(50.0, 25.0));
        assert!(close(c.x, 200.0) && close(c.y, 150.0));
        let big = Viewport::fit(Rect::new(0.0, 0.0, 2000.0, 1000.0), (440.0, 300.0), 20.0);
        assert!(close(big.zoom, 0.2));
    }

    #[test]
    fn two_pointers_pinch() {
        let mut p = Pointers::default();
        p.down(1, Point::new(100.0, 100.0));
        assert_eq!(p.pinch(1, Point::new(90.0, 100.0), Viewport::default()), None);
        p.down(2, Point::new(200.0, 100.0));
        let v = p.pinch(2, Point::new(300.0, 100.0), Viewport::default()).unwrap();
        assert!(close(v.zoom, 210.0 / 110.0));
    }

    #[test]
    fn geometry_basics() {
        assert!(close(segment_distance(Point::new(5.0, 5.0), Point::new(0.0, 0.0), Point::new(10.0, 0.0)), 5.0));
        let r = Rect::from_corners(Point::new(10.0, 10.0), Point::new(0.0, 4.0));
        assert_eq!(r, Rect::new(0.0, 4.0, 10.0, 6.0));
        assert!(r.contains(Point::new(5.0, 5.0)));
        let p = Point::new(1.0, 0.0).rotate(Point::default(), std::f64::consts::FRAC_PI_2);
        assert!(close(p.x, 0.0) && close(p.y, 1.0));
    }
}
