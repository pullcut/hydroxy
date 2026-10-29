//! `<Sheet>` / `<SheetTrigger>` / `<SheetContent>` / `<SheetHeader>` /
//! `<SheetTitle>` / `<SheetDescription>` / `<SheetBody>` / `<SheetFooter>` /
//! `<SheetClose>` / `<SheetCloseIcon>`.
//! A side drawer: a native `<dialog>` opened with `showModal()`, its panel
//! pinned to one edge of the viewport.

use hydroxy_headless::sheet::{self as h, CloseReason, SheetCommand, SheetEvent, SheetInput, SheetMachine, SheetState};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::dialog::actions::{close_button, close_icon_button};
use crate::drive::drive;
use crate::overlay::{close_dialog, show_modal};

pub use hydroxy_headless::sheet::SheetSide;

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<SheetState>,
    input: Memo<SheetInput>,
    on_open_change: Option<Callback<bool>>,
}

impl Ctx {
    fn send(self, event: SheetEvent) {
        // Events can still arrive after unmount (a cancel on a removed dialog).
        for command in drive::<SheetMachine, _>(self.state, self.input, event) {
            match command {
                SheetCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Sheet parts must be rendered inside <Sheet>")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Sheet(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| SheetInput { open: open.get(), default_open, alert: false });
    let state = RwSignal::new(input.with_untracked(SheetMachine::initial));
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| SheetMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    // Keep the native dialog in step with the machine.
    Effect::new(move |_| {
        let content = h::content_id(&id.get_value());
        if state.with(|s| s.open.value) {
            show_modal(&content);
        } else {
            close_dialog(&content);
        }
    });

    provide_context(Ctx { id, state, input, on_open_change });
    children()
}

/// The button that opens the sheet. Set `variant` for the hydroxy button look.
#[component]
pub fn SheetTrigger(
    /// Gives the trigger the [`Button`](crate::button::Button) look in this variant.
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s)));
    view! {
        <button
            type="button"
            aria-haspopup="dialog"
            aria-expanded=move || attrs.with(|x| x.aria_expanded).to_string()
            aria-controls=move || attrs.with(|x| x.aria_controls.clone())
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.with(|x| x.data_state)
            class=variant.map(|_| BUTTON_CLASS)
            data-variant=variant.map(ButtonVariant::as_str)
            on:click=move |_| ctx.send(SheetEvent::Open)
        >
            {children()}
        </button>
    }
}

/// The modal drawer: a full-viewport native `<dialog>` with the panel against
/// the `side` edge. A press outside the panel or Esc closes it.
#[component]
pub fn SheetContent(
    /// The edge the panel slides in from. Default the inline end (right in LTR).
    #[prop(optional)]
    side: SheetSide,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s)));
    view! {
        <dialog
            id=move || attrs.with(|x| x.id.clone())
            role="dialog"
            aria-labelledby=move || attrs.with(|x| x.aria_labelledby.clone())
            aria-describedby=move || attrs.with(|x| x.aria_describedby.clone())
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.with(|x| x.data_state)
            data-side=side.as_str()
            data-overlay="true"
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
            // Other close requests (e.g. Android back): keep the machine in charge.
            on:cancel=move |ev: leptos::ev::Event| {
                ev.prevent_default();
                ctx.send(SheetEvent::Close(CloseReason::Escape));
            }
            on:pointerdown=move |ev| {
                // Stops the press focusing the closing <dialog>, which would undo the
                // browser's focus return to the trigger.
                ev.prevent_default();
                ctx.send(SheetEvent::Close(CloseReason::Outside));
            }
        >
            <div data-scope=h::SCOPE data-part="panel" on:pointerdown=|ev| ev.stop_propagation()>
                {children()}
            </div>
        </dialog>
    }
}

/// The top row: title, description and usually a [`SheetCloseIcon`].
#[component]
pub fn SheetHeader(children: Children) -> impl IntoView {
    view! { <header data-scope=h::SCOPE data-part="header">{children()}</header> }
}

/// The sheet's heading. Labels the dialog.
#[component]
pub fn SheetTitle(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <h2 id=h::title_id(&ctx.id.get_value()) data-scope=h::SCOPE data-part="title">
            {children()}
        </h2>
    }
}

/// Supporting text under the title. Describes the dialog.
#[component]
pub fn SheetDescription(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <p id=h::description_id(&ctx.id.get_value()) data-scope=h::SCOPE data-part="description">
            {children()}
        </p>
    }
}

/// The main content. Scrolls on its own when it overflows.
#[component]
pub fn SheetBody(children: Children) -> impl IntoView {
    view! { <div data-scope=h::SCOPE data-part="body">{children()}</div> }
}

/// The bottom row of buttons, pinned below the body.
#[component]
pub fn SheetFooter(children: Children) -> impl IntoView {
    view! { <footer data-scope=h::SCOPE data-part="footer">{children()}</footer> }
}

/// A button that closes the sheet. Set `variant` for the hydroxy button look.
#[component]
pub fn SheetClose(
    /// Gives the button the [`Button`](crate::button::Button) look in this variant.
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    close_button(h::SCOPE, variant, move || ctx.send(SheetEvent::Close(CloseReason::CloseTrigger)), children)
}

/// The icon-only close button. Place it in the [`SheetHeader`].
#[component]
pub fn SheetCloseIcon() -> impl IntoView {
    let ctx = use_ctx();
    close_icon_button(h::SCOPE, move || ctx.send(SheetEvent::Close(CloseReason::CloseTrigger)))
}
