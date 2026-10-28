//! `<VoiceSession>`: the screen for a live voice conversation.

use hydroxy_headless::voice_session::{
    self as h, Turn, VoicePhase, VoiceSessionCommand, VoiceSessionEvent, VoiceSessionInput, VoiceSessionMachine,
    VoiceSessionState,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;

use crate::button::{Button, ButtonVariant};
use crate::drive::drive;
use crate::focus::focus_first;
use crate::icon::{Icon, IconName};

pub use h::{Speaker, Turn as TranscriptTurn, VoicePhase as Phase};

/// A voice conversation: status, mute and end controls, and a live transcript.
/// The audio transport is the owner's job. Report what it is doing through
/// `phase`; start and end it from `on_start` and `on_end`.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "one view! tree; props are owned by the macro"
)]
pub fn VoiceSession(
    /// Controlled phase. Pair with `on_phase_change`, and drive it from the transport.
    #[prop(optional, into)]
    phase: MaybeProp<VoicePhase>,
    /// Initial phase when uncontrolled.
    #[prop(optional)]
    default_phase: VoicePhase,
    /// Controlled mute. Pair with `on_mute_change`.
    #[prop(optional, into)]
    muted: MaybeProp<bool>,
    /// Initial mute when uncontrolled.
    #[prop(optional)]
    default_muted: bool,
    /// The transcript, oldest first. A turn with `partial` set is still arriving.
    #[prop(optional, into)]
    transcript: Signal<Vec<Turn>>,
    /// What went wrong, shown while the phase is `Error`.
    #[prop(optional, into)]
    error_message: MaybeProp<String>,
    /// The user asked to start or retry. Open the transport.
    #[prop(optional, into)]
    on_start: Option<Callback<()>>,
    /// The user ended or cancelled. Close the transport.
    #[prop(optional, into)]
    on_end: Option<Callback<()>>,
    /// Called with the new mute flag. Apply it to the microphone track.
    #[prop(optional, into)]
    on_mute_change: Option<Callback<bool>>,
    /// Called with the new phase whenever it changes or a change is requested.
    #[prop(optional, into)]
    on_phase_change: Option<Callback<VoicePhase>>,
) -> impl IntoView {
    let input =
        Memo::new(move |_| VoiceSessionInput { phase: phase.get(), default_phase, muted: muted.get(), default_muted });
    let state = RwSignal::new(input.with_untracked(VoiceSessionMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| VoiceSessionMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let root_id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let send = move |event: VoiceSessionEvent| {
        let swaps_controls = matches!(event, VoiceSessionEvent::Start | VoiceSessionEvent::End);
        let commands = drive::<VoiceSessionMachine, _>(state, input, event);
        if let (true, Some(root)) = (swaps_controls, root_id.try_get_value()) {
            // The button that was pressed is replaced; move focus to the one that took its place.
            request_animation_frame(move || focus_first(&root, "[data-part=controls] button:last-of-type"));
        }
        for command in commands {
            match command {
                VoiceSessionCommand::Start => {
                    if let Some(cb) = on_start {
                        cb.run(());
                    }
                }
                VoiceSessionCommand::End => {
                    if let Some(cb) = on_end {
                        cb.run(());
                    }
                }
                VoiceSessionCommand::PhaseChange(p) => {
                    if let Some(cb) = on_phase_change {
                        cb.run(p);
                    }
                }
                VoiceSessionCommand::MuteChange(m) => {
                    if let Some(cb) = on_mute_change {
                        cb.run(m);
                    }
                }
            }
        }
    };
    // The owner's events (`Connected`, `Speaking`…) arrive as `phase`; the machine only sees user events.
    let current = move || state.with(|s: &VoiceSessionState| *s);

    view! {
        <section
            id=root_id.get_value()
            data-scope=h::SCOPE
            data-part="root"
            data-state=move || current().phase.as_str()
            aria-label="Voice conversation"
        >
            <div data-scope=h::SCOPE data-part="orb" data-state=move || current().phase.as_str() data-muted=move || current().muted.then_some("") aria-hidden="true">
                <span></span>
            </div>
            <p data-scope=h::SCOPE data-part="status" role="status">
                {move || {
                    let s = current();
                    match (s.phase, error_message.get()) {
                        (VoicePhase::Error, Some(msg)) => msg,
                        (p, _) => h::status_text(p, s.muted).to_string(),
                    }
                }}
            </p>
            <div data-scope=h::SCOPE data-part="controls">
                {move || {
                    let s = current();
                    if matches!(s.phase, VoicePhase::Idle | VoicePhase::Error) {
                        Either::Left(view! {
                            <Button variant=ButtonVariant::Accent on_click=move |_| send(VoiceSessionEvent::Start)>
                                {h::start_label(s.phase)}
                            </Button>
                        })
                    } else {
                        Either::Right(view! {
                            <button
                                type="button"
                                data-scope=h::SCOPE
                                data-part="mute"
                                aria-pressed=s.muted.to_string()
                                aria-label=h::mute_label(s.muted)
                                title=h::mute_label(s.muted)
                                disabled=!s.phase.is_live()
                                on:click=move |_| send(VoiceSessionEvent::ToggleMute)
                            >
                                <Icon name=if s.muted { IconName::MicOff } else { IconName::Mic } size=20 />
                            </button>
                            <Button variant=ButtonVariant::Danger on_click=move |_| send(VoiceSessionEvent::End)>
                                {h::end_label(s.phase)}
                            </Button>
                        })
                    }
                }}
            </div>
            <ol data-scope=h::SCOPE data-part="transcript" role="log" aria-label="Transcript" aria-live="polite">
                {move || {
                    transcript
                        .get()
                        .into_iter()
                        .map(|turn| {
                            view! {
                                <li
                                    data-scope=h::SCOPE
                                    data-part="turn"
                                    data-speaker=turn.speaker.as_str()
                                    data-partial=turn.partial.then_some("")
                                >
                                    <span data-scope=h::SCOPE data-part="speaker">{h::speaker_label(turn.speaker)}</span>
                                    <span data-scope=h::SCOPE data-part="text">{turn.text}</span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ol>
        </section>
    }
}
