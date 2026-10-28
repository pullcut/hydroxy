//! Shortcut recorder owns a key combination. Start listening, press a chord,
//! and it is stored as a portable string such as `Mod+Shift+K`, where `Mod` is
//! ⌘ on a Mac and Ctrl elsewhere, ready for `hydroxy-hotkeys` to parse. Key
//! naming and display come from `hydroxy-hotkeys`; [`classify`] is the recorder's
//! own rule; the machine only tracks whether it is listening.

use hydroxy_statechart::{Machine, Transition};

pub use hydroxy_hotkeys::{KeyPress, Platform, display, key_name, spoken};

/// What a key press means to the recorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capture {
    /// A complete shortcut, in portable form.
    Combo(String),
    /// Esc on its own: stop listening.
    Cancel,
    /// Backspace or Delete on its own: remove the shortcut.
    Clear,
    /// A real key but no modifier (and not a function key).
    NeedsModifier,
    /// Only modifiers so far: keep listening.
    Ignore,
}

fn is_function_key(name: &str) -> bool {
    name.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()).is_some_and(|n| (1..=24).contains(&n))
}

/// Turn a key press into a [`Capture`]. Modifiers are normalised: the main
/// modifier becomes `Mod`, so a shortcut recorded on one platform reads right on the other.
#[must_use]
pub fn classify(press: &KeyPress, platform: Platform) -> Capture {
    let Some(key) = key_name(press) else { return Capture::Ignore };
    let any_mod = press.ctrl || press.alt || press.shift || press.meta;
    if !any_mod {
        match key.as_str() {
            "Escape" => return Capture::Cancel,
            "Backspace" | "Delete" => return Capture::Clear,
            _ => {}
        }
    }
    let (main, other) = match platform {
        Platform::Mac => (press.meta, press.ctrl),
        Platform::Other => (press.ctrl, press.meta),
    };
    if !(main || other || press.alt || is_function_key(&key)) {
        return Capture::NeedsModifier;
    }
    let mut parts = Vec::new();
    if main {
        parts.push("Mod");
    }
    if other {
        parts.push(if platform == Platform::Mac { "Ctrl" } else { "Meta" });
    }
    if press.alt {
        parts.push("Alt");
    }
    if press.shift {
        parts.push("Shift");
    }
    parts.push(&key);
    Capture::Combo(parts.join("+"))
}

/// Props for [`ShortcutRecorderMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShortcutRecorderInput {
    /// Controlled shortcut. `Some` at start fixes controlled mode; `Some("")` means none.
    pub value: Option<String>,
    /// Uncontrolled initial shortcut.
    pub default_value: Option<String>,
    /// Which keyboard convention to follow.
    pub platform: Platform,
    /// Block recording and clearing.
    pub disabled: bool,
}

/// What [`ShortcutRecorderMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShortcutRecorderState {
    /// The stored shortcut in portable form.
    pub value: Option<String>,
    /// Listening for a chord.
    pub recording: bool,
    /// The last chord had no modifier, so it was refused.
    pub needs_modifier: bool,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

/// Inputs to [`ShortcutRecorderMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutRecorderEvent {
    /// Click, Enter or Space on the control: start listening.
    Record,
    /// A key went down while listening.
    Press(KeyPress),
    /// The control lost focus: stop listening.
    Stop,
    /// The clear button.
    Clear,
}

/// Side effects from [`ShortcutRecorderMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutRecorderCommand {
    /// The shortcut changed (or should, when controlled). `None` means cleared.
    Change(Option<String>),
    /// Focus the control.
    FocusControl,
}

/// Listening state and stored shortcut.
#[derive(Debug, Clone, Copy)]
pub struct ShortcutRecorderMachine;

type Step = Transition<ShortcutRecorderState, ShortcutRecorderCommand>;

fn set_value(state: &ShortcutRecorderState, value: Option<String>) -> Step {
    let mut next = state.clone();
    next.recording = false;
    next.needs_modifier = false;
    let mut commands = Vec::new();
    if value != state.value {
        if !state.controlled {
            next.value.clone_from(&value);
        }
        commands.push(ShortcutRecorderCommand::Change(value));
    }
    Transition::with(next, commands)
}

impl Machine for ShortcutRecorderMachine {
    type Input = ShortcutRecorderInput;
    type State = ShortcutRecorderState;
    type Event = ShortcutRecorderEvent;
    type Command = ShortcutRecorderCommand;

