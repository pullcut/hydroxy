//! `PasswordField` owns the password text, whether it is shown, and whether Caps Lock is on.
//! [`estimate`] scores a password in the spirit of zxcvbn: it splits the text into the
//! cheapest guessable pieces (common passwords, repeats, sequences, keyboard runs, the
//! person's own name or email) and adds up how many guesses each piece costs. [`Rule`]s are
//! the checklist shown under a new password.

use hydroxy_statechart::{Machine, Transition};

/// Whether the field takes a new password (meter and rules shown) or an existing one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PasswordMode {
    /// Choosing a password: `autocomplete="new-password"`, meter and checklist.
    #[default]
    New,
    /// Signing in: `autocomplete="current-password"`, no meter.
    Current,
}

impl PasswordMode {
    /// The `autocomplete` attribute for the input.
    #[must_use]
    pub const fn autocomplete(self) -> &'static str {
        match self {
            PasswordMode::New => "new-password",
            PasswordMode::Current => "current-password",
        }
    }
}

/// What a [`Rule`] checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    /// At least this many characters.
    MinLength(usize),
    /// At most this many characters.
    MaxLength(usize),
    /// A lowercase letter.
    Lowercase,
    /// An uppercase letter.
    Uppercase,
    /// A digit.
    Digit,
    /// Something that is not a letter, digit or space.
    Symbol,
    /// Not in the common-password list, ignoring case and look-alike digits.
    NotCommon,
    /// [`estimate`] scores at least this (0 to 4).
    MinScore(u8),
}

/// One line of the checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// What it checks.
    pub kind: RuleKind,
    /// What the checklist says.
    pub label: String,
}

impl Rule {
    /// A rule with its default wording.
    #[must_use]
    pub fn new(kind: RuleKind) -> Self {
        let label = match kind {
            RuleKind::MinLength(n) => format!("At least {n} characters"),
            RuleKind::MaxLength(n) => format!("No more than {n} characters"),
            RuleKind::Lowercase => "A lowercase letter".into(),
            RuleKind::Uppercase => "An uppercase letter".into(),
            RuleKind::Digit => "A number".into(),
            RuleKind::Symbol => "A symbol, such as ! or #".into(),
            RuleKind::NotCommon => "Not a commonly used password".into(),
            RuleKind::MinScore(n) => format!("Strength of at least {}", score_label(n)),
        };
        Self { kind, label }
    }

    /// Replace the wording.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Whether `password`, scored as `strength`, meets the rule.
    #[must_use]
    pub fn check(&self, password: &str, strength: &Strength) -> bool {
        match self.kind {
            RuleKind::MinLength(n) => password.chars().count() >= n,
            RuleKind::MaxLength(n) => password.chars().count() <= n,
            RuleKind::Lowercase => password.chars().any(char::is_lowercase),
            RuleKind::Uppercase => password.chars().any(char::is_uppercase),
            RuleKind::Digit => password.chars().any(|c| c.is_ascii_digit()),
            RuleKind::Symbol => password.chars().any(is_symbol),
            RuleKind::NotCommon => !password.is_empty() && common_rank(password).is_none(),
            RuleKind::MinScore(n) => strength.score >= n,
        }
    }
}

/// Twelve characters, mixed case, a number, and not a common password.
#[must_use]
pub fn default_rules() -> Vec<Rule> {
    vec![
        Rule::new(RuleKind::MinLength(12)),
        Rule::new(RuleKind::Lowercase),
        Rule::new(RuleKind::Uppercase),
        Rule::new(RuleKind::Digit),
        Rule::new(RuleKind::NotCommon),
    ]
}

/// The `passwordrules` attribute (read by Safari's password generator) for `rules`.
#[must_use]
pub fn password_rules_attr(rules: &[Rule]) -> String {
    let mut parts = Vec::new();
    for rule in rules {
        match rule.kind {
            RuleKind::MinLength(n) => parts.push(format!("minlength: {n}")),
            RuleKind::MaxLength(n) => parts.push(format!("maxlength: {n}")),
            RuleKind::Lowercase => parts.push("required: lower".into()),
            RuleKind::Uppercase => parts.push("required: upper".into()),
            RuleKind::Digit => parts.push("required: digit".into()),
            RuleKind::Symbol => parts.push("required: special".into()),
            RuleKind::NotCommon | RuleKind::MinScore(_) => {}
        }
    }
    parts.join("; ")
}

