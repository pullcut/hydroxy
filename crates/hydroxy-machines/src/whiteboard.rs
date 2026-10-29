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

use crate::canvas::Pointers;
pub use crate::canvas::{Point, Rect, Viewport};

mod export;
mod gesture;
mod path;
mod shape;
#[cfg(test)]
mod tests;

pub use export::*;
pub use gesture::*;
pub use path::*;
pub use shape::*;

/// Screen pixels within which a handle or thin stroke counts as hit.
const TOLERANCE: f64 = 8.0;

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
    pub shapes: Arc<Vec<Shape>>,
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
    crate::text::plural(n, "shape")
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
        Arc::clone(&self.shapes)
    }

    fn commit(&mut self, before: Arc<Vec<Shape>>, out: &mut Vec<WhiteboardCommand>) {
        if Arc::ptr_eq(&before, &self.shapes) || before == self.shapes {
            return;
        }
        self.history.record(before);
        out.push(WhiteboardCommand::Change(self.shapes.to_vec()));
    }

    /// Drop selected ids and arrow bindings that point at missing shapes.
    fn prune(&mut self) {
        let ids: BTreeSet<String> = self.shapes.iter().map(|s| s.id.clone()).collect();
        self.selection.retain(|id| ids.contains(id));
        for s in Arc::make_mut(&mut self.shapes) {
            for end in [&mut s.start, &mut s.end] {
                if end.as_ref().is_some_and(|id| !ids.contains(id)) {
                    *end = None;
                }
            }
        }
    }

    /// Before shapes in `gone` are removed, pin arrows bound to them where they are drawn now.
    fn release(&mut self, gone: &BTreeSet<String>) {
        let now = Arc::clone(&self.shapes);
        let hit = |e: &Option<String>| e.as_ref().is_some_and(|id| gone.contains(id));
        for s in Arc::make_mut(&mut self.shapes)
            .iter_mut()
            .filter(|s| s.kind == ShapeKind::Arrow && (hit(&s.start) || hit(&s.end)))
        {
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
        for (s, b) in Arc::make_mut(&mut self.shapes).iter_mut().zip(before) {
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
        Arc::make_mut(&mut self.shapes).push(shape);
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
                _ => Gesture::Resize {
                    handle,
                    frame: self.frame().unwrap_or(Frame { center: world, w: 1.0, h: 1.0, angle: 0.0 }),
                    before,
                },
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
                let anchor = frame
                    .center
                    .plus(Point::new(-hx * frame.w / 2.0, -hy * frame.h / 2.0))
                    .rotate(frame.center, frame.angle);
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
                let target = shape_at(&before, world, 0.0, true)
                    .map(|i| before[i].id.clone())
                    .filter(|id| !self.selection.contains(id));
                let i = usize::from(handle == Handle::End);
                if let Some(s) = Arc::make_mut(&mut self.shapes).iter_mut().find(|s| self.selection.contains(&s.id)) {
                    if let Some(p) = s.points.get_mut(i) {
                        *p = world;
                    }
                    if i == 0 {
                        s.start.clone_from(&target);
                    } else {
                        s.end.clone_from(&target);
                    }
                    s.fit_arrow_box();
                }
                self.bind_hint = target;
            }
            Gesture::Marquee { start, base, .. } => {
                let r = Rect::from_corners(start, world);
                let mut sel = base.clone();
                sel.extend(
                    self.shapes.iter().filter(|s| r.contains_rect(bounds(s, &self.shapes))).map(|s| s.id.clone()),
                );
                self.selection = sel;
                self.gesture = Some(Gesture::Marquee { start, current: world, base });
            }
            Gesture::Create { id, start, mut ink, before } => {
                let hint = shape_at(&before, world, 0.0, true).map(|i| before[i].id.clone());
                let Some(s) = Arc::make_mut(&mut self.shapes).iter_mut().find(|s| s.id == id) else { return };
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
                            end = Point::new(
                                start.x + side.copysign(world.x - start.x),
                                start.y + side.copysign(world.y - start.y),
                            );
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
            Some(
                Gesture::Move { before, .. }
                | Gesture::Resize { before, .. }
                | Gesture::Rotate { before, .. }
                | Gesture::Retarget { before, .. },
            ) => {
                self.commit(before, out);
            }
            Some(Gesture::Create { id, start, ink, before }) => {
                let zoom = self.viewport.zoom;
                let Some(s) = Arc::make_mut(&mut self.shapes).iter_mut().find(|s| s.id == id) else { return };
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
            self.shapes = before;
            self.prune();
        }
    }

    fn end_edit(&mut self, out: &mut Vec<WhiteboardCommand>) {
        let Some((id, before)) = self.editing.take() else { return };
        if self.shapes.iter().any(|s| s.id == id && s.kind == ShapeKind::Text && s.text.trim().is_empty()) {
            let gone = BTreeSet::from([id.clone()]);
            self.release(&gone);
            Arc::make_mut(&mut self.shapes).retain(|s| s.id != id);
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
        let shapes = Arc::make_mut(&mut self.shapes);
        match order {
            Order::Front | Order::Back => {
                let (mut picked, rest): (Vec<Shape>, Vec<Shape>) =
                    std::mem::take(shapes).into_iter().partition(|s| sel.contains(&s.id));
                if order == Order::Front {
                    *shapes = rest;
                    shapes.append(&mut picked);
                } else {
                    *shapes = picked;
                    shapes.extend(rest);
                }
            }
            Order::Forward => {
                for i in (0..shapes.len().saturating_sub(1)).rev() {
                    if sel.contains(&shapes[i].id) && !sel.contains(&shapes[i + 1].id) {
                        shapes.swap(i, i + 1);
                    }
                }
            }
            Order::Backward => {
                for i in 1..shapes.len() {
                    if sel.contains(&shapes[i].id) && !sel.contains(&shapes[i - 1].id) {
                        shapes.swap(i, i - 1);
                    }
                }
            }
        }
    }
}

impl Machine for WhiteboardMachine {
    type Input = WhiteboardInput;
    type State = WhiteboardState;
    type Event = WhiteboardEvent;
    type Command = WhiteboardCommand;

    fn initial(input: &WhiteboardInput) -> WhiteboardState {
        WhiteboardState { shapes: Arc::new(input.default_shapes.clone()), color: 1, ..WhiteboardState::default() }
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
                    return Transition::stay();
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
                        let before = Arc::clone(&state.shapes);
                        if let Some(t) = Arc::make_mut(&mut s.shapes).iter_mut().find(|t| t.id == id) {
                            let (w, h) = ShapeKind::Text.default_size();
                            (t.w, t.h) = (w, h);
                            t.y -= h / 2.0;
                        }
                        s.editing = Some((id, before));
                    }
                }
            }
            WhiteboardEvent::Zoom { at, factor } => s.viewport = s.viewport.zoom_at(at, factor),
            WhiteboardEvent::ZoomIn => {
                s.viewport = s.viewport.zoom_at(Point::new(s.size.0 / 2.0, s.size.1 / 2.0), 1.25);
            }
            WhiteboardEvent::ZoomOut => {
                s.viewport = s.viewport.zoom_at(Point::new(s.size.0 / 2.0, s.size.1 / 2.0), 0.8);
            }
            WhiteboardEvent::ZoomReset => {
                s.viewport = s.viewport.zoom_at(Point::new(s.size.0 / 2.0, s.size.1 / 2.0), 1.0 / s.viewport.zoom);
            }
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
                    for x in Arc::make_mut(&mut s.shapes).iter_mut().filter(|x| sel.contains(&x.id)) {
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
                    let note = s
                        .selected()
                        .next()
                        .filter(|x| x.kind.has_text() && s.selection.len() == 1)
                        .map(|x| x.id.clone());
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
                    let center =
                        f.center.plus(Point::new((w - f.w) / 2.0, (h - f.h) / 2.0).rotate(Point::default(), f.angle));
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
                    Arc::make_mut(&mut s.shapes).retain(|x| !sel.contains(&x.id));
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
                let rename = |id: &Option<String>| {
                    id.as_ref().and_then(|id| renamed.iter().find(|r| r.0 == *id).map(|r| r.1.clone()))
                };
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
                Arc::make_mut(&mut s.shapes).extend(copies);
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
                let moved = if undo {
                    s.history.undo(Arc::make_mut(&mut s.shapes))
                } else {
                    s.history.redo(Arc::make_mut(&mut s.shapes))
                };
                if moved {
                    s.gesture = None;
                    s.prune();
                    out.push(WhiteboardCommand::Change(s.shapes.to_vec()));
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
                    if let Some(x) = Arc::make_mut(&mut s.shapes).iter_mut().find(|x| x.id == id) {
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
        Transition::update(state, s, out)
    }
}

/// The event for a key on the canvas, if any. `Some` means: prevent default.
/// `cmd` is Ctrl or Cmd. Not for keys typed into the text editor.
#[must_use]
pub fn key_event(key: &str, shift: bool, cmd: bool, alt: bool) -> Option<WhiteboardEvent> {
    let step = if shift { 10.0 } else { 1.0 };
    let arrow = |dx: f64, dy: f64| {
        Some(if alt {
            WhiteboardEvent::Grow { dw: dx * step, dh: dy * step }
        } else {
            WhiteboardEvent::Nudge { dx: dx * step, dy: dy * step }
        })
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
