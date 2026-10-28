use super::*;
use super::{SpreadsheetGridCommand as C, SpreadsheetGridEvent as E};
use hydroxy_statechart::step;

fn input() -> SpreadsheetGridInput {
    let cells: BTreeMap<Cell, String> = [((0, 0), "10"), ((1, 0), "20"), ((2, 0), "=SUM(A1:A2)"), ((0, 1), "note")]
        .into_iter()
        .map(|(c, v)| (c, v.to_string()))
        .collect();
    SpreadsheetGridInput {
        rows: 10_000,
        cols: 6,
        default_cells: Arc::new(cells),
        col_width: 100.0,
        height: 28.0 * 11.0,
    }
}

fn start() -> (SpreadsheetGridState, SpreadsheetGridInput) {
    let i = input();
    (SpreadsheetGridMachine::initial(&i), i)
}

#[test]
fn formulas_are_calculated_from_the_start() {
    let (s, _) = start();
    assert_eq!(s.display((2, 0)), "30");
    assert_eq!(s.raw((2, 0)), "=SUM(A1:A2)");
    assert_eq!(s.value((0, 1)), Value::Text("note".into()));
}

#[test]
fn arrows_move_and_shift_arrows_extend() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Right, extend: false }, &i).state;
    assert_eq!((s.active, s.extent), ((0, 1), (0, 1)));
    let s = step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Down, extend: true }, &i).state;
    let s = step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Right, extend: true }, &i).state;
    assert_eq!(s.selection(), Rect { top: 0, left: 1, bottom: 1, right: 2 });
    assert_eq!(s.active, (0, 1), "the active cell stays put while extending");
    assert_eq!(s.status, "B1:C2 selected, 4 cells");
    let s = step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Left, extend: false }, &i).state;
    assert_eq!((s.active, s.extent), ((0, 0), (0, 0)), "a plain arrow collapses");
    assert!(step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Up, extend: false }, &i).commands.is_empty());
    assert_eq!(
        step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::SheetEnd, extend: false }, &i).state.active,
        (9_999, 5)
    );
    assert_eq!(
        step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::PageDown, extend: false }, &i).state.active,
        (10, 0)
    );
}

#[test]
fn moving_off_screen_scrolls_and_windows_the_rows() {
    let (s, i) = start();
    assert_eq!(s.visible_rows(&i), 0..17);
    let step = hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Bottom, extend: false }, &i);
    assert_eq!(step.commands, [C::ScrollTo { top: 28.0 * 9_990.0, left: 0.0 }]);
    assert_eq!(step.state.visible_rows(&i), 9_984..10_000);
    let back =
        hydroxy_statechart::step::<SpreadsheetGridMachine>(&step.state, E::Move { nav: Nav::Top, extend: false }, &i);
    assert_eq!(back.commands, [C::ScrollTo { top: 0.0, left: 0.0 }]);
    let scrolled = hydroxy_statechart::step::<SpreadsheetGridMachine>(
        &s,
        E::Scroll { top: 2800.0, left: 0.0, width: 400.0, height: 308.0 },
        &i,
    )
    .state;
    assert_eq!(scrolled.visible_rows(&i), 94..117);
    // Column 5 sits past a 400px viewport: scroll right just enough.
    let right =
        hydroxy_statechart::step::<SpreadsheetGridMachine>(&scrolled, E::Move { nav: Nav::RowEnd, extend: false }, &i);
    assert_eq!(right.commands, [C::ScrollTo { top: 0.0, left: 252.0 }]);
}

#[test]
fn pointer_click_shift_click_and_drag_select() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (3, 1), shift: false }, &i).state;
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerEnter((5, 2)), &i).state;
    assert_eq!(s.selection(), Rect { top: 3, left: 1, bottom: 5, right: 2 });
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerUp, &i).state;
    assert_eq!(
        step::<SpreadsheetGridMachine>(&s, E::PointerEnter((9, 4)), &i).state.extent,
        (5, 2),
        "no drag after release"
    );
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (0, 0), shift: true }, &i).state;
    assert_eq!(s.selection(), Rect { top: 0, left: 0, bottom: 3, right: 1 });
    assert_eq!(s.active, (3, 1));
}

