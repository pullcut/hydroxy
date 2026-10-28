//! Parsing, matching and showing shortcuts like `Mod+K` and `G I`.

use std::fmt;
use std::str::FromStr;

use crate::keys::{KeyPress, Platform, display, event_key};

/// A modifier in a shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    /// ⌘ on a Mac, Ctrl elsewhere.
    Mod,
    /// Control, on every platform.
    Ctrl,
    /// ⌘ on a Mac, the Windows key elsewhere.
    Meta,
    /// Alt, or Option on a Mac.
    Alt,
    /// Shift.
    Shift,
}

impl Modifier {
    /// In the order they are written.
    pub const ALL: [Self; 5] = [Self::Mod, Self::Ctrl, Self::Meta, Self::Alt, Self::Shift];

    fn bit(self) -> u8 {
        1 << self as u8
    }

    /// The portable name, as in `Mod+K`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Mod => "Mod",
            Self::Ctrl => "Ctrl",
            Self::Meta => "Meta",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
        }
    }

    fn parse(token: &str) -> Option<Self> {
        Some(match token.to_ascii_lowercase().as_str() {
            "mod" => Self::Mod,
            "ctrl" | "control" => Self::Ctrl,
            "meta" | "cmd" | "command" | "super" | "win" => Self::Meta,
            "alt" | "option" | "opt" => Self::Alt,
            "shift" => Self::Shift,
            _ => return None,
        })
    }
}

/// What a chord's main key matches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyMatch {
    /// The key as the layout labels it (`KeyboardEvent.key`), canonical form like `K`, `Space`, `ArrowUp`, `?`.
    Key(String),
    /// The physical key (`KeyboardEvent.code`), like `KeyK`, whatever the layout. Written `code:KeyK`.
    Code(String),
}

/// One key with its modifiers, like `Mod+Shift+K`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chord {
    mods: u8,
    /// The main key.
    pub key: KeyMatch,
}

/// Why a shortcut string did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Nothing but whitespace.
    Empty,
    /// `Mod++`: an empty part. Write the plus key as `Plus`.
    EmptyToken,
    /// Only modifiers, like `Mod+Shift`.
    MissingKey,
    /// A part before the last that is not a modifier, like `A+B`.
    NotAModifier(String),
    /// The same modifier twice.
    DuplicateModifier(Modifier),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("shortcut is empty"),
            Self::EmptyToken => f.write_str("empty part in shortcut; write the + key as Plus"),
            Self::MissingKey => f.write_str("shortcut has modifiers but no key"),
            Self::NotAModifier(t) => write!(f, "{t:?} is not a modifier; only the last part may be a key"),
            Self::DuplicateModifier(m) => write!(f, "modifier {} appears twice", m.name()),
        }
    }
}

impl std::error::Error for ParseError {}

fn canonical_key(token: &str) -> String {
    let named = match token.to_ascii_lowercase().as_str() {
        "esc" | "escape" => "Escape",
        "space" | "spacebar" => "Space",
        "plus" => "Plus",
        "enter" | "return" => "Enter",
        "tab" => "Tab",
        "backspace" => "Backspace",
        "delete" | "del" => "Delete",
        "insert" => "Insert",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "up" | "arrowup" => "ArrowUp",
        "down" | "arrowdown" => "ArrowDown",
        "left" | "arrowleft" => "ArrowLeft",
        "right" | "arrowright" => "ArrowRight",
        lower if lower.len() > 1 && lower.starts_with('f') && lower[1..].parse::<u8>().is_ok() => {
            return token.to_ascii_uppercase();
        }
        _ if token.chars().count() == 1 => return token.to_uppercase(),
        _ => return token.to_string(),
    };
    named.to_string()
}

/// A symbol like `?` already says Shift on most layouts, so a binding without
/// Shift should not fail just because Shift is down.
pub(crate) fn shift_implied(key: &str) -> bool {
    let mut chars = key.chars();
    matches!((chars.next(), chars.next()), (Some(c), None) if !c.is_alphanumeric())
}

impl Chord {
    /// A chord from modifiers and a key.
    #[must_use]
    pub fn new(modifiers: &[Modifier], key: KeyMatch) -> Self {
        Self { mods: modifiers.iter().fold(0, |acc, m| acc | m.bit()), key }
    }

    /// Does the chord include `modifier`?
    #[must_use]
    pub fn has(&self, modifier: Modifier) -> bool {
        self.mods & modifier.bit() != 0
    }

