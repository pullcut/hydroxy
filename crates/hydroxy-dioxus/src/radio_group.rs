//! `RadioGroup` / `RadioGroupItem`.
//! Items read the machine from context and render the connector's attributes.
//! Focus goes through `MountedData`, so it works off the web too.
//!
//! Parts panic if rendered outside their root `RadioGroup`.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::radio_group::{
    self as h, ChangeReason, RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioGroupState,
};
use hydroxy_statechart::Machine;

pub use hydroxy_headless::radio_group::{Orientation, RadioItem};

use crate::drive::drive;
use crate::focus::FocusMap;

/// The machine, its input and the focus map, shared by [`RadioGroup`] and `Segmented`.
#[derive(Clone, Copy)]
pub(crate) struct Ctx {
    pub(crate) id: CopyValue<String>,
    pub(crate) state: Signal<RadioGroupState>,
    pub(crate) input: Memo<RadioGroupInput>,
    items: FocusMap,
    on_value_change: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(self, event: RadioGroupEvent) {
        for command in drive::<RadioGroupMachine>(self.state, self.input, event) {
            match command {
                RadioGroupCommand::FocusItem(value) => self.items.focus(&value),
                RadioGroupCommand::ValueChange { value, .. } => {
                    if let Some(handler) = self.on_value_change {
                        handler.call(value);
                    }
                }
            }
        }
    }

    /// Call from an item's `onmounted`.
    pub(crate) fn register(self, value: String, el: Rc<MountedData>) {
        self.items.register(value, el);
    }

    /// An item was clicked.
    pub(crate) fn click(self, value: String) {
        self.send(RadioGroupEvent::Select(value, ChangeReason::Pointer));
    }

    /// An item took focus.
    pub(crate) fn focus(self, value: String) {
        self.send(RadioGroupEvent::Focus(value));
    }

    /// A key went down on an item: arrow keys move and select, the rest passes through.
    pub(crate) fn keydown(self, e: &Event<KeyboardData>) {
        if let Some(event) = h::keydown(&e.key().to_string()) {
            e.prevent_default();
            self.send(event);
        }
    }

    /// The selected value, if any.
    pub(crate) fn selected(self) -> Option<String> {
        self.state.read().value.value.clone()
    }
}

/// Run the machine for a radio-like group and provide it to the items below. The
/// returned context is the one the items read.
#[allow(clippy::too_many_arguments, reason = "the props of the two components that share this")]
pub(crate) fn use_radio(
    scope: &'static str,
    items: ReadSignal<Vec<RadioItem>>,
    value: ReadSignal<Option<String>>,
    default_value: Option<String>,
    disabled: ReadSignal<bool>,
    loop_focus: bool,
    on_value_change: Option<EventHandler<String>>,
) -> Ctx {
    let input = use_memo(move || RadioGroupInput {
        items: items(),
        value: value(),
        default_value: default_value.clone(),
        disabled: disabled(),
        loop_focus,
    });
    let mut state = use_signal(|| RadioGroupMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = RadioGroupMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    use_context_provider(|| Ctx {
        id: CopyValue::new(hydroxy_headless::next_id(scope)),
        state,
        input,
        items: FocusMap::new(),
        on_value_change,
    })
}

#[allow(clippy::expect_used, reason = "rendering outside the parent component is a usage error")]
fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("RadioGroupItem must be rendered inside RadioGroup")
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn RadioGroup(
    /// The options, in display order.
    items: ReadSignal<Vec<RadioItem>>,
    /// Controlled value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<String>,
    /// Disable the whole group.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Layout axis, vertical by default. Arrow keys work on both axes either way.
    #[props(default = Orientation::Vertical)]
    orientation: Orientation,
    /// Wrap focus from the last option to the first and back.
    #[props(default = true)]
    loop_focus: bool,
    /// Form field name. When set, the selected value is submitted with the form.
    #[props(default)]
    name: Option<String>,
    /// Accessible name for the group (`aria-label`).
    #[props(default)]
    label: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    on_value_change: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let ctx = use_radio(h::SCOPE, items, value, default_value, disabled, loop_focus, on_value_change);
    let selected = ctx.selected();
    rsx! {
        div {
            role: "radiogroup",
            "aria-label": label,
            "aria-orientation": orientation.as_str(),
            "aria-disabled": ctx.input.read().disabled.then_some("true"),
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-orientation": orientation.as_str(),
            {children}
            if let (Some(name), Some(value)) = (name, selected) {
                input { r#type: "hidden", name, value }
            }
        }
    }
}

/// One option: the radio circle, its label and an optional description.
#[component]
pub fn RadioGroupItem(
    /// The option this item selects.
    value: String,
    /// Visible label.
    label: String,
    /// Optional hint under the label.
    description: Option<String>,
) -> Element {
    let ctx = use_ctx();
    let root = ctx.id.read().clone();
    let a = h::item_attrs(&root, &ctx.state.read(), &ctx.input.read(), &value);
    let describedby = description.is_some().then_some(a.aria_describedby);
    let (mounted, clicked, focused) = (value.clone(), value.clone(), value.clone());
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: a.role,
            "aria-checked": a.aria_checked.to_string(),
            "aria-labelledby": a.aria_labelledby,
            "aria-describedby": describedby,
            tabindex: a.tabindex,
            disabled: a.disabled,
            "data-scope": h::SCOPE,
            "data-part": "item",
            "data-state": a.data_state,
            onmounted: move |e| ctx.register(mounted.clone(), e.data()),
            onclick: move |_| ctx.click(clicked.clone()),
            onfocus: move |_| ctx.focus(focused.clone()),
            onkeydown: move |e| ctx.keydown(&e),
            span { "data-scope": h::SCOPE, "data-part": "control", "aria-hidden": "true" }
            span { "data-scope": h::SCOPE, "data-part": "text",
                span { id: h::label_id(&root, &value), "data-scope": h::SCOPE, "data-part": "label", {label} }
                if let Some(text) = description {
                    span {
                        id: h::description_id(&root, &value),
                        "data-scope": h::SCOPE,
                        "data-part": "description",
                        {text}
                    }
                }
            }
        }
    }
}
