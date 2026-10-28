use super::*;
use MentionInputCommand as C;
use MentionInputEvent as E;
use hydroxy_statechart::step;

fn run(
    s: &MentionInputState,
    e: MentionInputEvent,
) -> hydroxy_statechart::Step<MentionInputState, MentionInputCommand> {
    step::<MentionInputMachine>(s, e, &MentionInputInput::default())
}

fn typed(s: &MentionInputState, text: &str) -> MentionInputState {
    run(s, E::SetText { text: text.into(), caret: text.chars().count() }).state
}

fn pick(s: &MentionInputState, id: &str, label: &str) -> MentionInputState {
    run(s, E::Accept { id: id.into(), label: label.into() }).state
}

/// "Ask @Ana Lima about it" with the mention picked through the list.
fn with_mention() -> MentionInputState {
    let s = typed(&MentionInputState::default(), "Ask @an");
    let s = pick(&s, "u1", "Ana Lima");
    typed(&s, "Ask @Ana Lima about it")
}

#[test]
fn triggers_open_at_word_starts_only() {
    let t = MentionInputInput::default().triggers;
    assert_eq!(detect("hi @al", 6, &t, &[]), Some(Trigger { char: '@', start: 3, query: "al".into() }));
    assert_eq!(detect("mail a@b", 8, &t, &[]), None);
    assert_eq!(detect("hi @al x", 8, &t, &[]), None);
    assert_eq!(detect("run /dep", 8, &t, &[]).map(|t| t.char), Some('/'));
}

#[test]
fn start_only_and_custom_triggers() {
    let t = [TriggerSpec::start_only('/'), TriggerSpec::new('#')];
    assert_eq!(detect("a /x", 4, &t, &[]), None);
    assert_eq!(detect("/x", 2, &t, &[]).map(|t| t.query), Some("x".into()));
    assert_eq!(detect("see #ops", 8, &t, &[]).map(|t| t.char), Some('#'));
    assert_eq!(detect("hi @al", 6, &t, &[]), None);
}

#[test]
fn accept_inserts_an_atomic_mention_and_reports_it() {
    let s = typed(&MentionInputState::default(), "Ask @an");
    assert!(s.is_open());
    let step = run(&s, E::Accept { id: "u1".into(), label: "Ana Lima".into() });
    assert_eq!(step.state.text, "Ask @Ana Lima ");
    assert_eq!(step.state.caret, 14);
    assert!(!step.state.is_open());
    let m = &step.state.mentions[0];
    assert_eq!((m.id.as_str(), m.start, m.end), ("u1", 4, 13));
    assert_eq!(step.commands[0], C::Changed(step.state.value()));
    assert_eq!(step.commands[1], C::PlaceCaret { start: 14, end: 14 });
}

#[test]
fn accept_before_existing_text_shifts_later_mentions_and_reuses_a_space() {
    let s = with_mention();
    // Put the caret at the start, type "@b" and pick Bo.
    let s = run(&s, E::Select { start: 0, end: 0 }).state;
    let s = run(&s, E::SetText { text: "@bAsk @Ana Lima about it".into(), caret: 2 }).state;
    assert!(s.is_open());
    let s = pick(&s, "u2", "Bo");
    assert_eq!(s.text, "@Bo Ask @Ana Lima about it");
    assert_eq!((s.mentions[0].start, s.mentions[0].end), (0, 3));
    assert_eq!((s.mentions[1].start, s.mentions[1].end), (8, 17));
    assert_eq!(s.text.chars().skip(8).take(9).collect::<String>(), s.mentions[1].text());
    let s = typed(&MentionInputState::default(), "hi @b there");
    let s = run(&s, E::Select { start: 5, end: 5 }).state;
    assert_eq!(pick(&s, "u2", "Bo").text, "hi @Bo there");
}

#[test]
fn backspace_at_the_end_removes_the_whole_mention() {
    let s = with_mention();
    // Caret after "@Ana Lima" (index 13); the browser deletes the "a".
    let step = run(&s, E::SetText { text: "Ask @Ana Lim about it".into(), caret: 12 });
    assert_eq!(step.state.text, "Ask  about it");
    assert!(step.state.mentions.is_empty());
    assert_eq!(step.state.caret, 4);
    assert!(step.commands.contains(&C::PlaceCaret { start: 4, end: 4 }));
}

#[test]
fn delete_at_the_start_removes_the_whole_mention() {
    let s = with_mention();
    let step = run(&s, E::SetText { text: "Ask Ana Lima about it".into(), caret: 4 });
    assert_eq!(step.state.text, "Ask  about it");
}

#[test]
fn typing_inside_replaces_the_mention() {
    let s = with_mention();
    let step = run(&s, E::SetText { text: "Ask @Anxa Lima about it".into(), caret: 8 });
    assert_eq!(step.state.text, "Ask x about it");
    assert_eq!(step.state.caret, 5);
}

