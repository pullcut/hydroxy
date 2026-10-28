//! Lightbox: one media item at a time in a modal, with zoom and pan for images
//! and swipe or arrow navigation between items.
//!
//! The owner holds which item is open (`index`); the machine asks for changes
//! with [`LightboxCommand::Change`] and owns the zoom and the pointer gesture.
//! Positions are in CSS pixels relative to the centre of the stage, and sizes
//! are the image's unzoomed rendered size.

use hydroxy_statechart::{Machine, Transition};

/// Least zoom: the image fitted to the stage.
pub const MIN_SCALE: f64 = 1.0;
/// Most zoom.
pub const MAX_SCALE: f64 = 4.0;
/// Where a double-click or double-tap zooms to.
pub const DOUBLE_TAP_SCALE: f64 = 2.5;
/// Zoom step for the zoom buttons and the `+` / `-` keys.
pub const ZOOM_STEP: f64 = 1.5;
/// A horizontal drag at least this long (px) on an unzoomed image changes item.
pub const SWIPE_PX: f64 = 60.0;

/// A point relative to the centre of the stage, in CSS pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    /// Rightwards.
    pub x: f64,
    /// Downwards.
    pub y: f64,
}

impl Point {
    /// A point from its coordinates.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// The image's rendered size at zoom 1, in CSS pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

/// Scale about the image centre, then translate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    /// 1 is fitted.
    pub scale: f64,
    /// Horizontal offset, px.
    pub x: f64,
    /// Vertical offset, px.
    pub y: f64,
}

impl Default for Zoom {
    fn default() -> Self {
        Self { scale: MIN_SCALE, x: 0.0, y: 0.0 }
    }
}

impl Zoom {
    /// Whether the image is zoomed in past fitted.
    #[must_use]
    pub fn is_zoomed(self) -> bool {
        self.scale > MIN_SCALE + 1e-3
    }

    /// Zoom to `scale`, keeping the image point under `at` in place.
    #[must_use]
    pub fn scale_at(self, scale: f64, at: Point, size: Size) -> Self {
        let scale = scale.clamp(MIN_SCALE, MAX_SCALE);
        let k = scale / self.scale;
        Self { scale, x: at.x - (at.x - self.x) * k, y: at.y - (at.y - self.y) * k }.clamped(size)
    }

    /// Move by `dx`, `dy`, stopping where the image edge meets its fitted box.
    #[must_use]
    pub fn pan(self, dx: f64, dy: f64, size: Size) -> Self {
        Self { x: self.x + dx, y: self.y + dy, ..self }.clamped(size)
    }

    fn clamped(self, size: Size) -> Self {
        let mx = (self.scale - 1.0) * size.w / 2.0;
        let my = (self.scale - 1.0) * size.h / 2.0;
        Self { scale: self.scale, x: self.x.clamp(-mx, mx), y: self.y.clamp(-my, my) }
    }

    /// The CSS `transform` for the image.
    #[must_use]
    pub fn css(self) -> String {
        format!("translate({:.1}px, {:.1}px) scale({:.3})", self.x, self.y, self.scale)
    }
}

/// Props for [`LightboxMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LightboxInput {
    /// Number of items.
    pub count: usize,
    /// The open item, or `None` when closed. Owned by the owner.
    pub index: Option<usize>,
    /// Wrap from the last item to the first and back.
    pub looping: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Touch {
    id: i32,
    at: Point,
}

/// What [`LightboxMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LightboxState {
    /// Zoom of the current image.
    pub zoom: Zoom,
    /// Horizontal drag of an unzoomed image, for swipe feedback.
    pub drag: f64,
    /// The item the zoom belongs to; a different one resets it.
    shown: Option<usize>,
    touches: Vec<Touch>,
    start: Option<Point>,
    pinched: bool,
}

