use dioxus::prelude::*;
use hydroxy_dioxus::reaction_bar::{Reaction, ReactionBar, toggle};

#[component]
pub fn ReactionBarDemo() -> Element {
    let mut reactions = use_signal(|| {
        vec![
            Reaction::new("👍", "thumbs up", &["Jonas Berg", "Priya Raman", "Leo Martins"]).mine(),
            Reaction::new("🎉", "party popper", &["Sana Haddad"]),
            Reaction::new("👀", "eyes", &["Kofi Mensah", "Inès Laurent"]),
        ]
    });
    // A real app would store these per user, say in the profile.
    let mut recent = use_signal(|| vec!["🚀".to_string(), "🙏".to_string(), "✅".to_string()]);

    rsx! {
        div { style: "display: grid; gap: var(--space-3); max-width: 460px; padding: var(--space-4); border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-card);",
            p { style: "margin: 0; font: var(--text-body); color: var(--text-1);",
                "Release 2.4 is out. Thanks everyone who tested the beta over the weekend."
            }
            ReactionBar {
                reactions: reactions(),
                on_toggle: move |(emoji, name): (String, String)| toggle(&mut reactions.write(), &emoji, &name),
                recent: recent(),
                on_recent_change: move |list| recent.set(list),
            }
        }
    }
}
