//! Whiteboard: an infinite canvas of shapes, arrows, freehand ink, sticky
//! notes and text. Tools draw; the select tool moves, resizes and rotates.
//! Arrows can bind to shapes at either end and then follow them.
//!
//! Everything here is pure: hit-testing, transforms, smoothing, history and
//! the JSON and SVG export. Pointer events arrive in screen pixels.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::Arc;

use hydroxy_state::History;
use hydroxy_statechart::{Machine, Transition};

pub use crate::canvas::{Point, Rect, Viewport};
use crate::canvas::{Pointers, segment_distance};

/// Screen pixels within which a handle or thin stroke counts as hit.
const TOLERANCE: f64 = 8.0;
/// Gap, in world units, between a bound arrow's tip and its shape.
const BIND_GAP: f64 = 6.0;
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
    pub const ALL: [Self; 8] = [Self::Select, Self::Hand, Self::Rect, Self::Ellipse, Self::Arrow, Self::Pen, Self::Sticky, Self::Text];

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

    fn kind(self) -> Option<ShapeKind> {
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

    fn default_size(self) -> (f64, f64) {
        match self {
            Self::Rect => (160.0, 100.0),
            Self::Ellipse => (140.0, 100.0),
            Self::Sticky => (200.0, 160.0),
            Self::Text => (240.0, 32.0),
            Self::Arrow | Self::Pen => (120.0, 0.0),
        }
    }

    const fn has_text(self) -> bool {
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
        Self { id: id.into(), kind, x: rect.x, y: rect.y, w: rect.w, h: rect.h, rotation: 0.0, color: 0, points: Vec::new(), start: None, end: None, text: String::new() }
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

    fn center(&self) -> Point {
        self.rect().center()
    }

    fn set_center(&mut self, c: Point) {
        self.x = c.x - self.w / 2.0;
        self.y = c.y - self.h / 2.0;
    }

    /// Pen points in world units.
    #[must_use]
    pub fn ink(&self) -> Vec<Point> {
        self.points.iter().map(|p| Point::new(self.x + p.x * self.w, self.y + p.y * self.h)).collect()
    }

    fn translate(&mut self, d: Point) {
        self.x += d.x;
        self.y += d.y;
        if self.kind == ShapeKind::Arrow {
            for p in &mut self.points {
                *p = p.plus(d);
            }
        }
    }

    fn fit_arrow_box(&mut self) {
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
    let find = |id: &Option<String>| id.as_ref().and_then(|id| shapes.iter().find(|s| s.id == *id && s.kind != ShapeKind::Arrow));
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

/// Ramer–Douglas–Peucker: drop points within `epsilon` of the line they sit on.
#[must_use]
pub fn simplify(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    let (mut index, mut far) = (0, 0.0);
    for (i, p) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let d = segment_distance(*p, first, last);
        if d > far {
            (index, far) = (i, d);
        }
    }
    if far <= epsilon {
        return vec![first, last];
    }
    let mut left = simplify(&points[..=index], epsilon);
    left.pop();
    left.extend(simplify(&points[index..], epsilon));
    left
}

/// A smooth SVG path through `points`: quadratic curves between midpoints.
#[must_use]
pub fn smooth_path(points: &[Point]) -> String {
    let mut d = String::new();
    match points {
        [] => {}
        [p] => {
            let _ = write!(d, "M{:.1} {:.1}l0.1 0", p.x, p.y);
        }
        [a, b] => {
            let _ = write!(d, "M{:.1} {:.1}L{:.1} {:.1}", a.x, a.y, b.x, b.y);
        }
        _ => {
            let _ = write!(d, "M{:.1} {:.1}", points[0].x, points[0].y);
            for w in points.windows(2).skip(1) {
                let m = w[0].mid(w[1]);
                let _ = write!(d, "Q{:.1} {:.1} {:.1} {:.1}", w[0].x, w[0].y, m.x, m.y);
            }
            let l = points[points.len() - 1];
            let _ = write!(d, "L{:.1} {:.1}", l.x, l.y);
        }
    }
    d
}

/// A selection handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    /// Top edge.
    N,
    /// Top-right corner.
    Ne,
    /// Right edge.
    E,
    /// Bottom-right corner.
    Se,
    /// Bottom edge.
    S,
    /// Bottom-left corner.
    Sw,
    /// Left edge.
    W,
    /// Top-left corner.
    Nw,
    /// Above the top edge: rotate.
    Rotate,
    /// An arrow's start.
    Start,
    /// An arrow's end.
    End,
}

impl Handle {
    /// The eight resize handles.
    pub const RESIZE: [Self; 8] = [Self::Nw, Self::N, Self::Ne, Self::E, Self::Se, Self::S, Self::Sw, Self::W];

    /// Direction from the centre, each axis -1, 0 or 1.
    #[must_use]
    pub const fn dir(self) -> (f64, f64) {
        match self {
            Self::N => (0.0, -1.0),
            Self::Ne => (1.0, -1.0),
            Self::E => (1.0, 0.0),
            Self::Se => (1.0, 1.0),
            Self::S => (0.0, 1.0),
            Self::Sw => (-1.0, 1.0),
            Self::W => (-1.0, 0.0),
            Self::Nw => (-1.0, -1.0),
            Self::Rotate | Self::Start | Self::End => (0.0, 0.0),
        }
    }

    /// `data-handle` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::N => "n",
            Self::Ne => "ne",
            Self::E => "e",
            Self::Se => "se",
            Self::S => "s",
            Self::Sw => "sw",
            Self::W => "w",
            Self::Nw => "nw",
            Self::Rotate => "rotate",
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// The selection's box: a rotated rectangle for one shape, the bounds of all for several.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Centre, world.
    pub center: Point,
    /// Width before rotation.
    pub w: f64,
    /// Height before rotation.
    pub h: f64,
    /// Rotation, radians.
    pub angle: f64,
}

impl Frame {
    /// Where a resize or rotate handle sits, world. `zoom` keeps the rotate stalk a fixed screen length.
    #[must_use]
    pub fn handle(self, h: Handle, zoom: f64) -> Point {
        let (dx, dy) = h.dir();
        let local = if h == Handle::Rotate { Point::new(0.0, -self.h / 2.0 - 28.0 / zoom) } else { Point::new(dx * self.w / 2.0, dy * self.h / 2.0) };
        self.center.plus(local).rotate(self.center, self.angle)
    }

    /// The SVG `transform` placing an unrotated `w`×`h` box at the origin onto this frame.
    #[must_use]
    pub fn transform(self) -> String {
        let mut t = format!("translate({:.2} {:.2})", self.center.x - self.w / 2.0, self.center.y - self.h / 2.0);
        if self.angle != 0.0 {
            let _ = write!(t, " rotate({:.3} {:.2} {:.2})", self.angle.to_degrees(), self.w / 2.0, self.h / 2.0);
        }
        t
    }
}

/// A gesture in progress.
#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    /// Moving the view.
    Pan {
        /// Last pointer position, screen.
        last: Point,
    },
    /// Moving the selection.
    Move {
        /// Pointer at the start, world.
        start: Point,
        /// Shapes before, for undo and to move from.
        before: Arc<Vec<Shape>>,
    },
    /// Dragging a resize handle.
    Resize {
        /// Which one.
        handle: Handle,
        /// The frame when it started.
        frame: Frame,
        /// Shapes before.
        before: Arc<Vec<Shape>>,
    },
    /// Dragging the rotate handle.
    Rotate {
        /// Pivot, world.
        center: Point,
        /// Pointer angle at the start.
        from: f64,
        /// Shapes before.
        before: Arc<Vec<Shape>>,
    },
    /// Dragging one end of an arrow.
    Retarget {
        /// Which end.
        handle: Handle,
        /// Shapes before.
        before: Arc<Vec<Shape>>,
    },
    /// Drawing a selection box.
    Marquee {
        /// Start corner, world.
        start: Point,
        /// Opposite corner, world.
        current: Point,
        /// Selection kept (Shift adds).
        base: BTreeSet<String>,
    },
    /// Drawing a new shape.
    Create {
        /// The new shape's id.
        id: String,
        /// Pointer at the start, world.
        start: Point,
        /// Freehand points so far, world.
        ink: Vec<Point>,
        /// Shapes before.
        before: Arc<Vec<Shape>>,
    },
    /// Two pointers: zoom and pan.
    Pinch,
}

