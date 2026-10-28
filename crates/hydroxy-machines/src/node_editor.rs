//! Node editor: a flow builder on a pan/zoom canvas. Nodes carry typed input
//! and output ports; wires join an output to an input of a compatible type.
//!
//! Layout is fixed so the machine can hit-test without the DOM: a node is a
//! [`HEADER`] tall title bar, one [`ROW`] per port row (inputs on the left,
//! outputs on the right) and then `body` pixels for host content. Ports sit on
//! the node's left and right edges, centred in their row.
//!
//! Pointer events arrive in screen pixels; the machine finds what is under the
//! pointer, so adapters send every press, move and release from the canvas.

use std::collections::BTreeSet;
use std::sync::Arc;

use hydroxy_state::History;
use hydroxy_statechart::{Machine, Transition};

use crate::canvas::Pointers;
pub use crate::canvas::{Point, Rect, Viewport};

/// Height of a node's title bar.
pub const HEADER: f64 = 36.0;
/// Height of one port row.
pub const ROW: f64 = 28.0;
/// How close, in screen pixels, a dragged wire snaps to a port.
pub const SNAP_RADIUS: f64 = 22.0;

/// Which edge of a node a port sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    /// Left edge: receives one wire.
    Input,
    /// Right edge: sends any number of wires.
    Output,
}

impl Side {
    /// `"input"` or `"output"`, for `data-side`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

/// A typed connection point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    /// Unique within its node and side.
    pub id: String,
    /// Visible name.
    pub label: String,
    /// Value type, such as `"number"`. `"any"` connects to everything.
    pub kind: String,
}

impl Port {
    /// A port called `label` carrying `kind`.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), kind: kind.into() }
    }
}

/// One box on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Unique key.
    pub id: String,
    /// Title bar text.
    pub title: String,
    /// Free-form type for the host's renderer, such as `"http-request"`.
    pub kind: String,
    /// Left edge in world units.
    pub x: f64,
    /// Top edge in world units.
    pub y: f64,
    /// Width in world units.
    pub width: f64,
    /// Height of the host content area under the ports.
    pub body: f64,
    /// Left-edge ports, top to bottom.
    pub inputs: Vec<Port>,
    /// Right-edge ports, top to bottom.
    pub outputs: Vec<Port>,
}

impl Node {
    /// A 200-wide node with no ports and no content area.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            kind: String::new(),
            x,
            y,
            width: 200.0,
            body: 0.0,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Set the host's type key.
    #[must_use]
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    /// Set the input ports.
    #[must_use]
    pub fn inputs(mut self, ports: Vec<Port>) -> Self {
        self.inputs = ports;
        self
    }

    /// Set the output ports.
    #[must_use]
    pub fn outputs(mut self, ports: Vec<Port>) -> Self {
        self.outputs = ports;
        self
    }

    /// Set the width.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Reserve `height` for host content.
    #[must_use]
    pub fn body(mut self, height: f64) -> Self {
        self.body = height;
        self
    }

    /// Port rows: the longer of the two port lists.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.inputs.len().max(self.outputs.len())
    }

    /// Total height.
    #[must_use]
    pub fn height(&self) -> f64 {
        #[allow(clippy::cast_precision_loss, reason = "port counts are tiny")]
        let rows = self.rows() as f64;
        HEADER + ROW * rows + self.body + if rows > 0.0 || self.body > 0.0 { 8.0 } else { 0.0 }
    }

    /// The node's box.
    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height())
    }

    /// The ports on `side`.
    #[must_use]
    pub fn ports(&self, side: Side) -> &[Port] {
        match side {
            Side::Input => &self.inputs,
            Side::Output => &self.outputs,
        }
    }

    /// Where the `index`th port on `side` sits.
    #[must_use]
    pub fn port_point(&self, side: Side, index: usize) -> Point {
        #[allow(clippy::cast_precision_loss, reason = "port counts are tiny")]
        let y = self.y + HEADER + ROW * (index as f64 + 0.5);
        Point::new(if side == Side::Input { self.x } else { self.x + self.width }, y)
    }
}

/// A port on a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortRef {
    /// The node's id.
    pub node: String,
    /// Which edge.
    pub side: Side,
    /// The port's id.
    pub port: String,
}

impl PortRef {
    /// Port `port` on `side` of `node`.
    #[must_use]
    pub fn new(node: impl Into<String>, side: Side, port: impl Into<String>) -> Self {
        Self { node: node.into(), side, port: port.into() }
    }
}

/// A wire from an output to an input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Unique key, derived from the two ends.
    pub id: String,
    /// The output end.
    pub from: PortRef,
    /// The input end.
    pub to: PortRef,
}

