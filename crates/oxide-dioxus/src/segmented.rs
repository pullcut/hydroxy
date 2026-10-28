//! `Segmented` / `SegmentedItem`: a pill row of mutually exclusive options.
//! Same behaviour as `RadioGroup` (its machine and connector): roving focus,
//! arrow keys that select, controlled or uncontrolled, form participation.
//! Only the look differs.

use dioxus::prelude::*;
use oxide_headless::radio_group::{
    self as h, ChangeReason, RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioGroupState,
};
use oxide_statechart::Machine;

pub use oxide_headless::radio_group::RadioItem as SegmentedOption;

use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};

/// The `data-scope` value on every segmented part.
pub const SCOPE: &str = "segmented";

/// Item padding and type size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SegmentedSize {
    /// Compact, for toolbars.
    Sm,
    /// Default.
    #[default]
    Md,
}

impl SegmentedSize {
    /// The `data-size` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SegmentedSize::Sm => "sm",
            SegmentedSize::Md => "md",
        }
    }
}

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<RadioGroupState>,
    input: Memo<RadioGroupInput>,
    items: FocusMap,
    on_value_change: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(mut self, event: RadioGroupEvent) {
        let step = RadioGroupMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
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
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn Segmented(
    /// Accessible name for the group (`aria-label`), such as "Summary length".
    label: String,
    /// The options, in display order.
    items: ReadSignal<Vec<SegmentedOption>>,
    /// Controlled value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<String>,
    /// Disable the whole control.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Item padding and type size.
    #[props(default)]
    size: SegmentedSize,
    /// Form field name. When set, the selected value is submitted with the form.
    #[props(default)]
    name: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    on_value_change: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let input = use_memo(move || RadioGroupInput {
        items: items(),
        value: value(),
        default_value: default_value.clone(),
        disabled: disabled(),
        loop_focus: true,
    });
    let mut state = use_signal(|| RadioGroupMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = RadioGroupMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    use_context_provider(|| Ctx {
        id: CopyValue::new(oxide_headless::next_id(SCOPE)),
        state,
        input,
        items: FocusMap::new(),
        on_value_change,
    });

    let selected = state.read().value.clone();
    rsx! {
        div {
            role: "radiogroup",
            "aria-label": label,
            "aria-orientation": "horizontal",
            "aria-disabled": input.read().disabled.then_some("true"),
            "data-scope": SCOPE,
            "data-part": "root",
            "data-size": size.as_str(),
            {children}
            if let (Some(name), Some(value)) = (name, selected) {
                input { r#type: "hidden", name, value }
            }
        }
    }
}

/// One segment: a label and an optional leading icon.
#[component]
pub fn SegmentedItem(
    /// The option this segment selects.
    value: String,
    /// Visible label.
    label: String,
    /// Icon before the label.
    icon: Option<IconName>,
) -> Element {
    let ctx = try_use_context::<Ctx>().expect("SegmentedItem must be rendered inside Segmented");
    let a = h::item_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read(), &value);
    let (mounted, clicked, focused) = (value.clone(), value.clone(), value);
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: a.role,
            "aria-checked": a.aria_checked.to_string(),
            tabindex: a.tabindex,
            disabled: a.disabled,
            "data-scope": SCOPE,
            "data-part": "item",
            "data-state": a.data_state,
            onmounted: move |e| ctx.items.register(mounted.clone(), e.data()),
            onclick: move |_| ctx.send(RadioGroupEvent::Select(clicked.clone(), ChangeReason::Pointer)),
            onfocus: move |_| ctx.send(RadioGroupEvent::Focus(focused.clone())),
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            if let Some(name) = icon {
                Icon { name, size: 16 }
            }
            {label}
        }
    }
}
