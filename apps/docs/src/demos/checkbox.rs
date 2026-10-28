use leptos::prelude::*;
use hydroxy_leptos::checkbox::*;

pub fn demo() -> AnyView {
    let (interviews, set_interviews) = signal(true);
    let (tickets, set_tickets) = signal(false);
    // The parent is indeterminate while its children disagree.
    let all = move || match (interviews.get(), tickets.get()) {
        (true, true) => CheckedState::Checked,
        (false, false) => CheckedState::Unchecked,
        _ => CheckedState::Indeterminate,
    };
    let set_all = move |on| {
        set_interviews.set(on);
        set_tickets.set(on);
    };

    view! {
        <div style="display: grid; gap: var(--space-4);">
            <Checkbox checked=Signal::derive(move || Some(all())) on_checked_change=set_all>
                "All sources"
            </Checkbox>
            <div style="display: grid; gap: var(--space-4); padding-left: var(--space-8);">
                <Checkbox
                    checked=Signal::derive(move || Some(interviews.get().into()))
                    on_checked_change=move |on| set_interviews.set(on)
                    name="sources"
                    value="interviews"
                >
                    "interviews-q3.pdf"
                </Checkbox>
                <Checkbox
                    checked=Signal::derive(move || Some(tickets.get().into()))
                    on_checked_change=move |on| set_tickets.set(on)
                    name="sources"
                    value="tickets"
                >
                    "tickets-sept.csv"
                </Checkbox>
            </div>
        </div>

        <div style="display: grid; gap: var(--space-4);">
            <Checkbox default_checked=CheckedState::Checked>"Email me when the summary is ready"</Checkbox>
            <Checkbox disabled=true>"Share with the workspace"</Checkbox>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "checkbox",
    name: "Checkbox",
    group: "Forms",
    summary: "Turn an independent option on or off. A parent can show a mixed state when its children disagree.",
    demo,
    leptos: include_str!("checkbox.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/checkbox.rs"),
    parts: &[
        &["Checkbox", "root", "A <label>, so clicking the text toggles. data-state, data-disabled"],
        &["Checkbox", "control", "<button role=checkbox>, aria-checked=true|false|mixed"],
        &["Checkbox", "indicator", "Check or dash glyph, aria-hidden"],
        &["Checkbox", "label", "The children"],
        &["Checkbox", "-", "<input type=hidden> when name is set and the box is checked"],
    ],
    keys: &[
        &["Space", "Toggles. A mixed box becomes checked"],
        &["Enter", "Does nothing, as on a native checkbox"],
        &["Tab", "Moves focus to and from the box"],
    ],
    props: &[
        &["checked", "Option<CheckedState>", "Controlled state. Pair with on_checked_change"],
        &["default_checked", "CheckedState", "Initial state when uncontrolled. Default Unchecked"],
        &["disabled", "bool", "Ignore clicks and keys"],
        &["name", "String", "Form field name. A checked box submits value"],
        &["value", "String", "Submitted value. Default on"],
        &["on_checked_change", "Callback<bool>", "Called with the proposed state. Never mixed"],
    ],
};
