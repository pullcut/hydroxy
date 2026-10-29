use hydroxy_leptos::transcript_viewer::{TranscriptSegment, TranscriptViewer};
use leptos::prelude::*;

fn segments() -> Vec<TranscriptSegment> {
    let lines = [
        ("Ana", "Welcome back to the show. Today we are talking about how teams ship audio features."),
        ("Ben", "Thanks for having me. I have been building voice tools for about six years now."),
        ("Ana", "Let's start with the basics. What does a transcript give a listener?"),
        ("Ben", "A way in. You can scan it, search it, and jump to the part you care about."),
        ("Ana", "And the highlighted line follows along as the audio plays."),
        ("Ben", "Right, until you scroll away. Then it should leave you alone."),
        ("Ana", "That is the part people get wrong. The view fights the reader."),
        ("Ben", "So we pause following, and offer a button to catch up again."),
        ("Ana", "Which brings us to search. Do you filter the list or highlight matches?"),
        ("Ben", "Filtering keeps the page short. You still click a line to seek to it."),
        ("Ana", "Last question: what would you change about your first version?"),
        ("Ben", "I would write the timing checks earlier. Overlapping segments caused most of our bugs."),
    ];
    lines
        .iter()
        .enumerate()
        .map(|(i, (who, text))| {
            let start = f64::from(u8::try_from(i).unwrap_or(0)) * 6.0;
            TranscriptSegment::new(format!("s{i}"), *who, start, start + 6.0, *text)
        })
        .collect()
}

pub fn demo() -> AnyView {
    let (time, set_time) = signal(0.0_f64);
    let (playing, set_playing) = signal(false);
    // A stand-in for a real player's clock: 4 ticks a second while "playing".
    Effect::new(move |_| {
        let handle = set_interval_with_handle(
            move || {
                if playing.get_untracked() {
                    set_time.update(|t| *t = (*t + 0.25) % 72.0);
                }
            },
            std::time::Duration::from_millis(250),
        );
        on_cleanup(move || {
            if let Ok(h) = handle {
                h.clear();
            }
        });
    });

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 640px;">
            <div style="display: flex; align-items: center; gap: var(--space-4); font: var(--text-sm);">
                <button type="button" on:click=move |_| set_playing.update(|p| *p = !*p)>
                    {move || if playing.get() { "Pause" } else { "Play" }}
                </button>
                <span>{move || format!("{:.0} s", time.get())}</span>
            </div>
            // Scroll the list by hand and following stops. Click a line to jump to it.
            <TranscriptViewer
                segments=segments()
                current_time=time
                searchable=true
                max_height=260
                on_seek=move |t| set_time.set(t)
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "transcript-viewer",
    name: "Transcript viewer",
    group: "Display",
    summary: "Timed speech segments that highlight the line being spoken, scroll to follow playback, seek on click and filter by search. Following pauses when the reader scrolls by hand and resumes on request.",
    demo,
    leptos: include_str!("transcript_viewer.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/transcript_viewer.rs"),
    parts: &[
        &["TranscriptViewer", "root", "data-follow is following or suspended"],
        &["(inside)", "toolbar", "Holds the search field, the segment count and the follow button"],
        &["(inside)", "search", "Filters by speaker or text, ignoring case"],
        &["(inside)", "count", "role=status: 3 of 12 segments"],
        &["(inside)", "resume-follow", "Disabled while following"],
        &[
            "(inside)",
            "viewport",
            "Scrolling region, focusable. Wheel, touch, scroll keys and scrollbar drags suspend following",
        ],
        &["(inside)", "segment", "A button per segment. data-active marks the one being spoken; aria-current too"],
        &["(inside)", "speaker / time / text", "The pieces of one segment"],
    ],
    keys: &[
        &["Enter / Space", "On a segment: seek to its start"],
        &["↑ ↓ PgUp PgDn Home End", "On the viewport: scroll, and stop following"],
    ],
    props: &[
        &["segments", "Vec<Segment>", "id, speaker, start, end and text, in time order"],
        &["current_time", "f64", "Playback position in seconds"],
        &["on_seek", "Callback<f64>", "A segment was chosen: seek to this start time"],
        &["searchable", "bool", "Show the search field"],
        &["label", "Option<String>", "Accessible name of the scrolling region"],
        &["max_height", "u32", "Height of the region in pixels. Default 320"],
        &["start_suspended", "bool", "Start with following off"],
        &["on_follow_change", "Callback<bool>", "Following turned on or off"],
    ],
    events: &[],
};
