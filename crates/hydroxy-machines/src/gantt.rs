//! Gantt owns the task list, the time scale and the bar being moved or
//! resized. A move starts with a pointer drag or a keyboard pick-up (Space),
//! follows the pointer (in whole days) or the arrow keys, and ends with a drop
//! or a cancel. Only a drop that changes something reports a [`GanttChange`].
//!
//! The geometry is pure too: [`Timeline`] turns dates into pixels and back
//! (bars, tick marks, the today line, dependency connectors), so adapters only
//! place boxes. Dates are the pure [`Date`](crate::date::Date); the machine
//! never reads a clock, so "today" is an input.

use hydroxy_statechart::{Machine, Transition};

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

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// `"Oct 26"`.
#[must_use]
pub fn short_date(d: Date) -> String {
    format!("{} {}", MONTHS[usize::from(d.month() - 1)], d.day())
}

/// `"Oct 26 to Oct 30"`, or `"Oct 26"` for a single day.
#[must_use]
pub fn range_text(start: Date, end: Date) -> String {
    if start == end { short_date(start) } else { format!("{} to {}", short_date(start), short_date(end)) }
}

/// A column mark in the header.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Left edge in px.
    pub x: f64,
    /// Width in px.
    pub width: f64,
    /// Day number (day scale) or the week's first date (week scale).
    pub label: String,
    /// The first day or week of a month. Drawn with a heavier line.
    pub major: bool,
    /// A Saturday or Sunday, in day scale.
    pub weekend: bool,
}

/// A month in the header's upper row.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthSpan {
    /// Left edge in px.
    pub x: f64,
    /// Width in px.
    pub width: f64,
    /// `"October 2026"`-style text, short enough to fit: `"Oct 2026"`.
    pub label: String,
}

/// A box in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left.
    pub x: f64,
    /// Top.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// A line from one bar to the bar that depends on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Connector {
    /// The task it leaves.
    pub from: String,
    /// The task it enters.
    pub to: String,
    /// SVG path data for the line.
    pub line: String,
    /// SVG path data for the arrowhead at the end.
    pub arrow: String,
}

/// Dates mapped to px along a horizontal axis of whole days.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timeline {
    /// The first day on the axis.
    pub start: Date,
    /// How many days it spans.
    pub days: i64,
    /// Width of one day in px.
    pub px_per_day: f64,
    /// The scale, which decides the tick marks.
    pub scale: GanttScale,
}

/// Empty days kept before the first and after the last task, so bars can move.
const PAD_BEFORE: i64 = 7;
const PAD_AFTER: i64 = 14;
const MIN_DAYS: i64 = 28;

impl Timeline {
    /// An axis that fits `tasks` and `today` with room to move them. In week
    /// scale it starts on a Monday and ends on a Sunday.
    #[must_use]
    pub fn fit(scale: GanttScale, tasks: &[GanttTask], today: Option<Date>) -> Self {
        let mut first = tasks.iter().map(|t| t.start).chain(today).min().unwrap_or_default();
        let mut last = tasks.iter().map(|t| t.end).chain(today).max().unwrap_or_default();
        first = first.add_days(-PAD_BEFORE);
        last = last.add_days(PAD_AFTER);
        if scale == GanttScale::Week {
            first = first.add_days(-first.days_into_week(1));
            last = last.add_days(6 - last.days_into_week(1));
        }
        let days = (last.to_days() - first.to_days() + 1).max(MIN_DAYS);
        Self { start: first, days, px_per_day: scale.px_per_day(), scale }
    }

    /// The last day on the axis.
    #[must_use]
    pub fn end(&self) -> Date {
        self.start.add_days(self.days - 1)
    }

