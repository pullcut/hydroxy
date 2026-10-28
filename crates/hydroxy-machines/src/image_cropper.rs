//! Image cropper: a crop box over an image that can be moved, resized from
//! eight handles, locked to an aspect ratio, nudged from the keyboard, and kept
//! inside the image through quarter-turn rotations.
//!
//! The box lives in *view* pixels: the image's own pixels after rotation, so
//! dragging needs no trigonometry. [`to_source`] maps it back to the pixels of
//! the unrotated file, which is what the owner gets.

use hydroxy_statechart::{Machine, Transition};

/// A rectangle in pixels.
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
    fn center(self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

/// What part of the box a pointer holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    /// The box body: move it.
    Move,
    /// Top edge.
    N,
    /// Bottom edge.
    S,
    /// Right edge.
    E,
    /// Left edge.
    W,
    /// Top-right corner.
    NE,
    /// Top-left corner.
    NW,
    /// Bottom-right corner.
    SE,
    /// Bottom-left corner.
    SW,
}

impl Handle {
    /// The eight resize handles, corners first.
    pub const RESIZE: [Handle; 8] =
        [Handle::NW, Handle::NE, Handle::SE, Handle::SW, Handle::N, Handle::E, Handle::S, Handle::W];

    /// Short name used in `data-handle`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Handle::Move => "move",
            Handle::N => "n",
            Handle::S => "s",
            Handle::E => "e",
            Handle::W => "w",
            Handle::NE => "ne",
            Handle::NW => "nw",
            Handle::SE => "se",
            Handle::SW => "sw",
        }
    }

    /// Parse a `data-handle` value.
    #[must_use]
    pub fn parse(s: &str) -> Option<Handle> {
        std::iter::once(Handle::Move).chain(Handle::RESIZE).find(|h| h.as_str() == s)
    }

    fn west(self) -> bool {
        matches!(self, Handle::W | Handle::NW | Handle::SW)
    }
    fn east(self) -> bool {
        matches!(self, Handle::E | Handle::NE | Handle::SE)
    }
    fn north(self) -> bool {
        matches!(self, Handle::N | Handle::NE | Handle::NW)
    }
    fn south(self) -> bool {
        matches!(self, Handle::S | Handle::SE | Handle::SW)
    }
}

/// The crop handed to the owner, in pixels of the unrotated source image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crop {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// Clockwise turn to apply after cropping: 0, 90, 180 or 270.
    pub rotation: u16,
}

/// Props for [`ImageCropperMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCropperInput {
    /// Aspect lock (width / height) to start with; `None` is free.
    pub aspect: Option<f64>,
    /// Smallest box side, in source pixels.
    pub min_size: f64,
    /// Largest zoom.
    pub max_zoom: f64,
    /// Ignore every change.
    pub disabled: bool,
}

impl Default for ImageCropperInput {
    fn default() -> Self {
        Self { aspect: None, min_size: 24.0, max_zoom: 4.0, disabled: false }
    }
}

/// A pointer drag in progress.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drag {
    /// What is held.
    pub handle: Handle,
    /// Pointer at the press, in view pixels.
    pub from: (f64, f64),
    /// The box at the press.
    pub start: Rect,
}

/// What [`ImageCropperMachine`] remembers.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCropperState {
    /// Source image size; zero until it loads.
    pub image: (f64, f64),
    /// Clockwise quarter turns, 0 to 3.
    pub turns: u8,
    /// The box in view pixels.
    pub crop: Rect,
    /// Locked width / height, or free.
    pub aspect: Option<f64>,
    /// View zoom, 1 or more.
    pub zoom: f64,
    /// The drag under way.
    pub drag: Option<Drag>,
}

impl Default for ImageCropperState {
    fn default() -> Self {
        Self { image: (0.0, 0.0), turns: 0, crop: Rect::default(), aspect: None, zoom: 1.0, drag: None }
    }
}

