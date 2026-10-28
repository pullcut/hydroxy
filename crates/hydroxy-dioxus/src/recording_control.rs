//! `RecordingControl`: record, pause, resume and stop from the microphone,
//! with an elapsed timer and a playable result.
//!
//! Dioxus has no typed handle for the media APIs, so recording runs in a small
//! script through `document::eval`. That works on web and in a desktop or mobile
//! webview that grants microphone access. The recording comes back as an object
//! URL that only that webview can read, not as a `Blob`.

use dioxus::prelude::*;
use hydroxy_headless::recording_control::{
    self as h, RecordingControlCommand, RecordingControlEvent, RecordingControlInput, RecordingControlMachine,
    RecordingError, RecordingPhase,
};
use hydroxy_statechart::Machine;

pub use h::RecordingPhase as Phase;

/// A finished recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedAudio {
    /// A `blob:` URL for the audio. Play it in an `audio` element or `fetch` it for the bytes.
    pub url: String,
    /// The browser's recording format, such as `audio/webm;codecs=opus`.
    pub mime_type: String,
    /// Size in bytes.
    pub size: u64,
}

/// The recorder script. Commands arrive as strings (`start`, `pause`, `resume`, `stop`, `release|url`);
/// events go out as `kind|field|...`. `tick|ms` is a monotonic clock reading.
const SCRIPT: &str = r"
let stream, recorder, frame, chunks = [];
const release = () => {
  cancelAnimationFrame(frame);
  if (stream) stream.getTracks().forEach((t) => t.stop());
  stream = undefined;
};
while (true) {
  const command = await dioxus.recv();
  if (command === 'start') {
    if (!navigator.mediaDevices || typeof MediaRecorder === 'undefined') { dioxus.send('unsupported'); continue; }
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      chunks = [];
      recorder = new MediaRecorder(stream);
      recorder.ondataavailable = (e) => { if (e.data.size) chunks.push(e.data); };
      recorder.onstop = () => {
        release();
        const blob = new Blob(chunks, { type: recorder.mimeType });
        dioxus.send('recorded|' + URL.createObjectURL(blob) + '|' + blob.size + '|' + blob.type);
        dioxus.send('stopped');
      };
      recorder.start();
      dioxus.send('granted');
      const tick = () => { dioxus.send('tick|' + Math.round(performance.now())); frame = requestAnimationFrame(tick); };
      tick();
    } catch (e) {
      release();
      const name = e && e.name;
      dioxus.send(['NotAllowedError', 'PermissionDeniedError', 'SecurityError'].includes(name) ? 'denied'
        : ['NotFoundError', 'OverconstrainedError'].includes(name) ? 'nodevice' : 'failed');
    }
  } else if (command === 'pause') { if (recorder && recorder.state === 'recording') recorder.pause(); }
  else if (command === 'resume') { if (recorder && recorder.state === 'paused') recorder.resume(); }
  else if (command === 'stop') { if (recorder && recorder.state !== 'inactive') recorder.stop(); else release(); }
  else if (command.startsWith('release|')) URL.revokeObjectURL(command.slice(8));
}
";

/// One icon in a button.
#[component]
fn Glyph(markup: &'static str) -> Element {
    rsx! {
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
            dangerous_inner_html: markup,
        }
    }
}

