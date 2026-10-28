//! `<Whiteboard>`: an infinite canvas with shapes, arrows that stick to
//! shapes, freehand ink, sticky notes and text. Select, resize, rotate,
//! restack, undo, and export the scene as JSON or SVG.

use leptos::html::{Div, Textarea};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::node_editor::icon as icon_markup;
use oxide_headless::whiteboard::{
    self as h, Order, Tool, WhiteboardCommand, WhiteboardEvent, WhiteboardInput, WhiteboardMachine, WhiteboardState,
};
use oxide_statechart::Machine;

pub use oxide_headless::whiteboard::{PALETTE, Point, Rect, Shape, ShapeKind};

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<WhiteboardState>,
    live: RwSignal<String>,
    viewport: NodeRef<Div>,
    on_change: Option<Callback<Vec<Shape>>>,
}

impl Ctx {
    fn apply(self, event: WhiteboardEvent) {
        let input = WhiteboardInput::default();
        let Some(step) = self.state.try_with_untracked(|s| WhiteboardMachine::transition(s, event, &input)) else { return };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                WhiteboardCommand::Change(shapes) => {
                    if let Some(cb) = self.on_change {
                        cb.run(shapes);
                    }
                }
                WhiteboardCommand::Announce(text) => self.live.set(text),
                WhiteboardCommand::FocusCanvas => {
                    if let Some(el) = self.viewport.get_untracked() {
                        let _ = el.focus();
                    }
                }
            }
        }
    }

    fn measure(self) {
        if let Some(el) = self.viewport.get_untracked() {
            let r = el.get_bounding_client_rect();
            let size = (r.width(), r.height());
            if self.state.with_untracked(|s| (s.size.0 - size.0).abs() > 0.5 || (s.size.1 - size.1).abs() > 0.5) {
                self.apply(WhiteboardEvent::Resize(size.0, size.1));
            }
        }
    }

    fn send(self, event: WhiteboardEvent) {
        self.measure();
        self.apply(event);
    }

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
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" inner_html=icon_markup(name)></svg>
    }
}

fn tool_icon(t: Tool) -> &'static str {
    match t {
        Tool::Select => "select",
        Tool::Hand => "hand",
        Tool::Rect => "rect",
        Tool::Ellipse => "ellipse",
        Tool::Arrow => "arrow",
        Tool::Pen => "pen",
        Tool::Sticky => "sticky",
        Tool::Text => "text",
    }
}

/// Save `body` as a file through a temporary link.
fn download(name: &str, mime: &str, body: &str) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(el) = doc.create_element("a") else { return };
    let Ok(a) = el.dyn_into::<web_sys::HtmlAnchorElement>() else { return };
    a.set_href(&h::data_url(mime, body));
    a.set_download(name);
    a.click();
}

