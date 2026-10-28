//! `<NodeEditor>`: a flow builder. Nodes with typed ports on a pan/zoom
//! canvas; drag from a port to wire it up, drag nodes to arrange them, and
//! do all of it from the keyboard too. Node content comes from the host.

use std::collections::BTreeMap;

use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::node_editor::{
    self as h, NodeEditorCommand, NodeEditorEvent, NodeEditorInput, NodeEditorMachine, NodeEditorState, Point,
};
use oxide_statechart::Machine;

use crate::render_fn::RenderFn;

pub use oxide_headless::node_editor::{Edge, Graph, Node, Port, PortRef, Side};

/// Wires per port, so port labels don't each scan every edge.
type Wired = Memo<BTreeMap<PortRef, usize>>;

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<NodeEditorState>,
    input: Memo<NodeEditorInput>,
    live: RwSignal<String>,
    viewport: NodeRef<Div>,
    on_change: Option<Callback<Graph>>,
}

impl Ctx {
    fn apply(self, event: NodeEditorEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| NodeEditorMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                NodeEditorCommand::Change(g) => {
                    if let Some(cb) = self.on_change {
                        cb.run(g);
                    }
                }
                NodeEditorCommand::Announce(text) => self.live.set(text),
            }
        }
    }

    /// Keep the machine's idea of the canvas size current.
    fn measure(self) {
        if let Some(el) = self.viewport.get_untracked() {
            let r = el.get_bounding_client_rect();
            let size = (r.width(), r.height());
            if self.state.with_untracked(|s| (s.size.0 - size.0).abs() > 0.5 || (s.size.1 - size.1).abs() > 0.5) {
                self.apply(NodeEditorEvent::Resize(size.0, size.1));
            }
        }
    }

    fn send(self, event: NodeEditorEvent) {
        self.measure();
        self.apply(event);
    }

    /// A client point in canvas pixels.
    fn local(self, x: i32, y: i32) -> Point {
        self.viewport.get_untracked().map_or_else(Point::default, |el| {
            let r = el.get_bounding_client_rect();
            Point::new(f64::from(x) - r.left(), f64::from(y) - r.top())
        })
    }
}

fn icon(name: &'static str) -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.75"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" inner_html=h::icon(name)></svg>
    }
}

