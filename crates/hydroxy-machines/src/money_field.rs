//! Money field: an amount in a currency, held as integer minor units (cents,
//! fils, yen) so no float ever touches the value. Typing regroups the integer
//! part as you go and stops at the currency's precision (JPY 0, USD 2, BHD 3).
//! Pasting reads whatever a person copies, such as `$1,234.50` or `1.234,50 €`.
//! Enter or blur clamps to `min`/`max` and pads the fraction; arrows step.

use hydroxy_statechart::{Machine, Transition};

use crate::as_you_type as ayt;

/// A currency the field knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Currency {
    /// ISO 4217 code, such as `USD`.
    pub code: &'static str,
    /// Display symbol, such as `$`.
    pub symbol: &'static str,
    /// English name, for screen readers and the picker.
    pub name: &'static str,
    /// Digits after the decimal mark: 0 for JPY, 2 for USD, 3 for BHD.
    pub minor_digits: u8,
}

const fn cur(code: &'static str, symbol: &'static str, name: &'static str, minor_digits: u8) -> Currency {
    Currency { code, symbol, name, minor_digits }
}

/// Every currency the field knows, by code.
pub const CURRENCIES: &[Currency] = &[
    cur("AED", "AED", "UAE dirham", 2),
    cur("AUD", "A$", "Australian dollar", 2),
    cur("BHD", "BD", "Bahraini dinar", 3),
    cur("BRL", "R$", "Brazilian real", 2),
    cur("CAD", "CA$", "Canadian dollar", 2),
    cur("CHF", "CHF", "Swiss franc", 2),
    cur("CLP", "CLP$", "Chilean peso", 0),
    cur("CNY", "CN¥", "Chinese yuan", 2),
    cur("CZK", "Kč", "Czech koruna", 2),
    cur("DKK", "kr", "Danish krone", 2),
    cur("EUR", "€", "Euro", 2),
    cur("GBP", "£", "British pound", 2),
    cur("HKD", "HK$", "Hong Kong dollar", 2),
    cur("HUF", "Ft", "Hungarian forint", 2),
    cur("IDR", "Rp", "Indonesian rupiah", 2),
    cur("ILS", "₪", "Israeli new shekel", 2),
    cur("INR", "₹", "Indian rupee", 2),
    cur("ISK", "kr", "Icelandic króna", 0),
    cur("JOD", "JD", "Jordanian dinar", 3),
    cur("JPY", "¥", "Japanese yen", 0),
    cur("KRW", "₩", "South Korean won", 0),
    cur("KWD", "KD", "Kuwaiti dinar", 3),
    cur("MXN", "MX$", "Mexican peso", 2),
    cur("NOK", "kr", "Norwegian krone", 2),
    cur("NZD", "NZ$", "New Zealand dollar", 2),
    cur("OMR", "OMR", "Omani rial", 3),
    cur("PLN", "zł", "Polish złoty", 2),
    cur("SAR", "SAR", "Saudi riyal", 2),
    cur("SEK", "kr", "Swedish krona", 2),
    cur("SGD", "S$", "Singapore dollar", 2),
    cur("THB", "฿", "Thai baht", 2),
    cur("TND", "DT", "Tunisian dinar", 3),
    cur("TRY", "₺", "Turkish lira", 2),
    cur("USD", "$", "US dollar", 2),
    cur("VND", "₫", "Vietnamese đồng", 0),
    cur("ZAR", "R", "South African rand", 2),
];

/// Look a currency up by code, ignoring case.
#[must_use]
pub fn currency(code: &str) -> Option<&'static Currency> {
    CURRENCIES.iter().find(|c| c.code.eq_ignore_ascii_case(code))
}

/// Minor digits for `code`; unknown codes get 2.
#[must_use]
pub fn minor_digits(code: &str) -> u8 {
    currency(code).map_or(2, |c| c.minor_digits)
}

/// Most integer digits accepted, which keeps any value inside `i64` at three minor digits.
const MAX_INT_DIGITS: usize = 15;