impl Edge {
    /// A wire from output `from_port` of `from_node` to input `to_port` of `to_node`.
    #[must_use]
    pub fn new(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        Self {
            id: format!("{from_node}:{from_port}->{to_node}:{to_port}"),
            from: PortRef::new(from_node, Side::Output, from_port),
            to: PortRef::new(to_node, Side::Input, to_port),
        }
    }
}

/// Nodes and the wires between them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Graph {
    /// In paint order: later nodes draw on top.
    pub nodes: Vec<Node>,
    /// Wires.
    pub edges: Vec<Edge>,
}

impl Graph {
    /// A graph of `nodes` joined by `edges`.
    #[must_use]
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Self { nodes, edges }
    }

    /// The node with this id.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// The port and its index.
    #[must_use]
    pub fn port(&self, p: &PortRef) -> Option<(&Node, usize, &Port)> {
        let node = self.node(&p.node)?;
        let (i, port) = node.ports(p.side).iter().enumerate().find(|(_, q)| q.id == p.port)?;
        Some((node, i, port))
    }

    /// Where a port sits in the world.
    #[must_use]
    pub fn port_point(&self, p: &PortRef) -> Option<Point> {
        self.port(p).map(|(n, i, _)| n.port_point(p.side, i))
    }

    /// Both ends of a wire.
    #[must_use]
    pub fn edge_points(&self, e: &Edge) -> Option<(Point, Point)> {
        Some((self.port_point(&e.from)?, self.port_point(&e.to)?))
    }

    /// The box around every node.
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        self.nodes.iter().map(Node::rect).reduce(Rect::union)
    }

    /// Wires touching a port.
    pub fn edges_at<'a>(&'a self, p: &'a PortRef) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| e.from == *p || e.to == *p)
    }
}

/// Whether an output of type `out` may feed an input of type `input`.
#[must_use]
pub fn compatible(out: &str, input: &str) -> bool {
    out == input || out == "any" || input == "any"
}

/// Why a wire was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Both ends are inputs, or both outputs.
    SameSide,
    /// Both ends are on one node.
    SameNode,
    /// The types don't match.
    Types {
        /// The output's type.
        out: String,
        /// The input's type.
        input: String,
    },
    /// That wire exists already.
    Duplicate,
    /// A port is missing.
    Missing,
}

/// Check a wire between two ports, given in either order. `Ok` holds the wire.
///
/// # Errors
/// The [`Refusal`] when the ports can't be joined.
pub fn check(graph: &Graph, a: &PortRef, b: &PortRef) -> Result<Edge, Refusal> {
    if a.side == b.side {
        return Err(Refusal::SameSide);
    }
    if a.node == b.node {
        return Err(Refusal::SameNode);
    }
    let (out, input) = if a.side == Side::Output { (a, b) } else { (b, a) };
    let (Some((_, _, op)), Some((_, _, ip))) = (graph.port(out), graph.port(input)) else {
        return Err(Refusal::Missing);
    };
    if !compatible(&op.kind, &ip.kind) {
        return Err(Refusal::Types { out: op.kind.clone(), input: ip.kind.clone() });
    }
    let edge = Edge::new(&out.node, &out.port, &input.node, &input.port);
    if graph.edges.iter().any(|e| e.id == edge.id) {
        return Err(Refusal::Duplicate);
    }
    Ok(edge)
}

/// Control points for a wire leaving `a` rightwards and entering `b` from the left.
fn wire_controls(a: Point, b: Point) -> (Point, Point) {
    let pull = ((b.x - a.x).abs() / 2.0).max(48.0).min(160.0 + (b.y - a.y).abs() / 4.0);
    (Point::new(a.x + pull, a.y), Point::new(b.x - pull, b.y))
}

/// SVG `d` for a wire from output point `a` to input point `b`.
#[must_use]
pub fn wire_path(a: Point, b: Point) -> String {
    let (c1, c2) = wire_controls(a, b);
    format!("M{:.1} {:.1}C{:.1} {:.1} {:.1} {:.1} {:.1} {:.1}", a.x, a.y, c1.x, c1.y, c2.x, c2.y, b.x, b.y)
}

fn wire_point(a: Point, b: Point, t: f64) -> Point {
    let (c1, c2) = wire_controls(a, b);
    let u = 1.0 - t;
    a.scale(u * u * u).plus(c1.scale(3.0 * u * u * t)).plus(c2.scale(3.0 * u * t * t)).plus(b.scale(t * t * t))
}

