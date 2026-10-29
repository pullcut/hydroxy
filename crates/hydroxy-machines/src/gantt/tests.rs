//! Tests for the gantt machine and its geometry.

use super::*;
use crate::date::d;
use hydroxy_statechart::step as advance;

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

fn range(s: &GanttState, id: &str) -> (Date, Date) {
    let t = find(&s.tasks.value, id).unwrap();
    (t.start, t.end)
}

fn start(id: &str, mode: DragMode) -> GanttEvent {
    GanttEvent::DragStart { id: id.into(), mode }
}

#[test]
fn pointer_move_snaps_to_days_and_commits_on_drop() {
    let i = input();
    let s = advance::<GanttMachine>(&GanttMachine::initial(&i), start("design", DragMode::Move), &i).state;
    let s = advance::<GanttMachine>(&s, GanttEvent::DragBy(2), &i).state;
    let preview = s.preview();
    assert_eq!((preview[1].start, preview[1].end), (d("2026-10-31"), d("2026-11-05")));
    // The committed tasks haven't moved yet.
    assert_eq!(range(&s, "design"), (d("2026-10-29"), d("2026-11-03")));
    let step = advance::<GanttMachine>(&s, GanttEvent::Drop, &i);
    assert_eq!(
        step.commands,
        [GanttCommand::Change(GanttChange { id: "design".into(), start: d("2026-10-31"), end: d("2026-11-05") })]
    );
    assert_eq!(range(&step.state, "design"), (d("2026-10-31"), d("2026-11-05")));
    assert!(step.state.drag.is_none());
}

#[test]
fn resizing_keeps_at_least_one_day() {
    let i = input();
    let s = advance::<GanttMachine>(&GanttMachine::initial(&i), start("research", DragMode::ResizeEnd), &i).state;
    let s = advance::<GanttMachine>(&s, GanttEvent::DragBy(-10), &i).state;
    assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-26"), d("2026-10-26"))));
    let s = advance::<GanttMachine>(&s, GanttEvent::DragBy(3), &i).state;
    assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-26"), d("2026-10-31"))));
    let s = advance::<GanttMachine>(&GanttMachine::initial(&i), start("research", DragMode::ResizeStart), &i).state;
    let s = advance::<GanttMachine>(&s, GanttEvent::DragBy(10), &i).state;
    assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-28"), d("2026-10-28"))));
}

#[test]
fn dropping_in_place_or_cancelling_reports_nothing() {
    let i = input();
    let s = advance::<GanttMachine>(&GanttMachine::initial(&i), start("build", DragMode::Move), &i).state;
    assert!(advance::<GanttMachine>(&s, GanttEvent::Drop, &i).commands.is_empty());
    let s = advance::<GanttMachine>(&s, GanttEvent::DragBy(4), &i).state;
    let step = advance::<GanttMachine>(&s, GanttEvent::Cancel, &i);
    assert!(step.commands.is_empty());
    assert_eq!(step.state.preview(), tasks());
}

#[test]
fn keyboard_pick_up_nudge_resize_drop() {
    let i = input();
    let s = GanttMachine::initial(&i);
    // Nudges do nothing until a bar is picked up.
    assert!(advance::<GanttMachine>(&s, GanttEvent::Nudge { mode: DragMode::Move, days: 1 }, &i).commands.is_empty());
    let step = advance::<GanttMachine>(&s, GanttEvent::Grab("research".into()), &i);
    assert!(matches!(&step.commands[..], [GanttCommand::Announce(Announcement::Grabbed { .. })]));
    let s = advance::<GanttMachine>(&step.state, GanttEvent::Nudge { mode: DragMode::Move, days: 1 }, &i).state;
    let s = advance::<GanttMachine>(&s, GanttEvent::Nudge { mode: DragMode::ResizeEnd, days: 2 }, &i).state;
    let s = advance::<GanttMachine>(&s, GanttEvent::Nudge { mode: DragMode::ResizeStart, days: -1 }, &i).state;
    assert_eq!(s.drag.as_ref().map(|x| x.draft), Some((d("2026-10-26"), d("2026-10-31"))));
    // Pointer events are ignored while carrying.
    assert!(advance::<GanttMachine>(&s, GanttEvent::DragBy(9), &i).commands.is_empty());
    let step = advance::<GanttMachine>(&s, GanttEvent::Drop, &i);
    assert_eq!(
        step.commands[0],
        GanttCommand::Change(GanttChange { id: "research".into(), start: d("2026-10-26"), end: d("2026-10-31") })
    );
    assert!(matches!(step.commands[1], GanttCommand::Announce(Announcement::Dropped { .. })));
}