/// Props for [`WhiteboardMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhiteboardInput {
    /// The scene to start with.
    pub default_shapes: Vec<Shape>,
}

/// What [`WhiteboardMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhiteboardState {
    /// Every shape in paint order: later is on top.
    pub shapes: Vec<Shape>,
    /// Selected ids.
    pub selection: BTreeSet<String>,
    /// The active tool.
    pub tool: Tool,
    /// Palette index for new shapes.
    pub color: usize,
    /// Pan and zoom.
    pub viewport: Viewport,
    /// Canvas size in screen pixels.
    pub size: (f64, f64),
    /// What the pointer is doing.
    pub gesture: Option<Gesture>,
    /// Pointers down.
    pub pointers: Pointers,
    /// The sticky or text shape whose text is being edited, and the scene before.
    pub editing: Option<(String, Arc<Vec<Shape>>)>,
    /// A shape an arrow end would bind to on release.
    pub bind_hint: Option<String>,
    /// Undo and redo snapshots of the shapes.
    pub history: History<Vec<Shape>>,
    next: u64,
}

/// Inputs to [`WhiteboardMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum WhiteboardEvent {
    /// The canvas element's size.
    Resize(f64, f64),
    /// A pointer pressed on the canvas.
    PointerDown {
        /// `pointerId`.
        id: i32,
        /// Screen point.
        at: Point,
        /// Shift held.
        shift: bool,
        /// Middle button or Space held: pan.
        pan: bool,
    },
    /// A pointer moved.
    PointerMove {
        /// `pointerId`.
        id: i32,
        /// Screen point.
        at: Point,
        /// Shift held: keep proportions, snap angles.
        shift: bool,
    },
    /// A pointer lifted.
    PointerUp {
        /// `pointerId`.
        id: i32,
    },
    /// The browser took the pointer away.
    PointerCancel(i32),
    /// Double click: edit a note's text, or add text on empty canvas.
    DoubleClick(Point),
    /// Zoom by `factor` around screen point `at`.
    Zoom {
        /// Screen point.
        at: Point,
        /// Multiplier.
        factor: f64,
    },
    /// Zoom in a step.
    ZoomIn,
    /// Zoom out a step.
    ZoomOut,
    /// Back to 100%.
    ZoomReset,
    /// Frame every shape.
    Fit,
    /// Move the view by screen pixels.
    Pan(f64, f64),
    /// Pick a tool.
    Tool(Tool),
    /// Pick a colour; recolours the selection too.
    Color(usize),
    /// Enter: add a shape with the current tool at the centre, or edit the selected note.
    Activate,
    /// Move the selection by world units (or the view, when nothing is selected).
    Nudge {
        /// Right.
        dx: f64,
        /// Down.
        dy: f64,
    },
    /// Grow the selection.
    Grow {
        /// Wider.
        dw: f64,
        /// Taller.
        dh: f64,
    },
    /// Turn the selection by degrees, clockwise.
    Rotate(f64),
    /// Change paint order.
    Order(Order),
    /// Select the next (`1`) or previous (`-1`) shape.
    Step(i32),
    /// Delete the selection.
    Delete,
    /// Copy the selection, offset.
    Duplicate,
    /// Select everything.
    SelectAll,
    /// Undo.
    Undo,
    /// Redo.
    Redo,
    /// Escape.
    Cancel,
    /// The text editor's new value.
    EditText(String),
    /// Leave the text editor.
    EditEnd,
}

/// Paint order moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// One step up.
    Forward,
    /// One step down.
    Backward,
    /// To the top.
    Front,
    /// To the bottom.
    Back,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq)]
pub enum WhiteboardCommand {
    /// The scene changed (on commit, not every frame).
    Change(Vec<Shape>),
    /// Say this in the live region.
    Announce(String),
    /// Put focus back on the canvas (after text editing).
    FocusCanvas,
}

/// Scene, tools, selection, viewport and history for a whiteboard.
#[derive(Debug, Clone, Copy)]
pub struct WhiteboardMachine;

type Step = Transition<WhiteboardState, WhiteboardCommand>;

fn plural(n: usize) -> String {
    if n == 1 { "1 shape".into() } else { format!("{n} shapes") }
}

/// A short description of a shape for screen readers.
#[must_use]
pub fn describe(s: &Shape) -> String {
    let mut out = format!("{} {}", swatch(s.color).name, s.kind.label().to_lowercase());
    if !s.text.trim().is_empty() {
        let text: String = s.text.chars().take(60).collect();
        let _ = write!(out, ": {}", text.trim());
    }
    out
}

impl WhiteboardState {
    /// The selected shapes in paint order.
    pub fn selected(&self) -> impl Iterator<Item = &Shape> {
        self.shapes.iter().filter(|s| self.selection.contains(&s.id))
    }

    /// The selection's frame, if anything is selected.
    #[must_use]
    pub fn frame(&self) -> Option<Frame> {
        let mut it = self.selected();
        let first = it.next()?;
        if it.next().is_none() && first.kind != ShapeKind::Arrow {
            return Some(Frame { center: first.center(), w: first.w, h: first.h, angle: first.rotation });
        }
        let r = self.selected().map(|s| bounds(s, &self.shapes)).reduce(Rect::union)?;
        Some(Frame { center: r.center(), w: r.w, h: r.h, angle: 0.0 })
    }

    /// The one selected arrow, if that is the whole selection.
    #[must_use]
    pub fn lone_arrow(&self) -> Option<&Shape> {
        let mut it = self.selected();
        let first = it.next()?;
        (it.next().is_none() && first.kind == ShapeKind::Arrow).then_some(first)
    }

    /// The selection box, world, while one is drawn.
    #[must_use]
    pub fn marquee(&self) -> Option<Rect> {
        match &self.gesture {
            Some(Gesture::Marquee { start, current, .. }) => Some(Rect::from_corners(*start, *current)),
            _ => None,
        }
    }

    /// The shape whose text is being edited.
    #[must_use]
    pub fn editing(&self) -> Option<&Shape> {
        let (id, _) = self.editing.as_ref()?;
        self.shapes.iter().find(|s| s.id == *id)
    }

    /// Whether undo would do anything.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether redo would do anything.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Whether the pointer is busy with something other than drawing.
    #[must_use]
    pub fn panning(&self) -> bool {
        matches!(self.gesture, Some(Gesture::Pan { .. } | Gesture::Pinch))
    }

    fn tol(&self) -> f64 {
        TOLERANCE / self.viewport.zoom
    }