/// Distance from `p` to the wire `a`–`b`, sampled.
#[must_use]
pub fn wire_distance(p: Point, a: Point, b: Point) -> f64 {
    const STEPS: u32 = 24;
    let mut prev = a;
    let mut best = f64::INFINITY;
    for i in 1..=STEPS {
        let q = wire_point(a, b, f64::from(i) / f64::from(STEPS));
        best = best.min(crate::canvas::segment_distance(p, prev, q));
        prev = q;
    }
    best
}

/// What is under a world point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hit {
    /// A port.
    Port(PortRef),
    /// A node's body.
    Node(String),
    /// A wire.
    Edge(String),
    /// Empty canvas.
    Background,
}

/// The port nearest `p` within `radius` world units, topmost node first.
#[must_use]
pub fn port_at(graph: &Graph, p: Point, radius: f64, filter: impl Fn(&PortRef, &Port) -> bool) -> Option<PortRef> {
    let mut best: Option<(f64, PortRef)> = None;
    for n in graph.nodes.iter().rev() {
        if !n.rect().inflate(radius).contains(p) {
            continue;
        }
        for side in [Side::Input, Side::Output] {
            for (i, port) in n.ports(side).iter().enumerate() {
                let d = n.port_point(side, i).dist(p);
                let r = PortRef::new(&n.id, side, &port.id);
                if d <= radius && best.as_ref().is_none_or(|b| d < b.0) && filter(&r, port) {
                    best = Some((d, r));
                }
            }
        }
    }
    best.map(|b| b.1)
}

/// What is under world point `p`. `tolerance` is in world units.
#[must_use]
pub fn hit(graph: &Graph, p: Point, tolerance: f64) -> Hit {
    if let Some(port) = port_at(graph, p, tolerance, |_, _| true) {
        return Hit::Port(port);
    }
    if let Some(n) = graph.nodes.iter().rev().find(|n| n.rect().contains(p)) {
        return Hit::Node(n.id.clone());
    }
    graph
        .edges
        .iter()
        .filter_map(|e| graph.edge_points(e).map(|(a, b)| (wire_distance(p, a, b), e)))
        .filter(|(d, _)| *d <= tolerance)
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .map_or(Hit::Background, |(_, e)| Hit::Edge(e.id.clone()))
}

/// Selected nodes and wires.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    /// Node ids.
    pub nodes: BTreeSet<String>,
    /// Edge ids.
    pub edges: BTreeSet<String>,
}

impl Selection {
    /// Nothing selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    fn only_node(id: &str) -> Self {
        Self { nodes: BTreeSet::from([id.to_string()]), edges: BTreeSet::new() }
    }
}

/// A gesture in progress.
#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    /// Dragging the background (or with the middle button) moves the view.
    Pan {
        /// Last pointer position, screen.
        last: Point,
    },
    /// Moving the selected nodes.
    Drag {
        /// Pointer at the start, world.
        start: Point,
        /// The node that was grabbed; it snaps to the grid and the rest follow.
        lead: String,
        /// Each moving node's start position.
        origins: Vec<(String, Point)>,
        /// The graph before the drag, for undo.
        before: Arc<Graph>,
    },
    /// Shift-drag on the background draws a selection box.
    Marquee {
        /// Corner where it started, world.
        start: Point,
        /// The opposite corner, world.
        current: Point,
        /// Selection before the box, kept (Shift adds).
        base: BTreeSet<String>,
    },
    /// Dragging a wire out of a port, or a keyboard connection in progress.
    Connect {
        /// The port the wire starts at.
        anchor: PortRef,
        /// The loose end, world.
        cursor: Point,
        /// A compatible port the loose end has snapped to.
        target: Option<PortRef>,
        /// The wire picked up off an input, hidden while it is carried.
        detached: Option<String>,
        /// Started with Enter on a port.
        keyboard: bool,
    },
    /// Two pointers: zoom and pan together.
    Pinch,
}

/// Props for [`NodeEditorMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEditorInput {
    /// The graph from the owner. When it changes the editor adopts it (undoably).
    pub graph: Graph,
    /// Grid spacing in world units.
    pub grid: f64,
    /// Snap dragged nodes to the grid.
    pub snap: bool,
}

impl Default for NodeEditorInput {
    fn default() -> Self {
        Self { graph: Graph::default(), grid: 16.0, snap: true }
    }
}

/// What [`NodeEditorMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeEditorState {
    /// The graph shown, including a drag in progress.
    pub graph: Graph,
    /// Selected nodes and wires.
    pub selection: Selection,
    /// Pan and zoom.
    pub viewport: Viewport,
    /// Canvas size in screen pixels.
    pub size: (f64, f64),
    /// What the pointer or keyboard is doing.
    pub gesture: Option<Gesture>,
    /// Pointers down on the canvas.
    pub pointers: Pointers,
    /// Undo and redo snapshots of the graph.
    pub history: History<Graph>,
    /// The last graph the owner passed in.
    pub seen: Arc<Graph>,
}

