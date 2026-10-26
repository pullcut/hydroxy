//! `<ApprovalCard>`: asks a person to approve or reject an action, then shows
//! who decided and when.

use leptos::prelude::*;

use crate::badge::{Badge, Tone};
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::card::Card;
use crate::focus::focus_id;

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
    #[prop(into)]
    title: String,
    /// Why it needs approval, or what it will do.
    #[prop(optional, into)]
    description: Option<String>,
    /// Key/value facts about the action, in order.
    #[prop(optional, into)]
    details: Vec<(String, String)>,
    /// Shown as a badge next to the title.
    #[prop(optional)]
    risk: Option<Risk>,
    /// Where the request stands. Default pending.
    #[prop(optional, into)]
    status: Signal<ApprovalStatus>,
    /// Ask for an optional reason before rejecting.
    #[prop(optional)]
    reject_reason: bool,
    /// Approve button text. Default "Approve".
    #[prop(default = "Approve".into(), into)]
    approve_label: String,
    /// Reject button text. Default "Reject".
    #[prop(default = "Reject".into(), into)]
    reject_label: String,
    /// Called when the person approves.
    #[prop(optional, into)]
    on_approve: Option<Callback<()>>,
    /// Called when the person rejects, with the reason if one was typed.
    #[prop(optional, into)]
    on_reject: Option<Callback<Option<String>>>,
) -> impl IntoView {
    let id = crate::id::next_id(SCOPE);
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let reason_id = format!("{id}-reason");
    let rejecting = RwSignal::new(false);
    let reason = RwSignal::new(String::new());
    let (approve_label, reject_label) = (StoredValue::new(approve_label), StoredValue::new(reject_label));

    let reject = move || {
        let text = reason.get_untracked().trim().to_string();
        rejecting.set(false);
        if let Some(cb) = on_reject {
            cb.run((!text.is_empty()).then_some(text));
        }
    };
    let reason_box = StoredValue::new(reason_id);

    let actions = move || {
        if rejecting.get() {
            let rid = reason_box.get_value();
            view! {
                <div data-scope=SCOPE data-part="reason">
                    <label for=rid.clone()>"Reason (optional)"</label>
                    <textarea
                        id=rid
                        rows="3"
                        prop:value=move || reason.get()
                        on:input=move |ev| reason.set(event_target_value(&ev))
                    ></textarea>
                    <div data-scope=SCOPE data-part="actions">
                        <Button variant=ButtonVariant::Danger size=ButtonSize::Sm on_click=move |_| reject()>
                            {reject_label.get_value()}
                        </Button>
                        <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on_click=move |_| rejecting.set(false)>
                            "Cancel"
                        </Button>
                    </div>
                </div>
            }
            .into_any()
        } else {
            view! {
                <div data-scope=SCOPE data-part="actions">
                    <Button
                        size=ButtonSize::Sm
                        on_click=move |_| {
                            if let Some(cb) = on_approve {
                                cb.run(());
                            }
                        }
                    >
                        {approve_label.get_value()}
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=move |_| {
                            if reject_reason {
                                rejecting.set(true);
                                let rid = reason_box.get_value();
                                set_timeout(move || focus_id(&rid), std::time::Duration::ZERO);
                            } else {
                                reject();
                            }
                        }
                    >
                        {reject_label.get_value()}
                    </Button>
                </div>
            }
            .into_any()
        }
    };

    let footer = move || match status.get() {
        ApprovalStatus::Pending => actions().into_any(),
        ApprovalStatus::Approved { by, at } => resolution(Tone::Success, "Approved", by, at, None).into_any(),
        ApprovalStatus::Rejected { by, at, reason } => resolution(Tone::Danger, "Rejected", by, at, reason).into_any(),
    };

    view! {
        <Card padding=20>
            <div
                data-scope=SCOPE
                data-part="root"
                data-state=move || status.with(ApprovalStatus::as_str)
                role="group"
                aria-labelledby=labelled_by
            >
                <div data-scope=SCOPE data-part="header">
                    <h3 data-scope=SCOPE data-part="title" id=title_id>{title}</h3>
                    {risk.map(|r| view! { <Badge tone=risk_tone(r) dot=true>{r.label()}</Badge> })}
                </div>
                {description.map(|d| view! { <p data-scope=SCOPE data-part="description">{d}</p> })}
                {(!details.is_empty()).then(|| view! {
                    <dl data-scope=SCOPE data-part="details">
                        {details.into_iter().map(|(k, v)| view! { <div><dt>{k}</dt><dd>{v}</dd></div> }).collect_view()}
                    </dl>
                })}
                {footer}
            </div>
        </Card>
    }
}

fn resolution(tone: Tone, label: &'static str, by: String, at: String, reason: Option<String>) -> impl IntoView {
    view! {
        <div data-scope=SCOPE data-part="resolution" role="status">
            <div data-scope=SCOPE data-part="decision">
                <Badge tone=tone>{label}</Badge>
                <span>"by " <b>{by}</b> " · " {at}</span>
            </div>
            {reason.map(|r| view! { <blockquote data-scope=SCOPE data-part="resolution-reason">{r}</blockquote> })}
        </div>
    }
}
