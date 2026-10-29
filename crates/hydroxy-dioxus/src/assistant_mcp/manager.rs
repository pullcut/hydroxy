//! The server list manager.

use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_headless::assistant_mcp::{self as h, McpServerInfo};

use super::add_form::AssistantMcpAddForm;
use super::server::AssistantMcpServer;
use crate::button::{Button, ButtonSize, ButtonVariant};

/// A server row that looks its own outcome up in the manager's map.
#[component]
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
#[allow(clippy::implicit_hasher, reason = "props take a std HashMap; generalising the hasher only adds plumbing")]
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
