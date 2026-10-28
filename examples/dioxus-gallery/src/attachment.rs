use dioxus::prelude::*;
use hydroxy_dioxus::attachment::{Attachment, AttachmentVariant};

const THUMB: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 44 44'%3E%3Crect width='44' height='44' fill='%237c5cd6'/%3E%3Ccircle cx='32' cy='12' r='5' fill='%23f6d365'/%3E%3Cpath d='M0 44 16 22l12 14 6-6 10 14z' fill='%23fff' fill-opacity='.8'/%3E%3C/svg%3E";

#[component]
pub fn AttachmentDemo() -> Element {
    let mut files = use_signal(|| {
        vec![
            ("interviews", "interviews-q3.pdf", 2_516_582_u64, "application/pdf"),
            ("tickets", "tickets-sept.csv", 540_000, "text/csv"),
            ("deck", "board-update.pptx", 8_400_000, ""),
            ("export", "export-2026-10.zip", 120_500_000, ""),
        ]
    });

    rsx! {
        div { style: "display: grid; gap: var(--space-6);",
            // Chips with a remove button.
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-2);",
                for (id , name , size , mime) in files() {
                    Attachment {
                        key: "{id}",
                        name,
                        size,
                        mime,
                        on_remove: move |()| files.write().retain(|f| f.0 != id),
                    }
                }
            }
            // Cards, one with a thumbnail, without a remove button.
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
                Attachment {
                    name: "whiteboard.png",
                    size: 1_240_000_u64,
                    mime: "image/png",
                    thumbnail: THUMB,
                    variant: AttachmentVariant::Card,
                }
                Attachment { name: "weekly-digest.md", size: 4_300_u64, variant: AttachmentVariant::Card }
            }
        }
    }
}
