//! The add-server form.

use dioxus::prelude::*;
use hydroxy_headless::assistant_mcp::{
    self as h, FormStatus, McpAddFormCommand, McpAddFormEvent, McpAddFormMachine, McpField,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonType, ButtonVariant};
use crate::drive::drive;

/// The add-server form. `on_submit` runs with the trimmed name and URL once both pass
/// validation. Answer it by setting `result` to `Some(Ok(()))` or `Some(Err(message))`; the
/// message shows as an alert. Clear `result` in `on_submit` so a repeated answer is noticed.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    reason = "one rsx! tree; props are owned by the component macro"
)]
pub fn AssistantMcpAddForm(
    /// Called with (name, url) when the form is valid and submitted.
    on_submit: Option<EventHandler<(String, String)>>,
    /// The owner's answer to the last submit.
    #[props(default)]
    result: ReadSignal<Option<Result<(), String>>>,
    /// Called when Cancel is pressed. Cancel is hidden without it.
    on_cancel: Option<EventHandler<()>>,
) -> Element {
    let state = use_signal(|| McpAddFormMachine::initial(&()));
    let unit = use_signal(|| ());
    let send = move |event: McpAddFormEvent| {
        let commands = drive::<McpAddFormMachine>(state, unit, event);
        for command in commands {
            let McpAddFormCommand::Submit { name, url } = command;
            if let Some(handler) = on_submit {
                handler.call((name, url));
            }
        }
    };
    use_effect(move || {
        if let Some(r) = result() {
            send(match r {
                Ok(()) => McpAddFormEvent::Succeeded,
                Err(msg) => McpAddFormEvent::Failed(msg),
            });
        }
    });

    let s = state.read();
    let submitting = s.status == FormStatus::Submitting;
    let status = match s.status {
        FormStatus::Idle => "idle",
        FormStatus::Submitting => "submitting",
        FormStatus::Submitted => "submitted",
        FormStatus::Error => "error",
    };
    let fields = [
        (McpField::Name, "name", "Name", "Release notes", "text", s.name.clone(), s.name_error.clone()),
        (McpField::Url, "url", "Server URL", "https://example.com/mcp", "url", s.url.clone(), s.url_error.clone()),
    ];
    rsx! {
        form {
            "data-scope": h::SCOPE_ADD_FORM,
            "data-part": "root",
            "data-status": status,
            novalidate: true,
            onsubmit: move |e| {
                e.prevent_default();
                send(McpAddFormEvent::Submit);
            },
            for (field , key , label , placeholder , input_type , value , error) in fields {
                label { key: "{key}", "data-scope": h::SCOPE_ADD_FORM, "data-part": "field",
                    span { "data-scope": h::SCOPE_ADD_FORM, "data-part": "label", "{label}" }
                    input {
                        "data-scope": h::SCOPE_ADD_FORM,
                        "data-part": "input",
                        r#type: input_type,
                        placeholder,
                        autocomplete: "off",
                        "aria-invalid": error.is_some().then_some("true"),
                        "aria-describedby": "mcp-add-{key}-error",
                        value: "{value}",
                        oninput: move |e| send(McpAddFormEvent::Set(field, e.value())),
                    }
                    span {
                        "data-scope": h::SCOPE_ADD_FORM,
                        "data-part": "field-error",
                        id: "mcp-add-{key}-error",
                        {error.clone()}
                    }
                }
            }
            if let Some(e) = s.error.clone() {
                p { "data-scope": h::SCOPE_ADD_FORM, "data-part": "error", role: "alert", "{e}" }
            }
            div { "data-scope": h::SCOPE_ADD_FORM, "data-part": "actions",
                if let Some(cancel) = on_cancel {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        on_click: move |_| cancel.call(()),
                        "Cancel"
                    }
                }
                span { "data-scope": h::SCOPE_ADD_FORM, "data-part": "submit",
                    Button {
                        button_type: ButtonType::Submit,
                        size: ButtonSize::Sm,
                        disabled: submitting,
                        if submitting {
                            "Adding"
                        } else {
                            "Add server"
                        }
                    }
                }
            }
        }
    }
}
