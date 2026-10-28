//! `<DashboardLayout>`: widget tiles on a column grid. Drag a tile by its
//! handle, resize it from the corner, or do both from the keyboard.

use hydroxy_headless::dashboard_layout::{
    self as h, DashboardCommand, DashboardEvent, DashboardInput, DashboardLayoutMachine, DashboardState,
    announcement_text,
};
use hydroxy_statechart::Machine;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

pub use hydroxy_headless::dashboard_layout::DashboardItem;

/// One widget: an id that matches a [`DashboardItem`], a title and its content.
#[derive(Clone)]
pub struct Widget {
    id: String,
    title: String,
    content: ViewFn,
}

impl Widget {
    /// A widget showing `content` under `title`.
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<ViewFn>) -> Self {
        Self { id: id.into(), title: title.into(), content: content.into() }
    }
}

/// Where a pointer gesture started.
#[derive(Clone, Copy)]
struct Origin {
    x: f64,
    y: f64,
    width: f64,
}

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<DashboardState>,
    input: Memo<DashboardInput>,
    live: RwSignal<String>,
    titles: StoredValue<Vec<(String, String)>>,
    on_layout_change: Option<Callback<Vec<DashboardItem>>>,
}

impl Ctx {
    fn send(self, event: DashboardEvent) {
        for command in drive::<DashboardLayoutMachine, _>(self.state, self.input, event) {
            match command {
                DashboardCommand::LayoutChange(items) => {
                    if let Some(cb) = self.on_layout_change {
                        cb.run(items);
                    }
                }
                DashboardCommand::Announce(a) => {
                    let id = match &a {
                        h::Announcement::Grabbed(s) | h::Announcement::Moved(s) | h::Announcement::Dropped(s) => {
                            s.id.clone()
                        }
                        h::Announcement::Cancelled(id) => id.clone(),
                    };
                    let title = self.titles.with_value(|t| t.iter().find(|(i, _)| *i == id).map(|(_, t)| t.clone()));
                    self.live.set(announcement_text(&a, &title.unwrap_or(id)));
                }
            }
        }
    }
}

