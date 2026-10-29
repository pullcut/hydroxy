use super::*;
use crate::canvas::close;
use hydroxy_statechart::step;

fn graph() -> Graph {
    Graph::new(
        vec![
            Node::new("a", "Source", 0.0, 0.0)
                .outputs(vec![Port::new("n", "Number", "number"), Port::new("t", "Text", "text")]),
            Node::new("b", "Sink", 400.0, 0.0)
                .inputs(vec![Port::new("n", "Number", "number"), Port::new("x", "Any", PortKind::Any)]),
        ],
        vec![],
    )
}

fn start() -> (NodeEditorState, NodeEditorInput) {
    let input = NodeEditorInput { graph: graph(), ..NodeEditorInput::default() };
    let mut s = NodeEditorMachine::initial(&input);
    s.size = (800.0, 600.0);
    (s, input)
}

fn run(
    s: &NodeEditorState,
    input: &NodeEditorInput,
    events: Vec<NodeEditorEvent>,
) -> (NodeEditorState, Vec<NodeEditorCommand>) {
    let mut s = s.clone();
    let mut all = Vec::new();
    for e in events {
        let t = step::<NodeEditorMachine>(&s, e, input);
        s = t.state;
        all.extend(t.commands);
    }
    (s, all)
}

fn down(x: f64, y: f64, shift: bool) -> NodeEditorEvent {
    NodeEditorEvent::PointerDown { id: 1, at: Point::new(x, y), shift, pan: false }
}
fn mv(x: f64, y: f64) -> NodeEditorEvent {
    NodeEditorEvent::PointerMove { id: 1, at: Point::new(x, y) }
}
fn up(x: f64, y: f64) -> NodeEditorEvent {
    NodeEditorEvent::PointerUp { id: 1, at: Point::new(x, y) }
}

#[test]
fn layout_puts_ports_on_the_edges() {
    let g = graph();
    let a = g.node("a").unwrap();
    assert!(close(a.height(), HEADER + ROW * 2.0 + 8.0));
    let p = g.port_point(&PortRef::new("a", Side::Output, "t")).unwrap();
    assert!(close(p.x, 200.0) && close(p.y, HEADER + ROW * 1.5));
}

#[test]
fn types_must_match() {
    let g = graph();
    let out_t = PortRef::new("a", Side::Output, "t");
    assert_eq!(
        check(&g, &out_t, &PortRef::new("b", Side::Input, "n")),
        Err(Refusal::Types { out: "text".into(), input: "number".into() })
    );
    assert!(check(&g, &PortRef::new("b", Side::Input, "x"), &out_t).is_ok());
    assert_eq!(check(&g, &out_t, &PortRef::new("a", Side::Output, "n")), Err(Refusal::SameSide));
}

#[test]
fn dragging_a_wire_connects_compatible_ports() {
    let (s, input) = start();
    let out = s.graph.port_point(&PortRef::new("a", Side::Output, "n")).unwrap();
    let inp = s.graph.port_point(&PortRef::new("b", Side::Input, "n")).unwrap();
    let (mid, _) = run(&s, &input, vec![down(out.x, out.y, false), mv(inp.x - 10.0, inp.y + 5.0)]);
    assert_eq!(
        mid.gesture.as_ref().and_then(|g| if let Gesture::Connect { target, .. } = g { target.clone() } else { None }),
        Some(PortRef::new("b", Side::Input, "n"))
    );
    let (from, _) = mid.preview_wire().unwrap();
    assert!(close(from.x, out.x));
    let (done, cmds) = run(&mid, &input, vec![up(inp.x - 10.0, inp.y + 5.0)]);
    assert_eq!(done.graph.edges, vec![Edge::new("a", "n", "b", "n")]);
    assert!(matches!(cmds[0], NodeEditorCommand::Change(_)));
    assert!(done.can_undo());
}

#[test]
fn incompatible_drop_is_refused_and_announced() {
    let (s, input) = start();
    let out = s.graph.port_point(&PortRef::new("a", Side::Output, "t")).unwrap();
    let inp = s.graph.port_point(&PortRef::new("b", Side::Input, "n")).unwrap();
    let (done, cmds) = run(&s, &input, vec![down(out.x, out.y, false), mv(inp.x, inp.y), up(inp.x, inp.y)]);
    assert!(done.graph.edges.is_empty());
    assert_eq!(cmds, vec![NodeEditorCommand::Announce("Can't connect text to number.".into())]);
}

#[test]
fn pulling_a_wire_off_an_input_removes_it() {
    let (mut s, input) = start();
    Arc::make_mut(&mut s.graph).edges.push(Edge::new("a", "n", "b", "n"));
    let inp = s.graph.port_point(&PortRef::new("b", Side::Input, "n")).unwrap();
    let (mid, _) = run(&s, &input, vec![down(inp.x, inp.y, false), mv(600.0, 400.0)]);
    assert_eq!(mid.detached(), Some("a:n->b:n"));
    let (done, _) = run(&mid, &input, vec![up(600.0, 400.0)]);
    assert!(done.graph.edges.is_empty());
}