/// A flow builder canvas.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn NodeEditor(
    /// The graph. When it changes the editor adopts it; pair with `on_change` to keep it in step.
    #[prop(into)]
    graph: Signal<Graph>,
    /// Called with the whole graph after every committed change.
    #[prop(optional, into)]
    on_change: Option<Callback<Graph>>,
    /// Renders a node's content area (its `body` height). Gets the node.
    #[prop(optional, into)]
    render: Option<RenderFn<Node>>,
    /// Accessible name for the canvas.
    #[prop(into, default = "Flow editor".to_string())]
    label: String,
    /// Grid spacing in world units.
    #[prop(default = 16.0)]
    grid: f64,
    /// Snap dragged nodes to the grid.
    #[prop(default = true)]
    snap: bool,
    /// Show the minimap.
    #[prop(default = true)]
    minimap: bool,
    /// CSS height of the canvas.
    #[prop(into, default = "480px".to_string())]
    height: String,
) -> impl IntoView {
    let input = Memo::new(move |_| NodeEditorInput { graph: graph.get(), grid, snap });
    let state = RwSignal::new(input.with_untracked(NodeEditorMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| NodeEditorMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let viewport = NodeRef::<Div>::new();
    let ctx = Ctx { state, input, live: RwSignal::new(String::new()), viewport, on_change };
    let root = StoredValue::new(crate::id::next_id(h::SCOPE));
    let space = StoredValue::new(false);

    // Measure once mounted, then frame the graph.
    Effect::new(move |_| {
        if viewport.get().is_some() {
            ctx.measure();
            ctx.apply(NodeEditorEvent::Fit);
        }
    });
    let resize = window_event_listener(leptos::ev::resize, move |_| ctx.measure());
    on_cleanup(move || resize.remove());

    let view_style = Memo::new(move |_| state.with(|s| (s.viewport, h::viewport_style(s.viewport, grid))));
    let ids = Memo::new(move |_| state.with(|s| s.graph.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>()));
    let wired: Wired = Memo::new(move |_| {
        state.with(|s| {
            let mut counts = BTreeMap::new();
            for e in &s.graph.edges {
                *counts.entry(e.from.clone()).or_insert(0) += 1;
                *counts.entry(e.to.clone()).or_insert(0) += 1;
            }
            counts
        })
    });
    let anchor = Memo::new(move |_| state.with(|s| s.anchor().cloned()));
    let edges = Memo::new(move |_| {
        state.with(|s| {
            let hidden = s.detached();
            s.graph
                .edges
                .iter()
                .filter(|e| Some(e.id.as_str()) != hidden)
                .filter_map(|e| {
                    let (a, b) = s.graph.edge_points(e)?;
                    Some((e.id.clone(), h::wire_path(a, b), h::edge_color(&s.graph, e), s.selection.edges.contains(&e.id)))
                })
                .collect::<Vec<_>>()
        })
    });
    let preview = Memo::new(move |_| state.with(|s| s.preview_wire().map(|(a, b)| h::wire_path(a, b))));
    let marquee = Memo::new(move |_| {
        state.with(|s| {
            s.marquee().map(|r| {
                let a = s.viewport.to_screen(Point::new(r.x, r.y));
                format!("left: {:.1}px; top: {:.1}px; width: {:.1}px; height: {:.1}px;", a.x, a.y, r.w * s.viewport.zoom, r.h * s.viewport.zoom)
            })
        })
    });
    let mini = Memo::new(move |_| {
        state.with(|s| {
            let m = h::Minimap::new(s);
            let v = m.rect(s.viewport.visible(s.size));
            (m.nodes_path(&s.graph), v)
        })
    });
    let mini_drag = StoredValue::new(false);
    let mini_jump = move |ev: &leptos::ev::PointerEvent| {
        let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let r = el.get_bounding_client_rect();
        let p = Point::new(f64::from(ev.client_x()) - r.left(), f64::from(ev.client_y()) - r.top());
        let world = state.with_untracked(|s| h::Minimap::new(s).to_world(p));
        ctx.send(NodeEditorEvent::CenterOn(world));
    };

    let node_views = move |id: String| node_view(ctx, root, &id, render.clone(), wired, anchor);

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root.get_value())>{h::INSTRUCTIONS}</p>
            <div data-scope=h::SCOPE data-part="toolbar" role="toolbar" aria-label="Canvas">
                <button type="button" data-scope=h::SCOPE data-part="action" aria-label="Zoom out" on:click=move |_| ctx.send(NodeEditorEvent::ZoomOut)>{icon("minus")}</button>
                <span data-scope=h::SCOPE data-part="zoom" aria-live="polite">{move || format!("{}%", view_style.get().0.percent())}</span>
                <button type="button" data-scope=h::SCOPE data-part="action" aria-label="Zoom in" on:click=move |_| ctx.send(NodeEditorEvent::ZoomIn)>{icon("plus")}</button>
                <button type="button" data-scope=h::SCOPE data-part="action" aria-label="Zoom to fit" on:click=move |_| ctx.send(NodeEditorEvent::Fit)>{icon("fit")}</button>
                <span data-scope=h::SCOPE data-part="separator"></span>
                <button type="button" data-scope=h::SCOPE data-part="action" aria-label="Undo"
                    disabled=move || state.with(|s| !s.can_undo()) on:click=move |_| ctx.send(NodeEditorEvent::Undo)>{icon("undo")}</button>
                <button type="button" data-scope=h::SCOPE data-part="action" aria-label="Redo"
                    disabled=move || state.with(|s| !s.can_redo()) on:click=move |_| ctx.send(NodeEditorEvent::Redo)>{icon("redo")}</button>
                <button type="button" data-scope=h::SCOPE data-part="action" aria-label="Delete selection"
                    disabled=move || state.with(|s| s.selection.is_empty()) on:click=move |_| ctx.send(NodeEditorEvent::Delete)>{icon("trash")}</button>
            </div>
            <div
                node_ref=viewport
                tabindex="0"
                role="application"
                aria-roledescription="node editor"
                aria-label=label
                aria-describedby=h::instructions_id(&root.get_value())
                data-scope=h::SCOPE
                data-part="viewport"
                style=move || format!("height: {height}; {}", view_style.get().1)
                data-panning=move || state.with(NodeEditorState::panning).then_some("")
                data-connecting=move || anchor.with(Option::is_some).then_some("")
                on:pointerdown=move |ev| {
                    if ev.button() > 1 {
                        return;
                    }
                    ev.prevent_default();
                    if let Some(el) = viewport.get_untracked() {
                        let _ = el.set_pointer_capture(ev.pointer_id());
                        let _ = el.focus();
                    }
                    let at = ctx.local(ev.client_x(), ev.client_y());
                    let pan = ev.button() == 1 || space.get_value();
                    ctx.send(NodeEditorEvent::PointerDown { id: ev.pointer_id(), at, shift: ev.shift_key(), pan });
                }
                on:pointermove=move |ev| {
                    if state.with_untracked(|s| !s.pointers.is_empty()) {
                        let at = ctx.local(ev.client_x(), ev.client_y());
                        ctx.apply(NodeEditorEvent::PointerMove { id: ev.pointer_id(), at });
                    }
                }
                on:pointerup=move |ev| {
                    let at = ctx.local(ev.client_x(), ev.client_y());
                    ctx.apply(NodeEditorEvent::PointerUp { id: ev.pointer_id(), at });
                }
                on:pointercancel=move |ev| ctx.apply(NodeEditorEvent::PointerCancel(ev.pointer_id()))
                on:wheel=move |ev| {
                    ev.prevent_default();
                    let lines = if ev.delta_mode() == 1 { 16.0 } else { 1.0 };
                    let at = ctx.local(ev.client_x(), ev.client_y());
                    let factor = oxide_charts::zoom::wheel_factor(ev.delta_y() * lines);
                    ctx.send(NodeEditorEvent::Zoom { at, factor });
                }
                on:keydown=move |ev| {
                    if ev.key() == " " {
                        ev.prevent_default();
                        space.set_value(true);
                        return;
                    }
                    if let Some(event) = h::key_event(&ev.key(), ev.shift_key(), ev.ctrl_key() || ev.meta_key()) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
                on:keyup=move |ev| {
                    if ev.key() == " " {
                        space.set_value(false);
                    }
                }
            >
                <div data-scope=h::SCOPE data-part="world" style=move || view_style.get().0.css()>
                    <svg data-scope=h::SCOPE data-part="edges" aria-hidden="true">
                        <For each=move || edges.get() key=|e| e.clone() let:e>
                            <path
                                data-scope=h::SCOPE
                                data-part="edge"
                                data-selected=e.3.then_some("")
                                d=e.1
                                style=format!("--ox-wire: {};", e.2)
                            />
                        </For>
                        {move || preview.get().map(|d| view! { <path data-scope=h::SCOPE data-part="preview" d=d /> })}
                    </svg>
                    <For each=move || ids.get() key=|id| id.clone() children=node_views />
                </div>
                {move || marquee.get().map(|style| view! { <div data-scope=h::SCOPE data-part="marquee" style=style></div> })}
                <Show when=move || minimap>
                    <svg
                        data-scope=h::SCOPE
                        data-part="minimap"
                        aria-hidden="true"
                        width=h::MINIMAP.0.to_string()
                        height=h::MINIMAP.1.to_string()
                        on:pointerdown=move |ev| {
                            ev.stop_propagation();
                            ev.prevent_default();
                            if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                                let _ = el.set_pointer_capture(ev.pointer_id());
                            }
                            mini_drag.set_value(true);
                            mini_jump(&ev);
                        }
                        on:pointermove=move |ev| {
                            ev.stop_propagation();
                            if mini_drag.get_value() {
                                mini_jump(&ev);
                            }
                        }
                        on:pointerup=move |ev| {
                            ev.stop_propagation();
                            mini_drag.set_value(false);
                        }
                        on:wheel=move |ev| ev.stop_propagation()
                    >
                        <path data-scope=h::SCOPE data-part="minimap-nodes" d=move || mini.get().0 />
                        <rect
                            data-scope=h::SCOPE
                            data-part="minimap-view"
                            x=move || format!("{:.1}", mini.get().1.x)
                            y=move || format!("{:.1}", mini.get().1.y)
                            width=move || format!("{:.1}", mini.get().1.w)
                            height=move || format!("{:.1}", mini.get().1.h)
                            rx="2"
                        />
                    </svg>
                </Show>
            </div>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">{move || ctx.live.get()}</div>
        </div>
    }
}

