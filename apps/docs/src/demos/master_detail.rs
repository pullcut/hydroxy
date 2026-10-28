use leptos::prelude::*;
use hydroxy_leptos::master_detail::{MasterDetail, MasterDetailRow};

const MESSAGES: [(&str, &str, &str); 5] = [
    ("m1", "Launch checklist", "Everything on the list is done except the changelog. I will have it by Thursday."),
    ("m2", "Invoice 4021", "The invoice was sent on the 3rd and is due in two weeks. No reminders have gone out yet."),
    ("m3", "Standup notes", "Ana is blocked on the audio bug. Ben is picking up the transcript work next."),
    ("m4", "Design review", "The new player looks right. Spacing on the rate button needs another pass."),
    ("m5", "Welcome aboard", "Glad to have you. Your laptop ships Monday and your first week is planned."),
];

pub fn demo() -> AnyView {
    // The host owns the selection. Resize below 640px wide to see one pane at a time.
    let (selected, set_selected) = signal(Some("m1".to_string()));

    view! {
        <div style="max-width: 720px; height: 320px; display: grid;">
            <MasterDetail
                selected=selected
                on_select=move |id| set_selected.set(id)
                list_heading=move || "Inbox"
                list=move || {
                    MESSAGES
                        .iter()
                        .map(|(id, title, _)| {
                            view! { <MasterDetailRow id=*id><strong>{*title}</strong></MasterDetailRow> }
                        })
                        .collect_view()
                }
                detail=move || {
                    view! {
                        {move || {
                            let current = selected.get();
                            MESSAGES
                                .iter()
                                .find(|(id, _, _)| Some(*id) == current.as_deref())
                                .map_or_else(
                                    || view! { <p style="margin: 0; color: var(--text-2);">"Choose a message."</p> }.into_any(),
                                    |(_, title, body)| {
                                    view! {
                                        <h3 style="margin: 0 0 var(--space-3); font: var(--text-h3);">{*title}</h3>
                                        <p style="margin: 0; color: var(--text-2);">{*body}</p>
                                    }
                                        .into_any()
                                    },
                                )
                        }}
                    }
                }
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "master-detail",
    name: "Master detail",
    group: "Layout",
    summary: "A list beside its detail. Wide screens show both; narrow ones show one pane at a time with a Back button that returns focus to the row you left. The host owns the selection, so routing stays yours.",
    demo,
    leptos: include_str!("master_detail.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/master_detail.rs"),
    parts: &[
        &["MasterDetail", "root", "data-mobile-pane is list or detail"],
        &["(inside)", "list", "The list pane"],
        &["(inside)", "list-heading", "Optional h2. Takes focus when the row you left no longer exists"],
        &["MasterDetailRow", "row", "A button per item. data-selected and aria-current mark the chosen one"],
        &["(inside)", "detail", "The detail pane. Stays mounted"],
        &["(inside)", "back", "Shown on narrow screens only"],
    ],
    keys: &[
        &["Enter / Space", "On a row: choose it"],
        &["Tab", "Moves through rows. After Back, focus returns to the row you left"],
    ],
    props: &[
        &["selected", "Option<String>", "The host's selected row id"],
        &["list / detail", "ViewFn", "The two panes. Put MasterDetailRow elements in the list"],
        &["list_heading", "Option<ViewFn>", "A heading for the list"],
        &["mobile_pane", "PaneMode", "Auto follows the selection; List or Detail is host-controlled"],
        &["on_select", "Callback<Option<String>>", "A row was chosen (Some) or Back was pressed (None)"],
        &["on_pane_change", "Callback<Pane>", "The host should show this pane (explicit modes only)"],
        &["on_back", "Callback<String>", "Back was pressed on this row. Route here"],
    ],
};
