//! `<Gantt>`: a task list beside a timeline of bars on a day or week scale.
//! Drag a bar to move it, drag an edge to resize it (both snap to days), or
//! pick it up with Space and use the arrow keys. Dependencies are drawn as
//! connectors and a line marks today.

use crate::drive::drive;
use hydroxy_headless::gantt::{
    self as h, Drag, DragMode, GanttCommand, GanttEvent, GanttInput, GanttMachine, GanttState, Timeline,
    announcement_text,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::focus::focus_id;
use crate::modifiers::key_mods;

pub use hydroxy_headless::gantt::{Date, GanttChange, GanttScale, GanttTask, apply_change};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<GanttState>,
    input: Memo<GanttInput>,
    live: RwSignal<String>,
    /// Pointer x when the current drag began.
    origin_x: StoredValue<f64>,
    on_change: Option<Callback<GanttChange>>,
    on_scale_change: Option<Callback<GanttScale>>,
}

impl Ctx {
    fn send(self, event: GanttEvent) {
        for command in drive::<GanttMachine, _>(self.state, self.input, event) {
            match command {
                GanttCommand::Change(change) => {
                    if let Some(cb) = self.on_change {
                        cb.run(change);
                    }
                }
                GanttCommand::ScaleChange(scale) => {
                    if let Some(cb) = self.on_scale_change {
                        cb.run(scale);
                    }
                }
                GanttCommand::FocusBar(task) => focus_id(&h::bar_id(&self.id.get_value(), &task)),
                GanttCommand::Announce(a) => self.live.set(announcement_text(&a)),
            }
        }
    }
}

