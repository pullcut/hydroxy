//! Phone field: a country picker and a national number that groups as you type,
//! reported as E.164 (`+447400123456`). Typing or pasting a number that starts
//! with `+` switches to the country its dial code belongs to. Validity is a
//! length check against the country's plan, shown once the field is left.
//!
//! The picker is a searchable list (name, ISO code or dial code) with a virtual
//! highlight, like a combobox.

use hydroxy_statechart::{Machine, Transition};

use crate::as_you_type as ayt;
use crate::listbox::{self, Direction};

/// A country's dialling plan, simplified to what grouping and validity need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Country {
    /// ISO 3166-1 alpha-2 code, such as `GB`.
    pub iso: &'static str,
    /// English name.
    pub name: &'static str,
    /// Dial code without the `+`, such as `44`.
    pub dial: &'static str,
    /// Grouping for the national significant number: `#` is a digit, anything else a literal.
    pub pattern: &'static str,
    /// An example national significant number, for the placeholder.
    pub example: &'static str,
    /// Fewest digits in a valid national significant number.
    pub min: u8,
    /// Most digits in a valid national significant number.
    pub max: u8,
    /// A leading `0` is a domestic trunk prefix, typed at home and dropped in E.164.
    pub trunk: bool,
}

#[allow(clippy::too_many_arguments, reason = "one row of the table")]
const fn c(
    iso: &'static str,
    name: &'static str,
    dial: &'static str,
    pattern: &'static str,
    example: &'static str,
    min: u8,
    max: u8,
    trunk: bool,
) -> Country {
    Country { iso, name, dial, pattern, example, min, max, trunk }
}