/// Why a password scored low.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warning {
    /// Fewer than 8 characters.
    Short,
    /// In the common-password list.
    Common,
    /// Contains a common password.
    ContainsCommon,
    /// Repeated characters, such as `aaa`.
    Repeat,
    /// A run such as `abc` or `6543`.
    Sequence,
    /// Neighbouring keys, such as `qwerty` or `asdf`.
    Keyboard,
    /// Contains the person's name, email or another given word.
    Personal,
}

impl Warning {
    /// Plain-language explanation.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Warning::Short => "Short passwords are easy to guess.",
            Warning::Common => "This is one of the most common passwords.",
            Warning::ContainsCommon => "It contains a common password.",
            Warning::Repeat => "Repeated characters like \"aaa\" are easy to guess.",
            Warning::Sequence => "Sequences like \"abc\" or \"6543\" are easy to guess.",
            Warning::Keyboard => "Keyboard patterns like \"qwerty\" are easy to guess.",
            Warning::Personal => "Avoid your name or email address.",
        }
    }
}

/// The result of [`estimate`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strength {
    /// 0 (very weak) to 4 (strong).
    pub score: u8,
    /// Estimated guesses, as a power of ten.
    pub guesses_log10: f64,
    /// The weakest pattern found, if any.
    pub warning: Option<Warning>,
}

/// The name of a score: "Very weak", "Weak", "Fair", "Good", "Strong".
#[must_use]
pub const fn score_label(score: u8) -> &'static str {
    match score {
        0 => "Very weak",
        1 => "Weak",
        2 => "Fair",
        3 => "Good",
        _ => "Strong",
    }
}

/// The `data-strength` value for a score.
#[must_use]
pub const fn score_key(score: u8) -> &'static str {
    match score {
        0 => "very-weak",
        1 => "weak",
        2 => "fair",
        3 => "good",
        _ => "strong",
    }
}

/// About 200 of the most common passwords, most common first (from public breach corpora).
const COMMON: [&str; 213] = [
    "123456", "password", "12345678", "qwerty", "123456789", "12345", "1234", "111111", "1234567", "dragon",
    "123123", "baseball", "abc123", "football", "monkey", "letmein", "696969", "shadow", "master", "666666",
    "qwertyuiop", "123321", "mustang", "1234567890", "michael", "654321", "superman", "1qaz2wsx", "7777777",
    "121212", "000000", "qazwsx", "123qwe", "killer", "trustno1", "jordan", "jennifer", "zxcvbnm", "asdfgh",
    "hunter", "buster", "soccer", "harley", "batman", "andrew", "tigger", "sunshine", "iloveyou", "2000",
    "charlie", "robert", "thomas", "hockey", "ranger", "daniel", "starwars", "klaster", "112233", "george",
    "computer", "michelle", "jessica", "pepper", "1111", "zxcvbn", "555555", "11111111", "131313", "freedom",
    "777777", "pass", "maggie", "159753", "aaaaaa", "ginger", "princess", "joshua", "cheese", "amanda",
    "summer", "love", "ashley", "nicole", "chelsea", "biteme", "matthew", "access", "yankees", "987654321",
    "dallas", "austin", "thunder", "taylor", "matrix", "william", "corvette", "hello", "martin", "heather",
    "secret", "merlin", "diamond", "1234qwer", "gfhjkm", "hammer", "silver", "222222", "88888888", "anthony",
    "justin", "test", "bailey", "q1w2e3r4t5", "patrick", "internet", "scooter", "orange", "11111", "golfer",
    "cookie", "richard", "samantha", "bigdog", "guitar", "jackson", "whatever", "mickey", "chicken", "sparky",
    "snoopy", "maverick", "phoenix", "camaro", "peanut", "morgan", "welcome", "falcon", "cowboy", "ferrari",
    "samsung", "andrea", "smokey", "steelers", "joseph", "mercedes", "dakota", "arsenal", "eagles", "melissa",
    "boomer", "booboo", "spider", "nascar", "monster", "tigers", "yellow", "xxxxxx", "123123123", "gateway",
    "marina", "diablo", "bulldog", "qwer1234", "compaq", "purple", "hardcore", "banana", "junior", "hannah",
    "123654", "porsche", "lakers", "iceman", "money", "cowboys", "987654", "london", "tennis", "999999",
    "ncc1701", "coffee", "scooby", "0000", "miller", "boston", "q1w2e3r4", "brandon", "yamaha", "chester",
    "mother", "forever", "johnny", "edward", "333333", "oliver", "redsox", "player", "nikita", "knight",
    "admin", "passw0rd", "password1", "qwerty123", "welcome1", "letmein1", "abcdef", "abcd1234", "p@ssw0rd",
    "changeme", "monkey1", "dragon1", "iloveyou1", "sunshine1",
];

