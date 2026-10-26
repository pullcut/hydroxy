//! `Icon`: a line icon from the oxide set, sized in px, coloured by `currentColor`.

use dioxus::prelude::*;
pub use oxide_headless::icons::Icon as IconName;

/// A decorative line icon. Pair with visible text, or give the parent an `aria-label`.
#[component]
pub fn Icon(
    /// Which icon.
    name: IconName,
    /// Width and height in px.
    #[props(default = 16)]
    size: u16,
) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            width: "{size}",
            height: "{size}",
            dangerous_inner_html: name.markup(),
        }
    }
}
