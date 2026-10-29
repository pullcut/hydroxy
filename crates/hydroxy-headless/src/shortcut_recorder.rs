//! Shortcut recorder connector. Anatomy: root, label, control, keys, key,
//! placeholder, clear, status. Every part carries `data-scope="shortcut-recorder"`
//! and its `data-part`. The control is a native `<button>`; `data-recording` is
//! set while it listens.

pub use hydroxy_machines::shortcut_recorder::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "shortcut-recorder";

/// Element ids for one recorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The label, which names the control.
    pub label: String,
    /// The control button.
    pub control: String,
    /// The status region, which describes the control.
    pub status: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { label: format!("{root}-label"), control: format!("{root}-control"), status: format!("{root}-status") }
}

/// Map a key on the control to an event. `Some` means: prevent default.
/// While listening, every key is captured except a bare Tab, so focus can still leave.
#[must_use]
pub fn control_keydown(press: &KeyPress, recording: bool) -> Option<ShortcutRecorderEvent> {
    let bare = !(press.ctrl || press.alt || press.shift || press.meta);
    if recording {
        if press.key == "Tab" && bare {
            return None;
        }
        return Some(ShortcutRecorderEvent::Press(press.clone()));
    }
    (bare && matches!(press.key.as_str(), "Enter" | " ")).then_some(ShortcutRecorderEvent::Record)
}

/// What the status region says.
#[must_use]
pub fn status_text(state: &ShortcutRecorderState, platform: Platform) -> String {
    if state.needs_modifier {
        let main = if platform == Platform::Mac { "Command" } else { "Control" };
        return format!("Include {main}, Alt or Option as well, or use a function key.");
    }
    if state.recording {
        return "Press the new shortcut. Escape cancels, Backspace clears.".into();
    }
    match &state.value.value {
        Some(v) => format!("Shortcut is {}.", spoken(v, platform)),
        None => "No shortcut set.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: &str) -> KeyPress {
        KeyPress { key: key.into(), ..KeyPress::default() }
    }

    #[test]
    fn idle_enter_and_space_start_recording() {
        assert_eq!(control_keydown(&press("Enter"), false), Some(ShortcutRecorderEvent::Record));
        assert_eq!(control_keydown(&press(" "), false), Some(ShortcutRecorderEvent::Record));
        assert_eq!(control_keydown(&press("k"), false), None);
    }

    #[test]
    fn recording_captures_everything_but_tab() {
        assert_eq!(control_keydown(&press("k"), true), Some(ShortcutRecorderEvent::Press(press("k"))));
        assert_eq!(control_keydown(&press("Enter"), true), Some(ShortcutRecorderEvent::Press(press("Enter"))));
        assert_eq!(control_keydown(&press("Tab"), true), None);
    }

    #[test]
    fn status_describes_the_state() {
        let mut s = ShortcutRecorderState::default();
        assert_eq!(status_text(&s, Platform::Mac), "No shortcut set.");
        s.value.value = Some("Mod+K".into());
        assert_eq!(status_text(&s, Platform::Mac), "Shortcut is Command plus K.");
        s.recording = true;
        assert!(status_text(&s, Platform::Mac).starts_with("Press"));
    }
}
