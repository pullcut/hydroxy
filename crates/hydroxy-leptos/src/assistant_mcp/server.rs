//! One MCP server row.

use hydroxy_headless::assistant_mcp::{self as h, McpServerCommand, McpServerEvent, McpServerMachine};
use hydroxy_statechart::Machine;
use leptos::either::EitherOf3;
use leptos::prelude::*;

use super::{Connection, McpServerInfo};
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::drive::drive;

/// One server row. The owner holds the connection: press Connect and `on_connect` runs with the
/// server id; when the connection settles, set `outcome` to `Some(Ok(tools))` or `Some(Err(message))`.
/// Clear `outcome` in `on_connect` so a repeated result is noticed.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::implicit_hasher,
    reason = "one view! tree; props are owned by the component macro"
)]
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
    let input = Signal::stored(());
    let send = move |event: McpServerEvent| {
        for command in drive::<McpServerMachine, _>(state, input, event) {
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