/// A Gantt chart. Dates are [`Date`]s; ranges are inclusive.
#[component]
pub fn Gantt(
    /// Controlled tasks. Pair with `on_change` and [`apply_change`].
    #[prop(optional, into)]
    tasks: MaybeProp<Vec<GanttTask>>,
    /// Initial tasks when uncontrolled.
    #[prop(optional, into)]
    default_tasks: Vec<GanttTask>,
    /// Controlled scale. Pair with `on_scale_change`.
    #[prop(optional, into)]
    scale: MaybeProp<GanttScale>,
    /// Initial scale when uncontrolled. Default day.
    #[prop(optional)]
    default_scale: GanttScale,
    /// The date to draw the today line at. None draws no line.
    #[prop(optional, into)]
    today: MaybeProp<Date>,
    /// Block every move.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called when a task's dates change.
    #[prop(optional, into)]
    on_change: Option<Callback<GanttChange>>,
    /// Called when the scale toggle is used.
    #[prop(optional, into)]
    on_scale_change: Option<Callback<GanttScale>>,
) -> impl IntoView {
    let input = Memo::new(move |_| GanttInput {
        tasks: tasks.get(),
        default_tasks: default_tasks.clone(),
        scale: scale.get(),
        default_scale,
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(GanttMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| GanttMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let ctx = Ctx {
        id,
        state,
        input,
        live: RwSignal::new(String::new()),
        origin_x: StoredValue::new(0.0),
        on_change,
        on_scale_change,
    };
    let root = id.get_value();

    let preview = Memo::new(move |_| state.with(GanttState::preview));
    // The axis fits the committed tasks, so it holds still while a bar is dragged.
    let timeline = Memo::new(move |_| state.with(|s| Timeline::fit(s.scale.value, &s.tasks.value, today.get())));
    let ids = Memo::new(move |_| state.with(|s| s.tasks.value.iter().map(|t| t.id.clone()).collect::<Vec<_>>()));
    let connectors = Memo::new(move |_| timeline.with(|t| preview.with(|p| t.connectors(p))));
    let rows_height = move || f64::from(u32::try_from(ids.with(Vec::len)).unwrap_or(u32::MAX)) * h::ROW_HEIGHT;

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-scale=move || state.with(|s| s.scale.value.as_str())
            data-dragging=move || state.with(|s| s.drag.is_some()).then_some("")
            style=h::root_style()
        >
            <div data-scope=h::SCOPE data-part="toolbar">
                <div role="group" aria-label="Scale" data-scope=h::SCOPE data-part="scale-toggle">
                    {[GanttScale::Day, GanttScale::Week]
                        .into_iter()
                        .map(|choice| {
                            view! {
                                <button
                                    type="button"
                                    aria-pressed=move || state.with(|s| (s.scale.value == choice).to_string())
                                    on:click=move |_| ctx.send(GanttEvent::SetScale(choice))
                                >
                                    {if choice == GanttScale::Day { "Day" } else { "Week" }}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root)>{h::INSTRUCTIONS}</p>
            <div data-scope=h::SCOPE data-part="body">
                <div data-scope=h::SCOPE data-part="list">
                    <div data-scope=h::SCOPE data-part="list-header">"Task"</div>
                    <ol data-scope=h::SCOPE data-part="rows" aria-label="Tasks">
                        <For each=move || ids.get() key=String::clone children=move |task| row_view(preview, task) />
                    </ol>
                </div>
                <div data-scope=h::SCOPE data-part="scroller">
                    <div
                        data-scope=h::SCOPE
                        data-part="timeline"
                        style=move || format!("width: {}px;", timeline.get().width())
                    >
                        <div data-scope=h::SCOPE data-part="header" aria-hidden="true">
                            <div data-scope=h::SCOPE data-part="months">
                                {move || {
                                    timeline
                                        .get()
                                        .months()
                                        .into_iter()
                                        .map(|m| {
                                            view! {
                                                <span
                                                    data-scope=h::SCOPE
                                                    data-part="month"
                                                    style=format!("left: {}px; width: {}px;", m.x, m.width)
                                                >
                                                    {m.label}
                                                </span>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                            <div data-scope=h::SCOPE data-part="ticks">
                                {move || {
                                    timeline
                                        .get()
                                        .ticks()
                                        .into_iter()
                                        .map(|t| {
                                            view! {
                                                <span
                                                    data-scope=h::SCOPE
                                                    data-part="tick"
                                                    data-major=t.major.then_some("")
                                                    data-weekend=t.weekend.then_some("")
                                                    style=format!("left: {}px; width: {}px;", t.x, t.width)
                                                >
                                                    {t.label}
                                                </span>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </div>
                        <div
                            role="group"
                            aria-label="Timeline"
                            data-scope=h::SCOPE
                            data-part="grid"
                            style=move || {
                                let t = timeline.get();
                                let column = if t.scale == GanttScale::Week { 7.0 * t.px_per_day } else { t.px_per_day };
                                format!("height: {}px; --gantt-column: {column}px;", rows_height())
                            }
                        >
                            <svg
                                data-scope=h::SCOPE
                                data-part="connectors"
                                aria-hidden="true"
                                width=move || timeline.get().width()
                                height=rows_height
                            >
                                {move || {
                                    connectors
                                        .get()
                                        .into_iter()
                                        .map(|c| {
                                            view! {
                                                <path data-scope=h::SCOPE data-part="connector" d=c.line fill="none" />
                                                <path data-scope=h::SCOPE data-part="arrow" d=c.arrow />
                                            }
                                        })
                                        .collect_view()
                                }}
                            </svg>
                            {move || {
                                today
                                    .get()
                                    .and_then(|d| timeline.with(|t| t.today_x(d)))
                                    .map(|x| view! {
                                        <div data-scope=h::SCOPE data-part="today" title="Today" style=format!("left: {x}px;")></div>
                                    })
                            }}
                            <For
                                each=move || ids.get()
                                key=String::clone
                                children={
                                    let root = root.clone();
                                    move |task| bar_view(ctx, root.clone(), preview, timeline, task)
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || ctx.live.get()}
            </div>
        </div>
    }
}

fn row_view(preview: Memo<Vec<GanttTask>>, task: String) -> impl IntoView {
    let item = Memo::new(move |_| preview.with(|p| p.iter().find(|t| t.id == task).cloned()));
    view! {
        <li data-scope=h::SCOPE data-part="row">
            <span data-scope=h::SCOPE data-part="row-title">{move || item.with(|t| t.as_ref().map(|t| t.title.clone()))}</span>
            <span data-scope=h::SCOPE data-part="row-dates">
                {move || item.with(|t| t.as_ref().map(|t| h::range_text(t.start, t.end)))}
            </span>
        </li>
    }
}

fn bar_view(
    ctx: Ctx,
    root: String,
    preview: Memo<Vec<GanttTask>>,
    timeline: Memo<Timeline>,
    task: String,
) -> impl IntoView {
    let row = {
        let task = task.clone();
        Memo::new(move |_| preview.with(|p| p.iter().position(|t| t.id == task)))
    };
    let attrs =
        Memo::new(move |_| {
            let row = row.get()?;
            Some(ctx.state.with(|s| {
                ctx.input.with(|i| preview.with(|p| timeline.with(|t| h::bar_attrs(&root, s, i, p, t, row))))
            }))
        });
    let title = move || row.get().and_then(|r| preview.with(|p| p.get(r).map(|t| t.title.clone())));

    // Pointer down on the bar moves it; on an edge handle it resizes.
    let begin = {
        let task = task.clone();
        move |ev: leptos::ev::PointerEvent, mode: DragMode| {
            if ev.button() != 0 || ctx.input.with_untracked(|i| i.disabled) {
                return;
            }
            ev.stop_propagation();
            if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                let _ = el.set_pointer_capture(ev.pointer_id());
            }
            ctx.origin_x.set_value(f64::from(ev.client_x()));
            ctx.send(GanttEvent::DragStart { id: task.clone(), mode });
        }
    };
    let (begin_start, begin_end, begin_move) = (begin.clone(), begin.clone(), begin);
    let keyed = task;

    view! {
        <div
            role="button"
            id=move || attrs.get().map(|a| a.id)
            tabindex=move || attrs.get().is_some_and(|a| a.movable).then_some(0)
            aria-label=move || attrs.get().map(|a| a.label)
            aria-pressed=move || attrs.get().map(|a| a.pressed.to_string())
            aria-describedby=move || attrs.get().map(|a| a.described_by)
            data-scope=h::SCOPE
            data-part="bar"
            data-state=move || attrs.get().map(|a| a.state)
            style=move || attrs.get().map(|a| a.style)
            on:pointerdown=move |ev| begin_move(ev, DragMode::Move)
            on:pointermove=move |ev| {
                let dx = f64::from(ev.client_x()) - ctx.origin_x.get_value();
                let days = ctx.state.with_untracked(|s| {
                    s.drag.as_ref().filter(|d: &&Drag| !d.keyboard).map(|d| h::pointer_days(&timeline.get_untracked(), d, dx))
                });
                if let Some(days) = days {
                    ctx.send(GanttEvent::DragBy(days));
                }
            }
            on:pointerup=move |_| {
                if ctx.state.with_untracked(|s| s.drag.as_ref().is_some_and(|d| !d.keyboard)) {
                    ctx.send(GanttEvent::Drop);
                }
            }
            on:pointercancel=move |_| {
                if ctx.state.with_untracked(|s| s.drag.as_ref().is_some_and(|d| !d.keyboard)) {
                    ctx.send(GanttEvent::Cancel);
                }
            }
            on:keydown=move |ev| {
                let event = ctx.state.with_untracked(|s| h::bar_keydown(&ev.key(), key_mods(&ev), &keyed, s));
                if let Some(event) = event {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            <span
                aria-hidden="true"
                data-scope=h::SCOPE
                data-part="handle"
                data-edge="start"
                on:pointerdown=move |ev| begin_start(ev, DragMode::ResizeStart)
            ></span>
            <span data-scope=h::SCOPE data-part="bar-label">{title}</span>
            <span
                aria-hidden="true"
                data-scope=h::SCOPE
                data-part="handle"
                data-edge="end"
                on:pointerdown=move |ev| begin_end(ev, DragMode::ResizeEnd)
            ></span>
        </div>
    }
}
