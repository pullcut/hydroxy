//! `Toaster` and [`use_toasts`]. Wrap the app (or a page) in `Toaster`, then
//! call `use_toasts().push(...)` from anywhere inside it.
//!
//! The region is a `popover="manual"` shown from mount, so it sits in the top
//! layer. Lifetimes are CSS animations: no timers run in Rust.
//!
//! `use_toasts` panics if called outside `Toaster`.

use dioxus::prelude::*;
use hydroxy_headless::toast::{self as h, ToastEvent, ToastInput, ToastMachine, ToastState, ToastTone};
use hydroxy_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonSize, ButtonVariant};
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::overlay::show_popover;

pub use hydroxy_headless::toast::{DEFAULT_DURATION_MS, ToastOptions, ToastTone as Tone};

/// Handle to the nearest `Toaster`. Cheap to copy.
#[derive(Clone, Copy)]
pub struct Toasts {
    state: Signal<ToastState>,
    max_visible: usize,
}

impl Toasts {
    fn send(mut self, event: ToastEvent) {
        let input = ToastInput { max_visible: self.max_visible };
        let next = ToastMachine::transition(&self.state.peek(), event, &input).state;
        if let Some(next) = next {
            self.state.set(next);
        }
    }

    /// Show a toast. Returns its id, for [`Toasts::dismiss`] and `on_action`.
    #[allow(clippy::must_use_candidate, reason = "most callers fire and forget")]
    pub fn push(self, options: ToastOptions) -> u64 {
        let id = self.state.peek().next_id;
        self.send(ToastEvent::Push(options));
        id
    }

    /// Start a toast's exit.
    pub fn dismiss(self, id: u64) {
        self.send(ToastEvent::Dismiss(id));
    }
}

/// The handle from the nearest `Toaster`.
///
/// # Panics
/// When called outside a `Toaster`.
#[must_use]
pub fn use_toasts() -> Toasts {
    try_use_context::<Toasts>().expect("use_toasts must be called inside Toaster")
}

/// Provides [`use_toasts`] to its children and renders the notification region.
#[component]
pub fn Toaster(
    /// How many toasts show at once. The rest wait their turn.
    #[props(default = 3)]
    max_visible: usize,
    /// Called with the toast's id when its action button is clicked. The toast then closes.
    on_action: Option<EventHandler<u64>>,
    children: Element,
) -> Element {
    let toasts = use_context_provider(|| Toasts { state: Signal::new(ToastState::default()), max_visible });
    let region = use_hook(|| CopyValue::new(hydroxy_headless::next_id(h::SCOPE)));
    // shown once at mount, so a modal dialog opened later stacks above the
    // toasts; re-showing per push would restart every toast's lifetime animation.
    use_effect(move || show_popover(&region.peek()));

    let visible = toasts.state.read().visible(&ToastInput { max_visible }).to_vec();
    rsx! {
        {children}
        section {
            id: region.cloned(),
            "popover": "manual",
            "aria-live": "polite",
            "aria-label": "Notifications",
            "data-scope": h::SCOPE,
            "data-part": "region",
            for toast in visible {
                {
                    let id = toast.id;
                    let attrs = h::toast_attrs(&toast);
                    let leaving = toast.leaving;
                    let icon = match toast.options.tone {
                        ToastTone::Neutral => IconName::Info,
                        ToastTone::Success => IconName::Check,
                        ToastTone::Danger => IconName::CircleAlert,
                    };
                    let ToastOptions { title, description, action, .. } = toast.options;
                    rsx! {
                        div {
                            key: "{id}",
                            role: attrs.role,
                            "data-scope": h::SCOPE,
                            "data-part": "toast",
                            "data-tone": attrs.data_tone,
                            "data-state": attrs.data_state,
                            onanimationend: move |_| {
                                // The exit animation finished: drop the toast.
                                if leaving {
                                    toasts.send(ToastEvent::Remove(id));
                                }
                            },
                            span { "data-scope": h::SCOPE, "data-part": "icon", Icon { name: icon, size: 16 } }
                            div { "data-scope": h::SCOPE, "data-part": "body",
                                p { "data-scope": h::SCOPE, "data-part": "title", "{title}" }
                                if let Some(d) = description {
                                    p { "data-scope": h::SCOPE, "data-part": "description", "{d}" }
                                }
                            }
                            if let Some(label) = action {
                                button {
                                    r#type: "button",
                                    class: BUTTON_CLASS,
                                    "data-variant": ButtonVariant::Secondary.as_str(),
                                    "data-size": ButtonSize::Sm.as_str(),
                                    "data-scope": h::SCOPE,
                                    "data-part": "action",
                                    onclick: move |_| {
                                        if let Some(handler) = on_action {
                                            handler.call(id);
                                        }
                                        toasts.dismiss(id);
                                    },
                                    "{label}"
                                }
                            }
                            button {
                                r#type: "button",
                                "aria-label": "Dismiss",
                                title: "Dismiss",
                                class: ICON_BUTTON_CLASS,
                                "data-variant": ButtonVariant::Ghost.as_str(),
                                "data-size": ButtonSize::Sm.as_str(),
                                "data-scope": h::SCOPE,
                                "data-part": "close",
                                onclick: move |_| toasts.dismiss(id),
                                Icon { name: IconName::X, size: 14 }
                            }
                            if let Some(style) = attrs.timer_style {
                                div {
                                    "aria-hidden": "true",
                                    style,
                                    "data-scope": h::SCOPE,
                                    "data-part": "timer",
                                    onanimationend: move |e| {
                                        // Lifetime over. Stop here so the toast's own handler only sees its exit.
                                        e.stop_propagation();
                                        toasts.dismiss(id);
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
