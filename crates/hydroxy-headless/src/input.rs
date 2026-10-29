//! Text field connector: ids and ARIA wiring between the label, the native
//! `<input>`, and its hint or error. No machine: the native input owns its value.

/// The `data-scope` value on every input part.
pub const SCOPE: &str = "input";

/// Field height. Shared by `Input` and `SearchField`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Size {
    /// 36px.
    Sm,
    /// 44px.
    #[default]
    Md,
    /// 52px, for page-level search.
    Lg,
}

impl Size {
    /// The `data-size` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Size::Sm => "sm",
            Size::Md => "md",
            Size::Lg => "lg",
        }
    }
}

/// Ids for the parts of one field, all derived from the root id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldIds {
    /// The `<input>`; the label's `for`.
    pub control: String,
    /// The hint paragraph.
    pub hint: String,
    /// The error paragraph.
    pub error: String,
}

/// Ids for a field rooted at `root`.
#[must_use]
pub fn ids(root: &str) -> FieldIds {
    FieldIds { control: format!("{root}-control"), hint: format!("{root}-hint"), error: format!("{root}-error") }
}

/// ARIA attributes for the `<input>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlAttrs {
    /// The error when there is one, else the hint, else nothing.
    pub aria_describedby: Option<String>,
    /// `"true"` while there is an error.
    pub aria_invalid: Option<&'static str>,
}

/// ARIA for the control. An error replaces the hint, so only one is described.
#[must_use]
pub fn control_attrs(ids: &FieldIds, has_hint: bool, has_error: bool) -> ControlAttrs {
    let aria_describedby = if has_error {
        Some(ids.error.clone())
    } else if has_hint {
        Some(ids.hint.clone())
    } else {
        None
    };
    ControlAttrs { aria_describedby, aria_invalid: has_error.then_some("true") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_replaces_hint_and_marks_invalid() {
        let ids = ids("f");
        let attrs = control_attrs(&ids, true, true);
        assert_eq!(attrs.aria_describedby.as_deref(), Some("f-error"));
        assert_eq!(attrs.aria_invalid, Some("true"));
    }

    #[test]
    fn hint_alone_describes_without_invalid() {
        let ids = ids("f");
        let attrs = control_attrs(&ids, true, false);
        assert_eq!(attrs.aria_describedby.as_deref(), Some("f-hint"));
        assert_eq!(attrs.aria_invalid, None);
        assert_eq!(control_attrs(&ids, false, false).aria_describedby, None);
    }
}
