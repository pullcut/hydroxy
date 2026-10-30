use hydroxy_leptos::rating::Rating;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (score, set_score) = signal(3.5);

    view! {
        <div style="display: grid; gap: var(--space-6); justify-items: start;">
            // Controlled, half stars, click the current score again to clear.
            <div style="display: flex; align-items: center; gap: var(--space-4);">
                <Rating
                    label="Answer quality"
                    value=score
                    allow_half=true
                    clearable=true
                    on_value_change=move |v| set_score.set(v)
                />
                <span style="font: 500 12px/1 var(--font-mono); color: var(--text-2);">
                    {move || format!("{} / 5", score.get())}
                </span>
            </div>

            // Uncontrolled, three stars, submitted with a form.
            <Rating label="Difficulty" count=3 default_value=2.0 size=16 name="difficulty" />

            // Display only.
            <Rating default_value=4.5 allow_half=true read_only=true />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "rating",
    name: "Rating",
    group: "Forms",
    summary: "Score something with stars. A radio group with an optional half step, a hover preview and a read-only mode for showing a score.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("rating.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/rating.rs"),
    parts: &[
        &[
            "Rating",
            "root",
            "role=radiogroup, or role=img with \"Rated 4.5 out of 5\" when read-only. data-readonly, data-disabled",
        ],
        &[
            "(inside)",
            "star",
            "<button role=radio> with \"3 stars\". data-state=full|half|empty from the hover preview or value",
        ],
        &["(inside)", "half", "Left-half hit area that picks n - 0.5. Only with allow_half"],
        &["(inside)", "icon", "Outline star; its fill child is clipped to show full or half"],
        &["(inside)", "hidden input", "<input type=hidden> when named"],
    ],
    keys: &[
        &["ArrowRight / ArrowUp", "Up one step (half or whole)"],
        &["ArrowLeft / ArrowDown", "Down one step. Stops at the first step unless clearable"],
        &["Home / End", "Lowest / highest"],
        &["Space / Enter", "Pick the focused star"],
        &["Tab", "Into the group at the current star, then out"],
    ],
    props: &[
        &["label", "String", "Group name. Default \"Rating\""],
        &["count", "usize", "Number of stars. Default 5"],
        &["value", "Option<f64>", "Controlled value, 0 for unrated. Pair with on_value_change"],
        &["default_value", "f64", "Initial value when uncontrolled. Default 0"],
        &["allow_half", "bool", "Half steps"],
        &["clearable", "bool", "Picking the current value again clears it"],
        &["read_only", "bool", "Show the score only"],
        &["disabled", "bool", "Ignore picks and leave the tab order"],
        &["size", "u16", "Star size in px. Default 20"],
        &["name", "String", "Form field name"],
        &["on_value_change", "Callback<f64>", "Called with the new value"],
    ],
    events: &[],
};