// length ranges and one grouping per country, not libphonenumber's full
// per-prefix metadata. Mobile-first examples; fixed lines of another length still
// validate but may group a little differently from local habit.
/// The countries the picker offers, sorted by name.
pub const COUNTRIES: &[Country] = &[
    c("AR", "Argentina", "54", "## ####-####", "1123456789", 10, 10, true),
    c("AU", "Australia", "61", "### ### ###", "412345678", 9, 9, true),
    c("AT", "Austria", "43", "### #######", "6641234567", 4, 13, true),
    c("BH", "Bahrain", "973", "#### ####", "36001234", 8, 8, false),
    c("BD", "Bangladesh", "880", "####-######", "1812345678", 10, 10, true),
    c("BE", "Belgium", "32", "### ## ## ##", "470123456", 8, 9, true),
    c("BR", "Brazil", "55", "(##) #####-####", "11961234567", 10, 11, false),
    c("BG", "Bulgaria", "359", "## ### ####", "48123456", 8, 9, true),
    c("CA", "Canada", "1", "(###) ###-####", "6135550123", 10, 10, false),
    c("CL", "Chile", "56", "# #### ####", "221234567", 9, 9, false),
    c("CN", "China", "86", "### #### ####", "13123456789", 11, 11, false),
    c("CO", "Colombia", "57", "### #######", "3211234567", 10, 10, false),
    c("CR", "Costa Rica", "506", "#### ####", "83123456", 8, 8, false),
    c("HR", "Croatia", "385", "## ### ####", "921234567", 8, 9, true),
    c("CZ", "Czechia", "420", "### ### ###", "601123456", 9, 9, false),
    c("DK", "Denmark", "45", "## ## ## ##", "32123456", 8, 8, false),
    c("EG", "Egypt", "20", "### ### ####", "1001234567", 10, 10, true),
    c("FI", "Finland", "358", "## ### ####", "412345678", 5, 12, true),
    c("FR", "France", "33", "# ## ## ## ##", "612345678", 9, 9, true),
    c("DE", "Germany", "49", "### ########", "15123456789", 6, 11, true),
    c("GH", "Ghana", "233", "## ### ####", "231234567", 9, 9, true),
    c("GR", "Greece", "30", "### ### ####", "6912345678", 10, 10, false),
    c("HK", "Hong Kong", "852", "#### ####", "51234567", 8, 8, false),
    c("HU", "Hungary", "36", "## ### ####", "201234567", 8, 9, false),
    c("IS", "Iceland", "354", "### ####", "6111234", 7, 7, false),
    c("IN", "India", "91", "#####-#####", "8123456789", 10, 10, true),
    c("ID", "Indonesia", "62", "###-####-####", "812345678", 9, 12, true),
    c("IE", "Ireland", "353", "## ### ####", "851234567", 7, 9, true),
    c("IL", "Israel", "972", "##-###-####", "502345678", 8, 9, true),
    c("IT", "Italy", "39", "### ### ####", "3123456789", 6, 11, false),
    c("JP", "Japan", "81", "##-####-####", "9012345678", 9, 10, true),
    c("KE", "Kenya", "254", "### ######", "712123456", 9, 9, true),
    c("KW", "Kuwait", "965", "#### ####", "50012345", 8, 8, false),
    c("LU", "Luxembourg", "352", "### ### ###", "628123456", 4, 11, false),
    c("MY", "Malaysia", "60", "##-### ####", "123456789", 9, 10, true),
    c("MX", "Mexico", "52", "## #### ####", "2221234567", 10, 10, false),
    c("MA", "Morocco", "212", "###-######", "650123456", 9, 9, true),
    c("NL", "Netherlands", "31", "# ########", "612345678", 9, 9, true),
    c("NZ", "New Zealand", "64", "## ### ####", "211234567", 8, 10, true),
    c("NG", "Nigeria", "234", "### ### ####", "8021234567", 10, 10, true),
    c("NO", "Norway", "47", "### ## ###", "40612345", 8, 8, false),
    c("PK", "Pakistan", "92", "### #######", "3012345678", 10, 10, true),
    c("PE", "Peru", "51", "### ### ###", "912345678", 9, 9, false),
    c("PH", "Philippines", "63", "### ### ####", "9051234567", 10, 10, true),
    c("PL", "Poland", "48", "### ### ###", "512345678", 9, 9, false),
    c("PT", "Portugal", "351", "### ### ###", "912345678", 9, 9, false),
    c("QA", "Qatar", "974", "#### ####", "33123456", 8, 8, false),
    c("RO", "Romania", "40", "### ### ###", "712034567", 9, 9, true),
    c("RU", "Russia", "7", "### ###-##-##", "9123456789", 10, 10, false),
    c("SA", "Saudi Arabia", "966", "## ### ####", "512345678", 9, 9, true),
    c("RS", "Serbia", "381", "## #######", "601234567", 8, 9, true),
    c("SG", "Singapore", "65", "#### ####", "81234567", 8, 8, false),
    c("SK", "Slovakia", "421", "### ### ###", "912123456", 9, 9, true),
    c("SI", "Slovenia", "386", "## ### ###", "31234567", 8, 8, true),
    c("ZA", "South Africa", "27", "## ### ####", "711234567", 9, 9, true),
    c("KR", "South Korea", "82", "##-####-####", "1020000000", 9, 10, true),
    c("ES", "Spain", "34", "### ## ## ##", "612345678", 9, 9, false),
    c("LK", "Sri Lanka", "94", "## ### ####", "712345678", 9, 9, true),
    c("SE", "Sweden", "46", "##-### ## ##", "701234567", 7, 9, true),
    c("CH", "Switzerland", "41", "## ### ## ##", "781234567", 9, 9, true),
    c("TW", "Taiwan", "886", "### ### ###", "912345678", 9, 9, true),
    c("TH", "Thailand", "66", "## ### ####", "812345678", 9, 9, true),
    c("TR", "Turkey", "90", "### ### ####", "5012345678", 10, 10, true),
    c("UA", "Ukraine", "380", "## ### ####", "501234567", 9, 9, true),
    c("AE", "United Arab Emirates", "971", "## ### ####", "501234567", 8, 9, true),
    c("GB", "United Kingdom", "44", "#### ######", "7400123456", 9, 10, true),
    c("US", "United States", "1", "(###) ###-####", "2015550123", 10, 10, false),
    c("UY", "Uruguay", "598", "## ### ###", "94231234", 8, 8, true),
    c("VN", "Vietnam", "84", "## ### ## ##", "912345678", 9, 9, true),
];

/// Look a country up by ISO code, ignoring case.
#[must_use]
pub fn country(iso: &str) -> Option<&'static Country> {
    COUNTRIES.iter().find(|c| c.iso.eq_ignore_ascii_case(iso))
}

