//! `<Segmented>` / `<SegmentedItem>`: a pill row of mutually exclusive options.
//! Same behaviour as `RadioGroup` (its machine and connector): roving focus,
//! arrow keys that select, controlled or uncontrolled, form participation.
//! Only the look differs.

use leptos::prelude::*;
use oxide_headless::radio_group::{
    self as h, ChangeReason, RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioGroupState,
};
use oxide_statechart::Machine;

pub use oxide_headless::radio_group::RadioItem as SegmentedOption;

use crate::focus::focus_id;
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
    id: StoredValue<String>,
    state: RwSignal<RadioGroupState>,
    input: Memo<RadioGroupInput>,
    on_value_change: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: RadioGroupEvent) {
        // Events can still arrive after unmount (blur on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| RadioGroupMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        self.state.set(step.state);
        for command in step.commands {
            match command {
                RadioGroupCommand::FocusItem(value) => focus_id(&h::item_id(&self.id.get_value(), &value)),
                RadioGroupCommand::ValueChange { value, .. } => {
                    if let Some(cb) = self.on_value_change {
                        cb.run(value);
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
    #[prop(into)]
    label: String,
    /// The options, in display order.
    #[prop(into)]
    items: Signal<Vec<SegmentedOption>>,
    /// Controlled value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Disable the whole control.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Item padding and type size.
    #[prop(optional)]
    size: SegmentedSize,
    /// Form field name. When set, the selected value is submitted with the form.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| RadioGroupInput {
        items: items.get(),
        value: value.get(),
        default_value: default_value.clone(),
        disabled: disabled.get().unwrap_or(false),
        loop_focus: true,
    });
    let state = RwSignal::new(input.with_untracked(RadioGroupMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| RadioGroupMachine::sync(s, i)));
        state.set(synced);
    });
    provide_context(Ctx { id: StoredValue::new(oxide_headless::next_id(SCOPE)), state, input, on_value_change });

    let hidden = move || {
        let name = name.clone()?;
        let value = state.get().value?;
        Some(view! { <input type="hidden" name=name value=value /> })
    };

    view! {
        <div
            role="radiogroup"
            aria-label=label
            aria-orientation="horizontal"
            aria-disabled=move || input.with(|i| i.disabled).then_some("true")
            data-scope=SCOPE
            data-part="root"
            data-size=size.as_str()
        >
            {children()}
            {hidden}
        </div>
    }
}

/// One segment: a label and an optional leading icon.
#[component]
pub fn SegmentedItem(
    /// The option this segment selects.
    #[prop(into)]
    value: String,
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Icon before the label.
    #[prop(optional)]
    icon: Option<IconName>,
) -> impl IntoView {
    let ctx = use_context::<Ctx>().expect("SegmentedItem must be rendered inside <Segmented>");
    let attrs = {
        let value = value.clone();
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::item_attrs(&ctx.id.get_value(), s, i, &value))))
    };
    let (on_click, on_focus) = (value.clone(), value);
    view! {
        <button
            type="button"
            id=move || attrs.get().id
            role="radio"
            aria-checked=move || attrs.get().aria_checked.to_string()
            tabindex=move || attrs.get().tabindex
            disabled=move || attrs.get().disabled
            data-scope=SCOPE
            data-part="item"
            data-state=move || attrs.get().data_state
            on:click=move |_| ctx.send(RadioGroupEvent::Select(on_click.clone(), ChangeReason::Pointer))
            on:focus=move |_| ctx.send(RadioGroupEvent::Focus(on_focus.clone()))
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {icon.map(|name| view! { <Icon name size=16 /> })}
            {label}
        </button>
    }
}
