//! Keyboard shortcuts for oxide.
//!
//! Write shortcuts as strings: `Mod+K` (⌘ on a Mac, Ctrl elsewhere),
//! `Shift+?`, `Alt+code:KeyK` (a physical key) or `G I` (a sequence). A
//! [`Registry`] holds every live shortcut, picks the one that wins each key
//! press (innermost `within` element, then priority, then registration order),
//! skips typing in inputs unless asked, and lists everything for a cheat sheet.
//!
//! The core has no dependencies. The `leptos` and `dioxus` features add
//! `use_hotkey` hooks that run one page-wide registry.
//!
//! ```
//! use oxide_hotkeys::{Hotkey, Platform};
//! let search: Hotkey = "Mod+K".parse().unwrap();
//! assert_eq!(search.label(Platform::Mac), "⌘K");
//! assert_eq!(search.label(Platform::Other), "Ctrl+K");
//! ```

mod hotkey;
mod keys;
mod registry;

#[cfg(feature = "dioxus")]
pub mod dioxus;
#[cfg(feature = "leptos")]
pub mod leptos;

pub use hotkey::{Chord, Hotkey, KeyMatch, Modifier, ParseError};
pub use keys::{KeyPress, Platform, display, is_modifier_key, key_name, spoken};
pub use registry::{Armed, Dispatch, HotkeyId, HotkeyInfo, HotkeyOptions, KeyEvent, Registry};

/// Selector for "focus is somewhere you type". Adapters test it with `closest`.
pub const EDITABLE_SELECTOR: &str = "input, textarea, select, [contenteditable]:not([contenteditable=false]), \
     [role=textbox], [role=searchbox], [role=combobox], [role=spinbutton]";