/// A grid of widget tiles that can be rearranged and resized.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DashboardLayout(
    /// The widgets. Each needs a matching item in the layout.
    widgets: Vec<Widget>,
    /// Accessible name for the board.
    #[prop(into, default = "Dashboard widgets".to_string())]
    label: String,
    /// Controlled layout. Pair with `on_layout_change`.
    #[prop(optional, into)]
    items: MaybeProp<Vec<DashboardItem>>,
    /// Initial layout when uncontrolled.
    #[prop(optional, into)]
    default_items: Vec<DashboardItem>,
    /// Columns in the grid.
    #[prop(default = 12)]
    columns: u32,
    /// Height of one row in px.
    #[prop(default = 72.0)]
    row_height: f64,
    /// Gap between tiles in px.
    #[prop(default = 12.0)]
    gap: f64,
    /// Block every move and resize.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called with the whole layout when a gesture ends in a change.
    #[prop(optional, into)]
    on_layout_change: Option<Callback<Vec<DashboardItem>>>,
) -> impl IntoView {
    let input = Memo::new(move |_| DashboardInput {
        items: items.get(),
        default_items: default_items.clone(),
        columns,
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(DashboardLayoutMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DashboardLayoutMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let titles = StoredValue::new(widgets.iter().map(|w| (w.id.clone(), w.title.clone())).collect());
    let ctx = Ctx { state, input, live: RwSignal::new(String::new()), titles, on_layout_change };

    let root_id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let board = NodeRef::<Div>::new();
    let origin = StoredValue::new(None::<Origin>);
    let preview = Memo::new(move |_| state.with(|s| s.preview(columns)));

    let begin = move |ev: &leptos::ev::PointerEvent, id: &str, resize: bool| {
        if ev.button() != 0 || disabled.get_untracked() {
            return;
        }
        ev.prevent_default();
        if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        let width = board.get_untracked().map_or(0, |b| b.client_width());
        origin.set_value(Some(Origin {
            x: f64::from(ev.client_x()),
            y: f64::from(ev.client_y()),
            width: f64::from(width),
        }));
        ctx.send(DashboardEvent::DragStart { id: id.to_string(), resize });
    };
    let end = move |event: DashboardEvent| {
        if origin.get_value().is_some() {
            origin.set_value(None);
            ctx.send(event);
        }
    };

    let articles = widgets
        .into_iter()
        .map(|w| {
            let id = w.id.clone();
            let item = {
                let id = id.clone();
                Memo::new(move |_| preview.with(|p| p.iter().find(|i| i.id == id).cloned()))
            };
            let attrs = {
                let id = id.clone();
                Memo::new(move |_| {
                    state.with(|s| h::handle_attrs(&root_id.read_value(), s, &id))
                })
            };
            let (grab, drag, grow, keyed) = (id.clone(), id.clone(), id.clone(), id.clone());
            let name = w.title.clone();
            view! {
                <article
                    data-scope=h::SCOPE
                    data-part="item"
                    hidden=move || item.with(Option::is_none)
                    style=move || item.with(|i| i.as_ref().map(h::item_style).unwrap_or_default())
                    data-active=move || state.with(|s| s.active() == Some(grab.as_str())).then_some("")
                >
                    <header data-scope=h::SCOPE data-part="header">
                        <button
                            type="button"
                            id=move || attrs.get().id
                            aria-label=format!("Move {name}")
                            aria-pressed=move || attrs.get().aria_pressed.to_string()
                            aria-describedby=move || attrs.get().aria_describedby
                            data-scope=h::SCOPE
                            data-part="handle"
                            disabled=move || disabled.get()
                            on:pointerdown=move |ev| begin(&ev, &drag, false)
                            on:keydown=move |ev| {
                                let event = state.with_untracked(|s| h::handle_keydown(&ev.key(), ev.shift_key(), &keyed, s));
                                if let Some(event) = event {
                                    ev.prevent_default();
                                    ctx.send(event);
                                }
                            }
                            // A keyboard user who tabs away drops the tile where it is.
                            on:blur=move |_| {
                                if state.with_untracked(|s| s.gesture.as_ref().is_some_and(|g| g.keyboard)) {
                                    ctx.send(DashboardEvent::Drop);
                                }
                            }
                        >
                            <Icon name=IconName::Grip size=16 />
                        </button>
                        <h3 data-scope=h::SCOPE data-part="title">{w.title}</h3>
                    </header>
                    <div data-scope=h::SCOPE data-part="body">{w.content.run()}</div>
                    <span
                        aria-hidden="true"
                        data-scope=h::SCOPE
                        data-part="resize"
                        on:pointerdown=move |ev| begin(&ev, &grow, true)
                    ></span>
                </article>
            }
        })
        .collect_view();

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root_id.get_value())>{h::INSTRUCTIONS}</p>
            <div
                node_ref=board
                role="group"
                aria-label=label
                style=move || h::board_style(columns, row_height, gap)
                data-scope=h::SCOPE
                data-part="board"
                data-dragging=move || state.with(|s| s.gesture.as_ref().is_some_and(|g| !g.keyboard)).then_some("")
                on:pointermove=move |ev| {
                    if let Some(o) = origin.get_value() {
                        let travel = (f64::from(ev.client_x()) - o.x, f64::from(ev.client_y()) - o.y);
                        let (dx, dy) = h::cell_delta(travel, o.width, columns, row_height, gap);
                        ctx.send(DashboardEvent::DragTo { dx, dy });
                    }
                }
                on:pointerup=move |_| end(DashboardEvent::Drop)
                on:pointercancel=move |_| end(DashboardEvent::Cancel)
            >
                {articles}
            </div>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || ctx.live.get()}
            </div>
        </div>
    }
}