impl NodeEditorState {
    /// The selection box, in world units, while one is drawn.
    #[must_use]
    pub fn marquee(&self) -> Option<Rect> {
        match &self.gesture {
            Some(Gesture::Marquee { start, current, .. }) => Some(Rect::from_corners(*start, *current)),
            _ => None,
        }
    }

    /// The loose wire being drawn: its output end and input end, world.
    #[must_use]
    pub fn preview_wire(&self) -> Option<(Point, Point)> {
        let Some(Gesture::Connect { anchor, cursor, target, keyboard: false, .. }) = &self.gesture else { return None };
        let a = self.graph.port_point(anchor)?;
        let b = target.as_ref().and_then(|t| self.graph.port_point(t)).unwrap_or(*cursor);
        Some(if anchor.side == Side::Output { (a, b) } else { (b, a) })
    }

    /// The port a connection starts from.
    #[must_use]
    pub fn anchor(&self) -> Option<&PortRef> {
        match &self.gesture {
            Some(Gesture::Connect { anchor, .. }) => Some(anchor),
            _ => None,
        }
    }

    /// While connecting: whether `p` would accept the wire. `None` when not connecting.
    #[must_use]
    pub fn accepts(&self, p: &PortRef) -> Option<bool> {
        let anchor = self.anchor()?;
        Some(p == anchor || check(&self.graph, anchor, p).is_ok())
    }

    /// The wire hidden while it is carried off its input.
    #[must_use]
    pub fn detached(&self) -> Option<&str> {
        match &self.gesture {
            Some(Gesture::Connect { detached, .. }) => detached.as_deref(),
            _ => None,
        }
    }

    /// Whether nodes are being dragged.
    #[must_use]
    pub fn dragging(&self) -> bool {
        matches!(self.gesture, Some(Gesture::Drag { .. }))
    }

    /// Whether the view is being panned or pinched.
    #[must_use]
    pub fn panning(&self) -> bool {
        matches!(self.gesture, Some(Gesture::Pan { .. } | Gesture::Pinch))
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
}

/// Inputs to [`NodeEditorMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum NodeEditorEvent {
    /// The canvas element's size, in screen pixels.
    Resize(f64, f64),
    /// A pointer pressed on the canvas, at a screen point.
    PointerDown {
        /// `pointerId`.
        id: i32,
        /// Screen point.
        at: Point,
        /// Shift held: add to the selection, or draw a box on the background.
        shift: bool,
        /// Middle button or Space held: pan whatever is under the pointer.
        pan: bool,
    },
    /// A pointer moved.
    PointerMove {
        /// `pointerId`.
        id: i32,
        /// Screen point.
        at: Point,
    },
    /// A pointer lifted.
    PointerUp {
        /// `pointerId`.
        id: i32,
        /// Screen point.
        at: Point,
    },
    /// The browser took the pointer away: undo the gesture.
    PointerCancel(i32),
    /// Zoom by `factor` keeping screen point `at` still.
    Zoom {
        /// Screen point.
        at: Point,
        /// Multiplier.
        factor: f64,
    },
    /// Zoom in a step around the centre.
    ZoomIn,
    /// Zoom out a step around the centre.
    ZoomOut,
    /// Back to 100% around the centre.
    ZoomReset,
    /// Frame every node.
    Fit,
    /// Centre the view on a world point (the minimap).
    CenterOn(Point),
    /// Move the view by screen pixels.
    Pan(f64, f64),
    /// A node took keyboard focus: select it unless it already is.
    Focus(String),
    /// Enter or Space on a port: start a keyboard connection, finish one, or cancel it.
    PortKey(PortRef),
    /// Delete on a port: remove its wires.
    Disconnect(PortRef),
    /// Arrow keys: move the selection (or the view when nothing is selected).
    Nudge {
        /// Steps right.
        dx: i32,
        /// Steps down.
        dy: i32,
        /// Shift held: four grid steps.
        big: bool,
    },
    /// Delete the selection.
    Delete,
    /// Select every node.
    SelectAll,
    /// Undo the last change.
    Undo,
    /// Redo the last undo.
    Redo,
    /// Escape: drop a connection in progress, else clear the selection.
    Cancel,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeEditorCommand {
    /// The graph changed. Fires on commit, not on every drag frame.
    Change(Graph),
    /// Say this in the live region.
    Announce(String),
}

/// Graph, selection, viewport and history for a node editor.
#[derive(Debug, Clone, Copy)]
pub struct NodeEditorMachine;

type Step = Transition<NodeEditorState, NodeEditorCommand>;

fn plural(n: usize, one: &str) -> String {
    if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") }
}

fn port_name(g: &Graph, p: &PortRef) -> String {
    match g.port(p) {
        Some((n, _, port)) => format!("{} {}", n.title, port.label),
        None => p.port.clone(),
    }
}

fn refusal_text(r: &Refusal) -> String {
    match r {
        Refusal::SameSide => "Connect an output to an input.".into(),
        Refusal::SameNode => "A node can't connect to itself.".into(),
        Refusal::Types { out, input } => format!("Can't connect {out} to {input}."),
        Refusal::Duplicate => "Those ports are already connected.".into(),
        Refusal::Missing => "That port no longer exists.".into(),
    }
}

fn snap(v: f64, grid: f64) -> f64 {
    if grid > 0.0 { (v / grid).round() * grid } else { v }
}

impl NodeEditorState {
    fn center(&self) -> Point {
        Point::new(self.size.0 / 2.0, self.size.1 / 2.0)
    }

