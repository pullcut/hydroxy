//! `NodeEditor`: a flow builder. Nodes with typed ports on a pan/zoom
//! canvas; drag from a port to wire it up, drag nodes to arrange them, and
//! do all of it from the keyboard too. Node content comes from the host.

use std::collections::BTreeMap;
use std::rc::Rc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use oxide_headless::node_editor::{
    self as h, NodeEditorCommand, NodeEditorEvent, NodeEditorInput, NodeEditorMachine, Point,
};
use oxide_statechart::Machine;

pub use oxide_headless::node_editor::{Edge, Graph, Node, Port, PortRef, Side};

/// One node to draw: the node, selected, dragging, wire count and drop acceptance per port, and the pressed port.
type BoxRow = (Node, bool, bool, Vec<usize>, Vec<Option<bool>>, Option<PortRef>);

/// Inner markup for a toolbar icon.
fn icon(name: &'static str) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            dangerous_inner_html: h::icon(name),
        }
    }
}

/// Capture a pointer on the element with `id`, so moves keep arriving outside it.
pub(crate) fn capture(id: &str, pointer: i32) {
    let _ = document::eval(&format!("try {{ document.getElementById({id:?})?.setPointerCapture({pointer}); }} catch (_) {{}}"));
}

/// A flow builder canvas.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn NodeEditor(
    /// The graph. When it changes the editor adopts it; pair with `on_change` to keep it in step.
    graph: ReadSignal<Graph>,
    /// Called with the whole graph after every committed change.
    on_change: Option<EventHandler<Graph>>,
    /// Renders a node's content area (its `body` height). Gets the node.
    render: Option<Callback<Node, Element>>,
    /// Accessible name for the canvas.
    #[props(default = "Flow editor".to_string())]
    label: String,
    /// Grid spacing in world units.
    #[props(default = 16.0)]
    grid: f64,
    /// Snap dragged nodes to the grid.
    #[props(default = true)]
    snap: bool,
    /// Show the minimap.
    #[props(default = true)]
    minimap: bool,
    /// CSS height of the canvas.
    #[props(default = "480px".to_string())]
    height: String,
) -> Element {
    let input = use_memo(move || NodeEditorInput { graph: graph(), grid, snap });
    let mut state = use_signal(|| NodeEditorMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = NodeEditorMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let mut live = use_signal(String::new);
    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let vp_id = format!("{root}-viewport");
    let mini_id = format!("{root}-minimap");
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut origin = use_signal(|| (0.0_f64, 0.0_f64));
    let mut space = use_signal(|| false);
    let mut fitted = use_signal(|| false);
    let mut mini_drag = use_signal(|| false);

    let send = use_callback(move |event: NodeEditorEvent| {
        let step = NodeEditorMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                NodeEditorCommand::Change(g) => {
                    if let Some(handler) = on_change {
                        handler.call(g);
                    }
                }
                NodeEditorCommand::Announce(text) => live.set(text),
            }
        }
    });
    let refresh = move || {
        if let Some(el) = mounted() {
            spawn(async move {
                if let Ok(r) = el.get_client_rect().await {
                    origin.set((r.origin.x, r.origin.y));
                }
            });
        }
    };

    let s = state.read().clone();
    let v = s.viewport;
    let hidden = s.detached().map(str::to_string);
    let edges: Vec<(String, String, String, bool)> = s
        .graph
        .edges
        .iter()
        .filter(|e| Some(&e.id) != hidden.as_ref())
        .filter_map(|e| {
            let (a, b) = s.graph.edge_points(e)?;
            Some((e.id.clone(), h::wire_path(a, b), h::edge_color(&s.graph, e), s.selection.edges.contains(&e.id)))
        })
        .collect();
    let mut wired: BTreeMap<&PortRef, usize> = BTreeMap::new();
    for e in &s.graph.edges {
        *wired.entry(&e.from).or_insert(0) += 1;
        *wired.entry(&e.to).or_insert(0) += 1;
    }
    let anchor = s.anchor().cloned();
    let preview = s.preview_wire().map(|(a, b)| h::wire_path(a, b));
    let marquee = s.marquee().map(|r| {
        let a = v.to_screen(Point::new(r.x, r.y));
        format!("left: {:.1}px; top: {:.1}px; width: {:.1}px; height: {:.1}px;", a.x, a.y, r.w * v.zoom, r.h * v.zoom)
    });
    let mini = h::Minimap::new(&s);
    let mini_view = mini.rect(v.visible(s.size));
    let mini_nodes = mini.nodes_path(&s.graph);
    let boxes: Vec<BoxRow> = s
        .graph
        .nodes
        .iter()
        .map(|n| {
            let refs = n
                .inputs
                .iter()
                .map(|p| PortRef::new(&n.id, Side::Input, &p.id))
                .chain(n.outputs.iter().map(|p| PortRef::new(&n.id, Side::Output, &p.id)));
            let (wires, accept): (Vec<usize>, Vec<Option<bool>>) =
                refs.map(|r| (wired.get(&r).copied().unwrap_or(0), s.accepts(&r))).unzip();
            let selected = s.selection.nodes.contains(&n.id);
            (n.clone(), selected, selected && s.dragging(), wires, accept, anchor.clone().filter(|a| a.node == n.id))
        })
        .collect();
    let (panning, connecting, percent) = (s.panning(), anchor.is_some(), v.percent());
    let (can_undo, can_redo, empty) = (s.can_undo(), s.can_redo(), s.selection.is_empty());
    let view_style = format!("height: {height}; {}", h::viewport_style(v, grid));
    let world_style = v.css();
    let mini_rect = [mini_view.x, mini_view.y, mini_view.w, mini_view.h].map(|n| format!("{n:.1}"));
    let mini_size = (h::MINIMAP.0.to_string(), h::MINIMAP.1.to_string());
    let instructions = h::instructions_id(&root);
    let (down_id, mini_down) = (vp_id.clone(), mini_id.clone());

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: "{instructions}", {h::INSTRUCTIONS} }
            div { "data-scope": h::SCOPE, "data-part": "toolbar", role: "toolbar", "aria-label": "Canvas",
                button { r#type: "button", "data-scope": h::SCOPE, "data-part": "action", "aria-label": "Zoom out", onclick: move |_| send.call(NodeEditorEvent::ZoomOut), {icon("minus")} }
                span { "data-scope": h::SCOPE, "data-part": "zoom", "aria-live": "polite", "{percent}%" }
                button { r#type: "button", "data-scope": h::SCOPE, "data-part": "action", "aria-label": "Zoom in", onclick: move |_| send.call(NodeEditorEvent::ZoomIn), {icon("plus")} }
                button { r#type: "button", "data-scope": h::SCOPE, "data-part": "action", "aria-label": "Zoom to fit", onclick: move |_| send.call(NodeEditorEvent::Fit), {icon("fit")} }
                span { "data-scope": h::SCOPE, "data-part": "separator" }
                button { r#type: "button", "data-scope": h::SCOPE, "data-part": "action", "aria-label": "Undo", disabled: !can_undo, onclick: move |_| send.call(NodeEditorEvent::Undo), {icon("undo")} }
                button { r#type: "button", "data-scope": h::SCOPE, "data-part": "action", "aria-label": "Redo", disabled: !can_redo, onclick: move |_| send.call(NodeEditorEvent::Redo), {icon("redo")} }
                button { r#type: "button", "data-scope": h::SCOPE, "data-part": "action", "aria-label": "Delete selection", disabled: empty, onclick: move |_| send.call(NodeEditorEvent::Delete), {icon("trash")} }
            }
            div {
                id: "{vp_id}",
                tabindex: "0",
                role: "application",
                "aria-roledescription": "node editor",
                "aria-label": label,
                "aria-describedby": "{instructions}",
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                "data-panning": panning.then_some(""),
                "data-connecting": connecting.then_some(""),
                style: "{view_style}",
                onmounted: move |e| {
                    mounted.set(Some(e.data()));
                    refresh();
                },
                onresize: move |e| {
                    if let Ok(size) = e.data().get_border_box_size() {
                        send.call(NodeEditorEvent::Resize(size.width, size.height));
                        if !fitted() {
                            fitted.set(true);
                            send.call(NodeEditorEvent::Fit);
                        }
                    }
                },
                onpointerenter: move |_| refresh(),
                onpointerdown: move |e| {
                    let button = e.trigger_button();
                    if !matches!(button, None | Some(MouseButton::Primary | MouseButton::Auxiliary)) {
                        return;
                    }
                    e.prevent_default();
                    let (id, c, shift) = (e.pointer_id(), e.client_coordinates(), e.modifiers().shift());
                    let pan = button == Some(MouseButton::Auxiliary) || space();
                    capture(&down_id, id);
                    let el = mounted();
                    spawn(async move {
                        if let Some(el) = el {
                            let _ = el.set_focus(true).await;
                            if let Ok(r) = el.get_client_rect().await {
                                origin.set((r.origin.x, r.origin.y));
                            }
                        }
                        let o = origin();
                        send.call(NodeEditorEvent::PointerDown { id, at: Point::new(c.x - o.0, c.y - o.1), shift, pan });
                    });
                },
                onpointermove: move |e| {
                    if !state.peek().pointers.is_empty() {
                        let (c, o) = (e.client_coordinates(), origin());
                        send.call(NodeEditorEvent::PointerMove { id: e.pointer_id(), at: Point::new(c.x - o.0, c.y - o.1) });
                    }
                },
                onpointerup: move |e| {
                    let (c, o) = (e.client_coordinates(), origin());
                    send.call(NodeEditorEvent::PointerUp { id: e.pointer_id(), at: Point::new(c.x - o.0, c.y - o.1) });
                },
                onpointercancel: move |e| send.call(NodeEditorEvent::PointerCancel(e.pointer_id())),
                onwheel: move |e| {
                    e.prevent_default();
                    let (c, o) = (e.client_coordinates(), origin());
                    let factor = oxide_charts::zoom::wheel_factor(e.data().delta().strip_units().y);
                    send.call(NodeEditorEvent::Zoom { at: Point::new(c.x - o.0, c.y - o.1), factor });
                },
                onkeydown: move |e| {
                    let key = e.key().to_string();
                    if key == " " {
                        e.prevent_default();
                        space.set(true);
                        return;
                    }
                    let m = e.modifiers();
                    if let Some(event) = h::key_event(&key, m.shift(), m.ctrl() || m.meta()) {
                        e.prevent_default();
                        send.call(event);
                    }
                },
                onkeyup: move |e| {
                    if e.key().to_string() == " " {
                        space.set(false);
                    }
                },
                div { "data-scope": h::SCOPE, "data-part": "world", style: "{world_style}",
                    svg { "data-scope": h::SCOPE, "data-part": "edges", "aria-hidden": "true",
                        for (id , d , colour , selected) in edges {
                            path {
                                key: "{id}",
                                "data-scope": h::SCOPE,
                                "data-part": "edge",
                                "data-selected": selected.then_some(""),
                                d: "{d}",
                                style: "--ox-wire: {colour};",
                            }
                        }
                        if let Some(d) = preview {
                            path { "data-scope": h::SCOPE, "data-part": "preview", d: "{d}" }
                        }
                    }
                    for (node , selected , dragging , wires , accept , pressed) in boxes {
                        NodeBox {
                            key: "{node.id}",
                            node,
                            selected,
                            dragging,
                            wires,
                            accept,
                            pressed,
                            root: root.clone(),
                            render,
                            send,
                        }
                    }
                }
                if let Some(style) = marquee {
                    div { "data-scope": h::SCOPE, "data-part": "marquee", style: "{style}" }
                }
                if minimap {
                    svg {
                        id: "{mini_id}",
                        "data-scope": h::SCOPE,
                        "data-part": "minimap",
                        "aria-hidden": "true",
                        width: "{mini_size.0}",
                        height: "{mini_size.1}",
                        onpointerdown: move |e| {
                            e.stop_propagation();
                            e.prevent_default();
                            capture(&mini_down, e.pointer_id());
                            mini_drag.set(true);
                            let p = e.element_coordinates();
                            let world = h::Minimap::new(&state.peek()).to_world(Point::new(p.x, p.y));
                            send.call(NodeEditorEvent::CenterOn(world));
                        },
                        onpointermove: move |e| {
                            e.stop_propagation();
                            if mini_drag() {
                                let p = e.element_coordinates();
                                let world = h::Minimap::new(&state.peek()).to_world(Point::new(p.x, p.y));
                                send.call(NodeEditorEvent::CenterOn(world));
                            }
                        },
                        onpointerup: move |e| {
                            e.stop_propagation();
                            mini_drag.set(false);
                        },
                        onwheel: move |e| e.stop_propagation(),
                        path { "data-scope": h::SCOPE, "data-part": "minimap-nodes", d: "{mini_nodes}" }
                        rect {
                            "data-scope": h::SCOPE,
                            "data-part": "minimap-view",
                            x: "{mini_rect[0]}",
                            y: "{mini_rect[1]}",
                            width: "{mini_rect[2]}",
                            height: "{mini_rect[3]}",
                            rx: "2",
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "live", role: "status", "aria-live": "polite", "aria-atomic": "true", "{live}" }
        }
    }
}

/// One node. A component of its own so unchanged nodes skip re-rendering during a drag.
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
fn NodeBox(
    node: Node,
    selected: bool,
    dragging: bool,
    /// Wires per port: inputs then outputs.
    wires: Vec<usize>,
    /// While connecting, whether each port would accept the wire.
    accept: Vec<Option<bool>>,
    /// The port a keyboard connection starts from, when it is on this node.
    pressed: Option<PortRef>,
    root: String,
    render: Option<Callback<Node, Element>>,
    send: Callback<NodeEditorEvent>,
) -> Element {
    let ports: Vec<(usize, Side, Port)> = node
        .inputs
        .iter()
        .enumerate()
        .map(|(i, p)| (i, Side::Input, p.clone()))
        .chain(node.outputs.iter().enumerate().map(|(i, p)| (i, Side::Output, p.clone())))
        .collect();
    let content = (node.body > 0.0).then(|| (h::content_style(&node), render.map(|r| r.call(node.clone()))));
    let focus_id = node.id.clone();
    let instructions = h::instructions_id(&root);

    rsx! {
        div {
            id: h::node_id(&root, &node.id),
            tabindex: "0",
            role: "group",
            "aria-roledescription": "node",
            "aria-label": h::node_label(&node, selected),
            "aria-describedby": "{instructions}",
            "data-scope": h::SCOPE,
            "data-part": "node",
            "data-kind": "{node.kind}",
            "data-selected": selected.then_some(""),
            "data-dragging": dragging.then_some(""),
            style: h::node_style(&node),
            onfocus: move |_| send.call(NodeEditorEvent::Focus(focus_id.clone())),
            div { "data-scope": h::SCOPE, "data-part": "header",
                span { "data-scope": h::SCOPE, "data-part": "kind", "aria-hidden": "true" }
                span { "data-scope": h::SCOPE, "data-part": "title", "{node.title}" }
            }
            for (flat , (i , side , port)) in ports.into_iter().enumerate() {
                {
                    let r = PortRef::new(&node.id, side, &port.id);
                    let is_pressed = pressed.as_ref() == Some(&r);
                    let wired = wires.get(flat).copied().unwrap_or(0);
                    let accepts = accept.get(flat).copied().flatten();
                    let label = h::port_label(&node, side, &port, wired);
                    let style = format!("{} --ox-port: {};", h::port_style(i), h::kind_color(&port.kind));
                    let key = format!("{}-{}", side.as_str(), port.id);
                    rsx! {
                        button {
                            key: "{key}",
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "port",
                            "data-side": side.as_str(),
                            "data-kind": "{port.kind}",
                            "data-wired": (wired > 0).then_some(""),
                            "data-accept": accepts.map(|a| a.to_string()),
                            "aria-pressed": is_pressed.to_string(),
                            "aria-label": label,
                            style,
                            onkeydown: move |e| {
                                let event = match e.key().to_string().as_str() {
                                    "Enter" | " " => Some(NodeEditorEvent::PortKey(r.clone())),
                                    "Delete" | "Backspace" => Some(NodeEditorEvent::Disconnect(r.clone())),
                                    _ => None,
                                };
                                if let Some(event) = event {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    send.call(event);
                                }
                            },
                            span { "data-scope": h::SCOPE, "data-part": "dot", "aria-hidden": "true" }
                            span { "data-scope": h::SCOPE, "data-part": "port-label", "{port.label}" }
                        }
                    }
                }
            }
            if let Some((style, body)) = content {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "content",
                    style,
                    onpointerdown: move |e| e.stop_propagation(),
                    onkeydown: move |e| e.stop_propagation(),
                    onwheel: move |e| e.stop_propagation(),
                    {body}
                }
            }
        }
    }
}
