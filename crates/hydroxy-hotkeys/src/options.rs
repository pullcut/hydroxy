//! [`HotkeyOptions`]: how one registered shortcut behaves.

use std::time::Duration;

/// How a registered shortcut behaves. Build with the methods; the defaults
/// suit most app shortcuts.
///
/// ```
/// use hydroxy_hotkeys::HotkeyOptions;
/// let options = HotkeyOptions::new().description("Save").group("File").in_inputs(true);
/// assert!(options.prevent_default);
/// ```
#[allow(clippy::struct_excessive_bools, reason = "independent on/off switches")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyOptions {
    /// What it does, for the cheat sheet.
    pub description: Option<String>,
    /// A heading to file it under in the cheat sheet.
    pub group: Option<String>,
    /// A named scope; it fires only while that scope is active. `None` is global.
    pub scope: Option<String>,
    /// A CSS selector; it fires only while focus is inside a matching element.
    /// The innermost match beats shortcuts further out and global ones.
    pub within: Option<String>,
    /// Off means registered (and listed) but never fires. Default on.
    pub enabled: bool,
    /// Call `preventDefault` when it fires, or when a sequence advances. Default on.
    pub prevent_default: bool,
    /// Call `stopPropagation` when it fires. Default off.
    pub stop_propagation: bool,
    /// Fire while typing in an input, textarea, select or contenteditable. Default off.
    pub in_inputs: bool,
    /// Fire again on auto-repeat while the key is held. Default off.
    pub repeat: bool,
    /// Higher wins when two shortcuts match at the same depth. Default 0.
    pub priority: i32,
    /// Longest gap between the chords of a sequence. Default one second.
    pub timeout: Duration,
}

impl Default for HotkeyOptions {
    fn default() -> Self {
        Self {
            description: None,
            group: None,
            scope: None,
            within: None,
            enabled: true,
            prevent_default: true,
            stop_propagation: false,
            in_inputs: false,
            repeat: false,
            priority: 0,
            timeout: Duration::from_secs(1),
        }
    }
}

impl HotkeyOptions {
    /// The defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Set [`description`](Self::description).
    #[must_use]
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    /// Set [`group`](Self::group).
    #[must_use]
    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.group = Some(name.into());
        self
    }
    /// Set [`scope`](Self::scope).
    #[must_use]
    pub fn scope(mut self, name: impl Into<String>) -> Self {
        self.scope = Some(name.into());
        self
    }
    /// Set [`within`](Self::within).
    #[must_use]
    pub fn within(mut self, selector: impl Into<String>) -> Self {
        self.within = Some(selector.into());
        self
    }
    /// Set [`enabled`](Self::enabled).
    #[must_use]
    pub fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
        self
    }
    /// Set [`prevent_default`](Self::prevent_default).
    #[must_use]
    pub fn prevent_default(mut self, on: bool) -> Self {
        self.prevent_default = on;
        self
    }
    /// Set [`stop_propagation`](Self::stop_propagation).
    #[must_use]
    pub fn stop_propagation(mut self, on: bool) -> Self {
        self.stop_propagation = on;
        self
    }
    /// Set [`in_inputs`](Self::in_inputs).
    #[must_use]
    pub fn in_inputs(mut self, on: bool) -> Self {
        self.in_inputs = on;
        self
    }
    /// Set [`repeat`](Self::repeat).
    #[must_use]
    pub fn repeat(mut self, on: bool) -> Self {
        self.repeat = on;
        self
    }
    /// Set [`priority`](Self::priority).
    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    /// Set [`timeout`](Self::timeout).
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