    /// Record the current graph for undo and emit the new one.
    fn commit(&mut self, before: Arc<Graph>, commands: &mut Vec<NodeEditorCommand>) {
        if *before == self.graph {
            return;
        }
        self.history.record(before);
        commands.push(NodeEditorCommand::Change(self.graph.clone()));
    }

    fn prune_selection(&mut self) {
        let g = &self.graph;
        self.selection.nodes.retain(|id| g.node(id).is_some());
        self.selection.edges.retain(|id| g.edges.iter().any(|e| e.id == *id));
    }

    /// Join two ports. An input takes one wire, so an old one into it goes.
    fn connect(&mut self, a: &PortRef, b: &PortRef, drop: Option<&str>, commands: &mut Vec<NodeEditorCommand>) -> bool {
        let before = Arc::new(self.graph.clone());
        if let Some(id) = drop {
            self.graph.edges.retain(|e| e.id != id);
        }
        match check(&self.graph, a, b) {
            Ok(edge) => {
                self.graph.edges.retain(|e| e.to != edge.to);
                let text = format!(
                    "Connected {} to {}.",
                    port_name(&self.graph, &edge.from),
                    port_name(&self.graph, &edge.to)
                );
                self.graph.edges.push(edge);
                self.commit(before, commands);
                commands.push(NodeEditorCommand::Announce(text));
                true
            }
            Err(r) => {
                self.graph = (*before).clone();
                commands.push(NodeEditorCommand::Announce(refusal_text(&r)));
                false
            }
        }
    }

    /// Finish whatever gesture is running as a drop.
    fn finish(&mut self, at: Point, commands: &mut Vec<NodeEditorCommand>) {
        match self.gesture.take() {
            Some(Gesture::Drag { before, .. }) => self.commit(before, commands),
            Some(Gesture::Connect { anchor, target, detached, keyboard: false, .. }) => {
                let world = self.viewport.to_world(at);
                let over = target.or_else(|| {
                    port_at(&self.graph, world, SNAP_RADIUS / self.viewport.zoom, |p, _| p.side != anchor.side)
                });
                if let Some(t) = over {
                    self.connect(&anchor, &t, detached.as_deref(), commands);
                } else if let Some(id) = detached {
                    let before = Arc::new(self.graph.clone());
                    self.graph.edges.retain(|e| e.id != id);
                    self.prune_selection();
                    self.commit(before, commands);
                    commands.push(NodeEditorCommand::Announce("Wire removed.".into()));
                }
            }
            other @ Some(Gesture::Connect { keyboard: true, .. }) => self.gesture = other,
            _ => {}
        }
    }

    fn cancel(&mut self) {
        if let Some(Gesture::Drag { before, .. }) = &self.gesture {
            self.graph = (**before).clone();
        }
        self.gesture = None;
    }

