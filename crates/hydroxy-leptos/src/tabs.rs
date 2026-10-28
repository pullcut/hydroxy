//! `<Tabs>` / `<TabsList>` / `<TabsTrigger>` / `<TabsContent>`.
//! Parts read the machine from context and render the connector's attributes.

use hydroxy_headless::tabs::{self as h, TabsCommand, TabsEvent, TabsInput, TabsMachine, TabsState};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::focus::focus_id;

pub use hydroxy_headless::tabs::{Activation, ChangeReason, Orientation, TabsItem};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<TabsState>,
    input: Memo<TabsInput>,
    orientation: Orientation,
    on_value_change: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: TabsEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| TabsMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                TabsCommand::FocusTrigger(value) => {
                    focus_id(&h::trigger_id(&self.id.get_value(), &value));
                }
                TabsCommand::ValueChange { value, .. } => {
                    if let Some(cb) = self.on_value_change {
                        cb.run(value);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Tabs parts must be rendered inside <Tabs>")
}

/// Root. Owns the machine and provides it to the parts below.
#[component]
pub fn Tabs(
    /// The tabs, in display order.
    #[prop(into)]
    tabs: Signal<Vec<TabsItem>>,
    /// Controlled value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Whether arrow keys also select. See [`Activation`].
    #[prop(optional)]
    activation: Activation,
    /// Layout axis; picks the arrow keys.
    #[prop(optional)]
    orientation: Orientation,
    /// Wrap focus from the last tab to the first and back.
    #[prop(optional, default = true)]
    loop_focus: bool,
    /// Called with the new value whenever the selection changes or is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| TabsInput {
        tabs: tabs.get(),
        value: value.get(),
        default_value: default_value.clone(),
        activation,
        loop_focus,
    });
    let state = RwSignal::new(input.with_untracked(TabsMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| TabsMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    provide_context(Ctx {
        id: StoredValue::new(crate::id::next_id(h::SCOPE)),
        state,
        input,
        orientation,
        on_value_change,
    });

    view! {
        <div data-scope=h::SCOPE data-part="root" data-orientation=orientation.as_str()>
            {children()}
        </div>
    }
}

/// The `tablist` wrapping the triggers.
#[component]
pub fn TabsList(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <div
            role="tablist"
            aria-orientation=ctx.orientation.as_str()
            data-scope=h::SCOPE
            data-part="list"
        >
            {children()}
        </div>
    }
}

/// The button that selects the tab `value`.
#[component]
pub fn TabsTrigger(
    /// The tab this trigger selects.
    #[prop(into)]
    value: String,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = {
        let value = value.clone();
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s, i, &value))))
    };
    let (on_click, on_focus) = (value.clone(), value);
    view! {
        <button
            type="button"
            id=move || attrs.get().id
            role="tab"
            aria-selected=move || attrs.get().aria_selected.to_string()
            aria-controls=move || attrs.get().aria_controls
            tabindex=move || attrs.get().tabindex
            disabled=move || attrs.get().disabled
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.get().data_state
            on:click=move |_| ctx.send(TabsEvent::Select(on_click.clone(), ChangeReason::Pointer))
            on:focus=move |_| ctx.send(TabsEvent::Focus(on_focus.clone()))
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key(), ctx.orientation) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {children()}
        </button>
    }
}

/// The panel shown while `value` is selected.
#[component]
pub fn TabsContent(
    /// The tab this panel belongs to.
    #[prop(into)]
    value: String,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, &value)));
    view! {
        <div
            id=move || attrs.get().id
            role="tabpanel"
            aria-labelledby=move || attrs.get().aria_labelledby
            tabindex=move || attrs.get().tabindex
            hidden=move || attrs.get().hidden
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.get().data_state
        >
            {children()}
        </div>
    }
}
