//! `InsightNote`: an amethyst-tinted summary with the mark, a label and source chips.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};
use crate::source_chip::{SourceChip, SourceRow};

/// A short machine-written summary, with the files it came from.
#[component]
pub fn InsightNote(
    /// What the note is, such as "Summary" or "Weekly digest".
    #[props(default = "Summary".to_string())]
    label: String,
    /// File names behind the note, shown as source chips.
    #[props(default)]
    sources: Vec<String>,
    /// Control at the end of the header, such as a "Regenerate" button.
    action: Option<Element>,
    /// The note.
    children: Element,
) -> Element {
    rsx! {
        section { "data-scope": "insight-note", "aria-label": "{label}",
            div { "data-part": "head",
                Icon { name: IconName::Mark, size: 16 }
                span { "data-part": "label", "{label}" }
                if let Some(a) = action {
                    span { "data-part": "action", {a} }
                }
            }
            div { "data-part": "body", {children} }
            if !sources.is_empty() {
                SourceRow {
                    for s in sources {
                        SourceChip { key: "{s}", "{s}" }
                    }
                }
            }
        }
    }
}