    fn fresh_id(&mut self) -> String {
        loop {
            self.next += 1;
            let id = format!("s{}", self.next);
            if !self.shapes.iter().any(|s| s.id == id) {
                return id;
            }
        }
    }

    fn snapshot(&self) -> Arc<Vec<Shape>> {
        Arc::new(self.shapes.clone())
    }

    fn commit(&mut self, before: Arc<Vec<Shape>>, out: &mut Vec<WhiteboardCommand>) {
        if *before == self.shapes {
            return;
        }
        self.history.record(before);
        out.push(WhiteboardCommand::Change(self.shapes.clone()));
    }

    /// Drop selected ids and arrow bindings that point at missing shapes.
    fn prune(&mut self) {
        let ids: BTreeSet<String> = self.shapes.iter().map(|s| s.id.clone()).collect();
        self.selection.retain(|id| ids.contains(id));
        for s in &mut self.shapes {
            for end in [&mut s.start, &mut s.end] {
                if end.as_ref().is_some_and(|id| !ids.contains(id)) {
                    *end = None;
                }
            }
        }
    }

    /// Before shapes in `gone` are removed, pin arrows bound to them where they are drawn now.
    fn release(&mut self, gone: &BTreeSet<String>) {
        let now = self.shapes.clone();
        let hit = |e: &Option<String>| e.as_ref().is_some_and(|id| gone.contains(id));
        for s in self.shapes.iter_mut().filter(|s| s.kind == ShapeKind::Arrow && (hit(&s.start) || hit(&s.end))) {
            let (a, b) = arrow_ends(s, &now);
            s.points = vec![a, b];
            if hit(&s.start) {
                s.start = None;
            }
            if hit(&s.end) {
                s.end = None;
            }
            s.fit_arrow_box();
        }
    }

    fn center_world(&self) -> Point {
        self.viewport.to_world(Point::new(self.size.0 / 2.0, self.size.1 / 2.0))
    }

    /// Apply `f` to every selected shape, from `before`.
    fn each_selected(&mut self, before: &[Shape], mut f: impl FnMut(&mut Shape)) {
        for (s, b) in self.shapes.iter_mut().zip(before) {
            if self.selection.contains(&s.id) && s.id == b.id {
                *s = b.clone();
                f(s);
            }
        }
    }

    fn move_selection(&mut self, before: &[Shape], d: Point) {
        let sel = self.selection.clone();
        self.each_selected(before, |s| {
            if s.kind == ShapeKind::Arrow {
                // Start from where it is drawn; an arrow moved without its shapes lets go of them.
                let (a, b) = arrow_ends(s, before);
                s.points = vec![a, b];
                for end in [&mut s.start, &mut s.end] {
                    if end.as_ref().is_some_and(|id| !sel.contains(id)) {
                        *end = None;
                    }
                }
            }
            s.translate(d);
            if s.kind == ShapeKind::Arrow {
                s.fit_arrow_box();
            }
        });
    }

    /// Scale the selection inside `frame` so it becomes `w`×`h` centred on `center`.
    fn scale_selection(&mut self, before: &[Shape], frame: Frame, center: Point, w: f64, h: f64) {
        let (sx, sy) = (w / frame.w.max(1e-6), h / frame.h.max(1e-6));
        let map = |q: Point| {
            let l = q.rotate(frame.center, -frame.angle).minus(frame.center);
            center.plus(Point::new(l.x * sx, l.y * sy)).rotate(center, frame.angle)
        };
        self.each_selected(before, |s| {
            if s.kind == ShapeKind::Arrow {
                for p in &mut s.points {
                    *p = map(*p);
                }
                s.fit_arrow_box();
            } else {
                let c = map(s.center());
                s.w = (s.w * sx).max(1.0);
                s.h = (s.h * sy).max(1.0);
                s.set_center(c);
            }
        });
    }

    fn rotate_selection(&mut self, before: &[Shape], pivot: Point, angle: f64) {
        self.each_selected(before, |s| {
            if s.kind == ShapeKind::Arrow {
                for p in &mut s.points {
                    *p = p.rotate(pivot, angle);
                }
                s.fit_arrow_box();
            } else {
                let c = s.center().rotate(pivot, angle);
                s.rotation = (s.rotation + angle).rem_euclid(std::f64::consts::TAU);
                s.set_center(c);
            }
        });
    }

    fn handle_at(&self, p: Point) -> Option<Handle> {
        let tol = self.tol();
        if let Some(a) = self.lone_arrow() {
            let (s, e) = arrow_ends(a, &self.shapes);
            return [(Handle::Start, s), (Handle::End, e)].into_iter().find(|(_, q)| q.dist(p) <= tol).map(|(h, _)| h);
        }
        let f = self.frame()?;
        let z = self.viewport.zoom;
        std::iter::once(Handle::Rotate).chain(Handle::RESIZE).find(|h| f.handle(*h, z).dist(p) <= tol)
    }

    fn start_create(&mut self, kind: ShapeKind, world: Point) {
        let before = self.snapshot();
        let id = self.fresh_id();
        let mut shape = Shape::new(&id, kind, Rect::new(world.x, world.y, 0.0, 0.0)).color(self.color);
        if kind == ShapeKind::Arrow {
            shape.points = vec![world, world];
            shape.start = shape_at(&self.shapes, world, 0.0, true).map(|i| self.shapes[i].id.clone());
        }
        if kind == ShapeKind::Pen {
            shape.points = vec![Point::new(0.0, 0.0)];
        }
        self.shapes.push(shape);
        self.selection = BTreeSet::from([id.clone()]);
        self.gesture = Some(Gesture::Create { id, start: world, ink: vec![world], before });
    }

    fn pointer_down(&mut self, at: Point, shift: bool, pan: bool) {
        let world = self.viewport.to_world(at);
        if pan || self.tool == Tool::Hand {
            self.gesture = Some(Gesture::Pan { last: at });
            return;
        }
        if let Some(kind) = self.tool.kind() {
            self.start_create(kind, world);
            return;
        }
        if let Some(handle) = self.handle_at(world) {
            let before = self.snapshot();
            self.gesture = Some(match handle {
                Handle::Start | Handle::End => Gesture::Retarget { handle, before },
                Handle::Rotate => {
                    let center = self.frame().map_or(world, |f| f.center);
                    Gesture::Rotate { center, from: (world.y - center.y).atan2(world.x - center.x), before }
                }
                _ => Gesture::Resize { handle, frame: self.frame().unwrap_or(Frame { center: world, w: 1.0, h: 1.0, angle: 0.0 }), before },
            });
            return;
        }
        let Some(i) = shape_at(&self.shapes, world, self.tol(), false) else {
            let base = if shift { self.selection.clone() } else { BTreeSet::new() };
            self.selection.clone_from(&base);
            self.gesture = Some(Gesture::Marquee { start: world, current: world, base });
            return;
        };
        let id = self.shapes[i].id.clone();
        if shift {
            if !self.selection.remove(&id) {
                self.selection.insert(id.clone());
            }
        } else if !self.selection.contains(&id) {
            self.selection = BTreeSet::from([id.clone()]);
        }
        if self.selection.contains(&id) {
            self.gesture = Some(Gesture::Move { start: world, before: self.snapshot() });
        }
    }

