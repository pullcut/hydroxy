//! `AudioPlayer`: a compact player over a native `<audio>` element, with
//! seek, volume, playback rate, status, retry and an optional transcript link.
//!
//! Dioxus has no typed handle for media elements, so a small script run through
//! `document::eval` mirrors the element into the machine and applies commands.

use dioxus::prelude::*;
use oxide_headless::audio_player::{
    self as h, AudioPlayerCommand, AudioPlayerEvent, AudioPlayerInput, AudioPlayerMachine, AudioPlayerState, MediaFacts,
};
use oxide_statechart::Machine;

/// Waits for the element, then reports its state on every media event and applies `[command, value]`
/// messages. Messages out are string arrays: `f` plus ten facts, `r` for a refused `play()`, or `k`, the key
/// and the focused part for a claimed shortcut. The first message in lists the keys each part claims.
const SCRIPT: &str = r"
const claimed = Object.fromEntries(await dioxus.recv());
const el = await new Promise((resolve) => {
  const look = () => { const e = document.getElementById(__ID__); e ? resolve(e) : requestAnimationFrame(look); };
  look();
});
const post = () => dioxus.send(['f',
  el.currentTime, Number.isFinite(el.duration) ? el.duration : 0, el.paused ? 1 : 0, el.ended ? 1 : 0,
  el.seeking ? 1 : 0, el.readyState, el.error ? el.error.code : 0, el.volume, el.muted ? 1 : 0, el.playbackRate,
].map(String));
['timeupdate', 'durationchange', 'loadedmetadata', 'canplay', 'playing', 'waiting', 'seeking', 'seeked',
 'volumechange', 'ratechange', 'play', 'pause', 'ended', 'error', 'emptied'].forEach((n) => el.addEventListener(n, post));
el.closest('[data-part=root]')?.addEventListener('keydown', (e) => {
  if (e.metaKey || e.ctrlKey || e.altKey) return;
  const part = e.target.closest('[data-part]')?.dataset.part ?? '';
  const keys = claimed[part] ?? claimed[''];
  if (keys.includes(e.key)) { e.preventDefault(); dioxus.send(['k', e.key, part]); }
});
post();
while (true) {
  const [command, value] = await dioxus.recv();
  if (command === 'play') el.play().catch(() => dioxus.send(['r']));
  else if (command === 'pause') el.pause();
  else if (command === 'seek') el.currentTime = value;
  else if (command === 'volume') el.volume = value;
  else if (command === 'muted') el.muted = value > 0;
  else if (command === 'rate') el.playbackRate = value;
  else if (command === 'reload') el.load();
}
";

/// Every part name, plus the empty name for anything else, with the keys each claims.
fn claim_table(input: &AudioPlayerInput) -> Vec<(String, Vec<&'static str>)> {
    ["", "seek", "volume", "play", "mute", "rate", "retry", "transcript"]
        .into_iter()
        .map(|part| (part.to_string(), h::claimed_keys((!part.is_empty()).then_some(part), input)))
        .collect()
}

