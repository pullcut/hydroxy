use dioxus::prelude::*;
use hydroxy_dioxus::theme_menu::{Theme, ThemeMenu};

#[component]
pub fn ThemeMenuDemo() -> Element {
    let mut choice = use_signal(|| String::from("not picked yet"));

    rsx! {
        div { style: "display: flex; align-items: center; gap: var(--space-5);",
            // Sets data-theme on the page and saves it. Try it: this page changes with it.
            ThemeMenu { on_change: move |theme: Theme| choice.set(theme.label().to_string()) }
            span { "Picked: {choice}" }
        }
    }
}