    fn pointer_move(&mut self, at: Point, shift: bool) {
        let world = self.viewport.to_world(at);
        let Some(gesture) = self.gesture.clone() else { return };
        match gesture {
            Gesture::Pan { last } => {
                let d = at.minus(last);
                self.viewport = self.viewport.pan_by(d.x, d.y);
                self.gesture = Some(Gesture::Pan { last: at });
            }
            Gesture::Move { start, before } => self.move_selection(&before, world.minus(start)),
            Gesture::Resize { handle, frame, before } => {
                let (hx, hy) = handle.dir();
                let anchor = frame.center.plus(Point::new(-hx * frame.w / 2.0, -hy * frame.h / 2.0)).rotate(frame.center, frame.angle);
                let d = world.minus(anchor).rotate(Point::default(), -frame.angle);
                let mut w = if hx == 0.0 { frame.w } else { (d.x * hx).max(4.0) };
                let mut h = if hy == 0.0 { frame.h } else { (d.y * hy).max(4.0) };
                if shift && hx != 0.0 && hy != 0.0 {
                    let k = (w / frame.w.max(1e-6)).max(h / frame.h.max(1e-6));
                    (w, h) = (frame.w * k, frame.h * k);
                }
                let center = anchor.plus(Point::new(hx * w / 2.0, hy * h / 2.0).rotate(Point::default(), frame.angle));
                self.scale_selection(&before, frame, center, w, h);
            }
            Gesture::Rotate { center, from, before } => {
                let mut a = (world.y - center.y).atan2(world.x - center.x) - from;
                if shift {
                    let step = 15f64.to_radians();
                    a = (a / step).round() * step;
                }
                self.rotate_selection(&before, center, a);
            }
            Gesture::Retarget { handle, before } => {
                let target = shape_at(&before, world, 0.0, true).map(|i| before[i].id.clone()).filter(|id| !self.selection.contains(id));
                let i = usize::from(handle == Handle::End);
                if let Some(s) = self.shapes.iter_mut().find(|s| self.selection.contains(&s.id)) {
                    if let Some(p) = s.points.get_mut(i) {
                        *p = world;
                    }
                    if i == 0 { s.start.clone_from(&target) } else { s.end.clone_from(&target) }
                    s.fit_arrow_box();
                }
                self.bind_hint = target;
            }
            Gesture::Marquee { start, base, .. } => {
                let r = Rect::from_corners(start, world);
                let mut sel = base.clone();
                sel.extend(self.shapes.iter().filter(|s| r.contains_rect(bounds(s, &self.shapes))).map(|s| s.id.clone()));
                self.selection = sel;
                self.gesture = Some(Gesture::Marquee { start, current: world, base });
            }
            Gesture::Create { id, start, mut ink, before } => {
                let hint = shape_at(&before, world, 0.0, true).map(|i| before[i].id.clone());
                let Some(s) = self.shapes.iter_mut().find(|s| s.id == id) else { return };
                match s.kind {
                    ShapeKind::Arrow => {
                        s.points = vec![start, world];
                        s.end.clone_from(&hint);
                        s.fit_arrow_box();
                        self.bind_hint = hint;
                    }
                    ShapeKind::Pen => {
                        ink.push(world);
                        set_ink(s, &ink);
                    }
                    _ => {
                        let mut end = world;
                        if shift {
                            let side = (world.x - start.x).abs().max((world.y - start.y).abs());
                            end = Point::new(start.x + side.copysign(world.x - start.x), start.y + side.copysign(world.y - start.y));
                        }
                        let r = Rect::from_corners(start, end);
                        (s.x, s.y, s.w, s.h) = (r.x, r.y, r.w, r.h);
                    }
                }
                self.gesture = Some(Gesture::Create { id, start, ink, before });
            }
            Gesture::Pinch => {}
        }
    }

    fn finish(&mut self, out: &mut Vec<WhiteboardCommand>) {
        self.bind_hint = None;
        match self.gesture.take() {
            Some(Gesture::Move { before, .. } | Gesture::Resize { before, .. } | Gesture::Rotate { before, .. } | Gesture::Retarget { before, .. }) => {
                self.commit(before, out);
            }
            Some(Gesture::Create { id, start, ink, before }) => {
                let zoom = self.viewport.zoom;
                let Some(s) = self.shapes.iter_mut().find(|s| s.id == id) else { return };
                let tiny = 4.0 / zoom;
                let kind = s.kind;
                match kind {
                    ShapeKind::Pen => {
                        let pts = simplify(&ink, 0.8 / zoom);
                        set_ink(s, &pts);
                    }
                    ShapeKind::Arrow if s.points[0].dist(s.points[1]) < tiny => {
                        s.points[1] = start.plus(Point::new(kind.default_size().0, 0.0));
                        s.end = None;
                        s.fit_arrow_box();
                    }
                    _ if kind != ShapeKind::Arrow && s.w < tiny && s.h < tiny => {
                        let (w, h) = kind.default_size();
                        (s.x, s.y, s.w, s.h) = (start.x - w / 2.0, start.y - h / 2.0, w, h);
                    }
                    _ => {}
                }
                self.tool = Tool::Select;
                if kind.has_text() {
                    self.editing = Some((id, before));
                } else {
                    self.commit(before, out);
                    out.push(WhiteboardCommand::Announce(format!("{} added.", kind.label())));
                }
            }
            _ => {}
        }
    }

    fn cancel(&mut self) {
        self.bind_hint = None;
        if let Some(
            Gesture::Move { before, .. }
            | Gesture::Resize { before, .. }
            | Gesture::Rotate { before, .. }
            | Gesture::Retarget { before, .. }
            | Gesture::Create { before, .. },
        ) = self.gesture.take()
        {
            self.shapes.clone_from(&before);
            self.prune();
        }
    }

    fn end_edit(&mut self, out: &mut Vec<WhiteboardCommand>) {
        let Some((id, before)) = self.editing.take() else { return };
        if self.shapes.iter().any(|s| s.id == id && s.kind == ShapeKind::Text && s.text.trim().is_empty()) {
            let gone = BTreeSet::from([id.clone()]);
            self.release(&gone);
            self.shapes.retain(|s| s.id != id);
            self.prune();
        }
        let created = !before.iter().any(|s| s.id == id);
        let alive = self.shapes.iter().any(|s| s.id == id);
        self.commit(before, out);
        if created && alive {
            let label = self.shapes.iter().find(|s| s.id == id).map_or("Shape", |s| s.kind.label());
            out.push(WhiteboardCommand::Announce(format!("{label} added.")));
        }
        out.push(WhiteboardCommand::FocusCanvas);
    }

    fn reorder(&mut self, order: Order) {
        let sel = &self.selection;
        match order {
            Order::Front | Order::Back => {
                let (mut picked, rest): (Vec<Shape>, Vec<Shape>) = std::mem::take(&mut self.shapes).into_iter().partition(|s| sel.contains(&s.id));
                if order == Order::Front {
                    self.shapes = rest;
                    self.shapes.append(&mut picked);
                } else {
                    self.shapes = picked;
                    self.shapes.extend(rest);
                }
            }
            Order::Forward => {
                for i in (0..self.shapes.len().saturating_sub(1)).rev() {
                    if sel.contains(&self.shapes[i].id) && !sel.contains(&self.shapes[i + 1].id) {
                        self.shapes.swap(i, i + 1);
                    }
                }
            }
            Order::Backward => {
                for i in 1..self.shapes.len() {
                    if sel.contains(&self.shapes[i].id) && !sel.contains(&self.shapes[i - 1].id) {
                        self.shapes.swap(i, i - 1);
                    }
                }
            }
        }
    }
}

