use dioxus::prelude::*;
use oxide_dioxus::button::{Button, ButtonVariant};
use oxide_dioxus::command_palette::{CommandItem, CommandPalette};
use oxide_dioxus::icon::IconName;

#[component]
pub fn CommandPaletteDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut last = use_signal(|| String::from("none yet"));
    let commands = vec![
        CommandItem::new("new", "New project", "Projects").icon(IconName::Plus).hint("N"),
        CommandItem::new("open", "Open Q3 customer research", "Projects").icon(IconName::FileText),
        CommandItem::new("summary", "Draft a summary", "Actions").icon(IconName::Lightbulb).keywords("write report"),
        CommandItem::new("copy", "Copy share link", "Actions").icon(IconName::Copy).hint("C"),
        CommandItem::new("archive", "Archive project", "Actions").icon(IconName::Inbox),
        CommandItem::new("search", "Search sources", "Navigate").icon(IconName::Search).hint("/"),
    ];

    rsx! {
        div { style: "display: flex; align-items: center; gap: var(--space-5); flex-wrap: wrap;",
            Button {
                variant: ButtonVariant::Secondary,
                icon_left: IconName::Search,
                on_click: move |_| open.set(true),
                "Search or run a command"
            }
            span { "Last command: {last}" }
        }
        // Controlled: the page owns `open`.
        CommandPalette {
            items: commands,
            open: Some(open()),
            on_open_change: move |o| open.set(o),
            on_select: move |id| last.set(id),
        }
    }
}