#[test]
fn focus_step_moves_between_bars_and_stops_at_the_ends() {
    let i = input();
    let s = GanttMachine::initial(&i);
    let step = advance::<GanttMachine>(&s, GanttEvent::FocusStep { from: "research".into(), by: 1 }, &i);
    assert_eq!(step.commands, [GanttCommand::FocusBar("design".into())]);
    assert!(
        advance::<GanttMachine>(&s, GanttEvent::FocusStep { from: "research".into(), by: -1 }, &i).commands.is_empty()
    );
    assert!(advance::<GanttMachine>(&s, GanttEvent::FocusStep { from: "build".into(), by: 1 }, &i).commands.is_empty());
}

#[test]
fn scale_can_be_controlled() {
    let i = input();
    let step = advance::<GanttMachine>(&GanttMachine::initial(&i), GanttEvent::SetScale(GanttScale::Week), &i);
    assert_eq!(step.state.scale.value, GanttScale::Week);
    assert_eq!(step.commands, [GanttCommand::ScaleChange(GanttScale::Week)]);
    assert!(advance::<GanttMachine>(&step.state, GanttEvent::SetScale(GanttScale::Week), &i).commands.is_empty());
    let mut c = GanttInput { scale: Some(GanttScale::Day), ..input() };
    let step = advance::<GanttMachine>(&GanttMachine::initial(&c), GanttEvent::SetScale(GanttScale::Week), &c);
    assert_eq!(step.state.scale.value, GanttScale::Day);
    c.scale = Some(GanttScale::Week);
    assert_eq!(hydroxy_statechart::sync_or_keep::<GanttMachine>(&step.state, &c).scale.value, GanttScale::Week);
}

#[test]
fn controlled_requests_but_waits_for_sync() {
    let mut i = GanttInput { tasks: Some(tasks()), ..GanttInput::default() };
    let s = advance::<GanttMachine>(&GanttMachine::initial(&i), start("build", DragMode::Move), &i).state;
    let s = advance::<GanttMachine>(&s, GanttEvent::DragBy(-1), &i).state;
    let step = advance::<GanttMachine>(&s, GanttEvent::Drop, &i);
    let GanttCommand::Change(change) = step.commands[0].clone() else { panic!("expected a change") };
    assert_eq!(range(&step.state, "build"), (d("2026-11-04"), d("2026-11-10")));
    let mut next = tasks();
    apply_change(&mut next, &change);
    i.tasks = Some(next);
    assert_eq!(
        range(&hydroxy_statechart::sync_or_keep::<GanttMachine>(&step.state, &i), "build"),
        (d("2026-11-03"), d("2026-11-09"))
    );
}

#[test]
fn disabled_blocks_moves_but_not_the_scale() {
    let i = GanttInput { disabled: true, ..input() };
    let s = GanttMachine::initial(&i);
    assert!(advance::<GanttMachine>(&s, GanttEvent::Grab("build".into()), &i).state.drag.is_none());
    assert!(advance::<GanttMachine>(&s, start("build", DragMode::Move), &i).state.drag.is_none());
    assert_eq!(advance::<GanttMachine>(&s, GanttEvent::SetScale(GanttScale::Week), &i).commands.len(), 1);
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
    assert_eq!(
        c.iter().map(|c| (c.from.as_str(), c.to.as_str())).collect::<Vec<_>>(),
        [("research", "design"), ("design", "build"), ("research", "build")]
    );
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
    assert_eq!(
        connector_path(wide, Rect { x: 260.0, y: 8.0, width: 10.0, height: 24.0 }).0,
        "M250 60 H260 V40 H250 V20 H260"
    );
}

#[test]
fn text_helpers() {
    assert_eq!(short_date(d("2026-10-05")), "Oct 5");
    assert_eq!(range_text(d("2026-10-26"), d("2026-10-28")), "Oct 26 to Oct 28");
    assert_eq!(range_text(d("2026-10-26"), d("2026-10-26")), "Oct 26");
    assert_eq!(GanttTask::new("x", "X", d("2026-10-05"), d("2026-10-01")).days(), 1);
}
