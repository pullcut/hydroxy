//! `Reveal`: content that fades in once it scrolls into view. Visible with
//! no script, and shown straight away for people who prefer reduced motion.

use dioxus::prelude::*;
use hydroxy_headless::reveal::{self as h, RevealCommand, RevealEvent, RevealInput, RevealMachine, RevealState};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::js;

pub use hydroxy_headless::reveal::RevealReason;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<RevealState>,
    input: Signal<RevealInput>,
    armed: Signal<bool>,
    on_reveal: Option<EventHandler<RevealReason>>,
}

/// Starts an `IntersectionObserver` on the element and reports back through
/// `dioxus.send`: `"armed"` once it is being watched, `"hit"` when it scrolls in.
/// An element already in view is reported at once, so it never flashes hidden.
fn observe_script(id: &str) -> String {
    format!(
        r#"const el = document.getElementById({id});
        if (!el) {{ dioxus.send("hit"); return; }}
        const rect = el.getBoundingClientRect();
        if (rect.top < innerHeight * 0.9 && rect.bottom > 0) {{ dioxus.send("hit"); return; }}
        try {{
          const io = new IntersectionObserver((entries) => {{
            if (entries.some((e) => e.isIntersecting)) dioxus.send("hit");
          }}, {{ rootMargin: "0px 0px -10% 0px", threshold: 0.05 }});
          io.observe(el);
          el.__oxReveal = io;
          dioxus.send("armed");
        }} catch (_) {{ dioxus.send("hit"); }}"#,
        id = crate::js::js_string(id)
    )
}

fn unobserve_script(id: &str) -> String {
    format!(
        "const el = document.getElementById({id}); if (el && el.__oxReveal) {{ el.__oxReveal.disconnect(); el.__oxReveal = null; }}",
        id = crate::js::js_string(id)
    )
}

impl Ctx {
    fn send(mut self, event: RevealEvent) {
        for command in drive::<RevealMachine>(self.state, self.input, event) {
            match command {
                RevealCommand::Observe => {
                    let id = self.id.read().clone();
                    spawn(async move {
                        let mut watch = document::eval(&observe_script(&id));
                        while let Ok(msg) = watch.recv::<String>().await {
                            if msg == "armed" {
                                self.armed.set(true);
                            } else {
                                self.send(RevealEvent::Intersect);
                                break;
                            }
                        }
                    });
                }
                RevealCommand::Unobserve => {
                    let _ = document::eval(&unobserve_script(&self.id.read()));
                }
                // The clock runs through eval, so it works on web and desktop alike.
                // The backstop is for a watched element that never reports; one still below the fold waits.
                RevealCommand::Wait { id: seq, ms } => {
                    let el_id = self.id.read().clone();
                    spawn(async move {
                        js::sleep(ms).await;
                        let check = js::call(
                            &[("id", el_id.as_str())],
                            "const el = document.getElementById(id); if (!el) return false; \
                             const rect = el.getBoundingClientRect(); \
                             return rect.top < innerHeight * 0.9 && rect.bottom > 0;",
                        );
                        if matches!(document::eval(&check).join::<bool>().await, Ok(true)) {
                            self.send(RevealEvent::Elapsed(seq));
                        }
                    });
                }
                RevealCommand::Revealed(reason) => {
                    if let Some(handler) = self.on_reveal {
                        handler.call(reason);
                    }
                }
            }
        }
    }
}

/// Wraps its children and reveals them when they scroll into view.
#[component]
pub fn Reveal(
    /// Start shown, with no animation.
    #[props(default)]
    default_shown: bool,
    /// Called once, with why the content was revealed.
    on_reveal: Option<EventHandler<RevealReason>>,
    /// The content.
    children: Element,
) -> Element {
    let input = use_signal(|| RevealInput { default_shown, reduced_motion: false, can_observe: true });
    let state = use_signal(|| RevealMachine::initial(&input.peek()));
    let armed = use_signal(|| false);
    let id = use_hook(|| h::root_id(&hydroxy_headless::next_id(h::SCOPE)));
    let ctx = Ctx { id: use_hook(|| CopyValue::new(id.clone())), state, input, armed, on_reveal };

    use_future(move || async move {
        let reduced = js::prefers_reduced_motion().await;
        let can_observe =
            document::eval("return 'IntersectionObserver' in window;").join::<bool>().await.unwrap_or(false);
        let mut input = ctx.input;
        input.set(RevealInput { default_shown, reduced_motion: reduced, can_observe });
        ctx.send(RevealEvent::Start);
    });
    use_drop(move || {
        let _ = document::eval(&unobserve_script(&ctx.id.read()));
    });

    rsx! {
        div {
            id,
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": h::data_state(&state.read()),
            "data-armed": armed().then_some(""),
            {children}
        }
    }
}
