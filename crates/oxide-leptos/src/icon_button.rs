//! `<IconButton>`: a square, icon-only button for toolbars and compact actions.

use leptos::prelude::*;

use crate::button::{ButtonSize, ButtonVariant};
use crate::icon::{Icon, IconName};

pub use oxide_headless::button::ICON_BUTTON_CLASS;

/// An icon-only button. `label` is required: it is the accessible name and the hover title.
#[component]
pub fn IconButton(
    /// The icon.
    icon: IconName,
    /// What the button does, e.g. "Copy link". Becomes `aria-label` and `title`.
    #[prop(into)]
    label: String,
    /// Default ghost. Primary, accent, secondary and ghost are designed for icon buttons.
    #[prop(default = ButtonVariant::Ghost)]
    variant: ButtonVariant,
    /// 32, 40 or 48px square. Default medium.
    #[prop(optional)]
    size: ButtonSize,
    /// Greyed out and not clickable.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called on click.
    #[prop(optional, into)]
    on_click: Option<Callback<leptos::ev::MouseEvent>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=ICON_BUTTON_CLASS
            aria-label=label.clone()
            title=label
            data-variant=variant.as_str()
            data-size=size.as_str()
            disabled=disabled
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.run(ev);
                }
            }
        >
            <Icon name=icon size=size.icon_button_px() />
        </button>
    }
}
