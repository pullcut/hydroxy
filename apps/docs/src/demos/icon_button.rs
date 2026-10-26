use leptos::prelude::*;
use oxide_leptos::button::{ButtonSize, ButtonVariant};
use oxide_leptos::icon::IconName;
use oxide_leptos::icon_button::IconButton;

pub fn demo() -> AnyView {
    let (copied, set_copied) = signal(false);

    view! {
        <div style="display: grid; gap: var(--space-7);">
            // Variants. Ghost is the default for toolbars.
            <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);">
                <IconButton icon=IconName::Copy label="Copy link" on_click=move |_| set_copied.set(true) />
                <IconButton icon=IconName::Ellipsis label="More actions" variant=ButtonVariant::Secondary />
                <IconButton icon=IconName::Plus label="New project" variant=ButtonVariant::Primary />
                <IconButton icon=IconName::Search label="Search sources" variant=ButtonVariant::Accent />
                <IconButton icon=IconName::Trash label="Delete project" disabled=true />
            </div>

            // Sizes: 32, 40 and 48px squares.
            <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);">
                <IconButton icon=IconName::X label="Close" variant=ButtonVariant::Secondary size=ButtonSize::Sm />
                <IconButton icon=IconName::X label="Close" variant=ButtonVariant::Secondary size=ButtonSize::Md />
                <IconButton icon=IconName::X label="Close" variant=ButtonVariant::Secondary size=ButtonSize::Lg />
            </div>

            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || if copied.get() { "Link copied." } else { "Hover a button to see its label." }}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "icon-button",
    name: "Icon button",
    group: "Actions",
    summary: "A square, icon-only button for toolbars and compact actions. Its label is the accessible name and the hover title.",
    demo,
    leptos: include_str!("icon_button.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/icon_button.rs"),
    parts: &[&["IconButton", "-", "<button class=ox-icon-button> with aria-label, title, data-variant and data-size"]],
    keys: &[],
    props: &[
        &["icon", "IconName", "Required. The icon, drawn at 45% of the button's size"],
        &["label", "String", "Required. What the button does. Becomes aria-label and title"],
        &["variant", "ButtonVariant", "Ghost (default), Secondary, Primary or Accent"],
        &["size", "ButtonSize", "Sm (32px), Md (40px, default) or Lg (48px)"],
        &["disabled", "bool", "Greyed out and not clickable"],
        &["on_click", "Callback<MouseEvent>", "Called on click"],
    ],
};
