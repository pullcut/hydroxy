use hydroxy_leptos::audio_player::AudioPlayer;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (time, set_time) = signal(0.0_f64);

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 560px;">
            // Space or K plays, the arrows skip, Up and Down change the volume, M mutes, < and > change the rate.
            <AudioPlayer
                src="/demo-tone.wav"
                title="Demo melody"
                transcript_href="#transcript"
                on_time=move |t| set_time.set(t)
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || format!("Position reported to the page: {:.1} s", time.get())}
            </p>

            // A source that fails shows the reason and a retry button.
            <AudioPlayer src="/missing.wav" title="Broken source" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "audio-player",
    name: "Audio player",
    group: "Display",
    summary: "A compact player over a native audio element: play, seek, volume, playback rate, a status line that explains buffering and failures, a retry button and an optional transcript link.",
    demo,
    leptos: include_str!("audio_player.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/audio_player.rs"),
    parts: &[
        &[
            "AudioPlayer",
            "root",
            "role=group. data-status is idle, loading, ready, playing, buffering, paused, ended, blocked or error",
        ],
        &["(inside)", "media", "The native audio element. Visible with its own controls until the custom ones attach"],
        &["(inside)", "controls", "role=toolbar around the buttons and sliders"],
        &["(inside)", "play", "Play, pause or replay. aria-pressed while playing"],
        &["(inside)", "seek", "Range input with a spoken aria-valuetext, such as 0:05 of 3:20"],
        &["(inside)", "time", "Elapsed and total"],
        &["(inside)", "mute / volume", "Mute toggle and a range input"],
        &["(inside)", "rate", "Cycles 0.75×, 1×, 1.25×, 1.5×, 2×"],
        &["(inside)", "status", "role=status text that follows the player's state"],
        &["(inside)", "retry", "Appears after a failure and reloads the source"],
        &["(inside)", "transcript", "A link, or a button when you pass on_transcript_request"],
    ],
    keys: &[
        &["Space / K", "Play or pause (Space on a button still presses it)"],
        &["← / →", "Skip back or forward by seek_step"],
        &["J / L", "Skip twice as far"],
        &["Home / End", "Go to the start or end"],
        &["↑ / ↓", "Volume up or down"],
        &["M", "Mute or unmute"],
        &["< / >", "Slower or faster"],
    ],
    props: &[
        &["src", "String", "URL of the audio file"],
        &["title", "Option<String>", "A visible heading that also names the player"],
        &["seek_step", "f64", "Seconds a skip moves. Default 5"],
        &["rates", "Option<Vec<f64>>", "Playback rates, ascending"],
        &["transcript_href", "Option<String>", "Shows a transcript link"],
        &["on_transcript_request", "Callback<()>", "Shows a transcript button instead of a link"],
        &["on_time", "Callback<f64>", "Called as playback moves, with the position in seconds"],
    ],
};
