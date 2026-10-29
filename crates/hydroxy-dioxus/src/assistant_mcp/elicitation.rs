//! The elicitation card.

use dioxus::prelude::*;
use hydroxy_headless::assistant_mcp::{
    self as h, ElicitField, ElicitKind, ElicitStatus, McpElicitationCommand, McpElicitationEvent, McpElicitationInput,
    McpElicitationMachine,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonType, ButtonVariant};
use crate::drive::drive;

/// A card where a server asks the person for input. Submit sends the answers through
/// `on_submit` once every required field has one; Decline calls `on_cancel`. After either, the
/// form locks and a status line says what happened.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    reason = "one rsx! tree; props are owned by the component macro"
)]
pub fn AssistantMcpElicitation(
    /// Heading, such as "Linear needs a few details".
    title: String,
    /// Longer explanation of why the server is asking.
    message: Option<String>,
    /// The questions.
    fields: Vec<ElicitField>,
    /// Called with the answers as (field name, value) pairs.
    on_submit: Option<EventHandler<Vec<(String, String)>>>,
    /// Called when the person declines.
    on_cancel: Option<EventHandler<()>>,
) -> Element {
    let input = use_signal(|| McpElicitationInput { fields: fields.clone() });
    let state = use_signal(|| McpElicitationMachine::initial(&input.peek()));
    // Per-instance, so two cards asking for the same field don't share error ids.
    let uid = use_hook(|| hydroxy_headless::next_id(h::SCOPE_ELICITATION));
    let send = move |event: McpElicitationEvent| {
        let commands = drive::<McpElicitationMachine>(state, input, event);
        for command in commands {
            match command {
                McpElicitationCommand::Submit(answers) => {
                    if let Some(handler) = on_submit {
                        handler.call(answers);
                    }
                }
                McpElicitationCommand::Cancel => {
                    if let Some(handler) = on_cancel {
                        handler.call(());
                    }
                }
            }
        }
    };

    let s = state.read();
    let locked = s.status != ElicitStatus::Pending;
    rsx! {
        form {
            "data-scope": h::SCOPE_ELICITATION,
            "data-part": "root",
            "data-status": s.status.as_str(),
            novalidate: true,
            onsubmit: move |e| {
                e.prevent_default();
                send(McpElicitationEvent::Submit);
            },
            h3 { "data-scope": h::SCOPE_ELICITATION, "data-part": "title", "{title}" }
            if let Some(m) = message {
                p { "data-scope": h::SCOPE_ELICITATION, "data-part": "message", "{m}" }
            }
            div { "data-scope": h::SCOPE_ELICITATION, "data-part": "form",
                for f in fields.iter() {
                    {
                        let name = f.name.clone();
                        let missing = s.missing.contains(&f.name);
                        rsx! {
                            div {
                                key: "{f.name}",
                                "data-scope": h::SCOPE_ELICITATION,
                                "data-part": "field",
                                "data-invalid": missing.then_some(""),
                                span { "data-scope": h::SCOPE_ELICITATION, "data-part": "label",
                                    "{f.label}"
                                    if f.required {
                                        span { "data-part": "required", "aria-hidden": "true", " *" }
                                    }
                                }
                                match &f.kind {
                                    ElicitKind::Text => rsx! {
                                        input {
                                            "data-scope": h::SCOPE_ELICITATION,
                                            "data-part": "input",
                                            r#type: "text",
                                            name: "{f.name}",
                                            autocomplete: "off",
                                            disabled: locked,
                                            "aria-invalid": missing.then_some("true"),
                                            "aria-describedby": "{uid}-{f.name}-error",
                                            value: "{s.value(&f.name)}",
                                            oninput: move |e| send(McpElicitationEvent::Set { name: name.clone(), value: e.value() }),
                                        }
                                    },
                                    ElicitKind::Bool => rsx! {
                                        input {
                                            "data-scope": h::SCOPE_ELICITATION,
                                            "data-part": "checkbox",
                                            r#type: "checkbox",
                                            name: "{f.name}",
                                            disabled: locked,
                                            checked: s.value(&f.name) == "true",
                                            onchange: move |e| send(McpElicitationEvent::Set { name: name.clone(), value: e.checked().to_string() }),
                                        }
                                    },
                                    ElicitKind::Choice(options) => rsx! {
                                        span {
                                            "data-scope": h::SCOPE_ELICITATION,
                                            "data-part": "options",
                                            role: "radiogroup",
                                            "aria-label": "{f.label}",
                                            for o in options.iter() {
                                                {
                                                    let (name, value) = (name.clone(), o.clone());
                                                    rsx! {
                                                        label { key: "{o}", "data-scope": h::SCOPE_ELICITATION, "data-part": "option",
                                                            input {
                                                                r#type: "radio",
                                                                name: "{f.name}",
                                                                value: "{o}",
                                                                disabled: locked,
                                                                checked: s.value(&f.name) == o.as_str(),
                                                                onchange: move |_| send(McpElicitationEvent::Set { name: name.clone(), value: value.clone() }),
                                                            }
                                                            "{o}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                }
                                span {
                                    "data-scope": h::SCOPE_ELICITATION,
                                    "data-part": "field-error",
                                    id: "{uid}-{f.name}-error",
                                    role: "alert",
                                    if missing {
                                        "This answer is required."
                                    }
                                }
                            }
                        }
                    }
                }
            }
            match s.status {
                ElicitStatus::Pending => rsx! {
                    div { "data-scope": h::SCOPE_ELICITATION, "data-part": "actions",
                        span { "data-scope": h::SCOPE_ELICITATION, "data-part": "cancel",
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                on_click: move |_| send(McpElicitationEvent::Cancel),
                                "Decline"
                            }
                        }
                        span { "data-scope": h::SCOPE_ELICITATION, "data-part": "submit",
                            Button { button_type: ButtonType::Submit, size: ButtonSize::Sm, "Submit" }
                        }
                    }
                },
                ElicitStatus::Submitted => rsx! {
                    p { "data-scope": h::SCOPE_ELICITATION, "data-part": "result", role: "status", "Answers sent." }
                },
                ElicitStatus::Cancelled => rsx! {
                    p { "data-scope": h::SCOPE_ELICITATION, "data-part": "result", role: "status", "You declined." }
                },
            }
        }
    }
}
