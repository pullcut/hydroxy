//! `WeekScheduler`: events on a time grid in day, work-week or week view.
//! Drag on empty time to create an event, drag an event to move it (across days
//! too), drag its bottom edge to resize it; everything snaps to `snap` minutes.
//! The owner holds the events and hears about each change through callbacks.
//!
//! Dioxus has no pointer capture on every renderer, so while dragging a fixed
//! full-window layer catches the pointer, and positions are worked out from how
//! far it moved since the press. Scrolling the grid mid-drag is not followed.

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use hydroxy_headless::week_scheduler::{
    self as h, GridPoint, Hit, WeekSchedulerCommand, WeekSchedulerEvent, WeekSchedulerInput, WeekSchedulerMachine,
    WeekSchedulerState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

mod all_day;
mod event;

use all_day::all_day_chip;
use event::event_block;

pub use h::{Date, Placement, WeekEvent, WeekView, apply_change, event_label, placement_text};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<WeekSchedulerState>,
    input: Memo<WeekSchedulerInput>,
    live: Signal<String>,
    /// Width of the days layer in px.
    width: Signal<f64>,
    /// Pointer client position, and its position on the days layer, at the press.
    origin: Signal<((f64, f64), (f64, f64))>,
    slot: Signal<String>,
    on_create: Option<EventHandler<Placement>>,
    on_change: Option<EventHandler<WeekEvent>>,
    on_select: Option<EventHandler<String>>,
    on_range_change: Option<EventHandler<(Date, Date)>>,
}

impl Ctx {
    fn send(self, event: WeekSchedulerEvent) {
        for command in drive::<WeekSchedulerMachine>(self.state, self.input, event) {
            match command {
                WeekSchedulerCommand::Create(p) => {
                    if let Some(handler) = self.on_create {
                        handler.call(p);
                    }
                }
                WeekSchedulerCommand::Change(e) => {
                    if let Some(handler) = self.on_change {
                        handler.call(e);
                    }
                }
                WeekSchedulerCommand::Select(id) => {
                    if let Some(handler) = self.on_select {
                        handler.call(id);
                    }
                }
                WeekSchedulerCommand::RangeChange(a, b) => {
                    if let Some(handler) = self.on_range_change {
                        handler.call((a, b));
                    }
                }
                WeekSchedulerCommand::RevealCursor => crate::js::run(
                    &[("id", self.slot.peek().as_str())],
                    "requestAnimationFrame(() => document.getElementById(id)?.scrollIntoView({ block: 'nearest' }));",
                ),
                WeekSchedulerCommand::Announce(text) => {
                    let mut live = self.live;
                    live.set(text);
                }
            }
        }
    }

    fn dragging(self) -> bool {
        self.state.peek().drag.is_some()
    }

    fn count(self) -> usize {
        self.state.peek().days(&self.input.peek()).len()
    }

    /// Width of one day column in px.
    #[allow(clippy::cast_precision_loss, reason = "a handful of days")]
    fn column(self) -> f64 {
        *self.width.peek() / self.count().max(1) as f64
    }

    /// Pointer down at `grid` px on the days layer: start a drag.
    fn down(self, e: &Event<PointerData>, hit: Hit, grid: (f64, f64)) {
        if e.trigger_button() != Some(MouseButton::Primary) || self.input.peek().disabled {
            return;
        }
        e.stop_propagation();
        let c = e.client_coordinates();
        let mut origin = self.origin;
        origin.set(((c.x, c.y), grid));
        let at = h::point_at(grid.0, grid.1, *self.width.peek(), self.count());
        self.send(WeekSchedulerEvent::PointerDown { hit, at });
    }

    /// The pointer moved during a drag.
    fn follow(self, e: &Event<PointerData>) {
        if !self.dragging() {
            return;
        }
        let ((cx, cy), (gx, gy)) = *self.origin.peek();
        let c = e.client_coordinates();
        let at: GridPoint = h::point_at(gx + c.x - cx, gy + c.y - cy, *self.width.peek(), self.count());
        self.send(WeekSchedulerEvent::PointerMove(at));
    }

    fn release(self, cancel: bool) {
        if self.dragging() {
            self.send(if cancel { WeekSchedulerEvent::Cancel } else { WeekSchedulerEvent::PointerUp });
        }
    }
}

