//! Assistant MCP: a manager of tool servers, one server row, the add-server form, and an
//! elicitation card where a server asks the person for input.
//!
//! The owner does the connecting. A row asks through `on_connect` and the owner reports back
//! through `outcome`; the row's machine decides which transitions are allowed.

use std::collections::HashMap;

use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use hydroxy_headless::assistant_mcp::{
    self as h, McpAddFormCommand, McpAddFormEvent, McpAddFormMachine, McpElicitationCommand,
    McpElicitationEvent, McpElicitationInput, McpElicitationMachine, McpField, McpServerCommand,
    McpServerEvent, McpServerMachine,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonVariant};

pub use hydroxy_headless::assistant_mcp::{
    Connection, ElicitField, ElicitKind, ElicitStatus, FormStatus, McpServerInfo,
};

/// One server row. The owner holds the connection: press Connect and `on_connect` runs with the
/// server id; when the connection settles, set `outcome` to `Some(Ok(tools))` or `Some(Err(message))`.
/// Clear `outcome` in `on_connect` so a repeated result is noticed.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, clippy::implicit_hasher, reason = "one view! tree; props are owned by the component macro")]
pub fn AssistantMcpServer(
    /// The server to show.
    server: McpServerInfo,
    /// What the owner's last connection attempt produced: the tool names, or an error message.
    #[prop(optional, into)]
    outcome: MaybeProp<Result<Vec<String>, String>>,
    /// Called with the server id when Connect or Retry is pressed.
    #[prop(optional, into)]
    on_connect: Option<Callback<String>>,
    /// Called with the server id when Disconnect (or Cancel while connecting) is pressed.
    #[prop(optional, into)]
    on_disconnect: Option<Callback<String>>,
    /// Called with the server id when Remove is pressed. Only custom servers show Remove.
    #[prop(optional, into)]
    on_remove: Option<Callback<String>>,
) -> impl IntoView {
    let state = RwSignal::new(McpServerMachine::initial(&()));
    let id = StoredValue::new(server.id.clone());
    let send = move |event: McpServerEvent| {
        let Some(step) = state.try_with_untracked(|s| McpServerMachine::transition(s, event, &())) else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let (cb, sid) = match command {
                McpServerCommand::Connect => (on_connect, id.get_value()),
                McpServerCommand::Disconnect => (on_disconnect, id.get_value()),
            };
            if let Some(cb) = cb {
                cb.run(sid);
            }
        }
    };
    Effect::new(move |_| {
        if let Some(result) = outcome.get() {
            send(match result {
                Ok(tools) => McpServerEvent::Connected(tools),
                Err(msg) => McpServerEvent::Failed(msg),
            });
        }
    });

    let connection = move || state.with(|s| s.connection.clone());
    let name = server.name.clone();
    let remove_label = format!("Remove {name}");
    let removable = server.custom && on_remove.is_some();
    view! {
        <li
            data-scope=h::SCOPE_SERVER
            data-part="root"
            data-connection-state=move || connection().as_str()
        >
            <span data-scope=h::SCOPE_SERVER data-part="icon" aria-hidden="true">{server.initial()}</span>
            <div data-scope=h::SCOPE_SERVER data-part="body">
                <span data-scope=h::SCOPE_SERVER data-part="name">{server.name.clone()}</span>
                <span data-scope=h::SCOPE_SERVER data-part="description">{server.description.clone()}</span>
                <span data-scope=h::SCOPE_SERVER data-part="status" role="status">
                    {move || state.with(|s| h::status_label(&s.connection, s.tools.len()))}
                </span>
                {move || {
                    let tools = state.with(|s| s.tools.clone());
                    (connection() == Connection::Connected && !tools.is_empty())
                        .then(|| {
                            view! {
                                <ul data-scope=h::SCOPE_SERVER data-part="tools" aria-label="Tools">
                                    {tools
                                        .into_iter()
                                        .map(|t| view! { <li data-scope=h::SCOPE_SERVER data-part="tool">{t}</li> })
                                        .collect_view()}
                                </ul>
                            }
                        })
                }}
            </div>
            <div data-scope=h::SCOPE_SERVER data-part="actions">
                {move || {
                    let c = connection();
                    if h::can_connect(&c) {
                        let label = if matches!(c, Connection::Error(_)) { "Retry" } else { "Connect" };
                        EitherOf3::A(view! {
                            <span data-scope=h::SCOPE_SERVER data-part="connect-button">
                                <Button
                                    variant=ButtonVariant::Secondary
                                    size=ButtonSize::Sm
                                    on_click=move |_| send(McpServerEvent::Connect)
                                >
                                    {label}
                                </Button>
                            </span>
                        })
                    } else if h::can_disconnect(&c) {
                        let label = if c == Connection::Connecting { "Cancel" } else { "Disconnect" };
                        EitherOf3::B(view! {
                            <span data-scope=h::SCOPE_SERVER data-part="disconnect-button">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    on_click=move |_| send(McpServerEvent::Disconnect)
                                >
                                    {label}
                                </Button>
                            </span>
                        })
                    } else {
                        EitherOf3::C(())
                    }
                }}
                {removable
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                data-scope=h::SCOPE_SERVER
                                data-part="remove-button"
                                aria-label=remove_label
                                on:click=move |_| {
                                    if let Some(cb) = on_remove {
                                        cb.run(id.get_value());
                                    }
                                }
                            >
                                "Remove"
                            </button>
                        }
                    })}
            </div>
        </li>
    }
}

