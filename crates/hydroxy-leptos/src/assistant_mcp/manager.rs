//! The server list with its two sections and the add form.

use std::collections::HashMap;

use hydroxy_headless::assistant_mcp::{self as h};
use leptos::prelude::*;

use super::McpServerInfo;
use super::add_form::AssistantMcpAddForm;
use super::server::AssistantMcpServer;
use crate::button::{Button, ButtonSize, ButtonVariant};

/// The server list. Built-in connectors and custom servers sit in two sections. The owner holds
/// everything: `outcomes` maps a server id to its latest connection result (see
/// [`AssistantMcpServer`]); `on_add` receives (name, url) and is answered through `add_result`
/// (see [`AssistantMcpAddForm`]). The add form closes itself when the owner accepts.
#[component]
#[allow(clippy::implicit_hasher, reason = "props take a std HashMap; generalising the hasher only adds plumbing")]
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
