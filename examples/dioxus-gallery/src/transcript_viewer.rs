use dioxus::prelude::*;
use hydroxy_dioxus::transcript_viewer::{TranscriptSegment, TranscriptViewer};

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

#[component]
pub fn TranscriptViewerDemo() -> Element {
    let mut time = use_signal(|| 0.0_f64);
    let mut playing = use_signal(|| false);
    // A stand-in for a real player's clock: 4 ticks a second while "playing".
    use_future(move || async move {
        loop {
            document::eval("await new Promise((r) => setTimeout(r, 250));").await.ok();
            if playing() {
                time.set((time() + 0.25) % 72.0);
            }
        }
    });

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 640px;",
            div { style: "display: flex; align-items: center; gap: var(--space-4); font: var(--text-sm);",
                button { r#type: "button", onclick: move |_| playing.set(!playing()), if playing() { "Pause" } else { "Play" } }
                span { "{time():.0} s" }
            }
            // Scroll the list by hand and following stops. Click a line to jump to it.
            TranscriptViewer {
                segments: segments(),
                current_time: time,
                searchable: true,
                max_height: 260,
                on_seek: move |t| time.set(t),
            }
        }
    }
}
