//! `<Collapsible>` / `<CollapsibleTrigger>` / `<CollapsibleContent>`.

use hydroxy_headless::collapsible::{
    self as h, CollapsibleCommand, CollapsibleEvent, CollapsibleInput, CollapsibleMachine, CollapsibleState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<CollapsibleState>,
    input: Memo<CollapsibleInput>,
    on_open_change: Option<Callback<bool>>,
}

impl Ctx {
    fn send(self, event: CollapsibleEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        for command in drive::<CollapsibleMachine, _>(self.state, self.input, event) {
            match command {
                CollapsibleCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Collapsible parts must be rendered inside <Collapsible>")
}

/// Root. Owns the machine and provides it to the parts below.
#[component]
pub fn Collapsible(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Ignore toggles; the trigger is disabled.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| CollapsibleInput { open: open.get(), default_open, disabled: disabled.get() });
    let state = RwSignal::new(input.with_untracked(CollapsibleMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CollapsibleMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    provide_context(Ctx { id: StoredValue::new(crate::id::next_id(h::SCOPE)), state, input, on_open_change });

    view! {
        <div data-scope=h::SCOPE data-part="root" data-state=move || h::data_state(state.get().open.value)>
            {children()}
        </div>
    }
}

/// The button that opens and closes the content.
#[component]
pub fn CollapsibleTrigger(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s, i))));
    view! {
        <button
            type="button"
            id=move || attrs.with(|x| x.id.clone())
            aria-expanded=move || attrs.with(|x| x.aria_expanded).to_string()
            aria-controls=move || attrs.with(|x| x.aria_controls.clone())
            disabled=move || attrs.with(|x| x.disabled)
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.with(|x| x.data_state)
            on:click=move |_| ctx.send(CollapsibleEvent::Toggle)
        >
            {children()}
        </button>
    }
}

/// The region shown while open. Stays mounted, hidden when closed.
#[component]
pub fn CollapsibleContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s)));
    view! {
        <div
            id=move || attrs.with(|x| x.id.clone())
            role="region"
            aria-labelledby=move || attrs.with(|x| x.aria_labelledby.clone())
            hidden=move || attrs.with(|x| x.hidden)
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.with(|x| x.data_state)
        >
            {children()}
        </div>
    }
}
