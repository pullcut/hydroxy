//! Select: a native `<select>` styled like Input. Shares Input's id and aria wiring.

/// The `data-scope` value on every select part.
pub const SCOPE: &str = "select";

/// One choice in a select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    /// Submitted and reported value.
    pub value: String,
    /// Visible text.
    pub label: String,
}

impl SelectOption {
    /// An option with its own label.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }
}
