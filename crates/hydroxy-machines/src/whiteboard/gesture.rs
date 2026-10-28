//! Selection handles, the selection frame and the gesture in progress.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::Arc;

use super::{Point, Shape};

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
        let local = if h == Handle::Rotate {
            Point::new(0.0, -self.h / 2.0 - 28.0 / zoom)
        } else {
            Point::new(dx * self.w / 2.0, dy * self.h / 2.0)
        };
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
