//! `ApprovalCard`: asks a person to approve or reject an action, then shows
//! who decided and when.

use dioxus::prelude::*;

use crate::badge::{Badge, Tone};
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::card::Card;

pub use oxide_headless::approval::{ApprovalStatus, Risk};

const SCOPE: &str = "approval-card";

fn risk_tone(risk: Risk) -> Tone {
    match risk {
        Risk::Low => Tone::Neutral,
        Risk::Medium => Tone::Warning,
        Risk::High => Tone::Danger,
    }
}

/// An approval request. The owner holds `status` and updates it from the callbacks.
#[component]
pub fn ApprovalCard(
    /// The action, such as "Send 240 renewal emails".
    title: String,
    /// Why it needs approval, or what it will do.
    description: Option<String>,
    /// Key/value facts about the action, in order.
    #[props(default)]
    details: Vec<(String, String)>,
    /// Shown as a badge next to the title.
    risk: Option<Risk>,
    /// Where the request stands. Default pending.
    #[props(default)]
    status: ApprovalStatus,
    /// Ask for an optional reason before rejecting.
    #[props(default)]
    reject_reason: bool,
    /// Approve button text. Default "Approve".
    #[props(default = "Approve".to_string())]
    approve_label: String,
    /// Reject button text. Default "Reject".
    #[props(default = "Reject".to_string())]
    reject_label: String,
    /// Called when the person approves.
    on_approve: Option<EventHandler<()>>,
    /// Called when the person rejects, with the reason if one was typed.
    on_reject: Option<EventHandler<Option<String>>>,
) -> Element {
    let id = use_hook(|| oxide_headless::next_id(SCOPE));
    let title_id = format!("{id}-title");
    let reason_id = format!("{id}-reason");
    let mut rejecting = use_signal(|| false);
    let mut reason = use_signal(String::new);
    let mut reject = move || {
        let text = reason.peek().trim().to_string();
        rejecting.set(false);
        if let Some(handler) = on_reject {
            handler.call((!text.is_empty()).then_some(text));
        }
    };

    let footer = match &status {
        ApprovalStatus::Pending if rejecting() => rsx! {
            div { "data-scope": SCOPE, "data-part": "reason",
                label { r#for: reason_id.clone(), "Reason (optional)" }
                textarea {
                    id: reason_id,
                    rows: "3",
                    value: "{reason}",
                    onmounted: move |e| async move {
                        let _ = e.data().set_focus(true).await;
                    },
                    oninput: move |e| reason.set(e.value()),
                }
                div { "data-scope": SCOPE, "data-part": "actions",
                    Button { variant: ButtonVariant::Danger, size: ButtonSize::Sm, on_click: move |_| reject(), "{reject_label}" }
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, on_click: move |_| rejecting.set(false), "Cancel" }
                }
            }
        },
        ApprovalStatus::Pending => rsx! {
            div { "data-scope": SCOPE, "data-part": "actions",
                Button {
                    size: ButtonSize::Sm,
                    on_click: move |_| {
                        if let Some(handler) = on_approve {
                            handler.call(());
                        }
                    },
                    "{approve_label}"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| if reject_reason { rejecting.set(true) } else { reject() },
                    "{reject_label}"
                }
            }
        },
        ApprovalStatus::Approved { by, at } => resolution(Tone::Success, "Approved", by, at, None),
        ApprovalStatus::Rejected { by, at, reason } => resolution(Tone::Danger, "Rejected", by, at, reason.as_deref()),
    };

    rsx! {
        Card { padding: 20,
            div {
                "data-scope": SCOPE,
                "data-part": "root",
                "data-state": status.as_str(),
                role: "group",
                "aria-labelledby": title_id.clone(),
                div { "data-scope": SCOPE, "data-part": "header",
                    h3 { "data-scope": SCOPE, "data-part": "title", id: title_id, "{title}" }
                    if let Some(r) = risk {
                        Badge { tone: risk_tone(r), dot: true, "{r.label()}" }
                    }
                }
                if let Some(d) = description {
                    p { "data-scope": SCOPE, "data-part": "description", "{d}" }
                }
                if !details.is_empty() {
                    dl { "data-scope": SCOPE, "data-part": "details",
                        for (k, v) in details.iter() {
                            div { key: "{k}",
                                dt { "{k}" }
                                dd { "{v}" }
                            }
                        }
                    }
                }
                {footer}
            }
        }
    }
}

fn resolution(tone: Tone, label: &'static str, by: &str, at: &str, reason: Option<&str>) -> Element {
    rsx! {
        div { "data-scope": SCOPE, "data-part": "resolution", role: "status",
            div { "data-scope": SCOPE, "data-part": "decision",
                Badge { tone, "{label}" }
                span { "by " b { "{by}" } " · {at}" }
            }
            if let Some(r) = reason {
                blockquote { "data-scope": SCOPE, "data-part": "resolution-reason", "{r}" }
            }
        }
    }
}