#[test]
fn typing_replaces_and_enter_saves_then_moves_down() {
    let (s, i) = start();
    let s =
        hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Right, extend: false }, &i).state;
    let step = hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::StartEdit(Some("=".into())), &i);
    assert_eq!(step.commands, [C::FocusEditor]);
    let s = hydroxy_statechart::step::<SpreadsheetGridMachine>(&step.state, E::EditChange("=A3*2".into()), &i).state;
    assert!(
        hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Down, extend: false }, &i)
            .commands
            .is_empty(),
        "keys belong to the editor"
    );
    let done = hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::Commit(After::Down), &i);
    assert_eq!(done.state.display((0, 1)), "60");
    assert_eq!(done.state.active, (1, 1));
    assert_eq!(done.state.status, "B1 = 60");
    assert_eq!(
        done.commands,
        [C::Change(vec![CellChange { row: 0, col: 1, previous: "note".into(), value: "=A3*2".into() }]), C::FocusGrid]
    );
}

#[test]
fn f2_keeps_content_tab_moves_right_escape_discards() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(&s, E::StartEdit(None), &i).state;
    assert_eq!(s.editing.as_deref(), Some("10"));
    let s = step::<SpreadsheetGridMachine>(&s, E::EditChange("15".into()), &i).state;
    let tabbed = step::<SpreadsheetGridMachine>(&s, E::Commit(After::Right), &i).state;
    assert_eq!((tabbed.active, tabbed.display((2, 0))), ((0, 1), "35".to_string()), "dependents recalculate");
    let dropped = step::<SpreadsheetGridMachine>(&s, E::Cancel, &i);
    assert_eq!(dropped.state.raw((0, 0)), "10");
    assert_eq!(dropped.commands, [C::FocusGrid]);
}

#[test]
fn clicking_away_saves_and_blur_does_not_steal_focus() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(
        &step::<SpreadsheetGridMachine>(&s, E::StartEdit(Some("7".into())), &i).state,
        E::EditChange("7".into()),
        &i,
    )
    .state;
    let clicked = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (4, 4), shift: false }, &i);
    assert_eq!(clicked.state.raw((0, 0)), "7");
    assert_eq!(clicked.state.active, (4, 4));
    assert!(matches!(clicked.commands.as_slice(), [C::Change(_)]));
    let blurred = step::<SpreadsheetGridMachine>(&s, E::Blur, &i);
    assert!(matches!(blurred.commands.as_slice(), [C::Change(_)]));
    assert!(step::<SpreadsheetGridMachine>(&blurred.state, E::Blur, &i).commands.is_empty());
}

#[test]
fn double_click_edits_that_cell() {
    let (s, i) = start();
    let step = hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::EditAt((2, 0)), &i);
    assert_eq!(step.state.active, (2, 0));
    assert_eq!(step.state.editing.as_deref(), Some("=SUM(A1:A2)"));
}

#[test]
fn clear_empties_the_selection_and_reports_it() {
    let (s, i) = start();
    let s = hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Down, extend: true }, &i).state;
    let step = hydroxy_statechart::step::<SpreadsheetGridMachine>(&s, E::Clear, &i);
    assert_eq!(step.state.display((2, 0)), "0");
    assert_eq!(step.state.status, "Cleared A1:A2");
    let C::Change(changes) = &step.commands[0] else { panic!("expected a change") };
    assert_eq!(changes.len(), 2);
    assert!(
        hydroxy_statechart::step::<SpreadsheetGridMachine>(&step.state, E::Clear, &i).commands.is_empty(),
        "nothing left to clear"
    );
}

#[test]
fn copy_and_paste_round_trip_as_tsv() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(
        &step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::Down, extend: true }, &i).state,
        E::Move { nav: Nav::Right, extend: true },
        &i,
    )
    .state;
    assert_eq!(s.selection_tsv(), "10\tnote\n20\t");
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (5, 2), shift: false }, &i).state;
    let pasted = step::<SpreadsheetGridMachine>(&s, E::Paste("1\t2\n3\t=C6+D6+C7\n".into()), &i);
    assert_eq!(pasted.state.display((6, 3)), "6");
    assert_eq!(pasted.state.selection(), Rect { top: 5, left: 2, bottom: 6, right: 3 });
    assert_eq!(pasted.state.status, "Pasted into C6:D7");
    let C::Change(changes) = &pasted.commands[0] else { panic!("expected a change") };
    assert_eq!(changes.len(), 4);
}

