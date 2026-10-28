//! `<WeekScheduler>`: events on a time grid in day, work-week or week view.
//! Drag on empty time to create an event, drag an event to move it (across days
//! too), drag its bottom edge to resize it; everything snaps to `snap` minutes.
//! The owner holds the events and hears about each change through callbacks.

use leptos::html;
use leptos::prelude::*;
use hydroxy_headless::week_scheduler::{
    self as h, GridPoint, Hit, Placed, WeekSchedulerCommand, WeekSchedulerEvent, WeekSchedulerInput,
    WeekSchedulerMachine, WeekSchedulerState,
};
use hydroxy_statechart::Machine;

pub use h::{Date, Placement, WeekEvent, WeekView, apply_change, event_label, placement_text};

#[derive(Clone, Copy)]
struct Ctx {
    root: StoredValue<String>,
    state: RwSignal<WeekSchedulerState>,
    input: Memo<WeekSchedulerInput>,
    live: RwSignal<String>,
    days: NodeRef<html::Div>,
    scroller: NodeRef<html::Div>,
    on_create: Option<Callback<Placement>>,
    on_change: Option<Callback<WeekEvent>>,
    on_select: Option<Callback<String>>,
    on_range_change: Option<Callback<(Date, Date)>>,
}

#[allow(clippy::cast_possible_truncation, reason = "a scroll offset in px fits in i32")]
fn px(v: f64) -> i32 {
    v.round() as i32
}

impl Ctx {
    fn send(self, event: WeekSchedulerEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| WeekSchedulerMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                WeekSchedulerCommand::Create(p) => {
                    if let Some(cb) = self.on_create {
                        cb.run(p);
                    }
                }
                WeekSchedulerCommand::Change(e) => {
                    if let Some(cb) = self.on_change {
                        cb.run(e);
                    }
                }
                WeekSchedulerCommand::Select(id) => {
                    if let Some(cb) = self.on_select {
                        cb.run(id);
                    }
                }
                WeekSchedulerCommand::RangeChange(a, b) => {
                    if let Some(cb) = self.on_range_change {
                        cb.run((a, b));
                    }
                }
                WeekSchedulerCommand::RevealCursor => self.reveal(),
                WeekSchedulerCommand::Announce(text) => self.live.set(text),
            }
        }
    }

    fn reveal(self) {
        let Some(el) = self.scroller.get_untracked() else { return };
        let c = self.state.with_untracked(|s| s.cursor);
        let (top, height) = (f64::from(el.scroll_top()), f64::from(el.client_height()));
        if let Some(to) = h::reveal_scroll(c.start, c.end, top, height) {
            el.set_scroll_top(px(to));
        }
    }

    fn dragging(self) -> bool {
        self.state.with_untracked(|s| s.drag.is_some())
    }

    /// The grid point under a pointer event, from the days layer's box.
    fn point(self, ev: &leptos::ev::PointerEvent) -> GridPoint {
        let n = self.input.with_untracked(|i| self.state.with_untracked(|s| s.days(i).len()));
        self.days.get_untracked().map_or(GridPoint { day: 0, minute: 0 }, |el| {
            let r = el.get_bounding_client_rect();
            h::point_at(f64::from(ev.client_x()) - r.left(), f64::from(ev.client_y()) - r.top(), r.width(), n)
        })
    }

    /// Pointer down on the grid: capture the pointer on the days layer and start a drag.
    fn down(self, ev: &leptos::ev::PointerEvent, hit: Hit) {
        if ev.button() != 0 || self.input.with_untracked(|i| i.disabled) {
            return;
        }
        ev.stop_propagation();
        if let Some(el) = self.days.get_untracked() {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        self.send(WeekSchedulerEvent::PointerDown { hit, at: self.point(ev) });
    }
}

