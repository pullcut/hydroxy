use hydroxy_leptos::attachment::*;
use leptos::prelude::*;

const THUMB: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 44 44'%3E%3Crect width='44' height='44' fill='%237c5cd6'/%3E%3Ccircle cx='32' cy='12' r='5' fill='%23f6d365'/%3E%3Cpath d='M0 44 16 22l12 14 6-6 10 14z' fill='%23fff' fill-opacity='.8'/%3E%3C/svg%3E";

pub fn demo() -> AnyView {
    let files = RwSignal::new(vec![
        ("interviews", "interviews-q3.pdf", 2_516_582_u64, "application/pdf"),
        ("tickets", "tickets-sept.csv", 540_000, "text/csv"),
        ("deck", "board-update.pptx", 8_400_000, ""),
        ("export", "export-2026-10.zip", 120_500_000, ""),
    ]);

    view! {
        <div style="display: grid; gap: var(--space-6);">
            // Chips with a remove button.
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-2);">
                <For
                    each=move || files.get()
                    key=|f| f.0
                    children=move |(id, name, size, mime)| {
                        view! {
                            <Attachment
                                name=name
                                size=size
                                mime=mime
                                on_remove=move |()| files.update(|fs| fs.retain(|f| f.0 != id))
                            />
                        }
                    }
                />
            </div>
            // Cards, one with a thumbnail, without a remove button.
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
                <Attachment name="whiteboard.png" size=1_240_000 mime="image/png" thumbnail=THUMB variant=AttachmentVariant::Card />
                <Attachment name="weekly-digest.md" size=4_300 variant=AttachmentVariant::Card />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "attachment",
    name: "Attachment",
    group: "Forms",
    summary: "A file as a chip or a card: thumbnail or type badge, name, size and an optional remove button. It only displays; keep the list yourself.",
    demo,
    leptos: include_str!("attachment.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/attachment.rs"),
    parts: &[
        &[
            "Attachment",
            "root",
            "Wrapper. data-variant=chip|card, data-kind=image|pdf|document|spreadsheet|presentation|archive|audio|video|code|other",
        ],
        &["(inside)", "thumbnail", "<img alt=\"\"> preview, when a thumbnail URL is given"],
        &["(inside)", "badge", "Upper-case file extension, tinted by kind. Shown when there is no thumbnail"],
        &["(inside)", "body, name, meta", "The file name, and a line such as \"PDF, 2.4 MB\""],
        &["(inside)", "remove", "<button aria-label=\"Remove name\">, only with on_remove"],
    ],
    keys: &[&["Tab", "Moves to the remove button"], &["Space / Enter", "Removes the file"]],
    props: &[
        &["name", "String", "File name. Decides the badge when mime is empty"],
        &["size", "Option<u64>", "Size in bytes"],
        &["mime", "String", "MIME type, such as application/pdf"],
        &["thumbnail", "Option<String>", "URL of a preview image"],
        &["variant", "AttachmentVariant", "Chip (default) or Card"],
        &["on_remove", "Callback<()>", "Pressed remove. Without it there is no button"],
    ],
};
