//! `<Reveal>`: content that fades in once it scrolls into view. Visible with
//! no script, and shown straight away for people who prefer reduced motion.

use std::time::Duration;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use oxide_headless::reveal::{self as h, RevealCommand, RevealEvent, RevealInput, RevealMachine, RevealState};
use oxide_statechart::Machine;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

pub use oxide_headless::reveal::RevealReason;

/// The observer and the closure it calls, kept together so both live as long as the watch.
type Watch = (IntersectionObserver, Closure<dyn FnMut(Vec<IntersectionObserverEntry>)>);

fn prefers_reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

/// Whether `el` is already in the part of the viewport the observer would count.
fn in_view(el: &web_sys::Element) -> bool {
    let rect = el.get_bounding_client_rect();
    let height = window().inner_height().ok().and_then(|h| h.as_f64()).unwrap_or(0.0);
    rect.top() < height * 0.9 && rect.bottom() > 0.0
}

/// Wraps its children and reveals them when they scroll into view.
#[component]
pub fn Reveal(
    /// Start shown, with no animation.
    #[prop(optional)]
    default_shown: bool,
    /// Called once, with why the content was revealed.
    #[prop(optional, into)]
    on_reveal: Option<Callback<RevealReason>>,
    /// The content.
    children: Children,
) -> impl IntoView {
    let input = RwSignal::new(RevealInput { default_shown, reduced_motion: false, can_observe: true });
    let state = RwSignal::new(RevealMachine::initial(&input.get_untracked()));
    // The hidden look only applies once the element is being watched, so it never
    // hides content before the page is interactive.
    let armed = RwSignal::new(false);
    let id = h::root_id(&crate::id::next_id(h::SCOPE));
    let watch = StoredValue::new_local(None::<Watch>);

    let send_cell = StoredValue::new(None::<Callback<RevealEvent>>);
    let send = Callback::new({
        let id = id.clone();
        move |event: RevealEvent| {
            let Some(step) = input
                .try_get_untracked()
                .and_then(|i| state.try_with_untracked(|s| RevealMachine::transition(s, event, &i)))
            else {
                return;
            };
            if let Some(next) = step.state {
                state.set(next);
            }
            for command in step.commands {
                match command {
                    RevealCommand::Observe => {
                        let Some(el) = document().get_element_by_id(&id) else { continue };
                        if in_view(&el) {
                            if let Some(send) = send_cell.try_get_value().flatten() {
                                send.run(RevealEvent::Intersect);
                            }
                            continue;
                        }
                        armed.set(true);
                        let on_hit = Closure::<dyn FnMut(Vec<IntersectionObserverEntry>)>::new(move |entries: Vec<IntersectionObserverEntry>| {
                            if entries.iter().any(IntersectionObserverEntry::is_intersecting)
                                && let Some(send) = send_cell.try_get_value().flatten()
                            {
                                send.run(RevealEvent::Intersect);
                            }
                        });
                        let options = IntersectionObserverInit::new();
                        options.set_root_margin("0px 0px -10% 0px");
                        options.set_threshold(&0.05.into());
                        match IntersectionObserver::new_with_options(on_hit.as_ref().unchecked_ref(), &options) {
                            Ok(observer) => {
                                observer.observe(&el);
                                watch.set_value(Some((observer, on_hit)));
                            }
                            // No observer: show the content rather than hide it for good.
                            Err(_) => {
                                if let Some(send) = send_cell.try_get_value().flatten() {
                                    send.run(RevealEvent::Reveal);
                                }
                            }
                        }
                    }
                    RevealCommand::Unobserve => {
                        if let Some((observer, _)) = watch.try_update_value(Option::take).flatten() {
                            observer.disconnect();
                        }
                    }
                    RevealCommand::Wait { id: seq, ms } => {
                        let el_id = id.clone();
                        set_timeout(
                            move || {
                                // The backstop is for a watched element that never reports. One still
                                // below the fold stays hidden until it scrolls in.
                                if !document().get_element_by_id(&el_id).is_some_and(|el| in_view(&el)) {
                                    return;
                                }
                                if let Some(send) = send_cell.try_get_value().flatten() {
                                    send.run(RevealEvent::Elapsed(seq));
                                }
                            },
                            Duration::from_millis(ms.into()),
                        );
                    }
                    RevealCommand::Revealed(reason) => {
                        if let Some(cb) = on_reveal {
                            cb.run(reason);
                        }
                    }
                }
            }
        }
    });
    send_cell.set_value(Some(send));

    Effect::new(move |prev: Option<()>| {
        if prev.is_none() {
            input.update(|i| i.reduced_motion = prefers_reduced_motion());
            send.run(RevealEvent::Start);
        }
    });
    on_cleanup(move || send.run(RevealEvent::Stop));

    view! {
        <div
            id=id
            data-scope=h::SCOPE
            data-part="root"
            data-state=move || state.with(|s: &RevealState| h::data_state(s))
            data-armed=move || armed.get().then_some("")
        >
            {children()}
        </div>
    }
}