    fn pointer_down(&mut self, at: Point, shift: bool, pan: bool) {
        let world = self.viewport.to_world(at);
        if pan {
            self.gesture = Some(Gesture::Pan { last: at });
            return;
        }
        match hit(&self.graph, world, 8.0 / self.viewport.zoom) {
            Hit::Port(p) => {
                // Pulling on a connected input carries its wire away from the output end.
                let carried = self
                    .graph
                    .edges_at(&p)
                    .next()
                    .filter(|_| p.side == Side::Input)
                    .map(|e| (e.from.clone(), e.id.clone()));
                let (anchor, detached) = match carried {
                    Some((from, id)) => (from, Some(id)),
                    None => (p, None),
                };
                self.gesture =
                    Some(Gesture::Connect { anchor, cursor: world, target: None, detached, keyboard: false });
            }
            Hit::Node(id) => {
                if shift {
                    if !self.selection.nodes.remove(&id) {
                        self.selection.nodes.insert(id.clone());
                    }
                } else if !self.selection.nodes.contains(&id) {
                    self.selection = Selection::only_node(&id);
                }
                if !self.selection.nodes.contains(&id) {
                    return;
                }
                let origins = self
                    .graph
                    .nodes
                    .iter()
                    .filter(|n| self.selection.nodes.contains(&n.id))
                    .map(|n| (n.id.clone(), Point::new(n.x, n.y)))
                    .collect();
                self.gesture =
                    Some(Gesture::Drag { start: world, lead: id, origins, before: Arc::new(self.graph.clone()) });
            }
            Hit::Edge(id) => {
                if shift {
                    if !self.selection.edges.remove(&id) {
                        self.selection.edges.insert(id);
                    }
                } else {
                    self.selection = Selection { nodes: BTreeSet::new(), edges: BTreeSet::from([id]) };
                }
            }
            Hit::Background if shift => {
                self.gesture =
                    Some(Gesture::Marquee { start: world, current: world, base: self.selection.nodes.clone() });
            }
            Hit::Background => {
                if !self.selection.is_empty() {
                    self.selection = Selection::default();
                }
                self.gesture = Some(Gesture::Pan { last: at });
            }
        }
    }

    fn pointer_move(&mut self, at: Point, grid: f64) {
        let world = self.viewport.to_world(at);
        let zoom = self.viewport.zoom;
        match &mut self.gesture {
            Some(Gesture::Pan { last }) => {
                let d = at.minus(*last);
                *last = at;
                self.viewport = self.viewport.pan_by(d.x, d.y);
            }
            Some(Gesture::Drag { start, lead, origins, .. }) => {
                let raw = world.minus(*start);
                let delta = origins
                    .iter()
                    .find(|o| o.0 == *lead)
                    .map_or(raw, |(_, o)| Point::new(snap(o.x + raw.x, grid) - o.x, snap(o.y + raw.y, grid) - o.y));
                for (id, o) in origins.iter() {
                    if let Some(n) = self.graph.nodes.iter_mut().find(|n| n.id == *id) {
                        n.x = o.x + delta.x;
                        n.y = o.y + delta.y;
                    }
                }
            }
            Some(Gesture::Marquee { start, current, base }) => {
                *current = world;
                let r = Rect::from_corners(*start, world);
                let mut nodes = base.clone();
                nodes.extend(self.graph.nodes.iter().filter(|n| n.rect().intersects(r)).map(|n| n.id.clone()));
                self.selection.nodes = nodes;
            }
            Some(Gesture::Connect { anchor, cursor, target, keyboard: false, .. }) => {
                *cursor = world;
                let g = &self.graph;
                *target =
                    port_at(g, world, SNAP_RADIUS / zoom, |p, _| p.side != anchor.side && check(g, anchor, p).is_ok());
            }
            _ => {}
        }
    }

    fn delete(&mut self, commands: &mut Vec<NodeEditorCommand>) {
        if self.selection.is_empty() {
            return;
        }
        let before = Arc::new(self.graph.clone());
        let Selection { nodes, edges } = std::mem::take(&mut self.selection);
        let edge_count = self.graph.edges.len();
        self.graph.nodes.retain(|n| !nodes.contains(&n.id));
        self.graph
            .edges
            .retain(|e| !edges.contains(&e.id) && !nodes.contains(&e.from.node) && !nodes.contains(&e.to.node));
        let gone_edges = edge_count - self.graph.edges.len();
        self.commit(before, commands);
        let mut parts = Vec::new();
        if !nodes.is_empty() {
            parts.push(plural(nodes.len(), "node"));
        }
        if gone_edges > 0 {
            parts.push(plural(gone_edges, "wire"));
        }
        commands.push(NodeEditorCommand::Announce(format!("Deleted {}.", parts.join(" and "))));
    }

    fn restore(&mut self, undo: bool, commands: &mut Vec<NodeEditorCommand>) {
        let moved = if undo { self.history.undo(&mut self.graph) } else { self.history.redo(&mut self.graph) };
        if !moved {
            return;
        }
        self.gesture = None;
        self.prune_selection();
        commands.push(NodeEditorCommand::Change(self.graph.clone()));
        commands.push(NodeEditorCommand::Announce(if undo { "Undone." } else { "Redone." }.into()));
    }

