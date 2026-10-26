//! A small line-icon set for oxide components and demos.
//!
//! Paths are from Lucide (<https://lucide.dev>, ISC licence), drawn on a 24×24 grid
//! for a 1.75px stroke, matching the design system. Adapters render them with
//! [`SVG_ATTRS`] and [`Icon::markup`].

/// Attributes every icon `<svg>` carries, besides `width`/`height`.
pub const SVG_ATTRS: [(&str, &str); 7] = [
    ("viewBox", "0 0 24 24"),
    ("fill", "none"),
    ("stroke", "currentColor"),
    ("stroke-width", "1.75"),
    ("stroke-linecap", "round"),
    ("stroke-linejoin", "round"),
    ("aria-hidden", "true"),
];

/// Available icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// Rightward arrow.
    ArrowRight,
    /// Tick.
    Check,
    /// Downward chevron.
    ChevronDown,
    /// Rightward chevron.
    ChevronRight,
    /// Exclamation in a circle.
    CircleAlert,
    /// Two overlapping sheets.
    Copy,
    /// Three dots.
    Ellipsis,
    /// Page with lines.
    FileText,
    /// Tray.
    Inbox,
    /// "i" in a circle.
    Info,
    /// Light bulb.
    Lightbulb,
    /// Plus sign.
    Plus,
    /// Magnifying glass.
    Search,
    /// Bin.
    Trash,
    /// Exclamation in a triangle.
    TriangleAlert,
    /// Head and shoulders.
    User,
    /// Cross.
    X,
}

impl Icon {
    /// The inner SVG markup (paths, circles) for this icon.
    #[must_use]
    pub const fn markup(self) -> &'static str {
        match self {
            Icon::ArrowRight => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
            Icon::Check => r#"<path d="M20 6 9 17l-5-5"/>"#,
            Icon::ChevronDown => r#"<path d="m6 9 6 6 6-6"/>"#,
            Icon::ChevronRight => r#"<path d="m9 18 6-6-6-6"/>"#,
            Icon::CircleAlert => {
                r#"<circle cx="12" cy="12" r="10"/><line x1="12" x2="12" y1="8" y2="12"/><line x1="12" x2="12.01" y1="16" y2="16"/>"#
            }
            Icon::Copy => {
                r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#
            }
            Icon::Ellipsis => {
                r#"<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>"#
            }
            Icon::FileText => {
                r#"<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/>"#
            }
            Icon::Inbox => {
                r#"<polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>"#
            }
            Icon::Info => r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#,
            Icon::Lightbulb => {
                r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/><path d="M9 18h6"/><path d="M10 22h4"/>"#
            }
            Icon::Plus => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
            Icon::Search => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
            Icon::Trash => {
                r#"<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2"/>"#
            }
            Icon::TriangleAlert => {
                r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#
            }
            Icon::User => r#"<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#,
            Icon::X => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
        }
    }
}
