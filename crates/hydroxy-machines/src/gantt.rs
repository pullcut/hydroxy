//! Gantt owns the task list, the time scale and the bar being moved or
//! resized. A move starts with a pointer drag or a keyboard pick-up (Space),
//! follows the pointer (in whole days) or the arrow keys, and ends with a drop
//! or a cancel. Only a drop that changes something reports a [`GanttChange`].
//!
//! The geometry is pure too: [`Timeline`] turns dates into pixels and back
//! (bars, tick marks, the today line, dependency connectors), so adapters only
//! place boxes. Dates are the pure [`Date`](crate::date::Date); the machine
//! never reads a clock, so "today" is an input.

use hydroxy_statechart::{Controllable, Machine, Transition};

use crate::date::Date;

/// Height of a task row in px. Bars and the task list share it.
pub const ROW_HEIGHT: f64 = 40.0;
/// Height of a bar in px, centred in its row.
pub const BAR_HEIGHT: f64 = 24.0;
/// Height of the header (months over days) in px.
pub const HEADER_HEIGHT: f64 = 52.0;

/// One task: a bar from `start` to `end` inclusive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GanttTask {
    /// Unique key.
    pub id: String,
    /// Visible name.
    pub title: String,
    /// First day.
    pub start: Date,
    /// Last day. Never before `start`.
    pub end: Date,
    /// Ids of tasks this one comes after. Drawn as connectors; not enforced.
    pub depends_on: Vec<String>,
}

impl GanttTask {
    /// A task. An `end` before `start` is lifted to `start`.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, start: Date, end: Date) -> Self {
        Self { id: id.into(), title: title.into(), start, end: end.max(start), depends_on: Vec::new() }
    }

    /// Set the tasks this one comes after.
    #[must_use]
    pub fn depends_on<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.depends_on = ids.into_iter().map(Into::into).collect();
        self
    }

    /// How many days the bar covers.
    #[must_use]
    pub fn days(&self) -> i64 {
        self.end.to_days() - self.start.to_days() + 1
    }
}

/// How wide a day is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GanttScale {
    /// One column per day.
    #[default]
    Day,
    /// One column per week.
    Week,
}

impl GanttScale {
    /// The `data-scale` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            GanttScale::Day => "day",
            GanttScale::Week => "week",
        }
    }

    /// Width of one day in px.
    #[must_use]
    pub const fn px_per_day(self) -> f64 {
        match self {
            GanttScale::Day => 32.0,
            GanttScale::Week => 12.0,
        }
    }
}

/// `"Oct 26"`.
#[must_use]
pub fn short_date(d: Date) -> String {
    format!("{} {}", crate::date::MONTH_SHORT[usize::from(d.month() - 1)], d.day())
}

/// `"Oct 26 to Oct 30"`, or `"Oct 26"` for a single day.
#[must_use]
pub fn range_text(start: Date, end: Date) -> String {
    if start == end { short_date(start) } else { format!("{} to {}", short_date(start), short_date(end)) }
}

mod timeline;
pub use timeline::{Connector, MonthSpan, Rect, Tick, Timeline, connector_path};

/// A change to one task's dates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GanttChange {
    /// The task.
    pub id: String,
    /// Its new first day.
    pub start: Date,
    /// Its new last day.
    pub end: Date,
}

/// Apply `change` to `tasks`. Unknown ids leave them unchanged. Controlled
/// owners call this from `on_change`.
pub fn apply_change(tasks: &mut [GanttTask], change: &GanttChange) {
    if let Some(t) = tasks.iter_mut().find(|t| t.id == change.id) {
        t.start = change.start;
        t.end = change.end.max(change.start);
    }
}

/// Props for [`GanttMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GanttInput {
    /// Controlled tasks. `Some` at start fixes controlled mode.
    pub tasks: Option<Vec<GanttTask>>,
    /// Uncontrolled initial tasks.
    pub default_tasks: Vec<GanttTask>,
    /// Controlled scale. `Some` at start fixes controlled mode.
    pub scale: Option<GanttScale>,
    /// Uncontrolled initial scale.
    pub default_scale: GanttScale,
    /// Block every move.
    pub disabled: bool,
}

/// What part of a bar is being moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragMode {
    /// The whole bar.
    Move,
    /// The left edge.
    ResizeStart,
    /// The right edge.
    ResizeEnd,
}

