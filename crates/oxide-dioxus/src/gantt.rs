//! `Gantt`: a task list beside a timeline of bars on a day or week scale.
//! Drag a bar to move it, drag an edge to resize it (both snap to days), or
//! pick it up with Space and use the arrow keys. Dependencies are drawn as
//! connectors and a line marks today.
//!
//! Dioxus has no pointer capture on every renderer, so while a bar is being
//! dragged a fixed full-window layer catches the pointer instead.

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use oxide_headless::gantt::{
    self as h, DragMode, GanttCommand, GanttEvent, GanttInput, GanttMachine, GanttState, Timeline, announcement_text,
};
use oxide_statechart::Machine;

use crate::focus::FocusMap;

pub use oxide_headless::gantt::{Date, GanttChange, GanttScale, GanttTask, apply_change};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<GanttState>,
    input: Memo<GanttInput>,
    live: Signal<String>,
    bars: FocusMap,
    /// Pointer x when the current drag began.
    origin_x: Signal<f64>,
    on_change: Option<EventHandler<GanttChange>>,
    on_scale_change: Option<EventHandler<GanttScale>>,
    id: Signal<String>,
}

impl Ctx {
    fn send(mut self, event: GanttEvent) {
        let step = GanttMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state
            && *self.state.peek() != next
        {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                GanttCommand::Change(change) => {
                    if let Some(handler) = self.on_change {
                        handler.call(change);
                    }
                }
                GanttCommand::ScaleChange(scale) => {
                    if let Some(handler) = self.on_scale_change {
                        handler.call(scale);
                    }
                }
                GanttCommand::FocusBar(task) => self.bars.focus(&task),
                GanttCommand::Announce(a) => self.live.set(announcement_text(&a)),
            }
        }
    }

    /// Start a drag of `mode` on `task` from pointer x `x`.
    fn begin(mut self, task: &str, mode: DragMode, x: f64) {
        if self.input.peek().disabled {
            return;
        }
        self.origin_x.set(x);
        self.send(GanttEvent::DragStart { id: task.to_string(), mode });
    }

    /// The pointer is at `x`: tell the machine how many whole days that is.
    fn pointer_to(self, x: f64, timeline: &Timeline) {
        let dx = x - *self.origin_x.peek();
        let days = self.state.peek().drag.as_ref().filter(|d| !d.keyboard).map(|d| h::pointer_days(timeline, d, dx));
        if let Some(days) = days {
            self.send(GanttEvent::DragBy(days));
        }
    }

    fn release(self, cancel: bool) {
        if self.state.peek().drag.as_ref().is_some_and(|d| !d.keyboard) {
            self.send(if cancel { GanttEvent::Cancel } else { GanttEvent::Drop });
        }
    }
}

