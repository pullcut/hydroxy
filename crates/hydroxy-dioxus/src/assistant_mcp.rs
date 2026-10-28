//! Assistant MCP: a manager of tool servers, one server row, the add-server form, and an
//! elicitation card where a server asks the person for input.
//!
//! The owner does the connecting. A row asks through `on_connect` and the owner reports back
//! through `outcome`; the row's machine decides which transitions are allowed.

use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_headless::assistant_mcp::{
    self as h, McpAddFormCommand, McpAddFormEvent, McpAddFormMachine, McpElicitationCommand, McpElicitationEvent,
    McpElicitationInput, McpElicitationMachine, McpField, McpServerCommand, McpServerEvent, McpServerMachine,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonType, ButtonVariant};

pub use hydroxy_headless::assistant_mcp::{
    Connection, ElicitField, ElicitKind, ElicitStatus, FormStatus, McpServerInfo,
};

/// One server row. The owner holds the connection: press Connect and `on_connect` runs with the
/// server id; when the connection settles, set `outcome` to `Some(Ok(tools))` or `Some(Err(message))`.
/// Clear `outcome` in `on_connect` so a repeated result is noticed.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    reason = "one rsx! tree; props are owned by the component macro"
)]
pub fn AssistantMcpServer(
    /// The server to show.
    server: McpServerInfo,
    /// What the owner's last connection attempt produced: the tool names, or an error message.
    #[props(default)]
    outcome: ReadSignal<Option<Result<Vec<String>, String>>>,
    /// Called with the server id when Connect or Retry is pressed.
    on_connect: Option<EventHandler<String>>,
    /// Called with the server id when Disconnect (or Cancel while connecting) is pressed.
    on_disconnect: Option<EventHandler<String>>,
    /// Called with the server id when Remove is pressed. Only custom servers show Remove.
    on_remove: Option<EventHandler<String>>,
) -> Element {
    let mut state = use_signal(|| McpServerMachine::initial(&()));
    let id = use_signal(|| server.id.clone());
    let mut send = move |event: McpServerEvent| {
        let step = McpServerMachine::transition(&state.peek(), event, &());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let handler = match command {
                McpServerCommand::Connect => on_connect,
                McpServerCommand::Disconnect => on_disconnect,
            };
            if let Some(handler) = handler {
                handler.call(id.peek().clone());
            }
        }
    };
    use_effect(move || {
        if let Some(result) = outcome() {
            send(match result {
                Ok(tools) => McpServerEvent::Connected(tools),
                Err(msg) => McpServerEvent::Failed(msg),
            });
        }
    });

    let s = state.read();
    let connection = s.connection.clone();
    let removable = server.custom && on_remove.is_some();
    let remove_label = format!("Remove {}", server.name);
    rsx! {
        li {
            "data-scope": h::SCOPE_SERVER,
            "data-part": "root",
            "data-connection-state": connection.as_str(),
            span { "data-scope": h::SCOPE_SERVER, "data-part": "icon", "aria-hidden": "true", "{server.initial()}" }
            div { "data-scope": h::SCOPE_SERVER, "data-part": "body",
                span { "data-scope": h::SCOPE_SERVER, "data-part": "name", "{server.name}" }
                span { "data-scope": h::SCOPE_SERVER, "data-part": "description", "{server.description}" }
                span { "data-scope": h::SCOPE_SERVER, "data-part": "status", role: "status",
                    "{h::status_label(&s.connection, s.tools.len())}"
                }
                if connection == Connection::Connected && !s.tools.is_empty() {
                    ul { "data-scope": h::SCOPE_SERVER, "data-part": "tools", "aria-label": "Tools",
                        for t in s.tools.iter() {
                            li { key: "{t}", "data-scope": h::SCOPE_SERVER, "data-part": "tool", "{t}" }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE_SERVER, "data-part": "actions",
                if h::can_connect(&connection) {
                    span { "data-scope": h::SCOPE_SERVER, "data-part": "connect-button",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            on_click: move |_| send(McpServerEvent::Connect),
                            if matches!(connection, Connection::Error(_)) {
                                "Retry"
                            } else {
                                "Connect"
                            }
                        }
                    }
                } else if h::can_disconnect(&connection) {
                    span { "data-scope": h::SCOPE_SERVER, "data-part": "disconnect-button",
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            on_click: move |_| send(McpServerEvent::Disconnect),
                            if connection == Connection::Connecting {
                                "Cancel"
                            } else {
                                "Disconnect"
                            }
                        }
                    }
                }
                if removable {
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE_SERVER,
                        "data-part": "remove-button",
                        "aria-label": "{remove_label}",
                        onclick: move |_| {
                            if let Some(handler) = on_remove {
                                handler.call(id.peek().clone());
                            }
                        },
                        "Remove"
                    }
                }
            }
        }
    }
}

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
    let mut state = use_signal(|| McpAddFormMachine::initial(&()));
    let mut send = move |event: McpAddFormEvent| {
        let step = McpAddFormMachine::transition(&state.peek(), event, &());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
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

/// A server row that looks its own outcome up in the manager's map.
#[component]
#[allow(clippy::needless_pass_by_value, reason = "props are owned by the component macro")]
fn ManagedServer(
    server: McpServerInfo,
    outcomes: ReadSignal<HashMap<String, Result<Vec<String>, String>>>,
    on_connect: Option<EventHandler<String>>,
    on_disconnect: Option<EventHandler<String>>,
    on_remove: Option<EventHandler<String>>,
) -> Element {
    let id = server.id.clone();
    let outcome = use_memo(move || outcomes.read().get(&id).cloned());
    rsx! {
        AssistantMcpServer { server, outcome, on_connect, on_disconnect, on_remove }
    }
}

/// The server list. Built-in connectors and custom servers sit in two sections. The owner holds
/// everything: `outcomes` maps a server id to its latest connection result (see
/// [`AssistantMcpServer`]); `on_add` receives (name, url) and is answered through `add_result`
/// (see [`AssistantMcpAddForm`]). The add form closes itself when the owner accepts.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    reason = "one rsx! tree; props are owned by the component macro"
)]
pub fn AssistantMcpManager(
    /// The servers, connectors and custom.
    servers: ReadSignal<Vec<McpServerInfo>>,
    /// Latest connection result per server id.
    #[props(default)]
    outcomes: ReadSignal<HashMap<String, Result<Vec<String>, String>>>,
    /// Called with the server id when a connect is requested.
    on_connect: Option<EventHandler<String>>,
    /// Called with the server id when a disconnect is requested.
    on_disconnect: Option<EventHandler<String>>,
    /// Called with the server id when a custom server is removed.
    on_remove: Option<EventHandler<String>>,
    /// Called with (name, url) when the person adds a custom server.
    on_add: Option<EventHandler<(String, String)>>,
    /// The owner's answer to the last `on_add`.
    #[props(default)]
    add_result: ReadSignal<Option<Result<(), String>>>,
) -> Element {
    let mut adding = use_signal(|| false);
    use_effect(move || {
        if matches!(add_result(), Some(Ok(()))) {
            adding.set(false);
        }
    });
    let all = servers.read();
    let (connectors, customs) = h::split_servers(&all);
    rsx! {
        div { "data-scope": h::SCOPE_MANAGER, "data-part": "root",
            section { "data-scope": h::SCOPE_MANAGER, "data-part": "connectors",
                h3 { "data-scope": h::SCOPE_MANAGER, "data-part": "heading", "Connectors" }
                ul { "data-scope": h::SCOPE_MANAGER, "data-part": "list",
                    for s in connectors {
                        ManagedServer { key: "{s.id}", server: s.clone(), outcomes, on_connect, on_disconnect, on_remove }
                    }
                }
            }
            section { "data-scope": h::SCOPE_MANAGER, "data-part": "custom-servers",
                div { "data-scope": h::SCOPE_MANAGER, "data-part": "section-head",
                    h3 { "data-scope": h::SCOPE_MANAGER, "data-part": "heading", "Custom servers" }
                    if !adding() {
                        span { "data-scope": h::SCOPE_MANAGER, "data-part": "add-trigger",
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Sm,
                                on_click: move |_| adding.set(true),
                                "Add custom server"
                            }
                        }
                    }
                }
                if adding() {
                    AssistantMcpAddForm {
                        result: add_result,
                        on_submit: move |v| {
                            if let Some(handler) = on_add {
                                handler.call(v);
                            }
                        },
                        on_cancel: move |()| adding.set(false),
                    }
                }
                ul { "data-scope": h::SCOPE_MANAGER, "data-part": "list",
                    for s in customs.iter() {
                        ManagedServer { key: "{s.id}", server: (*s).clone(), outcomes, on_connect, on_disconnect, on_remove }
                    }
                }
                if customs.is_empty() && !adding() {
                    p { "data-scope": h::SCOPE_MANAGER, "data-part": "empty",
                        "No custom servers yet. Add one to give the assistant more tools."
                    }
                }
            }
        }
    }
}

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
    let mut state = use_signal(|| McpElicitationMachine::initial(&input.peek()));
    let mut send = move |event: McpElicitationEvent| {
        let step = McpElicitationMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
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
                                            "aria-describedby": "mcp-elicit-{f.name}-error",
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
                                    id: "mcp-elicit-{f.name}-error",
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