    /// Total width in px.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn width(&self) -> f64 {
        self.days as f64 * self.px_per_day
    }

    /// Left edge of `date`'s column.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn x(&self, date: Date) -> f64 {
        (date.to_days() - self.start.to_days()) as f64 * self.px_per_day
    }

    /// The box of a bar from `start` to `end` inclusive, in row `row`.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days and rows")]
    pub fn bar(&self, start: Date, end: Date, row: usize) -> Rect {
        let days = (end.to_days() - start.to_days() + 1).max(1);
        Rect {
            x: self.x(start),
            y: row as f64 * ROW_HEIGHT + (ROW_HEIGHT - BAR_HEIGHT) / 2.0,
            width: days as f64 * self.px_per_day,
            height: BAR_HEIGHT,
        }
    }

    /// The day under `x` px, clamped to the axis.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "a few hundred days")]
    pub fn date_at(&self, x: f64) -> Date {
        let day = (x / self.px_per_day).floor() as i64;
        self.start.add_days(day.clamp(0, self.days - 1))
    }

    /// Whole days for a pointer move of `dx` px. Snaps to the nearest day.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "a few hundred days")]
    pub fn days_for_px(&self, dx: f64) -> i64 {
        if dx.is_finite() { (dx / self.px_per_day).round() as i64 } else { 0 }
    }

    /// Limit a move of `delta` days so the bar `start..=end` stays on the axis.
    /// A resize passes the edge it moves as both ends.
    #[must_use]
    pub fn clamp_delta(&self, start: Date, end: Date, delta: i64) -> i64 {
        let lowest = self.start.to_days() - start.to_days();
        let highest = self.end().to_days() - end.to_days();
        if lowest > highest { 0 } else { delta.clamp(lowest, highest) }
    }

    /// The header's lower row: a column per day, or per week.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn ticks(&self) -> Vec<Tick> {
        let step_days: i64 = if self.scale == GanttScale::Week { 7 } else { 1 };
        let step = usize::try_from(step_days).unwrap_or(1);
        (0..self.days)
            .step_by(step)
            .map(|offset| {
                let date = self.start.add_days(offset);
                let width = step_days as f64 * self.px_per_day;
                let first_of_month = (0..step_days).any(|d| date.add_days(d).day() == 1);
                Tick {
                    x: offset as f64 * self.px_per_day,
                    width,
                    label: if self.scale == GanttScale::Week { short_date(date) } else { date.day().to_string() },
                    major: first_of_month,
                    weekend: self.scale == GanttScale::Day && matches!(date.weekday(), 0 | 6),
                }
            })
            .collect()
    }

    /// The header's upper row: one span per month, clipped to the axis.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn months(&self) -> Vec<MonthSpan> {
        let mut spans: Vec<MonthSpan> = Vec::new();
        let mut month_start = 0_i64;
        for offset in 0..self.days {
            let date = self.start.add_days(offset);
            let next = self.start.add_days(offset + 1);
            if offset == self.days - 1 || next.month() != date.month() {
                spans.push(MonthSpan {
                    x: month_start as f64 * self.px_per_day,
                    width: (offset - month_start + 1) as f64 * self.px_per_day,
                    label: format!("{} {}", MONTHS[usize::from(date.month() - 1)], date.year()),
                });
                month_start = offset + 1;
            }
        }
        spans
    }

    /// Where the today line goes: the middle of the day's column. `None` when off the axis.
    #[must_use]
    pub fn today_x(&self, today: Date) -> Option<f64> {
        (today >= self.start && today <= self.end()).then(|| self.x(today) + self.px_per_day / 2.0)
    }

    /// Connectors for every dependency whose two tasks are both listed.
    /// `tasks` is in row order; bars use the tasks' own dates.
    #[must_use]
    pub fn connectors(&self, tasks: &[GanttTask]) -> Vec<Connector> {
        let bars: Vec<Rect> = tasks.iter().enumerate().map(|(row, t)| self.bar(t.start, t.end, row)).collect();
        let mut out = Vec::new();
        for (to_row, task) in tasks.iter().enumerate() {
            for dep in &task.depends_on {
                let Some(from_row) = tasks.iter().position(|t| t.id == *dep) else { continue };
                let (line, arrow) = connector_path(bars[from_row], bars[to_row]);
                out.push(Connector { from: dep.clone(), to: task.id.clone(), line, arrow });
            }
        }
        out
    }
}

