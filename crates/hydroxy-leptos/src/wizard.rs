//! `<Wizard>` / `<WizardPanel>`: a multi-step flow with a step list, one
//! panel at a time and Back / Next buttons. Next asks `can_advance` first, so
//! a step can block until its fields are valid.

use hydroxy_headless::icons::Icon as IconName;
use hydroxy_headless::wizard::{
    self as h, ChangeReason, WizardCommand, WizardEvent, WizardInput, WizardMachine, WizardState,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::Icon;

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<WizardState>,
}

/// Root. Owns the machine and renders the step list, the current panel and the buttons.
#[component]
pub fn Wizard(
    /// Step names, in order.
    #[prop(into)]
    steps: Vec<String>,
    /// Controlled step index. Pair with `on_step_change`.
    #[prop(optional, into)]
    step: MaybeProp<usize>,
    /// Initial step when uncontrolled.
    #[prop(optional)]
    default_step: usize,
    /// Steps unlock in order. Off: any step can be opened from the list at any time.
    #[prop(default = true)]
    linear: bool,
    /// Asked with the current step before Next moves on. Return `false` to stay.
    #[prop(optional, into)]
    can_advance: Option<Callback<usize, bool>>,
    /// Called with the new step whenever it changes or is requested.
    #[prop(optional, into)]
    on_step_change: Option<Callback<usize>>,
    /// Called when Next is pressed on the last step and it passes.
    #[prop(optional, into)]
    on_finish: Option<Callback<()>>,
    /// Accessible name for the step list.
    #[prop(default = "Progress".into(), into)]
    label: String,
    /// Back button text.
    #[prop(default = "Back".into(), into)]
    back_label: String,
    /// Next button text.
    #[prop(default = "Next".into(), into)]
    next_label: String,
    /// Next button text on the last step.
    #[prop(default = "Finish".into(), into)]
    finish_label: String,
    /// `<WizardPanel>`s, one per step.
    children: Children,
) -> impl IntoView {
    let count = steps.len();
    let input = Memo::new(move |_| WizardInput { count, step: step.get(), default_step, linear });
    let state = RwSignal::new(input.with_untracked(WizardMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| WizardMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    provide_context(Ctx { id, state });

    let send = move |event: WizardEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| WizardMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                WizardCommand::StepChange { step, .. } => {
                    if let Some(cb) = on_step_change {
                        cb.run(step);
                    }
                }
                WizardCommand::Finish => {
                    if let Some(cb) = on_finish {
                        cb.run(());
                    }
                }
            }
        }
    };
    let next = move |_| {
        let current = state.with_untracked(|s| s.current);
        if can_advance.is_none_or(|cb| cb.run(current)) {
            send(WizardEvent::Next(ChangeReason::Pointer));
        }
    };
    let is_last = move || input.with(|i| state.with(|s| h::is_last(s, i)));

    let step_item = move |(index, name): (usize, String)| {
        let attrs = Memo::new(move |_| input.with(|i| state.with(|s| h::step_attrs(&id.get_value(), s, i, index))));
        view! {
            <li data-scope=h::SCOPE data-part="step" data-state=move || attrs.get().data_state>
                <button
                    type="button"
                    id=move || attrs.get().id
                    aria-current=move || attrs.get().aria_current
                    disabled=move || attrs.get().disabled
                    data-scope=h::SCOPE
                    data-part="trigger"
                    data-state=move || attrs.get().data_state
                    on:click=move |_| send(WizardEvent::GoTo(index, ChangeReason::Pointer))
                >
                    <span data-scope=h::SCOPE data-part="indicator" aria-hidden="true">
                        {move || if attrs.get().data_state == "complete" {
                            Either::Left(view! { <Icon name=IconName::Check size=12 /> })
                        } else {
                            Either::Right(index + 1)
                        }}
                    </span>
                    <span data-scope=h::SCOPE data-part="label">
                        {name}
                        <span data-scope=h::SCOPE data-part="sr">{move || state.with(|s| h::step_sr(s, index))}</span>
                    </span>
                </button>
            </li>
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <ol data-scope=h::SCOPE data-part="steps" aria-label=label>
                {steps.into_iter().enumerate().map(step_item).collect_view()}
            </ol>
            {children()}
            <div data-scope=h::SCOPE data-part="footer">
                <button
                    type="button"
                    disabled=move || state.with(|s| s.current == 0)
                    data-scope=h::SCOPE
                    data-part="back"
                    class="ox-button"
                    data-variant="secondary"
                    on:click=move |_| send(WizardEvent::Back(ChangeReason::Pointer))
                >
                    {back_label}
                </button>
                <button type="button" class="ox-button" data-variant="primary" data-scope=h::SCOPE data-part="next" on:click=next>
                    {move || if is_last() { finish_label.clone() } else { next_label.clone() }}
                </button>
            </div>
        </div>
    }
}

/// One step's content. Rendered only while its step is current.
#[component]
pub fn WizardPanel(
    /// The step index this panel belongs to.
    index: usize,
    /// The step's content.
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = use_context::<Ctx>().expect("WizardPanel must be rendered inside <Wizard>");
    let root = ctx.id.get_value();
    view! {
        <Show when=move || ctx.state.with(|s| s.current == index)>
            <section
                id=h::panel_id(&root)
                aria-labelledby=h::step_id(&root, index)
                data-scope=h::SCOPE
                data-part="panel"
            >
                {children()}
            </section>
        </Show>
    }
}