fn pow10(d: u8) -> i64 {
    10i64.pow(u32::from(d))
}

/// Group an unsigned digit string in threes: `1234567` → `1,234,567`.
fn group_digits(int: &str, group: char) -> String {
    let n = int.len();
    let mut out = String::with_capacity(n + n / 3);
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (n - i).is_multiple_of(3) {
            out.push(group);
        }
        out.push(c);
    }
    out
}

/// Format minor units in full: `123450` at 2 digits → `1,234.50`.
#[must_use]
pub fn format_minor(value: i64, digits: u8, group: char, decimal: char) -> String {
    let abs = value.unsigned_abs();
    let p = 10u64.pow(u32::from(digits));
    let mut out = String::new();
    if value < 0 {
        out.push('-');
    }
    out.push_str(&group_digits(&(abs / p).to_string(), group));
    if digits > 0 {
        out.push(decimal);
        let frac = (abs % p).to_string();
        out.extend(std::iter::repeat_n('0', usize::from(digits) - frac.len()));
        out.push_str(&frac);
    }
    out
}

/// Re-express minor units from one precision to another, rounding half away
/// from zero: 12.34 USD (1234) to JPY gives 12; 5 JPY to BHD gives 5000.
#[must_use]
pub fn convert(value: i64, from: u8, to: u8) -> i64 {
    if to >= from {
        return value.saturating_mul(pow10(to - from));
    }
    let p = pow10(from - to);
    let (q, r) = (value / p, value % p);
    if r.abs() * 2 >= p { q + value.signum() } else { q }
}

/// Read a pasted or free-form amount into minor units. Currency symbols, codes
/// and spaces are ignored; a minus or accounting brackets make it negative.
/// The decimal mark is worked out from the text: with both `.` and `,` the
/// later one wins; a lone mark is decimal when it is the field's own `decimal`
/// or isn't followed by exactly three digits. Extra fraction digits round half up.
#[must_use]
pub fn parse_amount(text: &str, digits: u8, decimal: char) -> Option<i64> {
    let t = text.trim();
    let negative = t.contains(['-', '−']) || (t.starts_with('(') && t.ends_with(')'));
    let marks: Vec<(usize, char)> = t.char_indices().filter(|(_, c)| matches!(c, '.' | ',')).collect();
    let decimal_at = match marks.last() {
        None => None,
        Some(&(i, last)) => {
            let mixed = marks.iter().any(|(_, c)| *c != last);
            let after = t[i + last.len_utf8()..].chars().take_while(char::is_ascii_digit).count();
            let lone = marks.len() == 1;
            (mixed || (lone && (last == decimal || after != 3))).then_some(i)
        }
    };
    let (int_part, frac_part) = match decimal_at {
        Some(i) => (&t[..i], &t[i + 1..]),
        None => (t, ""),
    };
    let int: String = int_part.chars().filter(char::is_ascii_digit).collect();
    let frac: String = frac_part.chars().filter(char::is_ascii_digit).collect();
    if int.is_empty() && frac.is_empty() {
        return None;
    }
    let int = int.trim_start_matches('0');
    if int.len() > MAX_INT_DIGITS {
        return None;
    }
    let whole: i64 = if int.is_empty() { 0 } else { int.parse().ok()? };
    let d = usize::from(digits);
    let mut kept: String = frac.chars().take(d).collect();
    while kept.len() < d {
        kept.push('0');
    }
    let mut minor = whole.checked_mul(pow10(digits))? + if kept.is_empty() { 0 } else { kept.parse::<i64>().ok()? };
    if frac.chars().nth(d).is_some_and(|c| c >= '5') {
        minor += 1;
    }
    Some(if negative { -minor } else { minor })
}

