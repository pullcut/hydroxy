use hydroxy_leptos::button::{Button, ButtonSize, ButtonVariant};
use hydroxy_leptos::node_editor::{Edge, Graph, Node, NodeEditor, Port, PortKind};
use leptos::prelude::*;

fn payments_flow() -> Graph {
    Graph::new(
        vec![
            Node::new("hook", "Stripe webhook", 0.0, 64.0)
                .kind("trigger")
                .outputs(vec![Port::new("event", "Event", "json")])
                .body(28.0),
            Node::new("pick", "Pick fields", 288.0, 48.0)
                .kind("transform")
                .inputs(vec![Port::new("in", "Payload", "json")])
                .outputs(vec![Port::new("email", "Email", "text"), Port::new("amount", "Amount", "number")]),
            Node::new("check", "Over £500?", 576.0, 0.0)
                .kind("condition")
                .inputs(vec![Port::new("value", "Amount", "number")])
                .outputs(vec![Port::new("yes", "Yes", "boolean")])
                .body(28.0),
            Node::new("slack", "Post to #sales", 864.0, 0.0)
                .kind("action")
                .inputs(vec![Port::new("when", "When", "boolean"), Port::new("text", "Message", "text")]),
            Node::new("mail", "Send receipt", 864.0, 176.0)
                .kind("action")
                .inputs(vec![Port::new("to", "To", "text"), Port::new("body", "Body", PortKind::Any)]),
        ],
        vec![
            Edge::new("hook", "event", "pick", "in"),
            Edge::new("pick", "amount", "check", "value"),
            Edge::new("check", "yes", "slack", "when"),
            Edge::new("pick", "email", "mail", "to"),
        ],
    )
}

/// 300 nodes in a chain, to feel how it holds up.
fn big_flow() -> Graph {
    let nodes = (0..300)
        .map(|i| {
            let (col, row) = (f64::from(i % 20), f64::from(i / 20));
            Node::new(format!("n{i}"), format!("Step {}", i + 1), col * 256.0, row * 128.0)
                .kind("transform")
                .inputs(vec![Port::new("in", "In", "json")])
                .outputs(vec![Port::new("out", "Out", "json")])
        })
        .collect();
    let edges = (1..300).map(|i| Edge::new(&format!("n{}", i - 1), "out", &format!("n{i}"), "in")).collect();
    Graph::new(nodes, edges)
}

#[allow(clippy::needless_pass_by_value, reason = "RenderFn hands each node over by value")]
fn body(node: Node) -> impl IntoView {
    let text = match node.kind.as_str() {
        "trigger" => "POST /hooks/stripe",
        "condition" => "amount > 500",
        _ => "",
    };
    view! { <code style="font: 500 12px/28px var(--font-mono); color: var(--text-3);">{text}</code> }
}

pub fn demo() -> AnyView {
    let graph = RwSignal::new(payments_flow());
    let added = StoredValue::new(0_u32);
    let status = move || graph.with(|g| format!("{} nodes, {} wires", g.nodes.len(), g.edges.len()));
    let add_delay = move |_: leptos::ev::MouseEvent| {
        added.update_value(|n| *n += 1);
        let n = added.get_value();
        graph.update(|g| {
            let node = Node::new(format!("delay{n}"), "Wait 10 minutes", 576.0, 208.0 + f64::from(n) * 24.0)
                .kind("delay")
                .inputs(vec![Port::new("in", "In", PortKind::Any)])
                .outputs(vec![Port::new("out", "Out", PortKind::Any)]);
            g.nodes.push(node);
        });
    };

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <div style="display: flex; gap: var(--space-2); flex-wrap: wrap; align-items: center;">
                <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=add_delay>"Add a delay"</Button>
                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on_click=move |_| graph.set(big_flow())>"Load 300 nodes"</Button>
                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on_click=move |_| graph.set(payments_flow())>"Reset"</Button>
                <span style="font: var(--text-label); color: var(--text-3);">{status}</span>
            </div>
            <NodeEditor graph=graph on_change=move |g: Graph| graph.set(g) render=body label="Payment automation" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "node-editor",
    name: "Node editor",
    group: "Data",
    summary: "A flow builder on a pan and zoom canvas. Nodes have typed ports; drag from one to another to wire them, and only matching types connect. Select, box-select, drag, snap, undo and minimap, all with a keyboard route.",
    demo,
    leptos: include_str!("node_editor.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/node_editor.rs"),
    parts: &[
        &["NodeEditor", "root", "Wrapper for the toolbar, canvas and screen reader text"],
        &["", "toolbar", "role=toolbar: zoom out, zoom level, zoom in, fit, undo, redo, delete"],
        &[
            "",
            "viewport",
            "role=application canvas that takes every pointer and key event. Grid dots via --ox-grid*. data-panning, data-connecting",
        ],
        &["", "world", "The layer that pans and zooms, by one CSS transform"],
        &[
            "",
            "edges",
            "<svg> of wires. Each edge is a <path>, coloured by its output's type (--ox-wire); data-selected",
        ],
        &["", "preview", "The dashed wire following the pointer while connecting"],
        &["", "node", "Tabbable role=group with aria-roledescription=node. data-selected, data-dragging, data-kind"],
        &["", "header / title", "Title bar; drag it to move the node"],
        &[
            "",
            "port",
            "<button> per port, data-side input or output, --ox-port colour. data-wired, data-accept while connecting, aria-pressed on the port a keyboard connection starts from",
        ],
        &["", "content", "The host's render output, in the node's body height. Keeps its own pointer and key events"],
        &["", "marquee", "Shift-drag selection box"],
        &["", "minimap", "Overview of every node and the view; click or drag to move the view"],
        &[
            "",
            "instructions / live",
            "Visually hidden keyboard help and a role=status region for connections, deletes and undo",
        ],
    ],
    keys: &[
        &["Tab", "Moves between nodes and their ports. Focusing a node selects it"],
        &[
            "Enter / Space on a port",
            "Starts a connection; on a port of the other side, finishes it. Again on the same port cancels",
        ],
        &["Delete on a port", "Removes that port's wires"],
        &[
            "Arrow keys",
            "Move the selected nodes one grid step, or pan when nothing is selected. Shift moves four steps",
        ],
        &["Delete / Backspace", "Deletes the selected nodes and wires"],
        &["Ctrl/Cmd + Z, Shift + Z, Y", "Undo and redo"],
        &["Ctrl/Cmd + A", "Selects every node"],
        &["+ / − / 0 / 1", "Zoom in, zoom out, back to 100%, fit everything"],
        &["Escape", "Cancels a connection, else clears the selection"],
        &["Space + drag, middle drag", "Pans from anywhere, even over a node"],
        &["Shift + click, Shift + drag", "Adds to the selection, draws a selection box"],
    ],
    props: &[
        &[
            "graph",
            "Signal<Graph>",
            "Required. Graph::new(nodes, edges). Node::new(id, title, x, y).inputs(..).outputs(..).body(h). Port::new(id, label, kind); PortKind::Any matches everything",
        ],
        &[
            "on_change",
            "Callback<Graph>",
            "Called with the graph after each committed change (drop, connect, delete, nudge, undo)",
        ],
        &["render", "RenderFn<Node>", "Renders a node's content area, body() pixels tall"],
        &["label", "String", "Accessible name for the canvas"],
        &["grid", "f64", "Grid spacing. Default 16"],
        &["snap", "bool", "Snap dragged nodes to the grid. Default true"],
        &["minimap", "bool", "Show the minimap. Default true"],
        &["height", "String", "CSS height of the canvas. Default 480px"],
    ],
    events: &[],
};
