//! Password field connector. Anatomy: root, label, field, control, toggle, caps-lock,
//! meter, meter-bar, strength, warning, rules, rule. Every part carries
//! `data-scope="password-field"` and its `data-part`.
//!
//! The control is a native `<input>` whose `type` flips between `password` and `text`. The
//! toggle is a `<button aria-pressed>` with a fixed name ("Show password"), so its state is
//! read from `aria-pressed`, per the APG toggle-button pattern. The meter is a `meter`-role
//! element whose text sits in a polite live region, so a score change is announced once.
//! The rules are a list whose items say "met" or "not met" to screen readers.

pub use oxide_machines::password_field::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "password-field";

/// Element ids for one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The `<input>`; the label's `for` and the toggle's `aria-controls`.
    pub control: String,
    /// The strength text.
    pub strength: String,
    /// The rules list.
    pub rules: String,
    /// The Caps Lock warning.
    pub caps: String,
    /// The hint.
    pub hint: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids {
        control: format!("{root}-control"),
        strength: format!("{root}-strength"),
        rules: format!("{root}-rules"),
        caps: format!("{root}-caps"),
        hint: format!("{root}-hint"),
    }
}

/// `aria-describedby` for the input: the hint, Caps Lock warning, strength and rules that
/// are showing.
#[must_use]
pub fn described_by(ids: &Ids, hint: bool, caps: bool, mode: PasswordMode, rules: bool) -> Option<String> {
    let mut parts = Vec::new();
    if hint {
        parts.push(ids.hint.as_str());
    }
    if caps {
        parts.push(ids.caps.as_str());
    }
    if mode == PasswordMode::New {
        parts.push(ids.strength.as_str());
        if rules {
            parts.push(ids.rules.as_str());
        }
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

/// The input's `type`.
#[must_use]
pub const fn input_type(visible: bool) -> &'static str {
    if visible { "text" } else { "password" }
}

/// The toggle's accessible name. It stays fixed; `aria-pressed` carries the state.
pub const TOGGLE_LABEL: &str = "Show password";

/// The strength line: "Strength: Fair", or a prompt while empty.
#[must_use]
pub fn strength_text(value: &str, strength: &Strength) -> String {
    if value.is_empty() {
        "Enter a password to see its strength".into()
    } else {
        format!("Strength: {}", score_label(strength.score))
    }
}

/// `aria-valuetext` for the meter.
#[must_use]
pub fn meter_valuetext(value: &str, strength: &Strength) -> String {
    if value.is_empty() { "No password".into() } else { score_label(strength.score).into() }
}

/// What a screen reader hears after a rule: "met" or "not met".
#[must_use]
pub const fn rule_status(passed: bool) -> &'static str {
    if passed { "met" } else { "not met" }
}

/// The `data-strength` value: empty while the field is.
#[must_use]
pub fn data_strength(value: &str, strength: &Strength) -> &'static str {
    if value.is_empty() { "none" } else { score_key(strength.score) }
}

/// `getModifierState('CapsLock')` only means something on key events for printable keys
/// and Caps Lock itself; this says which key events should update the warning.
#[must_use]
pub fn reports_caps_lock(key: &str) -> bool {
    key == "CapsLock" || key.chars().count() == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_what_is_showing() {
        let ids = ids("p");
        assert_eq!(described_by(&ids, false, false, PasswordMode::Current, true), None);
        assert_eq!(described_by(&ids, true, true, PasswordMode::New, true).as_deref(), Some("p-hint p-caps p-strength p-rules"));
    }

    #[test]
    fn strength_text_waits_for_input() {
        let s = estimate("", &[]);
        assert_eq!(strength_text("", &s), "Enter a password to see its strength");
        assert_eq!(data_strength("", &s), "none");
        let s = estimate("correct horse battery staple", &[]);
        assert_eq!(strength_text("x", &s), "Strength: Strong");
    }

    #[test]
    fn caps_lock_from_printable_keys() {
        assert!(reports_caps_lock("a"));
        assert!(reports_caps_lock("CapsLock"));
        assert!(!reports_caps_lock("Shift"));
        assert_eq!(input_type(true), "text");
    }
}