    fn initial(input: &ShortcutRecorderInput) -> ShortcutRecorderState {
        let value = match &input.value {
            Some(v) => Some(v.clone()),
            None => input.default_value.clone(),
        };
        ShortcutRecorderState {
            value: value.filter(|v| !v.is_empty()),
            controlled: input.value.is_some(),
            ..ShortcutRecorderState::default()
        }
    }

    fn transition(state: &ShortcutRecorderState, event: ShortcutRecorderEvent, input: &ShortcutRecorderInput) -> Step {
        if input.disabled {
            return Transition::stay();
        }
        match (event, state.recording) {
            (ShortcutRecorderEvent::Record, false) => {
                let mut next = state.clone();
                next.recording = true;
                next.needs_modifier = false;
                Transition::to(next)
            }
            (ShortcutRecorderEvent::Press(press), true) => match classify(&press, input.platform) {
                Capture::Combo(combo) => set_value(state, Some(combo)),
                Capture::Clear => {
                    let mut step = set_value(state, None);
                    step.commands.push(ShortcutRecorderCommand::FocusControl);
                    step
                }
                Capture::Cancel => {
                    let mut next = state.clone();
                    next.recording = false;
                    next.needs_modifier = false;
                    Transition::with(next, vec![ShortcutRecorderCommand::FocusControl])
                }
                Capture::NeedsModifier => {
                    let mut next = state.clone();
                    next.needs_modifier = true;
                    Transition::to(next)
                }
                Capture::Ignore => Transition::stay(),
            },
            (ShortcutRecorderEvent::Stop, true) => {
                let mut next = state.clone();
                next.recording = false;
                next.needs_modifier = false;
                Transition::to(next)
            }
            (ShortcutRecorderEvent::Clear, _) if state.value.is_some() => {
                let mut step = set_value(state, None);
                step.commands.push(ShortcutRecorderCommand::FocusControl);
                step
            }
            _ => Transition::stay(),
        }
    }

