//! Signature pad: strokes drawn with a pointer, or a name typed as a keyboard
//! alternative. Each finished stroke is turned into SVG path data once: the
//! points are smoothed with a Catmull-Rom spline and the line is drawn as a
//! filled outline whose width thins as the pen speeds up, like ink.

use std::fmt::Write as _;

use hydroxy_statechart::{Machine, Transition};

/// Width of the drawing surface in its own units (the SVG `viewBox`).
pub const VIEW_WIDTH: f64 = 600.0;
/// Height of the drawing surface in its own units.
pub const VIEW_HEIGHT: f64 = 200.0;
/// Samples closer than this to the last one are dropped.
const MIN_GAP: f64 = 1.5;
/// Speed (units per ms) at which the line reaches its thinnest.
const FAST: f64 = 2.5;
/// Spline steps between two samples.
const STEPS: u32 = 4;

/// One pointer sample, in view units, with its time in ms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Across, 0 to [`VIEW_WIDTH`].
    pub x: f64,
    /// Down, 0 to [`VIEW_HEIGHT`].
    pub y: f64,
    /// Time stamp in ms. Only differences matter.
    pub t: f64,
}

/// A finished stroke: its samples and the path drawn from them.
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// The samples. Empty for a stroke loaded from a saved path.
    pub points: Vec<Point>,
    /// SVG path data for the filled outline.
    pub d: String,
}

/// How the person is signing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    /// Pointer drawing.
    #[default]
    Draw,
    /// A typed name shown in a script face.
    Type,
}

/// The signature the pad holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValue {
    /// Path data of the drawn strokes, in a [`VIEW_WIDTH`] by [`VIEW_HEIGHT`] box.
    Drawn(String),
    /// A typed name.
    Typed(String),
}

/// Props for [`SignaturePadMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct SignaturePadInput {
    /// A signature to start with.
    pub default_value: Option<SignatureValue>,
    /// Thinnest line, in view units.
    pub min_width: f64,
    /// Thickest line, in view units.
    pub max_width: f64,
    /// Ignore every change and dim the pad.
    pub disabled: bool,
    /// Show the signature but allow no change.
    pub readonly: bool,
}

impl Default for SignaturePadInput {
    fn default() -> Self {
        Self { default_value: None, min_width: 1.2, max_width: 4.2, disabled: false, readonly: false }
    }
}

/// What [`SignaturePadMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SignaturePadState {
    /// Draw or type.
    pub mode: Mode,
    /// Finished strokes, oldest first.
    pub strokes: Vec<Stroke>,
    /// The stroke under the pointer.
    pub live: Option<Vec<Point>>,
    /// The typed name.
    pub typed: String,
}

impl SignaturePadState {
    /// The signature in the current mode, or `None` when that mode is blank.
    #[must_use]
    pub fn value(&self) -> Option<SignatureValue> {
        match self.mode {
            Mode::Draw if self.strokes.is_empty() => None,
            Mode::Draw => {
                Some(SignatureValue::Drawn(self.strokes.iter().map(|s| s.d.as_str()).collect::<Vec<_>>().join(" ")))
            }
            Mode::Type if self.typed.trim().is_empty() => None,
            Mode::Type => Some(SignatureValue::Typed(self.typed.trim().to_string())),
        }
    }

    /// Whether the current mode holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value().is_none()
    }
}

/// Inputs to [`SignaturePadMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum SignaturePadEvent {
    /// Pointer down on the surface.
    Start(Point),
    /// Pointer moved while down.
    Move(Point),
    /// Pointer up: finish the stroke.
    End,
    /// Pointer lost: drop the stroke.
    Cancel,
    /// Remove the last stroke.
    Undo,
    /// Remove everything in the current mode.
    Clear,
    /// Switch between drawing and typing.
    SetMode(Mode),
    /// The typed name changed.
    SetTyped(String),
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignaturePadCommand {
    /// The signature changed; `None` when the pad is now blank.
    Change(Option<SignatureValue>),
}

/// Strokes, undo and the typed alternative of a signature pad.
#[derive(Debug, Clone, Copy)]
pub struct SignaturePadMachine;

fn clamp_point(p: Point) -> Point {
    Point { x: p.x.clamp(0.0, VIEW_WIDTH), y: p.y.clamp(0.0, VIEW_HEIGHT), t: p.t }
}

impl Machine for SignaturePadMachine {
    type Input = SignaturePadInput;
    type State = SignaturePadState;
    type Event = SignaturePadEvent;
    type Command = SignaturePadCommand;

    fn initial(input: &SignaturePadInput) -> SignaturePadState {
        let mut state = SignaturePadState::default();
        match &input.default_value {
            Some(SignatureValue::Drawn(d)) if !d.trim().is_empty() => {
                state.strokes.push(Stroke { points: Vec::new(), d: d.clone() });
            }
            Some(SignatureValue::Typed(name)) => {
                state.mode = Mode::Type;
                state.typed.clone_from(name);
            }
            _ => {}
        }
        state
    }

