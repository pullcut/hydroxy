use dioxus::prelude::*;
use hydroxy_dioxus::callout::{Callout, CalloutKind};

#[component]
pub fn CalloutDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-4);",
            Callout { title: "Sources are attached",
                "Each answer lists the files it read, with the page for each figure."
            }
            Callout { kind: CalloutKind::Tip, title: "Compare two exports",
                "Drop both files in and ask what changed between them."
            }
            Callout { kind: CalloutKind::Warning, title: "Two items need your judgement",
                "The September export has 410 fewer tickets than the August summary."
            }
            Callout { kind: CalloutKind::Danger, title: "Deleting removes 12 files",
                "Summaries that cite them lose their sources. This can't be undone."
            }
            Callout { "A callout can be a single line with no title." }
        }
    }
}