/// A week of events on a time grid. Hours run down, days across; all-day
/// events sit in a row above. The grid opens scrolled to `scroll_to_hour`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn WeekScheduler(
    /// Accessible name for the whole scheduler, such as "Team calendar".
    label: String,
    /// Every event. Update it from `on_create` and `on_change`.
    events: ReadSignal<Vec<WeekEvent>>,
    /// Today: the Today button's target and the highlighted column.
    today: Date,
    /// The date shown first. Default today.
    #[props(default)]
    default_date: Option<Date>,
    /// The view shown first. Default week.
    #[props(default)]
    default_view: WeekView,
    /// First day of the week, 0 = Sunday. Default 1 (Monday).
    #[props(default = 1)]
    week_start: u8,
    /// Minutes drags and arrow keys snap to. Default 15.
    #[props(default = 15)]
    snap: u16,
    /// Length in minutes of an event made by a click or Enter. Default 60.
    #[props(default = 60)]
    default_duration: u16,
    /// The time zone the times are in, shown in the corner, such as "GMT".
    timezone: String,
    /// Minutes after midnight now, for the line across today. None draws no line.
    #[props(default)]
    now: ReadSignal<Option<u16>>,
    /// Hour shown at the top when the grid opens. Default 8.
    #[props(default = 8)]
    scroll_to_hour: u8,
    /// Height of the scrolling grid in px. Default 560.
    #[props(default = 560)]
    height: u32,
    /// Block creating, moving and resizing.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// A new event was drawn. Add one with this placement.
    on_create: Option<EventHandler<Placement>>,
    /// An event was moved or resized. Store it, for example with [`apply_change`].
    on_change: Option<EventHandler<WeekEvent>>,
    /// An event was clicked or Enter was pressed on it.
    on_select: Option<EventHandler<String>>,
    /// The visible days changed, first to last.
    on_range_change: Option<EventHandler<(Date, Date)>>,
) -> Element {
    let input = use_memo(move || WeekSchedulerInput {
        events: events(),
        today,
        default_date: default_date.unwrap_or(today),
        default_view,
        week_start,
        snap,
        default_duration,
        disabled: disabled(),
    });
    let mut state = use_signal(|| WeekSchedulerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = WeekSchedulerMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx {
        state,
        input,
        live: Signal::new(String::new()),
        width: Signal::new(0.0),
        origin: Signal::new(((0.0, 0.0), (0.0, 0.0))),
        slot: Signal::new(h::slot_id(&root)),
        on_create,
        on_change,
        on_select,
        on_range_change,
    });

    let s = state.read().clone();
    let i = input.read().clone();
    let days = s.days(&i);
    let n = days.len();
    let shown = h::preview(&s, &i);
    let placed = h::layout(&shown, &days);
    let all_day = h::all_day_rows(&shown, &days);
    let rows = all_day.iter().map(|p| p.lanes).max().unwrap_or(0).max(1);
    // Stable order (the owner's), so moving an event never reorders its element.
    let mut timed = placed.clone();
    timed.sort_unstable_by_key(|p| p.index);
    let mut chips = all_day.clone();
    chips.sort_unstable_by_key(|p| p.index);
    let today_at = days.iter().position(|d| *d == today);
    let now_line = today_at.zip(now());
    let ghost = h::ghost(&s).and_then(|g| days.iter().position(|d| *d == g.date).map(|at| (at, g)));
    let cursor = s.cursor;
    let cursor_at = days.iter().position(|d| *d == cursor.date);
    let pointer_drag = s.drag.is_some();
    let is_disabled = disabled();
    let scroller = h::scroller_id(&root);
    let (event_help, slot_help) = (h::event_help_id(&root), h::slot_help_id(&root));
    let top = h::scroll_top_for(scroll_to_hour);
    let scroll_target = scroller.clone();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "region",
            "aria-label": "{label}",
            "data-view": s.view.as_str(),
            "data-dragging": s.drag.as_ref().map(|d| d.kind.as_str()),
            "data-disabled": is_disabled.then_some(""),
            style: h::root_style(n),
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" && ctx.dragging() {
                    e.prevent_default();
                    ctx.send(WeekSchedulerEvent::Cancel);
                }
            },
            div { "data-scope": h::SCOPE, "data-part": "toolbar",
                div { "data-scope": h::SCOPE, "data-part": "nav",
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "prev",
                        "aria-label": h::nav_label(false, s.view),
                        onclick: move |_| ctx.send(WeekSchedulerEvent::Prev),
                        Icon { name: IconName::ChevronLeft, size: 16 }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "today",
                        onclick: move |_| ctx.send(WeekSchedulerEvent::Today),
                        "Today"
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "next",
                        "aria-label": h::nav_label(true, s.view),
                        onclick: move |_| ctx.send(WeekSchedulerEvent::Next),
                        Icon { name: IconName::ChevronRight, size: 16 }
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "title", {h::range_title(&days)} }
                div { "data-scope": h::SCOPE, "data-part": "views", role: "group", "aria-label": "View",
                    for v in WeekView::ALL {
                        button {
                            key: "{v.as_str()}",
                            r#type: "button",
                            "aria-pressed": (s.view == v).to_string(),
                            onclick: move |_| ctx.send(WeekSchedulerEvent::SetView(v)),
                            {v.label()}
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "header",
                div { "data-scope": h::SCOPE, "data-part": "zone", title: "Time zone", "{timezone}" }
                for d in days.iter().copied() {
                    div {
                        key: "{d}",
                        "data-scope": h::SCOPE,
                        "data-part": "day",
                        "data-today": (d == today).then_some(""),
                        span { "data-scope": h::SCOPE, "data-part": "weekday", {h::day_heading(d).0} }
                        span { "data-scope": h::SCOPE, "data-part": "date", "{d.day()}" }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "allday",
                div { "data-scope": h::SCOPE, "data-part": "allday-label", "All day" }
                div { "data-scope": h::SCOPE, "data-part": "allday-days", style: "--ws-allday-rows: {rows};",
                    for d in 0..n {
                        div {
                            key: "cell-{d}",
                            "data-scope": h::SCOPE,
                            "data-part": "allday-cell",
                            "aria-hidden": "true",
                            style: h::column_style(d, n),
                            onclick: move |_| ctx.send(WeekSchedulerEvent::AllDayCreate(d)),
                        }
                    }
                    for p in chips.iter().copied() {
                        {all_day_chip(ctx, p, &shown, &s, &root, &event_help, n, snap)}
                    }
                }
            }
            div {
                "data-scope": h::SCOPE,
                "data-part": "scroller",
                id: "{scroller}",
                style: "height: {height}px;",
                onmounted: move |_| crate::js::run(
                    &[("id", scroll_target.clone()), ("top", top.to_string())],
                    "const el = document.getElementById(id); if (el) el.scrollTop = Number(top);",
                ),
                div { "data-scope": h::SCOPE, "data-part": "body",
                    div { "data-scope": h::SCOPE, "data-part": "gutter", "aria-hidden": "true",
                        for (m , text) in h::hour_labels() {
                            span { key: "{m}", "data-scope": h::SCOPE, "data-part": "hour", style: "top: {h::minute_y(m)}px;", "{text}" }
                        }
                    }
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "days",
                        onresize: move |e| {
                            if let Ok(size) = e.data().get_border_box_size() {
                                let mut width = ctx.width;
                                width.set(size.width);
                            }
                        },
                        onpointerup: move |_| ctx.release(false),
                        for (col , d) in days.iter().copied().enumerate() {
                            div {
                                key: "{d}",
                                "data-scope": h::SCOPE,
                                "data-part": "column",
                                "aria-hidden": "true",
                                "data-today": (d == today).then_some(""),
                                "data-weekend": matches!(d.weekday(), 0 | 6).then_some(""),
                                style: h::column_style(col, n),
                                onpointerdown: move |e| {
                                    let at = e.data().element_coordinates();
                                    #[allow(clippy::cast_precision_loss, reason = "a handful of days")]
                                    let x = col as f64 * ctx.column() + at.x;
                                    ctx.down(&e, Hit::Empty, (x, at.y));
                                },
                            }
                        }
                        if let Some((at, m)) = now_line {
                            div { "data-scope": h::SCOPE, "data-part": "now", "aria-hidden": "true", style: h::block_style(at, 0, 1, n, m, m) }
                        }
                        for p in timed.iter().copied() {
                            {event_block(ctx, p, &shown, &s, &root, &event_help, n, snap)}
                        }
                        if let Some((at, g)) = ghost {
                            div {
                                "data-scope": h::SCOPE,
                                "data-part": "ghost",
                                "aria-hidden": "true",
                                style: h::block_style(at, 0, 1, n, g.start, g.end),
                                span { "data-scope": h::SCOPE, "data-part": "event-time", {h::time_range(g.start, g.end)} }
                            }
                        }
                        // One element that walks the grid, so it keeps focus as it moves.
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "slot",
                            role: "button",
                            tabindex: "0",
                            id: h::slot_id(&root),
                            hidden: cursor_at.is_none(),
                            "aria-label": h::slot_label(cursor),
                            "aria-describedby": "{slot_help}",
                            "aria-disabled": is_disabled.then_some("true"),
                            style: cursor_at.map(|at| h::block_style(at, 0, 1, n, cursor.start, cursor.end)),
                            onkeydown: move |ev| {
                                if let Some(event) = h::slot_keydown(&ev.key().to_string(), ev.modifiers().shift(), snap) {
                                    ev.prevent_default();
                                    ctx.send(event);
                                }
                            },
                            span { "data-scope": h::SCOPE, "data-part": "event-time", {h::time_range(cursor.start, cursor.end)} }
                        }
                    }
                }
            }
            if pointer_drag {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| ctx.follow(&e),
                    onpointerup: move |_| ctx.release(false),
                    onpointercancel: move |_| ctx.release(true),
                }
            }
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: "{event_help}", {h::EVENT_HELP} }
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: "{slot_help}", {h::SLOT_HELP} }
            div {
                "data-scope": h::SCOPE,
                "data-part": "live",
                role: "status",
                "aria-live": "polite",
                "aria-atomic": "true",
                "{ctx.live}"
            }
        }
    }
}
