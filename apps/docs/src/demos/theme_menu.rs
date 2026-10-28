use hydroxy_leptos::theme_menu::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (choice, set_choice) = signal(String::from("not picked yet"));

    view! {
        <div style="display: flex; align-items: center; gap: var(--space-5);">
            // Sets data-theme on the page and saves it. Try it: this site changes with it.
            <ThemeMenu on_change=move |theme: Theme| set_choice.set(theme.label().to_string()) />
            <span>"Picked: " {choice}</span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "theme-menu",
    name: "Theme menu",
    group: "Actions",
    summary: "An icon button with a menu of Light, Dark and System. It sets data-theme on the page, saves the choice in localStorage and follows the system setting while System is chosen.",
    demo,
    leptos: include_str!("theme_menu.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/theme_menu.rs"),
    parts: &[
        &["ThemeMenu", "trigger", "Icon button showing the current theme, aria-haspopup=menu"],
        &["", "content", "popover=auto, role=menu. Styled like Menu (data-scope=menu)"],
        &["", "item", "button, role=menuitemradio, aria-checked on the current theme"],
        &["", "check", "Check mark on the current item"],
    ],
    keys: &[
        &["Enter / Space / Down", "Opens the menu"],
        &["Down / Up", "Moves between Light, Dark and System, wrapping"],
        &["Home / End", "Moves to the first or last item"],
        &["Enter / Space", "Picks the focused theme and closes"],
        &["Escape", "Closes and returns focus to the button"],
    ],
    props: &[
        &["default_value", "Theme", "The choice before one is saved. Default System"],
        &["placement", "Placement", "Where the menu sits. Default below, end-aligned"],
        &["on_change", "Callback<Theme>", "Called with the new choice when one is picked"],
    ],
};
