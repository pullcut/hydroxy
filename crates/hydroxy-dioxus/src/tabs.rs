//! `Tabs` / `TabsList` / `TabsTrigger` / `TabsContent`.
//! Parts read the machine from context and render the connector's attributes.
//! Focus goes through `MountedData`, so it works off the web too.
//!
//! Parts panic if rendered outside their root `Tabs`.

use dioxus::prelude::*;
use hydroxy_headless::tabs::{self as h, TabsCommand, TabsEvent, TabsInput, TabsMachine, TabsState};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;

pub use hydroxy_headless::tabs::{Activation, ChangeReason, Orientation, TabsItem};

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<TabsState>,
    input: Memo<TabsInput>,
    orientation: Orientation,
    triggers: FocusMap,
    on_value_change: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(self, event: TabsEvent) {
        for command in drive::<TabsMachine>(self.state, self.input, event) {
            match command {
                TabsCommand::FocusTrigger(value) => {
                    self.triggers.focus(&value);
                }
                TabsCommand::ValueChange { value, .. } => {
                    if let Some(handler) = self.on_value_change {
                        handler.call(value);
                    }
                }
            }
        }
    }
}

#[allow(clippy::expect_used, reason = "rendering outside the parent component is a usage error")]
fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Tabs parts must be rendered inside Tabs")
}

/// Root. Owns the machine and provides it to the parts below.
#[component]
pub fn Tabs(
    /// The tabs, in display order.
    tabs: ReadSignal<Vec<TabsItem>>,
    /// Controlled value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<String>,
    /// Whether arrow keys also select. See [`Activation`].
    #[props(default)]
    activation: Activation,
    /// Layout axis; picks the arrow keys.
    #[props(default)]
    orientation: Orientation,
    /// Wrap focus from the last tab to the first and back.
    #[props(default = true)]
    loop_focus: bool,
    /// Called with the new value whenever the selection changes or is requested.
    on_value_change: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let input = use_memo(move || TabsInput {
        tabs: tabs(),
        value: value(),
        default_value: default_value.clone(),
        activation,
        loop_focus,
    });
    let mut state = use_signal(|| TabsMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = TabsMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let ctx = use_context_provider(|| Ctx {
        id: CopyValue::new(hydroxy_headless::next_id(h::SCOPE)),
        state,
        input,
        orientation,
        triggers: FocusMap::new(),
        on_value_change,
    });

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-orientation": ctx.orientation.as_str(),
            {children}
        }
    }
}

/// The `tablist` wrapping the triggers.
#[component]
pub fn TabsList(children: Element) -> Element {
    let ctx = use_ctx();
    rsx! {
        div {
            role: "tablist",
            "aria-orientation": ctx.orientation.as_str(),
            "data-scope": h::SCOPE,
            "data-part": "list",
            {children}
        }
    }
}

/// The button that selects the tab `value`.
#[component]
pub fn TabsTrigger(
    /// The tab this trigger selects.
    value: String,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::trigger_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read(), &value);
    let (mounted, clicked, focused) = (value.clone(), value.clone(), value);
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: a.role,
            "aria-selected": a.aria_selected.to_string(),
            "aria-controls": a.aria_controls,
            tabindex: a.tabindex,
            disabled: a.disabled,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            onmounted: move |e| {
                ctx.triggers.register(mounted.clone(), e.data());
            },
            onclick: move |_| ctx.send(TabsEvent::Select(clicked.clone(), ChangeReason::Pointer)),
            onfocus: move |_| ctx.send(TabsEvent::Focus(focused.clone())),
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string(), ctx.orientation) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            {children}
        }
    }
}

/// The panel shown while `value` is selected.
#[component]
pub fn TabsContent(
    /// The tab this panel belongs to.
    value: String,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), &value);
    rsx! {
        div {
            id: a.id,
            role: a.role,
            "aria-labelledby": a.aria_labelledby,
            tabindex: a.tabindex,
            hidden: a.hidden,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            {children}
        }
    }
}
