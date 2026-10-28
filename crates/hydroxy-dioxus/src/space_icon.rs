//! `SpaceIcon`: a rounded jewel tile with a line icon. Identifies a team, workspace or space.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::jewel::Jewel;

/// A jewel tile with an icon. Decorative unless `title` is set, which also names it.
#[component]
pub fn SpaceIcon(
    /// The icon.
    icon: IconName,
    /// Tile colour. Default amethyst.
    #[props(default = Jewel::Amethyst)]
    jewel: Jewel,
    /// Side in px.
    #[props(default = 24)]
    size: u16,
    /// Tooltip and accessible name, such as "Research team".
    title: Option<String>,
) -> Element {
    let named = title.is_some();
    let radius = (f32::from(size) * 0.3).round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a rounded fraction of a u16")]
    let icon_size = (f32::from(size) * 0.55).round() as u16;
    rsx! {
        span {
            "data-scope": "space-icon",
            "data-jewel": jewel.as_str(),
            style: "width: {size}px; height: {size}px; border-radius: {radius}px;",
            role: named.then_some("img"),
            "aria-label": title.clone(),
            "aria-hidden": (!named).then_some("true"),
            title,
            Icon { name: icon, size: icon_size }
        }
    }
}
