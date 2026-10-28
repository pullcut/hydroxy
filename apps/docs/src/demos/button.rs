use hydroxy_leptos::button::*;
use hydroxy_leptos::icon::IconName;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (drafts, set_drafts) = signal(0);

    view! {
        <div style="display: grid; gap: var(--space-7);">
            // Variants. One primary or accent action per view.
            <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);">
                <Button>"Open the project"</Button>
                <Button
                    variant=ButtonVariant::Accent
                    icon_left=IconName::Plus
                    on_click=move |_| set_drafts.update(|n| *n += 1)
                >
                    "Draft summary"
                </Button>
                <Button variant=ButtonVariant::Secondary>"Export sources"</Button>
                <Button variant=ButtonVariant::Ghost icon_right=IconName::ArrowRight>"View all"</Button>
                <Button variant=ButtonVariant::Danger icon_left=IconName::Trash>"Delete project"</Button>
            </div>

            // Sizes: 32, 40 and 52px.
            <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);">
                <Button size=ButtonSize::Sm>"Book a demo"</Button>
                <Button size=ButtonSize::Md>"Book a demo"</Button>
                <Button size=ButtonSize::Lg>"Book a demo"</Button>
            </div>

            // Disabled, and stretched to the container.
            <Button variant=ButtonVariant::Secondary disabled=true full_width=true>
                "Share with workspace"
            </Button>

            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">"Drafts started: " {drafts}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "button",
    name: "Button",
    group: "Actions",
    summary: "The main action control. Use one primary or accent button per view, and say what happens: \"Draft summary\", not \"OK\".",
    demo,
    leptos: include_str!("button.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/button.rs"),
    parts: &[&["Button", "-", "<button class=ox-button> with data-variant, data-size and data-full-width"]],
    keys: &[],
    props: &[
        &["variant", "ButtonVariant", "Primary (default), Accent, Secondary, Ghost or Danger"],
        &["size", "ButtonSize", "Sm (32px), Md (40px, default) or Lg (52px)"],
        &["icon_left / icon_right", "IconName", "Optional icon before or after the label"],
        &["full_width", "bool", "Stretch to the container's width"],
        &["disabled", "bool", "Greyed out and not clickable"],
        &["button_type", "ButtonType", "The type attribute: Button (default), Submit or Reset"],
        &["on_click", "Callback<MouseEvent>", "Called on click"],
    ],
};
