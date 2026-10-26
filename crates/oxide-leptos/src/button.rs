//! `<Button>`: the primary action control.
//!
//! Styled by the `ox-button` class with `data-variant` and `data-size`. Trigger
//! parts in other components (dialog, menu, tooltip) reuse the same hook.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

pub use oxide_headless::button::{BUTTON_CLASS, ButtonSize, ButtonType, ButtonVariant};

/// A button. Say what happens: "Share summary", not "OK".
#[component]
pub fn Button(
    /// See [`ButtonVariant`]. Default primary.
    #[prop(optional)]
    variant: ButtonVariant,
    /// See [`ButtonSize`]. Default medium.
    #[prop(optional)]
    size: ButtonSize,
    /// Icon before the label.
    #[prop(optional)]
    icon_left: Option<IconName>,
    /// Icon after the label.
    #[prop(optional)]
    icon_right: Option<IconName>,
    /// Stretch to the container's width.
    #[prop(optional)]
    full_width: bool,
    /// Greyed out and not clickable.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// The `type` attribute. Default `button`.
    #[prop(optional)]
    button_type: ButtonType,
    /// Called on click.
    #[prop(optional, into)]
    on_click: Option<Callback<leptos::ev::MouseEvent>>,
    /// The label.
    children: Children,
) -> impl IntoView {
    let icon = size.icon_px();
    view! {
        <button
            type=button_type.as_str()
            class=BUTTON_CLASS
            data-variant=variant.as_str()
            data-size=size.as_str()
            data-full-width=full_width.then_some("")
            disabled=disabled
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            {icon_left.map(|name| view! { <Icon name size=icon /> })}
            {children()}
            {icon_right.map(|name| view! { <Icon name size=icon /> })}
        </button>
    }
}