    fn port_key(&mut self, p: PortRef, commands: &mut Vec<NodeEditorCommand>) {
        match self.gesture.take() {
            Some(Gesture::Connect { anchor, keyboard: true, .. }) if anchor == p => {
                commands.push(NodeEditorCommand::Announce("Connection cancelled.".into()));
            }
            Some(Gesture::Connect { anchor, keyboard: true, .. }) => {
                self.connect(&anchor, &p, None, commands);
            }
            _ => {
                let cursor = self.graph.port_point(&p).unwrap_or_default();
                let text = format!(
                    "Connecting from {}. Move to an {} port and press Enter, or press Escape to cancel.",
                    port_name(&self.graph, &p),
                    if p.side == Side::Output { "input" } else { "output" }
                );
                self.gesture =
                    Some(Gesture::Connect { anchor: p, cursor, target: None, detached: None, keyboard: true });
                commands.push(NodeEditorCommand::Announce(text));
            }
        }
    }
}

impl Machine for NodeEditorMachine {
    type Input = NodeEditorInput;
    type State = NodeEditorState;
    type Event = NodeEditorEvent;
    type Command = NodeEditorCommand;

    fn initial(input: &NodeEditorInput) -> NodeEditorState {
        NodeEditorState {
            graph: input.graph.clone(),
            seen: Arc::new(input.graph.clone()),
            ..NodeEditorState::default()
        }
    }

    fn sync(state: &NodeEditorState, input: &NodeEditorInput) -> Option<NodeEditorState> {
        if *state.seen == input.graph {
            return None;
        }
        let mut next = state.clone();
        next.seen = Arc::new(input.graph.clone());
        if next.graph != input.graph {
            let mut sink = Vec::new();
            next.cancel();
            let before = Arc::new(std::mem::replace(&mut next.graph, input.graph.clone()));
            next.commit(before, &mut sink);
            next.prune_selection();
        }
        hydroxy_statechart::changed(state, next)
    }

