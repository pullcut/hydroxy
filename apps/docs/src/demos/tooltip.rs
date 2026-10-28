use hydroxy_leptos::tooltip::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-4);">
            // Default: above the trigger, centred.
            <Tooltip>
                <TooltipTrigger>"Copy link"</TooltipTrigger>
                <TooltipContent>"Copies a link to this summary"</TooltipContent>
            </Tooltip>

            <Tooltip placement=Placement::new(Side::Bottom, Align::Start)>
                <TooltipTrigger>"Sources"</TooltipTrigger>
                <TooltipContent>"3 files, read 26 October"</TooltipContent>
            </Tooltip>

            <Tooltip placement=Placement::new(Side::End, Align::Center)>
                <TooltipTrigger>"Retry"</TooltipTrigger>
                <TooltipContent>"Runs the checks again"</TooltipContent>
            </Tooltip>

            <Tooltip disabled=true>
                <TooltipTrigger>"No tooltip"</TooltipTrigger>
                <TooltipContent>"Never shown"</TooltipContent>
            </Tooltip>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tooltip",
    name: "Tooltip",
    group: "Overlays",
    summary: "A short hint for a control. Shows on hover or keyboard focus and flips to stay on screen.",
    demo,
    leptos: include_str!("tooltip.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tooltip.rs"),
    parts: &[
        &["Tooltip", "(none)", "Provides the machine. Renders no element"],
        &["TooltipTrigger", "trigger", "<button>, aria-describedby, anchor-name, data-state"],
        &["TooltipContent", "content", "role=tooltip, popover=manual, data-side, data-state"],
    ],
    keys: &[&["Tab", "Focusing the trigger opens the tooltip; leaving closes it"], &["Escape", "Closes the tooltip"]],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled"],
        &["disabled", "bool", "Never open"],
        &["placement", "Placement", "Side and alignment. Default: top, centre"],
        &["on_open_change", "Callback<bool>", "Called when hover, focus or Esc opens or closes it"],
        &["variant", "Option<ButtonVariant>", "On TooltipTrigger: the button look. Default secondary"],
    ],
};
