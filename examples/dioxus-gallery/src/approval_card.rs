use dioxus::prelude::*;
use hydroxy_dioxus::approval_card::{ApprovalCard, ApprovalStatus, Risk};

#[component]
pub fn ApprovalCardDemo() -> Element {
    let mut status = use_signal(|| ApprovalStatus::Pending);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 480px;",
            // The owner holds the status and records the decision.
            ApprovalCard {
                title: "Send 240 renewal emails",
                description: "The agent drafted renewal reminders for every account due in November.",
                details: vec![
                    ("Recipients".to_string(), "240 accounts".to_string()),
                    ("Template".to_string(), "renewal-nov.md".to_string()),
                    ("Send at".to_string(), "Mon 2 Nov, 09:00".to_string()),
                ],
                risk: Risk::High,
                status: status(),
                reject_reason: true,
                approve_label: "Send emails",
                on_approve: move |()| status.set(ApprovalStatus::Approved { by: "You".into(), at: "just now".into() }),
                on_reject: move |reason| status.set(ApprovalStatus::Rejected { by: "You".into(), at: "just now".into(), reason }),
            }

            ApprovalCard {
                title: "Archive 12 stale threads",
                risk: Risk::Low,
                status: ApprovalStatus::Rejected {
                    by: "Jo Marsh".into(),
                    at: "26 Oct, 14:02".into(),
                    reason: Some("Two of these are still active.".into()),
                },
            }
        }
    }
}
