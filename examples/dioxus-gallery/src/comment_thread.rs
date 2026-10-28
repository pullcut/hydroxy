use dioxus::prelude::*;
use hydroxy_dioxus::comment_thread::{CommentData, CommentThread, Jewel};

// A fixed "now" keeps output identical across renders.
const NOW: i64 = 1_792_990_000;

#[component]
pub fn CommentThreadDemo() -> Element {
    let mut comments = use_signal(|| {
        vec![
            CommentData::new(
                "c1",
                "Priya Nair",
                "PN",
                NOW - 5 * 3600,
                "The renewal figure here looks higher than the dashboard. Which export is this from?",
            )
            .tint(Jewel::Sapphire)
            .replies(vec![
                CommentData::new(
                    "c1r1",
                    "Tom Deakin",
                    "TD",
                    NOW - 3 * 3600,
                    "September's. I'll swap in the October numbers.",
                )
                .tint(Jewel::Amethyst),
                CommentData::new("c1r2", "Priya Nair", "PN", NOW - 3000, "Thanks, that matches now.")
                    .tint(Jewel::Sapphire),
            ]),
            CommentData::new(
                "c2",
                "Jo Marsh",
                "JM",
                NOW - 2 * 86_400,
                "Can we cut the second paragraph? It repeats the summary.",
            )
            .tint(Jewel::Emerald),
            CommentData::new(
                "c3",
                "Tom Deakin",
                "TD",
                NOW - 6 * 86_400,
                "Added sources for every claim in section two.",
            )
            .tint(Jewel::Amethyst),
        ]
    });
    let mut next = use_signal(|| 0_u32);
    let mut new_id = move || {
        next += 1;
        format!("n{}", *next.peek())
    };

    rsx! {
        div { style: "max-width: 560px;",
            CommentThread {
                comments,
                now: NOW,
                default_resolved: vec!["c3".to_string()],
                on_reply: move |(thread, text): (String, String)| {
                    let reply = CommentData::new(new_id(), "You", "YO", NOW, text);
                    if let Some(t) = comments.write().iter_mut().find(|t| t.id == thread) {
                        t.replies.push(reply);
                    }
                },
                on_comment: move |text: String| {
                    let comment = CommentData::new(new_id(), "You", "YO", NOW, text);
                    comments.write().push(comment);
                },
            }
        }
    }
}
