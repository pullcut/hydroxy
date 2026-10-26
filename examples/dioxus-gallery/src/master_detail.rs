use dioxus::prelude::*;
use oxide_dioxus::master_detail::{MasterDetail, MasterDetailRow};

const MESSAGES: [(&str, &str, &str); 5] = [
    ("m1", "Launch checklist", "Everything on the list is done except the changelog. I will have it by Thursday."),
    ("m2", "Invoice 4021", "The invoice was sent on the 3rd and is due in two weeks. No reminders have gone out yet."),
    ("m3", "Standup notes", "Ana is blocked on the audio bug. Ben is picking up the transcript work next."),
    ("m4", "Design review", "The new player looks right. Spacing on the rate button needs another pass."),
    ("m5", "Welcome aboard", "Glad to have you. Your laptop ships Monday and your first week is planned."),
];

#[component]
pub fn MasterDetailDemo() -> Element {
    // The host owns the selection. Resize below 640px wide to see one pane at a time.
    let mut selected = use_signal(|| Some("m1".to_string()));
    let current = MESSAGES.iter().find(|(id, _, _)| Some(*id) == selected().as_deref());

    rsx! {
        div { style: "max-width: 720px; height: 320px; display: grid;",
            MasterDetail {
                selected,
                on_select: move |id| selected.set(id),
                list_heading: rsx! { "Inbox" },
                list: rsx! {
                    for (id, title, _) in MESSAGES {
                        MasterDetailRow { key: "{id}", id, strong { "{title}" } }
                    }
                },
                detail: rsx! {
                    if let Some((_, title, body)) = current {
                        h3 { style: "margin: 0 0 var(--space-3); font: var(--text-h3);", "{title}" }
                        p { style: "margin: 0; color: var(--text-2);", "{body}" }
                    } else {
                        p { style: "margin: 0; color: var(--text-2);", "Choose a message." }
                    }
                },
            }
        }
    }
}
