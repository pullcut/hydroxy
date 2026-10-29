//! Week scheduler: events on a time grid in day, work-week or week view.
//!
//! The owner holds the events; the machine only remembers the view, the date
//! being shown, the selection, the keyboard slot cursor and the drag in flight.
//! A drag on empty time drafts a new event (snapped to the input's `snap`
//! minutes), a drag on an event moves it (across days too), a drag on its bottom
//! edge resizes it. Nothing reaches the owner until the pointer is released:
//! then it hears [`WeekSchedulerCommand::Create`] or
//! [`WeekSchedulerCommand::Change`]. A click without a drag selects.
//!
//! Overlapping events share their day column: [`pack`] gives each one a lane.
//! The machine never reads a clock, so "today" is an input.

use hydroxy_statechart::{Machine, Transition};

use crate::date::Date;

/// Minutes in a day. An event may end at this minute (midnight).
pub const MINUTES_PER_DAY: u16 = 1440;
/// Height of one hour on the grid, in px.
pub const HOUR_HEIGHT: f64 = 48.0;

/// One event. Timed events sit on one day, from `start` to `end` minutes after
/// midnight. All-day events sit in the row above the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekEvent {
    /// Unique key.
    pub id: String,
    /// Visible name.
    pub title: String,
    /// The day it is on.
    pub date: Date,
    /// Minutes after midnight it starts. 0 for all-day events.
    pub start: u16,
    /// Minutes after midnight it ends, after `start`, at most [`MINUTES_PER_DAY`].
    pub end: u16,
    /// Shown in the all-day row rather than on the grid.
    pub all_day: bool,
    /// Colour for `data-tone`.
    pub tone: Option<EventTone>,
}

/// An event's colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTone {
    /// The brand accent.
    Accent,
    /// Blue.
    Info,
    /// Green.
    Success,
    /// Amber.
    Warning,
    /// Red.
    Danger,
}

impl EventTone {
    /// The `data-tone` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            EventTone::Accent => "accent",
            EventTone::Info => "info",
            EventTone::Success => "success",
            EventTone::Warning => "warning",
            EventTone::Danger => "danger",
        }
    }
}

impl WeekEvent {
    /// A timed event. `start` and `end` are clamped into the day, `end` after `start`.
    #[must_use]
    pub fn timed(id: impl Into<String>, title: impl Into<String>, date: Date, start: u16, end: u16) -> Self {
        let start = start.min(MINUTES_PER_DAY - 1);
        Self {
            id: id.into(),
            title: title.into(),
            date,
            start,
            end: end.clamp(start + 1, MINUTES_PER_DAY),
            all_day: false,
            tone: None,
        }
    }

    /// An all-day event.
    #[must_use]
    pub fn all_day(id: impl Into<String>, title: impl Into<String>, date: Date) -> Self {
        Self { id: id.into(), title: title.into(), date, start: 0, end: MINUTES_PER_DAY, all_day: true, tone: None }
    }

    /// Set the colour.
    #[must_use]
    pub const fn with_tone(mut self, tone: EventTone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Where it sits.
    #[must_use]
    pub const fn placement(&self) -> Placement {
        // The fields are public, so keep the span inside the day and non-empty here.
        let start = if self.start < MINUTES_PER_DAY { self.start } else { MINUTES_PER_DAY - 1 };
        let end = if self.end > MINUTES_PER_DAY {
            MINUTES_PER_DAY
        } else if self.end <= start {
            start + 1
        } else {
            self.end
        };
        Placement { date: self.date, start, end, all_day: self.all_day }
    }

    /// A copy moved to `p`.
    #[must_use]
    pub fn placed(&self, p: Placement) -> Self {
        Self { date: p.date, start: p.start, end: p.end, all_day: p.all_day, ..self.clone() }
    }
}

/// A day and a time span: where an event is, or where a new one would go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    /// The day.
    pub date: Date,
    /// Start, in minutes after midnight.
    pub start: u16,
    /// End, in minutes after midnight.
    pub end: u16,
    /// The whole day, in the all-day row.
    pub all_day: bool,
}

/// Replace the event with `changed.id` by `changed`. Unknown ids change nothing.
/// Owners call this from `on_change`.
pub fn apply_change(events: &mut [WeekEvent], changed: &WeekEvent) {
    if let Some(e) = events.iter_mut().find(|e| e.id == changed.id) {
        e.clone_from(changed);
    }
}

/// How many days the grid shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WeekView {
    /// One day.
    Day,
    /// Monday to Friday.
    WorkWeek,
    /// Seven days from the week start.
    #[default]
    Week,
}