/// Store world-unit `ink` as a pen shape's box and fractional points.
fn set_ink(s: &mut Shape, ink: &[Point]) {
    let Some(r) = Rect::around(ink.iter().copied()) else { return };
    let (w, h) = (r.w.max(1.0), r.h.max(1.0));
    (s.x, s.y, s.w, s.h) = (r.x, r.y, w, h);
    s.points = ink.iter().map(|p| Point::new((p.x - r.x) / w, (p.y - r.y) / h)).collect();
}

/// Line height for text in a shape of this kind.
#[must_use]
pub fn line_height(kind: ShapeKind) -> f64 {
    if kind == ShapeKind::Text { TEXT_FONT * 1.3 } else { STICKY_FONT * 1.35 }
}

/// A shape's text broken into lines that fit its width. Glyph width is estimated.
#[must_use]
pub fn text_lines(s: &Shape) -> Vec<String> {
    let (font, pad) = if s.kind == ShapeKind::Text { (TEXT_FONT, 4.0) } else { (STICKY_FONT, STICKY_PAD) };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a small positive count")]
    let max = (((s.w - pad * 2.0) / (font * 0.55)).floor().max(1.0)) as usize;
    let mut lines = Vec::new();
    for para in s.text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let fits = line.chars().count() + word.chars().count() + usize::from(!line.is_empty()) <= max;
            if !line.is_empty() && !fits {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// SVG markup for one shape, in world units. Colours are token variables with fallbacks.
#[must_use]
pub fn shape_svg(s: &Shape, shapes: &[Shape]) -> String {
    let colour = swatch(s.color).css();
    let tint = |pct: u8, base: &str| format!("color-mix(in srgb, {colour} {pct}%, {base})");
    let rot = if s.rotation == 0.0 {
        String::new()
    } else {
        let c = s.center();
        format!(" transform=\"rotate({:.3} {:.2} {:.2})\"", s.rotation.to_degrees(), c.x, c.y)
    };
    let text = |fill: &str, font: f64, pad: f64| {
        let lh = line_height(s.kind);
        let mut t = format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" style=\"fill: {fill}; font: {font}px var(--font-body, system-ui, sans-serif);\">",
            s.x + pad,
            s.y + pad + font
        );
        for (i, line) in text_lines(s).iter().enumerate() {
            let dy = if i == 0 { 0.0 } else { lh };
            let _ = write!(t, "<tspan x=\"{:.2}\" dy=\"{dy:.2}\">{}</tspan>", s.x + pad, xml_escape(line));
        }
        t + "</text>"
    };
    let stroke = format!("stroke: {colour}; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round;");
    match s.kind {
        ShapeKind::Rect => format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"6\" style=\"{stroke} fill: {};\"{rot}/>",
            s.x,
            s.y,
            s.w,
            s.h,
            tint(12, "transparent")
        ),
        ShapeKind::Ellipse => {
            let c = s.center();
            format!(
                "<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" style=\"{stroke} fill: {};\"{rot}/>",
                c.x,
                c.y,
                s.w / 2.0,
                s.h / 2.0,
                tint(12, "transparent")
            )
        }
        ShapeKind::Arrow => {
            let (a, b) = arrow_ends(s, shapes);
            let ang = (b.y - a.y).atan2(b.x - a.x);
            let len = 14.0_f64.min(a.dist(b) * 0.5);
            let h1 = b.plus(Point::new(-len * (ang - 0.45).cos(), -len * (ang - 0.45).sin()));
            let h2 = b.plus(Point::new(-len * (ang + 0.45).cos(), -len * (ang + 0.45).sin()));
            format!(
                "<path d=\"M{:.2} {:.2}L{:.2} {:.2}M{:.2} {:.2}L{:.2} {:.2}L{:.2} {:.2}\" style=\"{stroke} fill: none;\"/>",
                a.x, a.y, b.x, b.y, h1.x, h1.y, b.x, b.y, h2.x, h2.y
            )
        }
        ShapeKind::Pen => format!(
            "<path d=\"{}\" style=\"stroke: {colour}; stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; fill: none;\"{rot}/>",
            smooth_path(&s.ink())
        ),
        ShapeKind::Sticky => format!(
            "<g{rot}><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"4\" style=\"fill: {}; stroke: {};\"/>{}</g>",
            s.x,
            s.y,
            s.w,
            s.h,
            tint(26, "var(--surface-card, #ffffff)"),
            tint(40, "transparent"),
            text("var(--text-1, #161e35)", STICKY_FONT, STICKY_PAD)
        ),
        ShapeKind::Text => format!("<g{rot}>{}</g>", text(&colour, TEXT_FONT, 4.0)),
    }
}

/// The whole scene as a standalone SVG document.
#[must_use]
pub fn export_svg(shapes: &[Shape]) -> String {
    let r = shapes.iter().map(|s| bounds(s, shapes)).reduce(Rect::union).unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0)).inflate(24.0);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {:.2} {:.2}\" width=\"{:.0}\" height=\"{:.0}\">",
        r.x, r.y, r.w, r.h, r.w, r.h
    );
    for s in shapes {
        out.push_str(&shape_svg(s, shapes));
    }
    out + "</svg>"
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out + "\""
}

/// The scene as JSON: `{"version":1,"shapes":[…]}`.
#[must_use]
pub fn export_json(shapes: &[Shape]) -> String {
    let opt = |o: &Option<String>| o.as_deref().map_or_else(|| "null".to_string(), json_str);
    let items: Vec<String> = shapes
        .iter()
        .map(|s| {
            let points: Vec<String> = s.points.iter().map(|p| format!("[{:.3},{:.3}]", p.x, p.y)).collect();
            format!(
                "{{\"id\":{},\"type\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"h\":{:.2},\"rotation\":{:.4},\"color\":\"{}\",\"points\":[{}],\"start\":{},\"end\":{},\"text\":{}}}",
                json_str(&s.id),
                s.kind.as_str(),
                s.x,
                s.y,
                s.w,
                s.h,
                s.rotation,
                swatch(s.color).name.to_lowercase(),
                points.join(","),
                opt(&s.start),
                opt(&s.end),
                json_str(&s.text)
            )
        })
        .collect();
    format!("{{\"version\":1,\"shapes\":[{}]}}", items.join(","))
}

impl Machine for WhiteboardMachine {
    type Input = WhiteboardInput;
    type State = WhiteboardState;
    type Event = WhiteboardEvent;
    type Command = WhiteboardCommand;

    fn initial(input: &WhiteboardInput) -> WhiteboardState {
        WhiteboardState { shapes: input.default_shapes.clone(), color: 1, ..WhiteboardState::default() }
    }