/// Record from the microphone. Start, pause and resume, then stop; the result
/// plays back in place and arrives in `on_recorded` as a [`RecordedAudio`].
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn RecordingControl(
    /// Stop on its own after this many seconds of recording.
    max_seconds: Option<u32>,
    /// Hide the pause button.
    #[props(default)]
    no_pause: bool,
    /// Block recording.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Accessible name of the group. Default "Recording".
    label: Option<String>,
    /// Called with the audio when a recording ends.
    on_recorded: Option<EventHandler<RecordedAudio>>,
    /// Called whenever the recorder phase changes.
    on_phase_change: Option<EventHandler<RecordingPhase>>,
) -> Element {
    let input = use_memo(move || RecordingControlInput { max_ms: max_seconds.map(|s| u64::from(s) * 1000), disabled: disabled(), no_pause });
    let mut state = use_signal(|| RecordingControlMachine::initial(&input.peek()));
    let mut result = use_signal(|| None::<RecordedAudio>);
    let eval = use_hook(|| document::eval(SCRIPT));

    let mut send = move |event: RecordingControlEvent| {
        let step = RecordingControlMachine::transition(&state.peek(), event, &input.peek());
        let before = state.peek().phase;
        let after = step.state.as_ref().map(|s| s.phase);
        if let Some(next) = step.state {
            state.set(next);
        }
        if let Some(after) = after
            && before != after
            && let Some(handler) = on_phase_change
        {
            handler.call(after);
        }
        for command in step.commands {
            let _ = match command {
                RecordingControlCommand::Start => eval.send("start".to_string()),
                RecordingControlCommand::Pause => eval.send("pause".to_string()),
                RecordingControlCommand::Resume => eval.send("resume".to_string()),
                RecordingControlCommand::Stop => eval.send("stop".to_string()),
                RecordingControlCommand::Release => {
                    if let Some(old) = result.take() {
                        let _ = eval.send(format!("release|{}", old.url));
                    }
                    Ok(())
                }
            };
        }
    };

    use_future(move || async move {
        let mut eval = eval;
        while let Ok(message) = eval.recv::<String>().await {
            let mut parts = message.split('|');
            match parts.next() {
                Some("granted") => send(RecordingControlEvent::Granted),
                Some("stopped") => send(RecordingControlEvent::Stopped),
                Some("denied") => send(RecordingControlEvent::Failed(RecordingError::Denied)),
                Some("nodevice") => send(RecordingControlEvent::Failed(RecordingError::NoDevice)),
                Some("unsupported") => send(RecordingControlEvent::Failed(RecordingError::Unsupported)),
                Some("failed") => send(RecordingControlEvent::Failed(RecordingError::Failed)),
                Some("tick") => {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a clock reading is positive and small")]
                    let now = parts.next().and_then(|p| p.parse::<f64>().ok()).unwrap_or(0.0) as u64;
                    send(RecordingControlEvent::Tick(now));
                }
                Some("recorded") => {
                    let url = parts.next().unwrap_or_default().to_string();
                    let size = parts.next().and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
                    let mime_type = parts.next().unwrap_or_default().to_string();
                    let audio = RecordedAudio { url, mime_type, size };
                    result.set(Some(audio.clone()));
                    if let Some(handler) = on_recorded {
                        handler.call(audio);
                    }
                }
                _ => {}
            }
        }
    });
    use_drop(move || {
        let _ = eval.send("stop".to_string());
    });

    let s = *state.read();
    let busy = matches!(s.phase, RecordingPhase::Requesting | RecordingPhase::Stopping).then_some("true");
    let off = disabled();
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    let can_pause = s.can_pause(&input.read());
    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": label,
            "aria-busy": busy,
            "data-state": s.phase.as_str(),
            div { "data-scope": h::SCOPE, "data-part": "buttons",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "start",
                    "aria-label": h::start_label(s.phase),
                    disabled: off || !s.can_start(),
                    onclick: move |_| send(RecordingControlEvent::Start),
                    Glyph { markup: h::MIC }
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "pause",
                    "aria-label": h::pause_label(s.phase),
                    hidden: no_pause,
                    disabled: off || !can_pause,
                    onclick: move |_| send(if state.peek().phase == RecordingPhase::Paused { RecordingControlEvent::Resume } else { RecordingControlEvent::Pause }),
                    Glyph { markup: if s.phase == RecordingPhase::Paused { h::PLAY } else { h::PAUSE } }
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "stop",
                    "aria-label": h::STOP_LABEL,
                    disabled: off || !s.can_stop(),
                    onclick: move |_| send(RecordingControlEvent::Stop),
                    Glyph { markup: h::STOP }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "elapsed", {h::format_elapsed(s.elapsed_ms)} }
            span { "data-scope": h::SCOPE, "data-part": "status", role: "status", "aria-live": "polite", {h::status_text(&s)} }
            if let Some(r) = result() {
                div { "data-scope": h::SCOPE, "data-part": "result",
                    audio { controls: true, src: "{r.url}" }
                    {
                        let name = h::file_name("1", &r.mime_type);
                        rsx! {
                            a { "data-scope": h::SCOPE, "data-part": "download", href: "{r.url}", download: "{name}", "Download {name} ({h::format_size(r.size)})" }
                        }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "discard",
                        onclick: move |_| send(RecordingControlEvent::Discard),
                        "Discard"
                    }
                }
            }
        }
    }
}
