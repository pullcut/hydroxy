//! `Button`: the primary action control.
//!
//! Styled by the `ox-button` class with `data-variant` and `data-size`. Trigger
//! parts in other components (dialog, menu, tooltip) reuse the same hook.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

pub use oxide_headless::button::{BUTTON_CLASS, ButtonSize, ButtonType, ButtonVariant};

/// A button. Say what happens: "Share summary", not "OK".
#[component]
pub fn Button(
    /// See [`ButtonVariant`]. Default primary.
    #[props(default)]
    variant: ButtonVariant,
    /// See [`ButtonSize`]. Default medium.
    #[props(default)]
    size: ButtonSize,
    /// Icon before the label.
    icon_left: Option<IconName>,
    /// Icon after the label.
    icon_right: Option<IconName>,
    /// Stretch to the container's width.
    #[props(default)]
    full_width: bool,
    /// Greyed out and not clickable.
    #[props(default)]
    disabled: bool,
    /// The `type` attribute. Default `button`.
    #[props(default)]
    button_type: ButtonType,
    /// Called on click.
    on_click: Option<EventHandler<MouseEvent>>,
    /// The label.
    children: Element,
) -> Element {
    let icon = size.icon_px();
    rsx! {
        button {
            r#type: button_type.as_str(),
            class: BUTTON_CLASS,
            "data-variant": variant.as_str(),
            "data-size": size.as_str(),
            "data-full-width": full_width.then_some(""),
            disabled,
            onclick: move |e| {
                if let Some(handler) = on_click {
                    handler.call(e);
                }
            },
            if let Some(name) = icon_left {
                Icon { name, size: icon }
            }
            {children}
            if let Some(name) = icon_right {
                Icon { name, size: icon }
            }
        }
    }
}
