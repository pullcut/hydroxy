//! `<ShortcutRecorder>`: click it, press a key combination, and it shows the
//! result as keycaps. Esc cancels, Backspace (or the clear button) removes it.
//! Shortcuts are stored portably (`Mod+Shift+K`) and shown per platform.

use leptos::either::EitherOf3;
use leptos::prelude::*;
use oxide_headless::shortcut_recorder::{
    self as h, KeyPress, Platform, ShortcutRecorderCommand, ShortcutRecorderEvent, ShortcutRecorderInput,
    ShortcutRecorderMachine, ShortcutRecorderState,
};
use oxide_statechart::Machine;

use crate::focus::focus_id;
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::kbd::Kbd;

pub use oxide_headless::shortcut_recorder::{Platform as ShortcutPlatform, display, spoken};

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<ShortcutRecorderState>,
    input: Memo<ShortcutRecorderInput>,
    ids: StoredValue<h::Ids>,
    on_change: Option<Callback<Option<String>>>,
}

impl Ctx {
    fn send(self, event: ShortcutRecorderEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| ShortcutRecorderMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                ShortcutRecorderCommand::Change(value) => {
                    if let Some(cb) = self.on_change {
                        cb.run(value);
                    }
                }
                ShortcutRecorderCommand::FocusControl => {
                    let id = self.ids.get_value().control;
                    set_timeout(move || focus_id(&id), std::time::Duration::ZERO);
                }
            }
        }
    }
}

fn press_of(ev: &leptos::ev::KeyboardEvent) -> KeyPress {
    KeyPress {
        key: ev.key(),
        code: ev.code(),
        ctrl: ev.ctrl_key(),
        alt: ev.alt_key(),
        shift: ev.shift_key(),
        meta: ev.meta_key(),
    }
}

/// A control that captures a keyboard shortcut.
#[component]
pub fn ShortcutRecorder(
    /// What the shortcut is for, such as "Open command palette".
    #[prop(into)]
    label: String,
    /// Controlled shortcut, like `Mod+K`. An empty string means none. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial shortcut when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Called with the new shortcut, or `None` when it is cleared.
    #[prop(optional, into)]
    on_change: Option<Callback<Option<String>>>,
    /// Keyboard convention for display. Default: detected from the browser.
    #[prop(optional)]
    platform: Option<Platform>,
    /// Shown when no shortcut is set.
    #[prop(default = "Not set".into(), into)]
    placeholder: String,
    /// Block recording and clearing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
) -> impl IntoView {
    // Detected after mount so server and browser render the same markup first.
    let detected = RwSignal::new(Platform::default());
    Effect::new(move |_| {
        if let Ok(p) = window().navigator().platform() {
            detected.set(Platform::detect(&p));
        }
    });
    let platform = Memo::new(move |_| platform.unwrap_or_else(|| detected.get()));
    let input = Memo::new(move |_| ShortcutRecorderInput {
        value: value.get(),
        default_value: default_value.clone(),
        platform: platform.get(),
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(ShortcutRecorderMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ShortcutRecorderMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let ctx = Ctx { state, input, ids: StoredValue::new(ids.clone()), on_change };
    let off = move || input.with(|i| i.disabled);
    let recording = move || state.with(|s| s.recording);
    let placeholder = StoredValue::new(placeholder);

    let content = move || {
        if recording() {
            return EitherOf3::A(view! { <span data-scope=h::SCOPE data-part="placeholder">"Press keys…"</span> });
        }
        match state.with(|s| s.value.clone()) {
            Some(v) => EitherOf3::B(view! {
                <span data-scope=h::SCOPE data-part="keys">
                    {h::display(&v, platform.get())
                        .into_iter()
                        .map(|k| view! { <Kbd>{k}</Kbd> })
                        .collect_view()}
                </span>
            }),
            None => {
                EitherOf3::C(view! { <span data-scope=h::SCOPE data-part="placeholder">{placeholder.get_value()}</span> })
            }
        }
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-recording=move || recording().then_some("")
            data-disabled=move || off().then_some("")
        >
            <span data-scope=h::SCOPE data-part="label" id=ids.label.clone()>{label}</span>
            <div data-scope=h::SCOPE data-part="field">
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="control"
                    id=ids.control.clone()
                    aria-labelledby=ids.label.clone()
                    aria-describedby=ids.status.clone()
                    data-recording=move || recording().then_some("")
                    disabled=off
                    on:click=move |ev| {
                        // Keyboard activation is handled on keydown; a click with no detail is that echo.
                        if ev.detail() > 0 {
                            ctx.send(ShortcutRecorderEvent::Record);
                        }
                    }
                    on:keydown=move |ev| {
                        let recording = state.with_untracked(|s| s.recording);
                        if let Some(event) = h::control_keydown(&press_of(&ev), recording) {
                            ev.prevent_default();
                            ctx.send(event);
                        }
                    }
                    // Space activates a button on keyup; don't let that restart recording.
                    on:keyup=move |ev| {
                        if ev.key() == " " {
                            ev.prevent_default();
                        }
                    }
                    on:blur=move |_| ctx.send(ShortcutRecorderEvent::Stop)
                >
                    {content}
                </button>
                <Show when=move || state.with(|s| s.value.is_some() && !s.recording)>
                    <IconButton
                        icon=IconName::X
                        label="Clear shortcut"
                        disabled=Signal::derive(off)
                        on_click=move |_| ctx.send(ShortcutRecorderEvent::Clear)
                    />
                </Show>
            </div>
            <p data-scope=h::SCOPE data-part="status" id=ids.status.clone() role="status" aria-live="polite">
                {move || state.with(|s| h::status_text(s, platform.get()))}
            </p>
        </div>
    }
}