    fn transition(
        state: &SignaturePadState,
        event: SignaturePadEvent,
        input: &SignaturePadInput,
    ) -> Transition<SignaturePadState, SignaturePadCommand> {
        if input.disabled || input.readonly {
            return Transition::stay();
        }
        let mut next = state.clone();
        match event {
            SignaturePadEvent::Start(p) if state.mode == Mode::Draw => next.live = Some(vec![clamp_point(p)]),
            SignaturePadEvent::Move(p) => {
                if let Some(live) = next.live.as_mut() {
                    let p = clamp_point(p);
                    let last = live[live.len() - 1];
                    if (p.x - last.x).hypot(p.y - last.y) >= MIN_GAP {
                        live.push(p);
                    }
                }
            }
            SignaturePadEvent::End => {
                if let Some(points) = next.live.take() {
                    let d = stroke_path(&points, input.min_width, input.max_width);
                    next.strokes.push(Stroke { points, d });
                }
            }
            SignaturePadEvent::Cancel => next.live = None,
            SignaturePadEvent::Undo if state.mode == Mode::Draw => {
                next.strokes.pop();
            }
            SignaturePadEvent::Clear => match state.mode {
                Mode::Draw => {
                    next.strokes.clear();
                    next.live = None;
                }
                Mode::Type => next.typed.clear(),
            },
            SignaturePadEvent::SetMode(mode) => {
                next.mode = mode;
                next.live = None;
            }
            SignaturePadEvent::SetTyped(text) => next.typed = text,
            SignaturePadEvent::Start(_) | SignaturePadEvent::Undo => {}
        }
        let mut commands = Vec::new();
        if next.value() != state.value() {
            commands.push(SignaturePadCommand::Change(next.value()));
        }
        Transition::with(next, commands)
    }
}

/// Line width at each sample: thinner when fast, eased so it doesn't flicker.
fn widths(points: &[Point], min_w: f64, max_w: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(points.len());
    let mut w = f64::midpoint(min_w, max_w);
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            let q = points[i - 1];
            // Samples without real time stamps arrive about once a frame.
            let dt = if p.t > q.t { p.t - q.t } else { 16.0 };
            let speed = (p.x - q.x).hypot(p.y - q.y) / dt;
            let target = max_w - (max_w - min_w) * (speed / FAST).min(1.0);
            w = w * 0.6 + target * 0.4;
        }
        out.push(w);
    }
    out
}

/// Catmull-Rom points between samples, each with an interpolated width.
fn smooth(points: &[Point], widths: &[f64]) -> Vec<(f64, f64, f64)> {
    let n = points.len();
    let at = |i: usize| points[i.min(n - 1)];
    let mut out = Vec::new();
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (at(i.saturating_sub(1)), at(i), at(i + 1), at(i + 2));
        for s in 0..STEPS {
            let t = f64::from(s) / f64::from(STEPS);
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f64, b: f64, c: f64, d: f64| {
                0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (3.0 * b - a - 3.0 * c + d) * t3)
            };
            let w = widths[i] + (widths[i + 1] - widths[i]) * t;
            out.push((f(p0.x, p1.x, p2.x, p3.x), f(p0.y, p1.y, p2.y, p3.y), w));
        }
    }
    let last = points[n - 1];
    out.push((last.x, last.y, widths[n - 1]));
    out
}

fn num(v: f64) -> String {
    let s = format!("{:.1}", (v * 10.0).round() / 10.0);
    match s.strip_suffix(".0") {
        Some(whole) if whole != "-0" => whole.to_string(),
        Some(_) => "0".to_string(),
        None => s,
    }
}