    #[allow(clippy::too_many_lines, reason = "one match over every event")]
    fn transition(state: &WhiteboardState, event: WhiteboardEvent, _input: &WhiteboardInput) -> Step {
        let mut s = state.clone();
        let mut out = Vec::new();
        let editing = s.editing.is_some();
        match event {
            WhiteboardEvent::Resize(w, h) => s.size = (w, h),
            WhiteboardEvent::PointerDown { id, at, shift, pan } => {
                s.end_edit(&mut out);
                if s.pointers.down(id, at) >= 2 {
                    s.cancel();
                    s.gesture = Some(Gesture::Pinch);
                } else {
                    s.pointer_down(at, shift, pan);
                }
            }
            WhiteboardEvent::PointerMove { id, at, shift } => {
                if s.gesture == Some(Gesture::Pinch) {
                    if let Some(v) = s.pointers.pinch(id, at, s.viewport) {
                        s.viewport = v;
                    }
                } else if s.pointers.len() == 1 && s.gesture.is_some() {
                    s.pointers.down(id, at);
                    s.pointer_move(at, shift);
                } else {
                    return Transition::to(s);
                }
            }
            WhiteboardEvent::PointerUp { id } => {
                let left = s.pointers.up(id);
                if s.gesture == Some(Gesture::Pinch) {
                    if left == 0 {
                        s.gesture = None;
                    }
                } else {
                    s.finish(&mut out);
                }
            }
            WhiteboardEvent::PointerCancel(id) => {
                s.pointers.up(id);
                s.cancel();
            }
            WhiteboardEvent::DoubleClick(at) => {
                let world = s.viewport.to_world(at);
                match shape_at(&s.shapes, world, s.tol(), false) {
                    Some(i) if s.shapes[i].kind.has_text() => {
                        let id = s.shapes[i].id.clone();
                        s.selection = BTreeSet::from([id.clone()]);
                        s.editing = Some((id, s.snapshot()));
                    }
                    Some(_) => {}
                    None => {
                        s.start_create(ShapeKind::Text, world);
                        s.gesture = None;
                        let id = s.selection.first().cloned().unwrap_or_default();
                        let before = Arc::new(state.shapes.clone());
                        if let Some(t) = s.shapes.iter_mut().find(|t| t.id == id) {
                            let (w, h) = ShapeKind::Text.default_size();
                            (t.w, t.h) = (w, h);
                            t.y -= h / 2.0;
                        }
                        s.editing = Some((id, before));
                    }
                }
            }
            WhiteboardEvent::Zoom { at, factor } => s.viewport = s.viewport.zoom_at(at, factor),
            WhiteboardEvent::ZoomIn => s.viewport = s.viewport.zoom_at(Point::new(s.size.0 / 2.0, s.size.1 / 2.0), 1.25),
            WhiteboardEvent::ZoomOut => s.viewport = s.viewport.zoom_at(Point::new(s.size.0 / 2.0, s.size.1 / 2.0), 0.8),
            WhiteboardEvent::ZoomReset => s.viewport = s.viewport.zoom_at(Point::new(s.size.0 / 2.0, s.size.1 / 2.0), 1.0 / s.viewport.zoom),
            WhiteboardEvent::Fit => {
                if let Some(r) = s.shapes.iter().map(|x| bounds(x, &s.shapes)).reduce(Rect::union) {
                    s.viewport = Viewport::fit(r, s.size, 48.0);
                }
            }
            WhiteboardEvent::Pan(dx, dy) => s.viewport = s.viewport.pan_by(dx, dy),
            WhiteboardEvent::Tool(t) => {
                s.end_edit(&mut out);
                s.tool = t;
                if t != Tool::Select {
                    s.selection.clear();
                }
                out.push(WhiteboardCommand::Announce(format!("{} tool.", t.label())));
            }
            WhiteboardEvent::Color(c) => {
                s.color = c.min(PALETTE.len() - 1);
                if !s.selection.is_empty() {
                    let before = s.snapshot();
                    let sel = s.selection.clone();
                    for x in s.shapes.iter_mut().filter(|x| sel.contains(&x.id)) {
                        x.color = s.color;
                    }
                    s.commit(before, &mut out);
                }
            }
            WhiteboardEvent::Activate if editing => {}
            WhiteboardEvent::Activate => {
                if let Some(kind) = s.tool.kind().filter(|k| *k != ShapeKind::Pen) {
                    let c = s.center_world();
                    s.start_create(kind, c);
                    s.finish(&mut out);
                } else {
                    let note = s.selected().next().filter(|x| x.kind.has_text() && s.selection.len() == 1).map(|x| x.id.clone());
                    if let Some(id) = note {
                        s.editing = Some((id, s.snapshot()));
                    }
                }
            }
            WhiteboardEvent::Nudge { dx, dy } => {
                if s.selection.is_empty() {
                    s.viewport = s.viewport.pan_by(-dx * 4.0, -dy * 4.0);
                } else {
                    let before = s.snapshot();
                    s.move_selection(&before, Point::new(dx, dy));
                    s.commit(before, &mut out);
                }
            }
            WhiteboardEvent::Grow { dw, dh } => {
                if let Some(f) = s.frame() {
                    let before = s.snapshot();
                    let (w, h) = ((f.w + dw).max(4.0), (f.h + dh).max(4.0));
                    let center = f.center.plus(Point::new((w - f.w) / 2.0, (h - f.h) / 2.0).rotate(Point::default(), f.angle));
                    s.scale_selection(&before, f, center, w, h);
                    s.commit(before, &mut out);
                }
            }
            WhiteboardEvent::Rotate(deg) => {
                if let Some(f) = s.frame() {
                    let before = s.snapshot();
                    s.rotate_selection(&before, f.center, deg.to_radians());
                    s.commit(before, &mut out);
                }
            }
            WhiteboardEvent::Order(o) => {
                let before = s.snapshot();
                s.reorder(o);
                s.commit(before, &mut out);
            }
            WhiteboardEvent::Step(dir) => {
                let n = s.shapes.len();
                if n > 0 {
                    let current = s.shapes.iter().rposition(|x| s.selection.contains(&x.id));
                    let i = match (current, dir >= 0) {
                        (None, true) => 0,
                        (None, false) => n - 1,
                        (Some(i), true) => (i + 1) % n,
                        (Some(i), false) => (i + n - 1) % n,
                    };
                    let shape = &s.shapes[i];
                    let text = format!("{}, {} of {n}.", describe(shape), i + 1);
                    let r = bounds(shape, &s.shapes);
                    s.selection = BTreeSet::from([shape.id.clone()]);
                    if !s.viewport.visible(s.size).contains_rect(r) {
                        s.viewport = s.viewport.center_on(r.center(), s.size);
                    }
                    out.push(WhiteboardCommand::Announce(text));
                }
            }
            WhiteboardEvent::Delete => {
                if !s.selection.is_empty() {
                    let before = s.snapshot();
                    let n = s.selection.len();
                    let sel = std::mem::take(&mut s.selection);
                    s.release(&sel);
                    s.shapes.retain(|x| !sel.contains(&x.id));
                    s.prune();
                    s.commit(before, &mut out);
                    out.push(WhiteboardCommand::Announce(format!("Deleted {}.", plural(n))));
                }
            }
            WhiteboardEvent::Duplicate => {
                let before = s.snapshot();
                let picked: Vec<Shape> = s.selected().cloned().collect();
                let mut renamed: Vec<(String, String)> = Vec::new();
                for x in &picked {
                    let id = s.fresh_id();
                    renamed.push((x.id.clone(), id));
                }
                let rename = |id: &Option<String>| id.as_ref().and_then(|id| renamed.iter().find(|r| r.0 == *id).map(|r| r.1.clone()));
                let copies: Vec<Shape> = picked
                    .iter()
                    .zip(&renamed)
                    .map(|(x, (_, id))| {
                        let mut c = x.clone();
                        c.id.clone_from(id);
                        c.translate(Point::new(16.0, 16.0));
                        if c.kind == ShapeKind::Arrow {
                            (c.start, c.end) = (rename(&x.start), rename(&x.end));
                        }
                        c
                    })
                    .collect();
                s.selection = copies.iter().map(|c| c.id.clone()).collect();
                s.shapes.extend(copies);
                s.commit(before, &mut out);
                if !picked.is_empty() {
                    out.push(WhiteboardCommand::Announce(format!("Duplicated {}.", plural(picked.len()))));
                }
            }
            WhiteboardEvent::SelectAll => {
                s.selection = s.shapes.iter().map(|x| x.id.clone()).collect();
                out.push(WhiteboardCommand::Announce(format!("Selected {}.", plural(s.selection.len()))));
            }
            ev @ (WhiteboardEvent::Undo | WhiteboardEvent::Redo) => {
                s.end_edit(&mut out);
                let undo = ev == WhiteboardEvent::Undo;
                let moved = if undo { s.history.undo(&mut s.shapes) } else { s.history.redo(&mut s.shapes) };
                if moved {
                    s.gesture = None;
                    s.prune();
                    out.push(WhiteboardCommand::Change(s.shapes.clone()));
                    out.push(WhiteboardCommand::Announce(if undo { "Undone." } else { "Redone." }.into()));
                }
            }
            WhiteboardEvent::Cancel => {
                if editing {
                    s.end_edit(&mut out);
                } else if s.gesture.is_some() {
                    s.cancel();
                } else if !s.selection.is_empty() {
                    s.selection.clear();
                } else if s.tool != Tool::Select {
                    s.tool = Tool::Select;
                    out.push(WhiteboardCommand::Announce("Select tool.".into()));
                }
            }
            WhiteboardEvent::EditText(text) => {
                if let Some((id, _)) = &s.editing {
                    let id = id.clone();
                    if let Some(x) = s.shapes.iter_mut().find(|x| x.id == id) {
                        x.text = text;
                        if x.kind == ShapeKind::Text {
                            #[allow(clippy::cast_precision_loss, reason = "line counts are small")]
                            let lines = text_lines(x).len().max(1) as f64;
                            x.h = lines * line_height(x.kind) + 8.0;
                        }
                    }
                }
            }
            WhiteboardEvent::EditEnd => s.end_edit(&mut out),
        }
        Transition::with(s, out)
    }
}