impl ImageCropperState {
    /// Image size after rotation.
    #[must_use]
    pub fn view_size(&self) -> (f64, f64) {
        let (w, h) = self.image;
        if self.turns.is_multiple_of(2) { (w, h) } else { (h, w) }
    }

    /// Whether the image size is known.
    #[must_use]
    pub fn loaded(&self) -> bool {
        self.image.0 > 0.0 && self.image.1 > 0.0
    }

    /// The crop in source pixels, rounded, with the rotation.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "rounded and clamped to the image")]
    pub fn crop(&self) -> Crop {
        let r = to_source(self.crop, self.turns, self.image);
        let px = |v: f64| v.round().max(0.0) as u32;
        Crop { x: px(r.x), y: px(r.y), width: px(r.w), height: px(r.h), rotation: u16::from(self.turns) * 90 }
    }

    /// Keyboard step in view pixels: a hundredth of the longer side, at least one.
    #[must_use]
    pub fn step(&self) -> f64 {
        (self.image.0.max(self.image.1) / 100.0).round().max(1.0)
    }
}

/// Inputs to [`ImageCropperMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum ImageCropperEvent {
    /// The image loaded with this natural size.
    Loaded {
        /// Width in pixels.
        width: f64,
        /// Height in pixels.
        height: f64,
    },
    /// Pointer down on the box or a handle, at view pixels.
    Press {
        /// What was pressed.
        handle: Handle,
        /// Pointer across.
        x: f64,
        /// Pointer down.
        y: f64,
    },
    /// Pointer moved while held, at view pixels.
    Drag {
        /// Pointer across.
        x: f64,
        /// Pointer down.
        y: f64,
    },
    /// Pointer released.
    Release,
    /// Move the box by view pixels.
    Nudge {
        /// Across.
        dx: f64,
        /// Down.
        dy: f64,
    },
    /// Grow or shrink the box from its bottom-right by view pixels.
    Grow {
        /// Width change.
        dw: f64,
        /// Height change.
        dh: f64,
    },
    /// Lock to width / height, or free.
    SetAspect(Option<f64>),
    /// Quarter turn: clockwise when true.
    Rotate(bool),
    /// Set the view zoom.
    SetZoom(f64),
    /// Box back to the default, no rotation, no zoom.
    Reset,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCropperCommand {
    /// The crop changed.
    Change(Crop),
}

/// Crop box geometry for an image.
#[derive(Debug, Clone, Copy)]
pub struct ImageCropperMachine;

/// Map a view rect to source pixels for `turns` clockwise quarter turns of an image `size`.
#[must_use]
pub fn to_source(r: Rect, turns: u8, (w, h): (f64, f64)) -> Rect {
    match turns % 4 {
        1 => Rect { x: r.y, y: h - r.x - r.w, w: r.h, h: r.w },
        2 => Rect { x: w - r.x - r.w, y: h - r.y - r.h, ..r },
        3 => Rect { x: w - r.y - r.h, y: r.x, w: r.h, h: r.w },
        _ => r,
    }
}

/// Map a source rect into view pixels: the inverse of [`to_source`].
#[must_use]
pub fn to_view(r: Rect, turns: u8, (w, h): (f64, f64)) -> Rect {
    match turns % 4 {
        1 => Rect { x: h - r.y - r.h, y: r.x, w: r.h, h: r.w },
        2 => Rect { x: w - r.x - r.w, y: h - r.y - r.h, ..r },
        3 => Rect { x: r.y, y: w - r.x - r.w, w: r.h, h: r.w },
        _ => r,
    }
}

/// The largest box of `aspect` that fits `bounds`, scaled by `scale` and centred.
fn centred(aspect: Option<f64>, (bw, bh): (f64, f64), scale: f64) -> Rect {
    let (w, h) = match aspect {
        Some(a) if bw / bh > a => (bh * a, bh),
        Some(a) => (bw, bw / a),
        None => (bw, bh),
    };
    let (w, h) = (w * scale, h * scale);
    Rect { x: (bw - w) / 2.0, y: (bh - h) / 2.0, w, h }
}

