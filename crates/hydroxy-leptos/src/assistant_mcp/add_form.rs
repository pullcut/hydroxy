//! The add-server form.

use hydroxy_headless::assistant_mcp::{self as h, McpAddFormCommand, McpAddFormEvent, McpAddFormMachine, McpField};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use super::FormStatus;
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::drive::drive;

/// The add-server form. `on_submit` runs with the trimmed name and URL once both pass
/// validation. Answer it by setting `result` to `Some(Ok(()))` or `Some(Err(message))`; the
/// message shows as an alert. Clear `result` in `on_submit` so a repeated answer is noticed.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    reason = "one view! tree; props are owned by the component macro"
)]
pub fn AssistantMcpAddForm(
    /// Called with (name, url) when the form is valid and submitted.
    #[prop(optional, into)]
    on_submit: Option<Callback<(String, String)>>,
    /// The owner's answer to the last submit.
    #[prop(optional, into)]
    result: MaybeProp<Result<(), String>>,
    /// Called when Cancel is pressed. Cancel is hidden without it.
    #[prop(optional, into)]
    on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let state = RwSignal::new(McpAddFormMachine::initial(&()));
    let input = Signal::stored(());
    let send = move |event: McpAddFormEvent| {
        for command in drive::<McpAddFormMachine, _>(state, input, event) {
            let McpAddFormCommand::Submit { name, url } = command;
            if let Some(cb) = on_submit {
                cb.run((name, url));
            }
        }
    };
    Effect::new(move |_| {
        if let Some(r) = result.get() {
            send(match r {
                Ok(()) => McpAddFormEvent::Succeeded,
                Err(msg) => McpAddFormEvent::Failed(msg),
            });
        }
    });

    let submitting = move || state.with(|s| s.status == FormStatus::Submitting);
    let field = move |f: McpField, label: &'static str, placeholder: &'static str, input_type: &'static str| {
        let error = move || {
            state.with(|s| match f {
                McpField::Name => s.name_error.clone(),
                McpField::Url => s.url_error.clone(),
            })
        };
        let error_id = format!("mcp-add-{}-error", if f == McpField::Name { "name" } else { "url" });
        view! {
            <label data-scope=h::SCOPE_ADD_FORM data-part="field">
                <span data-scope=h::SCOPE_ADD_FORM data-part="label">{label}</span>
                <input
                    data-scope=h::SCOPE_ADD_FORM
                    data-part="input"
                    type=input_type
                    placeholder=placeholder
                    autocomplete="off"
                    aria-invalid=move || error().is_some().then_some("true")
                    aria-describedby=error_id.clone()
                    prop:value=move || state.with(|s| if f == McpField::Name { s.name.clone() } else { s.url.clone() })
                    on:input=move |ev| send(McpAddFormEvent::Set(f, event_target_value(&ev)))
                />
                <span data-scope=h::SCOPE_ADD_FORM data-part="field-error" id=error_id>{error}</span>
            </label>
        }
    };
    view! {
        <form
            data-scope=h::SCOPE_ADD_FORM
            data-part="root"
            novalidate
            data-status=move || state.with(|s| match s.status {
                FormStatus::Idle => "idle",
                FormStatus::Submitting => "submitting",
                FormStatus::Submitted => "submitted",
                FormStatus::Error => "error",
            })
            on:submit=move |ev| {
                ev.prevent_default();
                send(McpAddFormEvent::Submit);
            }
        >
            {field(McpField::Name, "Name", "Release notes", "text")}
            {field(McpField::Url, "Server URL", "https://example.com/mcp", "url")}
            {move || {
                state
                    .with(|s| s.error.clone())
                    .map(|e| view! { <p data-scope=h::SCOPE_ADD_FORM data-part="error" role="alert">{e}</p> })
            }}
            <div data-scope=h::SCOPE_ADD_FORM data-part="actions">
                {on_cancel
                    .map(|cb| {
                        view! {
                            <Button
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Sm
                                on_click=move |_| cb.run(())
                            >
                                "Cancel"
                            </Button>
                        }
                    })}
                <span data-scope=h::SCOPE_ADD_FORM data-part="submit">
                    <Button
                        button_type=crate::button::ButtonType::Submit
                        size=ButtonSize::Sm
                        disabled=Signal::derive(submitting)
                    >
                        {move || if submitting() { "Adding" } else { "Add server" }}
                    </Button>
                </span>
            </div>
        </form>
    }
}