const KEYBOARD_ROWS: [&str; 4] = ["`1234567890-=", "qwertyuiop[]\\", "asdfghjkl;'", "zxcvbnm,./"];

/// Characters that count as symbols for [`RuleKind::Symbol`].
#[must_use]
pub fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

/// Undo look-alike substitutions: `p@ssw0rd` reads as `password`.
fn unleet(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => 'o',
            '1' | '!' => 'i',
            '3' => 'e',
            '4' | '@' => 'a',
            '5' | '$' => 's',
            '7' => 't',
            _ => c,
        })
        .collect()
}

/// Rank (0 = most common) of `password` in the common list, ignoring case, look-alike
/// substitutions and a short run of trailing digits or symbols (`Password1!`).
#[must_use]
pub fn common_rank(password: &str) -> Option<usize> {
    let lower = password.to_lowercase();
    let stripped = lower.trim_end_matches(|c: char| c.is_ascii_digit() || is_symbol(c));
    let suffix_ok = lower.len() - stripped.len() <= 4 && stripped.chars().count() >= 4;
    let mut candidates = vec![lower.clone(), unleet(&lower)];
    if suffix_ok {
        candidates.push(stripped.to_string());
        candidates.push(unleet(stripped));
    }
    COMMON.iter().position(|c| candidates.iter().any(|x| x == c))
}

/// Length of the run starting at `i` where each char steps by the same +1 or -1.
fn sequence_len(chars: &[char], i: usize) -> usize {
    let step = |a: char, b: char| i64::from(u32::from(b)) - i64::from(u32::from(a));
    if i + 1 >= chars.len() || !chars[i].is_alphanumeric() {
        return 1;
    }
    let d = step(chars[i], chars[i + 1]);
    if d.abs() != 1 {
        return 1;
    }
    let mut j = i + 1;
    while j + 1 < chars.len() && step(chars[j], chars[j + 1]) == d && chars[j + 1].is_alphanumeric() {
        j += 1;
    }
    j - i + 1
}

/// Length of the run of neighbouring keys (either direction along one row) at `i`.
fn keyboard_len(lower: &[char], i: usize) -> usize {
    let mut best = 1;
    for row in KEYBOARD_ROWS {
        let row: Vec<char> = row.chars().collect();
        let Some(start) = row.iter().position(|&c| c == lower[i]) else { continue };
        for dir in [1_isize, -1] {
            let mut len = 1;
            let mut pos = start.cast_signed();
            while let Some(&next) = lower.get(i + len) {
                pos += dir;
                if pos < 0 || row.get(pos.cast_unsigned()) != Some(&next) {
                    break;
                }
                len += 1;
            }
            best = best.max(len);
        }
    }
    best
}

/// Length of the longest word from `words` (lowercase) at `i` of at least `min` chars,
/// with its rank.
fn word_at(lower: &[char], i: usize, words: &[String], min: usize) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    for (rank, word) in words.iter().enumerate() {
        let w: Vec<char> = word.chars().collect();
        if w.len() >= min && lower[i..].starts_with(&w) && best.is_none_or(|(len, _)| w.len() > len) {
            best = Some((w.len(), rank));
        }
    }
    best
}

