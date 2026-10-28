use dioxus::prelude::*;
use hydroxy_dioxus::presence_bar::{Collaborator, PresenceBar, PresenceStatus};

#[component]
pub fn PresenceBarDemo() -> Element {
    let mut people = use_signal(|| {
        vec![
            Collaborator::new("u-maya", "Maya Okafor").me(),
            Collaborator::new("u-jonas", "Jonas Berg"),
            Collaborator::new("u-priya", "Priya Raman").status(PresenceStatus::Idle),
            Collaborator::new("u-leo", "Leo Martins"),
            Collaborator::new("u-sana", "Sana Haddad").status(PresenceStatus::Away),
            Collaborator::new("u-tom", "Tomás Ruiz"),
            Collaborator::new("u-ines", "Inès Laurent").status(PresenceStatus::Idle),
            Collaborator::new("u-kofi", "Kofi Mensah"),
        ]
    });
    let here = people.read().iter().any(|c| c.id == "u-ana");

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            div { style: "display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); padding: var(--space-3) var(--space-4); border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-card);",
                span { style: "font: var(--text-ui); color: var(--text-1);", "Q3 planning doc" }
                PresenceBar { people: people(), max: 5 }
            }
            button {
                r#type: "button",
                class: "ox-button",
                "data-variant": "secondary",
                "data-size": "sm",
                style: "justify-self: start;",
                onclick: move |_| {
                    let mut p = people.write();
                    if let Some(i) = p.iter().position(|c| c.id == "u-ana") {
                        p.remove(i);
                    } else {
                        p.push(Collaborator::new("u-ana", "Ana Lima"));
                    }
                },
                if here { "Ana leaves" } else { "Ana joins" }
            }
        }
    }
}
