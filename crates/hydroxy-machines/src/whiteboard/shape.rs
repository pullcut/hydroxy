//! Shapes: colours, tools, the shape model, binding and hit-testing.

use crate::canvas::segment_distance;

use super::{Point, Rect};

/// Gap, in world units, between a bound arrow's tip and its shape.
pub(super) const BIND_GAP: f64 = 6.0;
/// Font size of sticky notes.
pub const STICKY_FONT: f64 = 16.0;
/// Font size of text shapes.
pub const TEXT_FONT: f64 = 20.0;
/// Padding inside sticky notes.
pub const STICKY_PAD: f64 = 14.0;

/// A colour from the design tokens: name, CSS custom property and a fallback for export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swatch {
    /// Shown to people.
    pub name: &'static str,
    /// The token, such as `--chart-1`.
    pub token: &'static str,
    /// Used when the token isn't defined (an exported file opened elsewhere).
    pub fallback: &'static str,
}

impl Swatch {
    /// `var(--token, #fallback)`.
    #[must_use]
    pub fn css(self) -> String {
        format!("var({}, {})", self.token, self.fallback)
    }
}

/// The colours shapes can take, in menu order.
pub const PALETTE: [Swatch; 7] = [
    Swatch { name: "Ink", token: "--text-1", fallback: "#161e35" },
    Swatch { name: "Amethyst", token: "--chart-1", fallback: "#8772dd" },
    Swatch { name: "Garnet", token: "--chart-2", fallback: "#d45954" },
    Swatch { name: "Sapphire", token: "--chart-3", fallback: "#2c8dd2" },
    Swatch { name: "Emerald", token: "--chart-4", fallback: "#159d73" },
    Swatch { name: "Citrine", token: "--chart-5", fallback: "#a87f0a" },
    Swatch { name: "Rhodolite", token: "--chart-6", fallback: "#be5fa2" },
];

/// The swatch for a palette index, clamped.
#[must_use]
pub fn swatch(i: usize) -> Swatch {
    PALETTE[i.min(PALETTE.len() - 1)]
}

/// What a pointer press on the canvas does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    /// Pick, move, resize and rotate.
    #[default]
    Select,
    /// Drag to pan.
    Hand,
    /// Draw a rectangle.
    Rect,
    /// Draw an ellipse.
    Ellipse,
    /// Draw an arrow; its ends bind to shapes they start or finish on.
    Arrow,
    /// Freehand ink.
    Pen,
    /// A sticky note.
    Sticky,
    /// A line of text.
    Text,
}

impl Tool {
    /// Every tool in toolbar order.
    pub const ALL: [Self; 8] =
        [Self::Select, Self::Hand, Self::Rect, Self::Ellipse, Self::Arrow, Self::Pen, Self::Sticky, Self::Text];

    /// Name for buttons and announcements.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Hand => "Hand",
            Self::Rect => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Arrow => "Arrow",
            Self::Pen => "Pen",
            Self::Sticky => "Sticky note",
            Self::Text => "Text",
        }
    }

    /// Its keyboard shortcut.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Select => "V",
            Self::Hand => "H",
            Self::Rect => "R",
            Self::Ellipse => "O",
            Self::Arrow => "A",
            Self::Pen => "P",
            Self::Sticky => "N",
            Self::Text => "T",
        }
    }

    /// `data-tool` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Hand => "hand",
            Self::Rect => "rect",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Pen => "pen",
            Self::Sticky => "sticky",
            Self::Text => "text",
        }
    }

    pub(super) fn kind(self) -> Option<ShapeKind> {
        Some(match self {
            Self::Rect => ShapeKind::Rect,
            Self::Ellipse => ShapeKind::Ellipse,
            Self::Arrow => ShapeKind::Arrow,
            Self::Pen => ShapeKind::Pen,
            Self::Sticky => ShapeKind::Sticky,
            Self::Text => ShapeKind::Text,
            Self::Select | Self::Hand => return None,
        })
    }
}

