//! `<AudioPlayer>`: a compact player over a native `<audio>` element, with
//! seek, volume, playback rate, status, retry and an optional transcript link.

use hydroxy_headless::audio_player::{
    self as h, AudioPlayerCommand, AudioPlayerEvent, AudioPlayerInput, AudioPlayerMachine, AudioPlayerState, MediaFacts,
};
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Reflect;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

/// The media element with this id, if it is mounted.
fn media(id: &str) -> Option<web_sys::HtmlMediaElement> {
    document().get_element_by_id(id)?.dyn_into().ok()
}

/// What the element reports right now. `error` is read by reflection to avoid a `MediaError` binding.
fn facts(el: &web_sys::HtmlMediaElement) -> MediaFacts {
    let error_code = Reflect::get(el, &JsValue::from_str("error"))
        .ok()
        .filter(|e| !e.is_null() && !e.is_undefined())
        .and_then(|e| Reflect::get(&e, &JsValue::from_str("code")).ok())
        .and_then(|c| c.as_f64())
        .map_or(0, |c| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "MediaError.code is 1 to 4")]
            let code = c as u8;
            code
        });
    let duration = el.duration();
    MediaFacts {
        current: el.current_time(),
        duration: if duration.is_finite() { duration } else { 0.0 },
        paused: el.paused(),
        ended: el.ended(),
        seeking: el.seeking(),
        ready_state: el.ready_state().try_into().unwrap_or(0),
        error_code,
        volume: el.volume(),
        muted: el.muted(),
        rate: el.playback_rate(),
    }
}

