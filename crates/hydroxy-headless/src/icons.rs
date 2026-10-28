//! A small line-icon set for hydroxy components and demos.
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
    /// Upward arrow.
    ArrowUp,
    /// Rightward arrow.
    ArrowRight,
    /// Notification bell.
    Bell,
    /// Desktop screen.
    Monitor,
    /// Leftward chevron.
    ChevronLeft,
    /// Folder.
    Folder,
    /// Six dots in two columns, a drag handle.
    Grip,
    /// Microphone.
    Mic,
    /// Two vertical bars.
    Pause,
    /// Play triangle, outlined.
    Play,
    /// Play triangle, solid.
    PlayFilled,
    /// Arrow curling back to the start.
    Replay,
    /// Arrow turning anticlockwise.
    RotateCcw,
    /// Arrow turning clockwise.
    RotateCw,
    /// Smiling face with a plus.
    SmilePlus,
    /// Rounded square.
    Stop,
    /// Speaker with sound waves.
    Volume,
    /// Speaker with a cross.
    VolumeOff,
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
    /// Arrow into a tray.
    Download,
    /// Three dots.
    Ellipsis,
    /// Open eye.
    Eye,
    /// Eye struck through.
    EyeOff,
    /// Page with lines.
    FileText,
    /// Tray.
    Inbox,
    /// "i" in a circle.
    Info,
    /// Light bulb.
    Lightbulb,
    /// The hydroxy mark: a thick ring. Colour it with `color`; used as the assistant's avatar.
    Mark,
    /// Crescent moon.
    Moon,
    /// Plus sign.
    Plus,
    /// Magnifying glass.
    Search,
    /// Five-pointed star.
    Star,
    /// Sun with rays.
    Sun,
    /// Bin.
    Trash,
    /// Exclamation in a triangle.
    TriangleAlert,
    /// Head and shoulders.
    User,
    /// Cross.
    X,
    /// Three horizontal lines, a menu.
    Menu,
    /// Microphone struck through.
    MicOff,
    /// Wrench.
    Wrench,
    /// Speech bubble.
    MessageSquare,
    /// Nested squares.
    Layers,
}

