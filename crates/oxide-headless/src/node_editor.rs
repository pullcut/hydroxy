//! Node editor connector. Anatomy: root, toolbar, viewport, background, world,
//! edges, edge, preview, node, header, title, ports, port, port-label, content,
//! marquee, minimap, instructions, live. Every part carries
//! `data-scope="node-editor"` and its `data-part`.
//!
//! The viewport is one focusable `role="application"` element that receives
//! every pointer event; the machine hit-tests ports, nodes and wires from the
//! fixed node layout, so nodes don't need their own pointer handlers. Nodes
//! are tabbable groups (focus selects), ports are buttons (Enter connects).
//! Host content sits in `content`, which keeps its own pointer and key events.

pub use oxide_machines::node_editor::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "node-editor";

/// Read by screen readers on the viewport and every node, through `aria-describedby`.
pub const INSTRUCTIONS: &str = "Tab moves between nodes and their ports. Arrow keys move the selected nodes, or pan when nothing is selected; Shift moves further. On a port, press Enter to start a connection, then Enter on a port of the other side to finish it; Delete removes a port's wires. Delete removes the selection. Control or Command with Z undoes, with Shift and Z redoes, with A selects every node. Plus and minus zoom, 0 resets the zoom and 1 fits everything. Escape cancels.";

/// The instructions' id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// A node's element id.
#[must_use]
pub fn node_id(root: &str, node: &str) -> String {
    format!("{root}-node-{node}")
}

/// Inline style for the viewport: grid dots that pan and zoom with the world,
/// plus the layout constants the CSS reads.
#[must_use]
pub fn viewport_style(v: Viewport, grid: f64) -> String {
    let step = (grid * v.zoom).max(4.0);
    let step = if step < 8.0 { step * 4.0 } else { step };
    format!(
        "--ox-node-header: {HEADER}px; --ox-node-row: {ROW}px; --ox-grid: {step:.2}px; --ox-grid-x: {:.2}px; --ox-grid-y: {:.2}px;",
        v.x.rem_euclid(step),
        v.y.rem_euclid(step)
    )
}

/// Inline style placing a node in the world layer.
#[must_use]
pub fn node_style(n: &Node) -> String {
    format!("transform: translate({:.1}px, {:.1}px); width: {:.1}px; height: {:.1}px;", n.x, n.y, n.width, n.height())
}

/// Inline style for a node's content area.
#[must_use]
pub fn content_style(n: &Node) -> String {
    #[allow(clippy::cast_precision_loss, reason = "port counts are tiny")]
    let top = HEADER + ROW * n.rows() as f64 + 4.0;
    format!("top: {top:.1}px; height: {:.1}px;", n.body)
}

/// Inline style placing a port's row.
#[must_use]
pub fn port_style(index: usize) -> String {
    #[allow(clippy::cast_precision_loss, reason = "port counts are tiny")]
    let top = HEADER + ROW * index as f64;
    format!("top: {top:.1}px;")
}

/// A stable token colour for a port type: `"any"` is neutral, others pick a chart colour.
#[must_use]
pub fn kind_color(kind: &str) -> String {
    if kind == "any" {
        return "var(--text-3)".into();
    }
    let hash = kind.bytes().fold(5381u32, |h, b| h.wrapping_mul(33) ^ u32::from(b));
    format!("var(--chart-{})", hash % 6 + 1)
}

/// `aria-label` for a port.
#[must_use]
pub fn port_label(node: &Node, side: Side, port: &Port, wires: usize) -> String {
    let side = if side == Side::Input { "input" } else { "output" };
    let links = match wires {
        0 => "not connected".to_string(),
        1 => "1 wire".to_string(),
        n => format!("{n} wires"),
    };
    format!("{} {side} {}, {}, {links}", node.title, port.label, port.kind)
}

/// `aria-label` for a node.
#[must_use]
pub fn node_label(n: &Node, selected: bool) -> String {
    if selected { format!("{}, selected", n.title) } else { n.title.clone() }
}

/// Wire colour: the output port's type colour.
#[must_use]
pub fn edge_color(g: &Graph, e: &Edge) -> String {
    g.port(&e.from).map_or_else(|| "var(--text-3)".into(), |(_, _, p)| kind_color(&p.kind))
}

/// Minimap size in pixels.
pub const MINIMAP: (f64, f64) = (180.0, 120.0);

/// The minimap's mapping from world to its own pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Minimap {
    /// World area shown.
    pub world: Rect,
    /// Pixels per world unit.
    pub scale: f64,
}

impl Minimap {
    /// A minimap framing every node and the current view.
    #[must_use]
    pub fn new(state: &NodeEditorState) -> Self {
        let view = state.viewport.visible(state.size);
        let world = state.graph.bounds().map_or(view, |b| b.union(view)).inflate(40.0);
        let scale = (MINIMAP.0 / world.w.max(1.0)).min(MINIMAP.1 / world.h.max(1.0));
        Self { world, scale }
    }

