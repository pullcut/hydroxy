//! `<FilterBuilder>`: rows of field, operator and value, joined by AND or OR.
//! The operators on offer depend on the field's type. Produces a plain
//! [`Filter`] value the owner can store, send or test records against.

use crate::drive::drive;
use hydroxy_headless::filter_builder::{
    self as h, Field, FieldKind, Filter, FilterBuilderCommand, FilterBuilderEvent, FilterBuilderInput,
    FilterBuilderMachine, FilterBuilderState, Op, Rule,
};
use hydroxy_headless::select::SelectOption;
use hydroxy_statechart::Machine;
use leptos::either::EitherOf3;
use leptos::prelude::*;

use crate::button::{Button, ButtonVariant};
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::input::Input;
use crate::select::Select;

pub use hydroxy_headless::filter_builder::{
    Choice, Field as FilterField, FieldKind as FilterFieldKind, Join as FilterJoin,
};
pub use hydroxy_headless::filter_builder::{Filter as FilterValue, Op as FilterOp, Rule as FilterRule};

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<FilterBuilderState>,
    input: Memo<FilterBuilderInput>,
    on_change: Option<Callback<Filter>>,
}

impl Ctx {
    fn send(self, event: FilterBuilderEvent) {
        for command in drive::<FilterBuilderMachine, _>(self.state, self.input, event) {
            let FilterBuilderCommand::Change(filter) = command;
            if let Some(cb) = self.on_change {
                cb.run(filter);
            }
        }
    }
}

fn options(pairs: Vec<(String, String)>) -> Vec<SelectOption> {
    pairs.into_iter().map(|(v, l)| SelectOption::new(v, l)).collect()
}

/// A builder for filter conditions.
#[component]
pub fn FilterBuilder(
    /// The fields rules can test. The first is the default for a new rule.
    fields: Vec<Field>,
    /// Controlled filter. Pair with `on_change`.
    #[prop(optional, into)]
    filter: MaybeProp<Filter>,
    /// Initial filter when uncontrolled.
    #[prop(optional, into)]
    default_filter: Filter,
    /// Called with the whole filter after every edit.
    #[prop(optional, into)]
    on_change: Option<Callback<Filter>>,
) -> impl IntoView {
    let input = Memo::new(move |_| FilterBuilderInput {
        fields: fields.clone(),
        filter: filter.get(),
        default_filter: default_filter.clone(),
    });
    let state = RwSignal::new(input.with_untracked(FilterBuilderMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| FilterBuilderMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, on_change };
    let rows =
        Memo::new(move |_| state.with(|s| s.filter.value.rules.iter().map(|r| r.id).enumerate().collect::<Vec<_>>()));
    let count = move || state.with(|s| s.filter.value.rules.len());

    let (many, any) = (move || count() > 1, move || count() > 0);
    let empty = || view! { <p data-scope=h::SCOPE data-part="empty">"No conditions. Everything matches."</p> };

    view! {
        <div data-scope=h::SCOPE data-part="root" role="group" aria-label=h::LABEL>
            <Show when=many>
                <div data-scope=h::SCOPE data-part="join">
                    <Select
                        label="Combine conditions"
                        options=options(h::join_options())
                        value=Signal::derive(move || Some(state.with(|s| s.filter.value.join.as_str().to_string())))
                        on_change=move |v: String| ctx.send(FilterBuilderEvent::SetJoin(h::parse_join(&v)))
                    />
                </div>
            </Show>
            <Show
                when=any
                fallback=empty
            >
                <ul role="list" data-scope=h::SCOPE data-part="rules">
                    // The index is part of the key because it is part of each control's accessible name.
                    <For each=move || rows.get() key={|r: &(usize, u32)| *r} children=move |(index, id)| rule_view(ctx, index, id) />
                </ul>
            </Show>
            <div data-scope=h::SCOPE data-part="footer">
                <Button
                    variant=ButtonVariant::Secondary
                    icon_left=IconName::Plus
                    on_click=move |_| ctx.send(FilterBuilderEvent::Add)
                >
                    "Add condition"
                </Button>
            </div>
        </div>
    }
}

fn rule_view(ctx: Ctx, index: usize, id: u32) -> impl IntoView {
    let labels = h::rule_labels(index);
    let rule = Memo::new(move |_| ctx.state.with(|s| s.filter.value.rules.iter().find(|r| r.id == id).cloned()));
    let kind = move |r: &Rule| ctx.input.with(|i| i.fields.iter().find(|f| f.id == r.field).map(|f| f.kind.clone()));
    // Re-renders when the field or operator changes. Typing in the value never does.
    let shape = Memo::new(move |_| rule.with(|r| r.as_ref().map(|r| (r.field.clone(), r.op))));
    let field_options = Signal::derive(move || {
        ctx.input
            .with(|i| i.fields.iter().map(|f| SelectOption::new(f.id.clone(), f.label.clone())).collect::<Vec<_>>())
    });
    let op_options = Signal::derive(move || {
        let k = rule.with(|r| r.as_ref().and_then(kind)).unwrap_or(FieldKind::Text);
        options(h::op_options(&k))
    });
    let value_label = labels.value.clone();
    let value_control = move || {
        let (_, op) = shape.get()?;
        let k = rule.with(|r| r.as_ref().and_then(kind))?;
        if !op.needs_value() {
            return None;
        }
        let current =
            Signal::derive(move || Some(rule.with(|r| r.as_ref().map(|r| r.value.clone()).unwrap_or_default())));
        let set = move |value: String| ctx.send(FilterBuilderEvent::SetValue { rule: id, value });
        Some(match k {
            FieldKind::Choice(choices) => {
                let opts = choices.into_iter().map(|c| SelectOption::new(c.value, c.label)).collect::<Vec<_>>();
                EitherOf3::A(view! { <Select label=value_label.clone() options=opts value=current on_change=set /> })
            }
            FieldKind::Number => EitherOf3::B(view! {
                <Input label=value_label.clone() input_type="number" value=current on_input=set />
            }),
            FieldKind::Text => EitherOf3::C(view! { <Input label=value_label.clone() value=current on_input=set /> }),
        })
    };

    view! {
        <li role="group" aria-label=labels.group data-scope=h::SCOPE data-part="rule">
            <div data-scope=h::SCOPE data-part="field">
                <Select
                    label=labels.field
                    options=field_options
                    value=Signal::derive(move || shape.get().map(|(f, _)| f))
                    on_change=move |field: String| ctx.send(FilterBuilderEvent::SetField { rule: id, field })
                />
            </div>
            <div data-scope=h::SCOPE data-part="op">
                <Select
                    label=labels.op
                    options=op_options
                    value=Signal::derive(move || shape.get().map(|(_, op)| op.as_str().to_string()))
                    on_change=move |v: String| {
                        if let Some(op) = Op::parse(&v) {
                            ctx.send(FilterBuilderEvent::SetOp { rule: id, op });
                        }
                    }
                />
            </div>
            <div data-scope=h::SCOPE data-part="value">{value_control}</div>
            <div data-scope=h::SCOPE data-part="remove">
                <IconButton
                    icon=IconName::Trash
                    label=labels.remove
                    on_click=move |_| ctx.send(FilterBuilderEvent::Remove(id))
                />
            </div>
        </li>
    }
}