/// A week of events on a time grid. Hours run down, days across; all-day
/// events sit in a row above. The grid opens scrolled to `scroll_to_hour`.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
pub fn WeekScheduler(
    /// Accessible name for the whole scheduler, such as "Team calendar".
    #[prop(into)]
    label: String,
    /// Every event. Update it from `on_create` and `on_change`.
    #[prop(into)]
    events: Signal<Vec<WeekEvent>>,
    /// Today: the Today button's target and the highlighted column.
    today: Date,
    /// The date shown first. Default today.
    #[prop(optional)]
    default_date: Option<Date>,
    /// The view shown first. Default week.
    #[prop(optional)]
    default_view: WeekView,
    /// First day of the week, 0 = Sunday. Default 1 (Monday).
    #[prop(default = 1)]
    week_start: u8,
    /// Minutes drags and arrow keys snap to. Default 15.
    #[prop(default = 15)]
    snap: u16,
    /// Length in minutes of an event made by a click or Enter. Default 60.
    #[prop(default = 60)]
    default_duration: u16,
    /// The time zone the times are in, shown in the corner, such as "GMT".
    #[prop(into)]
    timezone: String,
    /// Minutes after midnight now, for the line across today. None draws no line.
    #[prop(optional, into)]
    now: MaybeProp<u16>,
    /// Hour shown at the top when the grid opens. Default 8.
    #[prop(default = 8)]
    scroll_to_hour: u8,
    /// Height of the scrolling grid in px. Default 560.
    #[prop(default = 560)]
    height: u32,
    /// Block creating, moving and resizing.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// A new event was drawn. Add one with this placement.
    #[prop(optional, into)]
    on_create: Option<Callback<Placement>>,
    /// An event was moved or resized. Store it, for example with [`apply_change`].
    #[prop(optional, into)]
    on_change: Option<Callback<WeekEvent>>,
    /// An event was clicked or Enter was pressed on it.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
    /// The visible days changed, first to last.
    #[prop(optional, into)]
    on_range_change: Option<Callback<(Date, Date)>>,
) -> impl IntoView {
    let input = Memo::new(move |_| WeekSchedulerInput {
        events: events.get(),
        today,
        default_date: default_date.unwrap_or(today),
        default_view,
        week_start,
        snap,
        default_duration,
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(WeekSchedulerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| WeekSchedulerMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx {
        root: StoredValue::new(crate::id::next_id(h::SCOPE)),
        state,
        input,
        live: RwSignal::new(String::new()),
        days: NodeRef::new(),
        scroller: NodeRef::new(),
        on_create,
        on_change,
        on_select,
        on_range_change,
    };
    let root = ctx.root.get_value();

    // Open on working hours.
    Effect::new(move |_| {
        if let Some(el) = ctx.scroller.get() {
            el.set_scroll_top(px(h::scroll_top_for(scroll_to_hour)));
        }
    });

    let days = Memo::new(move |_| input.with(|i| state.with(|s| s.days(i))));
    let count = Memo::new(move |_| days.with(Vec::len));
    let shown = Memo::new(move |_| input.with(|i| state.with(|s| h::preview(s, i))));
    let placed = Memo::new(move |_| shown.with(|e| days.with(|d| h::layout(e, d))));
    let all_day = Memo::new(move |_| shown.with(|e| days.with(|d| h::all_day_rows(e, d))));
    let rows = Memo::new(move |_| all_day.with(|a| a.iter().map(|p| p.lanes).max().unwrap_or(0)));
    // Stable order (the owner's), so moving an event never reorders its element.
    let ids = move |list: Memo<Vec<Placed>>| {
        move || {
            shown.with(|e| {
                list.with(|l| {
                    let mut idx: Vec<usize> = l.iter().map(|p| p.index).collect();
                    idx.sort_unstable();
                    idx.into_iter().map(|i| e[i].id.clone()).collect::<Vec<_>>()
                })
            })
        }
    };
    let (timed_ids, all_day_ids) = (ids(placed), ids(all_day));
    let is_today = move |d: Date| d == today;
    let cursor = Memo::new(move |_| state.with(|s| s.cursor));
    let cursor_at = Memo::new(move |_| {
        let c = cursor.get();
        days.with(|d| d.iter().position(|x| *x == c.date))
    });

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="region"
            aria-label=label
            data-view=move || state.with(|s| s.view.as_str())
            data-dragging=move || state.with(|s| s.drag.as_ref().map(|d| d.kind.as_str()))
            data-disabled=move || disabled.get().then_some("")
            style=move || h::root_style(count.get())
            on:keydown=move |ev| {
                if ev.key() == "Escape" && ctx.dragging() {
                    ev.prevent_default();
                    ctx.send(WeekSchedulerEvent::Cancel);
                }
            }
        >
            <div data-scope=h::SCOPE data-part="toolbar">
                <div data-scope=h::SCOPE data-part="nav">
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="prev"
                        aria-label=move || h::nav_label(false, state.with(|s| s.view))
                        on:click=move |_| ctx.send(WeekSchedulerEvent::Prev)
                    >
                        {chevron("m15 18-6-6 6-6")}
                    </button>
                    <button type="button" data-scope=h::SCOPE data-part="today" on:click=move |_| ctx.send(WeekSchedulerEvent::Today)>
                        "Today"
                    </button>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="next"
                        aria-label=move || h::nav_label(true, state.with(|s| s.view))
                        on:click=move |_| ctx.send(WeekSchedulerEvent::Next)
                    >
                        {chevron("m9 18 6-6-6-6")}
                    </button>
                </div>
                <span data-scope=h::SCOPE data-part="title">{move || days.with(|d| h::range_title(d))}</span>
                <div data-scope=h::SCOPE data-part="views" role="group" aria-label="View">
                    {WeekView::ALL
                        .into_iter()
                        .map(|v| {
                            view! {
                                <button
                                    type="button"
                                    aria-pressed=move || state.with(|s| (s.view == v).to_string())
                                    on:click=move |_| ctx.send(WeekSchedulerEvent::SetView(v))
                                >
                                    {v.label()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <div data-scope=h::SCOPE data-part="header">
                <div data-scope=h::SCOPE data-part="zone" title="Time zone">{timezone}</div>
                {move || {
                    days.get()
                        .into_iter()
                        .map(|d| {
                            let (weekday, day) = h::day_heading(d);
                            view! {
                                <div
                                    data-scope=h::SCOPE
                                    data-part="day"
                                    data-today=is_today(d).then_some("")
                                >
                                    <span data-scope=h::SCOPE data-part="weekday">{weekday}</span>
                                    <span data-scope=h::SCOPE data-part="date">{day}</span>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <div data-scope=h::SCOPE data-part="allday">
                <div data-scope=h::SCOPE data-part="allday-label">"All day"</div>
                <div
                    data-scope=h::SCOPE
                    data-part="allday-days"
                    style=move || format!("--ws-allday-rows: {};", rows.get().max(1))
                >
                    {move || {
                        (0..count.get())
                            .map(|d| {
                                view! {
                                    <div
                                        data-scope=h::SCOPE
                                        data-part="allday-cell"
                                        aria-hidden="true"
                                        style=h::column_style(d, count.get_untracked())
                                        on:click=move |_| ctx.send(WeekSchedulerEvent::AllDayCreate(d))
                                    ></div>
                                }
                            })
                            .collect_view()
                    }}
                    <For each=all_day_ids key=String::clone children=move |id| all_day_view(ctx, &id, shown, all_day, count) />
                </div>
            </div>
            <div
                data-scope=h::SCOPE
                data-part="scroller"
                id=h::scroller_id(&root)
                node_ref=ctx.scroller
                style=format!("height: {height}px;")
            >
                <div data-scope=h::SCOPE data-part="body">
                    <div data-scope=h::SCOPE data-part="gutter" aria-hidden="true">
                        {h::hour_labels()
                            .into_iter()
                            .map(|(m, text)| {
                                view! {
                                    <span data-scope=h::SCOPE data-part="hour" style=format!("top: {}px;", h::minute_y(m))>
                                        {text}
                                    </span>
                                }
                            })
                            .collect_view()}
                    </div>
                    <div
                        data-scope=h::SCOPE
                        data-part="days"
                        node_ref=ctx.days
                        on:pointerdown=move |ev| ctx.down(&ev, Hit::Empty)
                        on:pointermove=move |ev| {
                            if ctx.dragging() {
                                ctx.send(WeekSchedulerEvent::PointerMove(ctx.point(&ev)));
                            }
                        }
                        on:pointerup=move |_| {
                            if ctx.dragging() {
                                ctx.send(WeekSchedulerEvent::PointerUp);
                            }
                        }
                        on:pointercancel=move |_| {
                            if ctx.dragging() {
                                ctx.send(WeekSchedulerEvent::Cancel);
                            }
                        }
                    >
                        {move || {
                            let n = count.get();
                            days.get()
                                .into_iter()
                                .enumerate()
                                .map(|(i, d)| {
                                    view! {
                                        <div
                                            data-scope=h::SCOPE
                                            data-part="column"
                                            aria-hidden="true"
                                            data-today=is_today(d).then_some("")
                                            data-weekend=matches!(d.weekday(), 0 | 6).then_some("")
                                            style=h::column_style(i, n)
                                        ></div>
                                    }
                                })
                                .collect_view()
                        }}
                        {move || {
                            let n = count.get();
                            let at = days.with(|d| d.iter().position(|x| *x == today))?;
                            let m = now.get()?;
                            Some(view! {
                                <div
                                    data-scope=h::SCOPE
                                    data-part="now"
                                    aria-hidden="true"
                                    style=h::block_style(at, 0, 1, n, m, m)
                                ></div>
                            })
                        }}
                        <For each=timed_ids key=String::clone children=move |id| event_view(ctx, &id, shown, placed, count) />
                        {move || {
                            let g = state.with(h::ghost)?;
                            let at = days.with(|d| d.iter().position(|x| *x == g.date))?;
                            Some(view! {
                                <div
                                    data-scope=h::SCOPE
                                    data-part="ghost"
                                    aria-hidden="true"
                                    style=h::block_style(at, 0, 1, count.get(), g.start, g.end)
                                >
                                    <span data-scope=h::SCOPE data-part="event-time">{h::time_range(g.start, g.end)}</span>
                                </div>
                            })
                        }}
                        // One element that walks the grid, so it keeps focus as it moves.
                        <div
                            data-scope=h::SCOPE
                            data-part="slot"
                            role="button"
                            tabindex="0"
                            id=h::slot_id(&root)
                            hidden=move || cursor_at.get().is_none()
                            aria-label=move || h::slot_label(cursor.get())
                            aria-describedby=h::slot_help_id(&root)
                            aria-disabled=move || disabled.get().then_some("true")
                            style=move || {
                                let c = cursor.get();
                                cursor_at.get().map(|at| h::block_style(at, 0, 1, count.get(), c.start, c.end))
                            }
                            on:keydown=move |ev| {
                                if let Some(e) = h::slot_keydown(&ev.key(), ev.shift_key(), snap) {
                                    ev.prevent_default();
                                    ctx.send(e);
                                }
                            }
                        >
                            <span data-scope=h::SCOPE data-part="event-time">
                                {move || {
                                    let c = cursor.get();
                                    h::time_range(c.start, c.end)
                                }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
            <p data-scope=h::SCOPE data-part="instructions" id=h::event_help_id(&root)>{h::EVENT_HELP}</p>
            <p data-scope=h::SCOPE data-part="instructions" id=h::slot_help_id(&root)>{h::SLOT_HELP}</p>
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || ctx.live.get()}
            </div>
        </div>
    }
}

fn chevron(d: &'static str) -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d=d />
        </svg>
    }
}

type Found = Option<(WeekEvent, Placed)>;

fn find(shown: Memo<Vec<WeekEvent>>, list: Memo<Vec<Placed>>, id: String) -> Memo<Found> {
    Memo::new(move |_| {
        list.with(|l| shown.with(|e| l.iter().find(|p| e[p.index].id == id).map(|p| (e[p.index].clone(), *p))))
    })
}

fn event_view(ctx: Ctx, id: &str, shown: Memo<Vec<WeekEvent>>, placed: Memo<Vec<Placed>>, count: Memo<usize>) -> impl IntoView + use<> {
    let item = find(shown, placed, id.to_owned());
    let root = ctx.root.get_value();
    let selected = {
        let id = id.to_owned();
        Memo::new(move |_| ctx.state.with(|s| s.selected.as_deref() == Some(id.as_str())))
    };
    let dragged = {
        let id = id.to_owned();
        move || ctx.state.with(|s| s.drag.as_ref().is_some_and(|d| d.id.as_deref() == Some(id.as_str())))
    };
    let (moving, resizing, keyed) = (id.to_owned(), id.to_owned(), id.to_owned());
    let snap = ctx.input.with_untracked(|i| i.snap);

    view! {
        <div
            id=h::event_id(&root, id)
            role="button"
            tabindex="0"
            data-scope=h::SCOPE
            data-part="event"
            data-tone=move || item.with(|x| x.as_ref().and_then(|(e, _)| e.tone.clone()))
            data-selected=move || selected.get().then_some("")
            data-dragging=move || dragged().then_some("")
            data-short=move || item.with(|x| x.as_ref().is_some_and(|(e, _)| e.end.saturating_sub(e.start) < 45)).then_some("")
            aria-label=move || item.with(|x| x.as_ref().map(|(e, _)| h::event_label(e)))
            aria-pressed=move || selected.get().to_string()
            aria-describedby=h::event_help_id(&root)
            style=move || {
                item.with(|x| {
                    x.as_ref().map(|(e, p)| h::block_style(p.day, p.lane, p.lanes, count.get(), e.start, e.end))
                })
            }
            on:pointerdown=move |ev| ctx.down(&ev, Hit::Event(moving.clone()))
            on:keydown=move |ev| {
                if let Some(e) = h::event_keydown(&ev.key(), ev.shift_key(), &keyed, snap, ctx.dragging()) {
                    ev.prevent_default();
                    ctx.send(e);
                }
            }
        >
            <span data-scope=h::SCOPE data-part="event-title">
                {move || item.with(|x| x.as_ref().map(|(e, _)| e.title.clone()))}
            </span>
            <span data-scope=h::SCOPE data-part="event-time">
                {move || item.with(|x| x.as_ref().map(|(e, _)| h::time_range(e.start, e.end)))}
            </span>
            <span
                data-scope=h::SCOPE
                data-part="handle"
                aria-hidden="true"
                on:pointerdown=move |ev| ctx.down(&ev, Hit::Resize(resizing.clone()))
            ></span>
        </div>
    }
}

fn all_day_view(ctx: Ctx, id: &str, shown: Memo<Vec<WeekEvent>>, rows: Memo<Vec<Placed>>, count: Memo<usize>) -> impl IntoView + use<> {
    let item = find(shown, rows, id.to_owned());
    let root = ctx.root.get_value();
    let selected = {
        let id = id.to_owned();
        Memo::new(move |_| ctx.state.with(|s| s.selected.as_deref() == Some(id.as_str())))
    };
    let (clicked, keyed) = (id.to_owned(), id.to_owned());
    let snap = ctx.input.with_untracked(|i| i.snap);

    view! {
        <div
            id=h::event_id(&root, id)
            role="button"
            tabindex="0"
            data-scope=h::SCOPE
            data-part="allday-event"
            data-tone=move || item.with(|x| x.as_ref().and_then(|(e, _)| e.tone.clone()))
            data-selected=move || selected.get().then_some("")
            aria-label=move || item.with(|x| x.as_ref().map(|(e, _)| h::event_label(e)))
            aria-pressed=move || selected.get().to_string()
            aria-describedby=h::event_help_id(&root)
            style=move || item.with(|x| x.as_ref().map(|(_, p)| h::all_day_style(p.day, p.lane, count.get())))
            on:click=move |ev| {
                ev.stop_propagation();
                ctx.send(WeekSchedulerEvent::Select(clicked.clone()));
            }
            on:keydown=move |ev| {
                if let Some(e) = h::event_keydown(&ev.key(), ev.shift_key(), &keyed, snap, false) {
                    ev.prevent_default();
                    ctx.send(e);
                }
            }
        >
            {move || item.with(|x| x.as_ref().map(|(e, _)| e.title.clone()))}
        </div>
    }
}