    /// A world rectangle in minimap pixels.
    #[must_use]
    pub fn rect(self, r: Rect) -> Rect {
        let (ox, oy) = self.offset();
        Rect::new((r.x - self.world.x) * self.scale + ox, (r.y - self.world.y) * self.scale + oy, r.w * self.scale, r.h * self.scale)
    }

    /// A minimap pixel in world units.
    #[must_use]
    pub fn to_world(self, p: Point) -> Point {
        let (ox, oy) = self.offset();
        Point::new((p.x - ox) / self.scale + self.world.x, (p.y - oy) / self.scale + self.world.y)
    }

    fn offset(self) -> (f64, f64) {
        ((MINIMAP.0 - self.world.w * self.scale) / 2.0, (MINIMAP.1 - self.world.h * self.scale) / 2.0)
    }

    /// One SVG path drawing every node, so the minimap is a single element however big the graph.
    #[must_use]
    pub fn nodes_path(self, g: &Graph) -> String {
        use std::fmt::Write as _;
        let mut d = String::new();
        for n in &g.nodes {
            let r = self.rect(n.rect());
            let _ = write!(d, "M{:.1} {:.1}h{:.1}v{:.1}h{:.1}Z", r.x, r.y, r.w, r.h, -r.w);
        }
        d
    }
}

/// Inner markup for the canvas toolbars' 24×24 line icons (`node_editor` and `whiteboard`).
/// Unknown names draw nothing.
#[must_use]
pub fn icon(name: &str) -> &'static str {
    match name {
        "minus" => r#"<path d="M5 12h14"/>"#,
        "plus" => r#"<path d="M12 5v14M5 12h14"/>"#,
        "fit" => r#"<path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/>"#,
        "undo" => r#"<path d="M9 14 4 9l5-5"/><path d="M4 9h11a5 5 0 0 1 0 10h-3"/>"#,
        "redo" => r#"<path d="m15 14 5-5-5-5"/><path d="M20 9H9a5 5 0 0 0 0 10h3"/>"#,
        "trash" => r#"<path d="M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3"/>"#,
        "select" => r#"<path d="M6 3l12 9-5.5 1.2L10 19z"/>"#,
        "hand" => r#"<path d="M8 13V6a1.5 1.5 0 0 1 3 0v5M11 11V4.5a1.5 1.5 0 0 1 3 0V11M14 11V6.5a1.5 1.5 0 0 1 3 0V15a6 6 0 0 1-6 6h-.5a6 6 0 0 1-5-2.7L3.8 15a1.5 1.5 0 0 1 2.5-1.7L8 15"/>"#,
        "rect" => r#"<rect x="4" y="6" width="16" height="12" rx="2"/>"#,
        "ellipse" => r#"<ellipse cx="12" cy="12" rx="8.5" ry="6.5"/>"#,
        "arrow" => r#"<path d="M5 19 19 5M10 5h9v9"/>"#,
        "pen" => r#"<path d="M3 17c2.5-3.5 4.5 1.5 7.5-2s3.5-8 6.5-6.5-1 7 4 6.5"/>"#,
        "sticky" => r#"<path d="M5 4h14v10l-6 6H5z"/><path d="M13 20v-6h6"/>"#,
        "text" => r#"<path d="M5 7V4h14v3M12 4v16M9 20h6"/>"#,
        "copy" => r#"<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M4 16V6a2 2 0 0 1 2-2h10"/>"#,
        "forward" => r#"<path d="M12 19V5M6 11l6-6 6 6"/>"#,
        "backward" => r#"<path d="M12 5v14M6 13l6 6 6-6"/>"#,
        "download" => r#"<path d="M12 4v11M7 10l5 5 5-5M5 20h14"/>"#,
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_get_stable_token_colours() {
        assert_eq!(kind_color("number"), kind_color("number"));
        assert!(kind_color("text").starts_with("var(--chart-"));
        assert_eq!(kind_color("any"), "var(--text-3)");
    }

    #[test]
    fn minimap_round_trips() {
        let state = NodeEditorState {
            graph: Graph::new(vec![Node::new("a", "A", 0.0, 0.0), Node::new("b", "B", 800.0, 400.0)], vec![]),
            size: (600.0, 400.0),
            ..NodeEditorState::default()
        };
        let m = Minimap::new(&state);
        let p = Point::new(300.0, 120.0);
        let back = m.to_world(Point::new((p.x - m.world.x) * m.scale + (MINIMAP.0 - m.world.w * m.scale) / 2.0, (p.y - m.world.y) * m.scale + (MINIMAP.1 - m.world.h * m.scale) / 2.0));
        assert!((back.x - p.x).abs() < 1e-6 && (back.y - p.y).abs() < 1e-6);
        assert_eq!(m.nodes_path(&state.graph).matches('Z').count(), 2);
    }

    #[test]
    fn ports_are_named_for_screen_readers() {
        let n = Node::new("a", "Fetch", 0.0, 0.0);
        let p = Port::new("o", "Body", "text");
        assert_eq!(port_label(&n, Side::Output, &p, 2), "Fetch output Body, text, 2 wires");
    }
}