impl WeekView {
    /// Every view, in toolbar order.
    pub const ALL: [WeekView; 3] = [WeekView::Day, WeekView::WorkWeek, WeekView::Week];

    /// The `data-view` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            WeekView::Day => "day",
            WeekView::WorkWeek => "work-week",
            WeekView::Week => "week",
        }
    }

    /// Button text.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            WeekView::Day => "Day",
            WeekView::WorkWeek => "Work week",
            WeekView::Week => "Week",
        }
    }

    /// What previous and next step by: `"day"` or `"week"`.
    #[must_use]
    pub const fn unit(self) -> &'static str {
        match self {
            WeekView::Day => "day",
            _ => "week",
        }
    }

    const fn step(self) -> i64 {
        match self {
            WeekView::Day => 1,
            _ => 7,
        }
    }
}

/// The days `view` shows around `anchor`. Weeks start on `week_start` (0 = Sunday);
/// the work week is always Monday to Friday.
#[must_use]
pub fn visible_days(view: WeekView, anchor: Date, week_start: u8) -> Vec<Date> {
    let (first, n) = match view {
        WeekView::Day => (anchor, 1),
        WeekView::WorkWeek => (anchor.add_days(-anchor.days_into_week(1)), 5),
        WeekView::Week => (anchor.add_days(-anchor.days_into_week(week_start)), 7),
    };
    (0..n).map(|i| first.add_days(i)).collect()
}

/// Props for [`WeekSchedulerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekSchedulerInput {
    /// Every event. The owner holds them.
    pub events: Vec<WeekEvent>,
    /// Today, for the Today button and the highlighted column.
    pub today: Date,
    /// The date shown first.
    pub default_date: Date,
    /// The view shown first.
    pub default_view: WeekView,
    /// First day of the week, 0 = Sunday … 6 = Saturday.
    pub week_start: u8,
    /// Minutes that drags and arrow keys snap to.
    pub snap: u16,
    /// Length in minutes of an event made by a click or by Enter.
    pub default_duration: u16,
    /// Block every edit. Navigation and selection still work.
    pub disabled: bool,
}

impl Default for WeekSchedulerInput {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            today: Date::default(),
            default_date: Date::default(),
            default_view: WeekView::Week,
            week_start: 1,
            snap: 15,
            default_duration: 60,
            disabled: false,
        }
    }
}

impl WeekSchedulerInput {
    /// The snap step, kept between 1 and 60 minutes.
    #[must_use]
    pub fn step(&self) -> i32 {
        i32::from(self.snap.clamp(1, 60))
    }

    fn duration(&self) -> i32 {
        i32::from(self.default_duration).max(self.step())
    }

    fn find(&self, id: &str) -> Option<&WeekEvent> {
        self.events.iter().find(|e| e.id == id)
    }
}

/// A pointer position on the grid: a visible day's index and minutes after midnight
/// (unsnapped, possibly outside the day; the machine clamps).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPoint {
    /// Index into the visible days.
    pub day: usize,
    /// Minutes after midnight.
    pub minute: i32,
}

/// What a pointer went down on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hit {
    /// Empty time: draft a new event.
    Empty,
    /// An event's body: move it.
    Event(String),
    /// An event's bottom edge: resize it.
    Resize(String),
}

/// What a drag does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragKind {
    /// Drafting a new event.
    Create,
    /// Moving an event.
    Move,
    /// Moving an event's end.
    Resize,
}

impl DragKind {
    /// The `data-dragging` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DragKind::Create => "create",
            DragKind::Move => "move",
            DragKind::Resize => "resize",
        }
    }
}

/// A drag in flight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drag {
    /// What it does.
    pub kind: DragKind,
    /// The event being moved or resized.
    pub id: Option<String>,
    /// Minutes between the event's start and where it was grabbed.
    pub grab: i32,
    /// Where it started.
    pub origin: Placement,
    /// Where it would land now.
    pub draft: Placement,
}

/// What [`WeekSchedulerMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekSchedulerState {
    /// The view.
    pub view: WeekView,
    /// A date inside the range being shown.
    pub anchor: Date,
    /// The selected event.
    pub selected: Option<String>,
    /// The drag in flight.
    pub drag: Option<Drag>,
    /// The keyboard slot: where Enter would create an event.
    pub cursor: Placement,
}

impl WeekSchedulerState {
    /// The visible days.
    #[must_use]
    pub fn days(&self, input: &WeekSchedulerInput) -> Vec<Date> {
        visible_days(self.view, self.anchor, input.week_start)
    }
}