/// Props for [`MoneyFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoneyFieldInput {
    /// Controlled amount in minor units. `Some` at start fixes controlled mode.
    pub value: Option<i64>,
    /// Uncontrolled initial amount in minor units.
    pub default_value: Option<i64>,
    /// Controlled currency code. The owner's change re-reads the amount in it.
    pub currency: Option<String>,
    /// Currency to start in when uncontrolled.
    pub default_currency: String,
    /// Lowest amount, in minor units. Below zero (or unset) allows a minus sign.
    pub min: Option<i64>,
    /// Highest amount, in minor units.
    pub max: Option<i64>,
    /// Arrow-key step in minor units. Default: one major unit.
    pub step: Option<i64>,
    /// Thousands separator, such as `,` or `.` or a narrow space.
    pub group: char,
    /// Decimal mark, `.` or `,`.
    pub decimal: char,
    /// Ignore every change.
    pub disabled: bool,
    /// Focusable and selectable, but unchangeable.
    pub read_only: bool,
}

impl Default for MoneyFieldInput {
    fn default() -> Self {
        Self {
            value: None,
            default_value: None,
            currency: None,
            default_currency: "USD".into(),
            min: None,
            max: None,
            step: None,
            group: ',',
            decimal: '.',
            disabled: false,
            read_only: false,
        }
    }
}

impl MoneyFieldInput {
    /// Clamp into `[min, max]`.
    #[must_use]
    pub fn clamp(&self, v: i64) -> i64 {
        let v = self.max.map_or(v, |max| v.min(max));
        self.min.map_or(v, |min| v.max(min))
    }

    fn allow_negative(&self) -> bool {
        self.min.is_none_or(|m| m < 0)
    }

    fn is_sig(&self) -> impl Fn(char) -> bool + Copy {
        let decimal = self.decimal;
        move |c: char| c.is_ascii_digit() || c == decimal || c == '-' || c == '−'
    }

    /// Reformat typed text: sign, grouped integer part, then the decimal mark and
    /// up to `digits` fraction digits, exactly as far as the person has typed.
    /// Returns the text and its value.
    fn shape(&self, text: &str, digits: u8) -> (String, Option<i64>) {
        let (mut negative, mut decimal, mut int, mut frac, mut any) =
            (false, false, String::new(), String::new(), false);
        for c in text.chars() {
            if (c == '-' || c == '−') && self.allow_negative() && !any && !decimal {
                negative = true;
            } else if c.is_ascii_digit() {
                any = true;
                if decimal {
                    if frac.len() < usize::from(digits) {
                        frac.push(c);
                    }
                } else if int.len() < MAX_INT_DIGITS {
                    int.push(c);
                }
            } else if c == self.decimal && digits > 0 && !decimal {
                decimal = true;
            }
        }
        let trimmed = int.trim_start_matches('0');
        let int = if trimmed.is_empty() && (any || decimal) { "0" } else { trimmed };
        let mut out = String::new();
        if negative {
            out.push('-');
        }
        if int.is_empty() {
            return (out, None);
        }
        out.push_str(&group_digits(int, self.group));
        if decimal {
            out.push(self.decimal);
            out.push_str(&frac);
        }
        let mut padded = frac;
        while padded.len() < usize::from(digits) {
            padded.push('0');
        }
        let whole: i64 = int.parse().unwrap_or(0);
        let minor = whole * pow10(digits) + padded.parse::<i64>().unwrap_or(0);
        (out, Some(if negative { -minor } else { minor }))
    }
}

/// What [`MoneyFieldMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MoneyFieldState {
    /// The amount in minor units. `None` is empty.
    pub value: Option<i64>,
    /// The input's text.
    pub draft: String,
    /// Where the caret belongs after the last edit, as a char index.
    pub caret: usize,
    /// The currency code.
    pub currency: String,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

impl MoneyFieldState {
    /// The current currency's minor digits.
    #[must_use]
    pub fn digits(&self) -> u8 {
        minor_digits(&self.currency)
    }
}

/// Inputs to [`MoneyFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyFieldEvent {
    /// The person typed. `caret` is a char index.
    SetText {
        /// The input's new text.
        text: String,
        /// Caret after the edit.
        caret: usize,
    },
    /// Text pasted over the field.
    Paste(String),
    /// Clamp and pad (Enter or blur).
    Commit,
    /// Step up or down; `large` is ten steps.
    Step {
        /// Up (`true`) or down.
        up: bool,
        /// Ten steps at once.
        large: bool,
    },
    /// Jump to `min`, if set.
    ToMin,
    /// Jump to `max`, if set.
    ToMax,
    /// Switch currency, keeping the amount and re-rounding it to the new precision.
    SetCurrency(String),
}