impl DragMode {
    /// The `data-mode` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DragMode::Move => "move",
            DragMode::ResizeStart => "resize-start",
            DragMode::ResizeEnd => "resize-end",
        }
    }
}

/// The bar in flight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drag {
    /// The task.
    pub id: String,
    /// What a pointer drag moves. Keyboard nudges name their own mode.
    pub mode: DragMode,
    /// Dates when the drag began.
    pub origin: (Date, Date),
    /// Where it would land now.
    pub draft: (Date, Date),
    /// Picked up with Space rather than a pointer drag.
    pub keyboard: bool,
}

/// What [`GanttMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GanttState {
    /// The committed tasks.
    pub tasks: Controllable<Vec<GanttTask>>,
    /// The scale.
    pub scale: Controllable<GanttScale>,
    /// The bar being moved, if any.
    pub drag: Option<Drag>,
}

impl GanttState {
    /// The tasks to draw: the one in flight shows where it would land.
    #[must_use]
    pub fn preview(&self) -> Vec<GanttTask> {
        let mut tasks = self.tasks.value.clone();
        if let Some(d) = &self.drag {
            apply_change(&mut tasks, &GanttChange { id: d.id.clone(), start: d.draft.0, end: d.draft.1 });
        }
        tasks
    }
}

/// Inputs to [`GanttMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GanttEvent {
    /// Space on a bar: pick it up.
    Grab(String),
    /// A pointer drag started on this bar.
    DragStart {
        /// The task.
        id: String,
        /// What it moves.
        mode: DragMode,
    },
    /// The pointer is `delta` whole days from where the drag began.
    DragBy(i64),
    /// Arrow key while a bar is picked up: shift this part of it by `days`.
    Nudge {
        /// What to shift.
        mode: DragMode,
        /// Days, forward or back.
        days: i64,
    },
    /// Space or Enter while carrying, or a pointer release.
    Drop,
    /// Esc, or a pointer drag that was interrupted.
    Cancel,
    /// Change the scale.
    SetScale(GanttScale),
    /// Arrow up or down on a bar: focus the previous or next bar.
    FocusStep {
        /// The bar the key was pressed on.
        from: String,
        /// `-1` for up, `1` for down.
        by: isize,
    },
}

/// What a screen reader should hear. Format with your own words or the headless defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Announcement {
    /// Picked up a task.
    Grabbed {
        /// Task title.
        title: String,
        /// Its dates.
        range: (Date, Date),
    },
    /// Carried to new dates.
    Moved {
        /// Task title.
        title: String,
        /// Its dates now.
        range: (Date, Date),
    },
    /// Dropped on these dates.
    Dropped {
        /// Task title.
        title: String,
        /// Its dates.
        range: (Date, Date),
    },
    /// Put back where it started.
    Cancelled {
        /// Task title.
        title: String,
    },
}

/// Side effects from [`GanttMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GanttCommand {
    /// A task's dates changed (or should, when controlled).
    Change(GanttChange),
    /// The scale changed (or should, when controlled).
    ScaleChange(GanttScale),
    /// Move DOM focus to this task's bar.
    FocusBar(String),
    /// Say this in the live region.
    Announce(Announcement),
}

/// Tasks, scale and drag state for a Gantt chart.
#[derive(Debug, Clone, Copy)]
pub struct GanttMachine;

type Step = Transition<GanttState, GanttCommand>;

fn find<'a>(tasks: &'a [GanttTask], id: &str) -> Option<&'a GanttTask> {
    tasks.iter().find(|t| t.id == id)
}

/// Shift the edges of `range` that `mode` names by `days`, keeping at least one day.
fn shifted(range: (Date, Date), mode: DragMode, days: i64) -> (Date, Date) {
    let (start, end) = range;
    match mode {
        DragMode::Move => (start.add_days(days), end.add_days(days)),
        DragMode::ResizeStart => (start.add_days(days).min(end), end),
        DragMode::ResizeEnd => (start, end.add_days(days).max(start)),
    }
}

fn begin(state: &GanttState, id: &str, mode: DragMode, keyboard: bool) -> Option<Drag> {
    let task = find(&state.tasks.value, id)?;
    let origin = (task.start, task.end);
    Some(Drag { id: id.to_string(), mode, origin, draft: origin, keyboard })
}

impl Machine for GanttMachine {
    type Input = GanttInput;
    type State = GanttState;
    type Event = GanttEvent;
    type Command = GanttCommand;

