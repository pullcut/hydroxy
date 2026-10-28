//! `Whiteboard`: an infinite canvas with shapes, arrows that stick to
//! shapes, freehand ink, sticky notes and text. Select, resize, rotate,
//! restack, undo, and export the scene as JSON or SVG.

use std::rc::Rc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use hydroxy_headless::whiteboard::{
    self as h, Order, Tool, WhiteboardCommand, WhiteboardEvent, WhiteboardInput, WhiteboardMachine,
};
use hydroxy_statechart::Machine;

use crate::js::download;
use crate::node_editor::{capture, icon};

pub use hydroxy_headless::whiteboard::{PALETTE, Point, Rect, Shape, ShapeKind};

/// An infinite whiteboard.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Whiteboard(
    /// Shapes to start with.
    #[props(default)]
    default_shapes: Vec<Shape>,
    /// Called with every shape after each committed change.
    on_change: Option<EventHandler<Vec<Shape>>>,
    /// Accessible name for the canvas.
    #[props(default = "Whiteboard".to_string())]
    label: String,
    /// CSS height of the canvas.
    #[props(default = "520px".to_string())]
    height: String,
    /// File name for exports, without extension.
    #[props(default = "whiteboard".to_string())]
    file_name: String,
) -> Element {
    let mut state =
        use_signal(|| WhiteboardMachine::initial(&WhiteboardInput { default_shapes: default_shapes.clone() }));
    let mut live = use_signal(String::new);
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let vp_id = format!("{root}-viewport");
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut origin = use_signal(|| (0.0_f64, 0.0_f64));
    let mut space = use_signal(|| false);
    let mut fitted = use_signal(|| false);

    let send = use_callback(move |event: WhiteboardEvent| {
        let step = WhiteboardMachine::transition(&state.peek(), event, &WhiteboardInput::default());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                WhiteboardCommand::Change(shapes) => {
                    if let Some(handler) = on_change {
                        handler.call(shapes);
                    }
                }
                WhiteboardCommand::Announce(text) => live.set(text),
                WhiteboardCommand::FocusCanvas => {
                    if let Some(el) = mounted() {
                        spawn(async move {
                            let _ = el.set_focus(true).await;
                        });
                    }
                }
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
    let local = move |x: f64, y: f64| {
        let o = origin();
        Point::new(x - o.0, y - o.1)
    };

    let s = state.read().clone();
    let v = s.viewport;
    let editing_id = s.editing.as_ref().map(|e| e.0.clone());
    let shapes: Vec<(String, &'static str, String, bool, bool)> = s
        .shapes
        .iter()
        .map(|x| {
            (
                x.id.clone(),
                x.kind.as_str(),
                h::shape_svg(x, &s.shapes),
                s.bind_hint.as_deref() == Some(x.id.as_str()),
                editing_id.as_deref() == Some(x.id.as_str()),
            )
        })
        .collect();
    let selection = h::selection_draw(&s);
    let handle_r = format!("{:.2}", 5.0 / v.zoom);
    let marquee = s.marquee().map(|m| [m.x, m.y, m.w, m.h].map(|n| format!("{n:.2}")));
    let editor_style = h::editor_style(&s);
    let editor_text = s.editing().map(|x| x.text.clone()).unwrap_or_default();
    let (tool, colour, percent) = (s.tool, s.color, v.percent());
    let (can_undo, can_redo, has_sel) = (s.can_undo(), s.can_redo(), !s.selection.is_empty());
    let summary = h::canvas_label(&label, &s);
    let view_style = format!("height: {height}; {}", h::viewport_style(v));
    let world = h::world_transform(v);
    let panning = s.panning();
    let instructions = h::instructions_id(&root);
    let down_id = vp_id.clone();
    let (json_name, svg_name) = (format!("{file_name}.json"), format!("{file_name}.svg"));

    let action = move |name: &'static str, text: &'static str, event: WhiteboardEvent, enabled: bool| {
        rsx! {
            button {
                r#type: "button",
                "data-scope": h::SCOPE,
                "data-part": "action",
                "aria-label": text,
                title: text,
                disabled: !enabled,
                onclick: move |_| send.call(event.clone()),
                {icon(name)}
            }
        }
    };

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: "{instructions}", {h::INSTRUCTIONS} }
            div { "data-scope": h::SCOPE, "data-part": "toolbar", role: "toolbar", "aria-label": "Whiteboard tools",
                div { "data-scope": h::SCOPE, "data-part": "group", role: "group", "aria-label": "Tools",
                    for t in Tool::ALL {
                        {
                            let name = format!("{} ({})", t.label(), t.key());
                            let key = t.as_str();
                            rsx! {
                                button {
                                    key: "{key}",
                                    r#type: "button",
                                    "data-scope": h::SCOPE,
                                    "data-part": "tool",
                                    "data-tool": t.as_str(),
                                    "aria-label": "{name}",
                                    title: "{name}",
                                    "aria-pressed": (tool == t).to_string(),
                                    onclick: move |_| send.call(WhiteboardEvent::Tool(t)),
                                    {icon(h::tool_icon(t))}
                                }
                            }
                        }
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "separator" }
                div { "data-scope": h::SCOPE, "data-part": "group", role: "group", "aria-label": "Colour",
                    for (i , sw) in PALETTE.iter().enumerate() {
                        button {
                            key: "{sw.name}",
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "swatch",
                            "aria-label": sw.name,
                            title: sw.name,
                            style: format!("--ox-swatch: var({});", sw.token),
                            "aria-pressed": (colour == i).to_string(),
                            onclick: move |_| send.call(WhiteboardEvent::Color(i)),
                        }
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "separator" }
                div { "data-scope": h::SCOPE, "data-part": "group", role: "group", "aria-label": "Edit",
                    {action("undo", "Undo", WhiteboardEvent::Undo, can_undo)}
                    {action("redo", "Redo", WhiteboardEvent::Redo, can_redo)}
                    {action("copy", "Duplicate", WhiteboardEvent::Duplicate, has_sel)}
                    {action("forward", "Bring forward", WhiteboardEvent::Order(Order::Forward), has_sel)}
                    {action("backward", "Send backward", WhiteboardEvent::Order(Order::Backward), has_sel)}
                    {action("trash", "Delete", WhiteboardEvent::Delete, has_sel)}
                }
                span { "data-scope": h::SCOPE, "data-part": "separator" }
                div { "data-scope": h::SCOPE, "data-part": "group", role: "group", "aria-label": "Zoom",
                    {action("minus", "Zoom out", WhiteboardEvent::ZoomOut, true)}
                    span { "data-scope": h::SCOPE, "data-part": "zoom", "{percent}%" }
                    {action("plus", "Zoom in", WhiteboardEvent::ZoomIn, true)}
                    {action("fit", "Zoom to fit", WhiteboardEvent::Fit, true)}
                }
                span { "data-scope": h::SCOPE, "data-part": "separator" }
                div { "data-scope": h::SCOPE, "data-part": "group", role: "group", "aria-label": "Export",
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "action",
                        title: "Export JSON",
                        onclick: move |_| download(&json_name, "application/json", &h::export_json(&state.peek().shapes)),
                        {icon("download")}
                        " JSON"
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "action",
                        title: "Export SVG",
                        onclick: move |_| download(&svg_name, "image/svg+xml", &h::export_svg(&state.peek().shapes)),
                        {icon("download")}
                        " SVG"
                    }
                }
            }
            div {
                id: "{vp_id}",
                tabindex: "0",
                role: "application",
                "aria-roledescription": "whiteboard",
                "aria-label": "{summary}",
                "aria-describedby": "{instructions}",
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                "data-tool": tool.as_str(),
                "data-panning": panning.then_some(""),
                style: "{view_style}",
                onmounted: move |e| {
                    mounted.set(Some(e.data()));
                    refresh();
                },
                onresize: move |e| {
                    if let Ok(size) = e.data().get_border_box_size() {
                        send.call(WhiteboardEvent::Resize(size.width, size.height));
                        if !fitted() {
                            fitted.set(true);
                            send.call(WhiteboardEvent::Fit);
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
                        send.call(WhiteboardEvent::PointerDown { id, at: local(c.x, c.y), shift, pan });
                    });
                },
                onpointermove: move |e| {
                    if !state.peek().pointers.is_empty() {
                        let c = e.client_coordinates();
                        send.call(WhiteboardEvent::PointerMove { id: e.pointer_id(), at: local(c.x, c.y), shift: e.modifiers().shift() });
                    }
                },
                onpointerup: move |e| send.call(WhiteboardEvent::PointerUp { id: e.pointer_id() }),
                onpointercancel: move |e| send.call(WhiteboardEvent::PointerCancel(e.pointer_id())),
                ondoubleclick: move |e| {
                    let c = e.client_coordinates();
                    send.call(WhiteboardEvent::DoubleClick(local(c.x, c.y)));
                },
                onwheel: move |e| {
                    e.prevent_default();
                    let d = e.data().delta().strip_units();
                    let m = e.modifiers();
                    if m.ctrl() || m.meta() {
                        let c = e.client_coordinates();
                        send.call(WhiteboardEvent::Zoom { at: local(c.x, c.y), factor: hydroxy_charts::zoom::wheel_factor(d.y) });
                    } else {
                        send.call(WhiteboardEvent::Pan(-d.x, -d.y));
                    }
                },
                onkeydown: move |e| {
                    let key = e.key().to_string();
                    if key == " " {
                        e.prevent_default();
                        space.set(true);
                        return;
                    }
                    let m = e.modifiers();
                    if let Some(event) = h::key_event(&key, m.shift(), m.ctrl() || m.meta(), m.alt()) {
                        e.prevent_default();
                        send.call(event);
                    }
                },
                onkeyup: move |e| {
                    if e.key().to_string() == " " {
                        space.set(false);
                    }
                },
                svg { "data-scope": h::SCOPE, "data-part": "canvas", "aria-hidden": "true",
                    g { "data-scope": h::SCOPE, "data-part": "world", transform: "{world}",
                        for (id , kind , markup , hinted , editing) in shapes {
                            g {
                                key: "{id}",
                                "data-scope": h::SCOPE,
                                "data-part": "shape",
                                "data-id": "{id}",
                                "data-kind": kind,
                                "data-hint": hinted.then_some(""),
                                "data-editing": editing.then_some(""),
                                dangerous_inner_html: "{markup}",
                            }
                        }
                        if let Some(d) = selection {
                            g {
                                if d.w > 0.0 {
                                    rect {
                                        "data-scope": h::SCOPE,
                                        "data-part": "selection",
                                        transform: "{d.transform}",
                                        width: format!("{:.2}", d.w),
                                        height: format!("{:.2}", d.h),
                                    }
                                }
                                if let Some((a, b)) = d.stalk {
                                    line {
                                        "data-scope": h::SCOPE,
                                        "data-part": "stalk",
                                        x1: format!("{:.2}", a.x),
                                        y1: format!("{:.2}", a.y),
                                        x2: format!("{:.2}", b.x),
                                        y2: format!("{:.2}", b.y),
                                    }
                                }
                                for (hk , hd) in d.handles.iter().map(|hd| (hd.handle.as_str(), hd)) {
                                    circle {
                                        key: "{hk}",
                                        "data-scope": h::SCOPE,
                                        "data-part": "handle",
                                        "data-handle": hd.handle.as_str(),
                                        cx: format!("{:.2}", hd.at.x),
                                        cy: format!("{:.2}", hd.at.y),
                                        r: "{handle_r}",
                                    }
                                }
                            }
                        }
                        if let Some([x, y, w, hh]) = marquee {
                            rect { "data-scope": h::SCOPE, "data-part": "marquee", x, y, width: w, height: hh }
                        }
                    }
                }
                if let Some(style) = editor_style {
                    textarea {
                        "data-scope": h::SCOPE,
                        "data-part": "editor",
                        "aria-label": "Note text",
                        style,
                        value: "{editor_text}",
                        onmounted: move |e| {
                            let el = e.data();
                            spawn(async move {
                                let _ = el.set_focus(true).await;
                            });
                        },
                        oninput: move |e| send.call(WhiteboardEvent::EditText(e.value())),
                        onblur: move |_| {
                            if state.peek().editing.is_some() {
                                send.call(WhiteboardEvent::EditEnd);
                            }
                        },
                        onpointerdown: move |e| e.stop_propagation(),
                        ondoubleclick: move |e| e.stop_propagation(),
                        onwheel: move |e| e.stop_propagation(),
                        onkeydown: move |e| {
                            e.stop_propagation();
                            let key = e.key().to_string();
                            let m = e.modifiers();
                            if key == "Escape" || (key == "Enter" && (m.ctrl() || m.meta())) {
                                e.prevent_default();
                                send.call(WhiteboardEvent::EditEnd);
                            }
                        },
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "live", role: "status", "aria-live": "polite", "aria-atomic": "true", "{live}" }
        }
    }
}