fn country_or_first(iso: &str) -> &'static Country {
    country(iso).unwrap_or(&COUNTRIES[0])
}

/// The flag emoji for an ISO code: each letter maps to a regional indicator symbol.
#[must_use]
pub fn flag(iso: &str) -> String {
    iso.chars()
        .filter(char::is_ascii_alphabetic)
        .filter_map(|ch| char::from_u32(0x1F1E6 + u32::from(ch.to_ascii_uppercase()) - u32::from('A')))
        .collect()
}

/// Lay `digits` into `pattern`. Literals only appear once a digit follows them, so
/// partial numbers never end in a dangling separator. Digits past the pattern run on.
#[must_use]
pub fn group(digits: &str, pattern: &str) -> String {
    let mut out = String::new();
    let mut rest = digits.chars().peekable();
    for p in pattern.chars() {
        if rest.peek().is_none() {
            break;
        }
        if p == '#' {
            out.extend(rest.next());
        } else {
            out.push(p);
        }
    }
    out.extend(rest);
    out
}

impl Country {
    /// The number as shown while typing: a typed trunk `0` stays in front.
    #[must_use]
    pub fn display(&self, national: &str) -> String {
        match national.strip_prefix('0').filter(|_| self.trunk) {
            Some(rest) => format!("0{}", group(rest, self.pattern)),
            None => group(national, self.pattern),
        }
    }

    /// The national significant number: digits without the trunk `0`.
    #[must_use]
    pub fn significant<'a>(&self, national: &'a str) -> &'a str {
        if self.trunk { national.strip_prefix('0').unwrap_or(national) } else { national }
    }

    /// The example number laid out, for the placeholder.
    #[must_use]
    pub fn placeholder(&self) -> String {
        group(self.example, self.pattern)
    }

    /// Whether `national` has a plausible length for this country.
    #[must_use]
    pub fn is_valid(&self, national: &str) -> bool {
        let nsn = self.significant(national);
        let len = nsn.len();
        len >= usize::from(self.min) && len <= usize::from(self.max) && !nsn.starts_with('0')
    }

    /// `+` dial code and significant number, or `None` when no digits are typed.
    #[must_use]
    pub fn e164(&self, national: &str) -> Option<String> {
        let nsn = self.significant(national);
        (!nsn.is_empty()).then(|| format!("+{}{nsn}", self.dial))
    }

    /// Most digits the field accepts: the longest number plus a trunk `0`.
    fn max_typed(&self) -> usize {
        usize::from(self.max) + usize::from(self.trunk)
    }

    /// Text the picker search matches against.
    fn search_text(&self) -> String {
        format!("{} {} +{}", self.name, self.iso, self.dial)
    }
}

/// Split the digits after a `+` into a country and its number. Longest dial code
/// first; a code shared by several countries (`+1`) keeps `preferred` when it
/// shares it, else the most populous (the US for `+1`). `None` while the digits
/// don't yet name a dial code.
#[must_use]
pub fn split_international(digits: &str, preferred: &str) -> Option<(&'static Country, String)> {
    (1..=3).rev().filter(|n| digits.len() >= *n).find_map(|n| {
        let (dial, rest) = digits.split_at(n);
        let all: Vec<&'static Country> = COUNTRIES.iter().filter(|c| c.dial == dial).collect();
        let pick = all
            .iter()
            .find(|c| c.iso == preferred)
            .or_else(|| all.iter().find(|c| c.iso == "US"))
            .or_else(|| all.first())
            .copied()?;
        Some((pick, rest.to_string()))
    })
}

/// Parse an E.164 string into a country and its national digits.
#[must_use]
pub fn parse_e164(value: &str, preferred: &str) -> Option<(&'static Country, String)> {
    let digits: String = value.trim().strip_prefix('+')?.chars().filter(char::is_ascii_digit).collect();
    split_international(&digits, preferred)
}

/// The countries listed for a search, in display order.
#[must_use]
pub fn visible(query: &str) -> Vec<&'static Country> {
    COUNTRIES.iter().filter(|c| listbox::matches(&c.search_text(), query)).collect()
}

