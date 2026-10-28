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

use std::cmp::Reverse;

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
    /// Colour name for `data-tone`: `accent`, `info`, `success`, `warning` or `danger`.
    pub tone: Option<String>,
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
    pub fn with_tone(mut self, tone: impl Into<String>) -> Self {
        self.tone = Some(tone.into());
        self
    }

    /// Where it sits.
    #[must_use]
    pub const fn placement(&self) -> Placement {
        Placement { date: self.date, start: self.start, end: self.end, all_day: self.all_day }
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            let length = end - start;
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
                    let at = (start + minutes).clamp(0, DAY - (end - start));
                    span(date, at, at + end - start)
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
                let length = i32::from(c.end) - i32::from(c.start);
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

/// Lanes for overlapping spans. Returns `(lane, lanes)` for each span, in input
/// order: spans that overlap, directly or through others, form a cluster that
/// splits its width into `lanes`; each span takes the leftmost lane free at its
/// start. Earlier starts go first, and longer spans first on a tie.
#[must_use]
pub fn pack(spans: &[(u16, u16)]) -> Vec<(usize, usize)> {
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by_key(|&i| (spans[i].0, Reverse(spans[i].1)));
    let mut out = vec![(0, 1); spans.len()];
    let mut cluster: Vec<usize> = Vec::new();
    let mut lane_ends: Vec<u16> = Vec::new();
    let mut cluster_end = 0;
    let finish = |cluster: &mut Vec<usize>, lanes: usize, out: &mut [(usize, usize)]| {
        for &i in &*cluster {
            out[i].1 = lanes;
        }
        cluster.clear();
    };
    for i in order {
        let (start, end) = spans[i];
        let end = end.max(start + 1);
        if !cluster.is_empty() && start >= cluster_end {
            finish(&mut cluster, lane_ends.len(), &mut out);
            lane_ends.clear();
        }
        let lane = if let Some(free) = lane_ends.iter().position(|&e| e <= start) {
            free
        } else {
            lane_ends.push(0);
            lane_ends.len() - 1
        };
        lane_ends[lane] = end;
        out[i].0 = lane;
        cluster_end = if cluster.is_empty() { end } else { cluster_end.max(end) };
        cluster.push(i);
    }
    finish(&mut cluster, lane_ends.len(), &mut out);
    out
}

/// A timed event placed on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    /// Index into the events given to [`layout`].
    pub index: usize,
    /// Index into the visible days.
    pub day: usize,
    /// Its lane within the day.
    pub lane: usize,
    /// Lanes its cluster splits into.
    pub lanes: usize,
}

/// Place every timed event that falls on a visible day.
#[must_use]
pub fn layout(events: &[WeekEvent], days: &[Date]) -> Vec<Placed> {
    let mut out = Vec::new();
    for (day, date) in days.iter().enumerate() {
        let on: Vec<usize> = (0..events.len()).filter(|&i| !events[i].all_day && events[i].date == *date).collect();
        let spans: Vec<(u16, u16)> = on.iter().map(|&i| (events[i].start, events[i].end)).collect();
        for (&index, (lane, lanes)) in on.iter().zip(pack(&spans)) {
            out.push(Placed { index, day, lane, lanes });
        }
    }
    out
}

/// Rows for the all-day events on each visible day: `lane` is the row, `lanes`
/// the number of all-day events that day. Order follows `events`.
#[must_use]
pub fn all_day_rows(events: &[WeekEvent], days: &[Date]) -> Vec<Placed> {
    let mut out = Vec::new();
    for (day, date) in days.iter().enumerate() {
        let on: Vec<usize> = (0..events.len()).filter(|&i| events[i].all_day && events[i].date == *date).collect();
        let lanes = on.len();
        out.extend(on.into_iter().enumerate().map(|(lane, index)| Placed { index, day, lane, lanes }));
    }
    out
}

/// Px from the top of the grid for a minute of the day.
#[must_use]
pub fn minute_y(minute: u16) -> f64 {
    f64::from(minute) * HOUR_HEIGHT / 60.0
}

/// The grid point under `(x, y)` px, measured from the grid's top-left, for a grid
/// `width` px wide showing `days` days.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "clamped to the day range first"
)]
pub fn point_at(x: f64, y: f64, width: f64, days: usize) -> GridPoint {
    let n = days.max(1);
    let column = if width > 0.0 { width / n as f64 } else { 1.0 };
    let day = (x / column).floor().clamp(0.0, (n - 1) as f64) as usize;
    let minute = (y * 60.0 / HOUR_HEIGHT).round().clamp(-1.0e6, 1.0e6) as i32;
    GridPoint { day, minute }
}