/// How far a connector leaves a bar before turning, in px.
const STUB: f64 = 10.0;
const ARROW: f64 = 5.0;

/// The path from the right edge of `from` to the left edge of `to`, as an
/// elbow, and an arrowhead pointing right at its end. When `to` starts too
/// soon after `from` ends, the line runs through the gap between the rows.
#[must_use]
pub fn connector_path(from: Rect, to: Rect) -> (String, String) {
    let (x1, y1) = (from.x + from.width, from.y + from.height / 2.0);
    let (x2, y2) = (to.x, to.y + to.height / 2.0);
    let line = if x2 >= x1 + 2.0 * STUB || (y1 - y2).abs() < f64::EPSILON {
        let mid = if (y1 - y2).abs() < f64::EPSILON { x1 } else { x1 + STUB };
        format!("M{x1} {y1} H{mid} V{y2} H{x2}")
    } else {
        // Run along the row boundary between the two bars.
        let gap = if y2 > y1 { (from.y + from.height + to.y) / 2.0 } else { (to.y + to.height + from.y) / 2.0 };
        format!("M{x1} {y1} H{} V{gap} H{} V{y2} H{x2}", x1 + STUB, x2 - STUB)
    };
    let arrow = format!("M{x2} {y2} l{} {} v{} z", -ARROW, -ARROW * 0.6, ARROW * 1.2);
    (line, arrow)
}

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
    pub tasks: Vec<GanttTask>,
    /// The scale.
    pub scale: GanttScale,
    /// The bar being moved, if any.
    pub drag: Option<Drag>,
    /// Fixed at start: the owner holds the tasks.
    pub controlled: bool,
    /// Fixed at start: the owner holds the scale.
    pub controlled_scale: bool,
}