#[test]
fn one_value_fills_a_range_and_pastes_clip_at_the_edge() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (0, 3), shift: false }, &i).state;
    let s = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (2, 4), shift: true }, &i).state;
    let filled = step::<SpreadsheetGridMachine>(&s, E::Paste("x".into()), &i).state;
    assert!((0..3).all(|r| (3..5).all(|c| filled.raw((r, c)) == "x")));
    let edge = step::<SpreadsheetGridMachine>(&s, E::PointerDown { cell: (0, 5), shift: false }, &i).state;
    let clipped = step::<SpreadsheetGridMachine>(&edge, E::Paste("a\tb\tc".into()), &i).state;
    assert_eq!(clipped.raw((0, 5)), "a");
    assert_eq!(clipped.selection(), Rect { top: 0, left: 5, bottom: 0, right: 5 });
}

#[test]
fn a1_names_seed_cells() {
    let cells = cells_from_a1(&[("A1", "Rent"), ("c2", "=B2*2"), ("nope", "x")]);
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[&(1, 2)], "=B2*2");
}

#[test]
fn tsv_handles_quotes_tabs_and_newlines() {
    let rows = vec![vec!["a\tb".to_string(), "say \"hi\"".to_string()], vec!["line\nbreak".to_string(), String::new()]];
    let text = to_tsv(&rows);
    assert_eq!(parse_tsv(&text), rows);
    assert_eq!(parse_tsv("1\t2\r\n3\t4\r\n"), [["1", "2"], ["3", "4"]]);
    assert!(parse_tsv("").is_empty());
}

#[test]
fn select_all_and_escape() {
    let (s, i) = start();
    let all = step::<SpreadsheetGridMachine>(&s, E::SelectAll, &i).state;
    assert_eq!(all.selection().count(), 60_000);
    assert_eq!(all.status, "All 60000 cells selected");
    assert_eq!(step::<SpreadsheetGridMachine>(&all, E::Collapse, &i).state.selection().count(), 1);
}

#[test]
fn columns_resize_by_drag_and_keyboard_within_limits() {
    let (s, i) = start();
    let s = step::<SpreadsheetGridMachine>(&s, E::ResizeStart { col: 1, x: 300.0 }, &i).state;
    let s = step::<SpreadsheetGridMachine>(&s, E::ResizeMove(350.0), &i).state;
    assert!((s.widths[1] - 150.0).abs() < f64::EPSILON);
    let s = step::<SpreadsheetGridMachine>(&s, E::ResizeMove(0.0), &i).state;
    assert!((s.widths[1] - MIN_COL_WIDTH).abs() < f64::EPSILON);
    let s = step::<SpreadsheetGridMachine>(&s, E::ResizeEnd, &i).state;
    assert!(step::<SpreadsheetGridMachine>(&s, E::ResizeMove(500.0), &i).state.resizing.is_none());
    let s = step::<SpreadsheetGridMachine>(&s, E::ResizeBy { col: 0, delta: 16.0 }, &i).state;
    assert!((s.widths[0] - 116.0).abs() < f64::EPSILON);
    assert!((s.column_left(1) - (ROW_HEADER_WIDTH + 116.0)).abs() < f64::EPSILON);
}

#[test]
fn sync_clamps_to_a_smaller_sheet() {
    let (s, mut i) = start();
    let s = step::<SpreadsheetGridMachine>(&s, E::Move { nav: Nav::SheetEnd, extend: false }, &i).state;
    i.rows = 2;
    i.cols = 2;
    let synced = hydroxy_statechart::sync_or_keep::<SpreadsheetGridMachine>(&s, &i);
    assert_eq!(synced.active, (1, 1));
    assert_eq!(synced.widths.len(), 2);
    assert_eq!(synced.display((2, 0)), "", "rows past the end are dropped");
    assert_eq!(SpreadsheetGridMachine::sync(&synced, &i), None);
}
