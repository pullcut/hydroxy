use hydroxy_leptos::input::Input;
use hydroxy_leptos::wizard::{Wizard, WizardPanel};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (name, set_name) = signal(String::new());
    let (error, set_error) = signal(false);
    let (done, set_done) = signal(false);

    view! {
        <div style="max-width: 520px;">
            <Wizard
                steps=vec!["Workspace".to_string(), "Sources".to_string(), "Review".to_string()]
                // The first step needs a name before Next moves on.
                can_advance=move |step: usize| {
                    let ok = step != 0 || !name.get_untracked().trim().is_empty();
                    set_error.set(!ok);
                    ok
                }
                on_finish=move |()| set_done.set(true)
            >
                <WizardPanel index=0>
                    <Input
                        label="Workspace name"
                        value=name
                        on_input=move |v| set_name.set(v)
                        error=Signal::derive(move || error.get().then(|| "Give the workspace a name".to_string()))
                    />
                </WizardPanel>
                <WizardPanel index=1>
                    <p style="margin: 0; color: var(--text-2);">"Connect Drive, Notion or Slack. You can add more later."</p>
                </WizardPanel>
                <WizardPanel index=2>
                    <p style="margin: 0; color: var(--text-2);">
                        {move || if done.get() { format!("Created {}.", name.get()) } else { format!("Ready to create {}.", name.get()) }}
                    </p>
                </WizardPanel>
            </Wizard>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "wizard",
    name: "Wizard",
    group: "Navigation",
    summary: "A multi-step flow: a step list, one panel at a time and Back / Next. Each step can block Next until it is valid, and linear flows unlock steps in order.",
    demo,
    leptos: include_str!("wizard.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/wizard.rs"),
    parts: &[
        &["Wizard", "root", "Wrapper"],
        &["(inside)", "steps", "<ol aria-label> of steps"],
        &["(inside)", "step", "<li>. data-state=complete|current|upcoming"],
        &["(inside)", "trigger", "<button> that opens the step. aria-current=step, disabled while unreachable"],
        &["(inside)", "indicator", "Number, or a check once complete"],
        &["(inside)", "label", "Step name, with a screen reader \", done\" or \", current\""],
        &["WizardPanel", "panel", "<section> labelled by its step. Rendered only while current"],
        &["(inside)", "footer", "Back and Next (Finish on the last step), styled as .ox-button"],
    ],
    keys: &[
        &["Tab", "Step triggers, the panel's fields, then Back and Next"],
        &["Space / Enter", "Open the focused step, or press Back / Next"],
    ],
    props: &[
        &["steps", "Vec<String>", "Required. Step names, in order"],
        &["step", "Option<usize>", "Controlled step. Pair with on_step_change"],
        &["default_step", "usize", "Initial step when uncontrolled. Default 0"],
        &["linear", "bool", "Unlock steps in order. Off: any step at any time. Default true"],
        &["can_advance", "Callback<usize, bool>", "Asked with the current step before Next. false stays"],
        &["on_step_change", "Callback<usize>", "Called with the new step"],
        &["on_finish", "Callback<()>", "Next on the last step, after it passes"],
        &["label", "String", "Step list name. Default \"Progress\""],
        &["back_label / next_label / finish_label", "String", "Button text. Default Back, Next, Finish"],
        &["WizardPanel index", "usize", "The step a panel belongs to"],
    ],
};
