//! `<Rating>`: pick a score with stars. A radio group of star buttons with
//! optional half stars, a hover preview and a read-only display mode.

use hydroxy_headless::icons::Icon as IconName;
use hydroxy_headless::rating::{self as h, ChangeReason, RatingCommand, RatingEvent, RatingInput, RatingMachine};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::focus::focus_id;
use crate::icon::Icon;

/// A star rating. Uncontrolled by default; pass `value` to control it.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Rating(
    /// Accessible name for the group. Read-only ratings name themselves ("Rated 4 out of 5").
    #[prop(default = "Rating".into(), into)]
    label: String,
    /// Number of stars.
    #[prop(default = 5)]
    count: usize,
    /// Controlled value, `0` for unrated. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<f64>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_value: f64,
    /// Allow half stars: the left half of a star picks `n - 0.5`.
    #[prop(optional)]
    allow_half: bool,
    /// Picking the current value again clears the rating.
    #[prop(optional)]
    clearable: bool,
    /// Display only.
    #[prop(optional, into)]
    read_only: MaybeProp<bool>,
    /// Ignore picks and drop out of the tab order.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Star size in px.
    #[prop(default = 20)]
    size: u16,
    /// Form field name. When set, the value is submitted with the form.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the new value whenever it changes or is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<f64>>,
) -> impl IntoView {
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| RatingInput {
        count,
        value: value.get(),
        default_value,
        allow_half,
        clearable,
        read_only: read_only.get().unwrap_or(false),
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(RatingMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| RatingMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let star_id = move |index: usize| format!("{}-star-{index}", id.get_value());
    let send = move |event: RatingEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| RatingMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                RatingCommand::FocusStar(index) => focus_id(&star_id(index)),
                RatingCommand::ValueChange { value, .. } => {
                    if let Some(cb) = on_value_change {
                        cb.run(value);
                    }
                }
            }
        }
    };

    let root = Memo::new(move |_| input.with(|i| state.with(|s| h::root_attrs(s, i, &label))));
    let icon = move || {
        view! {
            <span data-scope=h::SCOPE data-part="icon" aria-hidden="true">
                <Icon name=IconName::Star size=size />
                <span data-scope=h::SCOPE data-part="fill"><Icon name=IconName::Star size=size /></span>
            </span>
        }
    };
    let star = move |index: usize| {
        let attrs = Memo::new(move |_| input.with(|i| state.with(|s| h::star_attrs(s, i, index))));
        let interactive = move || input.with(|i| !i.read_only);
        view! {
            <Show
                when=interactive
                fallback=move || view! {
                    <span data-scope=h::SCOPE data-part="star" data-state=move || attrs.get().data_state>{icon()}</span>
                }
            >
                <button
                    type="button"
                    id=star_id(index)
                    role="radio"
                    aria-checked=move || attrs.get().aria_checked.to_string()
                    aria-label=move || attrs.get().aria_label
                    tabindex=move || attrs.get().tabindex
                    disabled=move || attrs.get().disabled
                    data-scope=h::SCOPE
                    data-part="star"
                    data-state=move || attrs.get().data_state
                    on:pointerenter=move |_| send(RatingEvent::Hover(Some(h::pick(index, false))))
                    on:click=move |_| send(RatingEvent::Select(h::pick(index, false), ChangeReason::Pointer))
                    on:keydown=move |ev| {
                        if let Some(event) = h::keydown(&ev.key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                >
                    {icon()}
                    {allow_half.then(|| view! {
                        <span
                            data-scope=h::SCOPE
                            data-part="half"
                            aria-hidden="true"
                            on:pointerenter=move |_| send(RatingEvent::Hover(Some(h::pick(index, true))))
                            on:pointerleave=move |_| send(RatingEvent::Hover(Some(h::pick(index, false))))
                            on:click=move |ev| {
                                ev.stop_propagation();
                                send(RatingEvent::Select(h::pick(index, true), ChangeReason::Pointer));
                            }
                        ></span>
                    })}
                </button>
            </Show>
        }
    };
    let hidden = move || {
        let name = name.clone()?;
        Some(view! { <input type="hidden" name=name value=move || state.get().value.to_string() /> })
    };

    view! {
        <div
            role=move || root.get().role
            aria-label=move || root.get().aria_label
            aria-disabled=move || root.get().aria_disabled.then_some("true")
            data-scope=h::SCOPE
            data-part="root"
            data-readonly=move || input.with(|i| i.read_only).then_some("")
            data-disabled=move || input.with(|i| i.disabled).then_some("")
            on:pointerleave=move |_| send(RatingEvent::Hover(None))
        >
            {(1..=count).map(star).collect_view()}
            {hidden}
        </div>
    }
}
