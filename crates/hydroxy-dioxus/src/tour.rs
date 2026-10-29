//! `Tour`: onboarding coachmarks. Each step dims the page around its target,
//! scrolls it into view and explains it in a popover beside it. Arrow keys
//! step through, Escape skips, and focus goes back where it was when the tour
//! ends. A step whose target is missing shows centred instead.

use dioxus::prelude::*;
use hydroxy_headless::tour::{self as h, TourCommand, TourEnd, TourEvent, TourInput, TourMachine, TourState};
use hydroxy_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::drive::drive;
use crate::focus::focus_id;
use crate::js;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::overlay::Side;
pub use hydroxy_headless::tour::{TourEnd as TourOutcome, TourStep};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<TourState>,
    input: Memo<TourInput>,
    on_open_change: Option<EventHandler<bool>>,
    on_step_change: Option<EventHandler<usize>>,
    on_end: Option<EventHandler<TourEnd>>,
}

impl Ctx {
    fn send(self, event: TourEvent) {
        for command in drive::<TourMachine>(self.state, self.input, event) {
            match command {
                TourCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
                TourCommand::StepChange(i) => {
                    if let Some(handler) = self.on_step_change {
                        handler.call(i);
                    }
                }
                TourCommand::End(how) => {
                    if let Some(handler) = self.on_end {
                        handler.call(how);
                    }
                }
            }
        }
    }
}

/// A guided tour over the page.
#[component]
pub fn Tour(
    /// The steps, in order.
    steps: ReadSignal<Vec<TourStep>>,
    /// Controlled running state. Pair with `on_open_change`; setting it to `true` starts at step 1.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Start running on mount, when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Called when the tour starts or ends (or a change is requested).
    on_open_change: Option<EventHandler<bool>>,
    /// Called with each new step index.
    on_step_change: Option<EventHandler<usize>>,
    /// Called once when the tour ends: completed, or skipped at a step.
    on_end: Option<EventHandler<TourEnd>>,
) -> Element {
    let input = use_memo(move || TourInput { count: steps.read().len(), open: open(), default_open });
    let mut state = use_signal(|| TourMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = TourMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let ctx = use_hook(|| Ctx { state, input, on_open_change, on_step_change, on_end });
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let mut found = use_signal(|| false);
    let mut was_open = use_signal(|| false);

    // Show, aim and focus for each step; tidy up and hand focus back at the end.
    let effect_root = root.clone();
    use_effect(move || {
        let s = state.read().clone();
        let target = steps.read().get(s.index).and_then(|st| st.target.clone()).unwrap_or_default();
        let ids = h::ids(&effect_root);
        let r = effect_root.clone();
        if s.open.value {
            let script = js::call(&[("root", &r), ("target", &target), ("anchor", &ids.anchor)], h::SHOW_STEP_JS);
            spawn(async move {
                let hit = document::eval(&script).join::<bool>().await.unwrap_or(false);
                found.set(hit);
                show_popover(&ids.spotlight);
                show_popover(&ids.popover);
                focus_id(&ids.popover);
            });
            if !*was_open.peek() {
                was_open.set(true);
            }
        } else if *was_open.peek() {
            was_open.set(false);
            hide_popover(&ids.popover);
            hide_popover(&ids.spotlight);
            js::run(&[("root", &r)], h::END_JS);
        }
    });
    let drop_root = root.clone();
    use_drop(move || {
        if *was_open.peek() {
            js::run(&[("root", &drop_root)], h::END_JS);
        }
    });

    let s = state.read().clone();
    let inp = input.read().clone();
    let step = steps.read().get(s.index).cloned();
    let placement = step.as_ref().map_or(h::DEFAULT_PLACEMENT, |st| st.placement);
    let hit = found();

    rsx! {
        div {
            id: ids.spotlight.clone(),
            "popover": "manual",
            "aria-hidden": "true",
            "data-scope": h::SCOPE,
            "data-part": "spotlight",
            "data-missing": (!hit).then_some(""),
            style: h::spotlight_style(&ids),
        }
        div {
            id: ids.popover.clone(),
            "popover": "manual",
            role: "dialog",
            "aria-modal": "false",
            "aria-labelledby": ids.title.clone(),
            "aria-describedby": ids.body.clone(),
            tabindex: "-1",
            "data-overlay": "",
            "data-scope": h::SCOPE,
            "data-part": "popover",
            "data-side": placement.side_str(),
            "data-centred": (!hit).then_some(""),
            style: h::popover_style(&ids, placement, hit),
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            if let Some(st) = step {
                div { key: "{s.index}", "data-scope": h::SCOPE, "data-part": "step",
                    h2 { id: ids.title.clone(), "data-scope": h::SCOPE, "data-part": "title", "{st.title}" }
                    p { id: ids.body.clone(), "data-scope": h::SCOPE, "data-part": "body", "{st.body}" }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "footer",
                span { "data-scope": h::SCOPE, "data-part": "progress", {h::progress(s.index, inp.count)} }
                button {
                    r#type: "button",
                    class: BUTTON_CLASS,
                    "data-variant": ButtonVariant::Ghost.as_str(),
                    "data-size": "sm",
                    "data-scope": h::SCOPE,
                    "data-part": "skip",
                    onclick: move |_| ctx.send(TourEvent::Skip),
                    "Skip"
                }
                button {
                    r#type: "button",
                    class: BUTTON_CLASS,
                    "data-variant": ButtonVariant::Secondary.as_str(),
                    "data-size": "sm",
                    "data-scope": h::SCOPE,
                    "data-part": "prev",
                    hidden: s.index == 0,
                    onclick: move |_| ctx.send(TourEvent::Prev),
                    "Back"
                }
                button {
                    r#type: "button",
                    class: BUTTON_CLASS,
                    "data-variant": ButtonVariant::Primary.as_str(),
                    "data-size": "sm",
                    "data-scope": h::SCOPE,
                    "data-part": "next",
                    onclick: move |_| ctx.send(TourEvent::Next),
                    {h::next_label(&s, &inp)}
                }
            }
        }
    }
}