/// One node. Position and content are separate memos, so a drag never rebuilds the ports
/// (and never drops keyboard focus from one).
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
fn node_view(
    ctx: Ctx,
    root: StoredValue<String>,
    id: &str,
    render: Option<RenderFn<Node>>,
    wired: Wired,
    anchor: Memo<Option<PortRef>>,
) -> impl IntoView + use<> {
    let state = ctx.state;
    let node = {
        let id = id.to_owned();
        Memo::new(move |_| state.with(|s| s.graph.node(&id).cloned()))
    };
    let style = Memo::new(move |_| node.with(|n| n.as_ref().map(h::node_style).unwrap_or_default()));
    let face = Memo::new(move |_| {
        node.with(|n| n.as_ref().map(|n| (n.title.clone(), n.kind.clone(), n.inputs.clone(), n.outputs.clone(), n.body)))
    });
    let selected = {
        let id = id.to_owned();
        Memo::new(move |_| state.with(|s| s.selection.nodes.contains(&id)))
    };
    let dragging = Memo::new(move |_| selected.get() && state.with(NodeEditorState::dragging));
    let focus_id = id.to_owned();
    let node_id = id.to_owned();
    let first = node.get_untracked();

    let port_views = move || {
        let Some((_, _, inputs, outputs, _)) = face.get() else { return Vec::new() };
        let n = node.get_untracked().unwrap_or_else(|| Node::new("", "", 0.0, 0.0));
        let mut out = Vec::new();
        for (side, ports) in [(Side::Input, inputs), (Side::Output, outputs)] {
            for (i, port) in ports.into_iter().enumerate() {
                let r = PortRef::new(&node_id, side, &port.id);
                let (r1, r2, r3, r4, r5) = (r.clone(), r.clone(), r.clone(), r.clone(), r.clone());
                let n = n.clone();
                let label_port = port.clone();
                let label = move || h::port_label(&n, side, &label_port, wired.with(|w| w.get(&r5).copied().unwrap_or(0)));
                out.push(view! {
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="port"
                        data-side=side.as_str()
                        data-kind=port.kind.clone()
                        data-wired=move || wired.with(|w| w.contains_key(&r1)).then_some("")
                        data-accept=move || anchor.with(|a| a.as_ref().map(|_| state.with_untracked(|s| s.accepts(&r2) == Some(true)).to_string()))
                        aria-pressed=move || anchor.with(|a| (a.as_ref() == Some(&r3)).to_string())
                        aria-label=label
                        style=format!("{} --ox-port: {};", h::port_style(i), h::kind_color(&port.kind))
                        on:keydown=move |ev| {
                            let event = match ev.key().as_str() {
                                "Enter" | " " => Some(NodeEditorEvent::PortKey(r4.clone())),
                                "Delete" | "Backspace" => Some(NodeEditorEvent::Disconnect(r4.clone())),
                                _ => None,
                            };
                            if let Some(event) = event {
                                ev.prevent_default();
                                ev.stop_propagation();
                                ctx.send(event);
                            }
                        }
                    >
                        <span data-scope=h::SCOPE data-part="dot" aria-hidden="true"></span>
                        <span data-scope=h::SCOPE data-part="port-label">{port.label.clone()}</span>
                    </button>
                });
            }
        }
        out
    };

    let content = first.filter(|n| n.body > 0.0).map(|n| {
        let style = h::content_style(&n);
        let body = render.map(|r| r.run(n));
        view! {
            <div
                data-scope=h::SCOPE
                data-part="content"
                style=style
                on:pointerdown=|ev| ev.stop_propagation()
                on:keydown=|ev| ev.stop_propagation()
                on:wheel=|ev| ev.stop_propagation()
            >
                {body}
            </div>
        }
    });

    view! {
        <div
            id=h::node_id(&root.get_value(), id)
            tabindex="0"
            role="group"
            aria-roledescription="node"
            aria-label=move || node.with(|n| n.as_ref().map(|n| h::node_label(n, selected.get())).unwrap_or_default())
            aria-describedby=h::instructions_id(&root.get_value())
            data-scope=h::SCOPE
            data-part="node"
            data-kind=move || face.with(|f| f.as_ref().map(|f| f.1.clone()))
            data-selected=move || selected.get().then_some("")
            data-dragging=move || dragging.get().then_some("")
            style=move || style.get()
            on:focus=move |_| ctx.send(NodeEditorEvent::Focus(focus_id.clone()))
        >
            <div data-scope=h::SCOPE data-part="header">
                <span data-scope=h::SCOPE data-part="kind" aria-hidden="true"></span>
                <span data-scope=h::SCOPE data-part="title">{move || face.with(|f| f.as_ref().map(|f| f.0.clone()))}</span>
            </div>
            {port_views}
            {content}
        </div>
    }
}