/// Side effects from [`MoneyFieldMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyFieldCommand {
    /// The amount changed, in minor units of the current currency.
    ValueChange(Option<i64>),
    /// The currency changed.
    CurrencyChange(String),
}

/// An amount in a currency.
#[derive(Debug, Clone, Copy)]
pub struct MoneyFieldMachine;

type Step = Transition<MoneyFieldState, MoneyFieldCommand>;

fn settle(state: &MoneyFieldState, value: Option<i64>, input: &MoneyFieldInput) -> Step {
    let mut next = state.clone();
    next.value = value;
    next.draft = value.map(|v| format_minor(v, state.digits(), input.group, input.decimal)).unwrap_or_default();
    next.caret = next.draft.chars().count();
    let commands = if value == state.value { Vec::new() } else { vec![MoneyFieldCommand::ValueChange(value)] };
    Transition::with(next, commands)
}

impl Machine for MoneyFieldMachine {
    type Input = MoneyFieldInput;
    type State = MoneyFieldState;
    type Event = MoneyFieldEvent;
    type Command = MoneyFieldCommand;

    fn initial(input: &MoneyFieldInput) -> MoneyFieldState {
        let currency = input.currency.clone().unwrap_or_else(|| input.default_currency.clone());
        let value = input.value.or(input.default_value);
        let mut state = MoneyFieldState { currency, controlled: input.value.is_some(), ..MoneyFieldState::default() };
        state.draft = value.map(|v| format_minor(v, state.digits(), input.group, input.decimal)).unwrap_or_default();
        state.caret = state.draft.chars().count();
        state.value = value;
        state
    }

    fn transition(state: &MoneyFieldState, event: MoneyFieldEvent, input: &MoneyFieldInput) -> Step {
        if input.disabled || input.read_only {
            return Transition::stay();
        }
        let digits = state.digits();
        let sig = input.is_sig();
        match event {
            MoneyFieldEvent::SetText { mut text, mut caret } => {
                if ayt::deleted_literal(&state.draft, &text, sig) {
                    (text, caret) = ayt::drop_sig_before(&text, caret, sig);
                }
                let at_end = caret >= text.chars().count();
                let (draft, value) = input.shape(&text, digits);
                let mut next = state.clone();
                next.caret = if at_end {
                    draft.chars().count()
                } else {
                    ayt::caret_after(&draft, ayt::sig_before(&text, caret, sig), sig)
                };
                next.draft = draft;
                next.value = value;
                let commands =
                    if value == state.value { Vec::new() } else { vec![MoneyFieldCommand::ValueChange(value)] };
                Transition::with(next, commands)
            }
            MoneyFieldEvent::Paste(text) => match parse_amount(&text, digits, input.decimal) {
                Some(v) if v >= 0 || input.allow_negative() => settle(state, Some(v), input),
                Some(v) => settle(state, Some(-v), input),
                None => Transition::stay(),
            },
            MoneyFieldEvent::Commit => {
                let value = input.shape(&state.draft, digits).1.map(|v| input.clamp(v));
                settle(state, value, input)
            }
            MoneyFieldEvent::Step { up, large } => {
                let step = input.step.filter(|s| *s > 0).unwrap_or_else(|| pow10(digits));
                let delta = step.saturating_mul(if large { 10 } else { 1 });
                let base = input.shape(&state.draft, digits).1.unwrap_or(0);
                let v = if up { base.saturating_add(delta) } else { base.saturating_sub(delta) };
                let v = if input.allow_negative() { v } else { v.max(0) };
                settle(state, Some(input.clamp(v)), input)
            }
            MoneyFieldEvent::ToMin => match input.min {
                Some(min) => settle(state, Some(min), input),
                None => Transition::stay(),
            },
            MoneyFieldEvent::ToMax => match input.max {
                Some(max) => settle(state, Some(max), input),
                None => Transition::stay(),
            },
            MoneyFieldEvent::SetCurrency(code) => {
                let Some(c) = currency(&code) else { return Transition::stay() };
                if c.code == state.currency {
                    return Transition::stay();
                }
                let value = input.shape(&state.draft, digits).1.map(|v| convert(v, digits, c.minor_digits));
                let renamed = MoneyFieldState { currency: c.code.to_string(), ..state.clone() };
                let mut step = settle(&renamed, value, input);
                step.commands.insert(0, MoneyFieldCommand::CurrencyChange(c.code.to_string()));
                step
            }
        }
    }

