//! The six jewel colours of the oxide palette.

/// One of the six jewel hues, in mark order. Used by `Tag`, `Chip`, `Avatar`, `Card` and `ProgressBar`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Jewel {
    /// Red.
    Garnet,
    /// Magenta.
    Rhodolite,
    /// Yellow. Takes ink text on full-colour surfaces.
    Citrine,
    /// Violet, the accent.
    #[default]
    Amethyst,
    /// Blue.
    Sapphire,
    /// Green.
    Emerald,
}

impl Jewel {
    /// The `data-jewel` value the styles key off.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Jewel::Garnet => "garnet",
            Jewel::Rhodolite => "rhodolite",
            Jewel::Citrine => "citrine",
            Jewel::Amethyst => "amethyst",
            Jewel::Sapphire => "sapphire",
            Jewel::Emerald => "emerald",
        }
    }
}
