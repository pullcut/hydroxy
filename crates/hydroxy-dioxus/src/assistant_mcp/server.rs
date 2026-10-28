//! One MCP server row.

use dioxus::prelude::*;
use hydroxy_headless::assistant_mcp::{
    self as h, Connection, McpServerCommand, McpServerEvent, McpServerInfo, McpServerMachine,
};
use hydroxy_statechart::Machine;

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
    let state = use_signal(|| McpServerMachine::initial(&()));
    let unit = use_signal(|| ());
    let id = use_signal(|| server.id.clone());
    let send = move |event: McpServerEvent| {
        let commands = drive::<McpServerMachine>(state, unit, event);
        for command in commands {
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
