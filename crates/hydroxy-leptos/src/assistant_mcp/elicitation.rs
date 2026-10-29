//! The elicitation card.

use hydroxy_headless::assistant_mcp::{
    self as h, McpElicitationCommand, McpElicitationEvent, McpElicitationInput, McpElicitationMachine,
};
use hydroxy_statechart::Machine;
use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;

use super::{ElicitField, ElicitKind, ElicitStatus};
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::drive::drive;

/// A card where a server asks the person for input. Submit sends the answers through
/// `on_submit` once every required field has one; Decline calls `on_cancel`. After either, the
/// form locks and a status line says what happened.
#[component]
#[allow(clippy::implicit_hasher, reason = "props take a std HashMap; generalising the hasher only adds plumbing")]
pub fn AssistantMcpElicitation(
    /// Heading, such as "Linear needs a few details".
    #[prop(into)]
    title: String,
    /// Longer explanation of why the server is asking.
    #[prop(optional, into)]
    message: Option<String>,
    /// The questions.
    fields: Vec<ElicitField>,
    /// Called with the answers as (field name, value) pairs.
    #[prop(optional, into)]
    on_submit: Option<Callback<Vec<(String, String)>>>,
    /// Called when the person declines.
    #[prop(optional, into)]
    on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let input = Signal::stored(McpElicitationInput { fields: fields.clone() });
    let state = RwSignal::new(input.with_untracked(McpElicitationMachine::initial));
    let send = move |event: McpElicitationEvent| {
        for command in drive::<McpElicitationMachine, _>(state, input, event) {
            match command {
                McpElicitationCommand::Submit(answers) => {
                    if let Some(cb) = on_submit {
                        cb.run(answers);
                    }
                }
                McpElicitationCommand::Cancel => {
                    if let Some(cb) = on_cancel {
                        cb.run(());
                    }
                }
            }
        }
    };
    let locked = move || state.with(|s| s.status != ElicitStatus::Pending);
    // Per-instance, so two cards asking for the same field don't share error ids.
    let uid = crate::id::next_id(h::SCOPE_ELICITATION);
    let set = move |name: &str, value: String| send(McpElicitationEvent::Set { name: name.to_string(), value });

    let rows = fields
        .into_iter()
        .map(|f| {
            let ElicitField { name, label, kind, required } = f;
            let missing = {
                let name = name.clone();
                Memo::new(move |_| state.with(|s| s.missing.contains(&name)))
            };
            let err_id = h::field_error_id(&uid, &name);
            let error = {
                let err_id = err_id.clone();
                view! {
                    <span data-scope=h::SCOPE_ELICITATION data-part="field-error" id=err_id role="alert">
                        {move || missing.get().then_some(h::REQUIRED_TEXT)}
                    </span>
                }
            };
            let control = match kind {
                ElicitKind::Text => {
                    let (n, v) = (name.clone(), name.clone());
                    EitherOf3::A(view! {
                        <input
                            data-scope=h::SCOPE_ELICITATION
                            data-part="input"
                            type="text"
                            name=name.clone()
                            autocomplete="off"
                            required=required
                            disabled=locked
                            aria-invalid=move || missing.get().then_some("true")
                            aria-describedby=err_id
                            prop:value=move || state.with(|s| s.value(&v).to_string())
                            on:input=move |ev| set(&n, event_target_value(&ev))
                        />
                    })
                }
                ElicitKind::Bool => {
                    let (n, v) = (name.clone(), name.clone());
                    EitherOf3::B(view! {
                        <input
                            data-scope=h::SCOPE_ELICITATION
                            data-part="checkbox"
                            type="checkbox"
                            name=name.clone()
                            disabled=locked
                            prop:checked=move || state.with(|s| s.value(&v) == "true")
                            on:change=move |ev| set(&n, event_target_checked(&ev).to_string())
                        />
                    })
                }
                ElicitKind::Choice(options) => {
                    let group = name.clone();
                    EitherOf3::C(view! {
                        <span data-scope=h::SCOPE_ELICITATION data-part="options" role="radiogroup" aria-label=label.clone()>
                            {options
                                .into_iter()
                                .map(|o| {
                                    let (n, v, opt, shown) = (name.clone(), name.clone(), o.clone(), o.clone());
                                    view! {
                                        <label data-scope=h::SCOPE_ELICITATION data-part="option">
                                            <input
                                                type="radio"
                                                name=group.clone()
                                                value=o
                                                disabled=locked
                                                prop:checked={
                                                    let opt = opt.clone();
                                                    move || state.with(|s| s.value(&v) == opt)
                                                }
                                                on:change=move |_| set(&n, opt.clone())
                                            />
                                            {shown}
                                        </label>
                                    }
                                })
                                .collect_view()}
                        </span>
                    })
                }
            };
            view! {
                <div data-scope=h::SCOPE_ELICITATION data-part="field" data-invalid=move || missing.get().then_some("")>
                    <span data-scope=h::SCOPE_ELICITATION data-part="label">
                        {label}
                        {required.then(|| view! { <span data-part="required" aria-hidden="true">" *"</span> })}
                    </span>
                    {control}
                    {error}
                </div>
            }
        })
        .collect_view();

    view! {
        <form
            data-scope=h::SCOPE_ELICITATION
            data-part="root"
            novalidate
            data-status=move || state.with(|s| s.status.as_str())
            on:submit=move |ev| {
                ev.prevent_default();
                send(McpElicitationEvent::Submit);
            }
        >
            <h3 data-scope=h::SCOPE_ELICITATION data-part="title">{title}</h3>
            {message.map(|m| view! { <p data-scope=h::SCOPE_ELICITATION data-part="message">{m}</p> })}
            <div data-scope=h::SCOPE_ELICITATION data-part="form">{rows}</div>
            {move || {
                let status = state.with(|s| s.status);
                match status {
                    ElicitStatus::Pending => {
                        Either::Left(view! {
                            <div data-scope=h::SCOPE_ELICITATION data-part="actions">
                                <span data-scope=h::SCOPE_ELICITATION data-part="cancel">
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Sm
                                        on_click=move |_| send(McpElicitationEvent::Cancel)
                                    >
                                        "Decline"
                                    </Button>
                                </span>
                                <span data-scope=h::SCOPE_ELICITATION data-part="submit">
                                    <Button button_type=crate::button::ButtonType::Submit size=ButtonSize::Sm>
                                        "Submit"
                                    </Button>
                                </span>
                            </div>
                        })
                    }
                    ElicitStatus::Submitted | ElicitStatus::Cancelled => {
                        let text = if status == ElicitStatus::Submitted { "Answers sent." } else { "You declined." };
                        Either::Right(view! { <p data-scope=h::SCOPE_ELICITATION data-part="result" role="status">{text}</p> })
                    }
                }
            }}
        </form>
    }
}