    /// The same chord with `Mod` swapped for the platform's own key, so `Mod+K`
    /// and `Ctrl+K` compare equal on Windows.
    #[must_use]
    pub fn resolve(&self, platform: Platform) -> Self {
        let mut mods = self.mods & !Modifier::Mod.bit();
        if self.has(Modifier::Mod) {
            mods |= if platform == Platform::Mac { Modifier::Meta.bit() } else { Modifier::Ctrl.bit() };
        }
        Self { mods, key: self.key.clone() }
    }

    /// Does `press` trigger this chord? Modifiers must match exactly, except
    /// that Shift may be down for a symbol key like `?`.
    ///
    /// ```
    /// use oxide_hotkeys::{Chord, KeyPress, Platform};
    /// let chord: Chord = "Mod+K".parse().unwrap();
    /// let press = KeyPress { key: "k".into(), meta: true, ..KeyPress::default() };
    /// assert!(chord.matches(&press, Platform::Mac));
    /// assert!(!chord.matches(&press, Platform::Other));
    /// ```
    #[must_use]
    pub fn matches(&self, press: &KeyPress, platform: Platform) -> bool {
        let r = self.resolve(platform);
        let key_ok = match &self.key {
            KeyMatch::Key(k) => event_key(press) == *k,
            KeyMatch::Code(c) => press.code == *c,
        };
        let shift_ok = press.shift == r.has(Modifier::Shift)
            || matches!(&self.key, KeyMatch::Key(k) if !r.has(Modifier::Shift) && shift_implied(k));
        key_ok
            && shift_ok
            && press.ctrl == r.has(Modifier::Ctrl)
            && press.meta == r.has(Modifier::Meta)
            && press.alt == r.has(Modifier::Alt)
    }

    /// Keycaps for this chord, like `["⌘", "K"]`.
    #[must_use]
    pub fn keys(&self, platform: Platform) -> Vec<String> {
        let mut caps = display(&self.to_string(), platform);
        if let (KeyMatch::Code(code), Some(last)) = (&self.key, caps.last_mut()) {
            let label = code.strip_prefix("Key").or_else(|| code.strip_prefix("Digit")).unwrap_or(code);
            *last = label.to_string();
        }
        caps
    }
}

impl FromStr for Chord {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        let tokens: Vec<&str> = s.split('+').map(str::trim).collect();
        if tokens.iter().any(|t| t.is_empty()) {
            return Err(ParseError::EmptyToken);
        }
        let (key, mods) = tokens.split_last().expect("split yields at least one token");
        if Modifier::parse(key).is_some() {
            return Err(ParseError::MissingKey);
        }
        let mut chord = Self::new(&[], KeyMatch::Key(String::new()));
        for token in mods {
            let m = Modifier::parse(token).ok_or_else(|| ParseError::NotAModifier((*token).to_string()))?;
            if chord.has(m) {
                return Err(ParseError::DuplicateModifier(m));
            }
            chord.mods |= m.bit();
        }
        chord.key = match key.strip_prefix("code:") {
            Some(code) if !code.is_empty() => KeyMatch::Code(code.to_string()),
            _ => KeyMatch::Key(canonical_key(key)),
        };
        Ok(chord)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for m in Modifier::ALL.into_iter().filter(|m| self.has(*m)) {
            write!(f, "{}+", m.name())?;
        }
        match &self.key {
            KeyMatch::Key(k) => f.write_str(k),
            KeyMatch::Code(c) => write!(f, "code:{c}"),
        }
    }
}

/// A shortcut: one chord like `Mod+K`, or a sequence like `G I` pressed one after another.
///
/// ```
/// use oxide_hotkeys::{Hotkey, Platform};
/// let save: Hotkey = "mod+s".parse().unwrap();
/// assert_eq!(save.to_string(), "Mod+S");
/// assert_eq!(save.label(Platform::Mac), "⌘S");
/// assert_eq!(save.label(Platform::Other), "Ctrl+S");
///
/// let inbox: Hotkey = "g i".parse().unwrap();
/// assert_eq!(inbox.chords().len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hotkey {
    chords: Vec<Chord>,
}

impl Hotkey {
    /// The chords, in the order they are pressed. Never empty.
    #[must_use]
    pub fn chords(&self) -> &[Chord] {
        &self.chords
    }

    /// Keycaps per chord, like `[["⌘", "K"]]` or `[["G"], ["I"]]`.
    #[must_use]
    pub fn keys(&self, platform: Platform) -> Vec<Vec<String>> {
        self.chords.iter().map(|c| c.keys(platform)).collect()
    }

    /// One line of text: `⌘⇧K` on a Mac, `Ctrl+Shift+K` elsewhere, chords separated by spaces.
    #[must_use]
    pub fn label(&self, platform: Platform) -> String {
        let join = if platform == Platform::Mac { "" } else { "+" };
        self.keys(platform).iter().map(|caps| caps.join(join)).collect::<Vec<_>>().join(" ")
    }

