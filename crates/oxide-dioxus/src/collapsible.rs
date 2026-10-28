//! `Collapsible` / `CollapsibleTrigger` / `CollapsibleContent`.

use dioxus::prelude::*;
use oxide_headless::collapsible::{
    self as h, CollapsibleCommand, CollapsibleEvent, CollapsibleInput, CollapsibleMachine, CollapsibleState,
};
use oxide_statechart::Machine;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<CollapsibleState>,
    input: Memo<CollapsibleInput>,
    on_open_change: Option<EventHandler<bool>>,
}

impl Ctx {
    fn send(mut self, event: CollapsibleEvent) {
        let step = CollapsibleMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                CollapsibleCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Collapsible parts must be rendered inside Collapsible")
}

/// Root. Owns the machine and provides it to the parts below.
#[component]
pub fn Collapsible(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Ignore toggles; the trigger is disabled.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let input = use_memo(move || CollapsibleInput { open: open(), default_open, disabled: disabled() });
    let mut state = use_signal(|| CollapsibleMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CollapsibleMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    use_context_provider(|| Ctx {
        id: CopyValue::new(oxide_headless::next_id(h::SCOPE)),
        state,
        input,
        on_open_change,
    });

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": h::data_state(state.read().open),
            {children}
        }
    }
}

/// The button that opens and closes the content.
#[component]
pub fn CollapsibleTrigger(children: Element) -> Element {
    let ctx = use_ctx();
    let a = h::trigger_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read());
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            "aria-expanded": a.aria_expanded.to_string(),
            "aria-controls": a.aria_controls,
            disabled: a.disabled,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            onclick: move |_| ctx.send(CollapsibleEvent::Toggle),
            {children}
        }
    }
}

/// The region shown while open. Stays mounted, hidden when closed.
#[component]
pub fn CollapsibleContent(children: Element) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read());
    rsx! {
        div {
            id: a.id,
            role: a.role,
            "aria-labelledby": a.aria_labelledby,
            hidden: a.hidden,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            {children}
        }
    }
}
