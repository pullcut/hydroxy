use hydroxy_leptos::comment_thread::{CommentData, CommentThread, Jewel};
use leptos::prelude::*;

// A fixed "now" keeps server and browser output identical.
const NOW: i64 = 1_792_990_000;

#[component]
pub fn Demo() -> impl IntoView {
    let (comments, set_comments) = signal(vec![
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
            CommentData::new("c1r2", "Priya Nair", "PN", NOW - 3000, "Thanks, that matches now.").tint(Jewel::Sapphire),
        ]),
        CommentData::new(
            "c2",
            "Jo Marsh",
            "JM",
            NOW - 2 * 86_400,
            "Can we cut the second paragraph? It repeats the summary.",
        )
        .tint(Jewel::Emerald),
        CommentData::new("c3", "Tom Deakin", "TD", NOW - 6 * 86_400, "Added sources for every claim in section two.")
            .tint(Jewel::Amethyst),
    ]);
    let next = StoredValue::new(0_u32);
    let new_id = move || {
        next.update_value(|n| *n += 1);
        format!("n{}", next.get_value())
    };

    view! {
        <div style="max-width: 560px;">
            <CommentThread
                comments=comments
                now=NOW
                default_resolved=vec!["c3".to_string()]
                on_reply=move |(thread, text): (String, String)| {
                    let reply = CommentData::new(new_id(), "You", "YO", NOW, text);
                    set_comments.update(|all| {
                        if let Some(t) = all.iter_mut().find(|t| t.id == thread) {
                            t.replies.push(reply);
                        }
                    });
                }
                on_comment=move |text: String| {
                    let comment = CommentData::new(new_id(), "You", "YO", NOW, text);
                    set_comments.update(|all| all.push(comment));
                }
            />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "comment-thread",
    name: "Comment thread",
    group: "Activity",
    summary: "Threaded comments for a document or record. Replies sit under their comment, a thread can be resolved and reopened, and resolved threads fold away.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("comment_thread.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/comment_thread.rs"),
    parts: &[
        &["CommentThread", "root", "Threads, then the optional new-comment composer"],
        &["", "threads", "ol of threads"],
        &["", "thread", "One thread. data-state: open or resolved"],
        &["", "comment", "Avatar plus content. data-reply on replies"],
        &["", "author / time / body", "Name, relative time, plain text"],
        &["", "replies", "Replies, indented under a rule"],
        &["", "actions", "Reply and Resolve"],
        &["", "resolved", "Summary row of a resolved thread with Show and Reopen"],
        &["", "reply", "Reply composer with a Cancel button"],
        &["", "new", "Composer for a new thread, when on_comment is set"],
    ],
    keys: &[
        &["Enter", "In a composer: send"],
        &["Escape", "In a reply box: close it"],
        &["Tab", "Moves through the actions of each thread in order"],
    ],
    props: &[
        &["comments", "Vec<Comment>", "Top-level comments, each carrying its replies"],
        &[
            "now",
            "i64",
            "Current unix time in seconds, for relative times like \"5h\". Pass it in so server and browser agree",
        ],
        &["resolved", "Option<Vec<String>>", "Controlled resolved thread ids. Pair with on_resolved_change"],
        &["default_resolved", "Vec<String>", "Resolved thread ids when uncontrolled"],
        &["on_resolved_change", "Callback<(String, bool)>", "Thread id and whether it is now resolved"],
        &["on_reply", "Callback<(String, String)>", "Thread id and reply text. You append the reply"],
        &["on_comment", "Callback<String>", "Text of a new thread. Shows a composer under the threads"],
        &["label", "String", "Accessible name. Default \"Comments\""],
    ],
    events: &[],
};