fn visible_isos(query: &str) -> Vec<&'static str> {
    visible(query).into_iter().map(|c| c.iso).collect()
}

/// Props for [`PhoneFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneFieldInput {
    /// Controlled E.164 value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial E.164 value.
    pub default_value: Option<String>,
    /// ISO code of the country to start in when there is no value.
    pub default_country: String,
    /// Ignore every change.
    pub disabled: bool,
    /// Focusable and selectable, but unchangeable.
    pub read_only: bool,
}

impl Default for PhoneFieldInput {
    fn default() -> Self {
        Self { value: None, default_value: None, default_country: "US".into(), disabled: false, read_only: false }
    }
}

/// What [`PhoneFieldMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneFieldState {
    /// ISO code of the chosen country.
    pub country: String,
    /// Digits typed for the national number, a trunk `0` included.
    pub national: String,
    /// The input's text.
    pub text: String,
    /// Where the caret belongs after the last edit, as a char index.
    pub caret: usize,
    /// The field has been left once, so validity may show.
    pub touched: bool,
    /// The country list is open.
    pub picker_open: bool,
    /// The picker's search text.
    pub query: String,
    /// ISO code of the highlighted country in the picker.
    pub highlighted: Option<String>,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl PhoneFieldState {
    /// The chosen country.
    #[must_use]
    pub fn country(&self) -> &'static Country {
        country_or_first(&self.country)
    }

    /// The number in E.164, or `None` while empty.
    #[must_use]
    pub fn e164(&self) -> Option<String> {
        self.country().e164(&self.national)
    }

    /// Whether the number has a plausible length.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.country().is_valid(&self.national)
    }

    /// Show the field as invalid: left once, not empty, and not plausible.
    #[must_use]
    pub fn invalid(&self) -> bool {
        self.touched && !self.national.is_empty() && !self.is_valid()
    }

    fn load(&mut self, value: Option<&str>, fallback: &str) {
        if let Some((c, national)) = value.and_then(|v| parse_e164(v, fallback)) {
            self.country = c.iso.to_string();
            self.national = national.chars().take(c.max_typed()).collect();
        } else {
            self.country = country_or_first(fallback).iso.to_string();
            self.national.clear();
        }
        self.text = self.country().display(&self.national);
        self.caret = self.text.chars().count();
    }
}

/// Inputs to [`PhoneFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneFieldEvent {
    /// The person typed or pasted in the number. `caret` is a char index.
    SetText {
        /// The input's new text.
        text: String,
        /// Caret after the edit.
        caret: usize,
    },
    /// Focus left the number.
    Blur,
    /// Open the country list on the current country.
    OpenPicker,
    /// Close the list without choosing.
    ClosePicker {
        /// Put focus back on the country button (Escape), not when focus moved on its own.
        restore_focus: bool,
    },
    /// Toggle the list (the country button).
    TogglePicker,
    /// The picker search changed.
    Query(String),
    /// Arrow key in the picker.
    Move(Direction),
    /// Pointer over a country.
    Highlight(String),
    /// Choose a country by ISO code.
    Select(String),
    /// Choose the highlighted country.
    Enter,
}

/// Side effects from [`PhoneFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneFieldCommand {
    /// The number changed: E.164 (`None` when empty) and whether it looks valid.
    ValueChange {
        /// `+` dial code and significant number.
        e164: Option<String>,
        /// Plausible length for the country.
        valid: bool,
    },
    /// The country changed, by ISO code.
    CountryChange(String),
    /// Focus the number input (after picking a country).
    FocusNumber,
    /// Focus the country button (after Escape in the picker).
    FocusTrigger,
}

/// A phone number with a country.
#[derive(Debug, Clone, Copy)]
pub struct PhoneFieldMachine;

type Step = Transition<PhoneFieldState, PhoneFieldCommand>;

fn is_sig(c: char) -> bool {
    c.is_ascii_digit() || c == '+'
}

