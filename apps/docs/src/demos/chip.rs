use hydroxy_leptos::chip::{Chip, Jewel};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (labels, set_labels) = signal(vec!["onboarding", "billing", "export"]);

    view! {
        <div style="display: grid; gap: var(--space-5);">
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-2);">
                <Chip jewel=Jewel::Garnet>"P0"</Chip>
                <Chip jewel=Jewel::Sapphire>"OX-142"</Chip>
                <Chip>"22 of 24"</Chip>
                <Chip mono=false>"Design review"</Chip>
            </div>

            // Removable.
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-2);">
                <For
                    each=move || labels.get()
                    key=|label| *label
                    children=move |label| {
                        view! {
                            <Chip
                                jewel=Jewel::Amethyst
                                on_remove=move |()| set_labels.update(|list| list.retain(|l| *l != label))
                            >
                                {label}
                            </Chip>
                        }
                    }
                />
            </div>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "chip",
    name: "Chip",
    group: "Display",
    summary: "A compact pill label, smaller than a tag, for dense rows and cards. Monospaced by default for ids, counts and priorities.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("chip.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/chip.rs"),
    parts: &[
        &["Chip", "(root)", "<span data-scope=chip>, data-jewel, data-mono"],
        &["(inside)", "diamond", "6px rotated square when jewel is set, aria-hidden"],
        &["(inside)", "remove", "<button aria-label=Remove> with an x icon, when on_remove is set"],
    ],
    keys: &[],
    props: &[
        &["jewel", "Option<Jewel>", "Show a small diamond in this colour"],
        &["mono", "bool", "Monospaced text. Default true"],
        &["on_remove", "Callback<()>", "Adds a remove button"],
        &["children", "View", "The label"],
    ],
    events: &[],
};