#[allow(clippy::cast_precision_loss, reason = "small counts")]
fn log10(n: usize) -> f64 {
    (n.max(1) as f64).log10()
}

/// Estimate how hard `password` is to guess. `user_inputs` are words specific to the person
/// (name, email, site name) that should count as known.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "score is clamped to 0..=4")]
pub fn estimate(password: &str, user_inputs: &[String]) -> Strength {
    // only the first 128 chars are analysed; longer passwords are strong anyway.
    let chars: Vec<char> = password.chars().take(128).collect();
    if chars.is_empty() {
        return Strength { score: 0, guesses_log10: 0.0, warning: None };
    }
    if let Some(rank) = common_rank(password) {
        let g = log10(rank + 1) + 0.3;
        return Strength { score: score_for(g), guesses_log10: g, warning: Some(Warning::Common) };
    }
    let lower: Vec<char> = password.to_lowercase().chars().take(chars.len()).collect();
    let lower = if lower.len() == chars.len() { lower } else { chars.clone() };
    let deleet: Vec<char> = unleet(&lower.iter().collect::<String>()).chars().collect();
    let common: Vec<String> = COMMON.iter().map(ToString::to_string).collect();
    let personal: Vec<String> = user_inputs
        .iter()
        .flat_map(|s| s.to_lowercase().split(|c: char| !c.is_alphanumeric()).map(str::to_string).collect::<Vec<_>>())
        .filter(|w| w.chars().count() >= 3)
        .collect();

    let pool = [
        (chars.iter().any(|c| c.is_lowercase()), 26),
        (chars.iter().any(|c| c.is_uppercase()), 26),
        (chars.iter().any(char::is_ascii_digit), 10),
        (chars.iter().any(|&c| is_symbol(c) && c.is_ascii()), 33),
        (chars.iter().any(|&c| !c.is_ascii()), 100),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|(_, n)| n)
    .sum::<usize>();
    // Damped: real people do not pick characters uniformly from the whole pool.
    let per_char = log10(pool) * 0.6;

    let mut guesses = 0.0;
    let mut i = 0;
    // The pattern covering the most characters.
    let mut worst: Option<(usize, Warning)> = None;
    let mut note = |len: usize, w: Warning| {
        if worst.is_none_or(|(l, _)| len > l) {
            worst = Some((len, w));
        }
    };
    while i < chars.len() {
        let mut repeat = 1;
        while i + repeat < chars.len() && lower[i + repeat] == lower[i] {
            repeat += 1;
        }
        let seq = sequence_len(&lower, i);
        let keys = keyboard_len(&lower, i);
        let word = word_at(&deleet, i, &common, 4);
        let mine = word_at(&lower, i, &personal, 3);
        // Cost of each candidate pattern, in log10 guesses, and how many chars it covers.
        let mut options: Vec<(usize, f64, Warning)> = Vec::new();
        if repeat >= 3 {
            options.push((repeat, per_char + log10(repeat), Warning::Repeat));
        }
        if seq >= 3 {
            options.push((seq, log10(26 * 2) + log10(seq), Warning::Sequence));
        }
        if keys >= 4 {
            options.push((keys, log10(47 * 2) + log10(keys), Warning::Keyboard));
        }
        if let Some((len, rank)) = word {
            options.push((len, log10(rank + 1) + 1.0, Warning::ContainsCommon));
        }
        if let Some((len, _)) = mine {
            options.push((len, 1.0, Warning::Personal));
        }
        // Prefer whatever covers the most text for the fewest guesses.
        let best = options.into_iter().max_by(|a, b| {
            let ra = per_char * f64::from(u32::try_from(a.0).unwrap_or(u32::MAX)) - a.1;
            let rb = per_char * f64::from(u32::try_from(b.0).unwrap_or(u32::MAX)) - b.1;
            ra.total_cmp(&rb)
        });
        if let Some((len, cost, warning)) = best {
            guesses += cost;
            note(len, warning);
            i += len;
        } else {
            guesses += per_char;
            i += 1;
        }
    }
    let warning = worst.map(|(_, w)| w).or_else(|| (chars.len() < 8).then_some(Warning::Short));
    Strength { score: score_for(guesses), guesses_log10: guesses, warning }
}