    /// Does this shortcut start with all of `other` (or equal it), once `Mod` is resolved?
    #[must_use]
    pub fn starts_with(&self, other: &Self, platform: Platform) -> bool {
        other.chords.len() <= self.chords.len()
            && other.chords.iter().zip(&self.chords).all(|(a, b)| a.resolve(platform) == b.resolve(platform))
    }
}

impl From<Chord> for Hotkey {
    fn from(chord: Chord) -> Self {
        Self { chords: vec![chord] }
    }
}

impl FromStr for Hotkey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let chords = s.split_whitespace().map(str::parse).collect::<Result<Vec<Chord>, _>>()?;
        if chords.is_empty() {
            return Err(ParseError::Empty);
        }
        Ok(Self { chords })
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, c) in self.chords.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: &str, mods: &str) -> KeyPress {
        KeyPress {
            key: key.into(),
            ctrl: mods.contains('c'),
            alt: mods.contains('a'),
            shift: mods.contains('s'),
            meta: mods.contains('m'),
            ..KeyPress::default()
        }
    }

    fn chord(s: &str) -> Chord {
        s.parse().unwrap()
    }

    #[test]
    fn parses_aliases_into_canonical_form() {
        assert_eq!(chord("cmd+shift+k").to_string(), "Meta+Shift+K");
        assert_eq!(chord("Shift+Mod+esc").to_string(), "Mod+Shift+Escape");
        assert_eq!(chord("ctrl+plus").to_string(), "Ctrl+Plus");
        assert_eq!(chord("f5").to_string(), "F5");
        assert_eq!(chord("Alt+code:KeyK").key, KeyMatch::Code("KeyK".into()));
        assert_eq!("g  i".parse::<Hotkey>().unwrap().to_string(), "G I");
        // The recorder's output parses as-is.
        assert_eq!(chord("Mod+Ctrl+Alt+Shift+ArrowUp").to_string(), "Mod+Ctrl+Alt+Shift+ArrowUp");
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!("".parse::<Hotkey>(), Err(ParseError::Empty));
        assert_eq!("Mod++".parse::<Chord>(), Err(ParseError::EmptyToken));
        assert_eq!("Mod+Shift".parse::<Chord>(), Err(ParseError::MissingKey));
        assert_eq!("A+B".parse::<Chord>(), Err(ParseError::NotAModifier("A".into())));
        assert_eq!("Shift+shift+A".parse::<Chord>(), Err(ParseError::DuplicateModifier(Modifier::Shift)));
    }

    #[test]
    fn mod_follows_the_platform() {
        let c = chord("Mod+K");
        assert!(c.matches(&press("k", "m"), Platform::Mac));
        assert!(!c.matches(&press("k", "c"), Platform::Mac));
        assert!(c.matches(&press("K", "c"), Platform::Other));
        assert!(!c.matches(&press("k", "cs"), Platform::Other), "extra Shift does not match");
        assert!(chord("Ctrl+K").matches(&press("k", "c"), Platform::Mac));
    }

    #[test]
    fn shift_is_implied_for_symbols_only() {
        assert!(chord("?").matches(&press("?", "s"), Platform::Other));
        assert!(chord("Shift+?").matches(&press("?", "s"), Platform::Other));
        assert!(!chord("K").matches(&press("K", "s"), Platform::Other));
    }

    #[test]
    fn code_matches_the_physical_key() {
        let c = chord("Alt+code:KeyK");
        let p = KeyPress { key: "˚".into(), code: "KeyK".into(), alt: true, ..KeyPress::default() };
        assert!(c.matches(&p, Platform::Mac));
        assert_eq!(c.keys(Platform::Mac), ["⌥", "K"]);
    }

    #[test]
    fn labels() {
        let h: Hotkey = "Mod+Shift+K".parse().unwrap();
        assert_eq!(h.label(Platform::Mac), "⌘⇧K");
        assert_eq!(h.label(Platform::Other), "Ctrl+Shift+K");
        let seq: Hotkey = "g i".parse().unwrap();
        assert_eq!(seq.label(Platform::Other), "G I");
        assert!(seq.starts_with(&"G".parse().unwrap(), Platform::Other));
        let ctrl: Hotkey = "Ctrl+K".parse().unwrap();
        assert!(ctrl.starts_with(&"Mod+K".parse().unwrap(), Platform::Other));
        assert!(!ctrl.starts_with(&"Mod+K".parse().unwrap(), Platform::Mac));
    }
}