impl GanttState {
    /// The tasks to draw: the one in flight shows where it would land.
    #[must_use]
    pub fn preview(&self) -> Vec<GanttTask> {
        let mut tasks = self.tasks.clone();
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
    let task = find(&state.tasks, id)?;
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
            tasks: input.tasks.clone().unwrap_or_else(|| input.default_tasks.clone()),
            scale: input.scale.unwrap_or(input.default_scale),
            drag: None,
            controlled: input.tasks.is_some(),
            controlled_scale: input.scale.is_some(),
        }
    }

    fn transition(state: &GanttState, event: GanttEvent, input: &GanttInput) -> Step {
        let stay = || Transition::stay();
        if let GanttEvent::SetScale(scale) = event {
            if scale == state.scale {
                return stay();
            }
            let mut next = state.clone();
            if !state.controlled_scale {
                next.scale = scale;
            }
            return Transition::with(next, vec![GanttCommand::ScaleChange(scale)]);
        }
        if let GanttEvent::FocusStep { from, by } = event {
            let Some(at) = state.tasks.iter().position(|t| t.id == from) else { return stay() };
            let to = at.saturating_add_signed(by).min(state.tasks.len() - 1);
            if to == at {
                return stay();
            }
            return Transition::emit(vec![GanttCommand::FocusBar(state.tasks[to].id.clone())]);
        }
        if input.disabled {
            return stay();
        }
        let mut next = state.clone();
        match (event, &state.drag) {
            (GanttEvent::Grab(id), None) => {
                let Some(drag) = begin(state, &id, DragMode::Move, true) else { return stay() };
                let announce = Announcement::Grabbed { title: find(&state.tasks, &id).map(|t| t.title.clone()).unwrap_or_default(), range: drag.origin };
                next.drag = Some(drag);
                Transition::with(next, vec![GanttCommand::Announce(announce)])
            }
            (GanttEvent::DragStart { id, mode }, _) => {
                let Some(drag) = begin(state, &id, mode, false) else { return stay() };
                next.drag = Some(drag);
                Transition::to(next)
            }
            (GanttEvent::DragBy(delta), Some(d)) if !d.keyboard => {
                let draft = shifted(d.origin, d.mode, delta);
                if draft == d.draft {
                    return stay();
                }
                next.drag = Some(Drag { draft, ..d.clone() });
                Transition::to(next)
            }
            (GanttEvent::Nudge { mode, days }, Some(d)) if d.keyboard => {
                let draft = shifted(d.draft, mode, days);
                if draft == d.draft {
                    return stay();
                }
                let title = find(&state.tasks, &d.id).map(|t| t.title.clone()).unwrap_or_default();
                next.drag = Some(Drag { draft, ..d.clone() });
                Transition::with(next, vec![GanttCommand::Announce(Announcement::Moved { title, range: draft })])
            }
            (GanttEvent::Drop, Some(d)) => {
                next.drag = None;
                let mut commands = Vec::new();
                if d.draft != d.origin {
                    let change = GanttChange { id: d.id.clone(), start: d.draft.0, end: d.draft.1 };
                    if !state.controlled {
                        apply_change(&mut next.tasks, &change);
                    }
                    commands.push(GanttCommand::Change(change));
                }
                if d.keyboard {
                    let title = find(&state.tasks, &d.id).map(|t| t.title.clone()).unwrap_or_default();
                    commands.push(GanttCommand::Announce(Announcement::Dropped { title, range: d.draft }));
                }
                Transition::with(next, commands)
            }
            (GanttEvent::Cancel, Some(d)) => {
                next.drag = None;
                let commands = if d.keyboard {
                    let title = find(&state.tasks, &d.id).map(|t| t.title.clone()).unwrap_or_default();
                    vec![GanttCommand::Announce(Announcement::Cancelled { title })]
                } else {
                    Vec::new()
                };
                Transition::with(next, commands)
            }
            _ => stay(),
        }
    }

    fn sync(state: &GanttState, input: &GanttInput) -> Option<GanttState> {
        let mut next = state.clone();
        if state.controlled {
            next.tasks = input.tasks.clone().unwrap_or_default();
        }
        if state.controlled_scale {
            next.scale = input.scale.unwrap_or_default();
        }
        if input.disabled || next.drag.as_ref().is_some_and(|d| find(&next.tasks, &d.id).is_none()) {
            next.drag = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn tasks() -> Vec<GanttTask> {
        vec![
            GanttTask::new("research", "Research", d("2026-10-26"), d("2026-10-28")),
            GanttTask::new("design", "Design", d("2026-10-29"), d("2026-11-03")).depends_on(["research"]),
            GanttTask::new("build", "Build", d("2026-11-04"), d("2026-11-10")).depends_on(["design", "research"]),
        ]
    }

    fn input() -> GanttInput {
        GanttInput { default_tasks: tasks(), ..GanttInput::default() }
    }

    fn run(s: &GanttState, e: GanttEvent, i: &GanttInput) -> hydroxy_statechart::Step<GanttState, GanttCommand> {
        GanttMachine::transition(s, e, i).resolve(s)
    }

    fn range(s: &GanttState, id: &str) -> (Date, Date) {
        let t = find(&s.tasks, id).unwrap();
        (t.start, t.end)
    }

    fn start(id: &str, mode: DragMode) -> GanttEvent {
        GanttEvent::DragStart { id: id.into(), mode }
    }

    #[test]
    fn pointer_move_snaps_to_days_and_commits_on_drop() {
        let i = input();
        let s = run(&GanttMachine::initial(&i), start("design", DragMode::Move), &i).state;
        let s = run(&s, GanttEvent::DragBy(2), &i).state;
        let preview = s.preview();
        assert_eq!((preview[1].start, preview[1].end), (d("2026-10-31"), d("2026-11-05")));
        // The committed tasks haven't moved yet.
        assert_eq!(range(&s, "design"), (d("2026-10-29"), d("2026-11-03")));
        let step = run(&s, GanttEvent::Drop, &i);
        assert_eq!(step.commands, [GanttCommand::Change(GanttChange { id: "design".into(), start: d("2026-10-31"), end: d("2026-11-05") })]);
        assert_eq!(range(&step.state, "design"), (d("2026-10-31"), d("2026-11-05")));
        assert!(step.state.drag.is_none());
    }

    #[test]
    fn resizing_keeps_at_least_one_day() {
        let i = input();
        let s = run(&GanttMachine::initial(&i), start("research", DragMode::ResizeEnd), &i).state;
        let s = run(&s, GanttEvent::DragBy(-10), &i).state;
        assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-26"), d("2026-10-26"))));
        let s = run(&s, GanttEvent::DragBy(3), &i).state;
        assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-26"), d("2026-10-31"))));
        let s = run(&GanttMachine::initial(&i), start("research", DragMode::ResizeStart), &i).state;
        let s = run(&s, GanttEvent::DragBy(10), &i).state;
        assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-28"), d("2026-10-28"))));
    }

    #[test]
    fn dropping_in_place_or_cancelling_reports_nothing() {
        let i = input();
        let s = run(&GanttMachine::initial(&i), start("build", DragMode::Move), &i).state;
        assert!(run(&s, GanttEvent::Drop, &i).commands.is_empty());
        let s = run(&s, GanttEvent::DragBy(4), &i).state;
        let step = run(&s, GanttEvent::Cancel, &i);
        assert!(step.commands.is_empty());
        assert_eq!(step.state.preview(), tasks());
    }

    #[test]
    fn keyboard_pick_up_nudge_resize_drop() {
        let i = input();
        let s = GanttMachine::initial(&i);
        // Nudges do nothing until a bar is picked up.
        assert!(run(&s, GanttEvent::Nudge { mode: DragMode::Move, days: 1 }, &i).commands.is_empty());
        let step = run(&s, GanttEvent::Grab("research".into()), &i);
        assert!(matches!(&step.commands[..], [GanttCommand::Announce(Announcement::Grabbed { .. })]));
        let s = run(&step.state, GanttEvent::Nudge { mode: DragMode::Move, days: 1 }, &i).state;
        let s = run(&s, GanttEvent::Nudge { mode: DragMode::ResizeEnd, days: 2 }, &i).state;
        let s = run(&s, GanttEvent::Nudge { mode: DragMode::ResizeStart, days: -1 }, &i).state;
        assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-26"), d("2026-10-31"))));
        // Pointer events are ignored while carrying.
        assert!(run(&s, GanttEvent::DragBy(9), &i).commands.is_empty());
        let step = run(&s, GanttEvent::Drop, &i);
        assert_eq!(step.commands[0], GanttCommand::Change(GanttChange { id: "research".into(), start: d("2026-10-26"), end: d("2026-10-31") }));
        assert!(matches!(step.commands[1], GanttCommand::Announce(Announcement::Dropped { .. })));
    }

    #[test]
    fn focus_step_moves_between_bars_and_stops_at_the_ends() {
        let i = input();
        let s = GanttMachine::initial(&i);
        let step = run(&s, GanttEvent::FocusStep { from: "research".into(), by: 1 }, &i);
        assert_eq!(step.commands, [GanttCommand::FocusBar("design".into())]);
        assert!(run(&s, GanttEvent::FocusStep { from: "research".into(), by: -1 }, &i).commands.is_empty());
        assert!(run(&s, GanttEvent::FocusStep { from: "build".into(), by: 1 }, &i).commands.is_empty());
    }

    #[test]
    fn scale_can_be_controlled() {
        let i = input();
        let step = run(&GanttMachine::initial(&i), GanttEvent::SetScale(GanttScale::Week), &i);
        assert_eq!(step.state.scale, GanttScale::Week);
        assert_eq!(step.commands, [GanttCommand::ScaleChange(GanttScale::Week)]);
        assert!(run(&step.state, GanttEvent::SetScale(GanttScale::Week), &i).commands.is_empty());
        let mut c = GanttInput { scale: Some(GanttScale::Day), ..input() };
        let step = run(&GanttMachine::initial(&c), GanttEvent::SetScale(GanttScale::Week), &c);
        assert_eq!(step.state.scale, GanttScale::Day);
        c.scale = Some(GanttScale::Week);
        assert_eq!(hydroxy_statechart::sync_or_keep::<GanttMachine>(&step.state, &c).scale, GanttScale::Week);
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut i = GanttInput { tasks: Some(tasks()), ..GanttInput::default() };
        let s = run(&GanttMachine::initial(&i), start("build", DragMode::Move), &i).state;
        let s = run(&s, GanttEvent::DragBy(-1), &i).state;
        let step = run(&s, GanttEvent::Drop, &i);
        let GanttCommand::Change(change) = step.commands[0].clone() else { panic!("expected a change") };
        assert_eq!(range(&step.state, "build"), (d("2026-11-04"), d("2026-11-10")));
        let mut next = tasks();
        apply_change(&mut next, &change);
        i.tasks = Some(next);
        assert_eq!(range(&hydroxy_statechart::sync_or_keep::<GanttMachine>(&step.state, &i), "build"), (d("2026-11-03"), d("2026-11-09")));
    }

    #[test]
    fn disabled_blocks_moves_but_not_the_scale() {
        let i = GanttInput { disabled: true, ..input() };
        let s = GanttMachine::initial(&i);
        assert!(run(&s, GanttEvent::Grab("build".into()), &i).state.drag.is_none());
        assert!(run(&s, start("build", DragMode::Move), &i).state.drag.is_none());
        assert_eq!(run(&s, GanttEvent::SetScale(GanttScale::Week), &i).commands.len(), 1);
    }

    #[test]
    fn timeline_fits_tasks_and_maps_dates() {
        let t = Timeline::fit(GanttScale::Day, &tasks(), Some(d("2026-10-30")));
        assert_eq!(t.start, d("2026-10-19"));
        assert_eq!(t.end(), d("2026-11-24"));
        assert_eq!(t.days, 37);
        assert!((t.width() - 37.0 * 32.0).abs() < 1e-9);
        assert!((t.x(d("2026-10-26")) - 7.0 * 32.0).abs() < 1e-9);
        let bar = t.bar(d("2026-10-26"), d("2026-10-28"), 2);
        assert!((bar.x - 224.0).abs() < 1e-9 && (bar.width - 96.0).abs() < 1e-9);
        assert!((bar.y - (2.0 * ROW_HEIGHT + 8.0)).abs() < 1e-9);
        assert_eq!(t.date_at(224.0), d("2026-10-26"));
        assert_eq!(t.date_at(255.9), d("2026-10-26"));
        assert_eq!(t.date_at(-50.0), t.start);
        assert_eq!(t.date_at(1e9), t.end());
        assert_eq!(t.today_x(d("2026-10-30")), Some(11.0 * 32.0 + 16.0));
        assert_eq!(t.today_x(d("2027-01-01")), None);
    }

    #[test]
    fn pixels_snap_to_days_and_deltas_stay_on_the_axis() {
        let t = Timeline::fit(GanttScale::Day, &tasks(), None);
        assert_eq!(t.days_for_px(15.0), 0);
        assert_eq!(t.days_for_px(16.0), 1);
        assert_eq!(t.days_for_px(-50.0), -2);
        assert_eq!(t.days_for_px(f64::NAN), 0);
        let (s, e) = (d("2026-10-26"), d("2026-10-28"));
        assert_eq!(t.clamp_delta(s, e, -100), -7);
        assert_eq!(t.clamp_delta(s, e, 3), 3);
        assert_eq!(t.clamp_delta(s, e, 100), t.end().to_days() - e.to_days());
    }

    #[test]
    fn week_scale_aligns_to_whole_weeks() {
        let t = Timeline::fit(GanttScale::Week, &tasks(), None);
        assert_eq!(t.start.weekday(), 1);
        assert_eq!(t.end().weekday(), 0);
        assert_eq!(t.days % 7, 0);
        let ticks = t.ticks();
        assert_eq!(i64::try_from(ticks.len()).unwrap(), t.days / 7);
        assert!((ticks[0].width - 84.0).abs() < 1e-9);
        assert_eq!(ticks[0].label, short_date(t.start));
    }

    #[test]
    fn day_ticks_mark_weekends_and_month_starts() {
        let t = Timeline::fit(GanttScale::Day, &tasks(), None);
        let ticks = t.ticks();
        assert_eq!(i64::try_from(ticks.len()).unwrap(), t.days);
        let nov1 = ticks.iter().position(|k| k.major).unwrap();
        assert_eq!(t.start.add_days(i64::try_from(nov1).unwrap()), d("2026-11-01"));
        assert_eq!(ticks[nov1].label, "1");
        let sat = ticks.iter().position(|k| k.weekend).unwrap();
        assert_eq!(t.start.add_days(i64::try_from(sat).unwrap()).weekday(), 6);
    }

    #[test]
    fn months_cover_the_axis() {
        let t = Timeline::fit(GanttScale::Day, &tasks(), None);
        let spans = t.months();
        assert_eq!(spans.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(), ["Oct 2026", "Nov 2026"]);
        let total: f64 = spans.iter().map(|m| m.width).sum();
        assert!((total - t.width()).abs() < 1e-9);
        assert!((spans[1].x - spans[0].width).abs() < 1e-9);
    }

    #[test]
    fn connectors_link_each_dependency() {
        let t = Timeline::fit(GanttScale::Day, &tasks(), None);
        let c = t.connectors(&tasks());
        assert_eq!(c.iter().map(|c| (c.from.as_str(), c.to.as_str())).collect::<Vec<_>>(), [("research", "design"), ("design", "build"), ("research", "build")]);
        // research ends 28 Oct, design starts 29 Oct: one day is too tight for a straight elbow.
        assert!(c[0].line.contains('V') && c[0].line.matches('H').count() == 3);
        // Unknown dependencies are skipped.
        let mut list = tasks();
        list[0] = list[0].clone().depends_on(["ghost"]);
        assert_eq!(t.connectors(&list).len(), 3);
    }

    #[test]
    fn connector_shapes() {
        let from = Rect { x: 0.0, y: 8.0, width: 100.0, height: 24.0 };
        let wide = Rect { x: 200.0, y: 48.0, width: 50.0, height: 24.0 };
        let (line, arrow) = connector_path(from, wide);
        assert_eq!(line, "M100 20 H110 V60 H200");
        assert!(arrow.starts_with("M200 60 l-5"));
        let tight = Rect { x: 105.0, y: 48.0, width: 50.0, height: 24.0 };
        assert_eq!(connector_path(from, tight).0, "M100 20 H110 V40 H95 V60 H105");
        let same_row = Rect { x: 130.0, y: 8.0, width: 50.0, height: 24.0 };
        assert_eq!(connector_path(from, same_row).0, "M100 20 H100 V20 H130");
        // Going up runs through the gap above.
        assert_eq!(connector_path(wide, Rect { x: 260.0, y: 8.0, width: 10.0, height: 24.0 }).0, "M250 60 H260 V40 H250 V20 H260");
    }

    #[test]
    fn text_helpers() {
        assert_eq!(short_date(d("2026-10-05")), "Oct 5");
        assert_eq!(range_text(d("2026-10-26"), d("2026-10-28")), "Oct 26 to Oct 28");
        assert_eq!(range_text(d("2026-10-26"), d("2026-10-26")), "Oct 26");
        assert_eq!(GanttTask::new("x", "X", d("2026-10-05"), d("2026-10-01")).days(), 1);
    }
}