    fn sync(state: &ShortcutRecorderState, input: &ShortcutRecorderInput) -> Option<ShortcutRecorderState> {
        let mut next = state.clone();
        if state.controlled {
            next.value = input.value.clone().filter(|v| !v.is_empty());
        }
        if input.disabled {
            next.recording = false;
            next.needs_modifier = false;
        }
        hydroxy_statechart::changed(state, next)
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

    #[test]
    fn main_modifier_is_normalised_per_platform() {
        assert_eq!(classify(&press("k", "m"), Platform::Mac), Capture::Combo("Mod+K".into()));
        assert_eq!(classify(&press("k", "c"), Platform::Other), Capture::Combo("Mod+K".into()));
        // The other one stays distinct.
        assert_eq!(classify(&press("k", "c"), Platform::Mac), Capture::Combo("Ctrl+K".into()));
        assert_eq!(classify(&press("k", "m"), Platform::Other), Capture::Combo("Meta+K".into()));
        assert_eq!(classify(&press("ArrowUp", "mas"), Platform::Mac), Capture::Combo("Mod+Alt+Shift+ArrowUp".into()));
    }

    #[test]
    fn code_beats_key_for_letters() {
        let p = KeyPress { key: "˚".into(), code: "KeyK".into(), alt: true, ..KeyPress::default() };
        assert_eq!(classify(&p, Platform::Mac), Capture::Combo("Alt+K".into()));
        assert_eq!(key_name(&KeyPress { key: " ".into(), ..KeyPress::default() }).as_deref(), Some("Space"));
    }

    #[test]
    fn special_keys_without_modifiers() {
        assert_eq!(classify(&press("Escape", ""), Platform::Other), Capture::Cancel);
        assert_eq!(classify(&press("Backspace", ""), Platform::Other), Capture::Clear);
        assert_eq!(classify(&press("Control", "c"), Platform::Other), Capture::Ignore);
        assert_eq!(classify(&press("k", ""), Platform::Other), Capture::NeedsModifier);
        assert_eq!(classify(&press("K", "s"), Platform::Other), Capture::NeedsModifier);
        assert_eq!(classify(&press("F5", ""), Platform::Other), Capture::Combo("F5".into()));
        // Esc with a modifier is a shortcut, not a cancel.
        assert_eq!(classify(&press("Escape", "c"), Platform::Other), Capture::Combo("Mod+Escape".into()));
    }

    #[test]
    fn display_follows_the_platform() {
        assert_eq!(display("Mod+Shift+K", Platform::Mac), ["⌘", "⇧", "K"]);
        assert_eq!(display("Mod+Shift+K", Platform::Other), ["Ctrl", "Shift", "K"]);
        assert_eq!(display("Ctrl+Alt+ArrowLeft", Platform::Mac), ["⌃", "⌥", "←"]);
        assert_eq!(display("Meta+Escape", Platform::Other), ["Win", "Esc"]);
        assert_eq!(spoken("Mod+Alt+K", Platform::Mac), "Command plus Option plus K");
    }

    #[test]
    fn detect_platform() {
        assert_eq!(Platform::detect("MacIntel"), Platform::Mac);
        assert_eq!(Platform::detect("Win32"), Platform::Other);
    }

    fn input() -> ShortcutRecorderInput {
        ShortcutRecorderInput { platform: Platform::Mac, ..ShortcutRecorderInput::default() }
    }

    fn run(
        state: &ShortcutRecorderState,
        event: ShortcutRecorderEvent,
        input: &ShortcutRecorderInput,
    ) -> hydroxy_statechart::Step<ShortcutRecorderState, ShortcutRecorderCommand> {
        ShortcutRecorderMachine::transition(state, event, input).resolve(state)
    }

    fn recording(i: &ShortcutRecorderInput) -> ShortcutRecorderState {
        run(&ShortcutRecorderMachine::initial(i), ShortcutRecorderEvent::Record, i).state
    }

    #[test]
    fn records_a_chord_and_stops() {
        let i = input();
        let step = run(&recording(&i), ShortcutRecorderEvent::Press(press("k", "ms")), &i);
        assert!(!step.state.recording);
        assert_eq!(step.state.value.as_deref(), Some("Mod+Shift+K"));
        assert_eq!(step.commands, [ShortcutRecorderCommand::Change(Some("Mod+Shift+K".into()))]);
    }

    #[test]
    fn modifier_only_keeps_listening_and_bare_keys_are_refused() {
        let i = input();
        let s = recording(&i);
        let s = run(&s, ShortcutRecorderEvent::Press(press("Meta", "m")), &i).state;
        assert!(s.recording && !s.needs_modifier);
        let s = run(&s, ShortcutRecorderEvent::Press(press("k", "")), &i).state;
        assert!(s.recording && s.needs_modifier);
    }

    #[test]
    fn escape_cancels_and_keeps_the_old_value() {
        let i = ShortcutRecorderInput { default_value: Some("Mod+J".into()), ..input() };
        let s = recording(&i);
        let step = run(&s, ShortcutRecorderEvent::Press(press("Escape", "")), &i);
        assert!(!step.state.recording);
        assert_eq!(step.state.value.as_deref(), Some("Mod+J"));
        assert_eq!(step.commands, [ShortcutRecorderCommand::FocusControl]);
    }

    #[test]
    fn clear_button_and_backspace_remove_the_value() {
        let i = ShortcutRecorderInput { default_value: Some("Mod+J".into()), ..input() };
        let step = run(&ShortcutRecorderMachine::initial(&i), ShortcutRecorderEvent::Clear, &i);
        assert_eq!(step.state.value, None);
        assert_eq!(step.commands, [ShortcutRecorderCommand::Change(None), ShortcutRecorderCommand::FocusControl]);
        let step = run(&recording(&i), ShortcutRecorderEvent::Press(press("Backspace", "")), &i);
        assert_eq!(step.state.value, None);
    }

    #[test]
    fn controlled_reports_without_changing() {
        let i = ShortcutRecorderInput { value: Some(String::new()), ..input() };
        let step = run(&recording(&i), ShortcutRecorderEvent::Press(press("k", "m")), &i);
        assert_eq!(step.state.value, None);
        assert_eq!(step.commands, [ShortcutRecorderCommand::Change(Some("Mod+K".into()))]);
        let i2 = ShortcutRecorderInput { value: Some("Mod+K".into()), ..input() };
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<ShortcutRecorderMachine>(&step.state, &i2).value.as_deref(),
            Some("Mod+K")
        );
    }

    #[test]
    fn disabled_ignores_everything() {
        let i = ShortcutRecorderInput { disabled: true, ..input() };
        let step = run(&ShortcutRecorderMachine::initial(&i), ShortcutRecorderEvent::Record, &i);
        assert!(!step.state.recording);
    }
}
