//! `<SpaceIcon>`: a rounded jewel tile with a line icon. Identifies a team, workspace or space.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::jewel::Jewel;

/// A jewel tile with an icon. Decorative unless `title` is set, which also names it.
#[component]
pub fn SpaceIcon(
    /// The icon.
    icon: IconName,
    /// Tile colour. Default amethyst.
    #[prop(default = Jewel::Amethyst)]
    jewel: Jewel,
    /// Side in px.
    #[prop(default = 24)]
    size: u16,
    /// Tooltip and accessible name, such as "Research team".
    #[prop(optional, into)]
    title: Option<String>,
) -> impl IntoView {
    let named = title.is_some();
    let name = title.clone();
    let radius = (f32::from(size) * 0.3).round();
    let icon_size = (f32::from(size) * 0.55).round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a rounded fraction of a u16")]
    let icon_size = icon_size as u16;
    view! {
        <span
            data-scope="space-icon"
            data-jewel=jewel.as_str()
            style=format!("width: {size}px; height: {size}px; border-radius: {radius}px;")
            role=named.then_some("img")
            aria-label=name
            aria-hidden=(!named).then_some("true")
            title=title
        >
            <Icon name=icon size=icon_size />
        </span>
    }
}