impl Icon {
    /// The inner SVG markup (paths, circles) for this icon.
    #[must_use]
    pub const fn markup(self) -> &'static str {
        match self {
            Icon::ArrowUp => r#"<path d="m5 12 7-7 7 7"/><path d="M12 19V5"/>"#,
            Icon::ArrowRight => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
            Icon::Bell => {
                r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>"#
            }
            Icon::Monitor => {
                r#"<rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>"#
            }
            Icon::ChevronLeft => r#"<path d="m15 18-6-6 6-6"/>"#,
            Icon::Folder => {
                r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>"#
            }
            Icon::Grip => {
                r#"<circle cx="9" cy="6" r="1.6" fill="currentColor" stroke="none"/><circle cx="15" cy="6" r="1.6" fill="currentColor" stroke="none"/><circle cx="9" cy="12" r="1.6" fill="currentColor" stroke="none"/><circle cx="15" cy="12" r="1.6" fill="currentColor" stroke="none"/><circle cx="9" cy="18" r="1.6" fill="currentColor" stroke="none"/><circle cx="15" cy="18" r="1.6" fill="currentColor" stroke="none"/>"#
            }
            Icon::Mic => {
                r#"<path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" x2="12" y1="19" y2="22"/>"#
            }
            Icon::Pause => {
                r#"<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>"#
            }
            Icon::Play => r#"<polygon points="6 3 20 12 6 21 6 3"/>"#,
            Icon::PlayFilled => r#"<path d="M8 5.5v13l10.5-6.5z" fill="currentColor" stroke="none"/>"#,
            Icon::Replay => r#"<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>"#,
            Icon::RotateCcw => r#"<path d="M4 9a8 8 0 1 1 1.5 8M4 4v5h5"/>"#,
            Icon::RotateCw => r#"<path d="M20 9a8 8 0 1 0-1.5 8M20 4v5h-5"/>"#,
            Icon::SmilePlus => {
                r#"<circle cx="11" cy="12" r="8"/><path d="M8 14.5c.8 1 1.8 1.5 3 1.5s2.2-.5 3-1.5"/><path d="M8.5 9.5h.01M13.5 9.5h.01"/><path d="M20 3v5M17.5 5.5h5"/>"#
            }
            Icon::Stop => r#"<rect x="6" y="6" width="12" height="12" rx="2"/>"#,
            Icon::Volume => {
                r#"<path d="M11 5 6 9H2v6h4l5 4V5Z"/><path d="M15.5 8.5a5 5 0 0 1 0 7"/><path d="M19 5a10 10 0 0 1 0 14"/>"#
            }
            Icon::VolumeOff => {
                r#"<path d="M11 5 6 9H2v6h4l5 4V5Z"/><line x1="22" x2="16" y1="9" y2="15"/><line x1="16" x2="22" y1="9" y2="15"/>"#
            }
            Icon::Check => r#"<path d="M20 6 9 17l-5-5"/>"#,
            Icon::ChevronDown => r#"<path d="m6 9 6 6 6-6"/>"#,
            Icon::ChevronRight => r#"<path d="m9 18 6-6-6-6"/>"#,
            Icon::CircleAlert => {
                r#"<circle cx="12" cy="12" r="10"/><line x1="12" x2="12" y1="8" y2="12"/><line x1="12" x2="12.01" y1="16" y2="16"/>"#
            }
            Icon::Copy => {
                r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#
            }
            Icon::Download => {
                r#"<path d="M12 15V3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/>"#
            }
            Icon::Ellipsis => {
                r#"<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>"#
            }
            Icon::Eye => {
                r#"<path d="M2.06 12.35a1 1 0 0 1 0-.7 10.75 10.75 0 0 1 19.88 0 1 1 0 0 1 0 .7 10.75 10.75 0 0 1-19.88 0"/><circle cx="12" cy="12" r="3"/>"#
            }
            Icon::EyeOff => {
                r#"<path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c4.94 0 8.93 3 10 7a10.8 10.8 0 0 1-1.42 2.61"/><path d="M14.08 14.16a3 3 0 0 1-4.24-4.24"/><path d="M17.48 17.5A10.75 10.75 0 0 1 2 12a10.8 10.8 0 0 1 3.72-5.06"/><path d="m2 2 20 20"/>"#
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
            Icon::Mark => r#"<circle cx="12" cy="12" r="7.5" stroke-width="5"/>"#,
            Icon::Moon => r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#,
            Icon::Plus => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
            Icon::Search => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
            Icon::Star => {
                r#"<path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"/>"#
            }
            Icon::Sun => {
                r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>"#
            }
            Icon::Trash => {
                r#"<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2"/>"#
            }
            Icon::TriangleAlert => {
                r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#
            }
            Icon::User => r#"<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#,
            Icon::X => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
            Icon::Menu => {
                r#"<line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="18" y2="18"/>"#
            }
            Icon::MicOff => {
                r#"<line x1="2" x2="22" y1="2" y2="22"/><path d="M18.89 13.23A7.12 7.12 0 0 0 19 12v-2"/><path d="M5 10v2a7 7 0 0 0 12 5"/><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12"/><line x1="12" x2="12" y1="19" y2="22"/>"#
            }
            Icon::Wrench => {
                r#"<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.8-3.8a6 6 0 0 1-7.9 7.9l-6.9 6.9a2.1 2.1 0 0 1-3-3l6.9-6.9a6 6 0 0 1 7.9-7.9l-3.8 3.8Z"/>"#
            }
            Icon::MessageSquare => r#"<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2Z"/>"#,
            Icon::Layers => r#"<rect x="3" y="3" width="13" height="13" rx="2"/><path d="M8 21h11a2 2 0 0 0 2-2V8"/>"#,
        }
    }
}