/// `left`, `width`, `top` and `height` for a block in lane `lane` of `lanes`, on
/// visible day `day` of `days`, from `start` to `end`.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "a handful of days and lanes")]
pub fn block_style(day: usize, lane: usize, lanes: usize, days: usize, start: u16, end: u16) -> String {
    let (n, lanes) = (days.max(1) as f64, lanes.max(1) as f64);
    let left = (day as f64 + lane as f64 / lanes) / n * 100.0;
    let width = 100.0 / n / lanes;
    let (top, height) = (minute_y(start), minute_y(end.max(start)) - minute_y(start));
    format!("left: {left:.4}%; width: {width:.4}%; top: {top}px; height: {height}px;")
}

/// `left` and `width` for visible day `day` of `days`.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "a handful of days")]
pub fn column_style(day: usize, days: usize) -> String {
    let n = days.max(1) as f64;
    format!("left: {:.4}%; width: {:.4}%;", day as f64 / n * 100.0, 100.0 / n)
}

/// `left`, `width` and `top` for an all-day event in row `row` of day `day`. The
/// row height comes from the stylesheet's `--ws-chip`.
#[must_use]
pub fn all_day_style(day: usize, row: usize, days: usize) -> String {
    format!("{} top: calc({row} * var(--ws-chip));", column_style(day, days))
}

const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// `"Monday"`.
#[must_use]
pub fn weekday_name(d: Date) -> &'static str {
    WEEKDAYS[usize::from(d.weekday()) % 7]
}

/// `"Mon"`.
#[must_use]
pub fn weekday_short(d: Date) -> &'static str {
    &weekday_name(d)[..3]
}

/// `"November"`.
#[must_use]
pub fn month_name(d: Date) -> &'static str {
    MONTHS[usize::from(d.month().clamp(1, 12) - 1)]
}

/// `"Tuesday 3 November"`.
#[must_use]
pub fn date_text(d: Date) -> String {
    format!("{} {} {}", weekday_name(d), d.day(), month_name(d))
}