/// The add-server form. `on_submit` runs with the trimmed name and URL once both pass
/// validation. Answer it by setting `result` to `Some(Ok(()))` or `Some(Err(message))`; the
/// message shows as an alert. Clear `result` in `on_submit` so a repeated answer is noticed.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, clippy::implicit_hasher, reason = "one view! tree; props are owned by the component macro")]
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
    let send = move |event: McpAddFormEvent| {
        let Some(step) = state.try_with_untracked(|s| McpAddFormMachine::transition(s, event, &())) else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
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

/// The server list. Built-in connectors and custom servers sit in two sections. The owner holds
/// everything: `outcomes` maps a server id to its latest connection result (see
/// [`AssistantMcpServer`]); `on_add` receives (name, url) and is answered through `add_result`
/// (see [`AssistantMcpAddForm`]). The add form closes itself when the owner accepts.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, clippy::implicit_hasher, reason = "one view! tree; props are owned by the component macro")]
pub fn AssistantMcpManager(
    /// The servers, connectors and custom.
    #[prop(into)]
    servers: Signal<Vec<McpServerInfo>>,
    /// Latest connection result per server id.
    #[prop(optional, into)]
    outcomes: Signal<HashMap<String, Result<Vec<String>, String>>>,
    /// Called with the server id when a connect is requested.
    #[prop(optional, into)]
    on_connect: Option<Callback<String>>,
    /// Called with the server id when a disconnect is requested.
    #[prop(optional, into)]
    on_disconnect: Option<Callback<String>>,
    /// Called with the server id when a custom server is removed.
    #[prop(optional, into)]
    on_remove: Option<Callback<String>>,
    /// Called with (name, url) when the person adds a custom server.
    #[prop(optional, into)]
    on_add: Option<Callback<(String, String)>>,
    /// The owner's answer to the last `on_add`.
    #[prop(optional, into)]
    add_result: MaybeProp<Result<(), String>>,
) -> impl IntoView {
    let adding = RwSignal::new(false);
    Effect::new(move |_| {
        if matches!(add_result.get(), Some(Ok(()))) {
            adding.set(false);
        }
    });
    let section = move |custom: bool| {
        let rows = move || {
            servers.with(|all| {
                let (connectors, customs) = h::split_servers(all);
                (if custom { customs } else { connectors }).into_iter().cloned().collect::<Vec<_>>()
            })
        };
        view! {
            <For
                each=rows
                key=|s| s.id.clone()
                children=move |s| {
                    let sid = s.id.clone();
                    let outcome = Signal::derive(move || outcomes.with(|m| m.get(&sid).cloned()));
                    view! {
                        <AssistantMcpServer
                            server=s
                            outcome=outcome
                            on_connect=move |i: String| {
                                if let Some(cb) = on_connect {
                                    cb.run(i);
                                }
                            }
                            on_disconnect=move |i: String| {
                                if let Some(cb) = on_disconnect {
                                    cb.run(i);
                                }
                            }
                            on_remove=move |i: String| {
                                if let Some(cb) = on_remove {
                                    cb.run(i);
                                }
                            }
                        />
                    }
                }
            />
        }
    };
    let has_custom = move || servers.with(|all| all.iter().any(|s| s.custom));
    view! {
        <div data-scope=h::SCOPE_MANAGER data-part="root">
            <section data-scope=h::SCOPE_MANAGER data-part="connectors">
                <h3 data-scope=h::SCOPE_MANAGER data-part="heading">"Connectors"</h3>
                <ul data-scope=h::SCOPE_MANAGER data-part="list">{section(false)}</ul>
            </section>
            <section data-scope=h::SCOPE_MANAGER data-part="custom-servers">
                <div data-scope=h::SCOPE_MANAGER data-part="section-head">
                    <h3 data-scope=h::SCOPE_MANAGER data-part="heading">"Custom servers"</h3>
                    {move || {
                        (!adding.get())
                            .then(|| {
                                view! {
                                    <span data-scope=h::SCOPE_MANAGER data-part="add-trigger">
                                        <Button
                                            variant=ButtonVariant::Secondary
                                            size=ButtonSize::Sm
                                            on_click=move |_| adding.set(true)
                                        >
                                            "Add custom server"
                                        </Button>
                                    </span>
                                }
                            })
                    }}
                </div>
                {move || {
                    adding
                        .get()
                        .then(|| {
                            view! {
                                <AssistantMcpAddForm
                                    result=add_result
                                    on_submit=move |v| {
                                        if let Some(cb) = on_add {
                                            cb.run(v);
                                        }
                                    }
                                    on_cancel=move |()| adding.set(false)
                                />
                            }
                        })
                }}
                <ul data-scope=h::SCOPE_MANAGER data-part="list">{section(true)}</ul>
                {move || {
                    (!has_custom() && !adding.get())
                        .then(|| {
                            view! {
                                <p data-scope=h::SCOPE_MANAGER data-part="empty">
                                    "No custom servers yet. Add one to give the assistant more tools."
                                </p>
                            }
                        })
                }}
            </section>
        </div>
    }
}

/// A card where a server asks the person for input. Submit sends the answers through
/// `on_submit` once every required field has one; Decline calls `on_cancel`. After either, the
/// form locks and a status line says what happened.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, clippy::implicit_hasher, reason = "one view! tree; props are owned by the component macro")]
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
    let input = StoredValue::new(McpElicitationInput { fields: fields.clone() });
    let state = RwSignal::new(input.with_value(McpElicitationMachine::initial));
    let send = move |event: McpElicitationEvent| {
        let Some(step) = state
            .try_with_untracked(|s| input.try_with_value(|i| McpElicitationMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
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
    let set = move |name: &str, value: String| send(McpElicitationEvent::Set { name: name.to_string(), value });

    let rows = fields
        .into_iter()
        .map(|f| {
            let ElicitField { name, label, kind, required } = f;
            let missing = {
                let name = name.clone();
                Memo::new(move |_| state.with(|s| s.missing.contains(&name)))
            };
            let err_id = format!("mcp-elicit-{name}-error");
            let error = {
                let err_id = err_id.clone();
                view! {
                    <span data-scope=h::SCOPE_ELICITATION data-part="field-error" id=err_id role="alert">
                        {move || missing.get().then_some("This answer is required.")}
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
                            aria-describedby=err_id.clone()
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
                        {label.clone()}
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
