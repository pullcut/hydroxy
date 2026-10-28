use dioxus::prelude::*;
use hydroxy_dioxus::app_shell::{AppShell, ScrollMode};
use hydroxy_dioxus::icon::IconName;
use hydroxy_dioxus::nav_item::NavItem;

#[component]
pub fn AppShellDemo() -> Element {
    let mut page = use_signal(|| "Overview");
    let items = [
        ("Overview", IconName::Inbox),
        ("Projects", IconName::FileText),
        ("Starred", IconName::Star),
        ("People", IconName::User),
    ];

    rsx! {
        // The shell is a fixed 460px tall here, with only the main region scrolling.
        // Below 768px wide the navigation becomes a drawer behind the menu button.
        div { style: "--ox-app-shell-nav-width: 180px; --ox-app-shell-aside-width: 150px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden;",
            AppShell {
                scroll_mode: ScrollMode::Main,
                host_block_size: 460.0,
                main_id: "app-shell-demo-main",
                skip_label: "Skip to content",
                navigation: rsx! {
                    div { style: "display: grid; gap: 2px;",
                        for (label, icon) in items {
                            NavItem {
                                key: "{label}",
                                label,
                                icon,
                                active: page() == label,
                                on_click: move |()| page.set(label),
                            }
                        }
                    }
                },
                header: rsx! { strong { style: "font: var(--text-h4);", "Acme" } },
                aside: rsx! { p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Notes and activity sit in the aside." } },
                footer: rsx! { span { "Footer" } },
                h2 { style: "margin: 0 0 var(--space-4); font: var(--text-h3);", "{page}" }
                p { style: "margin: 0 0 var(--space-4); color: var(--text-2);", "The header, navigation, aside and footer stay put while this region scrolls." }
                for i in 1..=12 {
                    p { key: "{i}", style: "margin: 0 0 var(--space-4); color: var(--text-2);", "Paragraph {i} of placeholder content to scroll." }
                }
            }
        }
    }
}
