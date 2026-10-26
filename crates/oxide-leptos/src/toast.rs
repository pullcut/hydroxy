//! `<Toaster>` and [`use_toasts`]. Wrap the app (or a page) in `<Toaster>`, then
//! call `use_toasts().push(...)` from anywhere inside it.
//!
//! The region is a `popover="manual"` shown from mount, so it sits in the top
//! layer. Lifetimes are CSS animations: no timers run in Rust.

use leptos::prelude::*;
use oxide_headless::toast::{self as h, Toast, ToastEvent, ToastInput, ToastMachine, ToastState, ToastTone};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonSize, ButtonVariant};
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::overlay::show_popover;

pub use oxide_headless::toast::{DEFAULT_DURATION_MS, ToastOptions, ToastTone as Tone};

/// Handle to the nearest `<Toaster>`. Cheap to copy.
#[derive(Clone, Copy)]
pub struct Toasts {
    state: RwSignal<ToastState>,
    max_visible: usize,
}

impl Toasts {
    fn send(self, event: ToastEvent) {
        let input = ToastInput { max_visible: self.max_visible };
        // Events can still arrive after unmount (an animation ending on a removed toast).
        if let Some(step) = self.state.try_with_untracked(|s| ToastMachine::transition(s, event, &input)) {
            self.state.set(step.state);
        }
    }

    /// Show a toast. Returns its id, for [`Toasts::dismiss`] and `on_action`.
    pub fn push(self, options: ToastOptions) -> u64 {
        let id = self.state.with_untracked(|s| s.next_id);
        self.send(ToastEvent::Push(options));
        id
    }

    /// Start a toast's exit.
    pub fn dismiss(self, id: u64) {
        self.send(ToastEvent::Dismiss(id));
    }
}

/// The handle from the nearest `<Toaster>`.
///
/// # Panics
/// When called outside a `<Toaster>`.
#[must_use]
pub fn use_toasts() -> Toasts {
    use_context::<Toasts>().expect("use_toasts must be called inside <Toaster>")
}

/// Provides [`use_toasts`] to its children and renders the notification region.
#[component]
pub fn Toaster(
    /// How many toasts show at once. The rest wait their turn.
    #[prop(default = 3)]
    max_visible: usize,
    /// Called with the toast's id when its action button is clicked. The toast then closes.
    #[prop(optional, into)]
    on_action: Option<Callback<u64>>,
    children: Children,
) -> impl IntoView {
    let toasts = Toasts { state: RwSignal::new(ToastState::default()), max_visible };
    provide_context(toasts);

    let region = StoredValue::new(oxide_headless::next_id(h::SCOPE));
    // shown once at mount, so a modal dialog opened later stacks above the
    // toasts; re-showing per push would restart every toast's lifetime animation.
    Effect::new(move |_| show_popover(&region.get_value()));

    let visible = move || toasts.state.with(|s| s.visible(&ToastInput { max_visible }).to_vec());
    view! {
        {children()}
        <section
            id=region.get_value()
            popover="manual"
            aria-live="polite"
            aria-label="Notifications"
            data-scope=h::SCOPE
            data-part="region"
        >
            <For each=visible key=|t| t.id children=move |t| toast_view(t, toasts, on_action) />
        </section>
    }
}

fn toast_view(toast: Toast, toasts: Toasts, on_action: Option<Callback<u64>>) -> impl IntoView {
    let id = toast.id;
    let attrs = h::toast_attrs(&toast);
    let leaving = move || toasts.state.with(|s| s.toasts.iter().any(|t| t.id == id && t.leaving));
    let icon = match toast.options.tone {
        ToastTone::Neutral => IconName::Info,
        ToastTone::Success => IconName::Check,
        ToastTone::Danger => IconName::CircleAlert,
    };
    let ToastOptions { title, description, action, .. } = toast.options;
    view! {
        <div
            role=attrs.role
            data-scope=h::SCOPE
            data-part="toast"
            data-tone=attrs.data_tone
            data-state=move || if leaving() { "leaving" } else { "open" }
            on:animationend=move |_| {
                // The exit animation finished: drop the toast.
                if leaving() {
                    toasts.send(ToastEvent::Remove(id));
                }
            }
        >
            <span data-scope=h::SCOPE data-part="icon">
                <Icon name=icon size=16 />
            </span>
            <div data-scope=h::SCOPE data-part="body">
                <p data-scope=h::SCOPE data-part="title">{title}</p>
                {description.map(|d| view! { <p data-scope=h::SCOPE data-part="description">{d}</p> })}
            </div>
            {action
                .map(|label| {
                    view! {
                        <button
                            type="button"
                            class=BUTTON_CLASS
                            data-variant=ButtonVariant::Secondary.as_str()
                            data-size=ButtonSize::Sm.as_str()
                            data-scope=h::SCOPE
                            data-part="action"
                            on:click=move |_| {
                                if let Some(cb) = on_action {
                                    cb.run(id);
                                }
                                toasts.dismiss(id);
                            }
                        >
                            {label}
                        </button>
                    }
                })}
            <button
                type="button"
                aria-label="Dismiss"
                title="Dismiss"
                class=ICON_BUTTON_CLASS
                data-variant=ButtonVariant::Ghost.as_str()
                data-size=ButtonSize::Sm.as_str()
                data-scope=h::SCOPE
                data-part="close"
                on:click=move |_| toasts.dismiss(id)
            >
                <Icon name=IconName::X size=14 />
            </button>
            {attrs
                .timer_style
                .map(|style| {
                    view! {
                        <div
                            aria-hidden="true"
                            style=style
                            data-scope=h::SCOPE
                            data-part="timer"
                            on:animationend=move |ev| {
                                // Lifetime over. Stop here so the toast's own handler only sees its exit.
                                ev.stop_propagation();
                                toasts.dismiss(id);
                            }
                        ></div>
                    }
                })}
        </div>
    }
}
