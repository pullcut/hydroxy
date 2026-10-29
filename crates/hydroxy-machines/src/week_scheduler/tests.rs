use super::*;
use crate::date::d;
use WeekSchedulerEvent as E;
use hydroxy_statechart::step;

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
        let t = step::<WeekSchedulerMachine>(&s, e, input);
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
        vec![E::PointerDown { hit: Hit::Event("review".into()), at: GridPoint { day: 1, minute: 630 } }, E::PointerUp],
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

#[test]
fn events_longer_than_a_day_do_not_panic() {
    let mut i = input();
    i.events.push(WeekEvent { end: 3000, ..WeekEvent::timed("long", "Long", d("2026-11-03"), 600, 700) });
    let (_, cmds) = run(&i, vec![E::Nudge { id: "long".into(), days: 0, minutes: 15 }]);
    assert_eq!(changed(&cmds).unwrap().end, MINUTES_PER_DAY);
    let (_, cmds) = run(
        &i,
        vec![
            E::PointerDown { hit: Hit::Event("long".into()), at: GridPoint { day: 1, minute: 600 } },
            E::PointerMove(GridPoint { day: 1, minute: 700 }),
            E::PointerUp,
        ],
    );
    assert!(changed(&cmds).is_none_or(|e| e.end <= MINUTES_PER_DAY));
}
