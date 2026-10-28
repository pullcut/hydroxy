//! `VoiceInput`: a microphone button that records audio and reports a level meter.
//!
//! Dioxus has no typed handle for the media APIs, so recording runs in a small
//! script through `document::eval`. That works on web and in a desktop or mobile
//! webview, as long as the webview grants microphone access: on desktop the
//! platform may show no permission prompt and refuse, which surfaces as the
//! denied state. The recording comes back as an object URL that only that
//! webview can read, not as a `Blob`.

use dioxus::prelude::*;
use hydroxy_headless::voice_input::{
    self as h, VoiceInputCommand, VoiceInputEvent, VoiceInputInput, VoiceInputMachine, VoiceInputPhase,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

/// A finished recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedAudio {
    /// A `blob:` URL for the audio. Play it in an `audio` element or `fetch` it for the bytes.
    pub url: String,
    /// The browser's recording format, such as `audio/webm;codecs=opus`.
    pub mime_type: String,
    /// Size in bytes.
    pub size: u64,
    /// Length in milliseconds.
    pub duration_ms: u64,
}

/// The recorder script. Commands arrive as strings (`start`, `stop`); events go out as `kind|field|...`.
const SCRIPT: &str = r"
let stream, recorder, audio, frame, started, chunks = [];
const release = () => {
  cancelAnimationFrame(frame);
  if (stream) stream.getTracks().forEach((t) => t.stop());
  if (audio) audio.close().catch(() => {});
  stream = audio = undefined;
};
while (true) {
  const command = await dioxus.recv();
  if (command === 'start') {
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      chunks = [];
      recorder = new MediaRecorder(stream);
      recorder.ondataavailable = (e) => { if (e.data.size) chunks.push(e.data); };
      recorder.onstop = () => {
        const ms = Math.round(performance.now() - started);
        release();
        const blob = new Blob(chunks, { type: recorder.mimeType });
        dioxus.send('recorded|' + URL.createObjectURL(blob) + '|' + blob.size + '|' + ms + '|' + blob.type);
        dioxus.send('stopped');
      };
      audio = new AudioContext();
      const analyser = audio.createAnalyser();
      analyser.fftSize = 256;
      audio.createMediaStreamSource(stream).connect(analyser);
      const samples = new Uint8Array(analyser.fftSize);
      recorder.start();
      started = performance.now();
      dioxus.send('granted');
      const tick = () => {
        analyser.getByteTimeDomainData(samples);
        let power = 0;
        for (const b of samples) { const x = (b - 128) / 128; power += x * x; }
        const level = Math.min(1, Math.sqrt(power / samples.length) * 4);
        dioxus.send('level|' + level + '|' + Math.round(performance.now() - started));
        frame = requestAnimationFrame(tick);
      };
      tick();
    } catch (e) {
      release();
      const denied = e && ['NotAllowedError', 'PermissionDeniedError', 'SecurityError'].includes(e.name);
      dioxus.send(denied ? 'denied' : 'failed');
    }
  } else if (command === 'stop') {
    if (recorder && recorder.state === 'recording') recorder.stop(); else release();
  }
}
";

/// A button that records from the microphone, with a level meter. Press to
/// start, press again to stop. The recording arrives in `on_recorded`.
#[component]
pub fn VoiceInput(
    /// Stop on its own after this many seconds.
    max_seconds: Option<u32>,
    /// Block recording.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the audio when a recording ends.
    on_recorded: Option<EventHandler<RecordedAudio>>,
    /// Called whenever the recorder phase changes.
    on_phase_change: Option<EventHandler<VoiceInputPhase>>,
) -> Element {
    let input =
        use_memo(move || VoiceInputInput { max_ms: max_seconds.map(|s| u64::from(s) * 1000), disabled: disabled() });
    let state = use_signal(|| VoiceInputMachine::initial(&input.peek()));
    let eval = use_hook(|| document::eval(SCRIPT));

    let send = move |event: VoiceInputEvent| {
        let before = state.peek().phase;
        let commands = drive::<VoiceInputMachine>(state, input, event);
        let after = state.peek().phase;
        if before != after
            && let Some(handler) = on_phase_change
        {
            handler.call(after);
        }
        for command in commands {
            let _ = eval.send(match command {
                VoiceInputCommand::Start => "start",
                VoiceInputCommand::Stop => "stop",
            });
        }
    };

    use_future(move || async move {
        let mut eval = eval;
        while let Ok(message) = eval.recv::<String>().await {
            let mut parts = message.split('|');
            let number = |part: Option<&str>| part.and_then(|p| p.parse::<f64>().ok()).unwrap_or(0.0);
            match parts.next() {
                Some("granted") => send(VoiceInputEvent::Granted),
                Some("denied") => send(VoiceInputEvent::Denied),
                Some("failed") => send(VoiceInputEvent::Failed),
                Some("stopped") => send(VoiceInputEvent::Stopped),
                Some("level") => {
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "level is 0 to 1, time is small"
                    )]
                    {
                        send(VoiceInputEvent::Level(number(parts.next()) as f32));
                        send(VoiceInputEvent::Tick(number(parts.next()) as u64));
                    }
                }
                Some("recorded") => {
                    let url = parts.next().unwrap_or_default().to_string();
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "sizes and times are positive and small"
                    )]
                    let (size, duration_ms) = (number(parts.next()) as u64, number(parts.next()) as u64);
                    let mime_type = parts.next().unwrap_or_default().to_string();
                    if let Some(handler) = on_recorded {
                        handler.call(RecordedAudio { url, mime_type, size, duration_ms });
                    }
                }
                _ => {}
            }
        }
    });
    use_drop(move || {
        let _ = eval.send("stop");
    });

    let s = *state.read();
    let is_disabled = disabled();
    let asking = (s.phase == VoiceInputPhase::Requesting).then_some("true");
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-state": s.phase.as_str(),
            button {
                r#type: "button",
                "data-scope": h::SCOPE,
                "data-part": "button",
                "aria-pressed": h::pressed(s.phase).to_string(),
                "aria-label": h::button_label(s.phase),
                disabled: is_disabled,
                // Not `disabled` while asking: that would drop keyboard focus from the button.
                "aria-disabled": asking,
                onclick: move |_| send(VoiceInputEvent::Press),
                Icon { name: if s.phase == VoiceInputPhase::Recording { IconName::Stop } else { IconName::Mic }, size: 22 }
            }
            span { "data-scope": h::SCOPE, "data-part": "meter", "aria-hidden": "true",
                for i in 0..h::BARS {
                    span {
                        key: "{i}",
                        "data-scope": h::SCOPE,
                        "data-part": "bar",
                        style: format!("transform: scaleY({:.2})", h::bar_scale(i, s.level)),
                    }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "status", role: "status", {h::status_text(s.phase, s.elapsed_ms)} }
        }
    }
}
