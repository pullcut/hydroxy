//! Async validated input connector. Wraps the existing `Input` and decides
//! what its hint and error say for each [`Validity`]. Parts: root, live.
//! Every part carries `data-scope="async-validated-input"`.

pub use hydroxy_machines::async_validated_input::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "async-validated-input";

/// Words shown for each state. The owner supplies them; these are the defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Messages {
    /// While the validator runs.
    pub validating: String,
    /// After it accepts.
    pub valid: String,
}

impl Default for Messages {
    fn default() -> Self {
        Self { validating: "Checking…".into(), valid: "Looks good.".into() }
    }
}

/// The `data-status` value: `idle`, `validating`, `valid` or `invalid`.
#[must_use]
pub const fn data_status(validity: &Validity) -> &'static str {
    match validity {
        Validity::Idle => "idle",
        Validity::Validating => "validating",
        Validity::Valid => "valid",
        Validity::Invalid(_) => "invalid",
    }
}

/// What the field shows below it: (hint, error). An error replaces the hint, as in `Input`.
#[must_use]
pub fn feedback(validity: &Validity, messages: &Messages, base_hint: Option<&str>) -> (Option<String>, Option<String>) {
    match validity {
        Validity::Idle => (base_hint.map(str::to_string), None),
        Validity::Validating => (Some(messages.validating.clone()), None),
        Validity::Valid => (Some(messages.valid.clone()), None),
        Validity::Invalid(e) => (None, Some(e.clone())),
    }
}

/// What the live region says. Idle stays quiet.
#[must_use]
pub fn live_text(validity: &Validity, messages: &Messages) -> String {
    match validity {
        Validity::Idle => String::new(),
        Validity::Validating => messages.validating.clone(),
        Validity::Valid => messages.valid.clone(),
        Validity::Invalid(e) => e.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_replaces_the_hint() {
        let m = Messages::default();
        assert_eq!(feedback(&Validity::Idle, &m, Some("3+ chars")), (Some("3+ chars".into()), None));
        assert_eq!(feedback(&Validity::Invalid("Taken".into()), &m, Some("3+ chars")), (None, Some("Taken".into())));
        assert_eq!(feedback(&Validity::Valid, &m, None).0.as_deref(), Some("Looks good."));
        assert_eq!(live_text(&Validity::Idle, &m), "");
        assert_eq!(data_status(&Validity::Validating), "validating");
    }
}