/// Inputs to [`WeekSchedulerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeekSchedulerEvent {
    /// Pointer down on the grid.
    PointerDown {
        /// What it landed on.
        hit: Hit,
        /// Where.
        at: GridPoint,
    },
    /// The pointer moved during a drag.
    PointerMove(GridPoint),
    /// The pointer was released: commit the drag.
    PointerUp,
    /// Escape, or an interrupted pointer: drop the drag.
    Cancel,
    /// A click on an empty all-day cell of this visible day.
    AllDayCreate(usize),
    /// Enter, or a click on an all-day event: select it.
    Select(String),
    /// Move an event by days and minutes.
    Nudge {
        /// The event.
        id: String,
        /// Days, forward or back. Kept inside the visible days.
        days: i32,
        /// Minutes, later or earlier.
        minutes: i32,
    },
    /// Move an event's end by minutes.
    Stretch {
        /// The event.
        id: String,
        /// Minutes, later or earlier.
        minutes: i32,
    },
    /// Move the slot cursor by days and minutes.
    CursorMove {
        /// Days.
        days: i32,
        /// Minutes.
        minutes: i32,
    },
    /// Make the slot cursor longer or shorter.
    CursorStretch(i32),
    /// Enter on the slot cursor: create an event there.
    CursorCreate,
    /// Show the previous day or week.
    Prev,
    /// Show the next day or week.
    Next,
    /// Show today.
    Today,
    /// Switch view.
    SetView(WeekView),
}

/// Side effects from [`WeekSchedulerMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeekSchedulerCommand {
    /// The user made a new event here. The owner adds it.
    Create(Placement),
    /// The user moved or resized an event. The owner stores it (see [`apply_change`]).
    Change(WeekEvent),
    /// The user selected this event.
    Select(String),
    /// The visible days changed, first to last.
    RangeChange(Date, Date),
    /// Scroll the slot cursor into view.
    RevealCursor,
    /// Say this in the live region.
    Announce(String),
}

/// View, navigation, selection and drags for a week scheduler.
#[derive(Debug, Clone, Copy)]
pub struct WeekSchedulerMachine;

type Step = Transition<WeekSchedulerState, WeekSchedulerCommand>;

const DAY: i32 = 1440;

/// An event's length, kept in `0..=DAY` so clamping its start never sees `lo > hi`.
fn length(start: i32, end: i32) -> i32 {
    (end - start).clamp(0, DAY)
}

fn snap_floor(minute: i32, step: i32) -> i32 {
    minute.div_euclid(step) * step
}

fn snap_round(minute: i32, step: i32) -> i32 {
    (minute + step / 2).div_euclid(step) * step
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, reason = "clamped into 0..=1440 first")]
fn minute(m: i32) -> u16 {
    m.clamp(0, DAY) as u16
}

fn span(date: Date, start: i32, end: i32) -> Placement {
    Placement { date, start: minute(start), end: minute(end), all_day: false }
}

fn day_index(days: &[Date], date: Date) -> Option<usize> {
    days.iter().position(|d| *d == date)
}

/// `date` moved by `by` days, kept inside the visible days when it starts inside them.
fn shift_day(days: &[Date], date: Date, by: i32) -> Date {
    match day_index(days, date) {
        Some(i) => {
            let last = i64::try_from(days.len()).unwrap_or(1) - 1;
            let to = (i64::try_from(i).unwrap_or(0) + i64::from(by)).clamp(0, last);
            days[usize::try_from(to).unwrap_or(0)]
        }
        None => date.add_days(i64::from(by)),
    }
}

/// The draft for a drag with the pointer at `at`.
fn drafted(drag: &Drag, at: GridPoint, days: &[Date], step: i32) -> Placement {
    let day = days[at.day.min(days.len() - 1)];
    let (start, end) = (i32::from(drag.origin.start), i32::from(drag.origin.end));
    match drag.kind {
        DragKind::Create => {
            let here = snap_floor(at.minute, step).clamp(0, DAY - step);
            span(drag.origin.date, start.min(here), (here + step).max(end))
        }
        DragKind::Move => {
            let length = length(start, end);
            let to = snap_round(at.minute - drag.grab, step).clamp(0, DAY - length);
            span(day, to, to + length)
        }
        DragKind::Resize => {
            span(drag.origin.date, start, snap_round(at.minute, step).clamp((start + step).min(DAY), DAY))
        }
    }
}

fn placed_text(title: &str, p: Placement) -> String {
    format!("{title}, {}", placement_text(p))
}

