//! Gantt connector. Anatomy: root, toolbar, scale-toggle, body, list, row,
//! scroller, timeline, header, month, tick, grid, connectors, today, bar,
//! handle, instructions, live. Every part carries `data-scope="gantt"` and
//! its `data-part`. The root carries `data-scale` (`day` or `week`).
//!
//! Bars are focusable `role="button"`s. Pointer drags use pointer capture on the
//! bar (or its two edge handles), snap to whole days and show the landing dates
//! live; keyboard moves follow Space, arrows, Space. Dependencies are SVG
//! paths computed by [`Timeline::connectors`].

pub use hydroxy_machines::date::Date;
pub use hydroxy_machines::gantt::*;

/// The `data-scope` value on every gantt part.
pub const SCOPE: &str = "gantt";

/// Read by screen readers on every bar, through `aria-describedby`.
pub const INSTRUCTIONS: &str = "Press Space to pick up the bar. Arrow keys move it a day at a time, Shift with the arrows moves its end, Alt with the arrows moves its start. Space drops it, Escape cancels.";

/// The bar's DOM id.
#[must_use]
pub fn bar_id(root: &str, task: &str) -> String {
    format!("{root}-bar-{task}")
}

/// The instructions' DOM id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// Custom properties the stylesheet reads: row, bar and header heights in px.
#[must_use]
pub fn root_style() -> String {
    format!("--gantt-row: {ROW_HEIGHT}px; --gantt-bar: {BAR_HEIGHT}px; --gantt-header: {HEADER_HEIGHT}px;")
}

/// Attributes for a bar (`<div role="button">`).
#[derive(Debug, Clone, PartialEq)]
pub struct BarAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// `aria-label`: title, dates and what it follows.
    pub label: String,
    /// `tabindex="0"`; false when the chart is disabled.
    pub movable: bool,
    /// `data-state`: `idle`, `grabbed` (keyboard) or `dragging` (pointer).
    pub state: &'static str,
    /// `aria-pressed`: picked up with the keyboard.
    pub pressed: bool,
    /// `aria-describedby`: the instructions' id.
    pub described_by: String,
    /// `style`: position and size in px.
    pub style: String,
}

/// The accessible name: `"Design, Oct 29 to Nov 3, after Research"`.
#[must_use]
pub fn bar_label(task: &GanttTask, all: &[GanttTask]) -> String {
    let mut label = format!("{}, {}", task.title, range_text(task.start, task.end));
    let after: Vec<&str> =
        task.depends_on.iter().filter_map(|id| all.iter().find(|t| t.id == *id)).map(|t| t.title.as_str()).collect();
    if !after.is_empty() {
        label.push_str(", after ");
        label.push_str(&after.join(" and "));
    }
    label
}

/// `left`, `top`, `width` and `height` for a box.
#[must_use]
pub fn rect_style(r: Rect) -> String {
    format!("left: {}px; top: {}px; width: {}px; height: {}px;", r.x, r.y, r.width, r.height)
}

/// Attributes for `task` in row `row` of `tasks`, the list being drawn (the preview).
#[must_use]
pub fn bar_attrs(
    root: &str,
    state: &GanttState,
    input: &GanttInput,
    tasks: &[GanttTask],
    timeline: &Timeline,
    row: usize,
) -> BarAttrs {
    let task = &tasks[row];
    let drag = state.drag.as_ref().filter(|d| d.id == task.id);
    BarAttrs {
        id: bar_id(root, &task.id),
        label: bar_label(task, tasks),
        movable: !input.disabled,
        state: match drag {
            Some(d) if d.keyboard => "grabbed",
            Some(_) => "dragging",
            None => "idle",
        },
        pressed: drag.is_some_and(|d| d.keyboard),
        described_by: instructions_id(root),
        style: rect_style(timeline.bar(task.start, task.end, row)),
    }
}

/// Days to hand the machine for a pointer that moved `dx` px, kept on the axis.
#[must_use]
pub fn pointer_days(timeline: &Timeline, drag: &Drag, dx: f64) -> i64 {
    let days = timeline.days_for_px(dx);
    let (start, end) = drag.origin;
    match drag.mode {
        DragMode::Move => timeline.clamp_delta(start, end, days),
        DragMode::ResizeStart => timeline.clamp_delta(start, start, days),
        DragMode::ResizeEnd => timeline.clamp_delta(end, end, days),
    }
}

/// Map a `KeyboardEvent.key` on bar `task` to a machine event. `Some` means: prevent default.
#[must_use]
pub fn bar_keydown(key: &str, shift: bool, alt: bool, task: &str, state: &GanttState) -> Option<GanttEvent> {
    let carrying = state.drag.as_ref().is_some_and(|d| d.keyboard);
    let mode = if shift {
        DragMode::ResizeEnd
    } else if alt {
        DragMode::ResizeStart
    } else {
        DragMode::Move
    };
    match (key, carrying) {
        (" " | "Enter", false) => Some(GanttEvent::Grab(task.to_string())),
        (" " | "Enter", true) => Some(GanttEvent::Drop),
        ("Escape", _) if state.drag.is_some() => Some(GanttEvent::Cancel),
        ("ArrowLeft", true) => Some(GanttEvent::Nudge { mode, days: -1 }),
        ("ArrowRight", true) => Some(GanttEvent::Nudge { mode, days: 1 }),
        ("ArrowUp", false) => Some(GanttEvent::FocusStep { from: task.to_string(), by: -1 }),
        ("ArrowDown", false) => Some(GanttEvent::FocusStep { from: task.to_string(), by: 1 }),
        _ => None,
    }
}