/// Reshape `r` to `aspect` around its centre, keeping roughly its area, inside `bounds`.
fn refit(r: Rect, aspect: f64, (bw, bh): (f64, f64), min: f64) -> Rect {
    let area = (r.w * r.h).max(min * min);
    let mut w = (area * aspect).sqrt();
    let mut h = w / aspect;
    let shrink = (bw / w).min(bh / h).min(1.0);
    w *= shrink;
    h *= shrink;
    let (cx, cy) = r.center();
    Rect { x: (cx - w / 2.0).clamp(0.0, bw - w), y: (cy - h / 2.0).clamp(0.0, bh - h), w, h }
}

/// The box after dragging `handle` by `(dx, dy)` from `s`, kept in `bounds`.
#[must_use]
#[allow(clippy::many_single_char_names, reason = "edges and sizes read best short")]
pub fn resize(
    s: Rect,
    handle: Handle,
    (dx, dy): (f64, f64),
    aspect: Option<f64>,
    (bw, bh): (f64, f64),
    min: f64,
) -> Rect {
    if handle == Handle::Move {
        return Rect { x: (s.x + dx).clamp(0.0, bw - s.w), y: (s.y + dy).clamp(0.0, bh - s.h), ..s };
    }
    let min = min.min(bw).min(bh);
    let (right, bottom) = (s.x + s.w, s.y + s.h);
    let Some(a) = aspect else {
        let left = if handle.west() { (s.x + dx).clamp(0.0, right - min) } else { s.x };
        let r = if handle.east() { (right + dx).clamp(s.x + min, bw) } else { right };
        let top = if handle.north() { (s.y + dy).clamp(0.0, bottom - min) } else { s.y };
        let b = if handle.south() { (bottom + dy).clamp(s.y + min, bh) } else { bottom };
        return Rect { x: left, y: top, w: r - left, h: b - top };
    };
    let (cx, cy) = s.center();
    let horizontal = handle.east() || handle.west();
    let vertical = handle.north() || handle.south();
    // Room from the fixed side (or centre line, for an edge) to the image border.
    let room_w = if handle.east() {
        bw - s.x
    } else if handle.west() {
        right
    } else {
        2.0 * cx.min(bw - cx)
    };
    let room_h = if handle.south() {
        bh - s.y
    } else if handle.north() {
        bottom
    } else {
        2.0 * cy.min(bh - cy)
    };
    let from_x = if handle.east() { s.w + dx } else { s.w - dx };
    let from_y = if handle.south() { s.h + dy } else { s.h - dy };
    let mut w = match (horizontal, vertical) {
        (true, true) => from_x.max(from_y * a),
        (true, false) => from_x,
        _ => from_y * a,
    };
    let min_w = min.max(min * a);
    w = w.min(room_w).min(room_h * a).max(min_w.min(room_w).min(room_h * a));
    let h = w / a;
    let x = if handle.east() {
        s.x
    } else if handle.west() {
        right - w
    } else {
        cx - w / 2.0
    };
    let y = if handle.south() {
        s.y
    } else if handle.north() {
        bottom - h
    } else {
        cy - h / 2.0
    };
    Rect { x, y, w, h }
}

impl Machine for ImageCropperMachine {
    type Input = ImageCropperInput;
    type State = ImageCropperState;
    type Event = ImageCropperEvent;
    type Command = ImageCropperCommand;

    fn initial(input: &ImageCropperInput) -> ImageCropperState {
        ImageCropperState { aspect: input.aspect, ..ImageCropperState::default() }
    }

