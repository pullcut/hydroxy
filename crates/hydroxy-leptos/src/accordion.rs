//! `<Accordion>` / `<AccordionItem>` / `<AccordionTrigger>` / `<AccordionContent>`.
//! Parts read the machine from context and render the connector's attributes.
//!
//! Parts panic if rendered outside their root `Accordion`; `AccordionTrigger` and
//! `AccordionContent` also need an `AccordionItem`.

use hydroxy_headless::accordion::{
    self as h, AccordionCommand, AccordionEvent, AccordionInput, AccordionMachine, AccordionState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};

pub use hydroxy_headless::accordion::AccordionEntry;

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<AccordionState>,
    input: Memo<AccordionInput>,
    on_value_change: Option<Callback<Vec<String>>>,
}

impl Ctx {
    fn send(self, event: AccordionEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        for command in drive::<AccordionMachine, _>(self.state, self.input, event) {
            match command {
                AccordionCommand::FocusTrigger(value) => focus_id(&h::trigger_id(&self.id.get_value(), &value)),
                AccordionCommand::ValueChange(value) => {
                    if let Some(cb) = self.on_value_change {
                        cb.run(value);
                    }
                }
            }
        }
    }
}

/// The item value, provided by `<AccordionItem>` to its trigger and content.
#[derive(Clone)]
struct ItemValue(String);

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Accordion parts must be rendered inside <Accordion>")
}

fn use_item() -> String {
    use_context::<ItemValue>().expect("AccordionTrigger and AccordionContent must be inside <AccordionItem>").0
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn Accordion(
    /// The items, in display order.
    #[prop(into)]
    items: Signal<Vec<AccordionEntry>>,
    /// Controlled open items. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<Vec<String>>,
    /// Initially open items when uncontrolled.
    #[prop(optional, into)]
    default_value: Vec<String>,
    /// Allow several items open at once.
    #[prop(optional)]
    multiple: bool,
    /// In single mode, whether the open item can be closed again.
    #[prop(optional, default = true)]
    collapsible: bool,
    /// Called with the open items whenever they change or a change is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<Vec<String>>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| AccordionInput {
        items: items.get(),
        multiple,
        value: value.get(),
        default_value: default_value.clone(),
        collapsible,
    });
    let state = RwSignal::new(input.with_untracked(AccordionMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| AccordionMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    provide_context(Ctx { id: StoredValue::new(crate::id::next_id(h::SCOPE)), state, input, on_value_change });

    view! {
        <div data-scope=h::SCOPE data-part="root" data-orientation="vertical">
            {children()}
        </div>
    }
}

/// One item. Wraps its trigger and content.
#[component]
pub fn AccordionItem(
    /// The item this wraps. Must match an entry in `items`.
    #[prop(into)]
    value: String,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    provide_context(ItemValue(value.clone()));
    let attrs = Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::item_attrs(s, i, &value))));
    view! {
        <div
            data-scope=h::SCOPE
            data-part="item"
            data-state=move || attrs.with(|x| x.data_state)
            data-disabled=move || attrs.with(|x| x.disabled).then_some("")
        >
            {children()}
        </div>
    }
}

/// The heading button that opens and closes its item.
#[component]
pub fn AccordionTrigger(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let value = use_item();
    let attrs = {
        let value = value.clone();
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s, i, &value))))
    };
    let (on_click, on_focus) = (value.clone(), value);
    view! {
        <h3 data-scope=h::SCOPE data-part="heading">
            <button
                type="button"
                id=move || attrs.with(|x| x.id.clone())
                aria-expanded=move || attrs.with(|x| x.aria_expanded).to_string()
                aria-controls=move || attrs.with(|x| x.aria_controls.clone())
                disabled=move || attrs.with(|x| x.disabled)
                data-scope=h::SCOPE
                data-part="trigger"
                data-state=move || attrs.with(|x| x.data_state)
                data-disabled=move || attrs.with(|x| x.disabled).then_some("")
                on:click=move |_| ctx.send(AccordionEvent::Toggle(on_click.clone()))
                on:focus=move |_| ctx.send(AccordionEvent::Focus(on_focus.clone()))
                on:keydown=move |ev| {
                    if let Some(event) = h::keydown(&ev.key()) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
            >
                <span>{children()}</span>
                <Icon name=IconName::ChevronDown size=16 />
            </button>
        </h3>
    }
}

/// The region shown while its item is open.
#[component]
pub fn AccordionContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let value = use_item();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, &value)));
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