fn change(event: &WeekEvent, to: Placement, mut next: WeekSchedulerState) -> Step {
    if to == event.placement() {
        return Transition::stay();
    }
    next.selected = Some(event.id.clone());
    let moved = event.placed(to);
    let said = placed_text(&moved.title, to);
    Transition::with(next, vec![WeekSchedulerCommand::Change(moved), WeekSchedulerCommand::Announce(said)])
}

fn navigate(state: &WeekSchedulerState, input: &WeekSchedulerInput, view: WeekView, anchor: Date) -> Step {
    let before = state.days(input);
    let after = visible_days(view, anchor, input.week_start);
    let mut next = state.clone();
    next.view = view;
    next.anchor = anchor;
    next.drag = None;
    let column = day_index(&before, state.cursor.date).unwrap_or(0).min(after.len() - 1);
    next.cursor.date = if after.contains(&anchor) && anchor == input.today { anchor } else { after[column] };
    if view == WeekView::Day {
        next.cursor.date = anchor;
    }
    let mut commands = Vec::new();
    if before != after {
        commands.push(WeekSchedulerCommand::RangeChange(after[0], after[after.len() - 1]));
        commands.push(WeekSchedulerCommand::Announce(range_title(&after)));
    }
    if next == *state { Transition::emit(commands) } else { Transition::with(next, commands) }
}

impl Machine for WeekSchedulerMachine {
    type Input = WeekSchedulerInput;
    type State = WeekSchedulerState;
    type Event = WeekSchedulerEvent;
    type Command = WeekSchedulerCommand;

    fn initial(input: &WeekSchedulerInput) -> WeekSchedulerState {
        let days = visible_days(input.default_view, input.default_date, input.week_start);
        let date = if days.contains(&input.today) { input.today } else { days[0] };
        WeekSchedulerState {
            view: input.default_view,
            anchor: input.default_date,
            selected: None,
            drag: None,
            cursor: span(date, 9 * 60, 9 * 60 + input.duration()),
        }
    }

    #[allow(clippy::too_many_lines, reason = "one match over every event")]
    fn transition(state: &WeekSchedulerState, event: WeekSchedulerEvent, input: &WeekSchedulerInput) -> Step {
        use WeekSchedulerCommand as C;
        use WeekSchedulerEvent as E;
        let step = input.step();
        let days = state.days(input);
        let mut next = state.clone();
        match event {
            E::Prev => navigate(state, input, state.view, state.anchor.add_days(-state.view.step())),
            E::Next => navigate(state, input, state.view, state.anchor.add_days(state.view.step())),
            E::Today => navigate(state, input, state.view, input.today),
            E::SetView(view) => navigate(state, input, view, state.cursor.date),
            E::Select(id) => {
                if input.find(&id).is_none() {
                    return Transition::stay();
                }
                next.selected = Some(id.clone());
                Transition::with(next, vec![C::Select(id)])
            }
            _ if input.disabled => Transition::stay(),
            E::PointerDown { hit, at } => {
                let date = days[at.day.min(days.len() - 1)];
                let drag = match hit {
                    Hit::Empty => {
                        let here = snap_floor(at.minute, step).clamp(0, DAY - step);
                        let origin = span(date, here, here + step);
                        next.cursor = span(date, here, here + input.duration());
                        Drag { kind: DragKind::Create, id: None, grab: 0, origin, draft: origin }
                    }
                    Hit::Event(id) | Hit::Resize(id) if input.find(&id).is_some_and(|e| e.all_day) => {
                        return Transition::stay();
                    }
                    Hit::Event(id) => {
                        let Some(e) = input.find(&id) else { return Transition::stay() };
                        let grab = at.minute - i32::from(e.start);
                        Drag { kind: DragKind::Move, id: Some(id), grab, origin: e.placement(), draft: e.placement() }
                    }
                    Hit::Resize(id) => {
                        let Some(e) = input.find(&id) else { return Transition::stay() };
                        Drag {
                            kind: DragKind::Resize,
                            id: Some(id),
                            grab: 0,
                            origin: e.placement(),
                            draft: e.placement(),
                        }
                    }
                };
                next.drag = Some(drag);
                Transition::to(next)
            }
            E::PointerMove(at) => {
                let Some(drag) = &mut next.drag else { return Transition::stay() };
                let draft = drafted(drag, at, &days, step);
                if draft == drag.draft {
                    return Transition::stay();
                }
                drag.draft = draft;
                Transition::to(next)
            }
            E::PointerUp => {
                let Some(drag) = next.drag.take() else { return Transition::stay() };
                let still = drag.draft == drag.origin;
                match (drag.kind, drag.id) {
                    (DragKind::Create, _) => {
                        let p = if still {
                            let start = i32::from(drag.origin.start).min(DAY - input.duration());
                            span(drag.origin.date, start, start + input.duration())
                        } else {
                            drag.draft
                        };
                        let said = format!("New event, {}", placement_text(p));
                        Transition::with(next, vec![C::Create(p), C::Announce(said)])
                    }
                    (_, Some(id)) if still => {
                        next.selected = Some(id.clone());
                        Transition::with(next, vec![C::Select(id)])
                    }
                    (_, Some(id)) => match input.find(&id) {
                        Some(e) => {
                            let t = change(e, drag.draft, next.clone());
                            if t.state.is_none() { Transition::to(next) } else { t }
                        }
                        None => Transition::to(next),
                    },
                    (_, None) => Transition::to(next),
                }
            }
            E::Cancel => {
                if next.drag.take().is_none() {
                    return Transition::stay();
                }
                Transition::to(next)
            }
            E::AllDayCreate(day) => {
                let date = days[day.min(days.len() - 1)];
                let p = Placement { date, start: 0, end: MINUTES_PER_DAY, all_day: true };
                let said = format!("New event, {}", placement_text(p));
                Transition::emit(vec![C::Create(p), C::Announce(said)])
            }
            E::Nudge { id, days: by, minutes } => {
                let Some(e) = input.find(&id) else { return Transition::stay() };
                let date = shift_day(&days, e.date, by);
                let to = if e.all_day {
                    Placement { date, ..e.placement() }
                } else {
                    let (start, end) = (i32::from(e.start), i32::from(e.end));
                    let length = length(start, end);
                    let at = (start + minutes).clamp(0, DAY - length);
                    span(date, at, at + length)
                };
                change(e, to, next)
            }
            E::Stretch { id, minutes } => {
                let Some(e) = input.find(&id).filter(|e| !e.all_day) else { return Transition::stay() };
                let start = i32::from(e.start);
                let end = (i32::from(e.end) + minutes).clamp((start + step).min(DAY), DAY);
                change(e, span(e.date, start, end), next)
            }
            E::CursorMove { days: by, minutes } => {
                let c = state.cursor;
                let length = length(i32::from(c.start), i32::from(c.end));
                let at = (i32::from(c.start) + minutes).clamp(0, DAY - length);
                next.cursor = span(shift_day(&days, c.date, by), at, at + length);
                cursor_moved(state, next)
            }
            E::CursorStretch(minutes) => {
                let c = state.cursor;
                let start = i32::from(c.start);
                next.cursor.end = minute((i32::from(c.end) + minutes).clamp((start + step).min(DAY), DAY));
                cursor_moved(state, next)
            }
            E::CursorCreate => {
                let said = format!("New event, {}", placement_text(state.cursor));
                Transition::emit(vec![C::Create(state.cursor), C::Announce(said)])
            }
        }
    }

