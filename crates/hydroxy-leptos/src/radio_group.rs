//! `<RadioGroup>` / `<RadioGroupItem>`.
//! Items read the machine from context and render the connector's attributes.

use crate::drive::drive;
use hydroxy_headless::radio_group::{
    self as h, ChangeReason, RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioGroupState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

pub use hydroxy_headless::radio_group::{Orientation, RadioItem};

use crate::focus::focus_id;

/// The machine, shared by the group's root and its items. `Segmented` reuses it.
#[derive(Clone, Copy)]
pub(crate) struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<RadioGroupState>,
    input: Memo<RadioGroupInput>,
    on_value_change: Option<Callback<String>>,
}

impl Ctx {
    /// Create the machine for a group whose parts use `scope` for their ids, and provide it to the items.
    pub(crate) fn provide(
        scope: &str,
        items: Signal<Vec<RadioItem>>,
        value: MaybeProp<String>,
        default_value: Option<String>,
        disabled: MaybeProp<bool>,
        loop_focus: bool,
        on_value_change: Option<Callback<String>>,
    ) -> Self {
        let input = Memo::new(move |_| RadioGroupInput {
            items: items.get(),
            value: value.get(),
            default_value: default_value.clone(),
            disabled: disabled.get().unwrap_or(false),
            loop_focus,
        });
        let state = RwSignal::new(input.with_untracked(RadioGroupMachine::initial));
        Effect::new(move |_| {
            let synced = input.with(|i| state.with_untracked(|s| RadioGroupMachine::sync(s, i)));
            if let Some(synced) = synced {
                state.set(synced);
            }
        });
        let ctx = Self { id: StoredValue::new(crate::id::next_id(scope)), state, input, on_value_change };
        provide_context(ctx);
        ctx
    }

    /// Whether the whole group is disabled.
    pub(crate) fn disabled(self) -> bool {
        self.input.with(|i| i.disabled)
    }

    /// The hidden form field carrying the selected value, when the group has a `name`.
    pub(crate) fn hidden_field(self, name: Option<String>) -> impl IntoView {
        move || {
            let name = name.clone()?;
            let value = self.state.get().value?;
            Some(view! { <input type="hidden" name=name value=value /> })
        }
    }

    /// The reactive attributes of the item holding `value`.
    pub(crate) fn item_attrs(self, value: &str) -> Memo<h::ItemAttrs> {
        let value = value.to_string();
        Memo::new(move |_| self.input.with(|i| self.state.with(|s| h::item_attrs(&self.id.get_value(), s, i, &value))))
    }

    /// The root id the item ids derive from.
    pub(crate) fn root_id(self) -> String {
        self.id.get_value()
    }

    /// A pointer click on the item holding `value`.
    pub(crate) fn click(self, value: &str) {
        self.send(RadioGroupEvent::Select(value.to_string(), ChangeReason::Pointer));
    }

    /// Focus landed on the item holding `value`.
    pub(crate) fn focused(self, value: &str) {
        self.send(RadioGroupEvent::Focus(value.to_string()));
    }

    /// Arrow, Home and End keys on an item.
    pub(crate) fn keydown(self, ev: &leptos::ev::KeyboardEvent) {
        if let Some(event) = h::keydown(&ev.key()) {
            ev.prevent_default();
            self.send(event);
        }
    }

    fn send(self, event: RadioGroupEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        for command in drive::<RadioGroupMachine, _>(self.state, self.input, event) {
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

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("RadioGroupItem must be rendered inside <RadioGroup>")
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn RadioGroup(
    /// The options, in display order.
    #[prop(into)]
    items: Signal<Vec<RadioItem>>,
    /// Controlled value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Disable the whole group.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Layout axis, vertical by default. Arrow keys work on both axes either way.
    #[prop(default = Orientation::Vertical)]
    orientation: Orientation,
    /// Wrap focus from the last option to the first and back.
    #[prop(optional, default = true)]
    loop_focus: bool,
    /// Form field name. When set, the selected value is submitted with the form.
    #[prop(optional, into)]
    name: Option<String>,
    /// Accessible name for the group (`aria-label`).
    #[prop(optional, into)]
    label: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let ctx = Ctx::provide(h::SCOPE, items, value, default_value, disabled, loop_focus, on_value_change);
    let hidden = ctx.hidden_field(name);

    view! {
        <div
            role="radiogroup"
            aria-label=label
            aria-orientation=orientation.as_str()
            aria-disabled=move || ctx.disabled().then_some("true")
            data-scope=h::SCOPE
            data-part="root"
            data-orientation=orientation.as_str()
        >
            {children()}
            {hidden}
        </div>
    }
}

/// One option: the radio circle, its label and an optional description.
#[component]
pub fn RadioGroupItem(
    /// The option this item selects.
    #[prop(into)]
    value: String,
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Optional hint under the label.
    #[prop(optional, into)]
    description: Option<String>,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = ctx.item_attrs(&value);
    let root = ctx.root_id();
    let label_id = h::label_id(&root, &value);
    let description = description.map(|text| {
        view! {
            <span id=h::description_id(&root, &value) data-scope=h::SCOPE data-part="description">
                {text}
            </span>
        }
    });
    let has_description = description.is_some();
    let (on_click, on_focus) = (value.clone(), value);
    view! {
        <button
            type="button"
            id=move || attrs.with(|x| x.id.clone())
            role="radio"
            aria-checked=move || attrs.with(|x| x.aria_checked).to_string()
            aria-labelledby=move || attrs.with(|x| x.aria_labelledby.clone())
            aria-describedby=move || has_description.then(|| attrs.with(|x| x.aria_describedby.clone()))
            tabindex=move || attrs.with(|x| x.tabindex)
            disabled=move || attrs.with(|x| x.disabled)
            data-scope=h::SCOPE
            data-part="item"
            data-state=move || attrs.with(|x| x.data_state)
            on:click=move |_| ctx.click(&on_click)
            on:focus=move |_| ctx.focused(&on_focus)
            on:keydown=move |ev| ctx.keydown(&ev)
        >
            <span data-scope=h::SCOPE data-part="control" aria-hidden="true"></span>
            <span data-scope=h::SCOPE data-part="text">
                <span id=label_id data-scope=h::SCOPE data-part="label">
                    {label}
                </span>
                {description}
            </span>
        </button>
    }
}
