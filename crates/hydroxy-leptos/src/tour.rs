//! `<Tour>`: onboarding coachmarks. Each step dims the page around its target,
//! scrolls it into view and explains it in a popover beside it. Arrow keys
//! step through, Escape skips, and focus goes back where it was when the tour
//! ends. A step whose target is missing shows centred instead.

use hydroxy_headless::tour::{self as h, TourCommand, TourEnd, TourEvent, TourInput, TourMachine, TourState};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::drive::drive;
use crate::focus::focus_id;
use crate::js;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::overlay::Side;
pub use hydroxy_headless::tour::{TourEnd as TourOutcome, TourStep};

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<TourState>,
    input: Memo<TourInput>,
    on_open_change: Option<Callback<bool>>,
    on_step_change: Option<Callback<usize>>,
    on_end: Option<Callback<TourEnd>>,
}

impl Ctx {
    fn send(self, event: TourEvent) {
        for command in drive::<TourMachine, _>(self.state, self.input, event) {
            match command {
                TourCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
                TourCommand::StepChange(i) => {
                    if let Some(cb) = self.on_step_change {
                        cb.run(i);
                    }
                }
                TourCommand::End(how) => {
                    if let Some(cb) = self.on_end {
                        cb.run(how);
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
    #[prop(into)]
    steps: Signal<Vec<TourStep>>,
    /// Controlled running state. Pair with `on_open_change`; setting it to `true` starts at step 1.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Start running on mount, when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Called when the tour starts or ends (or a change is requested).
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// Called with each new step index.
    #[prop(optional, into)]
    on_step_change: Option<Callback<usize>>,
    /// Called once when the tour ends: completed, or skipped at a step.
    #[prop(optional, into)]
    on_end: Option<Callback<TourEnd>>,
) -> impl IntoView {
    let input = Memo::new(move |_| TourInput { count: steps.with(Vec::len), open: open.get(), default_open });
    let state = RwSignal::new(input.with_untracked(TourMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| TourMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, on_open_change, on_step_change, on_end };
    let root = crate::id::next_id(h::SCOPE);
    let ids = StoredValue::new(h::ids(&root));
    let root = StoredValue::new(root);
    let found = RwSignal::new(false);
    let was_open = StoredValue::new(false);

    // Show, aim and focus for each step; tidy up and hand focus back at the end.
    Effect::new(move |_| {
        let (open, index) = state.with(|s| (s.open.value, s.index));
        let target = steps.with(|s| s.get(index).and_then(|st| st.target.clone())).unwrap_or_default();
        let ids = ids.get_value();
        let r = root.get_value();
        if open {
            let hit = js::call(
                "root, target, anchor",
                h::SHOW_STEP_JS,
                &[r.into(), target.into(), ids.anchor.clone().into()],
            );
            found.set(hit.as_bool().unwrap_or(false));
            show_popover(&ids.spotlight);
            show_popover(&ids.popover);
            focus_id(&ids.popover);
            was_open.set_value(true);
        } else if was_open.get_value() {
            was_open.set_value(false);
            hide_popover(&ids.popover);
            hide_popover(&ids.spotlight);
            js::call("root", h::END_JS, &[JsValue::from(r)]);
        }
    });
    on_cleanup(move || {
        if let (Some(true), Some(r)) = (was_open.try_get_value(), root.try_get_value()) {
            js::call("root", h::END_JS, &[JsValue::from(r)]);
        }
    });

    let step = move || state.with(|s| steps.with(|list| list.get(s.index).cloned()));
    let placement = move || step().map_or(h::DEFAULT_PLACEMENT, |s| s.placement);
    let first = move || state.with(|s| s.index == 0);
    let ids_v = ids.get_value();

    view! {
        <div
            id=ids_v.spotlight.clone()
            popover="manual"
            aria-hidden="true"
            data-scope=h::SCOPE
            data-part="spotlight"
            data-missing=move || (!found.get()).then_some("")
            style=h::spotlight_style(&ids_v)
        ></div>
        <div
            id=ids_v.popover.clone()
            popover="manual"
            role="dialog"
            aria-modal="false"
            aria-labelledby=ids_v.title.clone()
            aria-describedby=ids_v.body.clone()
            tabindex="-1"
            data-overlay=""
            data-scope=h::SCOPE
            data-part="popover"
            data-side=move || placement().side_str()
            data-centred=move || (!found.get()).then_some("")
            style=move || h::popover_style(&ids.get_value(), placement(), found.get())
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {move || step().map(|s| view! {
                <div data-scope=h::SCOPE data-part="step">
                    <h2 id=ids.get_value().title data-scope=h::SCOPE data-part="title">{s.title}</h2>
                    <p id=ids.get_value().body data-scope=h::SCOPE data-part="body">{s.body}</p>
                </div>
            })}
            <div data-scope=h::SCOPE data-part="footer">
                <span data-scope=h::SCOPE data-part="progress">
                    {move || state.with(|s| h::progress(s.index, input.with(|i| i.count)))}
                </span>
                <button
                    type="button"
                    class=BUTTON_CLASS
                    data-variant=ButtonVariant::Ghost.as_str()
                    data-size="sm"
                    data-scope=h::SCOPE
                    data-part="skip"
                    on:click=move |_| ctx.send(TourEvent::Skip)
                >
                    "Skip"
                </button>
                <button
                    type="button"
                    class=BUTTON_CLASS
                    data-variant=ButtonVariant::Secondary.as_str()
                    data-size="sm"
                    data-scope=h::SCOPE
                    data-part="prev"
                    hidden=first
                    on:click=move |_| ctx.send(TourEvent::Prev)
                >
                    "Back"
                </button>
                <button
                    type="button"
                    class=BUTTON_CLASS
                    data-variant=ButtonVariant::Primary.as_str()
                    data-size="sm"
                    data-scope=h::SCOPE
                    data-part="next"
                    on:click=move |_| ctx.send(TourEvent::Next)
                >
                    {move || state.with(|s| input.with(|i| h::next_label(s, i)))}
                </button>
            </div>
        </div>
    }
}