    #[allow(clippy::too_many_lines, reason = "one match over every event")]
    fn transition(state: &NodeEditorState, event: NodeEditorEvent, input: &NodeEditorInput) -> Step {
        let mut s = state.clone();
        let mut commands = Vec::new();
        let grid = if input.snap { input.grid } else { 0.0 };
        match event {
            NodeEditorEvent::Resize(w, h) => s.size = (w, h),
            NodeEditorEvent::PointerDown { id, at, shift, pan } => {
                if s.pointers.down(id, at) >= 2 {
                    s.cancel();
                    s.gesture = Some(Gesture::Pinch);
                } else {
                    if matches!(s.gesture, Some(Gesture::Connect { keyboard: true, .. })) {
                        s.gesture = None;
                    }
                    s.pointer_down(at, shift, pan);
                }
            }
            NodeEditorEvent::PointerMove { id, at } => {
                if s.gesture == Some(Gesture::Pinch) {
                    if let Some(v) = s.pointers.pinch(id, at, s.viewport) {
                        s.viewport = v;
                    }
                } else if s.pointers.len() == 1 {
                    s.pointers.down(id, at);
                    s.pointer_move(at, grid);
                } else {
                    return Transition::to(s);
                }
            }
            NodeEditorEvent::PointerUp { id, at } => {
                let left = s.pointers.up(id);
                if s.gesture == Some(Gesture::Pinch) {
                    if left == 0 {
                        s.gesture = None;
                    }
                } else {
                    s.finish(at, &mut commands);
                }
            }
            NodeEditorEvent::PointerCancel(id) => {
                s.pointers.up(id);
                s.cancel();
            }
            NodeEditorEvent::Zoom { at, factor } => s.viewport = s.viewport.zoom_at(at, factor),
            NodeEditorEvent::ZoomIn => s.viewport = s.viewport.zoom_at(s.center(), 1.25),
            NodeEditorEvent::ZoomOut => s.viewport = s.viewport.zoom_at(s.center(), 0.8),
            NodeEditorEvent::ZoomReset => s.viewport = s.viewport.zoom_at(s.center(), 1.0 / s.viewport.zoom),
            NodeEditorEvent::Fit => {
                if let Some(b) = s.graph.bounds() {
                    s.viewport = Viewport::fit(b, s.size, 48.0);
                }
            }
            NodeEditorEvent::CenterOn(p) => s.viewport = s.viewport.center_on(p, s.size),
            NodeEditorEvent::Pan(dx, dy) => s.viewport = s.viewport.pan_by(dx, dy),
            NodeEditorEvent::Focus(id) => {
                if !s.selection.nodes.contains(&id) && s.graph.node(&id).is_some() {
                    s.selection = Selection::only_node(&id);
                }
            }
            NodeEditorEvent::PortKey(p) => s.port_key(p, &mut commands),
            NodeEditorEvent::Disconnect(p) => {
                let before = Arc::new(s.graph.clone());
                let n = s.graph.edges_at(&p).count();
                s.graph.edges.retain(|e| e.from != p && e.to != p);
                if n > 0 {
                    s.prune_selection();
                    s.commit(before, &mut commands);
                    commands.push(NodeEditorCommand::Announce(format!("Removed {}.", plural(n, "wire"))));
                }
            }
            NodeEditorEvent::Nudge { dx, dy, big } => {
                let step = if grid > 0.0 { grid } else { 8.0 } * if big { 4.0 } else { 1.0 };
                if s.selection.nodes.is_empty() {
                    s.viewport = s.viewport.pan_by(-f64::from(dx) * 40.0, -f64::from(dy) * 40.0);
                } else {
                    let before = Arc::new(s.graph.clone());
                    for n in s.graph.nodes.iter_mut().filter(|n| state.selection.nodes.contains(&n.id)) {
                        n.x = snap(n.x, grid) + f64::from(dx) * step;
                        n.y = snap(n.y, grid) + f64::from(dy) * step;
                    }
                    s.commit(before, &mut commands);
                }
            }
            NodeEditorEvent::Delete => s.delete(&mut commands),
            NodeEditorEvent::SelectAll => {
                s.selection =
                    Selection { nodes: s.graph.nodes.iter().map(|n| n.id.clone()).collect(), edges: BTreeSet::new() };
                commands.push(NodeEditorCommand::Announce(format!(
                    "Selected {}.",
                    plural(s.selection.nodes.len(), "node")
                )));
            }
            NodeEditorEvent::Undo => s.restore(true, &mut commands),
            NodeEditorEvent::Redo => s.restore(false, &mut commands),
            NodeEditorEvent::Cancel => match s.gesture {
                Some(Gesture::Connect { keyboard: true, .. }) => {
                    s.gesture = None;
                    commands.push(NodeEditorCommand::Announce("Connection cancelled.".into()));
                }
                Some(_) => s.cancel(),
                None => s.selection = Selection::default(),
            },
        }
        Transition::with(s, commands)
    }
}

/// The event for a key pressed on the canvas or a node, if any. `Some` means: prevent default.
/// `cmd` is Ctrl or Cmd.
#[must_use]
pub fn key_event(key: &str, shift: bool, cmd: bool) -> Option<NodeEditorEvent> {
    let nudge = |dx, dy| Some(NodeEditorEvent::Nudge { dx, dy, big: shift });
    match (key, cmd) {
        ("z" | "Z", true) if shift => Some(NodeEditorEvent::Redo),
        ("z" | "Z", true) => Some(NodeEditorEvent::Undo),
        ("y" | "Y", true) => Some(NodeEditorEvent::Redo),
        ("a" | "A", true) => Some(NodeEditorEvent::SelectAll),
        ("Delete" | "Backspace", false) => Some(NodeEditorEvent::Delete),
        ("Escape", _) => Some(NodeEditorEvent::Cancel),
        ("ArrowLeft", false) => nudge(-1, 0),
        ("ArrowRight", false) => nudge(1, 0),
        ("ArrowUp", false) => nudge(0, -1),
        ("ArrowDown", false) => nudge(0, 1),
        ("+" | "=", false) => Some(NodeEditorEvent::ZoomIn),
        ("-" | "_", false) => Some(NodeEditorEvent::ZoomOut),
        ("0", false) => Some(NodeEditorEvent::ZoomReset),
        ("1", false) => Some(NodeEditorEvent::Fit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::close;

    fn graph() -> Graph {
        Graph::new(
            vec![
                Node::new("a", "Source", 0.0, 0.0)
                    .outputs(vec![Port::new("n", "Number", "number"), Port::new("t", "Text", "text")]),
                Node::new("b", "Sink", 400.0, 0.0)
                    .inputs(vec![Port::new("n", "Number", "number"), Port::new("x", "Any", "any")]),
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
            let t = NodeEditorMachine::transition(&s, e, input).resolve(&s);
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
            mid.gesture.as_ref().and_then(|g| if let Gesture::Connect { target, .. } = g {
                target.clone()
            } else {
                None
            }),
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
        s.graph.edges.push(Edge::new("a", "n", "b", "n"));
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
        let (n, _) = run(
            &k,
            &input,
            vec![NodeEditorEvent::Focus("b".into()), NodeEditorEvent::Nudge { dx: 1, dy: 0, big: true }],
        );
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
        assert_eq!(key_event("z", true, true), Some(NodeEditorEvent::Redo));
        assert_eq!(key_event("Backspace", false, false), Some(NodeEditorEvent::Delete));
        assert_eq!(key_event("q", false, false), None);
    }
}
