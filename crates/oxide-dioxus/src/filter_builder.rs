//! `FilterBuilder`: rows of field, operator and value, joined by AND or OR.
//! The operators on offer depend on the field's type. Produces a plain
//! [`Filter`] value the owner can store, send or test records against.

use dioxus::prelude::*;
use oxide_headless::filter_builder::{
    self as h, Field, FieldKind, Filter, FilterBuilderCommand, FilterBuilderEvent, FilterBuilderInput,
    FilterBuilderMachine, FilterBuilderState, Op,
};
use oxide_headless::select::SelectOption;
use oxide_statechart::Machine;

use crate::button::{Button, ButtonVariant};
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::input::Input;
use crate::select::Select;

pub use oxide_headless::filter_builder::{Choice, Field as FilterField, FieldKind as FilterFieldKind, Join as FilterJoin};
pub use oxide_headless::filter_builder::{Filter as FilterValue, Op as FilterOp, Rule as FilterRule};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<FilterBuilderState>,
    input: Memo<FilterBuilderInput>,
    on_change: Option<EventHandler<Filter>>,
}

impl Ctx {
    fn send(mut self, event: FilterBuilderEvent) {
        let step = FilterBuilderMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            let FilterBuilderCommand::Change(filter) = command;
            if let Some(handler) = self.on_change {
                handler.call(filter);
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
    #[props(default)]
    filter: ReadSignal<Option<Filter>>,
    /// Initial filter when uncontrolled.
    #[props(default)]
    default_filter: Filter,
    /// Called with the whole filter after every edit.
    on_change: Option<EventHandler<Filter>>,
) -> Element {
    let input = use_memo(move || FilterBuilderInput {
        fields: fields.clone(),
        filter: filter(),
        default_filter: default_filter.clone(),
    });
    let mut state = use_signal(|| FilterBuilderMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = FilterBuilderMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = use_hook(|| Ctx { state, input, on_change });

    let s = state.read();
    let i = input.read();
    let field_options: Vec<SelectOption> =
        i.fields.iter().map(|f| SelectOption::new(f.id.clone(), f.label.clone())).collect();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", role: "group", "aria-label": h::LABEL,
            if s.filter.rules.len() > 1 {
                div { "data-scope": h::SCOPE, "data-part": "join",
                    Select {
                        label: "Combine conditions",
                        options: options(h::join_options()),
                        value: s.filter.join.as_str().to_string(),
                        on_change: move |v: String| ctx.send(FilterBuilderEvent::SetJoin(h::parse_join(&v))),
                    }
                }
            }
            if s.filter.rules.is_empty() {
                p { "data-scope": h::SCOPE, "data-part": "empty", "No conditions. Everything matches." }
            } else {
                ul { role: "list", "data-scope": h::SCOPE, "data-part": "rules",
                    for (index , rule) in s.filter.rules.iter().enumerate() {
                        {
                            let id = rule.id;
                            let labels = h::rule_labels(index);
                            let kind = i.fields.iter().find(|f| f.id == rule.field).map_or(FieldKind::Text, |f| f.kind.clone());
                            let ops = options(h::op_options(&kind));
                            let value_label = labels.value.clone();
                            let value = rule.value.clone();
                            let set = move |value: String| ctx.send(FilterBuilderEvent::SetValue { rule: id, value });
                            rsx! {
                                li {
                                    key: "{id}-{index}",
                                    role: "group",
                                    "aria-label": labels.group,
                                    "data-scope": h::SCOPE,
                                    "data-part": "rule",
                                    div { "data-scope": h::SCOPE, "data-part": "field",
                                        Select {
                                            label: labels.field,
                                            options: field_options.clone(),
                                            value: rule.field.clone(),
                                            on_change: move |field: String| ctx.send(FilterBuilderEvent::SetField { rule: id, field }),
                                        }
                                    }
                                    div { "data-scope": h::SCOPE, "data-part": "op",
                                        Select {
                                            label: labels.op,
                                            options: ops,
                                            value: rule.op.as_str().to_string(),
                                            on_change: move |v: String| {
                                                if let Some(op) = Op::parse(&v) {
                                                    ctx.send(FilterBuilderEvent::SetOp { rule: id, op });
                                                }
                                            },
                                        }
                                    }
                                    div { "data-scope": h::SCOPE, "data-part": "value",
                                        if rule.op.needs_value() {
                                            match kind {
                                                FieldKind::Choice(choices) => rsx! {
                                                    Select {
                                                        label: value_label,
                                                        options: choices.into_iter().map(|c| SelectOption::new(c.value, c.label)).collect::<Vec<_>>(),
                                                        value,
                                                        on_change: set,
                                                    }
                                                },
                                                FieldKind::Number => rsx! {
                                                    Input { label: value_label, input_type: "number", value, on_input: set }
                                                },
                                                FieldKind::Text => rsx! {
                                                    Input { label: value_label, value, on_input: set }
                                                },
                                            }
                                        }
                                    }
                                    div { "data-scope": h::SCOPE, "data-part": "remove",
                                        IconButton {
                                            icon: IconName::Trash,
                                            label: labels.remove,
                                            on_click: move |_| ctx.send(FilterBuilderEvent::Remove(id)),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "footer",
                Button {
                    variant: ButtonVariant::Secondary,
                    icon_left: IconName::Plus,
                    on_click: move |_| ctx.send(FilterBuilderEvent::Add),
                    "Add condition"
                }
            }
        }
    }
}