#[test]
fn edits_elsewhere_keep_and_shift_mentions() {
    let s = with_mention();
    let s = run(&s, E::Select { start: 0, end: 0 }).state;
    let step = run(&s, E::SetText { text: "Please ask @Ana Lima about it".into(), caret: 7 });
    let m = &step.state.mentions[0];
    assert_eq!((m.start, m.end), (11, 20));
    assert!(!step.commands.iter().any(|c| matches!(c, C::PlaceCaret { .. })));
    // Typing straight after the mention keeps it.
    let s = typed(&MentionInputState::default(), "@a");
    let s = pick(&s, "u1", "Ana");
    let s = run(&s, E::Select { start: 4, end: 4 }).state;
    let s = run(&s, E::SetText { text: "@Ana!".into(), caret: 5 }).state;
    assert_eq!(s.mentions.len(), 1, "replacing the space after it keeps the mention");
}

#[test]
fn repeated_characters_use_the_caret() {
    let old: Vec<char> = "aa".chars().collect();
    let new: Vec<char> = "a".chars().collect();
    assert_eq!(edit_span(&old, &new, 1), (1, 2, 1));
    assert_eq!(edit_span(&old, &new, 0), (0, 1, 0));
}

#[test]
fn a_selection_over_part_of_a_mention_widens_to_all_of_it() {
    let s = with_mention();
    let step = run(&s, E::Select { start: 2, end: 6 });
    assert_eq!(step.commands, vec![C::PlaceCaret { start: 2, end: 13 }]);
    // Cutting "k @An" removes the whole mention.
    let step = run(&s, E::SetText { text: "Asa Lima about it".into(), caret: 2 });
    assert_eq!(step.state.text, "As about it");
}

#[test]
fn the_caret_skips_over_mentions() {
    let s = with_mention();
    let s = run(&s, E::Select { start: 4, end: 4 }).state;
    // ArrowRight from 4 lands on 5, inside: carry on to the end.
    assert_eq!(run(&s, E::Select { start: 5, end: 5 }).commands, vec![C::PlaceCaret { start: 13, end: 13 }]);
    let s = run(&s, E::Select { start: 13, end: 13 }).state;
    // ArrowLeft from 13 lands on 12: carry on to the start.
    assert_eq!(run(&s, E::Select { start: 12, end: 12 }).commands, vec![C::PlaceCaret { start: 4, end: 4 }]);
    // A click near the end goes to the end.
    let s = run(&s, E::Select { start: 0, end: 0 }).state;
    assert_eq!(run(&s, E::Select { start: 11, end: 11 }).state.caret, 13);
}

#[test]
fn no_list_right_after_a_mention() {
    let s = with_mention();
    let s = run(&s, E::Select { start: 13, end: 13 }).state;
    assert!(!s.is_open());
}

#[test]
fn escape_keeps_the_list_closed_until_the_trigger_is_left() {
    let s = typed(&MentionInputState::default(), "hi @a");
    let s = run(&s, E::Dismiss).state;
    assert!(!s.is_open());
    let s = typed(&s, "hi @al");
    assert!(!s.is_open());
    let s = typed(&s, "hi @al ");
    let s = typed(&s, "hi @al @");
    assert!(s.is_open());
}

#[test]
fn active_row_wraps_and_resets_for_a_new_trigger() {
    let s = typed(&MentionInputState::default(), "@");
    let s = run(&s, E::MoveActive { delta: -1, count: 3 }).state;
    assert_eq!(s.active, 2);
    let s = typed(&s, "@a");
    assert_eq!(s.active, 2, "same trigger keeps the row");
    let s = typed(&s, "@a @");
    assert_eq!(s.active, 0);
}

#[test]
fn from_value_drops_bad_mentions() {
    let good = Mention { id: "u1".into(), label: "Ana".into(), trigger: '@', start: 3, end: 7 };
    let wrong = Mention { id: "u2".into(), label: "Bo".into(), trigger: '@', start: 0, end: 3 };
    let s = MentionInputState::from_value(MentionValue { text: "hi @Ana".into(), mentions: vec![good.clone(), wrong] });
    assert_eq!(s.mentions, vec![good]);
    assert_eq!(s.caret, 7);
}

#[test]
fn filter_ranks_prefix_then_word_then_contains() {
    let items = vec![
        MentionItem::new("1", "Dana Ortiz"),
        MentionItem::new("2", "Ana Lima"),
        MentionItem::new("3", "Joan Park").description("Design"),
        MentionItem::new("4", "Bo Chen").description("Analytics"),
    ];
    let ids: Vec<&str> = filter(&items, "an").iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["2", "1", "3", "4"]);
    assert_eq!(filter(&items, "").len(), 4);
    assert_eq!(filter(&items, "zz").len(), 0);
    let many: Vec<MentionItem> = (0..500).map(|i| MentionItem::new(i.to_string(), format!("Person {i}"))).collect();
    assert_eq!(filter(&many, "person").len(), MAX_ROWS);
}

#[test]
fn no_op_events_report_no_state() {
    let s = MentionInputState::default();
    assert_eq!(MentionInputMachine::transition(&s, E::Dismiss, &MentionInputInput::default()).state, None);
    let t = MentionInputMachine::transition(
        &s,
        E::Accept { id: "x".into(), label: "x".into() },
        &MentionInputInput::default(),
    );
    assert_eq!(t, Transition::stay());
}
