use std::{collections::HashMap, time::Duration};

use hydroxy_leptos::assistant_mcp::{AssistantMcpElicitation, AssistantMcpManager, ElicitField, McpServerInfo};
use leptos::prelude::*;

fn tools_for(id: &str) -> Vec<String> {
    let names: &[&str] = match id {
        "github" => &["search_repositories", "get_pull_request", "create_issue", "list_commits"],
        "linear" => &["search_issues", "create_issue", "update_issue"],
        "filesystem" => &["read_file", "list_directory"],
        _ => &["list_items", "get_item", "search"],
    };
    names.iter().map(|n| (*n).to_string()).collect()
}

pub fn demo() -> AnyView {
    let (servers, set_servers) = signal(vec![
        McpServerInfo::connector("github", "GitHub", "Read repositories, pull requests and issues."),
        McpServerInfo::connector("linear", "Linear", "Search and update issues in your workspace."),
        McpServerInfo::connector("filesystem", "Filesystem", "Read files from a folder you choose."),
        McpServerInfo::custom("notes", "Team notes", "https://notes.acme.dev/mcp"),
    ]);
    let outcomes = RwSignal::new(HashMap::<String, Result<Vec<String>, String>>::new());
    let (add_result, set_add_result) = signal(None::<Result<(), String>>);
    // Linear fails its first attempt so Retry shows.
    let linear_failed = StoredValue::new(false);
    let next_custom = StoredValue::new(1_u32);
    let (answers, set_answers) = signal(None::<Vec<(String, String)>>);

    let on_answers = move |a: Vec<(String, String)>| set_answers.set(Some(a));

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 560px;">
            <AssistantMcpManager
                servers=servers
                outcomes=outcomes
                add_result=add_result
                on_connect=move |id: String| {
                    outcomes.update(|m| {
                        m.remove(&id);
                    });
                    set_timeout(
                        move || {
                            let first_linear = id == "linear" && !linear_failed.get_value();
                            let result = if first_linear {
                                linear_failed.set_value(true);
                                Err("Authorization expired".to_string())
                            } else {
                                Ok(tools_for(&id))
                            };
                            outcomes.try_update(|m| m.insert(id, result));
                        },
                        Duration::from_millis(1200),
                    );
                }
                on_disconnect=move |id: String| {
                    outcomes.update(|m| {
                        m.remove(&id);
                    });
                }
                on_remove=move |id: String| {
                    set_servers.update(|v| v.retain(|s| s.id != id));
                    outcomes.update(|m| {
                        m.remove(&id);
                    });
                }
                on_add=move |(name, url): (String, String)| {
                    set_add_result.set(None);
                    set_timeout(
                        move || {
                            if url.contains("taken") {
                                set_add_result.try_set(Some(Err("A server at that address is already added.".to_string())));
                                return;
                            }
                            let n = next_custom.get_value();
                            next_custom.set_value(n + 1);
                            set_servers.try_update(|v| v.push(McpServerInfo::custom(format!("custom-{n}"), name, url)));
                            set_add_result.try_set(Some(Ok(())));
                        },
                        Duration::from_millis(900),
                    );
                }
            />
            <div style="display: grid; gap: var(--space-3);">
                <AssistantMcpElicitation
                    title="Linear needs a few details"
                    message="To file the issue, Linear asks which repository it belongs to."
                    fields=vec![
                        ElicitField::text("repository", "Repository"),
                        ElicitField::choice("branch", "Branch", &["main", "develop", "release"]),
                        ElicitField::bool("private", "Keep the issue private"),
                    ]
                    on_submit=on_answers
                    on_cancel=move |()| set_answers.set(None)
                />
                {move || {
                    answers
                        .get()
                        .map(|a| {
                            let text = a.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ");
                            view! { <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">{text}</p> }
                        })
                }}
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "assistant-mcp",
    name: "Assistant MCP",
    group: "Chat",
    summary: "Manage the tool servers an assistant can use. Built-in connectors and custom servers sit in two lists, each row connecting on request and listing its tools; custom servers are added from an inline form. A separate card lets a server ask the person for input.",
    demo,
    leptos: include_str!("assistant_mcp.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/assistant_mcp.rs"),
    parts: &[
        &["AssistantMcpManager", "connectors / custom-servers", "The two sections, each with a heading and a list"],
        &["", "add-trigger", "Button that reveals the add form"],
        &["", "empty", "Shown when there are no custom servers"],
        &["AssistantMcpServer", "root", "A row. data-connection-state is disconnected, connecting, connected or error"],
        &["", "icon / name / description / status", "Initial letter, text and the status line"],
        &["", "tools", "Tool names, shown while connected"],
        &["", "connect-button / disconnect-button", "Whichever the connection allows"],
        &["", "remove-button", "Custom servers only"],
        &["AssistantMcpAddForm", "root / field / input", "Name and URL fields with inline errors in field-error"],
        &["", "error / submit", "The owner's rejection (role alert) and the submit button"],
        &[
            "AssistantMcpElicitation",
            "root / title / message",
            "The card. data-status is pending, submitted or cancelled",
        ],
        &["", "field / label / input / checkbox / options", "One question and its control"],
        &["", "submit / cancel / result", "Buttons while pending, then the outcome line"],
    ],
    keys: &[&["Enter", "In a field: submit the form"], &["Space", "On a checkbox or radio: toggle or choose"]],
    props: &[
        &["servers", "Vec<McpServerInfo>", "Manager: the servers. A server has id, name, description and custom"],
        &[
            "outcomes",
            "HashMap<String, Result<Vec<String>, String>>",
            "Manager: latest connection result per server id, tool names or an error. Clear an entry in on_connect",
        ],
        &[
            "on_connect / on_disconnect / on_remove",
            "Callback<String>",
            "Server id. You do the connecting, then fill outcomes",
        ],
        &[
            "on_add / add_result",
            "Callback<(String, String)> / Result<(), String>",
            "Name and URL after validation. Answer with add_result; the form closes on Ok and shows the message on Err",
        ],
        &["outcome", "Result<Vec<String>, String>", "Server row: the same result for one server"],
        &[
            "title / message / fields",
            "String / String / Vec<ElicitField>",
            "Elicitation: text, bool and choice fields, required unless marked optional",
        ],
        &[
            "on_submit / on_cancel",
            "Callback<Vec<(String, String)>> / Callback<()>",
            "Elicitation: answers once every required field has one, or the decline",
        ],
    ],
};
