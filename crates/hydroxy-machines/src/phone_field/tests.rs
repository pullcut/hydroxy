use super::*;
use crate::listbox::Direction;
use PhoneFieldCommand as C;
use PhoneFieldEvent as E;
use hydroxy_statechart::{Machine, step};

fn start(iso: &str) -> (PhoneFieldState, PhoneFieldInput) {
    let input = PhoneFieldInput { default_country: iso.into(), ..PhoneFieldInput::default() };
    (PhoneFieldMachine::initial(&input), input)
}

fn typed(
    s: &PhoneFieldState,
    text: &str,
    input: &PhoneFieldInput,
) -> hydroxy_statechart::Step<PhoneFieldState, PhoneFieldCommand> {
    step::<PhoneFieldMachine>(s, E::SetText { text: text.into(), caret: text.chars().count() }, input)
}

#[test]
fn table_is_sane() {
    assert!(COUNTRIES.len() >= 60);
    assert!(COUNTRIES.windows(2).all(|w| w[0].name < w[1].name), "sorted by name");
    for c in COUNTRIES {
        let n = c.example.len();
        assert!(n >= usize::from(c.min) && n <= usize::from(c.max), "{} example length", c.iso);
        assert!(c.is_valid(c.example), "{} example validates", c.iso);
    }
}

#[test]
fn flags_come_from_regional_indicators() {
    assert_eq!(flag("gb"), "🇬🇧");
    assert_eq!(flag("US"), "🇺🇸");
}

#[test]
fn groups_as_you_type_without_dangling_literals() {
    let (s, input) = start("US");
    assert_eq!(typed(&s, "201", &input).state.text, "(201");
    assert_eq!(typed(&s, "2015", &input).state.text, "(201) 5");
    let t = typed(&s, "2015550123", &input);
    assert_eq!(t.state.text, "(201) 555-0123");
    assert_eq!(t.commands, vec![C::ValueChange { e164: Some("+12015550123".into()), valid: true }]);
    // Past the plan's length, extra digits are dropped.
    assert_eq!(typed(&s, "20155501239", &input).state.national, "2015550123");
}

#[test]
fn trunk_zero_shows_but_is_dropped_from_e164() {
    let (s, input) = start("GB");
    let t = typed(&s, "07400123456", &input);
    assert_eq!(t.state.text, "07400 123456");
    assert_eq!(t.state.e164().as_deref(), Some("+447400123456"));
    assert!(t.state.is_valid());
}

#[test]
fn plus_switches_country_from_the_dial_code() {
    let (s, input) = start("US");
    let t = typed(&s, "+4", &input);
    assert_eq!((t.state.text.as_str(), t.state.country.as_str()), ("+4", "US"));
    let t = typed(&s, "+33 6 12 34 56 78", &input);
    assert_eq!(t.state.country, "FR");
    assert_eq!(t.state.text, "6 12 34 56 78");
    assert_eq!(t.commands[0], C::CountryChange("FR".into()));
    // +1 stays on Canada when Canada is chosen.
    let (s, input) = start("CA");
    assert_eq!(typed(&s, "+16135550123", &input).state.country, "CA");
    assert_eq!(split_international("12015550123", "GB").map(|(c, _)| c.iso), Some("US"));
    assert_eq!(split_international("97336001234", "US").map(|(c, _)| c.iso), Some("BH"));
}

#[test]
fn backspace_over_a_literal_removes_the_digit_before() {
    let (s, input) = start("US");
    let s = typed(&s, "2015", &input).state;
    assert_eq!(s.text, "(201) 5");
    // Backspace after "5" then over the space: "(201)5" with the caret at 5.
    let t = step::<PhoneFieldMachine>(&s, E::SetText { text: "(201)5".into(), caret: 5 }, &input);
    assert_eq!(t.state.national, "205");
}

#[test]
fn caret_stays_on_its_digit_mid_edit() {
    let (s, input) = start("US");
    let s = typed(&s, "2015550123", &input).state;
    // Insert "9" after "(20": "(209 1) 555-0123"-ish raw text, caret after the 9.
    let t = step::<PhoneFieldMachine>(&s, E::SetText { text: "(2091) 555-0123".into(), caret: 4 }, &input);
    assert_eq!(t.state.text, "(209) 155-5012");
    assert_eq!(t.state.caret, 4);
}

#[test]
fn validity_shows_only_after_blur() {
    let (s, input) = start("FR");
    let s = typed(&s, "0612", &input).state;
    assert!(!s.invalid());
    let s = step::<PhoneFieldMachine>(&s, E::Blur, &input).state;
    assert!(s.invalid());
}

#[test]
fn picker_searches_moves_and_picks() {
    let (s, input) = start("US");
    let s = step::<PhoneFieldMachine>(&s, E::OpenPicker, &input).state;
    assert_eq!(s.highlighted.as_deref(), Some("US"));
    let s = step::<PhoneFieldMachine>(&s, E::Query("+44".into()), &input).state;
    assert_eq!(s.highlighted.as_deref(), Some("GB"));
    let s = step::<PhoneFieldMachine>(&s, E::Query("land".into()), &input).state;
    assert_eq!(visible("land").first().map(|c| c.iso), Some("FI"));
    let s = step::<PhoneFieldMachine>(&s, E::Move(Direction::Next), &input).state;
    assert_eq!(s.highlighted.as_deref(), Some("IS"));
    let t = step::<PhoneFieldMachine>(&s, E::Enter, &input);
    assert!(!t.state.picker_open);
    assert_eq!(t.commands, vec![C::CountryChange("IS".into()), C::FocusNumber]);
}

#[test]
fn picking_a_country_regroups_and_trims_the_number() {
    let (s, input) = start("US");
    let s = typed(&s, "2015550123", &input).state;
    let t = step::<PhoneFieldMachine>(&s, E::Select("NO".into()), &input);
    assert_eq!(t.state.text, "201 55 501");
    assert_eq!(t.state.e164().as_deref(), Some("+4720155501"));
}

#[test]
fn escape_restores_focus_outside_click_does_not() {
    let (s, input) = start("US");
    let s = step::<PhoneFieldMachine>(&s, E::OpenPicker, &input).state;
    let t = step::<PhoneFieldMachine>(&s, E::ClosePicker { restore_focus: true }, &input);
    assert_eq!(t.commands, vec![C::FocusTrigger]);
    assert!(step::<PhoneFieldMachine>(&s, E::ClosePicker { restore_focus: false }, &input).commands.is_empty());
}

#[test]
fn controlled_value_syncs() {
    let mut input = PhoneFieldInput { value: Some("+447400123456".into()), ..PhoneFieldInput::default() };
    let s = PhoneFieldMachine::initial(&input);
    assert_eq!((s.country.as_str(), s.text.as_str()), ("GB", "7400 123456"));
    input.value = Some("+81312345678".into());
    let s = hydroxy_statechart::sync_or_keep::<PhoneFieldMachine>(&s, &input);
    assert_eq!(s.country, "JP");
    assert_eq!(s.e164().as_deref(), Some("+81312345678"));
}

#[test]
fn read_only_ignores_input() {
    let input = PhoneFieldInput { read_only: true, ..PhoneFieldInput::default() };
    let s = PhoneFieldMachine::initial(&input);
    assert_eq!(typed(&s, "5", &input).state, s);
}
