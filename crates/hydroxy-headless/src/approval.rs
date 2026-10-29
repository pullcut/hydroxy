//! Shared vocabulary for the approval card: where a request stands and how risky it is.

use crate::badge::Tone;

/// Ids derived from one root id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The title, which labels the card.
    pub title: String,
    /// The reason box.
    pub reason: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { title: format!("{root}-title"), reason: format!("{root}-reason") }
}

/// Where an approval request stands. The owner holds it and updates it from
/// `on_approve` / `on_reject`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ApprovalStatus {
    /// Waiting for a decision. Shows the buttons.
    #[default]
    Pending,
    /// Approved by `by` at `at`, such as "Jo Marsh" and "26 Oct, 14:02".
    Approved {
        /// Who decided.
        by: String,
        /// When, as display text.
        at: String,
    },
    /// Rejected by `by` at `at`, with an optional reason.
    Rejected {
        /// Who decided.
        by: String,
        /// When, as display text.
        at: String,
        /// Why, if given.
        reason: Option<String>,
    },
}

impl ApprovalStatus {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            ApprovalStatus::Pending => "pending",
            ApprovalStatus::Approved { .. } => "approved",
            ApprovalStatus::Rejected { .. } => "rejected",
        }
    }
}

/// How much damage the action could do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Risk {
    /// Easy to undo.
    Low,
    /// Worth a second look.
    Medium,
    /// Hard or impossible to undo.
    High,
}

impl Risk {
    /// The badge text.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Risk::Low => "Low risk",
            Risk::Medium => "Medium risk",
            Risk::High => "High risk",
        }
    }

    /// The badge tone.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Risk::Low => Tone::Neutral,
            Risk::Medium => Tone::Warning,
            Risk::High => Tone::Danger,
        }
    }
}