/// A Gantt chart. Dates are [`Date`]s; ranges are inclusive.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Gantt(
    /// Controlled tasks. Pair with `on_change` and [`apply_change`].
    #[props(default)]
    tasks: ReadSignal<Option<Vec<GanttTask>>>,
    /// Initial tasks when uncontrolled.
    #[props(default)]
    default_tasks: Vec<GanttTask>,
    /// Controlled scale. Pair with `on_scale_change`.
    #[props(default)]
    scale: ReadSignal<Option<GanttScale>>,
    /// Initial scale when uncontrolled. Default day.
    #[props(default)]
    default_scale: GanttScale,
    /// The date to draw the today line at. None draws no line.
    #[props(default)]
    today: ReadSignal<Option<Date>>,
    /// Block every move.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called when a task's dates change.
    on_change: Option<EventHandler<GanttChange>>,
    /// Called when the scale toggle is used.
    on_scale_change: Option<EventHandler<GanttScale>>,
) -> Element {
    let input = use_memo(move || GanttInput {
        tasks: tasks(),
        default_tasks: default_tasks.clone(),
        scale: scale(),
        default_scale,
        disabled: disabled(),
    });
    let mut state = use_signal(|| GanttMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = GanttMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx {
        state,
        input,
        live: Signal::new(String::new()),
        bars: FocusMap::new(),
        origin_x: Signal::new(0.0),
        on_change,
        on_scale_change,
        id: Signal::new(root.clone()),
    });

    let s = state.read().clone();
    let i = input.read().clone();
    let preview = s.preview();
    let timeline = Timeline::fit(s.scale, &s.tasks, today());
    let connectors = timeline.connectors(&preview);
    let rows_height = f64::from(u32::try_from(preview.len()).unwrap_or(u32::MAX)) * h::ROW_HEIGHT;
    let column = if timeline.scale == GanttScale::Week { 7.0 * timeline.px_per_day } else { timeline.px_per_day };
    let today_x = today().and_then(|d| timeline.today_x(d));
    let pointer_drag = s.drag.as_ref().is_some_and(|d| !d.keyboard);
    let bar_root = ctx.id.peek().clone();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-scale": s.scale.as_str(),
            "data-dragging": s.drag.is_some().then_some(""),
            style: h::root_style(),
            div { "data-scope": h::SCOPE, "data-part": "toolbar",
                div { role: "group", "aria-label": "Scale", "data-scope": h::SCOPE, "data-part": "scale-toggle",
                    for choice in [GanttScale::Day, GanttScale::Week] {
                        button {
                            key: "{choice.as_str()}",
                            r#type: "button",
                            "aria-pressed": (s.scale == choice).to_string(),
                            onclick: move |_| ctx.send(GanttEvent::SetScale(choice)),
                            if choice == GanttScale::Day { "Day" } else { "Week" }
                        }
                    }
                }
            }
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: h::instructions_id(&root), {h::INSTRUCTIONS} }
            div { "data-scope": h::SCOPE, "data-part": "body",
                div { "data-scope": h::SCOPE, "data-part": "list",
                    div { "data-scope": h::SCOPE, "data-part": "list-header", "Task" }
                    ol { "data-scope": h::SCOPE, "data-part": "rows", "aria-label": "Tasks",
                        for task in preview.iter() {
                            li { key: "{task.id}", "data-scope": h::SCOPE, "data-part": "row",
                                span { "data-scope": h::SCOPE, "data-part": "row-title", "{task.title}" }
                                span { "data-scope": h::SCOPE, "data-part": "row-dates", {h::range_text(task.start, task.end)} }
                            }
                        }
                    }
                }
                div { "data-scope": h::SCOPE, "data-part": "scroller",
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "timeline",
                        style: "width: {timeline.width()}px;",
                        div { "data-scope": h::SCOPE, "data-part": "header", "aria-hidden": "true",
                            div { "data-scope": h::SCOPE, "data-part": "months",
                                for (n , m) in timeline.months().into_iter().enumerate() {
                                    span {
                                        key: "{n}",
                                        "data-scope": h::SCOPE,
                                        "data-part": "month",
                                        style: "left: {m.x}px; width: {m.width}px;",
                                        "{m.label}"
                                    }
                                }
                            }
                            div { "data-scope": h::SCOPE, "data-part": "ticks",
                                for (n , t) in timeline.ticks().into_iter().enumerate() {
                                    span {
                                        key: "{n}",
                                        "data-scope": h::SCOPE,
                                        "data-part": "tick",
                                        "data-major": t.major.then_some(""),
                                        "data-weekend": t.weekend.then_some(""),
                                        style: "left: {t.x}px; width: {t.width}px;",
                                        "{t.label}"
                                    }
                                }
                            }
                        }
                        div {
                            role: "group",
                            "aria-label": "Timeline",
                            "data-scope": h::SCOPE,
                            "data-part": "grid",
                            style: "height: {rows_height}px; --gantt-column: {column}px;",
                            svg {
                                "data-scope": h::SCOPE,
                                "data-part": "connectors",
                                "aria-hidden": "true",
                                width: "{timeline.width()}",
                                height: "{rows_height}",
                                for (n , c) in connectors.into_iter().enumerate() {
                                    path { key: "line-{n}", "data-scope": h::SCOPE, "data-part": "connector", d: c.line, fill: "none" }
                                    path { "data-scope": h::SCOPE, "data-part": "arrow", d: c.arrow }
                                }
                            }
                            if let Some(x) = today_x {
                                div { "data-scope": h::SCOPE, "data-part": "today", title: "Today", style: "left: {x}px;" }
                            }
                            for (row , task) in preview.iter().enumerate() {
                                {
                                    let a = h::bar_attrs(&bar_root, &s, &i, &preview, &timeline, row);
                                    let (mounted, keyed, moving, start_edge, end_edge) =
                                        (task.id.clone(), task.id.clone(), task.id.clone(), task.id.clone(), task.id.clone());
                                    rsx! {
                                        div {
                                            key: "{task.id}",
                                            role: "button",
                                            id: a.id,
                                            tabindex: a.movable.then_some("0"),
                                            "aria-label": a.label,
                                            "aria-pressed": a.pressed.to_string(),
                                            "aria-describedby": a.described_by,
                                            "data-scope": h::SCOPE,
                                            "data-part": "bar",
                                            "data-state": a.state,
                                            style: a.style,
                                            onmounted: move |e| ctx.bars.register(mounted.clone(), e.data()),
                                            onpointerdown: move |e| {
                                                if e.trigger_button() == Some(MouseButton::Primary) {
                                                    ctx.begin(&moving, DragMode::Move, e.client_coordinates().x);
                                                }
                                            },
                                            onkeydown: move |e| {
                                                let m = e.modifiers();
                                                let event = h::bar_keydown(&e.key().to_string(), m.shift(), m.alt(), &keyed, &ctx.state.peek());
                                                if let Some(event) = event {
                                                    e.prevent_default();
                                                    ctx.send(event);
                                                }
                                            },
                                            span {
                                                "aria-hidden": "true",
                                                "data-scope": h::SCOPE,
                                                "data-part": "handle",
                                                "data-edge": "start",
                                                onpointerdown: move |e| {
                                                    if e.trigger_button() == Some(MouseButton::Primary) {
                                                        e.stop_propagation();
                                                        ctx.begin(&start_edge, DragMode::ResizeStart, e.client_coordinates().x);
                                                    }
                                                },
                                            }
                                            span { "data-scope": h::SCOPE, "data-part": "bar-label", "{task.title}" }
                                            span {
                                                "aria-hidden": "true",
                                                "data-scope": h::SCOPE,
                                                "data-part": "handle",
                                                "data-edge": "end",
                                                onpointerdown: move |e| {
                                                    if e.trigger_button() == Some(MouseButton::Primary) {
                                                        e.stop_propagation();
                                                        ctx.begin(&end_edge, DragMode::ResizeEnd, e.client_coordinates().x);
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if pointer_drag {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| ctx.pointer_to(e.client_coordinates().x, &timeline),
                    onpointerup: move |_| ctx.release(false),
                    onpointercancel: move |_| ctx.release(true),
                }
            }
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