/// SVG path data for one stroke: a filled outline with round ends, or a dot for a tap.
#[must_use]
pub fn stroke_path(points: &[Point], min_width: f64, max_width: f64) -> String {
    let Some(first) = points.first() else { return String::new() };
    if points.len() == 1 {
        let r = max_width / 2.0;
        return format!(
            "M{} {}a{r2} {r2} 0 1 0 {d} 0a{r2} {r2} 0 1 0 -{d} 0Z",
            num(first.x - r),
            num(first.y),
            r2 = num(r),
            d = num(r * 2.0)
        );
    }
    let line = smooth(points, &widths(points, min_width, max_width));
    let mut left = Vec::with_capacity(line.len());
    let mut right = Vec::with_capacity(line.len());
    for (i, &(x, y, w)) in line.iter().enumerate() {
        let (ax, ay, _) = line[i.saturating_sub(1)];
        let (bx, by, _) = line[(i + 1).min(line.len() - 1)];
        let (dx, dy) = (bx - ax, by - ay);
        let len = dx.hypot(dy).max(f64::EPSILON);
        let (nx, ny) = (-dy / len * w / 2.0, dx / len * w / 2.0);
        left.push((x + nx, y + ny));
        right.push((x - nx, y - ny));
    }
    let r_end = num(line[line.len() - 1].2 / 2.0);
    let r_start = num(line[0].2 / 2.0);
    let mut d = format!("M{} {}", num(left[0].0), num(left[0].1));
    for &(x, y) in &left[1..] {
        let _ = write!(d, "L{} {}", num(x), num(y));
    }
    let (ex, ey) = right[right.len() - 1];
    let _ = write!(d, "A{r_end} {r_end} 0 0 1 {} {}", num(ex), num(ey));
    for &(x, y) in right.iter().rev().skip(1) {
        let _ = write!(d, "L{} {}", num(x), num(y));
    }
    let _ = write!(d, "A{r_start} {r_start} 0 0 1 {} {}Z", num(left[0].0), num(left[0].1));
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use SignaturePadEvent as E;
    use hydroxy_statechart::step;

    fn p(x: f64, y: f64, t: f64) -> Point {
        Point { x, y, t }
    }

    fn run(
        s: &SignaturePadState,
        e: SignaturePadEvent,
    ) -> hydroxy_statechart::Step<SignaturePadState, SignaturePadCommand> {
        step::<SignaturePadMachine>(s, e, &SignaturePadInput::default())
    }

    fn draw(s: &SignaturePadState, pts: &[Point]) -> SignaturePadState {
        let mut s = run(s, E::Start(pts[0])).state;
        for &q in &pts[1..] {
            s = run(&s, E::Move(q)).state;
        }
        s
    }

    #[test]
    fn a_stroke_finishes_into_path_data_and_reports_a_change() {
        let s = draw(&SignaturePadState::default(), &[p(10.0, 10.0, 0.0), p(40.0, 30.0, 16.0), p(80.0, 20.0, 32.0)]);
        let t = run(&s, E::End);
        assert_eq!(t.state.strokes.len(), 1);
        assert!(t.state.strokes[0].d.starts_with('M') && t.state.strokes[0].d.ends_with('Z'));
        assert!(matches!(t.commands.as_slice(), [SignaturePadCommand::Change(Some(SignatureValue::Drawn(_)))]));
    }

    #[test]
    fn close_samples_are_dropped_and_points_clamped() {
        let s = draw(&SignaturePadState::default(), &[p(10.0, 10.0, 0.0), p(10.5, 10.5, 8.0), p(-50.0, 900.0, 16.0)]);
        let live = s.live.unwrap();
        assert_eq!(live.len(), 2);
        assert_eq!((live[1].x, live[1].y), (0.0, VIEW_HEIGHT));
    }

    #[test]
    fn a_tap_draws_a_dot() {
        let d = stroke_path(&[p(50.0, 50.0, 0.0)], 1.0, 4.0);
        assert_eq!(d, "M48 50a2 2 0 1 0 4 0a2 2 0 1 0 -4 0Z");
    }

    #[test]
    fn fast_strokes_are_thinner_than_slow_ones() {
        let slow: Vec<Point> = (0..10).map(|i| p(f64::from(i) * 2.0, 0.0, f64::from(i) * 16.0)).collect();
        let fast: Vec<Point> = (0..10).map(|i| p(f64::from(i) * 60.0, 0.0, f64::from(i) * 16.0)).collect();
        let ws = widths(&slow, 1.0, 5.0);
        let wf = widths(&fast, 1.0, 5.0);
        assert!(wf[9] < ws[9]);
        assert!(wf[9] >= 1.0 && ws[9] <= 5.0);
    }

    #[test]
    fn undo_and_clear() {
        let mut s = SignaturePadState::default();
        for y in [20.0, 60.0] {
            s = run(&draw(&s, &[p(10.0, y, 0.0), p(90.0, y, 16.0)]), E::End).state;
        }
        let t = run(&s, E::Undo);
        assert_eq!(t.state.strokes.len(), 1);
        let t = run(&t.state, E::Clear);
        assert!(t.state.strokes.is_empty());
        assert_eq!(t.commands, vec![SignaturePadCommand::Change(None)]);
    }

    #[test]
    fn typing_is_the_keyboard_alternative() {
        let s = run(&SignaturePadState::default(), E::SetMode(Mode::Type)).state;
        assert!(run(&s, E::Start(p(1.0, 1.0, 0.0))).state.live.is_none());
        let t = run(&s, E::SetTyped(" Ada Lovelace ".into()));
        assert_eq!(t.commands, vec![SignaturePadCommand::Change(Some(SignatureValue::Typed("Ada Lovelace".into())))]);
    }

    #[test]
    fn disabled_and_readonly_ignore_input() {
        let s = SignaturePadState::default();
        for input in [
            SignaturePadInput { disabled: true, ..SignaturePadInput::default() },
            SignaturePadInput { readonly: true, ..SignaturePadInput::default() },
        ] {
            let t = step::<SignaturePadMachine>(&s, E::Start(p(1.0, 1.0, 0.0)), &input);
            assert!(t.state.live.is_none());
        }
    }

    #[test]
    fn a_saved_path_loads_as_one_undoable_stroke() {
        let input = SignaturePadInput {
            default_value: Some(SignatureValue::Drawn("M0 0L5 5Z".into())),
            ..SignaturePadInput::default()
        };
        let s = SignaturePadMachine::initial(&input);
        assert_eq!(s.value(), Some(SignatureValue::Drawn("M0 0L5 5Z".into())));
        assert!(run(&s, E::Undo).state.is_empty());
    }
}
