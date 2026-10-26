use dioxus::prelude::*;
use oxide_dioxus::media_timeline::{Marker, MediaKind, MediaTimeline};

/// Builds a short WAV in the webview and returns an object URL for it.
const TONE: &str = r"
const rate = 8000, seconds = 24, notes = [261.63, 329.63, 392, 523.25, 392, 329.63, 293.66, 349.23, 440, 587.33, 440, 349.23];
const data = new Uint8Array(44 + rate * seconds);
const view = new DataView(data.buffer);
const text = (at, s) => [...s].forEach((c, i) => view.setUint8(at + i, c.charCodeAt(0)));
text(0, 'RIFF'); view.setUint32(4, 36 + rate * seconds, true); text(8, 'WAVEfmt ');
view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, 1, true);
view.setUint32(24, rate, true); view.setUint32(28, rate, true); view.setUint16(32, 1, true); view.setUint16(34, 8, true);
text(36, 'data'); view.setUint32(40, rate * seconds, true);
const per = seconds / notes.length;
for (let i = 0; i < rate * seconds; i++) {
  const t = i / rate, f = notes[Math.floor(t / per) % notes.length], local = t % per;
  const env = Math.min(1, local * 20) * Math.max(0, 1 - (local / per) * 0.9);
  const v = 0.5 * Math.sin(2 * Math.PI * f * t) + 0.2 * Math.sin(4 * Math.PI * f * t);
  data[44 + i] = 128 + 110 * env * v;
}
return URL.createObjectURL(new Blob([data], { type: 'audio/wav' }));
";

#[component]
pub fn MediaTimelineDemo() -> Element {
    let src = use_resource(|| async { document::eval(TONE).join::<String>().await.unwrap_or_default() });

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 560px;",
            if let Some(src) = src.read().clone().filter(|s| !s.is_empty()) {
                // A native <audio> underneath, with chapters as ticks. Try the arrow keys on the thumb.
                MediaTimeline {
                    src: src.clone(),
                    label: "Demo melody",
                    markers: vec![Marker::new(0.0, "Opening phrase"), Marker::new(8.0, "Rising line"), Marker::new(16.0, "Resolution")],
                }

                // Disabled.
                MediaTimeline { src, kind: MediaKind::Audio, label: "Unavailable clip", disabled: true }
            }
        }
    }
}