/// Inputs to [`LightboxMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightboxEvent {
    /// Open on an item.
    Open(usize),
    /// Close.
    Close,
    /// Next item.
    Next,
    /// Previous item.
    Prev,
    /// First item.
    First,
    /// Last item.
    Last,
    /// Zoom in a step about the centre.
    ZoomIn(Size),
    /// Zoom out a step about the centre.
    ZoomOut(Size),
    /// Back to fitted.
    ResetZoom,
    /// Double-click or double-tap: zoom in at the point, or back to fitted.
    DoubleTap(Point, Size),
    /// Wheel or trackpad pinch: `delta` is the wheel's `deltaY`.
    Wheel {
        /// `deltaY`; negative zooms in.
        delta: f64,
        /// Under the pointer.
        at: Point,
        /// The image's size.
        size: Size,
    },
    /// A pointer went down on the stage.
    PointerDown(i32, Point),
    /// A pointer moved.
    PointerMove(i32, Point, Size),
    /// A pointer lifted or was cancelled.
    PointerUp(i32),
}

/// Side effects from [`LightboxMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightboxCommand {
    /// Ask the owner to show this item, or close with `None`.
    Change(Option<usize>),
}

/// Zoom, pan and navigation for one lightbox.
#[derive(Debug, Clone, Copy)]
pub struct LightboxMachine;

type Step = Transition<LightboxState, LightboxCommand>;

fn change(state: &LightboxState, to: Option<usize>, input: &LightboxInput) -> Step {
    if to == input.index {
        return Transition::to(state.clone());
    }
    let mut next = state.clone();
    next.zoom = Zoom::default();
    next.drag = 0.0;
    next.shown = to;
    Transition::with(next, vec![LightboxCommand::Change(to)])
}

