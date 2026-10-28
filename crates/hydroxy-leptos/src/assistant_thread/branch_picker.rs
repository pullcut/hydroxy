//! The branch picker.

use hydroxy_headless::assistant_thread::{
    self as h, BranchPickerCommand, BranchPickerEvent, BranchPickerInput, BranchPickerMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;
use crate::icon::{Icon, IconName};

/// Previous, "2 / 3", next: steps through the branches of one message.
#[component]
pub fn AssistantBranchPicker(
    /// How many branches there are.
    #[prop(into)]
    count: Signal<usize>,
    /// Controlled position (zero-based). Pair with `on_switch`.
    #[prop(optional, into)]
    index: MaybeProp<usize>,
    /// A reply is running: switching is disabled unless `switch_during_run`.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// Allow switching while a reply runs.
    #[prop(optional)]
    switch_during_run: bool,
    /// Called with the zero-based branch to show.
    #[prop(optional, into)]
    on_switch: Option<Callback<usize>>,
) -> impl IntoView {
    let input = Memo::new(move |_| BranchPickerInput {
        count: count.get(),
        index: index.get(),
        running: running.get(),
        switch_during_run,
    });
    let state = RwSignal::new(input.with_untracked(BranchPickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| BranchPickerMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let send = move |event: BranchPickerEvent| {
        for BranchPickerCommand::Switch(i) in drive::<BranchPickerMachine, _>(state, input, event) {
            if let Some(cb) = on_switch {
                cb.run(i);
            }
        }
    };
    let prev_ok = move || input.with(|i| state.with(|s| h::can_previous(s, i)));
    let next_ok = move || input.with(|i| state.with(|s| h::can_next(s, i)));
    view! {
        <nav data-scope=h::SCOPE_BRANCH_PICKER data-part="root" aria-label="Branches">
            <button
                type="button"
                data-scope=h::SCOPE_BRANCH_PICKER
                data-part="previous"
                aria-label="Previous branch"
                disabled=move || !prev_ok()
                on:click=move |_| send(BranchPickerEvent::Previous)
            >
                <Icon name=IconName::ChevronRight size=14 />
            </button>
            <span data-scope=h::SCOPE_BRANCH_PICKER data-part="label" aria-live="polite">
                {move || state.with(|s| h::branch_label(s.index.value, count.get()))}
            </span>
            <button
                type="button"
                data-scope=h::SCOPE_BRANCH_PICKER
                data-part="next"
                aria-label="Next branch"
                disabled=move || !next_ok()
                on:click=move |_| send(BranchPickerEvent::Next)
            >
                <Icon name=IconName::ChevronRight size=14 />
            </button>
        </nav>
    }
}