/// The event for a key on the canvas, if any. `Some` means: prevent default.
/// `cmd` is Ctrl or Cmd. Not for keys typed into the text editor.
#[must_use]
pub fn key_event(key: &str, shift: bool, cmd: bool, alt: bool) -> Option<WhiteboardEvent> {
    let step = if shift { 10.0 } else { 1.0 };
    let arrow = |dx: f64, dy: f64| {
        Some(if alt { WhiteboardEvent::Grow { dw: dx * step, dh: dy * step } } else { WhiteboardEvent::Nudge { dx: dx * step, dy: dy * step } })
    };
    if cmd {
        return match key.to_ascii_lowercase().as_str() {
            "z" if shift => Some(WhiteboardEvent::Redo),
            "z" => Some(WhiteboardEvent::Undo),
            "y" => Some(WhiteboardEvent::Redo),
            "a" => Some(WhiteboardEvent::SelectAll),
            "d" => Some(WhiteboardEvent::Duplicate),
            "]" | "}" if shift => Some(WhiteboardEvent::Order(Order::Front)),
            "]" | "}" => Some(WhiteboardEvent::Order(Order::Forward)),
            "[" | "{" if shift => Some(WhiteboardEvent::Order(Order::Back)),
            "[" | "{" => Some(WhiteboardEvent::Order(Order::Backward)),
            _ => None,
        };
    }
    match key {
        "ArrowLeft" => arrow(-1.0, 0.0),
        "ArrowRight" => arrow(1.0, 0.0),
        "ArrowUp" => arrow(0.0, -1.0),
        "ArrowDown" => arrow(0.0, 1.0),
        "Delete" | "Backspace" => Some(WhiteboardEvent::Delete),
        "Escape" => Some(WhiteboardEvent::Cancel),
        "Enter" => Some(WhiteboardEvent::Activate),
        "]" => Some(WhiteboardEvent::Step(1)),
        "[" => Some(WhiteboardEvent::Step(-1)),
        "," | "<" => Some(WhiteboardEvent::Rotate(-15.0)),
        "." | ">" => Some(WhiteboardEvent::Rotate(15.0)),
        "+" | "=" => Some(WhiteboardEvent::ZoomIn),
        "-" | "_" => Some(WhiteboardEvent::ZoomOut),
        "0" => Some(WhiteboardEvent::ZoomReset),
        "1" => Some(WhiteboardEvent::Fit),
        _ if !alt => Tool::ALL.iter().find(|t| t.key().eq_ignore_ascii_case(key)).map(|t| WhiteboardEvent::Tool(*t)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::close;

    fn board(shapes: Vec<Shape>) -> WhiteboardState {
        let mut s = WhiteboardMachine::initial(&WhiteboardInput { default_shapes: shapes });
        s.size = (800.0, 600.0);
        s
    }

    fn run(s: &WhiteboardState, events: Vec<WhiteboardEvent>) -> (WhiteboardState, Vec<WhiteboardCommand>) {
        let input = WhiteboardInput::default();
        let mut s = s.clone();
        let mut all = Vec::new();
        for e in events {
            let t = WhiteboardMachine::transition(&s, e, &input).resolve(&s);
            s = t.state;
            all.extend(t.commands);
        }
        (s, all)
    }

    fn down(x: f64, y: f64) -> WhiteboardEvent {
        WhiteboardEvent::PointerDown { id: 1, at: Point::new(x, y), shift: false, pan: false }
    }
    fn mv(x: f64, y: f64) -> WhiteboardEvent {
        WhiteboardEvent::PointerMove { id: 1, at: Point::new(x, y), shift: false }
    }
    fn up() -> WhiteboardEvent {
        WhiteboardEvent::PointerUp { id: 1 }
    }

    fn rect(id: &str, x: f64, y: f64) -> Shape {
        Shape::new(id, ShapeKind::Rect, Rect::new(x, y, 100.0, 60.0))
    }

    #[test]
    fn drawing_a_rectangle_then_undo() {
        let s = board(vec![]);
        let (s, cmds) = run(&s, vec![WhiteboardEvent::Tool(Tool::Rect), down(10.0, 20.0), mv(110.0, 80.0), up()]);
        assert_eq!(s.shapes.len(), 1);
        assert_eq!(s.shapes[0].rect(), Rect::new(10.0, 20.0, 100.0, 60.0));
        assert_eq!(s.tool, Tool::Select);
        assert!(cmds.iter().any(|c| matches!(c, WhiteboardCommand::Change(_))));
        let (s, _) = run(&s, vec![WhiteboardEvent::Undo]);
        assert!(s.shapes.is_empty() && s.selection.is_empty());
    }

    #[test]
    fn click_with_a_tool_makes_a_default_shape_and_notes_open_for_editing() {
        let s = board(vec![]);
        let (s, cmds) = run(&s, vec![WhiteboardEvent::Tool(Tool::Sticky), down(300.0, 300.0), up()]);
        assert!(close(s.shapes[0].w, 200.0));
        assert!(s.editing().is_some());
        assert!(!cmds.iter().any(|c| matches!(c, WhiteboardCommand::Change(_))));
        let (s, cmds) = run(&s, vec![WhiteboardEvent::EditText("Ship it".into()), WhiteboardEvent::EditEnd]);
        assert_eq!(s.shapes[0].text, "Ship it");
        assert!(cmds.contains(&WhiteboardCommand::FocusCanvas));
        assert_eq!(s.history.undo_len(), 1);
    }

    #[test]
    fn select_move_and_marquee() {
        let s = board(vec![rect("a", 0.0, 0.0), rect("b", 300.0, 0.0)]);
        let (m, _) = run(&s, vec![down(50.0, 30.0), mv(70.0, 40.0), up()]);
        assert_eq!(m.shapes[0].rect(), Rect::new(20.0, 10.0, 100.0, 60.0));
        let (box_sel, _) = run(&s, vec![down(-10.0, -10.0), mv(500.0, 100.0), up()]);
        assert_eq!(box_sel.selection.len(), 2);
        let (none, _) = run(&s, vec![down(-10.0, -10.0), mv(150.0, 100.0), up()]);
        assert_eq!(none.selection, BTreeSet::from(["a".to_string()]));
    }

    #[test]
    fn bound_arrows_follow_their_shapes() {
        let arrow = Shape::arrow("x", Point::new(50.0, 30.0), Point::new(350.0, 30.0)).bind(Some("a"), Some("b"));
        let s = board(vec![rect("a", 0.0, 0.0), rect("b", 300.0, 0.0), arrow]);
        let (a, b) = arrow_ends(&s.shapes[2], &s.shapes);
        assert!(close(a.x, 100.0 + BIND_GAP) && close(b.x, 300.0 - BIND_GAP));
        let (moved, _) = run(&s, vec![down(350.0, 30.0), mv(350.0, 230.0), up()]);
        let (_, b2) = arrow_ends(&moved.shapes[2], &moved.shapes);
        assert!(b2.y > 150.0);
    }

    #[test]
    fn arrow_tool_binds_both_ends() {
        let s = board(vec![rect("a", 0.0, 0.0), rect("b", 300.0, 0.0)]);
        let (s, _) = run(&s, vec![WhiteboardEvent::Tool(Tool::Arrow), down(50.0, 30.0), mv(340.0, 30.0), up()]);
        let arrow = s.shapes.last().unwrap();
        assert_eq!((arrow.start.as_deref(), arrow.end.as_deref()), (Some("a"), Some("b")));
        // Deleting a bound shape leaves the arrow where it was.
        let (s, _) = run(&s, vec![down(50.0, 30.0), up(), WhiteboardEvent::Delete]);
        let arrow = s.shapes.last().unwrap();
        assert_eq!(arrow.start, None);
        assert!(close(arrow.points[0].x, 100.0 + BIND_GAP));
    }

    #[test]
    fn resize_from_a_corner_and_rotate() {
        let s = board(vec![rect("a", 0.0, 0.0)]);
        let (s, _) = run(&s, vec![down(50.0, 30.0), up()]);
        let (r, _) = run(&s, vec![down(100.0, 60.0), mv(150.0, 100.0), up()]);
        assert_eq!(r.shapes[0].rect(), Rect::new(0.0, 0.0, 150.0, 100.0));
        let (rot, _) = run(&s, vec![WhiteboardEvent::Rotate(90.0)]);
        assert!(close(rot.shapes[0].rotation, std::f64::consts::FRAC_PI_2));
        // Rotated 90°, the box is 60 wide on screen.
        assert!(close(bounds(&rot.shapes[0], &rot.shapes).w, 60.0));
        assert!(contains(&rot.shapes[0], Point::new(50.0, 75.0), 0.0, &rot.shapes));
    }

    #[test]
    fn pen_strokes_are_simplified_and_scale_with_their_box() {
        let s = board(vec![]);
        let mut events = vec![WhiteboardEvent::Tool(Tool::Pen), down(0.0, 0.0)];
        events.extend((1..=20).map(|i| mv(f64::from(i) * 5.0, 0.0)));
        events.push(mv(100.0, 100.0));
        events.push(up());
        let (s, _) = run(&s, events);
        let pen = &s.shapes[0];
        assert_eq!(pen.points.len(), 3);
        assert!(close(pen.w, 100.0));
        assert!(smooth_path(&pen.ink()).starts_with("M0.0 0.0Q"));
    }

    #[test]
    fn z_order_duplicate_and_delete() {
        let s = board(vec![rect("a", 0.0, 0.0), rect("b", 0.0, 0.0), rect("c", 0.0, 0.0)]);
        let (s, _) = run(&s, vec![WhiteboardEvent::Step(1)]);
        assert_eq!(s.selection, BTreeSet::from(["a".to_string()]));
        let (f, _) = run(&s, vec![WhiteboardEvent::Order(Order::Front)]);
        assert_eq!(f.shapes.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["b", "c", "a"]);
        let (b, _) = run(&f, vec![WhiteboardEvent::Order(Order::Backward)]);
        assert_eq!(b.shapes.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["b", "a", "c"]);
        let (d, _) = run(&b, vec![WhiteboardEvent::Duplicate]);
        assert_eq!(d.shapes.len(), 4);
        assert!(close(d.shapes[3].x, 16.0));
        let (gone, _) = run(&d, vec![WhiteboardEvent::SelectAll, WhiteboardEvent::Delete]);
        assert!(gone.shapes.is_empty());
    }

    #[test]
    fn hit_testing_respects_shape_outlines() {
        let mut e = Shape::new("e", ShapeKind::Ellipse, Rect::new(0.0, 0.0, 100.0, 50.0));
        assert!(contains(&e, Point::new(50.0, 25.0), 0.0, &[]));
        assert!(!contains(&e, Point::new(2.0, 2.0), 0.0, &[]));
        e.kind = ShapeKind::Rect;
        assert!(contains(&e, Point::new(2.0, 2.0), 0.0, &[]));
    }

    #[test]
    fn exports_json_and_svg() {
        let shapes = vec![rect("a", 0.0, 0.0).text("say \"hi\"\n"), Shape::new("n", ShapeKind::Sticky, Rect::new(0.0, 100.0, 200.0, 100.0)).text("a < b")];
        let json = export_json(&shapes);
        assert!(json.starts_with("{\"version\":1,\"shapes\":[{\"id\":\"a\",\"type\":\"rect\""));
        assert!(json.contains("\"text\":\"say \\\"hi\\\"\\n\""));
        let svg = export_svg(&shapes);
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"-24.00 -24.00 248.00 248.00\""));
        assert!(svg.contains("a &lt; b") && svg.contains("var(--text-1, #161e35)"));
    }

    #[test]
    fn text_wraps_to_the_box() {
        let s = Shape::new("n", ShapeKind::Sticky, Rect::new(0.0, 0.0, 120.0, 100.0)).text("one two three four five");
        assert_eq!(text_lines(&s), ["one two", "three four", "five"]);
    }

    #[test]
    fn keys() {
        assert_eq!(key_event("r", false, false, false), Some(WhiteboardEvent::Tool(Tool::Rect)));
        assert_eq!(key_event("Z", true, true, false), Some(WhiteboardEvent::Redo));
        assert_eq!(key_event("ArrowLeft", true, false, true), Some(WhiteboardEvent::Grow { dw: -10.0, dh: 0.0 }));
        assert_eq!(key_event("]", true, true, false), Some(WhiteboardEvent::Order(Order::Front)));
        assert_eq!(key_event("q", false, false, false), None);
    }
}
