use leptos::prelude::*;
use hydroxy_leptos::audio_waveform::{AudioWaveform, Peaks, Range};

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

pub fn demo() -> AnyView {
    let (time, set_time) = signal(6.0_f64);
    let (range, set_range) = signal(Some(Range { start: 8.0, end: 14.0 }));
    let (looping, set_looping) = signal(false);

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 640px;">
            // Click or drag the track to move the playhead. The arrow keys step it.
            <AudioWaveform
                peaks=peaks()
                duration=Some(24.0)
                current_time=time
                selected_range=range
                looping=looping
                label="Voice memo"
                on_seek=move |t| set_time.set(t)
                on_range_change=move |r| set_range.set(Some(r))
                on_loop_request=move |on| set_looping.set(on)
            />

            // Zoomed to the second half. The scale still comes from the peaks.
            <AudioWaveform peaks=peaks() duration=Some(24.0) current_time=18.0 zoom=2.0 viewport_start=12.0 label="Zoomed" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "audio-waveform",
    name: "Audio waveform",
    group: "Display",
    summary: "A peak display for audio you already analysed: bars, a playhead you can click or drag, a selectable range with edge sliders, a loop toggle and a status line that flags bad input. The host owns the clock.",
    demo,
    leptos: include_str!("audio_waveform.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/audio_waveform.rs"),
    parts: &[
        &["AudioWaveform", "root", "data-has-peaks and data-issue describe the input"],
        &["(inside)", "track", "role=slider. Click, drag or use the keys to seek"],
        &["(inside)", "bars / bar", "One bar per peak bin in view. Decorative"],
        &["(inside)", "range", "The selected span, shaded"],
        &["(inside)", "playhead", "A line at the current time"],
        &["(inside)", "selection", "Holds the two edge sliders and the loop button when a range is set"],
        &["(inside)", "range-start / range-end", "Range inputs for the selection edges"],
        &["(inside)", "loop", "aria-pressed toggle"],
        &["(inside)", "status", "Position, selection and any problems with the input"],
    ],
    keys: &[
        &["← / ↓", "Step the playhead back by one second"],
        &["→ / ↑", "Step the playhead forward by one second"],
        &["Home / End", "Go to the start or end"],
    ],
    props: &[
        &["peaks", "Peaks", "Bin values (0 to 1) with samples_per_peak and sample_rate, which set the seconds per bin"],
        &["duration", "Option<f64>", "Length of the audio. Peaks that add up to a different length are flagged"],
        &["current_time", "f64", "Playhead in seconds"],
        &["selected_range", "Option<Range>", "The selection, clamped to the duration"],
        &["zoom / viewport_start", "f64", "Show a slice of the audio. The start is clamped inside it"],
        &["looping", "bool", "Whether the selection is looping"],
        &["label", "Option<String>", "Accessible name of the track"],
        &["on_seek", "Callback<f64>", "The reader picked a time"],
        &["on_range_change", "Callback<Range>", "A selection edge moved. Edges never cross"],
        &["on_loop_request", "Callback<bool>", "The loop toggle was pressed; the value is the new state"],
    ],
};
