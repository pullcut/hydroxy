//! `<StatusDiamond>`: the rotated-square status marker, filled or hollow.

use leptos::prelude::*;

pub use hydroxy_headless::jewel::Jewel;

/// A small diamond for workflow stages, label colours and legends.
/// Decorative unless `title` is set, which also becomes its accessible name.
#[component]
pub fn StatusDiamond(
    /// Colour. None draws a hollow grey diamond (no status).
    #[prop(into, default = None)]
    jewel: Option<Jewel>,
    /// Side in px before rotation.
    #[prop(default = 9)]
    size: u16,
    /// Outline only, in the jewel colour.
    #[prop(optional)]
    hollow: bool,
    /// Tooltip and accessible name, such as "In review".
    #[prop(optional, into)]
    title: Option<String>,
) -> impl IntoView {
    let named = title.is_some();
    view! {
        <span
            data-scope="status-diamond"
            data-jewel=jewel.map(Jewel::as_str)
            data-hollow=hollow.then_some("")
            style=format!("width: {size}px; height: {size}px;")
            role=named.then_some("img")
            aria-label=title.clone()
            aria-hidden=(!named).then_some("true")
            title=title
        ></span>
    }
}