/// A player with a play button, scrubber, time, mute, volume, rate and status line.
/// Keys: Space or K play, arrows or J/L skip, Up/Down volume, M mute, `<` `>` rate.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "one view! tree; props are owned by the macro"
)]
pub fn AudioPlayer(
    /// URL of the audio file.
    #[prop(into)]
    src: String,
    /// A visible title. Also names the player.
    #[prop(optional, into)]
    title: Option<String>,
    /// Seconds a skip moves. Default 5.
    #[prop(default = 5.0)]
    seek_step: f64,
    /// Playback rates, ascending. Default 0.75 to 2.
    #[prop(optional)]
    rates: Option<Vec<f64>>,
    /// Where the transcript lives. Shows a link.
    #[prop(optional, into)]
    transcript_href: Option<String>,
    /// Called when the transcript button is pressed (when there is no `transcript_href`).
    #[prop(optional, into)]
    on_transcript_request: Option<Callback<()>>,
    /// Called as playback moves, with the position in seconds.
    #[prop(optional, into)]
    on_time: Option<Callback<f64>>,
) -> impl IntoView {
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let input =
        AudioPlayerInput { seek_step, volume_step: 0.1, rates: rates.unwrap_or_else(|| h::DEFAULT_RATES.to_vec()) };
    let input = Signal::stored(input);
    let state = RwSignal::new(AudioPlayerState::default());
    let label = title.clone().unwrap_or_else(|| "Audio player".to_string());

    // The browser refused `play()`: autoplay policy, or no user gesture.
    let reject = move || {
        drive::<AudioPlayerMachine, _>(state, input, AudioPlayerEvent::PlayRejected);
    };
    let send = move |event: AudioPlayerEvent| {
        let commands = drive::<AudioPlayerMachine, _>(state, input, event);
        let Some(el) = id.try_get_value().and_then(|id| media(&id)) else { return };
        for command in commands {
            match command {
                AudioPlayerCommand::Play => match el.play() {
                    Ok(promise) => {
                        let on_err = Closure::once(move |_: JsValue| reject());
                        let _ = promise.catch(&on_err);
                        on_err.forget();
                    }
                    Err(_) => reject(),
                },
                AudioPlayerCommand::Pause => {
                    let _ = el.pause();
                }
                AudioPlayerCommand::Seek(t) => el.set_current_time(t),
                AudioPlayerCommand::Volume(v) => el.set_volume(v),
                AudioPlayerCommand::Muted(m) => el.set_muted(m),
                AudioPlayerCommand::Rate(r) => el.set_playback_rate(r),
                AudioPlayerCommand::Reload => el.load(),
            }
        }
    };
    // Mirror the element into the machine on every media event and once on mount.
    let sync = move || {
        let Some(el) = id.try_get_value().and_then(|id| media(&id)) else { return };
        let f = facts(&el);
        if let Some(cb) = on_time {
            cb.run(f.current);
        }
        send(AudioPlayerEvent::Facts(f));
    };
    Effect::new(move |_| {
        send(AudioPlayerEvent::Attached);
        sync();
    });
    let on_media = move |_: web_sys::Event| sync();

    let current = move || state.get();
    let status = move || current().status();
    let duration = move || state.with(AudioPlayerState::duration);

    let key = move |ev: web_sys::KeyboardEvent| {
        if ev.meta_key() || ev.ctrl_key() || ev.alt_key() {
            return;
        }
        let part = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("[data-part]").ok().flatten())
            .and_then(|el| el.get_attribute("data-part"));
        let target = h::Target::of(part.as_deref());
        if let Some(event) = input.with_untracked(|i| h::resolve_key(&ev.key(), target, i)) {
            ev.prevent_default();
            send(event);
        }
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=label
            data-status=move || status().as_str()
            aria-busy=move || matches!(status(), h::PlayerStatus::Loading | h::PlayerStatus::Buffering).then_some("true")
            on:keydown=key
        >
            {title.map(|t| view! { <p data-scope=h::SCOPE data-part="title">{t}</p> })}
            <audio
                id=id.get_value()
                data-scope=h::SCOPE
                data-part="media"
                src=src.clone()
                preload="metadata"
                controls=move || status() == h::PlayerStatus::Idle
                on:timeupdate=on_media
                on:durationchange=on_media
                on:loadedmetadata=on_media
                on:canplay=on_media
                on:playing=on_media
                on:waiting=on_media
                on:seeking=on_media
                on:seeked=on_media
                on:volumechange=on_media
                on:ratechange=on_media
                on:play=on_media
                on:pause=on_media
                on:ended=on_media
                on:error=move |_: web_sys::ErrorEvent| sync()
                on:emptied=on_media
            >
                <a data-scope=h::SCOPE data-part="download" href=src.clone() download="">
                    "Download the audio"
                </a>
            </audio>
            <div data-scope=h::SCOPE data-part="controls" role="toolbar" aria-label="Playback controls">
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="play"
                    aria-label=move || h::play_label(status(), current().playing())
                    aria-pressed=move || current().playing().to_string()
                    disabled=move || h::controls_disabled(status())
                    on:click=move |_| send(AudioPlayerEvent::Toggle)
                >
                    {move || {
                        let name = if current().playing() {
                            IconName::Pause
                        } else if status() == h::PlayerStatus::Ended {
                            IconName::Replay
                        } else {
                            IconName::Play
                        };
                        view! { <Icon name=name size=18 /> }
                    }}
                </button>
                <input
                    type="range"
                    data-scope=h::SCOPE
                    data-part="seek"
                    aria-label="Seek"
                    min="0"
                    max=move || duration().unwrap_or(0.0).to_string()
                    step="any"
                    prop:value=move || duration().map_or(0.0, |d| current().facts.current.min(d)).to_string()
                    disabled=move || duration().is_none() || h::controls_disabled(status())
                    aria-valuetext=move || h::seek_value_text(current().facts.current, duration())
                    style=move || {
                        format!("--ox-played: {:.2}%", duration().map_or(0.0, |d| (current().facts.current / d).clamp(0.0, 1.0) * 100.0))
                    }
                    on:input=move |ev| {
                        if let Ok(t) = event_target_value(&ev).parse::<f64>() {
                            send(AudioPlayerEvent::SeekTo(t));
                        }
                    }
                />
                <span data-scope=h::SCOPE data-part="time">{move || h::time_text(current().facts.current, duration())}</span>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="mute"
                    aria-label=move || h::mute_label(current().facts.muted)
                    aria-pressed=move || current().facts.muted.to_string()
                    on:click=move |_| send(AudioPlayerEvent::ToggleMute)
                >
                    {move || {
                        let name = if current().facts.muted { IconName::VolumeOff } else { IconName::Volume };
                        view! { <Icon name=name size=18 /> }
                    }}
                </button>
                <input
                    type="range"
                    data-scope=h::SCOPE
                    data-part="volume"
                    aria-label="Volume"
                    min="0"
                    max="1"
                    step="0.05"
                    prop:value=move || {
                        let f = current().facts;
                        if f.muted { 0.0 } else { f.volume }.to_string()
                    }
                    aria-valuetext=move || {
                        let f = current().facts;
                        format!("{}%", (if f.muted { 0.0 } else { f.volume } * 100.0).round())
                    }
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                            send(AudioPlayerEvent::SetVolume(v));
                        }
                    }
                />
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="rate"
                    aria-label=move || format!("Playback rate {}", h::rate_text(current().facts.rate))
                    on:click=move |_| send(AudioPlayerEvent::CycleRate(1))
                >
                    {move || h::rate_text(current().facts.rate)}
                </button>
            </div>
            <div data-scope=h::SCOPE data-part="footer">
                <span data-scope=h::SCOPE data-part="status" role="status" aria-live="polite">
                    {move || h::status_text(status(), current().facts.error_code)}
                </span>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="retry"
                    hidden=move || status() != h::PlayerStatus::Error
                    on:click=move |_| send(AudioPlayerEvent::Retry)
                >
                    "Try again"
                </button>
                {match (transcript_href, on_transcript_request) {
                    (Some(href), _) => EitherOf3::A(view! { <a data-scope=h::SCOPE data-part="transcript" href=href>"Transcript"</a> }),
                    (None, Some(cb)) => {
                        EitherOf3::B(view! {
                            <button type="button" data-scope=h::SCOPE data-part="transcript" on:click=move |_| cb.run(())>
                                "Transcript"
                            </button>
                        })
                    }
                    (None, None) => EitherOf3::C(()),
                }}
            </div>
        </div>
    }
}
