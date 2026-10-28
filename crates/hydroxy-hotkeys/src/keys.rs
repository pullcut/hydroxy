//! Key presses, platforms and how keys are named and shown. Shared with
//! `hydroxy-machines`' shortcut recorder, so a recorded `Mod+Shift+K` is exactly
//! the string a hotkey parses.

/// Which keyboard convention to follow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Platform {
    /// ⌘ is the main modifier, written ⌘ ⌥ ⇧ ⌃.
    Mac,
    /// Ctrl is the main modifier, written Ctrl, Alt, Shift.
    #[default]
    Other,
}

impl Platform {
    /// Guess from `navigator.platform` or a user agent string.
    ///
    /// ```
    /// use hydroxy_hotkeys::Platform;
    /// assert_eq!(Platform::detect("MacIntel"), Platform::Mac);
    /// ```
    #[must_use]
    pub fn detect(navigator_platform: &str) -> Self {
        let p = navigator_platform.to_ascii_lowercase();
        if p.contains("mac") || p.contains("iphone") || p.contains("ipad") { Self::Mac } else { Self::Other }
    }
}

/// A key press as the browser reports it.
#[allow(clippy::struct_excessive_bools, reason = "one flag per modifier key, mirroring `KeyboardEvent`")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyPress {
    /// `KeyboardEvent.key`.
    pub key: String,
    /// `KeyboardEvent.code`. Optional, but it keeps Option+K from turning into a symbol on a Mac.
    pub code: String,
    /// Ctrl is down.
    pub ctrl: bool,
    /// Alt (Option) is down.
    pub alt: bool,
    /// Shift is down.
    pub shift: bool,
    /// Meta (⌘ or the Windows key) is down.
    pub meta: bool,
}

/// Is `key` (a `KeyboardEvent.key`) a modifier on its own, like `Shift`?
#[must_use]
pub fn is_modifier_key(key: &str) -> bool {
    matches!(key, "Control" | "Alt" | "Shift" | "Meta" | "AltGraph" | "OS" | "Hyper" | "Super")
}

fn code_char(code: &str) -> Option<&str> {
    code.strip_prefix("Key").or_else(|| code.strip_prefix("Digit")).filter(|c| c.len() == 1)
}

/// The portable name of the main key, or `None` for a modifier key. Letters
/// and digits come from the physical `code`, which is what the shortcut
/// recorder stores.
#[must_use]
pub fn key_name(press: &KeyPress) -> Option<String> {
    if is_modifier_key(&press.key) {
        return None;
    }
    if let Some(c) = code_char(&press.code) {
        return Some(c.to_string());
    }
    Some(match press.key.as_str() {
        " " => "Space".to_string(),
        k if k.chars().count() == 1 => k.to_uppercase(),
        k => k.to_string(),
    })
}

/// The name a hotkey matches against. Unlike [`key_name`] this follows the
/// layout (`key`), so Mod+Z means the key labelled Z on AZERTY too. It falls back
/// to `code` only when the layout turned the key into a symbol, like Option+K → `˚`.
#[must_use]
pub(crate) fn event_key(press: &KeyPress) -> String {
    let k = press.key.as_str();
    match k {
        " " => return "Space".into(),
        "+" => return "Plus".into(),
        _ => {}
    }
    let mut chars = k.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if !c.is_ascii() && press.alt => {
            code_char(&press.code).map_or_else(|| k.to_uppercase(), str::to_string)
        }
        (Some(_), None) => k.to_uppercase(),
        _ => k.to_string(),
    }
}

/// The keys to show for a portable combo, one string per keycap.
///
/// ```
/// use hydroxy_hotkeys::{display, Platform};
/// assert_eq!(display("Mod+Shift+K", Platform::Mac), ["⌘", "⇧", "K"]);
/// assert_eq!(display("Mod+Shift+K", Platform::Other), ["Ctrl", "Shift", "K"]);
/// ```
#[must_use]
pub fn display(combo: &str, platform: Platform) -> Vec<String> {
    combo
        .split('+')
        .map(|part| {
            let mac = platform == Platform::Mac;
            match (part, mac) {
                ("Mod" | "Meta", true) => "⌘",
                ("Mod", false) => "Ctrl",
                ("Ctrl", _) => {
                    if mac {
                        "⌃"
                    } else {
                        "Ctrl"
                    }
                }
                ("Meta", false) => "Win",
                ("Alt", true) => "⌥",
                ("Shift", true) => "⇧",
                ("ArrowUp", _) => "↑",
                ("ArrowDown", _) => "↓",
                ("ArrowLeft", _) => "←",
                ("ArrowRight", _) => "→",
                ("Enter", true) => "↵",
                ("Backspace", true) => "⌫",
                ("Delete", true) => "⌦",
                ("Tab", true) => "⇥",
                ("Escape", _) => "Esc",
                ("Plus", _) => "+",
                (other, _) => other,
            }
            .to_string()
        })
        .collect()
}

/// A spoken form of a combo, for `aria-label`s: "Command plus Shift plus K".
#[must_use]
pub fn spoken(combo: &str, platform: Platform) -> String {
    combo
        .split('+')
        .map(|part| match (part, platform) {
            ("Mod" | "Meta", Platform::Mac) => "Command",
            ("Mod", Platform::Other) | ("Ctrl", _) => "Control",
            ("Alt", Platform::Mac) => "Option",
            ("Meta", _) => "Windows",
            (other, _) => other,
        })
        .collect::<Vec<_>>()
        .join(" plus ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kp(key: &str, code: &str) -> KeyPress {
        KeyPress { key: key.into(), code: code.into(), ..KeyPress::default() }
    }

    #[test]
    fn event_key_follows_layout_then_code() {
        assert_eq!(event_key(&kp("k", "KeyK")), "K");
        // AZERTY: the key labelled A sits where QWERTY has Q.
        assert_eq!(event_key(&kp("a", "KeyQ")), "A");
        // Option+K on a Mac.
        let option_k = KeyPress { alt: true, ..kp("˚", "KeyK") };
        assert_eq!(event_key(&option_k), "K");
        assert_eq!(event_key(&kp("é", "Digit2")), "É");
        assert_eq!(event_key(&kp("?", "Slash")), "?");
        assert_eq!(event_key(&kp(" ", "Space")), "Space");
        assert_eq!(event_key(&kp("+", "Equal")), "Plus");
        assert_eq!(event_key(&kp("ArrowUp", "ArrowUp")), "ArrowUp");
    }

    #[test]
    fn display_and_spoken() {
        assert_eq!(display("Ctrl+Alt+ArrowLeft", Platform::Mac), ["⌃", "⌥", "←"]);
        assert_eq!(display("Meta+Escape", Platform::Other), ["Win", "Esc"]);
        assert_eq!(display("Meta+K", Platform::Mac), ["⌘", "K"]);
        assert_eq!(spoken("Mod+Alt+K", Platform::Mac), "Command plus Option plus K");
        assert_eq!(Platform::detect("Win32"), Platform::Other);
    }
}
