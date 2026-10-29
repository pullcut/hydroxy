use super::shape::BIND_GAP;
use super::*;
use crate::canvas::close;
use hydroxy_statechart::step;

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
        let t = step::<WhiteboardMachine>(&s, e, &input);
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
    let shapes = vec![
        rect("a", 0.0, 0.0).text("say \"hi\"\n"),
        Shape::new("n", ShapeKind::Sticky, Rect::new(0.0, 100.0, 200.0, 100.0)).text("a < b"),
    ];
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
fn key_event_maps_tools_undo_resize_and_order() {
    assert_eq!(key_event("r", Modifiers::NONE), Some(WhiteboardEvent::Tool(Tool::Rect)));
    assert_eq!(key_event("Z", Modifiers { shift: true, ctrl: true, ..Modifiers::NONE }), Some(WhiteboardEvent::Redo));
    assert_eq!(
        key_event("ArrowLeft", Modifiers { shift: true, alt: true, ..Modifiers::NONE }),
        Some(WhiteboardEvent::Grow { dw: -10.0, dh: 0.0 })
    );
    assert_eq!(
        key_event("]", Modifiers { shift: true, ctrl: true, ..Modifiers::NONE }),
        Some(WhiteboardEvent::Order(Order::Front))
    );
    assert_eq!(key_event("q", Modifiers::NONE), None);
}
