//! The per-message action bar: copy, reload, edit, export, read aloud and votes.

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::{
    self as h, ActionBarCommand, ActionBarEvent, ActionBarInput, ActionBarMachine,
};

use super::{ActionBarState, Autohide, Feedback, Role};
use crate::drive::drive;
use crate::js;

/// Copy, reload, edit, export, read aloud and the two votes for one message. Which
/// buttons show depends on the role.
#[component]
pub fn AssistantActionBar(
    /// Whose message this is.
    #[props(default)]
    role: Role,
    /// What Copy writes to the clipboard.
    text: String,
    /// What Export hands to `on_export`.
    markdown: String,
    /// A reply is running.
    #[props(default)]
    running: bool,
    /// This is the last message in the thread.
    #[props(default)]
    is_last: bool,
    /// Hover-reveal rule.
    #[props(default)]
    autohide: Autohide,
    /// Hide the bar while a reply runs.
    #[props(default)]
    hide_when_running: bool,
    /// Called when Reload is pressed.
    on_reload: Option<EventHandler<()>>,
    /// Called when Edit is pressed.
    on_edit: Option<EventHandler<()>>,
    /// Called with the vote (`None` when taken back).
    on_feedback: Option<EventHandler<Option<Feedback>>>,
    /// Called with the Markdown export.
    on_export: Option<EventHandler<String>>,
    /// Called with whether reading aloud should start.
    on_speak: Option<EventHandler<bool>>,
) -> Element {
    let input = ActionBarInput { running, hide_when_running, autohide, is_last };
    // Props aren't reactive inside handlers; refresh the value they read each render.
    let mut input_cell = use_hook(|| CopyValue::new(input));
    *input_cell.write() = input;
    let state = use_signal(ActionBarState::default);
    // Strings aren't Copy; hold them where the handlers can all reach them.
    let mut payload = use_hook(|| CopyValue::new((String::new(), String::new())));
    *payload.write() = (text, markdown);
    let send = move |event: ActionBarEvent| {
        for command in drive::<ActionBarMachine>(state, input_cell, event) {
            match command {
                ActionBarCommand::Copy => {
                    let text = payload.read().0.clone();
                    spawn(async move {
                        // Best-effort: without clipboard access the button just confirms anyway.
                        js::copy(&text);
                        js::sleep(1400).await;
                        drive::<ActionBarMachine>(state, input_cell, ActionBarEvent::CopiedReset);
                    });
                }
                ActionBarCommand::Feedback(v) => {
                    if let Some(handler) = on_feedback {
                        handler.call(v);
                    }
                }
                ActionBarCommand::Reload => {
                    if let Some(handler) = on_reload {
                        handler.call(());
                    }
                }
                ActionBarCommand::Edit => {
                    if let Some(handler) = on_edit {
                        handler.call(());
                    }
                }
                ActionBarCommand::Export => {
                    if let Some(handler) = on_export {
                        handler.call(payload.read().1.clone());
                    }
                }
                ActionBarCommand::Speak => {
                    if let Some(handler) = on_speak {
                        handler.call(true);
                    }
                }
                ActionBarCommand::StopSpeaking => {
                    if let Some(handler) = on_speak {
                        handler.call(false);
                    }
                }
            }
        }
    };
    if h::action_bar_hidden(&input) {
        return rsx! {};
    }
    let s = state.read().clone();
    rsx! {
        div {
            "data-scope": h::SCOPE_ACTION_BAR,
            "data-part": "root",
            role: "toolbar",
            "aria-label": "Message actions",
            "aria-live": "polite",
            "data-role": role.as_str(),
            "data-floating": h::action_bar_floating(&input).then_some(""),
            for name in h::actions_for(role).iter().copied() {
                {
                    let label = match name {
                        "copy" if s.copied => "Copied",
                        "copy" => "Copy",
                        "reload" => "Reload",
                        "edit" => "Edit",
                        "export" => "Export",
                        "speak" if s.speaking => "Stop",
                        "speak" => "Read aloud",
                        "positive" => "Helpful",
                        _ => "Not helpful",
                    };
                    let pressed = match name {
                        "speak" => Some(s.speaking),
                        "positive" => Some(s.feedback == Some(Feedback::Positive)),
                        "negative" => Some(s.feedback == Some(Feedback::Negative)),
                        _ => None,
                    };
                    let event = match name {
                        "copy" => ActionBarEvent::Copy,
                        "reload" => ActionBarEvent::Reload,
                        "edit" => ActionBarEvent::Edit,
                        "export" => ActionBarEvent::Export,
                        "speak" => ActionBarEvent::ToggleSpeak,
                        "positive" => ActionBarEvent::Vote(Feedback::Positive),
                        _ => ActionBarEvent::Vote(Feedback::Negative),
                    };
                    rsx! {
                        button {
                            key: "{name}",
                            r#type: "button",
                            "data-scope": h::SCOPE_ACTION_BAR,
                            "data-part": "action",
                            "data-action": name,
                            "aria-pressed": pressed.map(|p| p.to_string()),
                            disabled: name == "copy" && running,
                            onclick: move |_| send(event),
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
