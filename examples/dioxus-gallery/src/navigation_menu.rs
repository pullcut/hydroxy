use dioxus::prelude::*;
use oxide_dioxus::navigation_menu::{
    NavigationMenu, NavigationMenuEntry, NavigationMenuItem, NavigationMenuLink, NavigationMenuPanelLink,
};

#[component]
pub fn NavigationMenuDemo() -> Element {
    rsx! {
        // Hover a trigger and rest on it, or focus it and press Down. Moving to a neighbour switches at once.
        NavigationMenu {
            label: "Site",
            items: vec![
                NavigationMenuEntry::panel("product"),
                NavigationMenuEntry::panel("company"),
                NavigationMenuEntry::link("pricing"),
                NavigationMenuEntry::link("docs"),
            ],
            NavigationMenuItem { value: "product", label: "Product",
                NavigationMenuPanelLink { href: "#research", description: "Ask questions across every source you've added.", "Research" }
                NavigationMenuPanelLink { href: "#summaries", description: "Turn long threads into a one-page brief.", "Summaries" }
                NavigationMenuPanelLink { href: "#sources", description: "See which file each claim came from.", "Sources" }
            }
            NavigationMenuItem { value: "company", label: "Company",
                NavigationMenuPanelLink { href: "#about", description: "Who we are and why we built this.", "About" }
                NavigationMenuPanelLink { href: "#careers", description: "Open roles on the team.", "Careers" }
            }
            NavigationMenuLink { value: "pricing", href: "#pricing", "Pricing" }
            NavigationMenuLink { value: "docs", href: "#docs", current: true, "Docs" }
        }
    }
}
