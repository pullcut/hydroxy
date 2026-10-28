//! `Wizard` / `WizardPanel`: a multi-step flow with a step list, one panel at
//! a time and Back / Next buttons. Next asks `can_advance` first, so a step
//! can block until its fields are valid.

use dioxus::prelude::*;
use hydroxy_headless::icons::Icon as IconName;
use hydroxy_headless::wizard::{
    self as h, ChangeReason, WizardCommand, WizardEvent, WizardInput, WizardMachine, WizardState,
};
use hydroxy_statechart::Machine;

use crate::icon::Icon;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<WizardState>,
}

/// Root. Owns the machine and renders the step list, the current panel and the buttons.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Wizard(
    /// Step names, in order.
    steps: Vec<String>,
    /// Controlled step index. Pair with `on_step_change`.
    #[props(default)]
    step: ReadSignal<Option<usize>>,
    /// Initial step when uncontrolled.
    #[props(default)]
    default_step: usize,
    /// Steps unlock in order. Off: any step can be opened from the list at any time.
    #[props(default = true)]
    linear: bool,
    /// Asked with the current step before Next moves on. Return `false` to stay.
    can_advance: Option<Callback<usize, bool>>,
    /// Called with the new step whenever it changes or is requested.
    on_step_change: Option<EventHandler<usize>>,
    /// Called when Next is pressed on the last step and it passes.
    on_finish: Option<EventHandler<()>>,
    /// Accessible name for the step list.
    #[props(default = "Progress".to_string())]
    label: String,
    /// Back button text.
    #[props(default = "Back".to_string())]
    back_label: String,
    /// Next button text.
    #[props(default = "Next".to_string())]
    next_label: String,
    /// Next button text on the last step.
    #[props(default = "Finish".to_string())]
    finish_label: String,
    /// `WizardPanel`s, one per step.
    children: Element,
) -> Element {
    let count = steps.len();
    let input = use_memo(move || WizardInput { count, step: step(), default_step, linear });
    let mut state = use_signal(|| WizardMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = WizardMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = use_context_provider(|| Ctx { id: CopyValue::new(hydroxy_headless::next_id(h::SCOPE)), state });

    let mut send = move |event: WizardEvent| {
        let step = WizardMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                WizardCommand::StepChange { step, .. } => {
                    if let Some(handler) = on_step_change {
                        handler.call(step);
                    }
                }
                WizardCommand::Finish => {
                    if let Some(handler) = on_finish {
                        handler.call(());
                    }
                }
            }
        }
    };

    let root = ctx.id.read().clone();
    let s = state.read();
    let i = input.read();
    let at_start = s.current == 0;
    let next_text = if h::is_last(&s, &i) { finish_label } else { next_label };

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            ol { "data-scope": h::SCOPE, "data-part": "steps", "aria-label": label,
                for (index, name) in steps.into_iter().enumerate() {
                    {
                        let a = h::step_attrs(&root, &s, &i, index);
                        let sr = h::step_sr(&s, index);
                        rsx! {
                            li { key: "{index}", "data-scope": h::SCOPE, "data-part": "step", "data-state": a.data_state,
                                button {
                                    r#type: "button",
                                    id: a.id,
                                    "aria-current": a.aria_current,
                                    disabled: a.disabled,
                                    "data-scope": h::SCOPE,
                                    "data-part": "trigger",
                                    "data-state": a.data_state,
                                    onclick: move |_| send(WizardEvent::GoTo(index, ChangeReason::Pointer)),
                                    span { "data-scope": h::SCOPE, "data-part": "indicator", "aria-hidden": "true",
                                        if a.data_state == "complete" {
                                            Icon { name: IconName::Check, size: 12 }
                                        } else {
                                            "{index + 1}"
                                        }
                                    }
                                    span { "data-scope": h::SCOPE, "data-part": "label",
                                        "{name}"
                                        span { "data-scope": h::SCOPE, "data-part": "sr", {sr} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            {children}
            div { "data-scope": h::SCOPE, "data-part": "footer",
                button {
                    r#type: "button",
                    disabled: at_start,
                    "data-scope": h::SCOPE,
                    "data-part": "back",
                    class: "ox-button",
                    "data-variant": "secondary",
                    onclick: move |_| send(WizardEvent::Back(ChangeReason::Pointer)),
                    {back_label}
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "next",
                    class: "ox-button",
                    "data-variant": "primary",
                    onclick: move |_| {
                        let current = state.peek().current;
                        if can_advance.is_none_or(|cb| cb.call(current)) {
                            send(WizardEvent::Next(ChangeReason::Pointer));
                        }
                    },
                    {next_text}
                }
            }
        }
    }
}

/// One step's content. Rendered only while its step is current.
#[component]
pub fn WizardPanel(
    /// The step index this panel belongs to.
    index: usize,
    /// The step's content.
    children: Element,
) -> Element {
    let ctx = try_use_context::<Ctx>().expect("WizardPanel must be rendered inside Wizard");
    if ctx.state.read().current != index {
        return rsx! {};
    }
    let root = ctx.id.read().clone();
    rsx! {
        section {
            id: h::panel_id(&root),
            "aria-labelledby": h::step_id(&root, index),
            "data-scope": h::SCOPE,
            "data-part": "panel",
            {children}
        }
    }
}
