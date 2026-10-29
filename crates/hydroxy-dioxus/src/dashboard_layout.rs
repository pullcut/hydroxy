//! `DashboardLayout`: widget tiles on a column grid. Drag a tile by its
//! handle, resize it from the corner, or do both from the keyboard.

use std::rc::Rc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use hydroxy_headless::dashboard_layout::{
    self as h, DashboardCommand, DashboardEvent, DashboardInput, DashboardLayoutMachine, DashboardState,
    announcement_text,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::modifiers::mods;

pub use hydroxy_headless::dashboard_layout::DashboardItem;

/// One widget: an id that matches a [`DashboardItem`], a title and its content.
#[derive(Clone, PartialEq)]
pub struct Widget {
    id: String,
    title: String,
    content: Element,
}

impl Widget {
    /// A widget showing `content` under `title`.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: Element) -> Self {
        Self { id: id.into(), title: title.into(), content }
    }
}

/// Where a pointer gesture started.
#[derive(Clone, Copy)]
struct Origin {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<DashboardState>,
    input: Memo<DashboardInput>,
    live: Signal<String>,
    on_layout_change: Option<EventHandler<Vec<DashboardItem>>>,
}

impl Ctx {
    fn send(mut self, titles: &[(String, String)], event: DashboardEvent) {
        let commands = drive::<DashboardLayoutMachine>(self.state, self.input, event);
        for command in commands {
            match command {
                DashboardCommand::LayoutChange(items) => {
                    if let Some(handler) = self.on_layout_change {
                        handler.call(items);
                    }
                }
                DashboardCommand::Announce(a) => {
                    let id = match &a {
                        h::Announcement::Grabbed(s) | h::Announcement::Moved(s) | h::Announcement::Dropped(s) => {
                            s.id.clone()
                        }
                        h::Announcement::Cancelled(id) => id.clone(),
                    };
                    let title = titles.iter().find(|(i, _)| *i == id).map_or_else(|| id.clone(), |(_, t)| t.clone());
                    self.live.set(announcement_text(&a, &title));
                }
            }
        }
    }
}

