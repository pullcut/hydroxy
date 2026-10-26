//! `IconButton`: a square, icon-only button for toolbars and compact actions.

use dioxus::prelude::*;

use crate::button::{ButtonSize, ButtonVariant};
use crate::icon::{Icon, IconName};

pub use oxide_headless::button::ICON_BUTTON_CLASS;

/// An icon-only button. `label` is required: it is the accessible name and the hover title.
#[component]
pub fn IconButton(
    /// The icon.
    icon: IconName,
    /// What the button does, e.g. "Copy link". Becomes `aria-label` and `title`.
    label: String,
    /// Default ghost. Primary, accent, secondary and ghost are designed for icon buttons.
    #[props(default = ButtonVariant::Ghost)]
    variant: ButtonVariant,
    /// 32, 40 or 48px square. Default medium.
    #[props(default)]
    size: ButtonSize,
    /// Greyed out and not clickable.
    #[props(default)]
    disabled: bool,
    /// Called on click.
    on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: ICON_BUTTON_CLASS,
            "aria-label": "{label}",
            title: "{label}",
            "data-variant": variant.as_str(),
            "data-size": size.as_str(),
            disabled,
            onclick: move |e| {
                if let Some(handler) = on_click {
                    handler.call(e);
                }
            },
            Icon { name: icon, size: size.icon_button_px() }
        }
    }
}
