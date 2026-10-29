use hydroxy_leptos::search_field::*;
use leptos::prelude::*;

const SOURCES: [&str; 5] =
    ["interviews-q3.pdf", "tickets-sept.csv", "tickets-aug.csv", "survey-results.xlsx", "onboarding-notes.md"];

pub fn demo() -> AnyView {
    let (query, set_query) = signal(String::new());
    let matches = move || {
        let q = query.get().to_lowercase();
        SOURCES.iter().filter(|s| s.contains(q.as_str())).copied().collect::<Vec<_>>()
    };

    view! {
        <div style="display: grid; gap: var(--space-8);">
            // Controlled: filters the list as you type. Esc or the cross clears it.
            <div style="display: grid; gap: var(--space-4); max-width: 360px;">
                <SearchField
                    label="Search sources"
                    placeholder="Search sources"
                    value=query
                    on_input=move |v| set_query.set(v)
                />
                <p style="margin: 0; font: var(--text-label); letter-spacing: var(--tracking-label); text-transform: uppercase; color: var(--text-3);">
                    {move || format!("{} of {} sources", matches().len(), SOURCES.len())}
                </p>
                <ul style="margin: 0; padding: 0; list-style: none; display: grid; gap: var(--space-2); font: var(--text-sm);">
                    {move || matches().into_iter().map(|s| view! { <li>{s}</li> }).collect_view()}
                </ul>
            </div>
            // Uncontrolled, large: for page-level search.
            <SearchField label="Search the workspace" placeholder="Search the workspace" size=Size::Lg />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "search-field",
    name: "Search field",
    group: "Forms",
    summary: "A search input with a clear button. Use the large size for page-level search.",
    demo,
    leptos: include_str!("search_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/search_field.rs"),
    parts: &[
        &["SearchField", "root", "The bordered row. Takes the focus ring via :focus-within. data-size"],
        &["SearchField", "icon", "Search icon, aria-hidden"],
        &["SearchField", "control", "Native <input type=search>, so role=searchbox. aria-label from label"],
        &["SearchField", "clear", "<button aria-label=\"Clear search\">, hidden while empty"],
    ],
    keys: &[
        &["Escape", "Clears the text. When already empty, passes through, so a dialog can close"],
        &["Enter", "Native: submits the surrounding form, if any"],
    ],
    props: &[
        &["label", "String", "Accessible name. Default Search"],
        &["value", "Option<String>", "Controlled value. Pair with on_input"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["on_input", "Callback<String>", "Called on every keystroke, and with an empty string on clear"],
        &["placeholder", "String", "Default Search"],
        &["name", "String", "Form field name"],
        &["disabled", "bool", "Block editing"],
        &["size", "Size", "Md (40px, default) or Lg (48px) for page-level search"],
    ],
    events: &[],
};