/// A grid of widget tiles that can be rearranged and resized.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn DashboardLayout(
    /// The widgets. Each needs a matching item in the layout.
    widgets: Vec<Widget>,
    /// Accessible name for the board.
    #[props(default = "Dashboard widgets".to_string())]
    label: String,
    /// Controlled layout. Pair with `on_layout_change`.
    #[props(default)]
    items: ReadSignal<Option<Vec<DashboardItem>>>,
    /// Initial layout when uncontrolled.
    #[props(default)]
    default_items: Vec<DashboardItem>,
    /// Columns in the grid.
    #[props(default = 12)]
    columns: u32,
    /// Height of one row in px.
    #[props(default = 72.0)]
    row_height: f64,
    /// Gap between tiles in px.
    #[props(default = 12.0)]
    gap: f64,
    /// Block every move and resize.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the whole layout when a gesture ends in a change.
    on_layout_change: Option<EventHandler<Vec<DashboardItem>>>,
) -> Element {
    let input = use_memo(move || DashboardInput {
        items: items(),
        default_items: default_items.clone(),
        columns,
        disabled: disabled(),
    });
    let mut state = use_signal(|| DashboardLayoutMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DashboardLayoutMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, live: use_signal(String::new), on_layout_change };
    let titles: Vec<(String, String)> = widgets.iter().map(|w| (w.id.clone(), w.title.clone())).collect();

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut origin = use_signal(|| None::<Origin>);
    let mut width = use_signal(|| 0.0_f64);

    let s = state.read().clone();
    let preview = s.preview(columns);
    let off = disabled();

    let end_titles = titles.clone();
    let end = move |event: DashboardEvent| {
        if origin.peek().is_some() {
            origin.set(None);
            ctx.send(&end_titles, event);
        }
    };
    let (mut up_end, mut cancel_end, move_titles) = (end.clone(), end, titles.clone());

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: h::instructions_id(&root), {h::INSTRUCTIONS} }
            div {
                role: "group",
                "aria-label": label,
                style: h::board_style(columns, row_height, gap),
                "data-scope": h::SCOPE,
                "data-part": "board",
                "data-dragging": s.gesture.as_ref().is_some_and(|g| !g.keyboard).then_some(""),
                onmounted: move |e| mounted.set(Some(e.data())),
                // Moves land on the board: the handle captures the pointer, and the events bubble here.
                onpointermove: move |e| {
                    if let Some(o) = origin() {
                        let p = e.client_coordinates();
                        let (dx, dy) = h::cell_delta((p.x - o.x, p.y - o.y), width(), columns, row_height, gap);
                        ctx.send(&move_titles, DashboardEvent::DragTo { dx, dy });
                    }
                },
                onpointerup: move |_| up_end(DashboardEvent::Drop),
                onpointercancel: move |_| cancel_end(DashboardEvent::Cancel),
                for w in widgets {
                    {
                        let item = preview.iter().find(|i| i.id == w.id).cloned();
                        let handle = h::handle_attrs(&root, &s, &w.id);
                        let grip_id = format!("{root}-grip-{}", w.id);
                        let (grab_id, drag_id, grow_id) = (w.id.clone(), w.id.clone(), w.id.clone());
                        let (key_titles, drag_titles, grow_titles, blur_titles) =
                            (titles.clone(), titles.clone(), titles.clone(), titles.clone());
                        let (handle_dom, grip_dom) = (handle.id.clone(), grip_id.clone());
                        let mut begin = move |e: &PointerEvent, dom_id: String, titles: Vec<(String, String)>, id: String, resize: bool| {
                            if e.trigger_button() != Some(MouseButton::Primary) || off {
                                return;
                            }
                            e.prevent_default();
                            let _ = document::eval(&format!(
                                "try {{ document.getElementById({dom_id})?.setPointerCapture({}); }} catch (_) {{}}",
                                e.pointer_id(), dom_id = crate::js::js_string(&dom_id)));
                            let p = e.client_coordinates();
                            origin.set(Some(Origin { x: p.x, y: p.y }));
                            if let Some(el) = mounted() {
                                spawn(async move {
                                    if let Ok(rect) = el.get_client_rect().await {
                                        width.set(rect.width());
                                    }
                                });
                            }
                            ctx.send(&titles, DashboardEvent::DragStart { id, resize });
                        };
                        let mut begin_grip = begin;
                        rsx! {
                            article {
                                key: "{w.id}",
                                hidden: item.is_none(),
                                style: item.as_ref().map(h::item_style).unwrap_or_default(),
                                "data-scope": h::SCOPE,
                                "data-part": "item",
                                "data-active": (s.active() == Some(grab_id.as_str())).then_some(""),
                                header { "data-scope": h::SCOPE, "data-part": "header",
                                    button {
                                        r#type: "button",
                                        id: handle.id,
                                        "aria-label": "Move {w.title}",
                                        "aria-pressed": handle.aria_pressed.to_string(),
                                        "aria-describedby": handle.aria_describedby,
                                        "data-scope": h::SCOPE,
                                        "data-part": "handle",
                                        disabled: off,
                                        onpointerdown: move |e| begin(&e, handle_dom.clone(), drag_titles.clone(), drag_id.clone(), false),
                                        onkeydown: {
                                            let id = grab_id.clone();
                                            move |e| {
                                                let event = h::handle_keydown(&e.key().to_string(), mods(e.modifiers()), &id, &ctx.state.peek());
                                                if let Some(event) = event {
                                                    e.prevent_default();
                                                    ctx.send(&key_titles, event);
                                                }
                                            }
                                        },
                                        // A keyboard user who tabs away drops the tile where it is.
                                        onblur: move |_| {
                                            if ctx.state.peek().gesture.as_ref().is_some_and(|g| g.keyboard) {
                                                ctx.send(&blur_titles, DashboardEvent::Drop);
                                            }
                                        },
                                        Icon { name: IconName::Grip, size: 16 }
                                    }
                                    h3 { "data-scope": h::SCOPE, "data-part": "title", "{w.title}" }
                                }
                                div { "data-scope": h::SCOPE, "data-part": "body", {w.content} }
                                span {
                                    id: grip_id,
                                    "aria-hidden": "true",
                                    "data-scope": h::SCOPE,
                                    "data-part": "resize",
                                    onpointerdown: move |e| begin_grip(&e, grip_dom.clone(), grow_titles.clone(), grow_id.clone(), true),
                                }
                            }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "live", role: "status", "aria-live": "polite", "aria-atomic": "true", "{ctx.live}" }
        }
    }
}
