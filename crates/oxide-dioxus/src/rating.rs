//! `Rating`: pick a score with stars. A radio group of star buttons with
//! optional half stars, a hover preview and a read-only display mode.

use dioxus::prelude::*;
use oxide_headless::icons::Icon as IconName;
use oxide_headless::rating::{self as h, ChangeReason, RatingCommand, RatingEvent, RatingInput, RatingMachine};
use oxide_statechart::Machine;

use crate::focus::FocusMap;
use crate::icon::Icon;

/// A star rating. Uncontrolled by default; pass `value` to control it.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Rating(
    /// Accessible name for the group. Read-only ratings name themselves ("Rated 4 out of 5").
    #[props(default = "Rating".to_string())]
    label: String,
    /// Number of stars.
    #[props(default = 5)]
    count: usize,
    /// Controlled value, `0` for unrated. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<f64>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: f64,
    /// Allow half stars: the left half of a star picks `n - 0.5`.
    #[props(default)]
    allow_half: bool,
    /// Picking the current value again clears the rating.
    #[props(default)]
    clearable: bool,
    /// Display only.
    #[props(default)]
    read_only: ReadSignal<bool>,
    /// Ignore picks and drop out of the tab order.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Star size in px.
    #[props(default = 20)]
    size: u16,
    /// Form field name. When set, the value is submitted with the form.
    name: Option<String>,
    /// Called with the new value whenever it changes or is requested.
    on_value_change: Option<EventHandler<f64>>,
) -> Element {
    let input = use_memo(move || RatingInput {
        count,
        value: value(),
        default_value,
        allow_half,
        clearable,
        read_only: read_only(),
        disabled: disabled(),
    });
    let mut state = use_signal(|| RatingMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = RatingMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });
    let stars = use_hook(FocusMap::new);

    let mut send = move |event: RatingEvent| {
        let step = RatingMachine::transition(&state.peek(), event, &input.peek());
        state.set(step.state);
        for command in step.commands {
            match command {
                RatingCommand::FocusStar(index) => stars.focus(&index.to_string()),
                RatingCommand::ValueChange { value, .. } => {
                    if let Some(handler) = on_value_change {
                        handler.call(value);
                    }
                }
            }
        }
    };

    let s = state.read();
    let i = input.read();
    let root = h::root_attrs(&s, &i, &label);
    let read_only = i.read_only;
    let current = s.value;
    let icon = rsx! {
        span { "data-scope": h::SCOPE, "data-part": "icon", "aria-hidden": "true",
            Icon { name: IconName::Star, size }
            span { "data-scope": h::SCOPE, "data-part": "fill",
                Icon { name: IconName::Star, size }
            }
        }
    };

    rsx! {
        div {
            role: root.role,
            "aria-label": root.aria_label,
            "aria-disabled": root.aria_disabled.then_some("true"),
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-readonly": read_only.then_some(""),
            "data-disabled": i.disabled.then_some(""),
            onpointerleave: move |_| send(RatingEvent::Hover(None)),
            for index in 1..=count {
                if read_only {
                    span {
                        key: "{index}",
                        "data-scope": h::SCOPE,
                        "data-part": "star",
                        "data-state": h::fill(s.display(), index),
                        {icon.clone()}
                    }
                } else {
                    {
                        let a = h::star_attrs(&s, &i, index);
                        rsx! {
                            button {
                                key: "{index}",
                                r#type: "button",
                                role: a.role,
                                "aria-checked": a.aria_checked.to_string(),
                                "aria-label": a.aria_label,
                                tabindex: a.tabindex,
                                disabled: a.disabled,
                                "data-scope": h::SCOPE,
                                "data-part": "star",
                                "data-state": a.data_state,
                                onmounted: move |e| stars.register(index.to_string(), e.data()),
                                onpointerenter: move |_| send(RatingEvent::Hover(Some(h::pick(index, false)))),
                                onclick: move |_| send(RatingEvent::Select(h::pick(index, false), ChangeReason::Pointer)),
                                onkeydown: move |e| {
                                    if let Some(event) = h::keydown(&e.key().to_string()) {
                                        e.prevent_default();
                                        send(event);
                                    }
                                },
                                {icon.clone()}
                                if allow_half {
                                    span {
                                        "data-scope": h::SCOPE,
                                        "data-part": "half",
                                        "aria-hidden": "true",
                                        onpointerenter: move |_| send(RatingEvent::Hover(Some(h::pick(index, true)))),
                                        onpointerleave: move |_| send(RatingEvent::Hover(Some(h::pick(index, false)))),
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            send(RatingEvent::Select(h::pick(index, true), ChangeReason::Pointer));
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: "{current}" }
            }
        }
    }
}
