use hydroxy_leptos::media_timeline::{Marker, MediaKind, MediaTimeline};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 560px;">
            // A native <audio> underneath, with chapters as ticks. Try the arrow keys on the thumb.
            <MediaTimeline
                src="/demo-tone.wav"
                label="Demo melody"
                markers=vec![
                    Marker::new(0.0, "Opening phrase"),
                    Marker::new(8.0, "Rising line"),
                    Marker::new(16.0, "Resolution"),
                ]
            />

            // Disabled.
            <MediaTimeline src="/demo-tone.wav" kind=MediaKind::Audio label="Unavailable clip" disabled=true />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "media-timeline",
    name: "Media timeline",
    group: "Display",
    summary: "A scrubber over a native audio or video element: play button, buffered and played ranges, chapter ticks, a hover time preview and keyboard seeking.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("media_timeline.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/media_timeline.rs"),
    parts: &[
        &["MediaTimeline", "root", "Wraps everything. data-state is playing or paused; data-kind is audio or video"],
        &["(inside)", "media", "The native <audio> or <video>. Hidden for audio"],
        &["(inside)", "play", "Play and pause button"],
        &["(inside)", "current / duration", "Elapsed and total time"],
        &["(inside)", "track", "Pointer target. Click or drag to seek; data-scrubbing while held"],
        &["(inside)", "buffered / played", "The loaded range and the played range"],
        &["(inside)", "marker", "A chapter tick, with the chapter name as its title"],
        &["(inside)", "preview", "Time under the pointer while hovering"],
        &["(inside)", "thumb", "role=slider with aria-valuetext such as \"0:12 of 0:24\""],
        &["(inside)", "chapter", "The name of the chapter the playhead is in"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Seek back or forward by step seconds (Shift: a fifth of it)"],
        &["PageUp / PageDown", "Seek by a tenth of the duration"],
        &["Home / End", "Jump to the start or the end"],
        &["Space / K", "Play or pause"],
    ],
    props: &[
        &["src", "String", "Required. URL of the audio or video file"],
        &["label", "String", "Required. Accessible name for the scrubber"],
        &["kind", "MediaKind", "Audio (default) or Video"],
        &["markers", "Vec<Marker>", "Chapters as (time in seconds, label)"],
        &["step", "f64", "Seconds an arrow key moves. Default 5"],
        &["disabled", "bool", "Block seeking and playback controls"],
    ],
    events: &[],
};
