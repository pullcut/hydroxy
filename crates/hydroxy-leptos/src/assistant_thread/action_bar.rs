//! The per-message action bar.

use std::time::Duration;

use hydroxy_headless::assistant_thread::{
    self as h, ActionBarCommand, ActionBarEvent, ActionBarInput, ActionBarMachine,
};
use leptos::prelude::*;

use super::{ActionBarState, Autohide, Feedback, Role};
use crate::drive::drive;

/// Copy, reload, edit, export, read aloud and the two votes for one message. Which
/// buttons show depends on the role.
#[component]
pub fn AssistantActionBar(
    /// Whose message this is.
    #[prop(optional)]
    role: Role,
    /// What Copy writes to the clipboard.
    #[prop(into)]
    text: Signal<String>,
    /// What Export hands to `on_export`.
    #[prop(into)]
    markdown: Signal<String>,
    /// A reply is running.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// This is the last message in the thread.
    #[prop(optional)]
    is_last: bool,
    /// Hover-reveal rule.
    #[prop(optional)]
    autohide: Autohide,
    /// Hide the bar while a reply runs.
    #[prop(optional)]
    hide_when_running: bool,
    /// Called when Reload is pressed.
    #[prop(optional, into)]
    on_reload: Option<Callback<()>>,
    /// Called when Edit is pressed.
    #[prop(optional, into)]
    on_edit: Option<Callback<()>>,
    /// Called with the vote (`None` when taken back).
    #[prop(optional, into)]
    on_feedback: Option<Callback<Option<Feedback>>>,
    /// Called with the Markdown export.
    #[prop(optional, into)]
    on_export: Option<Callback<String>>,
    /// Called with whether reading aloud should start.
    #[prop(optional, into)]
    on_speak: Option<Callback<bool>>,
) -> impl IntoView {
    let input = Memo::new(move |_| ActionBarInput { running: running.get(), hide_when_running, autohide, is_last });
    let state = RwSignal::new(ActionBarState::default());
    let send = move |event: ActionBarEvent| {
        for command in drive::<ActionBarMachine, _>(state, input, event) {
            match command {
                ActionBarCommand::Copy => {
                    // Best-effort: without clipboard access the button just confirms anyway.
                    crate::js::copy(&text.get_untracked(), || {});
                    set_timeout(
                        move || {
                            drive::<ActionBarMachine, _>(state, input, ActionBarEvent::CopiedReset);
                        },
                        Duration::from_millis(1400),
                    );
                }
                ActionBarCommand::Feedback(v) => {
                    if let Some(cb) = on_feedback {
                        cb.run(v);
                    }
                }
                ActionBarCommand::Reload => {
                    if let Some(cb) = on_reload {
                        cb.run(());
                    }
                }
                ActionBarCommand::Edit => {
                    if let Some(cb) = on_edit {
                        cb.run(());
                    }
                }
                ActionBarCommand::Export => {
                    if let Some(cb) = on_export {
                        cb.run(markdown.get_untracked());
                    }
                }
                ActionBarCommand::Speak => {
                    if let Some(cb) = on_speak {
                        cb.run(true);
                    }
                }
                ActionBarCommand::StopSpeaking => {
                    if let Some(cb) = on_speak {
                        cb.run(false);
                    }
                }
            }
        }
    };

    let buttons = move || {
        h::actions_for(role)
            .iter()
            .map(|&name| {
                let label = move || {
                    state.with(|s| match name {
                        "copy" if s.copied => "Copied",
                        "copy" => "Copy",
                        "reload" => "Reload",
                        "edit" => "Edit",
                        "export" => "Export",
                        "speak" if s.speaking => "Stop",
                        "speak" => "Read aloud",
                        "positive" => "Helpful",
                        _ => "Not helpful",
                    })
                };
                let pressed = move || {
                    state.with(|s| match name {
                        "speak" => Some(s.speaking),
                        "positive" => Some(s.feedback == Some(Feedback::Positive)),
                        "negative" => Some(s.feedback == Some(Feedback::Negative)),
                        _ => None,
                    })
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
                view! {
                    <button
                        type="button"
                        data-scope=h::SCOPE_ACTION_BAR
                        data-part="action"
                        data-action=name
                        aria-pressed=move || pressed().map(|p| p.to_string())
                        disabled=move || name == "copy" && running.get()
                        on:click=move |_| send(event)
                    >
                        {label}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        {move || {
            (!input.with(h::action_bar_hidden))
                .then(|| {
                    view! {
                        <div
                            data-scope=h::SCOPE_ACTION_BAR
                            data-part="root"
                            role="toolbar"
                            aria-label="Message actions"
                            aria-live="polite"
                            data-role=role.as_str()
                            data-floating=input.with(h::action_bar_floating).then_some("")
                        >
                            {buttons()}
                        </div>
                    }
                })
        }}
    }
}