/// Default English wording for an announcement.
#[must_use]
pub fn announcement_text(a: &Announcement) -> String {
    match a {
        Announcement::Grabbed { title, range } => {
            format!("Picked up {title}, {}. Arrow keys move it.", range_text(range.0, range.1))
        }
        Announcement::Moved { title, range } => format!("{title}, {}.", range_text(range.0, range.1)),
        Announcement::Dropped { title, range } => format!("Dropped {title}, {}.", range_text(range.0, range.1)),
        Announcement::Cancelled { title } => format!("Move cancelled. {title} is back where it started."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn tasks() -> Vec<GanttTask> {
        vec![
            GanttTask::new("a", "Research", d("2026-10-26"), d("2026-10-28")),
            GanttTask::new("b", "Design", d("2026-10-29"), d("2026-11-03")).depends_on(["a"]),
        ]
    }

    #[test]
    fn labels_name_the_dates_and_predecessors() {
        let t = tasks();
        assert_eq!(bar_label(&t[0], &t), "Research, Oct 26 to Oct 28");
        assert_eq!(bar_label(&t[1], &t), "Design, Oct 29 to Nov 3, after Research");
    }

    #[test]
    fn bar_attrs_show_the_drag_state() {
        let i = GanttInput { default_tasks: tasks(), ..GanttInput::default() };
        let mut s = GanttState { tasks: Controllable::new(tasks(), false), ..GanttState::default() };
        let tl = Timeline::fit(GanttScale::Day, &tasks(), None);
        assert_eq!(bar_attrs("g", &s, &i, &tasks(), &tl, 0).state, "idle");
        s.drag = Some(Drag {
            id: "a".into(),
            mode: DragMode::Move,
            origin: (d("2026-10-26"), d("2026-10-28")),
            draft: (d("2026-10-26"), d("2026-10-28")),
            keyboard: true,
        });
        let a = bar_attrs("g", &s, &i, &tasks(), &tl, 0);
        assert_eq!((a.state, a.pressed), ("grabbed", true));
        assert_eq!(bar_attrs("g", &s, &i, &tasks(), &tl, 1).state, "idle");
        assert_eq!(a.style, "left: 224px; top: 8px; width: 96px; height: 24px;");
    }

    #[test]
    fn keys_pick_up_nudge_and_drop() {
        let mut s = GanttState { tasks: Controllable::new(tasks(), false), ..GanttState::default() };
        assert_eq!(bar_keydown(" ", false, false, "a", &s), Some(GanttEvent::Grab("a".into())));
        assert_eq!(bar_keydown("ArrowLeft", false, false, "a", &s), None);
        assert_eq!(
            bar_keydown("ArrowDown", false, false, "a", &s),
            Some(GanttEvent::FocusStep { from: "a".into(), by: 1 })
        );
        let origin = (d("2026-10-26"), d("2026-10-28"));
        s.drag = Some(Drag { id: "a".into(), mode: DragMode::Move, origin, draft: origin, keyboard: true });
        assert_eq!(
            bar_keydown("ArrowRight", false, false, "a", &s),
            Some(GanttEvent::Nudge { mode: DragMode::Move, days: 1 })
        );
        assert_eq!(
            bar_keydown("ArrowRight", true, false, "a", &s),
            Some(GanttEvent::Nudge { mode: DragMode::ResizeEnd, days: 1 })
        );
        assert_eq!(
            bar_keydown("ArrowLeft", false, true, "a", &s),
            Some(GanttEvent::Nudge { mode: DragMode::ResizeStart, days: -1 })
        );
        assert_eq!(bar_keydown("ArrowDown", false, false, "a", &s), None);
        assert_eq!(bar_keydown(" ", false, false, "a", &s), Some(GanttEvent::Drop));
        assert_eq!(bar_keydown("Escape", false, false, "a", &s), Some(GanttEvent::Cancel));
    }

    #[test]
    fn pointer_days_stay_on_the_axis() {
        let tl = Timeline::fit(GanttScale::Day, &tasks(), None);
        let drag = Drag {
            id: "a".into(),
            mode: DragMode::Move,
            origin: (d("2026-10-26"), d("2026-10-28")),
            draft: (d("2026-10-26"), d("2026-10-28")),
            keyboard: false,
        };
        assert_eq!(pointer_days(&tl, &drag, 64.0), 2);
        assert_eq!(pointer_days(&tl, &drag, -10_000.0), -7);
        let resize = Drag { mode: DragMode::ResizeEnd, ..drag };
        assert_eq!(pointer_days(&tl, &resize, 33.0), 1);
    }

    #[test]
    fn announcements_read_well() {
        let a = Announcement::Dropped { title: "Design".into(), range: (d("2026-10-29"), d("2026-11-03")) };
        assert_eq!(announcement_text(&a), "Dropped Design, Oct 29 to Nov 3.");
    }
}
