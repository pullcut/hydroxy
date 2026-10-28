//! `ShortcutRecorder`: click it, press a key combination, and it shows the
//! result as keycaps. Esc cancels, Backspace (or the clear button) removes it.
//! Shortcuts are stored portably (`Mod+Shift+K`) and shown per platform.

use dioxus::prelude::*;
use hydroxy_headless::shortcut_recorder::{
    self as h, KeyPress, Platform, ShortcutRecorderCommand, ShortcutRecorderEvent, ShortcutRecorderInput,
    ShortcutRecorderMachine, ShortcutRecorderState,
};
use hydroxy_statechart::Machine;

use crate::focus::FocusMap;
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::kbd::Kbd;

pub use hydroxy_headless::shortcut_recorder::{Platform as ShortcutPlatform, display, spoken};

const CONTROL: &str = "control";

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<ShortcutRecorderState>,
    input: Memo<ShortcutRecorderInput>,
    focus: FocusMap,
    on_change: Option<EventHandler<Option<String>>>,
}

impl Ctx {
    fn send(mut self, event: ShortcutRecorderEvent) {
        let step = ShortcutRecorderMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                ShortcutRecorderCommand::Change(value) => {
                    if let Some(handler) = self.on_change {
                        handler.call(value);
                    }
                }
                ShortcutRecorderCommand::FocusControl => self.focus.focus(CONTROL),
            }
        }
    }
}

/// A control that captures a keyboard shortcut.
#[component]
pub fn ShortcutRecorder(
    /// What the shortcut is for, such as "Open command palette".
    label: String,
    /// Controlled shortcut, like `Mod+K`. An empty string means none. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial shortcut when uncontrolled.
    default_value: Option<String>,
    /// Called with the new shortcut, or `None` when it is cleared.
    on_change: Option<EventHandler<Option<String>>>,
    /// Keyboard convention for display. Default: detected from the host.
    platform: Option<Platform>,
    /// Shown when no shortcut is set.
    #[props(default = "Not set".to_string())]
    placeholder: String,
    /// Block recording and clearing.
    #[props(default)]
    disabled: bool,
) -> Element {
    // Detected after mount so server and client render the same markup first.
    let mut detected = use_signal(Platform::default);
    use_future(move || async move {
        if let Ok(p) = document::eval("return navigator.platform").join::<String>().await {
            detected.set(Platform::detect(&p));
        }
    });
    let platform = platform.unwrap_or_else(|| *detected.read());
    let input = use_memo(move || ShortcutRecorderInput {
        value: value(),
        default_value: default_value.clone(),
        platform,
        disabled,
    });
    let mut state = use_signal(|| ShortcutRecorderMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ShortcutRecorderMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let ctx = use_hook(|| Ctx { state, input, focus: FocusMap::new(), on_change });

    let s = state.read();
    let recording = s.recording;

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-recording": recording.then_some(""),
            "data-disabled": disabled.then_some(""),
            span { "data-scope": h::SCOPE, "data-part": "label", id: ids.label.clone(), {label} }
            div { "data-scope": h::SCOPE, "data-part": "field",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "control",
                    id: ids.control.clone(),
                    "aria-labelledby": ids.label.clone(),
                    "aria-describedby": ids.status.clone(),
                    "data-recording": recording.then_some(""),
                    disabled,
                    onmounted: move |e| ctx.focus.register(CONTROL.to_string(), e.data()),
                    onclick: move |_| ctx.send(ShortcutRecorderEvent::Record),
                    onkeydown: move |e| {
                        let m = e.modifiers();
                        let press = KeyPress {
                            key: e.key().to_string(),
                            code: e.code().to_string(),
                            ctrl: m.ctrl(),
                            alt: m.alt(),
                            shift: m.shift(),
                            meta: m.meta(),
                        };
                        let recording = ctx.state.peek().recording;
                        if let Some(event) = h::control_keydown(&press, recording) {
                            e.prevent_default();
                            ctx.send(event);
                        }
                    },
                    // Space activates a button on keyup; don't let that restart recording.
                    onkeyup: move |e| {
                        if e.key().to_string() == " " {
                            e.prevent_default();
                        }
                    },
                    onblur: move |_| ctx.send(ShortcutRecorderEvent::Stop),
                    if recording {
                        span { "data-scope": h::SCOPE, "data-part": "placeholder", "Press keys…" }
                    } else if let Some(v) = s.value.clone() {
                        span { "data-scope": h::SCOPE, "data-part": "keys",
                            for k in h::display(&v, platform) {
                                Kbd { "{k}" }
                            }
                        }
                    } else {
                        span { "data-scope": h::SCOPE, "data-part": "placeholder", "{placeholder}" }
                    }
                }
                if s.value.is_some() && !recording {
                    IconButton {
                        icon: IconName::X,
                        label: "Clear shortcut",
                        disabled,
                        on_click: move |_| ctx.send(ShortcutRecorderEvent::Clear),
                    }
                }
            }
            p {
                "data-scope": h::SCOPE,
                "data-part": "status",
                id: ids.status.clone(),
                role: "status",
                "aria-live": "polite",
                {h::status_text(&s, platform)}
            }
        }
    }
}