/// `"09:00"`. Midnight at the end of a day is `"24:00"`.
#[must_use]
pub fn clock(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

/// `"09:00 to 10:30"`.
#[must_use]
pub fn time_range(start: u16, end: u16) -> String {
    format!("{} to {}", clock(start), clock(end))
}

/// `"Tuesday 3 November, 09:00 to 10:00"`, or `"Tuesday 3 November, all day"`.
#[must_use]
pub fn placement_text(p: Placement) -> String {
    if p.all_day {
        format!("{}, all day", date_text(p.date))
    } else {
        format!("{}, {}", date_text(p.date), time_range(p.start, p.end))
    }
}

/// An event's accessible name: `"Design review, Tuesday 3 November, 10:00 to 11:00"`.
#[must_use]
pub fn event_label(e: &WeekEvent) -> String {
    placed_text(&e.title, e.placement())
}

/// The toolbar title for the visible days: `"Tuesday 3 November 2026"`,
/// `"2 – 8 November 2026"`, `"30 November – 6 December 2026"` or
/// `"28 December 2026 – 3 January 2027"`.
#[must_use]
pub fn range_title(days: &[Date]) -> String {
    let (Some(&a), Some(&b)) = (days.first(), days.last()) else { return String::new() };
    if a == b {
        format!("{} {}", date_text(a), a.year())
    } else if a.year() != b.year() {
        format!("{} {} {} – {} {} {}", a.day(), month_name(a), a.year(), b.day(), month_name(b), b.year())
    } else if a.month() != b.month() {
        format!("{} {} – {} {} {}", a.day(), month_name(a), b.day(), month_name(b), b.year())
    } else {
        format!("{} – {} {} {}", a.day(), b.day(), month_name(b), b.year())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use WeekSchedulerEvent as E;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn input() -> WeekSchedulerInput {
        WeekSchedulerInput {
            events: vec![
                WeekEvent::timed("standup", "Standup", d("2026-11-03"), 9 * 60, 9 * 60 + 15),
                WeekEvent::timed("review", "Design review", d("2026-11-03"), 10 * 60, 11 * 60),
                WeekEvent::all_day("offsite", "Offsite", d("2026-11-05")),
            ],
            today: d("2026-11-03"),
            default_date: d("2026-11-03"),
            ..WeekSchedulerInput::default()
        }
    }

    fn run(input: &WeekSchedulerInput, events: Vec<E>) -> (WeekSchedulerState, Vec<WeekSchedulerCommand>) {
        let mut s = WeekSchedulerMachine::initial(input);
        let mut all = Vec::new();
        for e in events {
            let t = WeekSchedulerMachine::transition(&s, e, input).resolve(&s);
            s = t.state;
            all.extend(t.commands);
        }
        (s, all)
    }

    fn changed(cmds: &[WeekSchedulerCommand]) -> Option<&WeekEvent> {
        cmds.iter().find_map(|c| if let WeekSchedulerCommand::Change(e) = c { Some(e) } else { None })
    }

    fn created(cmds: &[WeekSchedulerCommand]) -> Option<Placement> {
        cmds.iter().find_map(|c| if let WeekSchedulerCommand::Create(p) = c { Some(*p) } else { None })
    }

    #[test]
    fn packing_puts_overlaps_side_by_side() {
        // Disjoint spans each get the full width.
        assert_eq!(pack(&[(0, 60), (60, 120)]), [(0, 1), (0, 1)]);
        // Two overlapping.
        assert_eq!(pack(&[(0, 60), (30, 90)]), [(0, 2), (1, 2)]);
        // A chain: a overlaps b, b overlaps c, but a and c don't. c reuses a's lane,
        // and all three share the cluster's two lanes.
        assert_eq!(pack(&[(0, 60), (30, 120), (90, 150)]), [(0, 2), (1, 2), (0, 2)]);
        // Three at once, then a later lone event back at full width.
        assert_eq!(pack(&[(60, 120), (60, 90), (70, 80), (200, 260)]), [(0, 3), (1, 3), (2, 3), (0, 1)]);
        // Input order doesn't matter; longer first on a tie.
        assert_eq!(pack(&[(30, 90), (0, 60)]), [(1, 2), (0, 2)]);
        assert_eq!(pack(&[(0, 30), (0, 90)]), [(1, 2), (0, 2)]);
        assert!(pack(&[]).is_empty());
    }

    #[test]
    fn layout_places_only_timed_events_on_visible_days() {
        let i = input();
        let days = visible_days(WeekView::Week, d("2026-11-03"), 1);
        let placed = layout(&i.events, &days);
        assert_eq!(placed.len(), 2);
        assert!(placed.iter().all(|p| p.day == 1 && p.lanes == 1));
    }

    #[test]
    fn all_day_events_stack_per_day() {
        let mut events = input().events;
        events.push(WeekEvent::all_day("holiday", "Bank holiday", d("2026-11-05")));
        let days = visible_days(WeekView::Week, d("2026-11-03"), 1);
        let rows = all_day_rows(&events, &days);
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].index, rows[0].day, rows[0].lane, rows[0].lanes), (2, 3, 0, 2));
        assert_eq!((rows[1].index, rows[1].lane), (3, 1));
        assert_eq!(all_day_style(3, 1, 7), "left: 42.8571%; width: 14.2857%; top: calc(1 * var(--ws-chip));");
    }

    #[test]
    fn views_show_the_right_days() {
        let wed = d("2026-11-04");
        assert_eq!(visible_days(WeekView::Day, wed, 1), [wed]);
        let week = visible_days(WeekView::Week, wed, 1);
        assert_eq!((week[0], week[6]), (d("2026-11-02"), d("2026-11-08")));
        assert_eq!(visible_days(WeekView::Week, wed, 0)[0], d("2026-11-01"));
        let work = visible_days(WeekView::WorkWeek, d("2026-11-08"), 0);
        assert_eq!((work[0], work[4]), (d("2026-11-02"), d("2026-11-06")));
    }

    #[test]
    fn navigation_steps_and_reports_the_range() {
        let i = input();
        let (s, cmds) = run(&i, vec![E::Next]);
        assert_eq!(s.anchor, d("2026-11-10"));
        assert_eq!(cmds[0], WeekSchedulerCommand::RangeChange(d("2026-11-09"), d("2026-11-15")));
        assert_eq!(s.cursor.date, d("2026-11-10"));
        let (s, _) = run(&i, vec![E::Next, E::Next, E::Today]);
        assert_eq!(s.anchor, d("2026-11-03"));
        let (s, cmds) = run(&i, vec![E::SetView(WeekView::Day), E::Prev]);
        assert_eq!(s.days(&i), [d("2026-11-02")]);
        assert!(cmds.contains(&WeekSchedulerCommand::Announce("Monday 2 November 2026".into())));
        // Today while already showing today changes nothing.
        let s0 = WeekSchedulerMachine::initial(&i);
        let t = WeekSchedulerMachine::transition(&s0, E::Today, &i);
        assert!(t.state.is_none() && t.commands.is_empty());
    }

    #[test]
    fn drag_on_empty_time_drafts_a_snapped_event() {
        let i = input();
        let down = E::PointerDown { hit: Hit::Empty, at: GridPoint { day: 2, minute: 13 * 60 + 7 } };
        let (s, cmds) = run(&i, vec![down.clone(), E::PointerMove(GridPoint { day: 4, minute: 14 * 60 + 20 })]);
        assert!(cmds.is_empty());
        // Day stays where the drag began; the end covers the slot under the pointer.
        assert_eq!(ghost(&s), Some(span(d("2026-11-04"), 13 * 60, 14 * 60 + 30)));
        let (_, cmds) =
            run(&i, vec![down.clone(), E::PointerMove(GridPoint { day: 2, minute: 12 * 60 + 40 }), E::PointerUp]);
        // Dragging upward extends the start instead.
        assert_eq!(created(&cmds), Some(span(d("2026-11-04"), 12 * 60 + 30, 13 * 60 + 15)));
        // A plain click makes a default-length event.
        let (s, cmds) = run(&i, vec![down, E::PointerUp]);
        assert_eq!(created(&cmds), Some(span(d("2026-11-04"), 13 * 60, 14 * 60)));
        assert!(s.drag.is_none());
    }

    #[test]
    fn dragging_an_event_moves_it_across_days() {
        let i = input();
        // Grabbed 20 minutes into the review, dropped on Thursday around 14:22.
        let (s, cmds) = run(
            &i,
            vec![
                E::PointerDown { hit: Hit::Event("review".into()), at: GridPoint { day: 1, minute: 10 * 60 + 20 } },
                E::PointerMove(GridPoint { day: 3, minute: 14 * 60 + 42 }),
            ],
        );
        assert!(cmds.is_empty());
        let shown = preview(&s, &i);
        let review = shown.iter().find(|e| e.id == "review").unwrap();
        assert_eq!((review.date, review.start, review.end), (d("2026-11-05"), 14 * 60 + 15, 15 * 60 + 15));
        let (s, cmds) = run(
            &i,
            vec![
                E::PointerDown { hit: Hit::Event("review".into()), at: GridPoint { day: 1, minute: 10 * 60 + 20 } },
                E::PointerMove(GridPoint { day: 3, minute: 14 * 60 + 42 }),
                E::PointerUp,
            ],
        );
        let e = changed(&cmds).unwrap();
        assert_eq!((e.date, e.start, e.end), (d("2026-11-05"), 14 * 60 + 15, 15 * 60 + 15));
        assert_eq!(s.selected.as_deref(), Some("review"));
        // A click without a move selects.
        let (_, cmds) = run(
            &i,
            vec![
                E::PointerDown { hit: Hit::Event("review".into()), at: GridPoint { day: 1, minute: 630 } },
                E::PointerUp,
            ],
        );
        assert_eq!(cmds, [WeekSchedulerCommand::Select("review".into())]);
    }

    #[test]
    fn moves_stay_inside_the_day() {
        let i = input();
        let (_, cmds) = run(
            &i,
            vec![
                E::PointerDown { hit: Hit::Event("review".into()), at: GridPoint { day: 1, minute: 600 } },
                E::PointerMove(GridPoint { day: 1, minute: 23 * 60 + 50 }),
                E::PointerUp,
            ],
        );
        let e = changed(&cmds).unwrap();
        assert_eq!((e.start, e.end), (23 * 60, 24 * 60));
    }

    #[test]
    fn resizing_snaps_and_keeps_one_step() {
        let i = input();
        let down = E::PointerDown { hit: Hit::Resize("review".into()), at: GridPoint { day: 1, minute: 660 } };
        let (_, cmds) =
            run(&i, vec![down.clone(), E::PointerMove(GridPoint { day: 5, minute: 12 * 60 + 8 }), E::PointerUp]);
        let e = changed(&cmds).unwrap();
        assert_eq!((e.date, e.start, e.end), (d("2026-11-03"), 600, 12 * 60 + 15));
        let (_, cmds) = run(&i, vec![down, E::PointerMove(GridPoint { day: 1, minute: 0 }), E::PointerUp]);
        assert_eq!(changed(&cmds).unwrap().end, 615);
    }

    #[test]
    fn keyboard_moves_and_resizes() {
        let i = input();
        let (_, cmds) = run(&i, vec![E::Nudge { id: "review".into(), days: 0, minutes: 15 }]);
        assert_eq!(changed(&cmds).unwrap().start, 615);
        let (_, cmds) = run(&i, vec![E::Nudge { id: "review".into(), days: 1, minutes: 0 }]);
        assert_eq!(changed(&cmds).unwrap().date, d("2026-11-04"));
        // Kept inside the visible week.
        let (_, cmds) = run(&i, vec![E::Nudge { id: "review".into(), days: -5, minutes: 0 }]);
        assert_eq!(changed(&cmds).unwrap().date, d("2026-11-02"));
        let (_, cmds) = run(&i, vec![E::Stretch { id: "review".into(), minutes: -15 }]);
        assert_eq!(changed(&cmds).unwrap().end, 645);
        // All-day events only move between days.
        let (_, cmds) = run(&i, vec![E::Nudge { id: "offsite".into(), days: 0, minutes: 15 }]);
        assert!(cmds.is_empty());
        let (_, cmds) = run(&i, vec![E::Nudge { id: "offsite".into(), days: 1, minutes: 0 }]);
        assert_eq!(changed(&cmds).unwrap().date, d("2026-11-06"));
        // Earlier than midnight is clamped.
        let (_, cmds) = run(&i, vec![E::Nudge { id: "standup".into(), days: 0, minutes: -10_000 }]);
        assert_eq!(changed(&cmds).unwrap().start, 0);
    }

    #[test]
    fn keyboard_creation_uses_the_slot_cursor() {
        let i = input();
        let (s, cmds) = run(
            &i,
            vec![
                E::CursorMove { days: 1, minutes: 15 },
                E::CursorStretch(30),
                E::CursorStretch(-1000),
                E::CursorStretch(15),
                E::CursorCreate,
            ],
        );
        assert_eq!(s.cursor, span(d("2026-11-04"), 9 * 60 + 15, 9 * 60 + 45));
        assert_eq!(created(&cmds), Some(s.cursor));
        assert!(cmds.contains(&WeekSchedulerCommand::RevealCursor));
    }

    #[test]
    fn disabled_blocks_edits_but_not_navigation() {
        let i = WeekSchedulerInput { disabled: true, ..input() };
        let (_, cmds) = run(&i, vec![E::Nudge { id: "review".into(), days: 1, minutes: 0 }, E::CursorCreate]);
        assert!(cmds.is_empty());
        let (s, _) = run(&i, vec![E::Next, E::Select("review".into())]);
        assert_eq!(s.anchor, d("2026-11-10"));
        assert_eq!(s.selected.as_deref(), Some("review"));
    }

    #[test]
    fn sync_forgets_removed_events() {
        let i = input();
        let (s, _) = run(&i, vec![E::Select("review".into())]);
        let gone = WeekSchedulerInput { events: vec![], ..input() };
        assert_eq!(hydroxy_statechart::sync_or_keep::<WeekSchedulerMachine>(&s, &gone).selected, None);
        assert_eq!(WeekSchedulerMachine::sync(&s, &i), None);
    }

    #[test]
    fn geometry_and_text() {
        assert_eq!(point_at(250.0, 480.0, 700.0, 7), GridPoint { day: 2, minute: 600 });
        assert_eq!(point_at(-5.0, -10.0, 700.0, 7).day, 0);
        assert_eq!(point_at(9999.0, 0.0, 700.0, 7).day, 6);
        assert!((minute_y(90) - 72.0).abs() < 1e-9);
        assert_eq!(block_style(1, 1, 2, 2, 60, 120), "left: 75.0000%; width: 25.0000%; top: 48px; height: 48px;");
        assert_eq!(clock(1440), "24:00");
        assert_eq!(date_text(d("2026-11-03")), "Tuesday 3 November");
        let week = visible_days(WeekView::Week, d("2026-11-03"), 1);
        assert_eq!(range_title(&week), "2 – 8 November 2026");
        assert_eq!(range_title(&visible_days(WeekView::Week, d("2026-12-01"), 1)), "30 November – 6 December 2026");
        assert_eq!(range_title(&visible_days(WeekView::Week, d("2026-12-30"), 1)), "28 December 2026 – 3 January 2027");
        assert_eq!(event_label(&input().events[1]), "Design review, Tuesday 3 November, 10:00 to 11:00");
        assert_eq!(placement_text(input().events[2].placement()), "Thursday 5 November, all day");
    }
}
