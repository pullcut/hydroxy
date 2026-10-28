use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_dioxus::assistant_mcp::{
    AssistantMcpElicitation, AssistantMcpManager, ElicitField, McpServerInfo,
};

async fn sleep(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

fn tools_for(id: &str) -> Vec<String> {
    let names: &[&str] = match id {
        "github" => &["search_repositories", "get_pull_request", "create_issue", "list_commits"],
        "linear" => &["search_issues", "create_issue", "update_issue"],
        "filesystem" => &["read_file", "list_directory"],
        _ => &["list_items", "get_item", "search"],
    };
    names.iter().map(|n| (*n).to_string()).collect()
}

#[component]
pub fn AssistantMcpDemo() -> Element {
    let mut servers = use_signal(|| {
        vec![
            McpServerInfo::connector("github", "GitHub", "Read repositories, pull requests and issues."),
            McpServerInfo::connector("linear", "Linear", "Search and update issues in your workspace."),
            McpServerInfo::connector("filesystem", "Filesystem", "Read files from a folder you choose."),
            McpServerInfo::custom("notes", "Team notes", "https://notes.acme.dev/mcp"),
        ]
    });
    let mut outcomes = use_signal(HashMap::<String, Result<Vec<String>, String>>::new);
    let mut add_result = use_signal(|| None::<Result<(), String>>);
    // Linear fails its first attempt so Retry shows.
    let mut linear_failed = use_signal(|| false);
    let mut next_custom = use_signal(|| 1_u32);
    let mut answers = use_signal(|| None::<Vec<(String, String)>>);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 560px;",
            AssistantMcpManager {
                servers,
                outcomes,
                add_result,
                on_connect: move |id: String| {
                    outcomes.write().remove(&id);
                    spawn(async move {
                        sleep(1200).await;
                        let result = if id == "linear" && !linear_failed() {
                            linear_failed.set(true);
                            Err("Authorization expired".to_string())
                        } else {
                            Ok(tools_for(&id))
                        };
                        outcomes.write().insert(id, result);
                    });
                },
                on_disconnect: move |id: String| {
                    outcomes.write().remove(&id);
                },
                on_remove: move |id: String| {
                    servers.write().retain(|s| s.id != id);
                    outcomes.write().remove(&id);
                },
                on_add: move |(name, url): (String, String)| {
                    add_result.set(None);
                    spawn(async move {
                        sleep(900).await;
                        if url.contains("taken") {
                            add_result.set(Some(Err("A server at that address is already added.".to_string())));
                            return;
                        }
                        let n = next_custom();
                        next_custom.set(n + 1);
                        servers.write().push(McpServerInfo::custom(format!("custom-{n}"), name, url));
                        add_result.set(Some(Ok(())));
                    });
                },
            }
            div { style: "display: grid; gap: var(--space-3);",
                AssistantMcpElicitation {
                    title: "Linear needs a few details",
                    message: "To file the issue, Linear asks which repository it belongs to.",
                    fields: vec![
                        ElicitField::text("repository", "Repository"),
                        ElicitField::choice("branch", "Branch", &["main", "develop", "release"]),
                        ElicitField::bool("private", "Keep the issue private"),
                    ],
                    on_submit: move |a: Vec<(String, String)>| answers.set(Some(a)),
                    on_cancel: move |()| answers.set(None),
                }
                if let Some(a) = answers() {
                    p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                        {a.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ")}
                    }
                }
            }
        }
    }
}