/// An infinite whiteboard.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
#[component]
pub fn Whiteboard(
    /// Shapes to start with.
    #[prop(optional)]
    default_shapes: Vec<Shape>,
    /// Called with every shape after each committed change.
    #[prop(optional, into)]
    on_change: Option<Callback<Vec<Shape>>>,
    /// Accessible name for the canvas.
    #[prop(into, default = "Whiteboard".to_string())]
    label: String,
    /// CSS height of the canvas.
    #[prop(into, default = "520px".to_string())]
    height: String,
    /// File name for exports, without extension.
    #[prop(into, default = "whiteboard".to_string())]
    file_name: String,
) -> impl IntoView {
    let state = RwSignal::new(WhiteboardMachine::initial(&WhiteboardInput { default_shapes }));
    let viewport = NodeRef::<Div>::new();
    let editor = NodeRef::<Textarea>::new();
    let ctx = Ctx { state, live: RwSignal::new(String::new()), viewport, on_change };
    let root = StoredValue::new(crate::id::next_id(h::SCOPE));
    let space = StoredValue::new(false);
    let file_name = StoredValue::new(file_name);

    Effect::new(move |_| {
        if viewport.get().is_some() {
            ctx.measure();
            ctx.apply(WhiteboardEvent::Fit);
        }
    });
    let resize = window_event_listener(leptos::ev::resize, move |_| ctx.measure());
    on_cleanup(move || resize.remove());

    let editing = Memo::new(move |_| state.with(|s| s.editing.as_ref().map(|e| e.0.clone())));
    Effect::new(move |_| {
        if editing.get().is_some()
            && let Some(el) = editor.get()
        {
            let _ = el.focus();
        }
    });

    let vp = Memo::new(move |_| state.with(|s| s.viewport));
    let tool = Memo::new(move |_| state.with(|s| s.tool));
    let colour = Memo::new(move |_| state.with(|s| s.color));
    let ids = Memo::new(move |_| state.with(|s| s.shapes.iter().map(|x| x.id.clone()).collect::<Vec<_>>()));
    let selection = Memo::new(move |_| state.with(h::selection_draw));
    let marquee = Memo::new(move |_| state.with(WhiteboardState::marquee));
    let hint = Memo::new(move |_| state.with(|s| s.bind_hint.clone()));
    let editor_style = Memo::new(move |_| state.with(h::editor_style));
    let editor_text = Memo::new(move |_| state.with(|s| s.editing().map(|x| x.text.clone()).unwrap_or_default()));
    let summary = {
        let label = label.clone();
        Memo::new(move |_| state.with(|s| h::canvas_label(&label, s)))
    };

    let shape_view = move |id: String| {
        let markup = {
            let id = id.clone();
            Memo::new(move |_| state.with(|s| s.shapes.iter().find(|x| x.id == id).map(|x| (x.kind, h::shape_svg(x, &s.shapes)))))
        };
        let (hint_id, edit_id) = (id.clone(), id.clone());
        view! {
            <g
                data-scope=h::SCOPE
                data-part="shape"
                data-id=id
                data-kind=move || markup.with(|m| m.as_ref().map(|m| m.0.as_str()))
                data-hint=move || hint.with(|x| x.as_deref() == Some(hint_id.as_str())).then_some("")
                data-editing=move || editing.with(|x| x.as_deref() == Some(edit_id.as_str())).then_some("")
                inner_html=move || markup.with(|m| m.as_ref().map(|m| m.1.clone()).unwrap_or_default())
            ></g>
        }
    };

    let action = move |name: &'static str, text: &'static str, event: WhiteboardEvent, enabled: Signal<bool>| {
        view! {
            <button type="button" data-scope=h::SCOPE data-part="action" aria-label=text title=text
                disabled=move || !enabled.get() on:click=move |_| ctx.send(event.clone())>{icon(name)}</button>
        }
    };
    let has_sel = Signal::derive(move || state.with(|s| !s.selection.is_empty()));
    let always = Signal::derive(|| true);

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root.get_value())>{h::INSTRUCTIONS}</p>
            <div data-scope=h::SCOPE data-part="toolbar" role="toolbar" aria-label="Whiteboard tools">
                <div data-scope=h::SCOPE data-part="group" role="group" aria-label="Tools">
                    {Tool::ALL.iter().map(|t| {
                        let t = *t;
                        let name = format!("{} ({})", t.label(), t.key());
                        view! {
                            <button type="button" data-scope=h::SCOPE data-part="tool" data-tool=t.as_str()
                                aria-label=name.clone() title=name
                                aria-pressed=move || (tool.get() == t).to_string()
                                on:click=move |_| ctx.send(WhiteboardEvent::Tool(t))>{icon(tool_icon(t))}</button>
                        }
                    }).collect_view()}
                </div>
                <span data-scope=h::SCOPE data-part="separator"></span>
                <div data-scope=h::SCOPE data-part="group" role="group" aria-label="Colour">
                    {PALETTE.iter().enumerate().map(|(i, sw)| view! {
                        <button type="button" data-scope=h::SCOPE data-part="swatch" aria-label=sw.name title=sw.name
                            style=format!("--ox-swatch: var({});", sw.token)
                            aria-pressed=move || (colour.get() == i).to_string()
                            on:click=move |_| ctx.send(WhiteboardEvent::Color(i))></button>
                    }).collect_view()}
                </div>
                <span data-scope=h::SCOPE data-part="separator"></span>
                <div data-scope=h::SCOPE data-part="group" role="group" aria-label="Edit">
                    {action("undo", "Undo", WhiteboardEvent::Undo, Signal::derive(move || state.with(WhiteboardState::can_undo)))}
                    {action("redo", "Redo", WhiteboardEvent::Redo, Signal::derive(move || state.with(WhiteboardState::can_redo)))}
                    {action("copy", "Duplicate", WhiteboardEvent::Duplicate, has_sel)}
                    {action("forward", "Bring forward", WhiteboardEvent::Order(Order::Forward), has_sel)}
                    {action("backward", "Send backward", WhiteboardEvent::Order(Order::Backward), has_sel)}
                    {action("trash", "Delete", WhiteboardEvent::Delete, has_sel)}
                </div>
                <span data-scope=h::SCOPE data-part="separator"></span>
                <div data-scope=h::SCOPE data-part="group" role="group" aria-label="Zoom">
                    {action("minus", "Zoom out", WhiteboardEvent::ZoomOut, always)}
                    <span data-scope=h::SCOPE data-part="zoom">{move || format!("{}%", vp.get().percent())}</span>
                    {action("plus", "Zoom in", WhiteboardEvent::ZoomIn, always)}
                    {action("fit", "Zoom to fit", WhiteboardEvent::Fit, always)}
                </div>
                <span data-scope=h::SCOPE data-part="separator"></span>
                <div data-scope=h::SCOPE data-part="group" role="group" aria-label="Export">
                    <button type="button" data-scope=h::SCOPE data-part="action" title="Export JSON"
                        on:click=move |_| {
                            let body = state.with_untracked(|s| h::export_json(&s.shapes));
                            download(&format!("{}.json", file_name.get_value()), "application/json", &body);
                        }>{icon("download")}" JSON"</button>
                    <button type="button" data-scope=h::SCOPE data-part="action" title="Export SVG"
                        on:click=move |_| {
                            let body = state.with_untracked(|s| h::export_svg(&s.shapes));
                            download(&format!("{}.svg", file_name.get_value()), "image/svg+xml", &body);
                        }>{icon("download")}" SVG"</button>
                </div>
            </div>
            <div
                node_ref=viewport
                tabindex="0"
                role="application"
                aria-roledescription="whiteboard"
                aria-label=move || summary.get()
                aria-describedby=h::instructions_id(&root.get_value())
                data-scope=h::SCOPE
                data-part="viewport"
                data-tool=move || tool.get().as_str()
                data-panning=move || state.with(WhiteboardState::panning).then_some("")
                style=move || format!("height: {height}; {}", h::viewport_style(vp.get()))
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
                    ctx.send(WhiteboardEvent::PointerDown { id: ev.pointer_id(), at, shift: ev.shift_key(), pan });
                }
                on:pointermove=move |ev| {
                    if state.with_untracked(|s| !s.pointers.is_empty()) {
                        let at = ctx.local(ev.client_x(), ev.client_y());
                        ctx.apply(WhiteboardEvent::PointerMove { id: ev.pointer_id(), at, shift: ev.shift_key() });
                    }
                }
                on:pointerup=move |ev| ctx.apply(WhiteboardEvent::PointerUp { id: ev.pointer_id() })
                on:pointercancel=move |ev| ctx.apply(WhiteboardEvent::PointerCancel(ev.pointer_id()))
                on:dblclick=move |ev| {
                    let at = ctx.local(ev.client_x(), ev.client_y());
                    ctx.send(WhiteboardEvent::DoubleClick(at));
                }
                on:wheel=move |ev| {
                    ev.prevent_default();
                    let lines = if ev.delta_mode() == 1 { 16.0 } else { 1.0 };
                    if ev.ctrl_key() || ev.meta_key() {
                        let at = ctx.local(ev.client_x(), ev.client_y());
                        ctx.send(WhiteboardEvent::Zoom { at, factor: oxide_charts::zoom::wheel_factor(ev.delta_y() * lines) });
                    } else {
                        ctx.send(WhiteboardEvent::Pan(-ev.delta_x() * lines, -ev.delta_y() * lines));
                    }
                }
                on:keydown=move |ev| {
                    if ev.key() == " " {
                        ev.prevent_default();
                        space.set_value(true);
                        return;
                    }
                    if let Some(event) = h::key_event(&ev.key(), ev.shift_key(), ev.ctrl_key() || ev.meta_key(), ev.alt_key()) {
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
                <svg data-scope=h::SCOPE data-part="canvas" aria-hidden="true">
                    <g data-scope=h::SCOPE data-part="world" transform=move || h::world_transform(vp.get())>
                        <For each=move || ids.get() key=|id| id.clone() children=shape_view />
                        {move || selection.get().map(|d| {
                            let z = vp.with_untracked(|v| v.zoom);
                            let r = 5.0 / z;
                            view! {
                                <g>
                                    {(d.w > 0.0).then(|| view! {
                                        <rect data-scope=h::SCOPE data-part="selection" transform=d.transform.clone()
                                            width=format!("{:.2}", d.w) height=format!("{:.2}", d.h) />
                                    })}
                                    {d.stalk.map(|(a, b)| view! {
                                        <line data-scope=h::SCOPE data-part="stalk" x1=format!("{:.2}", a.x) y1=format!("{:.2}", a.y)
                                            x2=format!("{:.2}", b.x) y2=format!("{:.2}", b.y) />
                                    })}
                                    {d.handles.into_iter().map(|hd| view! {
                                        <circle data-scope=h::SCOPE data-part="handle" data-handle=hd.handle.as_str()
                                            cx=format!("{:.2}", hd.at.x) cy=format!("{:.2}", hd.at.y) r=format!("{r:.2}") />
                                    }).collect_view()}
                                </g>
                            }
                        })}
                        {move || marquee.get().map(|m| view! {
                            <rect data-scope=h::SCOPE data-part="marquee" x=format!("{:.2}", m.x) y=format!("{:.2}", m.y)
                                width=format!("{:.2}", m.w) height=format!("{:.2}", m.h) />
                        })}
                    </g>
                </svg>
                {move || editing.get().map(|_| view! {
                    <textarea
                        node_ref=editor
                        data-scope=h::SCOPE
                        data-part="editor"
                        aria-label="Note text"
                        style=move || editor_style.get().unwrap_or_default()
                        prop:value=editor_text.get_untracked()
                        on:input=move |ev| ctx.apply(WhiteboardEvent::EditText(event_target_value(&ev)))
                        on:blur=move |_| {
                            if editing.get_untracked().is_some() {
                                ctx.apply(WhiteboardEvent::EditEnd);
                            }
                        }
                        on:pointerdown=|ev| ev.stop_propagation()
                        on:dblclick=|ev| ev.stop_propagation()
                        on:wheel=|ev| ev.stop_propagation()
                        on:keydown=move |ev| {
                            ev.stop_propagation();
                            if ev.key() == "Escape" || (ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key())) {
                                ev.prevent_default();
                                ctx.apply(WhiteboardEvent::EditEnd);
                            }
                        }
                    ></textarea>
                })}
            </div>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">{move || ctx.live.get()}</div>
        </div>
    }
}
