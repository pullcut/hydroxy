//! Shared button vocabulary for the `Button` and `IconButton` adapters, and for any
//! part that opts into the button look (dialog and menu triggers).

/// The class that gives a `<button>` the hydroxy button look.
pub const BUTTON_CLASS: &str = "ox-button";

/// Visual weight. Use one primary or accent button per view.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Ink fill. The main action.
    #[default]
    Primary,
    /// Amethyst fill. The single most important action.
    Accent,
    /// Bordered, transparent.
    Secondary,
    /// Borderless, transparent.
    Ghost,
    /// Garnet fill, for destructive actions.
    Danger,
}

impl ButtonVariant {
    /// The `data-variant` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Accent => "accent",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Danger => "danger",
        }
    }
}

/// Height and type size: 32, 40 or 52px for buttons, 32, 40 or 48px for icon buttons.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Small.
    Sm,
    /// Medium.
    #[default]
    Md,
    /// Large.
    Lg,
}

impl ButtonSize {
    /// The `data-size` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ButtonSize::Sm => "sm",
            ButtonSize::Md => "md",
            ButtonSize::Lg => "lg",
        }
    }

    /// Icon size inside a an hydroxy `Button`.
    #[must_use]
    pub const fn icon_px(self) -> u16 {
        match self {
            ButtonSize::Sm => 15,
            ButtonSize::Md => 17,
            ButtonSize::Lg => 19,
        }
    }

    /// Icon size inside an icon button: 45% of its side.
    #[must_use]
    pub const fn icon_button_px(self) -> u16 {
        match self {
            ButtonSize::Sm => 14,
            ButtonSize::Md => 18,
            ButtonSize::Lg => 22,
        }
    }
}

/// The `type` attribute.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonType {
    /// Does nothing by itself.
    #[default]
    Button,
    /// Submits its form.
    Submit,
    /// Resets its form.
    Reset,
}

impl ButtonType {
    /// The `type` attribute value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ButtonType::Button => "button",
            ButtonType::Submit => "submit",
            ButtonType::Reset => "reset",
        }
    }
}

/// The class that gives a `<button>` the hydroxy icon-button look.
pub const ICON_BUTTON_CLASS: &str = "ox-icon-button";
