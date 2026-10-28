use dioxus::prelude::*;
use hydroxy_dioxus::audio_player::AudioPlayer;

#[component]
pub fn AudioPlayerDemo() -> Element {
    let mut time = use_signal(|| 0.0_f64);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 560px;",
            // Space or K plays, the arrows skip, Up and Down change the volume, M mutes, < and > change the rate.
            AudioPlayer {
                src: "/demo-tone.wav",
                title: "Demo melody",
                transcript_href: "#transcript",
                on_time: move |t| time.set(t),
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Position reported to the page: {time():.1} s" }

            // A source that fails shows the reason and a retry button.
            AudioPlayer { src: "/missing.wav", title: "Broken source" }
        }
    }
}
