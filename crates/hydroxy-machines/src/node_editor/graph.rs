//! Graph model: nodes, ports, wires and the rules for joining them.

use crate::canvas::{Point, Rect};

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

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::SameSide => f.write_str("both ends are inputs, or both outputs"),
            Refusal::SameNode => f.write_str("both ends are on one node"),
            Refusal::Types { out, input } => write!(f, "can't connect {out} to {input}"),
            Refusal::Duplicate => f.write_str("that wire exists already"),
            Refusal::Missing => f.write_str("a port is missing"),
        }
    }
}

impl std::error::Error for Refusal {}

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