/// A player with a play button, scrubber, time, mute, volume, rate and status line.
/// Keys: Space or K play, arrows or J/L skip, Up/Down volume, M mute, `<` `>` rate.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AudioPlayer(
    /// URL of the audio file.
    src: String,
    /// A visible title. Also names the player.
    title: Option<String>,
    /// Seconds a skip moves. Default 5.
    #[props(default = 5.0)]
    seek_step: f64,
    /// Playback rates, ascending. Default 0.75 to 2.
    rates: Option<Vec<f64>>,
    /// Where the transcript lives. Shows a link.
    transcript_href: Option<String>,
    /// Called when the transcript button is pressed (when there is no `transcript_href`).
    on_transcript_request: Option<EventHandler<()>>,
    /// Called as playback moves, with the position in seconds.
    on_time: Option<EventHandler<f64>>,
) -> Element {
    let id = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let input = use_hook(|| AudioPlayerInput { seek_step, volume_step: 0.1, rates: rates.clone().unwrap_or_else(|| h::DEFAULT_RATES.to_vec()) });
    let mut state = use_signal(AudioPlayerState::default);
    let eval = use_hook({
        let id = id.clone();
        move || document::eval(&SCRIPT.replace("__ID__", &format!("{id:?}")))
    });

    let send = {
        let input = input.clone();
        Callback::new(move |event: AudioPlayerEvent| {
            let step = AudioPlayerMachine::transition(&state.peek(), event, &input);
            if let Some(next) = step.state {
                state.set(next);
            }
            for command in step.commands {
                let _ = match command {
                    AudioPlayerCommand::Play => eval.send(("play", 0.0)),
                    AudioPlayerCommand::Pause => eval.send(("pause", 0.0)),
                    AudioPlayerCommand::Seek(t) => eval.send(("seek", t)),
                    AudioPlayerCommand::Volume(v) => eval.send(("volume", v)),
                    AudioPlayerCommand::Muted(m) => eval.send(("muted", f64::from(u8::from(m)))),
                    AudioPlayerCommand::Rate(r) => eval.send(("rate", r)),
                    AudioPlayerCommand::Reload => eval.send(("reload", 0.0)),
                };
            }
        })
    };
    let table = claim_table(&input);
    let keys_input = input.clone();
    use_future(move || {
        let table = table.clone();
        let keys_input = keys_input.clone();
        async move {
            let mut eval = eval;
            let _ = eval.send(table);
            send.call(AudioPlayerEvent::Attached);
            while let Ok(message) = eval.recv::<Vec<String>>().await {
                let n = |i: usize| message.get(i).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                match message.first().map(String::as_str) {
                    Some("f") => {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "readyState and error.code are 0 to 4")]
                        let facts = MediaFacts {
                            current: n(1),
                            duration: n(2),
                            paused: n(3) > 0.0,
                            ended: n(4) > 0.0,
                            seeking: n(5) > 0.0,
                            ready_state: n(6) as u8,
                            error_code: n(7) as u8,
                            volume: n(8),
                            muted: n(9) > 0.0,
                            rate: n(10),
                        };
                        if let Some(handler) = on_time {
                            handler.call(facts.current);
                        }
                        send.call(AudioPlayerEvent::Facts(facts));
                    }
                    Some("r") => send.call(AudioPlayerEvent::PlayRejected),
                    Some("k") => {
                        let target = h::Target::of(message.get(2).map(String::as_str).filter(|p| !p.is_empty()));
                        if let Some(event) = message.get(1).and_then(|k| h::resolve_key(k, target, &keys_input)) {
                            send.call(event);
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    let s = state.read().clone();
    let status = s.status();
    let playing = s.playing();
    let duration = s.duration();
    let f = s.facts;
    let volume = if f.muted { 0.0 } else { f.volume };
    let played = duration.map_or(0.0, |d| (f.current / d).clamp(0.0, 1.0) * 100.0);
    let label = title.clone().unwrap_or_else(|| "Audio player".to_string());
    let busy = matches!(status, h::PlayerStatus::Loading | h::PlayerStatus::Buffering).then_some("true");

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": label,
            "data-status": status.as_str(),
            "aria-busy": busy,
            if let Some(t) = title {
                p { "data-scope": h::SCOPE, "data-part": "title", "{t}" }
            }
            audio {
                id: "{id}",
                "data-scope": h::SCOPE,
                "data-part": "media",
                src: "{src}",
                preload: "metadata",
                controls: status == h::PlayerStatus::Idle,
                a { "data-scope": h::SCOPE, "data-part": "download", href: "{src}", download: "", "Download the audio" }
            }
            div { "data-scope": h::SCOPE, "data-part": "controls", role: "toolbar", "aria-label": "Playback controls",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "play",
                    "aria-label": h::play_label(status, playing),
                    "aria-pressed": playing.to_string(),
                    disabled: h::controls_disabled(status),
                    onclick: move |_| send.call(AudioPlayerEvent::Toggle),
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.75",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "aria-hidden": "true",
                        width: "18",
                        height: "18",
                        dangerous_inner_html: if playing { h::PAUSE } else if status == h::PlayerStatus::Ended { h::REPLAY } else { h::PLAY },
                    }
                }
                input {
                    r#type: "range",
                    "data-scope": h::SCOPE,
                    "data-part": "seek",
                    "aria-label": "Seek",
                    min: "0",
                    max: "{duration.unwrap_or(0.0)}",
                    step: "any",
                    value: "{duration.map_or(0.0, |d| f.current.min(d))}",
                    disabled: duration.is_none() || h::controls_disabled(status),
                    "aria-valuetext": h::seek_value_text(f.current, duration),
                    style: "--ox-played: {played:.2}%",
                    oninput: move |e| {
                        if let Ok(t) = e.value().parse::<f64>() {
                            send.call(AudioPlayerEvent::SeekTo(t));
                        }
                    },
                }
                span { "data-scope": h::SCOPE, "data-part": "time", {h::time_text(f.current, duration)} }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "mute",
                    "aria-label": h::mute_label(f.muted),
                    "aria-pressed": f.muted.to_string(),
                    onclick: move |_| send.call(AudioPlayerEvent::ToggleMute),
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.75",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "aria-hidden": "true",
                        width: "18",
                        height: "18",
                        dangerous_inner_html: if f.muted { h::MUTED } else { h::VOLUME },
                    }
                }
                input {
                    r#type: "range",
                    "data-scope": h::SCOPE,
                    "data-part": "volume",
                    "aria-label": "Volume",
                    min: "0",
                    max: "1",
                    step: "0.05",
                    value: "{volume}",
                    "aria-valuetext": format!("{}%", (volume * 100.0).round()),
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            send.call(AudioPlayerEvent::SetVolume(v));
                        }
                    },
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "rate",
                    "aria-label": format!("Playback rate {}", h::rate_text(f.rate)),
                    onclick: move |_| send.call(AudioPlayerEvent::CycleRate(1)),
                    {h::rate_text(f.rate)}
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "footer",
                span { "data-scope": h::SCOPE, "data-part": "status", role: "status", "aria-live": "polite", {h::status_text(status, f.error_code)} }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "retry",
                    hidden: status != h::PlayerStatus::Error,
                    onclick: move |_| send.call(AudioPlayerEvent::Retry),
                    "Try again"
                }
                if let Some(href) = transcript_href {
                    a { "data-scope": h::SCOPE, "data-part": "transcript", href: "{href}", "Transcript" }
                } else if let Some(handler) = on_transcript_request {
                    button { r#type: "button", "data-scope": h::SCOPE, "data-part": "transcript", onclick: move |_| handler.call(()), "Transcript" }
                }
            }
        }
    }
}
