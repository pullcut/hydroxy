use leptos::prelude::*;
use oxide_leptos::filter_chip::FilterChip;
use oxide_leptos::icon::IconName;

pub fn demo() -> AnyView {
    let (overdue, set_overdue) = signal(true);
    let (mine, set_mine) = signal(false);
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
            <FilterChip label="Status" count=2 active=true dropdown=true />
            <FilterChip label="Owner" dropdown=true />
            <FilterChip
                label="Overdue"
                icon=IconName::CircleAlert
                active=overdue
                on_click=move || set_overdue.update(|v| *v = !*v)
            />
            <FilterChip label="Mine" icon=IconName::User active=mine on_click=move || set_mine.update(|v| *v = !*v) />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "filter-chip",
    name: "Filter chip",
    group: "Forms",
    summary: "A dashed toggle that turns solid amethyst when active. Pair with Menu, using dropdown, for filters with several values.",
    demo,
    leptos: include_str!("filter_chip.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/filter_chip.rs"),
    parts: &[
        &["FilterChip", "—", "button[data-scope=filter-chip] with aria-pressed"],
        &["", "count", "Selected-value count in mono"],
    ],
    keys: &[&["Enter / Space", "Toggles the filter, or opens its menu"]],
    props: &[
        &["label", "String", "Required. The filter's name"],
        &["icon", "IconName", "Leading icon"],
        &["count", "u32", "How many values are selected"],
        &["active", "bool", "On. Reported as aria-pressed"],
        &["dropdown", "bool", "Adds a chevron for chips that open a menu"],
        &["on_click", "Callback<()>", "Toggles or opens the filter"],
    ],
};
