use leptos::prelude::*;
use oxide_leptos::navigation_menu::*;

pub fn demo() -> AnyView {
    view! {
        // Hover a trigger and rest on it, or focus it and press Down. Moving to a neighbour switches at once.
        <NavigationMenu
            label="Site"
            items=vec![
                NavigationMenuEntry::panel("product"),
                NavigationMenuEntry::panel("company"),
                NavigationMenuEntry::link("pricing"),
                NavigationMenuEntry::link("docs"),
            ]
        >
            <NavigationMenuItem value="product" label="Product">
                <NavigationMenuPanelLink href="#research" description="Ask questions across every source you've added.">
                    "Research"
                </NavigationMenuPanelLink>
                <NavigationMenuPanelLink href="#summaries" description="Turn long threads into a one-page brief.">
                    "Summaries"
                </NavigationMenuPanelLink>
                <NavigationMenuPanelLink href="#sources" description="See which file each claim came from.">
                    "Sources"
                </NavigationMenuPanelLink>
            </NavigationMenuItem>
            <NavigationMenuItem value="company" label="Company">
                <NavigationMenuPanelLink href="#about" description="Who we are and why we built this.">
                    "About"
                </NavigationMenuPanelLink>
                <NavigationMenuPanelLink href="#careers" description="Open roles on the team.">
                    "Careers"
                </NavigationMenuPanelLink>
            </NavigationMenuItem>
            <NavigationMenuLink value="pricing" href="#pricing">"Pricing"</NavigationMenuLink>
            <NavigationMenuLink value="docs" href="#docs" current=true>"Docs"</NavigationMenuLink>
        </NavigationMenu>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "navigation-menu",
    name: "Navigation menu",
    group: "Navigation",
    summary: "A site nav with flyout panels. Hover opens a panel after a short rest and a close delay lets the pointer cross to it; the keyboard opens panels with Down.",
    demo,
    leptos: include_str!("navigation_menu.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/navigation_menu.rs"),
    parts: &[
        &["NavigationMenu", "root", "nav with an aria-label, holding the list. Owns the hover intent"],
        &["", "list", "ul of items"],
        &["NavigationMenuItem", "item / trigger", "li with a button: aria-expanded, aria-controls, popovertarget"],
        &["", "content", "popover=auto flyout panel anchored under its trigger"],
        &["NavigationMenuLink", "item / link", "li with a plain top-level link; aria-current=page when current"],
        &["NavigationMenuPanelLink", "panel-link", "A link in a panel, with an optional description"],
    ],
    keys: &[
        &["Left / Right", "Focuses the previous or next top-level item. An open panel follows"],
        &["Home / End", "Focuses the first or last top-level item"],
        &["Down", "On a trigger: opens its panel and focuses the first link"],
        &["Enter / Space", "Toggles the panel (a click on a hover-opened trigger keeps it open once)"],
        &["Escape", "Closes the panel"],
        &["Tab", "Moves through the panel's links; leaving the nav closes it"],
    ],
    props: &[
        &["items", "Vec<NavigationMenuEntry>", "On NavigationMenu: top-level items in order, each a panel or a link"],
        &["label", "String", "On NavigationMenu: accessible name. Default Main"],
        &["open", "Option<Option<String>>", "On NavigationMenu: controlled open panel. Pair with on_open_change"],
        &["default_open", "Option<String>", "On NavigationMenu: initially open panel when uncontrolled"],
        &["open_delay_ms", "u32", "On NavigationMenu: rest time before a panel opens. Default 150"],
        &["close_delay_ms", "u32", "On NavigationMenu: delay before a panel closes after the pointer leaves. Default 250"],
        &["on_open_change", "Callback<Option<String>>", "On NavigationMenu: called with the open panel, or None"],
        &["value", "String", "On NavigationMenuItem and NavigationMenuLink: the key. Must match an entry"],
        &["label", "String", "On NavigationMenuItem: text of the trigger"],
        &["placement", "Placement", "On NavigationMenuItem: where the panel sits. Default below, start-aligned"],
        &["href", "String", "On NavigationMenuLink and NavigationMenuPanelLink: where the link goes"],
        &["current", "bool", "On NavigationMenuLink: marks the current page"],
        &["description", "Option<String>", "On NavigationMenuPanelLink: second line under the title"],
    ],
};