/// What a shape is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    /// Rectangle.
    Rect,
    /// Ellipse inside the box.
    Ellipse,
    /// Straight arrow between two points.
    Arrow,
    /// Freehand stroke.
    Pen,
    /// Coloured note with wrapped text.
    Sticky,
    /// Text on its own.
    Text,
}

impl ShapeKind {
    /// JSON and `data-kind` name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rect => "rect",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Pen => "pen",
            Self::Sticky => "sticky",
            Self::Text => "text",
        }
    }

    /// Name for people.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rect => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Arrow => "Arrow",
            Self::Pen => "Drawing",
            Self::Sticky => "Sticky note",
            Self::Text => "Text",
        }
    }

    pub(super) fn default_size(self) -> (f64, f64) {
        match self {
            Self::Rect => (160.0, 100.0),
            Self::Ellipse => (140.0, 100.0),
            Self::Sticky => (200.0, 160.0),
            Self::Text => (240.0, 32.0),
            Self::Arrow | Self::Pen => (120.0, 0.0),
        }
    }

    pub(super) const fn has_text(self) -> bool {
        matches!(self, Self::Sticky | Self::Text)
    }
}

/// One item on the board.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Unique key.
    pub id: String,
    /// What it is.
    pub kind: ShapeKind,
    /// Box left edge, before rotation. For arrows, the box around the two ends.
    pub x: f64,
    /// Box top edge, before rotation.
    pub y: f64,
    /// Box width.
    pub w: f64,
    /// Box height.
    pub h: f64,
    /// Clockwise rotation around the box centre, in radians. Arrows don't rotate.
    pub rotation: f64,
    /// Index into [`PALETTE`].
    pub color: usize,
    /// Pen: points as fractions of the box (0–1), so resizing scales the ink.
    /// Arrow: the start and end in world units.
    pub points: Vec<Point>,
    /// Arrow: the shape its start is attached to.
    pub start: Option<String>,
    /// Arrow: the shape its end is attached to.
    pub end: Option<String>,
    /// Sticky and text content.
    pub text: String,
}

impl Shape {
    /// A `kind` shape filling `rect`.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: ShapeKind, rect: Rect) -> Self {
        Self {
            id: id.into(),
            kind,
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            rotation: 0.0,
            color: 0,
            points: Vec::new(),
            start: None,
            end: None,
            text: String::new(),
        }
    }

    /// An arrow from `a` to `b`.
    #[must_use]
    pub fn arrow(id: impl Into<String>, a: Point, b: Point) -> Self {
        let mut s = Self::new(id, ShapeKind::Arrow, Rect::from_corners(a, b));
        s.points = vec![a, b];
        s
    }

    /// Set the colour.
    #[must_use]
    pub fn color(mut self, i: usize) -> Self {
        self.color = i;
        self
    }

    /// Set the text.
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Bind an arrow's ends.
    #[must_use]
    pub fn bind(mut self, start: Option<&str>, end: Option<&str>) -> Self {
        self.start = start.map(str::to_string);
        self.end = end.map(str::to_string);
        self
    }

    /// The unrotated box.
    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }

    pub(super) fn center(&self) -> Point {
        self.rect().center()
    }

    pub(super) fn set_center(&mut self, c: Point) {
        self.x = c.x - self.w / 2.0;
        self.y = c.y - self.h / 2.0;
    }

    /// Pen points in world units.
    #[must_use]
    pub fn ink(&self) -> Vec<Point> {
        self.points.iter().map(|p| Point::new(self.x + p.x * self.w, self.y + p.y * self.h)).collect()
    }

    pub(super) fn translate(&mut self, d: Point) {
        self.x += d.x;
        self.y += d.y;
        if self.kind == ShapeKind::Arrow {
            for p in &mut self.points {
                *p = p.plus(d);
            }
        }
    }

    pub(super) fn fit_arrow_box(&mut self) {
        if let Some(r) = Rect::around(self.points.iter().copied()) {
            (self.x, self.y, self.w, self.h) = (r.x, r.y, r.w, r.h);
        }
    }
}

