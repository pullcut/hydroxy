use dioxus::prelude::*;
use hydroxy_dioxus::node_editor::{Edge, Graph, Node, NodeEditor, Port};

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
                .inputs(vec![Port::new("to", "To", "text"), Port::new("body", "Body", "any")]),
        ],
        vec![
            Edge::new("hook", "event", "pick", "in"),
            Edge::new("pick", "amount", "check", "value"),
            Edge::new("check", "yes", "slack", "when"),
            Edge::new("pick", "email", "mail", "to"),
        ],
    )
}

#[allow(clippy::needless_pass_by_value, reason = "RenderFn hands each node over by value")]
fn body(node: Node) -> Element {
    let text = match node.kind.as_str() {
        "trigger" => "POST /hooks/stripe",
        "condition" => "amount > 500",
        _ => "",
    };
    rsx! { code { style: "font: 500 12px/28px var(--font-mono); color: var(--text-3);", "{text}" } }
}

#[component]
pub fn NodeEditorDemo() -> Element {
    let mut graph = use_signal(payments_flow);
    let g = graph.read();
    let status = format!("{} nodes, {} wires", g.nodes.len(), g.edges.len());
    drop(g);

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "{status}" }
            NodeEditor {
                graph,
                on_change: move |g: Graph| graph.set(g),
                render: body,
                label: "Payment automation",
            }
        }
    }
}