/// Report what changed between `before` and `next`.
fn diff(before: &PhoneFieldState, next: PhoneFieldState, mut commands: Vec<PhoneFieldCommand>) -> Step {
    if next.country != before.country {
        commands.insert(0, PhoneFieldCommand::CountryChange(next.country.clone()));
    }
    let (e164, valid) = (next.e164(), next.is_valid());
    if e164 != before.e164() || valid != before.is_valid() {
        commands.push(PhoneFieldCommand::ValueChange { e164, valid });
    }
    // Focus commands go last so they run after the value is reported.
    commands.sort_by_key(|c| matches!(c, PhoneFieldCommand::FocusNumber | PhoneFieldCommand::FocusTrigger));
    Transition::with(next, commands)
}

fn choose(state: &PhoneFieldState, iso: &str) -> Step {
    let Some(c) = country(iso) else { return Transition::stay() };
    let mut next = state.clone();
    next.country = c.iso.to_string();
    next.national = state.national.chars().take(c.max_typed()).collect();
    next.text = c.display(&next.national);
    next.caret = next.text.chars().count();
    next.picker_open = false;
    diff(state, next, vec![PhoneFieldCommand::FocusNumber])
}

impl Machine for PhoneFieldMachine {
    type Input = PhoneFieldInput;
    type State = PhoneFieldState;
    type Event = PhoneFieldEvent;
    type Command = PhoneFieldCommand;

    fn initial(input: &PhoneFieldInput) -> PhoneFieldState {
        let mut state = PhoneFieldState { controlled: input.value.is_some(), ..PhoneFieldState::default() };
        state.load(input.value.as_deref().or(input.default_value.as_deref()), &input.default_country);
        state
    }

    fn transition(state: &PhoneFieldState, event: PhoneFieldEvent, input: &PhoneFieldInput) -> Step {
        use PhoneFieldEvent as E;
        if input.disabled || input.read_only {
            return Transition::stay();
        }
        let mut next = state.clone();
        match event {
            E::SetText { mut text, mut caret } => {
                if ayt::deleted_literal(&state.text, &text, is_sig) {
                    (text, caret) = ayt::drop_sig_before(&text, caret, is_sig);
                }
                let digits: String = text.chars().filter(char::is_ascii_digit).collect();
                if text.trim_start().starts_with('+') {
                    // International entry: find the country from the dial code.
                    if let Some((c, rest)) = split_international(&digits, &state.country) {
                        next.country = c.iso.to_string();
                        next.national = rest.chars().take(c.max_typed()).collect();
                        next.text = c.display(&next.national);
                    } else {
                        next.national.clear();
                        next.text = format!("+{digits}");
                    }
                    next.caret = next.text.chars().count();
                } else {
                    let c = state.country();
                    let at_end = caret >= text.chars().count();
                    next.national = digits.chars().take(c.max_typed()).collect();
                    next.text = c.display(&next.national);
                    next.caret = if at_end {
                        next.text.chars().count()
                    } else {
                        ayt::caret_after(&next.text, ayt::sig_before(&text, caret, is_sig), is_sig)
                    };
                }
                diff(state, next, Vec::new())
            }
            E::Blur => {
                next.touched = true;
                Transition::to(next)
            }
            E::OpenPicker => {
                next.picker_open = true;
                next.query.clear();
                next.highlighted = Some(state.country.clone());
                Transition::to(next)
            }
            E::TogglePicker if state.picker_open => {
                next.picker_open = false;
                Transition::to(next)
            }
            E::TogglePicker => Self::transition(state, E::OpenPicker, input),
            E::ClosePicker { restore_focus } => {
                if !state.picker_open {
                    return Transition::to(next);
                }
                next.picker_open = false;
                let commands = if restore_focus { vec![PhoneFieldCommand::FocusTrigger] } else { Vec::new() };
                Transition::with(next, commands)
            }
            E::Query(query) => {
                next.highlighted = listbox::keep(&visible_isos(&query), None);
                next.query = query;
                Transition::to(next)
            }
            E::Move(direction) => {
                next.highlighted = listbox::step(&visible_isos(&state.query), state.highlighted.as_deref(), direction);
                Transition::to(next)
            }
            E::Highlight(iso) => {
                if visible_isos(&state.query).contains(&iso.as_str()) {
                    next.highlighted = Some(iso);
                }
                Transition::to(next)
            }
            E::Select(iso) => choose(state, &iso),
            E::Enter => match state.highlighted.as_deref().filter(|_| state.picker_open) {
                Some(iso) if visible_isos(&state.query).contains(&iso) => choose(state, iso),
                _ => Transition::to(next),
            },
        }
    }