/// Where an arrow's ends are, after following the shapes they are bound to.
#[must_use]
pub fn arrow_ends(arrow: &Shape, shapes: &[Shape]) -> (Point, Point) {
    let a = arrow.points.first().copied().unwrap_or_default();
    let b = arrow.points.get(1).copied().unwrap_or(a);
    let find = |id: &Option<String>| {
        id.as_ref().and_then(|id| shapes.iter().find(|s| s.id == *id && s.kind != ShapeKind::Arrow))
    };
    let (sa, sb) = (find(&arrow.start), find(&arrow.end));
    let aim_a = sb.map_or(b, Shape::center);
    let aim_b = sa.map_or(a, Shape::center);
    let a = sa.map_or(a, |s| boundary(s, aim_a));
    let b = sb.map_or(b, |s| boundary(s, aim_b));
    (a, b)
}

/// Where a ray from `s`'s centre towards `toward` leaves the shape, plus a small gap.
fn boundary(s: &Shape, toward: Point) -> Point {
    let c = s.center();
    let local = toward.rotate(c, -s.rotation).minus(c);
    let (hw, hh) = (s.w / 2.0 + BIND_GAP, s.h / 2.0 + BIND_GAP);
    if local.x.abs() < 1e-9 && local.y.abs() < 1e-9 {
        return c;
    }
    let k = if s.kind == ShapeKind::Ellipse {
        1.0 / ((local.x / hw).powi(2) + (local.y / hh).powi(2)).sqrt()
    } else {
        let kx = if local.x.abs() > 1e-9 { hw / local.x.abs() } else { f64::INFINITY };
        let ky = if local.y.abs() > 1e-9 { hh / local.y.abs() } else { f64::INFINITY };
        kx.min(ky)
    };
    c.plus(local.scale(k.min(1.0))).rotate(c, s.rotation)
}

/// The axis-aligned box around a shape as drawn.
#[must_use]
pub fn bounds(s: &Shape, shapes: &[Shape]) -> Rect {
    match s.kind {
        ShapeKind::Arrow => {
            let (a, b) = arrow_ends(s, shapes);
            Rect::from_corners(a, b)
        }
        _ if s.rotation == 0.0 => s.rect(),
        _ => Rect::around(s.rect().corners().map(|p| p.rotate(s.center(), s.rotation))).unwrap_or(s.rect()),
    }
}

/// Whether world point `p` touches `s`, within `tol` world units.
#[must_use]
pub fn contains(s: &Shape, p: Point, tol: f64, shapes: &[Shape]) -> bool {
    let local = p.rotate(s.center(), -s.rotation);
    match s.kind {
        ShapeKind::Rect | ShapeKind::Sticky | ShapeKind::Text => s.rect().inflate(tol).contains(local),
        ShapeKind::Ellipse => {
            let c = s.center();
            let (hw, hh) = ((s.w / 2.0 + tol).max(1e-6), (s.h / 2.0 + tol).max(1e-6));
            ((local.x - c.x) / hw).powi(2) + ((local.y - c.y) / hh).powi(2) <= 1.0
        }
        ShapeKind::Arrow => {
            let (a, b) = arrow_ends(s, shapes);
            segment_distance(p, a, b) <= tol + 2.0
        }
        ShapeKind::Pen => {
            let ink = s.ink();
            match ink.as_slice() {
                [only] => only.dist(local) <= tol + 2.0,
                _ => ink.windows(2).any(|w| segment_distance(local, w[0], w[1]) <= tol + 2.0),
            }
        }
    }
}

/// The topmost shape under `p`, by index.
#[must_use]
pub fn shape_at(shapes: &[Shape], p: Point, tol: f64, skip_arrows: bool) -> Option<usize> {
    shapes.iter().rposition(|s| !(skip_arrows && s.kind == ShapeKind::Arrow) && contains(s, p, tol, shapes))
}
