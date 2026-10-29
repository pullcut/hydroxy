use hydroxy_leptos::collapsible::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (open, set_open) = signal(Some(true));

    view! {
        // Controlled: the owner keeps the open state.
        <Collapsible open=open on_open_change=move |o| set_open.set(Some(o))>
            <CollapsibleTrigger>"Sources"</CollapsibleTrigger>
            <CollapsibleContent>
                <p>"interviews-q3.pdf"</p>
                <p>"tickets-sept.csv"</p>
            </CollapsibleContent>
        </Collapsible>

        // Uncontrolled, starts closed.
        <Collapsible>
            <CollapsibleTrigger>"Archived"</CollapsibleTrigger>
            <CollapsibleContent>"Nothing archived."</CollapsibleContent>
        </Collapsible>

        // Disabled: the trigger ignores clicks.
        <Collapsible disabled=true>
            <CollapsibleTrigger>"Shared with you"</CollapsibleTrigger>
            <CollapsibleContent>"No shared projects."</CollapsibleContent>
        </Collapsible>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "collapsible",
    name: "Collapsible",
    group: "Disclosure",
    summary: "Show and hide one section, such as a sidebar group. The content stays in the page while closed, just hidden.",
    demo,
    leptos: include_str!("collapsible.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/collapsible.rs"),
    parts: &[
        &["Collapsible", "root", "Provides the machine. data-state=open|closed"],
        &["CollapsibleTrigger", "trigger", "button, aria-expanded, aria-controls, disabled"],
        &["CollapsibleContent", "content", "role=region, aria-labelledby, hidden when closed"],
    ],
    keys: &[&["Enter / Space", "Opens or closes (native button)"]],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial state when uncontrolled. Default false"],
        &["disabled", "bool", "Ignore toggles"],
        &["on_open_change", "Callback<bool>", "Called with the new open state"],
    ],
    events: &[],
};