    fn transition(
        state: &ImageCropperState,
        event: ImageCropperEvent,
        input: &ImageCropperInput,
    ) -> Transition<ImageCropperState, ImageCropperCommand> {
        let loading = matches!(event, ImageCropperEvent::Loaded { .. });
        if !loading && (input.disabled || !state.loaded()) {
            return Transition::stay();
        }
        let mut next = state.clone();
        let bounds = state.view_size();
        match event {
            ImageCropperEvent::Loaded { width, height } => {
                if width > 0.0 && height > 0.0 {
                    next.image = (width, height);
                    next.turns = 0;
                    next.crop = centred(state.aspect, (width, height), 0.8);
                }
            }
            ImageCropperEvent::Press { handle, x, y } => {
                next.drag = Some(Drag { handle, from: (x, y), start: state.crop });
            }
            ImageCropperEvent::Drag { x, y } => {
                if let Some(d) = state.drag {
                    let delta = (x - d.from.0, y - d.from.1);
                    next.crop = resize(d.start, d.handle, delta, state.aspect, bounds, input.min_size);
                }
            }
            ImageCropperEvent::Release => next.drag = None,
            ImageCropperEvent::Nudge { dx, dy } => {
                next.crop = resize(state.crop, Handle::Move, (dx, dy), None, bounds, input.min_size);
            }
            ImageCropperEvent::Grow { dw, dh } => {
                let (dw, dh) = if state.aspect.is_some() && dw == 0.0 { (dh, dh) } else { (dw, dh) };
                next.crop = resize(state.crop, Handle::SE, (dw, dh), state.aspect, bounds, input.min_size);
            }
            ImageCropperEvent::SetAspect(aspect) => {
                next.aspect = aspect.filter(|a| a.is_finite() && *a > 0.0);
                if let Some(a) = next.aspect {
                    next.crop = refit(state.crop, a, bounds, input.min_size);
                }
            }
            ImageCropperEvent::Rotate(clockwise) => {
                next.turns = if clockwise { (state.turns + 1) % 4 } else { (state.turns + 3) % 4 };
                let source = to_source(state.crop, state.turns, state.image);
                next.crop = to_view(source, next.turns, state.image);
                if let Some(a) = next.aspect {
                    next.crop = refit(next.crop, a, next.view_size(), input.min_size);
                }
                next.drag = None;
            }
            ImageCropperEvent::SetZoom(z) => next.zoom = z.clamp(1.0, input.max_zoom.max(1.0)),
            ImageCropperEvent::Reset => {
                next.turns = 0;
                next.zoom = 1.0;
                next.drag = None;
                next.crop = centred(state.aspect, state.image, 0.8);
            }
        }
        let mut commands = Vec::new();
        if next.loaded() && (next.crop() != state.crop() || !state.loaded()) {
            commands.push(ImageCropperCommand::Change(next.crop()));
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ImageCropperEvent as E;

    fn loaded(w: f64, h: f64) -> ImageCropperState {
        run(&ImageCropperState::default(), E::Loaded { width: w, height: h }).state
    }

    fn run(
        s: &ImageCropperState,
        e: ImageCropperEvent,
    ) -> hydroxy_statechart::Step<ImageCropperState, ImageCropperCommand> {
        ImageCropperMachine::transition(s, e, &ImageCropperInput::default()).resolve(s)
    }

    fn drag(s: &ImageCropperState, handle: Handle, from: (f64, f64), to: (f64, f64)) -> ImageCropperState {
        let s = run(s, E::Press { handle, x: from.0, y: from.1 }).state;
        let s = run(&s, E::Drag { x: to.0, y: to.1 }).state;
        run(&s, E::Release).state
    }

    #[test]
    fn loading_centres_a_box_and_reports_it() {
        let t = run(&ImageCropperState::default(), E::Loaded { width: 1000.0, height: 500.0 });
        assert_eq!(t.state.crop, Rect { x: 100.0, y: 50.0, w: 800.0, h: 400.0 });
        assert_eq!(
            t.commands,
            vec![ImageCropperCommand::Change(Crop { x: 100, y: 50, width: 800, height: 400, rotation: 0 })]
        );
    }

    #[test]
    fn moving_stays_inside_the_image() {
        let s = drag(&loaded(1000.0, 500.0), Handle::Move, (500.0, 250.0), (5000.0, -900.0));
        assert_eq!((s.crop.x, s.crop.y), (200.0, 0.0));
    }

    #[test]
    fn free_resize_clamps_and_keeps_a_minimum() {
        let s = loaded(1000.0, 500.0);
        let s2 = drag(&s, Handle::NW, (100.0, 50.0), (-50.0, -50.0));
        assert_eq!(s2.crop, Rect { x: 0.0, y: 0.0, w: 900.0, h: 450.0 });
        let s3 = drag(&s, Handle::E, (900.0, 250.0), (0.0, 250.0));
        assert_eq!(s3.crop.w, 24.0);
        assert_eq!(s3.crop.x, 100.0);
    }

    #[test]
    fn locked_aspect_holds_through_corner_and_edge_drags() {
        let s = run(&loaded(1000.0, 500.0), E::SetAspect(Some(1.0))).state;
        assert!((s.crop.w - s.crop.h).abs() < 1e-9);
        let s = drag(&s, Handle::SE, (0.0, 0.0), (300.0, 10.0));
        assert!((s.crop.w - s.crop.h).abs() < 1e-9);
        assert!(s.crop.y + s.crop.h <= 500.0 + 1e-9);
        let s = drag(&s, Handle::N, (0.0, 0.0), (0.0, -1000.0));
        assert!((s.crop.w - s.crop.h).abs() < 1e-9);
        assert!(s.crop.y >= 0.0 && s.crop.x >= 0.0 && s.crop.x + s.crop.w <= 1000.0 + 1e-9);
    }

    #[test]
    fn rotation_round_trips_and_reports_source_pixels() {
        let r = Rect { x: 10.0, y: 20.0, w: 30.0, h: 40.0 };
        for turns in 0..4 {
            assert_eq!(to_source(to_view(r, turns, (200.0, 100.0)), turns, (200.0, 100.0)), r);
        }
        let mut s = loaded(200.0, 100.0);
        s.crop = r;
        let s = run(&s, E::Rotate(true)).state;
        assert_eq!(s.view_size(), (100.0, 200.0));
        assert_eq!(s.crop(), Crop { x: 10, y: 20, width: 30, height: 40, rotation: 90 });
        let s = run(&run(&run(&s, E::Rotate(false)).state, E::Rotate(false)).state, E::Rotate(false)).state;
        assert_eq!(s.crop(), Crop { x: 10, y: 20, width: 30, height: 40, rotation: 180 });
    }

    #[test]
    fn keyboard_nudge_and_grow() {
        let s = loaded(1000.0, 500.0);
        let s = run(&s, E::Nudge { dx: 10.0, dy: -5.0 }).state;
        assert_eq!((s.crop.x, s.crop.y), (110.0, 45.0));
        let s = run(&s, E::Grow { dw: 10.0, dh: 0.0 }).state;
        assert_eq!(s.crop.w, 810.0);
        assert_eq!(s.step(), 10.0);
    }

    #[test]
    fn zoom_is_clamped_and_unloaded_ignores_input() {
        let s = run(&loaded(10.0, 10.0), E::SetZoom(9.0)).state;
        assert_eq!(s.zoom, 4.0);
        let t = run(&ImageCropperState::default(), E::Nudge { dx: 1.0, dy: 1.0 });
        assert!(t.commands.is_empty());
    }

    #[test]
    fn handles_round_trip_through_their_names() {
        for h in Handle::RESIZE {
            assert_eq!(Handle::parse(h.as_str()), Some(h));
        }
        assert_eq!(Handle::parse("move"), Some(Handle::Move));
    }
}
