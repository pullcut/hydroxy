//! The close buttons that `<Dialog>` and `<Sheet>` both offer.

use leptos::prelude::*;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;

/// A text button that runs `on_click`. `variant` gives it the hydroxy button look.
pub(crate) fn close_button(
    scope: &'static str,
    variant: Option<ButtonVariant>,
    on_click: impl Fn() + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-scope=scope
            data-part="close"
            class=variant.map(|_| BUTTON_CLASS)
            data-variant=variant.map(ButtonVariant::as_str)
            on:click=move |_| on_click()
        >
            {children()}
        </button>
    }
}

/// The icon-only close button that runs `on_click`.
pub(crate) fn close_icon_button(scope: &'static str, on_click: impl Fn() + 'static) -> impl IntoView {
    view! {
        <button
            type="button"
            aria-label="Close"
            title="Close"
            class=ICON_BUTTON_CLASS
            data-variant="ghost"
            data-size="sm"
            data-scope=scope
            data-part="close-icon"
            on:click=move |_| on_click()
        >
            <Icon name=IconName::X size=14 />
        </button>
    }
}
