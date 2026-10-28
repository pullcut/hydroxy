//! The previous / next branch picker for one message.

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::{
    self as h, BranchPickerCommand, BranchPickerEvent, BranchPickerInput, BranchPickerMachine,
};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};

/// Previous, "2 / 3", next: steps through the branches of one message.
#[component]
pub fn AssistantBranchPicker(
    /// How many branches there are.
    count: usize,
    /// Controlled position (zero-based). Pair with `on_switch`.
    index: Option<usize>,
    /// A reply is running: switching is disabled unless `switch_during_run`.
    #[props(default)]
    running: bool,
    /// Allow switching while a reply runs.
    #[props(default)]
    switch_during_run: bool,
    /// Called with the zero-based branch to show.
    on_switch: Option<EventHandler<usize>>,
) -> Element {
    let input = BranchPickerInput { count, index, running, switch_during_run };
    let mut state = use_signal(|| BranchPickerMachine::initial(&input));
    // Follow the owner's index and count on every render.
    let current = BranchPickerMachine::sync(&state.read(), &input).unwrap_or_else(|| state.read().clone());
    let mut send = move |event: BranchPickerEvent| {
        let synced = BranchPickerMachine::sync(&state.peek(), &input);
        let base = synced.clone().unwrap_or_else(|| state.peek().clone());
        let step = BranchPickerMachine::transition(&base, event, &input);
        if let Some(next) = step.state.or(synced) {
            state.set(next);
        }
        for BranchPickerCommand::Switch(i) in step.commands {
            if let Some(handler) = on_switch {
                handler.call(i);
            }
        }
    };
    rsx! {
        nav { "data-scope": h::SCOPE_BRANCH_PICKER, "data-part": "root", "aria-label": "Branches",
            button {
                r#type: "button",
                "data-scope": h::SCOPE_BRANCH_PICKER,
                "data-part": "previous",
                "aria-label": "Previous branch",
                disabled: !h::can_previous(&current, &input),
                onclick: move |_| send(BranchPickerEvent::Previous),
                Icon { name: IconName::ChevronRight, size: 14 }
            }
            span { "data-scope": h::SCOPE_BRANCH_PICKER, "data-part": "label", "aria-live": "polite",
                "{h::branch_label(current.index.value, count)}"
            }
            button {
                r#type: "button",
                "data-scope": h::SCOPE_BRANCH_PICKER,
                "data-part": "next",
                "aria-label": "Next branch",
                disabled: !h::can_next(&current, &input),
                onclick: move |_| send(BranchPickerEvent::Next),
                Icon { name: IconName::ChevronRight, size: 14 }
            }
        }
    }
}
