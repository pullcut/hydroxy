//! Money field connector. Anatomy, all under `data-scope="money-field"`: root,
//! label, field, currency (a native `<select>`), symbol, control (a text input with
//! `role="spinbutton"`), hidden input. The value is integer minor units; the
//! spinbutton's `aria-valuetext` reads it back as money, such as "$1,234.50".

pub use hydroxy_machines::money_field::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "money-field";

/// Element ids for one money field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The amount input; the label's `for`.
    pub control: String,
    /// The currency `<select>`.
    pub currency: String,
    /// The visible label, so the select can name itself after it.
    pub label: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { control: format!("{root}-control"), currency: format!("{root}-currency"), label: format!("{root}-label") }
}

/// ARIA for the spinbutton.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlAttrs {
    /// `aria-valuenow`, in major units.
    pub valuenow: Option<f64>,
    /// `aria-valuemin`, in major units.
    pub valuemin: Option<f64>,
    /// `aria-valuemax`, in major units.
    pub valuemax: Option<f64>,
    /// `aria-valuetext`, such as `$1,234.50`.
    pub valuetext: Option<String>,
}

#[allow(clippy::cast_precision_loss, reason = "ARIA wants a number; amounts above 2^53 minor units only lose screen-reader precision")]
fn major(v: i64, digits: u8) -> f64 {
    v as f64 / 10f64.powi(i32::from(digits))
}

/// Attributes for the control.
#[must_use]
pub fn control_attrs(state: &MoneyFieldState, input: &MoneyFieldInput) -> ControlAttrs {
    let d = state.digits();
    ControlAttrs {
        valuenow: state.value.map(|v| major(v, d)),
        valuemin: input.min.map(|v| major(v, d)),
        valuemax: input.max.map(|v| major(v, d)),
        valuetext: state.value.map(|v| value_text(v, &state.currency, input)),
    }
}

/// An amount as money: symbol, then the formatted number. `-$5.00` for negatives.
#[must_use]
pub fn value_text(value: i64, code: &str, input: &MoneyFieldInput) -> String {
    let c = currency(code);
    let symbol = c.map_or(code, |c| c.symbol);
    let digits = c.map_or(2, |c| c.minor_digits);
    let body = format_minor(value.abs(), digits, input.group, input.decimal);
    if value < 0 { format!("-{symbol}{body}") } else { format!("{symbol}{body}") }
}

/// The symbol shown inside the field before the amount.
#[must_use]
pub fn symbol(code: &str) -> &str {
    currency(code).map_or(code, |c| c.symbol)
}

/// What the currency `<select>` lists for `codes`: `(code, "USD — US dollar")`.
/// Unknown codes are left out; an empty list offers every currency.
#[must_use]
pub fn currency_options(codes: &[String]) -> Vec<(&'static str, String)> {
    let list: Vec<&'static Currency> = if codes.is_empty() {
        CURRENCIES.iter().collect()
    } else {
        codes.iter().filter_map(|c| currency(c)).collect()
    };
    list.into_iter().map(|c| (c.code, format!("{} — {}", c.code, c.name))).collect()
}

/// The hidden input's value: minor units, empty when there is no amount.
#[must_use]
pub fn form_value(state: &MoneyFieldState) -> String {
    state.value.map(|v| v.to_string()).unwrap_or_default()
}

/// Map a `KeyboardEvent.key` on the control. `Some` means: prevent the default.
/// Shift with an arrow steps ten at a time, like PageUp/PageDown.
#[must_use]
pub fn keydown(key: &str, shift: bool, input: &MoneyFieldInput) -> Option<MoneyFieldEvent> {
    match key {
        "ArrowUp" => Some(MoneyFieldEvent::Step { up: true, large: shift }),
        "ArrowDown" => Some(MoneyFieldEvent::Step { up: false, large: shift }),
        "PageUp" => Some(MoneyFieldEvent::Step { up: true, large: true }),
        "PageDown" => Some(MoneyFieldEvent::Step { up: false, large: true }),
        "Home" if input.min.is_some() => Some(MoneyFieldEvent::ToMin),
        "End" if input.max.is_some() => Some(MoneyFieldEvent::ToMax),
        "Enter" => Some(MoneyFieldEvent::Commit),
        _ => None,
    }
}

/// `inputmode` for the control: no decimal key for zero-digit currencies.
#[must_use]
pub fn inputmode(state: &MoneyFieldState) -> &'static str {
    if state.digits() == 0 { "numeric" } else { "decimal" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valuetext_reads_as_money() {
        let input = MoneyFieldInput::default();
        assert_eq!(value_text(123_450, "USD", &input), "$1,234.50");
        assert_eq!(value_text(-500, "GBP", &input), "-£5.00");
        assert_eq!(value_text(1_500, "JPY", &input), "¥1,500");
        let s = MoneyFieldState { value: Some(1_250), currency: "BHD".into(), ..MoneyFieldState::default() };
        let a = control_attrs(&s, &input);
        assert_eq!(a.valuenow, Some(1.25));
        assert_eq!(inputmode(&s), "decimal");
    }

    #[test]
    fn keys_and_options() {
        let input = MoneyFieldInput::default();
        assert_eq!(keydown("ArrowUp", true, &input), Some(MoneyFieldEvent::Step { up: true, large: true }));
        assert_eq!(keydown("Home", false, &input), None);
        let opts = currency_options(&["usd".into(), "XXX".into(), "JPY".into()]);
        assert_eq!(opts.iter().map(|o| o.0).collect::<Vec<_>>(), ["USD", "JPY"]);
        assert_eq!(currency_options(&[]).len(), CURRENCIES.len());
    }
}
