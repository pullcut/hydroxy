use leptos::prelude::*;
use hydroxy_leptos::button::{Button, ButtonVariant};
use hydroxy_leptos::tour::{Side, Tour, TourStep};

pub fn demo() -> AnyView {
    let (open, set_open) = signal(false);
    let (outcome, set_outcome) = signal(String::new());
    let steps = vec![
        TourStep::centred("Welcome to Projects", "A quick look at where things live. It takes about thirty seconds."),
        TourStep::new("tour-demo-search", "Find anything", "Search across projects, notes and files. Press / from anywhere to jump here."),
        TourStep::new("tour-demo-new", "Start a project", "New projects begin from a template, so research plans come with interview guides ready to go.").side(Side::Start),
        TourStep::new("tour-demo-list", "Recent work", "Your last five projects, newest first. Drag one onto the sidebar to pin it.").side(Side::End),
        TourStep::new("#tour-demo-missing", "Shared with you", "When teammates share a project it shows up here. Nothing yet, so this step sits in the middle."),
    ];

    view! {
        <div style="display: grid; gap: var(--space-4);">
            <div style="display: flex; gap: var(--space-3); align-items: center;">
                <Button on_click=move |_| set_open.set(true)>"Take the tour"</Button>
                <span style="font: var(--text-xs); color: var(--text-3);">{outcome}</span>
            </div>
            <div style="display: grid; gap: var(--space-4); padding: var(--space-5); border-radius: var(--radius-lg); background: var(--surface-sunken);">
                <div style="display: flex; gap: var(--space-3);">
                    <div id="tour-demo-search" data-scope="input" data-part="field" style="flex: 1; min-width: 0;">
                        <input type="search" aria-label="Search projects" placeholder="Search projects" style="flex: 1; min-width: 0; border: none; background: transparent; font: var(--text-body); color: inherit; outline: none;" />
                    </div>
                    <span id="tour-demo-new"><Button variant=ButtonVariant::Accent>"New project"</Button></span>
                </div>
                <ul id="tour-demo-list" style="margin: 0; padding: 0; list-style: none; display: grid; gap: var(--space-2); max-width: 320px; font: var(--text-sm);">
                    <li>"Q3 onboarding interviews"</li>
                    <li>"Pricing page experiment"</li>
                    <li>"Churn survey, September"</li>
                </ul>
            </div>
            <Tour
                steps=steps
                open=open
                on_open_change=move |o| set_open.set(o)
                on_end=move |end| set_outcome.set(format!("Ended: {end:?}"))
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tour",
    name: "Tour",
    group: "Overlays",
    summary: "Onboarding coachmarks. Each step spotlights an element, scrolls it into view and explains it in a popover beside it. Missing targets show centred instead.",
    demo,
    leptos: include_str!("tour.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tour.rs"),
    parts: &[
        &["Tour", "spotlight", "Dims the page with a rounded, padded cut-out around the target. data-missing dims it all"],
        &["Tour", "popover", "role=\"dialog\" labelled by the title, anchored beside the target. data-centred without one"],
        &["Tour", "title / body", "The step's heading and explanation"],
        &["Tour", "progress", "\"Step 2 of 5\""],
        &["Tour", "skip / prev / next", "Buttons. Next reads \"Done\" on the last step"],
    ],
    keys: &[
        &["→", "Next step (or finish)"],
        &["←", "Previous step"],
        &["Escape", "Skip the rest. Focus returns to where it was"],
        &["Tab", "Move between the popover's buttons"],
    ],
    props: &[
        &["steps", "Vec<TourStep>", "Required. TourStep::new(target, title, body) or ::centred(title, body)"],
        &["TourStep.target", "String", "An element id, or any CSS selector"],
        &[".side", "Side", "Which side of the target the popover sits on. Default below"],
        &["open", "Option<bool>", "Controlled running state; true starts at step 1"],
        &["default_open", "bool", "Start on mount, when uncontrolled"],
        &["on_open_change", "Callback<bool>", "Started or ended"],
        &["on_step_change", "Callback<usize>", "A new step is showing"],
        &["on_end", "Callback<TourOutcome>", "Completed, or Skipped(at step)"],
    ],
};
