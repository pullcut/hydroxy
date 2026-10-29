use hydroxy_leptos::button::ButtonVariant;
use hydroxy_leptos::popover::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (open, set_open) = signal(Some(false));

    view! {
        <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-5);">
            // Uncontrolled. Opens below the trigger and focuses the first button inside.
            <Popover>
                <PopoverTrigger>"Sharing"</PopoverTrigger>
                <PopoverContent>
                    <p style="margin: 0 0 4px; font: var(--text-ui);">"Shared with the workspace"</p>
                    <p style="margin: 0 0 12px; color: var(--text-2);">
                        "22 of 24 people have opened it. Sources stay attached."
                    </p>
                    <PopoverClose variant=ButtonVariant::Secondary>"Done"</PopoverClose>
                </PopoverContent>
            </Popover>

            // Controlled, to the side. Escape closes and returns focus to the trigger.
            <Popover
                open=open
                on_open_change=move |v| set_open.set(Some(v))
                placement=Placement::new(Side::End, Align::Start)
            >
                <PopoverTrigger variant=ButtonVariant::Ghost>"Where this figure comes from"</PopoverTrigger>
                <PopoverContent>
                    <p style="margin: 0; color: var(--text-2);">
                        "Nine of twelve interviewees raised onboarding. From interviews-q3.pdf, pages 4 to 11."
                    </p>
                </PopoverContent>
            </Popover>
            <span>"Open: " {move || open.get().unwrap_or(false).to_string()}</span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "popover",
    name: "Popover",
    group: "Overlays",
    summary: "A non-modal panel anchored to a button, for details or a small form. Escape or a click outside closes it.",
    demo,
    leptos: include_str!("popover.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/popover.rs"),
    parts: &[
        &["Popover", "(none)", "Provides the machine. Renders no element"],
        &[
            "PopoverTrigger",
            "trigger",
            "button: aria-haspopup=dialog, aria-expanded, aria-controls, popovertarget, CSS anchor",
        ],
        &["PopoverContent", "content", "role=dialog, popover=auto, tabindex=-1, anchored with flip, data-side"],
        &["PopoverClose", "close", "button that closes and returns focus to the trigger"],
    ],
    keys: &[
        &["Enter / Space", "On the trigger: opens and focuses the first focusable element inside"],
        &["Tab", "Moves through the panel's content, then on through the page. The panel stays open"],
        &["Escape", "Closes and returns focus to the trigger"],
    ],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled. Default false"],
        &["placement", "Placement", "Side and alignment. Default below, centred"],
        &["on_open_change", "Callback<bool>", "Called when the panel opens or closes"],
        &[
            "variant",
            "Option<ButtonVariant>",
            "On PopoverTrigger: the button look, default secondary. On PopoverClose: opt into the button look",
        ],
    ],
    events: &[],
};
