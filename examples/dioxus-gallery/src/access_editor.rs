use dioxus::prelude::*;
use oxide_dioxus::access_editor::{AccessEditor, AccessPerson};
use oxide_dioxus::select::SelectOption;

#[component]
pub fn AccessEditorDemo() -> Element {
    let roles = vec![SelectOption::new("editor", "Can edit"), SelectOption::new("viewer", "Can view")];
    let people = vec![
        AccessPerson::new("ana@oxide.dev", "Ana Kowal", "owner").owner(),
        AccessPerson::new("jo@oxide.dev", "Jo Marsh", "editor"),
        AccessPerson::new("sam@partner.io", "Sam Lee", "viewer"),
    ];
    let mut count = use_signal(|| people.len());

    rsx! {
        div { style: "display: grid; gap: var(--space-5); max-width: 520px;",
            AccessEditor {
                label: "Who has access",
                roles,
                new_role: "viewer",
                default_people: people,
                on_people_change: move |list: Vec<AccessPerson>| count.set(list.len()),
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "{count()} people have access" }
        }
    }
}