    fn sync(state: &PhoneFieldState, input: &PhoneFieldInput) -> Option<PhoneFieldState> {
        if !state.controlled || input.value == state.e164() {
            return None;
        }
        let mut next = state.clone();
        next.load(input.value.as_deref(), &state.country);
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PhoneFieldCommand as C;
    use PhoneFieldEvent as E;

    fn start(iso: &str) -> (PhoneFieldState, PhoneFieldInput) {
        let input = PhoneFieldInput { default_country: iso.into(), ..PhoneFieldInput::default() };
        (PhoneFieldMachine::initial(&input), input)
    }

    fn typed(s: &PhoneFieldState, text: &str, input: &PhoneFieldInput) -> hydroxy_statechart::Step<PhoneFieldState, PhoneFieldCommand> {
        PhoneFieldMachine::transition(s, E::SetText { text: text.into(), caret: text.chars().count() }, input).resolve(s)
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
        let t = PhoneFieldMachine::transition(&s, E::SetText { text: "(201)5".into(), caret: 5 }, &input).resolve(&s);
        assert_eq!(t.state.national, "205");
    }

    #[test]
    fn caret_stays_on_its_digit_mid_edit() {
        let (s, input) = start("US");
        let s = typed(&s, "2015550123", &input).state;
        // Insert "9" after "(20": "(209 1) 555-0123"-ish raw text, caret after the 9.
        let t = PhoneFieldMachine::transition(&s, E::SetText { text: "(2091) 555-0123".into(), caret: 4 }, &input).resolve(&s);
        assert_eq!(t.state.text, "(209) 155-5012");
        assert_eq!(t.state.caret, 4);
    }

    #[test]
    fn validity_shows_only_after_blur() {
        let (s, input) = start("FR");
        let s = typed(&s, "0612", &input).state;
        assert!(!s.invalid());
        let s = PhoneFieldMachine::transition(&s, E::Blur, &input).resolve(&s).state;
        assert!(s.invalid());
    }

    #[test]
    fn picker_searches_moves_and_picks() {
        let (s, input) = start("US");
        let s = PhoneFieldMachine::transition(&s, E::OpenPicker, &input).resolve(&s).state;
        assert_eq!(s.highlighted.as_deref(), Some("US"));
        let s = PhoneFieldMachine::transition(&s, E::Query("+44".into()), &input).resolve(&s).state;
        assert_eq!(s.highlighted.as_deref(), Some("GB"));
        let s = PhoneFieldMachine::transition(&s, E::Query("land".into()), &input).resolve(&s).state;
        assert_eq!(visible("land").first().map(|c| c.iso), Some("FI"));
        let s = PhoneFieldMachine::transition(&s, E::Move(Direction::Next), &input).resolve(&s).state;
        assert_eq!(s.highlighted.as_deref(), Some("IS"));
        let t = PhoneFieldMachine::transition(&s, E::Enter, &input).resolve(&s);
        assert!(!t.state.picker_open);
        assert_eq!(t.commands, vec![C::CountryChange("IS".into()), C::FocusNumber]);
    }

    #[test]
    fn picking_a_country_regroups_and_trims_the_number() {
        let (s, input) = start("US");
        let s = typed(&s, "2015550123", &input).state;
        let t = PhoneFieldMachine::transition(&s, E::Select("NO".into()), &input).resolve(&s);
        assert_eq!(t.state.text, "201 55 501");
        assert_eq!(t.state.e164().as_deref(), Some("+4720155501"));
    }

    #[test]
    fn escape_restores_focus_outside_click_does_not() {
        let (s, input) = start("US");
        let s = PhoneFieldMachine::transition(&s, E::OpenPicker, &input).resolve(&s).state;
        let t = PhoneFieldMachine::transition(&s, E::ClosePicker { restore_focus: true }, &input).resolve(&s);
        assert_eq!(t.commands, vec![C::FocusTrigger]);
        assert!(PhoneFieldMachine::transition(&s, E::ClosePicker { restore_focus: false }, &input).resolve(&s).commands.is_empty());
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
}