/// The item `delta` away from `index`, wrapping or stopping at the ends.
fn offset(index: usize, delta: isize, input: &LightboxInput) -> usize {
    let last = input.count.saturating_sub(1);
    match (index.checked_add_signed(delta), input.looping) {
        (Some(i), _) if i <= last => i,
        (Some(_), true) | (None, false) => 0,
        (None, true) | (Some(_), false) => last,
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn mid(a: Point, b: Point) -> Point {
    Point::new(f64::midpoint(a.x, b.x), f64::midpoint(a.y, b.y))
}

fn with_zoom(state: &LightboxState, zoom: Zoom) -> Step {
    let mut next = state.clone();
    next.zoom = zoom;
    Transition::to(next)
}

fn pointer_move(state: &LightboxState, id: i32, at: Point, size: Size) -> Step {
    let Some(slot) = state.touches.iter().position(|t| t.id == id) else {
        return Transition::stay();
    };
    let mut next = state.clone();
    let before = state.touches[slot].at;
    next.touches[slot].at = at;
    if let [a, b] = state.touches[..] {
        let (a2, b2) = (next.touches[0].at, next.touches[1].at);
        let old = distance(a.at, b.at).max(1.0);
        let (m1, m2) = (mid(a.at, b.at), mid(a2, b2));
        next.zoom = state.zoom.scale_at(state.zoom.scale * distance(a2, b2) / old, m2, size).pan(m2.x - m1.x, m2.y - m1.y, size);
    } else if state.touches.len() == 1 {
        if state.zoom.is_zoomed() {
            next.zoom = state.zoom.pan(at.x - before.x, at.y - before.y, size);
        } else if let Some(start) = state.start {
            next.drag = at.x - start.x;
        }
    }
    Transition::to(next)
}

fn pointer_up(state: &LightboxState, id: i32, input: &LightboxInput) -> Step {
    let mut next = state.clone();
    next.touches.retain(|t| t.id != id);
    if !next.touches.is_empty() {
        return Transition::to(next);
    }
    let swipe = (!state.pinched && !state.zoom.is_zoomed()).then_some(state.drag);
    next.drag = 0.0;
    next.start = None;
    next.pinched = false;
    match (swipe, input.index) {
        (Some(dx), Some(i)) if dx.abs() >= SWIPE_PX => {
            let to = offset(i, if dx < 0.0 { 1 } else { -1 }, input);
            change(&next, Some(to), input)
        }
        _ => Transition::to(next),
    }
}

impl Machine for LightboxMachine {
    type Input = LightboxInput;
    type State = LightboxState;
    type Event = LightboxEvent;
    type Command = LightboxCommand;

    fn initial(input: &LightboxInput) -> LightboxState {
        LightboxState { shown: input.index, ..LightboxState::default() }
    }

    fn transition(state: &LightboxState, event: LightboxEvent, input: &LightboxInput) -> Step {
        let Some(index) = input.index.filter(|_| input.count > 0) else {
            return match event {
                LightboxEvent::Open(i) if i < input.count => change(state, Some(i), input),
                _ => Transition::stay(),
            };
        };
        let centre = Point::default();
        match event {
            LightboxEvent::Open(i) => change(state, Some(i.min(input.count - 1)), input),
            LightboxEvent::Close => change(state, None, input),
            LightboxEvent::Next => change(state, Some(offset(index, 1, input)), input),
            LightboxEvent::Prev => change(state, Some(offset(index, -1, input)), input),
            LightboxEvent::First => change(state, Some(0), input),
            LightboxEvent::Last => change(state, Some(input.count - 1), input),
            LightboxEvent::ZoomIn(size) => with_zoom(state, state.zoom.scale_at(state.zoom.scale * ZOOM_STEP, centre, size)),
            LightboxEvent::ZoomOut(size) => with_zoom(state, state.zoom.scale_at(state.zoom.scale / ZOOM_STEP, centre, size)),
            LightboxEvent::ResetZoom => with_zoom(state, Zoom::default()),
            LightboxEvent::DoubleTap(at, size) => {
                let zoom = if state.zoom.is_zoomed() { Zoom::default() } else { state.zoom.scale_at(DOUBLE_TAP_SCALE, at, size) };
                with_zoom(state, zoom)
            }
            LightboxEvent::Wheel { delta, at, size } => {
                with_zoom(state, state.zoom.scale_at(state.zoom.scale * (-delta * 0.002).exp(), at, size))
            }
            LightboxEvent::PointerDown(id, at) => {
                let mut next = state.clone();
                next.touches.retain(|t| t.id != id);
                next.touches.push(Touch { id, at });
                if next.touches.len() == 1 {
                    next.start = Some(at);
                } else {
                    next.pinched = true;
                    next.drag = 0.0;
                }
                next.touches.truncate(2);
                Transition::to(next)
            }
            LightboxEvent::PointerMove(id, at, size) => pointer_move(state, id, at, size),
            LightboxEvent::PointerUp(id) => pointer_up(state, id, input),
        }
    }

    fn sync(state: &LightboxState, input: &LightboxInput) -> Option<LightboxState> {
        if state.shown == input.index {
            return None;
        }
        Some(LightboxState { shown: input.index, ..LightboxState::default() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: Size = Size { w: 400.0, h: 300.0 };

    fn input(index: Option<usize>) -> LightboxInput {
        LightboxInput { count: 4, index, looping: false }
    }

    fn run(state: &LightboxState, event: LightboxEvent, input: &LightboxInput) -> hydroxy_statechart::Step<LightboxState, LightboxCommand> {
        LightboxMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn navigation_asks_the_owner() {
        let s = LightboxState::default();
        assert_eq!(run(&s, LightboxEvent::Open(2), &input(None)).commands, vec![LightboxCommand::Change(Some(2))]);
        assert_eq!(run(&s, LightboxEvent::Next, &input(Some(1))).commands, vec![LightboxCommand::Change(Some(2))]);
        assert!(run(&s, LightboxEvent::Next, &input(Some(3))).commands.is_empty());
        assert_eq!(run(&s, LightboxEvent::Close, &input(Some(3))).commands, vec![LightboxCommand::Change(None)]);
    }

    #[test]
    fn looping_wraps_both_ways() {
        let s = LightboxState::default();
        let mut i = input(Some(3));
        i.looping = true;
        assert_eq!(run(&s, LightboxEvent::Next, &i).commands, vec![LightboxCommand::Change(Some(0))]);
        i.index = Some(0);
        assert_eq!(run(&s, LightboxEvent::Prev, &i).commands, vec![LightboxCommand::Change(Some(3))]);
    }

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let z = Zoom::default().scale_at(2.0, Point::new(100.0, 50.0), SIZE);
        // The image point under (100, 50) was (100, 50); after: x + 2 * 100 = 100.
        assert!((z.x + 100.0).abs() < 1e-9 && (z.y + 50.0).abs() < 1e-9);
        assert!((Zoom::default().scale_at(9.0, Point::default(), SIZE).scale - MAX_SCALE).abs() < 1e-9);
    }

    #[test]
    fn pan_stops_at_the_edges() {
        let z = Zoom { scale: 2.0, x: 0.0, y: 0.0 }.pan(1000.0, -1000.0, SIZE);
        assert!((z.x - 200.0).abs() < 1e-9 && (z.y + 150.0).abs() < 1e-9);
        assert_eq!(Zoom::default().pan(50.0, 50.0, SIZE), Zoom::default());
    }

    #[test]
    fn double_tap_toggles_zoom() {
        let i = input(Some(0));
        let s = run(&LightboxState::default(), LightboxEvent::DoubleTap(Point::default(), SIZE), &i).state;
        assert!((s.zoom.scale - DOUBLE_TAP_SCALE).abs() < 1e-9);
        let s = run(&s, LightboxEvent::DoubleTap(Point::default(), SIZE), &i).state;
        assert!(!s.zoom.is_zoomed());
    }

    #[test]
    fn swipe_left_goes_next_and_short_drags_do_not() {
        let i = input(Some(1));
        let mut s = run(&LightboxState::default(), LightboxEvent::PointerDown(1, Point::new(0.0, 0.0)), &i).state;
        s = run(&s, LightboxEvent::PointerMove(1, Point::new(-30.0, 0.0), SIZE), &i).state;
        assert!((s.drag + 30.0).abs() < 1e-9);
        assert!(run(&s, LightboxEvent::PointerUp(1), &i).commands.is_empty());
        s = run(&s, LightboxEvent::PointerMove(1, Point::new(-90.0, 5.0), SIZE), &i).state;
        let up = run(&s, LightboxEvent::PointerUp(1), &i);
        assert_eq!(up.commands, vec![LightboxCommand::Change(Some(2))]);
        assert!(up.state.drag.abs() < 1e-9);
    }

    #[test]
    fn pinch_zooms_and_does_not_swipe() {
        let i = input(Some(1));
        let mut s = LightboxState::default();
        s = run(&s, LightboxEvent::PointerDown(1, Point::new(-50.0, 0.0)), &i).state;
        s = run(&s, LightboxEvent::PointerDown(2, Point::new(50.0, 0.0)), &i).state;
        s = run(&s, LightboxEvent::PointerMove(2, Point::new(150.0, 0.0), SIZE), &i).state;
        assert!((s.zoom.scale - 2.0).abs() < 1e-9);
        s = run(&s, LightboxEvent::PointerUp(2), &i).state;
        let up = run(&s, LightboxEvent::PointerMove(1, Point::new(-200.0, 0.0), SIZE), &i).state;
        assert!(run(&up, LightboxEvent::PointerUp(1), &i).commands.is_empty());
    }

    #[test]
    fn drag_pans_when_zoomed() {
        let i = input(Some(0));
        let mut s = LightboxState { zoom: Zoom { scale: 2.0, x: 0.0, y: 0.0 }, ..Default::default() };
        s = run(&s, LightboxEvent::PointerDown(1, Point::default()), &i).state;
        s = run(&s, LightboxEvent::PointerMove(1, Point::new(40.0, 20.0), SIZE), &i).state;
        assert!((s.zoom.x - 40.0).abs() < 1e-9 && (s.zoom.y - 20.0).abs() < 1e-9);
    }

    #[test]
    fn a_new_item_resets_zoom() {
        let s = LightboxState { zoom: Zoom { scale: 3.0, x: 5.0, y: 5.0 }, shown: Some(0), ..Default::default() };
        assert!(!hydroxy_statechart::sync_or_keep::<LightboxMachine>(&s, &input(Some(1))).zoom.is_zoomed());
        assert!(hydroxy_statechart::sync_or_keep::<LightboxMachine>(&s, &input(Some(0))).zoom.is_zoomed());
    }
}
