//! `StatusDiamond`: the rotated-square status marker, filled or hollow.

use dioxus::prelude::*;

pub use oxide_headless::jewel::Jewel;

/// A small diamond for workflow stages, label colours and legends.
/// Decorative unless `title` is set, which also becomes its accessible name.
#[component]
pub fn StatusDiamond(
    /// Colour. None draws a hollow grey diamond (no status).
    jewel: Option<Jewel>,
    /// Side in px before rotation.
    #[props(default = 9)]
    size: u16,
    /// Outline only, in the jewel colour.
    #[props(default)]
    hollow: bool,
    /// Tooltip and accessible name, such as "In review".
    title: Option<String>,
) -> Element {
    let named = title.is_some();
    rsx! {
        span {
            "data-scope": "status-diamond",
            "data-jewel": jewel.map(Jewel::as_str),
            "data-hollow": hollow.then_some(""),
            style: "width: {size}px; height: {size}px;",
            role: named.then_some("img"),
            "aria-label": title.clone(),
            "aria-hidden": (!named).then_some("true"),
            title,
        }
    }
}