#[test]
fn dragging_moves_the_selection_on_the_grid() {
    let (s, input) = start();
    let (done, cmds) = run(&s, &input, vec![down(20.0, 10.0, false), mv(45.0, 15.0), up(45.0, 15.0)]);
    let a = done.graph.node("a").unwrap();
    assert!(close(a.x, 32.0) && close(a.y, 0.0));
    assert_eq!(done.selection.nodes, BTreeSet::from(["a".to_string()]));
    assert!(matches!(cmds.as_slice(), [NodeEditorCommand::Change(_)]));
    let (undone, _) = run(&done, &input, vec![NodeEditorEvent::Undo]);
    assert!(close(undone.graph.node("a").unwrap().x, 0.0));
    let (redone, _) = run(&undone, &input, vec![NodeEditorEvent::Redo]);
    assert!(close(redone.graph.node("a").unwrap().x, 32.0));
}

#[test]
fn shift_selects_and_multi_drags() {
    let (s, input) = start();
    let (sel, _) = run(
        &s,
        &input,
        vec![down(20.0, 10.0, false), up(20.0, 10.0), down(420.0, 10.0, true), mv(436.0, 42.0), up(436.0, 42.0)],
    );
    assert_eq!(sel.selection.nodes.len(), 2);
    assert!(close(sel.graph.node("a").unwrap().x, 16.0) && close(sel.graph.node("b").unwrap().y, 32.0));
}

#[test]
fn marquee_and_background_pan() {
    let (s, input) = start();
    let (m, _) = run(&s, &input, vec![down(-20.0, -20.0, true), mv(250.0, 50.0)]);
    assert_eq!(m.marquee(), Some(Rect::new(-20.0, -20.0, 270.0, 70.0)));
    assert_eq!(m.selection.nodes, BTreeSet::from(["a".to_string()]));
    let (p, _) = run(&s, &input, vec![down(300.0, 300.0, false), mv(320.0, 290.0), up(320.0, 290.0)]);
    assert!(close(p.viewport.x, 20.0) && close(p.viewport.y, -10.0));
}

#[test]
fn keyboard_connect_nudge_delete() {
    let (s, input) = start();
    let (k, _) = run(
        &s,
        &input,
        vec![
            NodeEditorEvent::PortKey(PortRef::new("a", Side::Output, "n")),
            NodeEditorEvent::PortKey(PortRef::new("b", Side::Input, "x")),
        ],
    );
    assert_eq!(k.graph.edges.len(), 1);
    let (n, _) =
        run(&k, &input, vec![NodeEditorEvent::Focus("b".into()), NodeEditorEvent::Nudge { dx: 1, dy: 0, big: true }]);
    assert!(close(n.graph.node("b").unwrap().x, 464.0));
    let (d, cmds) = run(&n, &input, vec![NodeEditorEvent::Delete]);
    assert_eq!(d.graph.nodes.len(), 1);
    assert!(d.graph.edges.is_empty());
    assert!(cmds.contains(&NodeEditorCommand::Announce("Deleted 1 node and 1 wire.".into())));
}

#[test]
fn edges_are_hit_along_the_curve() {
    let mut g = graph();
    g.edges.push(Edge::new("a", "n", "b", "n"));
    let (a, b) = g.edge_points(&g.edges[0]).unwrap();
    assert_eq!(hit(&g, a.mid(b), 4.0), Hit::Edge("a:n->b:n".into()));
    assert_eq!(hit(&g, Point::new(300.0, 300.0), 4.0), Hit::Background);
}

#[test]
fn owner_changes_are_adopted_undoably() {
    let (s, mut input) = start();
    input.graph.nodes.push(Node::new("c", "New", 0.0, 200.0));
    let s2 = hydroxy_statechart::sync_or_keep::<NodeEditorMachine>(&s, &input);
    assert_eq!(s2.graph.nodes.len(), 3);
    assert!(s2.can_undo());
    // Echoing our own change back is a no-op.
    assert_eq!(hydroxy_statechart::sync_or_keep::<NodeEditorMachine>(&s2, &input), s2);
}

#[test]
fn keys_map_to_events() {
    assert_eq!(key_event("z", Modifiers { shift: true, ctrl: true, ..Modifiers::NONE }), Some(NodeEditorEvent::Redo));
    assert_eq!(key_event("Backspace", Modifiers::NONE), Some(NodeEditorEvent::Delete));
    assert_eq!(key_event("q", Modifiers::NONE), None);
}