fn score_for(guesses_log10: f64) -> u8 {
    match guesses_log10 {
        g if g < 3.0 => 0,
        g if g < 6.0 => 1,
        g if g < 8.0 => 2,
        g if g < 10.0 => 3,
        _ => 4,
    }
}

/// Props for [`PasswordFieldMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PasswordFieldInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: String,
    /// Block editing and the toggle.
    pub disabled: bool,
}

/// What [`PasswordFieldMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PasswordFieldState {
    /// The password.
    pub value: String,
    /// Shown as plain text.
    pub visible: bool,
    /// Caps Lock was on at the last key press in the field.
    pub caps_lock: bool,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

/// Inputs to [`PasswordFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordFieldEvent {
    /// The field's text changed.
    Input(String),
    /// The show/hide button was pressed.
    ToggleVisible,
    /// A key event reported Caps Lock's state.
    CapsLock(bool),
    /// The field lost focus: hide the Caps Lock warning.
    Blur,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordFieldCommand {
    /// The value changed (or should, when controlled).
    Change(String),
}

/// Password text, visibility and Caps Lock.
#[derive(Debug, Clone, Copy)]
pub struct PasswordFieldMachine;

impl Machine for PasswordFieldMachine {
    type Input = PasswordFieldInput;
    type State = PasswordFieldState;
    type Event = PasswordFieldEvent;
    type Command = PasswordFieldCommand;

    fn initial(input: &PasswordFieldInput) -> PasswordFieldState {
        PasswordFieldState {
            value: input.value.clone().unwrap_or_else(|| input.default_value.clone()),
            controlled: input.value.is_some(),
            ..PasswordFieldState::default()
        }
    }

    fn transition(
        state: &PasswordFieldState,
        event: PasswordFieldEvent,
        input: &PasswordFieldInput,
    ) -> Transition<PasswordFieldState, PasswordFieldCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        if input.disabled {
            return Transition::to(next);
        }
        match event {
            PasswordFieldEvent::Input(text) => {
                if text != state.value {
                    if !state.controlled {
                        next.value.clone_from(&text);
                    }
                    commands.push(PasswordFieldCommand::Change(text));
                }
            }
            PasswordFieldEvent::ToggleVisible => next.visible = !state.visible,
            PasswordFieldEvent::CapsLock(on) => next.caps_lock = on,
            PasswordFieldEvent::Blur => next.caps_lock = false,
        }
        Transition::with(next, commands)
    }

    fn sync(state: &PasswordFieldState, input: &PasswordFieldInput) -> Option<PasswordFieldState> {
        let mut next = state.clone();
        if let (true, Some(v)) = (state.controlled, &input.value) {
            next.value.clone_from(v);
        }
        if input.disabled {
            next.caps_lock = false;
        }
        hydroxy_statechart::changed(state, next)
    }
}

/// The checklist and meter for one password.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// The estimate.
    pub strength: Strength,
    /// Whether each rule passed, in order.
    pub passed: Vec<bool>,
    /// Every rule passed.
    pub valid: bool,
}