    fn sync(state: &WeekSchedulerState, input: &WeekSchedulerInput) -> Option<WeekSchedulerState> {
        // Forget a selection or drag whose event the owner removed.
        let mut next = state.clone();
        if next.selected.as_deref().is_some_and(|id| input.find(id).is_none()) {
            next.selected = None;
        }
        if next.drag.as_ref().and_then(|d| d.id.as_deref()).is_some_and(|id| input.find(id).is_none()) {
            next.drag = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

fn cursor_moved(state: &WeekSchedulerState, next: WeekSchedulerState) -> Step {
    if next.cursor == state.cursor {
        return Transition::stay();
    }
    let said = placement_text(next.cursor);
    Transition::with(next, vec![WeekSchedulerCommand::RevealCursor, WeekSchedulerCommand::Announce(said)])
}

/// The events to draw: the owner's, with a move or resize in flight applied.
#[must_use]
pub fn preview(state: &WeekSchedulerState, input: &WeekSchedulerInput) -> Vec<WeekEvent> {
    let mut events = input.events.clone();
    if let Some(Drag { id: Some(id), draft, .. }) = &state.drag
        && let Some(e) = events.iter_mut().find(|e| e.id == *id)
    {
        *e = e.placed(*draft);
    }
    events
}

/// The new event being drafted by a drag, if any.
#[must_use]
pub fn ghost(state: &WeekSchedulerState) -> Option<Placement> {
    state.drag.as_ref().filter(|d| d.kind == DragKind::Create).map(|d| d.draft)
}

mod layout;
#[cfg(test)]
mod tests;
mod text;

pub use layout::{Placed, all_day_rows, all_day_style, block_style, column_style, layout, minute_y, pack, point_at};
pub use text::{
    clock, date_text, event_label, month_name, placement_text, range_title, time_range, weekday_name, weekday_short,
};
