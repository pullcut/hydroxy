use hydroxy_leptos::approval_card::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let status = RwSignal::new(ApprovalStatus::Pending);

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 480px;">
            // The owner holds the status and records the decision.
            <ApprovalCard
                title="Send 240 renewal emails"
                description="The agent drafted renewal reminders for every account due in November."
                details=vec![
                    ("Recipients".to_string(), "240 accounts".to_string()),
                    ("Template".to_string(), "renewal-nov.md".to_string()),
                    ("Send at".to_string(), "Mon 2 Nov, 09:00".to_string()),
                ]
                risk=Risk::High
                status=status
                reject_reason=true
                approve_label="Send emails"
                on_approve=move |()| status.set(ApprovalStatus::Approved { by: "You".into(), at: "just now".into() })
                on_reject=move |reason| status.set(ApprovalStatus::Rejected { by: "You".into(), at: "just now".into(), reason })
            />

            <ApprovalCard
                title="Archive 12 stale threads"
                risk=Risk::Low
                status=ApprovalStatus::Rejected {
                    by: "Jo Marsh".into(),
                    at: "26 Oct, 14:02".into(),
                    reason: Some("Two of these are still active.".into()),
                }
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "approval-card",
    name: "Approval card",
    group: "Feedback",
    summary: "Asks a person to approve or reject an action, with the facts they need to decide. Once decided, it shows who decided and when.",
    demo,
    leptos: include_str!("approval_card.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/approval_card.rs"),
    parts: &[
        &["ApprovalCard", "root", "role=group named by the title, inside a Card. data-state=pending|approved|rejected"],
        &["ApprovalCard", "header, title", "<h3> and the risk Badge"],
        &["ApprovalCard", "description", "Why it needs approval"],
        &["ApprovalCard", "details", "<dl> of key/value facts"],
        &["ApprovalCard", "actions", "Approve and Reject Buttons"],
        &["ApprovalCard", "reason", "Labelled textarea shown before rejecting, when reject_reason is set"],
        &[
            "ApprovalCard",
            "resolution, decision, resolution-reason",
            "role=status: outcome Badge, who and when, and the reason",
        ],
    ],
    keys: &[
        &["Tab", "Moves between the buttons and the reason field"],
        &["Enter / Space", "Activates the focused button"],
    ],
    props: &[
        &["title", "String", "Required. The action, such as \"Send 240 renewal emails\""],
        &["description", "Option<String>", "Why it needs approval"],
        &["details", "Vec<(String, String)>", "Key/value facts, in order"],
        &["risk", "Option<Risk>", "Low, Medium or High. Shown as a badge"],
        &["status", "ApprovalStatus", "Pending (default), Approved { by, at } or Rejected { by, at, reason }"],
        &["reject_reason", "bool", "Ask for an optional reason before rejecting"],
        &["approve_label / reject_label", "String", "Button text. Default \"Approve\" and \"Reject\""],
        &["on_approve", "Callback<()>", "Called on approve"],
        &["on_reject", "Callback<Option<String>>", "Called on reject, with the reason if one was typed"],
    ],
    events: &[],
};
