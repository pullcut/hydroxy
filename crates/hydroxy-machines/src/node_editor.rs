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

use crate::Modifiers;
use crate::canvas::Pointers;
pub use crate::canvas::{Point, Rect, Viewport};
use crate::text::plural;

mod graph;
mod hit;
#[cfg(test)]
mod tests;

pub use graph::*;
pub use hit::*;

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
    pub graph: Arc<Graph>,
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
    pub fn is_dragging(&self) -> bool {
        matches!(self.gesture, Some(Gesture::Drag { .. }))
    }

    /// Whether the view is being panned or pinched.
    #[must_use]
    pub fn is_panning(&self) -> bool {
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

fn port_name(g: &Graph, p: &PortRef) -> String {
    match g.port(p) {
        Some((n, _, port)) => format!("{} {}", n.title, port.label),
        None => p.port.to_string(),
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
        if Arc::ptr_eq(&before, &self.graph) || before == self.graph {
            return;
        }
        self.history.record(before);
        commands.push(NodeEditorCommand::Change((*self.graph).clone()));
    }

    fn prune_selection(&mut self) {
        let g = &self.graph;
        self.selection.nodes.retain(|id| g.node(id).is_some());
        self.selection.edges.retain(|id| g.edges.iter().any(|e| e.id == *id));
    }

    /// Join two ports. An input takes one wire, so an old one into it goes.
    fn connect(&mut self, a: &PortRef, b: &PortRef, drop: Option<&str>, commands: &mut Vec<NodeEditorCommand>) -> bool {
        let before = Arc::clone(&self.graph);
        if let Some(id) = drop {
            Arc::make_mut(&mut self.graph).edges.retain(|e| e.id != id);
        }
        match check(&self.graph, a, b) {
            Ok(edge) => {
                Arc::make_mut(&mut self.graph).edges.retain(|e| e.to != edge.to);
                let text = format!(
                    "Connected {} to {}.",
                    port_name(&self.graph, &edge.from),
                    port_name(&self.graph, &edge.to)
                );
                Arc::make_mut(&mut self.graph).edges.push(edge);
                self.commit(before, commands);
                commands.push(NodeEditorCommand::Announce(text));
                true
            }
            Err(r) => {
                self.graph = before;
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
                    let before = Arc::clone(&self.graph);
                    Arc::make_mut(&mut self.graph).edges.retain(|e| e.id != id);
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
            self.graph = Arc::clone(before);
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
                self.gesture = Some(Gesture::Drag { start: world, lead: id, origins, before: Arc::clone(&self.graph) });
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
                    if let Some(n) = Arc::make_mut(&mut self.graph).nodes.iter_mut().find(|n| n.id == *id) {
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
        let before = Arc::clone(&self.graph);
        let Selection { nodes, edges } = std::mem::take(&mut self.selection);
        let edge_count = self.graph.edges.len();
        Arc::make_mut(&mut self.graph).nodes.retain(|n| !nodes.contains(&n.id));
        Arc::make_mut(&mut self.graph).edges.retain(|e| {
            !edges.contains(&e.id) && !nodes.contains(e.from.node.as_str()) && !nodes.contains(e.to.node.as_str())
        });
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
        let moved = if undo {
            self.history.undo(Arc::make_mut(&mut self.graph))
        } else {
            self.history.redo(Arc::make_mut(&mut self.graph))
        };
        if !moved {
            return;
        }
        self.gesture = None;
        self.prune_selection();
        commands.push(NodeEditorCommand::Change((*self.graph).clone()));
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
            graph: Arc::new(input.graph.clone()),
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
        if *next.graph != input.graph {
            let mut sink = Vec::new();
            next.cancel();
            let before = std::mem::replace(&mut next.graph, Arc::clone(&next.seen));
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
                    return Transition::stay();
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
                let before = Arc::clone(&s.graph);
                let n = s.graph.edges_at(&p).count();
                Arc::make_mut(&mut s.graph).edges.retain(|e| e.from != p && e.to != p);
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
                    let before = Arc::clone(&s.graph);
                    for n in
                        Arc::make_mut(&mut s.graph).nodes.iter_mut().filter(|n| state.selection.nodes.contains(&n.id))
                    {
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
        Transition::update(state, s, commands)
    }
}

/// The event for a key pressed on the canvas or a node, if any. `Some` means: prevent default.
/// `cmd` is Ctrl or Cmd.
#[must_use]
pub fn key_event(key: &str, mods: Modifiers) -> Option<NodeEditorEvent> {
    let (shift, cmd) = (mods.shift, mods.command());
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