    fn initial(input: &GanttInput) -> GanttState {
        GanttState {
            tasks: Controllable::new(
                input.tasks.clone().unwrap_or_else(|| input.default_tasks.clone()),
                input.tasks.is_some(),
            ),
            scale: Controllable::new(input.scale.unwrap_or(input.default_scale), input.scale.is_some()),
            drag: None,
        }
    }

    fn transition(state: &GanttState, event: GanttEvent, input: &GanttInput) -> Step {
        let stay = || Transition::stay();
        if let GanttEvent::SetScale(scale) = event {
            if scale == state.scale.value {
                return stay();
            }
            let mut next = state.clone();
            next.scale.set(scale);
            return Transition::update(state, next, vec![GanttCommand::ScaleChange(scale)]);
        }
        if let GanttEvent::FocusStep { from, by } = event {
            let Some(at) = state.tasks.value.iter().position(|t| t.id == from) else { return stay() };
            let to = at.saturating_add_signed(by).min(state.tasks.value.len() - 1);
            if to == at {
                return stay();
            }
            return Transition::emit(vec![GanttCommand::FocusBar(state.tasks.value[to].id.clone())]);
        }
        if input.disabled {
            return stay();
        }
        let mut next = state.clone();
        match (event, &state.drag) {
            (GanttEvent::Grab(id), None) => {
                let Some(drag) = begin(state, &id, DragMode::Move, true) else { return stay() };
                let announce = Announcement::Grabbed {
                    title: find(&state.tasks.value, &id).map(|t| t.title.clone()).unwrap_or_default(),
                    range: drag.origin,
                };
                next.drag = Some(drag);
                Transition::update(state, next, vec![GanttCommand::Announce(announce)])
            }
            (GanttEvent::DragStart { id, mode }, _) => {
                let Some(drag) = begin(state, &id, mode, false) else { return stay() };
                next.drag = Some(drag);
                Transition::update(state, next, Vec::new())
            }
            (GanttEvent::DragBy(delta), Some(d)) if !d.keyboard => {
                let draft = shifted(d.origin, d.mode, delta);
                if draft == d.draft {
                    return stay();
                }
                next.drag = Some(Drag { draft, ..d.clone() });
                Transition::update(state, next, Vec::new())
            }
            (GanttEvent::Nudge { mode, days }, Some(d)) if d.keyboard => {
                let draft = shifted(d.draft, mode, days);
                if draft == d.draft {
                    return stay();
                }
                let title = find(&state.tasks.value, &d.id).map(|t| t.title.clone()).unwrap_or_default();
                next.drag = Some(Drag { draft, ..d.clone() });
                Transition::update(
                    state,
                    next,
                    vec![GanttCommand::Announce(Announcement::Moved { title, range: draft })],
                )
            }
            (GanttEvent::Drop, Some(d)) => {
                next.drag = None;
                let mut commands = Vec::new();
                if d.draft != d.origin {
                    let change = GanttChange { id: d.id.clone(), start: d.draft.0, end: d.draft.1 };
                    let mut tasks = state.tasks.value.clone();
                    apply_change(&mut tasks, &change);
                    next.tasks.set(tasks);
                    commands.push(GanttCommand::Change(change));
                }
                if d.keyboard {
                    let title = find(&state.tasks.value, &d.id).map(|t| t.title.clone()).unwrap_or_default();
                    commands.push(GanttCommand::Announce(Announcement::Dropped { title, range: d.draft }));
                }
                Transition::update(state, next, commands)
            }
            (GanttEvent::Cancel, Some(d)) => {
                next.drag = None;
                let commands = if d.keyboard {
                    let title = find(&state.tasks.value, &d.id).map(|t| t.title.clone()).unwrap_or_default();
                    vec![GanttCommand::Announce(Announcement::Cancelled { title })]
                } else {
                    Vec::new()
                };
                Transition::update(state, next, commands)
            }
            _ => stay(),
        }
    }

    fn sync(state: &GanttState, input: &GanttInput) -> Option<GanttState> {
        let mut next = state.clone();
        next.tasks.sync(input.tasks.clone());
        next.scale.sync(input.scale);
        if input.disabled || next.drag.as_ref().is_some_and(|d| find(&next.tasks.value, &d.id).is_none()) {
            next.drag = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests;
