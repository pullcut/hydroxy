use dioxus::prelude::*;
use hydroxy_dioxus::audio_waveform::{AudioWaveform, Peaks, Range};

/// 160 bins of a made-up voice memo: a swell, a pause, a louder swell.
fn peaks() -> Peaks {
    let values = (0..160_u16)
        .map(|i| {
            let x = f32::from(i) / 160.0;
            let swell = (x * std::f32::consts::PI * 3.0).sin().abs();
            let grain = ((f32::from(i) * 12.9898).sin() * 43758.547).fract().abs();
            (0.12 + swell * 0.6 + grain * 0.25).min(1.0)
        })
        .collect();
    // 24 seconds at 16 kHz, 2400 samples per bin: 0.15 s per bin.
    Peaks::new(values, 2400.0, 16000.0)
}

#[component]
pub fn AudioWaveformDemo() -> Element {
    let mut time = use_signal(|| 6.0_f64);
    let mut range = use_signal(|| Some(Range { start: 8.0, end: 14.0 }));
    let mut looping = use_signal(|| false);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 640px;",
            // Click or drag the track to move the playhead. The arrow keys step it.
            AudioWaveform {
                peaks: peaks(),
                duration: Some(24.0),
                current_time: time,
                selected_range: range,
                looping: looping(),
                label: "Voice memo",
                on_seek: move |t| time.set(t),
                on_range_change: move |r| range.set(Some(r)),
                on_loop_request: move |on| looping.set(on),
            }

            // Zoomed to the second half. The scale still comes from the peaks.
            AudioWaveform { peaks: peaks(), duration: Some(24.0), current_time: 18.0, zoom: 2.0, viewport_start: 12.0, label: "Zoomed" }
        }
    }
}
