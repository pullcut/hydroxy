//! `Accordion` / `AccordionItem` / `AccordionTrigger` / `AccordionContent`.
//! Parts read the machine from context and render the connector's attributes.
//!
//! Parts panic if rendered outside their root `Accordion`; `AccordionTrigger` and
//! `AccordionContent` also need an `AccordionItem`.

use dioxus::prelude::*;
use hydroxy_headless::accordion::{
    self as h, AccordionCommand, AccordionEvent, AccordionInput, AccordionMachine, AccordionState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};

pub use hydroxy_headless::accordion::AccordionEntry;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<AccordionState>,
    input: Memo<AccordionInput>,
    triggers: FocusMap,
    on_value_change: Option<EventHandler<Vec<String>>>,
}

impl Ctx {
    fn send(self, event: AccordionEvent) {
        let commands = drive::<AccordionMachine>(self.state, self.input, event);
        for command in commands {
            match command {
                AccordionCommand::FocusTrigger(value) => self.triggers.focus(&value),
                AccordionCommand::ValueChange(value) => {
                    if let Some(handler) = self.on_value_change {
                        handler.call(value);
                    }
                }
            }
        }
    }
}

/// The item value, provided by `AccordionItem` to its trigger and content.
#[derive(Clone)]
struct ItemValue(String);

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Accordion parts must be rendered inside Accordion")
}

fn use_item() -> String {
    try_use_context::<ItemValue>().expect("AccordionTrigger and AccordionContent must be inside AccordionItem").0
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn Accordion(
    /// The items, in display order.
    items: ReadSignal<Vec<AccordionEntry>>,
    /// Controlled open items. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<Vec<String>>>,
    /// Initially open items when uncontrolled.
    #[props(default)]
    default_value: Vec<String>,
    /// Allow several items open at once.
    #[props(default)]
    multiple: bool,
    /// In single mode, whether the open item can be closed again.
    #[props(default = true)]
    collapsible: bool,
    /// Called with the open items whenever they change or a change is requested.
    on_value_change: Option<EventHandler<Vec<String>>>,
    children: Element,
) -> Element {
    let input = use_memo(move || AccordionInput {
        items: items(),
        multiple,
        value: value(),
        default_value: default_value.clone(),
        collapsible,
    });
    let mut state = use_signal(|| AccordionMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = AccordionMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    use_context_provider(|| Ctx {
        id: CopyValue::new(hydroxy_headless::next_id(h::SCOPE)),
        state,
        input,
        triggers: FocusMap::new(),
        on_value_change,
    });

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-orientation": "vertical", {children} }
    }
}

/// One item. Wraps its trigger and content.
#[component]
pub fn AccordionItem(
    /// The item this wraps. Must match an entry in `items`.
    value: String,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    use_context_provider(|| ItemValue(value.clone()));
    let a = h::item_attrs(&ctx.state.read(), &ctx.input.read(), &value);
    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "item",
            "data-state": a.data_state,
            "data-disabled": a.disabled.then_some(""),
            {children}
        }
    }
}

/// The heading button that opens and closes its item.
#[component]
pub fn AccordionTrigger(children: Element) -> Element {
    let ctx = use_ctx();
    let value = use_item();
    let a = h::trigger_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read(), &value);
    let (mounted, clicked, focused) = (value.clone(), value.clone(), value);
    rsx! {
        h3 { "data-scope": h::SCOPE, "data-part": "heading",
            button {
                r#type: "button",
                id: a.id,
                "aria-expanded": a.aria_expanded.to_string(),
                "aria-controls": a.aria_controls,
                disabled: a.disabled,
                "data-scope": h::SCOPE,
                "data-part": "trigger",
                "data-state": a.data_state,
                "data-disabled": a.disabled.then_some(""),
                onmounted: move |e| ctx.triggers.register(mounted.clone(), e.data()),
                onclick: move |_| ctx.send(AccordionEvent::Toggle(clicked.clone())),
                onfocus: move |_| ctx.send(AccordionEvent::Focus(focused.clone())),
                onkeydown: move |e| {
                    if let Some(event) = h::keydown(&e.key().to_string()) {
                        e.prevent_default();
                        ctx.send(event);
                    }
                },
                span { {children} }
                Icon { name: IconName::ChevronDown, size: 16 }
            }
        }
    }
}

/// The region shown while its item is open.
#[component]
pub fn AccordionContent(children: Element) -> Element {
    let ctx = use_ctx();
    let value = use_item();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), &value);
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