    fn sync(state: &MoneyFieldState, input: &MoneyFieldInput) -> Option<MoneyFieldState> {
        let mut next = state.clone();
        if let Some(code) = input.currency.as_deref().and_then(currency).map(|c| c.code)
            && code != state.currency
        {
            next.currency = code.to_string();
        }
        let value = if state.controlled { input.value } else { state.value };
        if next.currency != state.currency || value != state.value {
            next.value = value;
            next.draft = value.map(|v| format_minor(v, next.digits(), input.group, input.decimal)).unwrap_or_default();
            next.caret = next.draft.chars().count();
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MoneyFieldCommand as C;
    use MoneyFieldEvent as E;
    use hydroxy_statechart::step;

    fn typed(
        s: &MoneyFieldState,
        text: &str,
        input: &MoneyFieldInput,
    ) -> hydroxy_statechart::Step<MoneyFieldState, MoneyFieldCommand> {
        step::<MoneyFieldMachine>(s, E::SetText { text: text.into(), caret: text.chars().count() }, input)
    }

    fn start(currency: &str) -> (MoneyFieldState, MoneyFieldInput) {
        let input = MoneyFieldInput { default_currency: currency.into(), ..MoneyFieldInput::default() };
        (MoneyFieldMachine::initial(&input), input)
    }

    #[test]
    fn typing_groups_and_reports_minor_units() {
        let (s, input) = start("USD");
        let t = typed(&s, "1234567.5", &input);
        assert_eq!(t.state.draft, "1,234,567.5");
        assert_eq!(t.commands, vec![C::ValueChange(Some(123_456_750))]);
    }

    #[test]
    fn precision_follows_the_currency() {
        let (s, input) = start("USD");
        assert_eq!(typed(&s, "1.239", &input).state.draft, "1.23");
        let (s, input) = start("JPY");
        let t = typed(&s, "1500.5", &input);
        assert_eq!((t.state.draft.as_str(), t.state.value), ("15,005", Some(15_005)));
        let (s, input) = start("BHD");
        assert_eq!(typed(&s, "2.5", &input).state.value, Some(2_500));
    }

    #[test]
    fn leading_zeros_and_bare_decimal() {
        let (s, input) = start("USD");
        assert_eq!(typed(&s, "007", &input).state.draft, "7");
        assert_eq!(typed(&s, ".", &input).state.draft, "0.");
        assert_eq!(typed(&s, "", &input).state.value, None);
    }

    #[test]
    fn caret_keeps_its_digit_across_regrouping() {
        let (s, input) = start("USD");
        let s = typed(&s, "1234", &input).state;
        assert_eq!(s.draft, "1,234");
        // Type "9" after the "1": the caret stays after the "9".
        let t = step::<MoneyFieldMachine>(&s, E::SetText { text: "19,234".into(), caret: 2 }, &input);
        assert_eq!(t.state.draft, "19,234");
        assert_eq!(t.state.caret, 2);
    }

    #[test]
    fn backspace_over_a_group_separator_removes_the_digit() {
        let (s, input) = start("USD");
        let s = typed(&s, "1234", &input).state;
        let t = step::<MoneyFieldMachine>(&s, E::SetText { text: "1234".into(), caret: 1 }, &input);
        assert_eq!(t.state.draft, "234");
        assert_eq!(t.state.caret, 0);
    }

    #[test]
    fn paste_reads_symbols_and_either_decimal_mark() {
        assert_eq!(parse_amount("$1,234.50", 2, '.'), Some(123_450));
        assert_eq!(parse_amount("1.234,50 €", 2, '.'), Some(123_450));
        assert_eq!(parse_amount("¥1,234", 0, '.'), Some(1_234));
        assert_eq!(parse_amount("12,5", 2, '.'), Some(1_250));
        assert_eq!(parse_amount("(45.00)", 2, '.'), Some(-4_500));
        assert_eq!(parse_amount("1.2345", 3, '.'), Some(1_235));
        assert_eq!(parse_amount("USD", 2, '.'), None);
        let (s, input) = start("USD");
        let t = step::<MoneyFieldMachine>(&s, E::Paste("USD 1,234.5".into()), &input);
        assert_eq!(t.state.draft, "1,234.50");
    }

    #[test]
    fn commit_clamps_and_pads() {
        let input = MoneyFieldInput { min: Some(0), max: Some(10_000), ..MoneyFieldInput::default() };
        let s = MoneyFieldMachine::initial(&input);
        let s = typed(&s, "250", &input).state;
        let t = step::<MoneyFieldMachine>(&s, E::Commit, &input);
        assert_eq!((t.state.draft.as_str(), t.state.value), ("100.00", Some(10_000)));
        // No minus allowed when min is 0.
        assert_eq!(typed(&s, "-5", &input).state.value, Some(500));
    }

    #[test]
    fn steps_by_one_major_unit_or_ten() {
        let (s, input) = start("USD");
        let s = typed(&s, "1.25", &input).state;
        assert_eq!(step::<MoneyFieldMachine>(&s, E::Step { up: true, large: false }, &input).state.value, Some(225));
        assert_eq!(step::<MoneyFieldMachine>(&s, E::Step { up: false, large: true }, &input).state.draft, "-8.75");
        let (s, input) = start("JPY");
        assert_eq!(step::<MoneyFieldMachine>(&s, E::Step { up: true, large: false }, &input).state.value, Some(1));
    }

    #[test]
    fn switching_currency_keeps_the_amount() {
        let (s, input) = start("USD");
        let s = typed(&s, "12.50", &input).state;
        let t = step::<MoneyFieldMachine>(&s, E::SetCurrency("JPY".into()), &input);
        assert_eq!(t.state.value, Some(13));
        assert_eq!(t.commands, vec![C::CurrencyChange("JPY".into()), C::ValueChange(Some(13))]);
        let t = step::<MoneyFieldMachine>(&t.state, E::SetCurrency("BHD".into()), &input);
        assert_eq!(t.state.draft, "13.000");
        assert_eq!(convert(-1_250, 2, 0), -13);
    }

    #[test]
    fn controlled_value_and_currency_sync() {
        let mut input = MoneyFieldInput { value: Some(100), ..MoneyFieldInput::default() };
        let s = MoneyFieldMachine::initial(&input);
        assert_eq!(s.draft, "1.00");
        input.value = Some(250);
        input.currency = Some("KWD".into());
        let s = hydroxy_statechart::sync_or_keep::<MoneyFieldMachine>(&s, &input);
        assert_eq!((s.currency.as_str(), s.draft.as_str()), ("KWD", "0.250"));
    }

    #[test]
    fn european_separators() {
        let input =
            MoneyFieldInput { group: '.', decimal: ',', default_currency: "EUR".into(), ..MoneyFieldInput::default() };
        let s = MoneyFieldMachine::initial(&input);
        assert_eq!(typed(&s, "1234,5", &input).state.draft, "1.234,5");
        assert_eq!(format_minor(-123_456, 2, '.', ','), "-1.234,56");
    }

    #[test]
    fn disabled_ignores_input() {
        let input = MoneyFieldInput { disabled: true, ..MoneyFieldInput::default() };
        let s = MoneyFieldMachine::initial(&input);
        assert_eq!(typed(&s, "5", &input).state, s);
    }
}