/// Score `password` and check it against `rules`.
#[must_use]
pub fn report(password: &str, rules: &[Rule], user_inputs: &[String]) -> Report {
    let strength = estimate(password, user_inputs);
    let passed: Vec<bool> = rules.iter().map(|r| r.check(password, &strength)).collect();
    let valid = passed.iter().all(|p| *p);
    Report { strength, passed, valid }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PasswordFieldEvent as E;

    fn score(p: &str) -> u8 {
        estimate(p, &[]).score
    }

    #[test]
    fn common_passwords_score_zero_even_disguised() {
        for p in ["password", "Password1!", "p@ssw0rd", "QWERTY", "iloveyou"] {
            assert_eq!(score(p), 0, "{p}");
            assert_eq!(estimate(p, &[]).warning, Some(Warning::Common), "{p}");
        }
    }

    #[test]
    fn patterns_are_cheap() {
        assert!(score("aaaaaaaaaaaa") <= 1);
        assert_eq!(estimate("aaaaaaaaaaaa", &[]).warning, Some(Warning::Repeat));
        assert_eq!(estimate("abcdefghijk", &[]).warning, Some(Warning::Sequence));
        assert!(score("abcdefghijk") <= 1);
        assert_eq!(estimate("poiuytrewq", &[]).warning, Some(Warning::Keyboard));
    }

    #[test]
    fn long_random_passphrases_are_strong() {
        assert_eq!(score("correct horse battery staple"), 4);
        assert_eq!(score("Tq9#vLp2!xWm"), 4);
        assert!(score("Kp3x") <= 1);
        assert_eq!(estimate("Kp3x", &[]).warning, Some(Warning::Short));
    }

    #[test]
    fn personal_words_count_as_known() {
        let mine = vec!["ada.lovelace@example.com".to_string()];
        let s = estimate("lovelace1815", &mine);
        assert_eq!(s.warning, Some(Warning::Personal));
        assert!(s.score < estimate("lovelace1815", &[]).score.max(1) + 1);
    }

    #[test]
    fn contained_common_word_is_flagged() {
        assert_eq!(estimate("xmonkeyQ", &[]).warning, Some(Warning::ContainsCommon));
    }

    #[test]
    fn score_only_grows_with_random_chars() {
        let mut last = 0;
        for p in ["k", "kV", "kV8", "kV8#", "kV8#q", "kV8#qZ", "kV8#qZ2", "kV8#qZ2m", "kV8#qZ2m!", "kV8#qZ2m!r"] {
            let s = score(p);
            assert!(s >= last, "{p}");
            last = s;
        }
        assert_eq!(last, 4);
    }

    #[test]
    fn rules_check_their_kind() {
        let rules = default_rules();
        let r = report("Tq9vLpxWmz2k", &rules, &[]);
        assert!(r.valid);
        let r = report("tq9vlpxwmz2k", &rules, &[]);
        assert_eq!(r.passed, vec![true, true, false, true, true]);
        assert!(!report("Password1234", &rules, &[]).passed[4]);
        assert!(Rule::new(RuleKind::Symbol).check("a#", &estimate("a#", &[])));
        assert!(!Rule::new(RuleKind::MaxLength(3)).check("abcd", &estimate("abcd", &[])));
    }

    #[test]
    fn passwordrules_attr_lists_requirements() {
        assert_eq!(
            password_rules_attr(&default_rules()),
            "minlength: 12; required: lower; required: upper; required: digit"
        );
    }

    #[test]
    fn autocomplete_follows_mode() {
        assert_eq!(PasswordMode::New.autocomplete(), "new-password");
        assert_eq!(PasswordMode::Current.autocomplete(), "current-password");
    }

    #[test]
    fn toggle_caps_and_blur() {
        let input = PasswordFieldInput::default();
        let s = PasswordFieldMachine::initial(&input);
        let s = PasswordFieldMachine::transition(&s, E::ToggleVisible, &input).resolve(&s).state;
        assert!(s.visible);
        let s = PasswordFieldMachine::transition(&s, E::CapsLock(true), &input).resolve(&s).state;
        assert!(s.caps_lock);
        let s = PasswordFieldMachine::transition(&s, E::Blur, &input).resolve(&s).state;
        assert!(!s.caps_lock);
    }

    #[test]
    fn controlled_input_reports_without_changing() {
        let input = PasswordFieldInput { value: Some("a".into()), ..Default::default() };
        let s = PasswordFieldMachine::initial(&input);
        let t = PasswordFieldMachine::transition(&s, E::Input("ab".into()), &input).resolve(&s);
        assert_eq!(t.commands, vec![PasswordFieldCommand::Change("ab".into())]);
        assert_eq!(t.state.value, "a");
        let synced = hydroxy_statechart::sync_or_keep::<PasswordFieldMachine>(&t.state, &PasswordFieldInput { value: Some("ab".into()), ..input });
        assert_eq!(synced.value, "ab");
    }

    #[test]
    fn disabled_ignores_events() {
        let input = PasswordFieldInput { disabled: true, ..Default::default() };
        let s = PasswordFieldMachine::initial(&input);
        let t = PasswordFieldMachine::transition(&s, E::Input("x".into()), &input).resolve(&s);
        assert!(t.commands.is_empty());
    }
}
