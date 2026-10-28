//! `SignaturePad`: draw a signature with a mouse, pen or finger, or type a
//! name instead. Strokes are smoothed and thin as the pen speeds up; the value
//! is SVG path data (or the typed name), ready to store or export with
//! [`to_svg`].
//!
//! While drawing, a fixed full-window layer catches pointer moves, since not
//! every Dioxus renderer has pointer capture.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::signature_pad::{
    self as h, Mode, SignaturePadCommand, SignaturePadEvent, SignaturePadInput, SignaturePadMachine,
};
use hydroxy_statechart::Machine;

pub use hydroxy_headless::signature_pad::{SCRIPT_FONTS, SignatureValue, to_svg};

/// A signature drawn with a pointer, or typed as the keyboard alternative.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn SignaturePad(
    /// Accessible name, such as "Signature".
    label: String,
    /// A signature to start with, such as one saved earlier.
    #[props(default)]
    default_value: Option<SignatureValue>,
    /// Block every change and dim the pad.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Show the signature with no way to change it.
    #[props(default)]
    readonly: ReadSignal<bool>,
    /// Form field name. Submits `drawn:<path data>` or `typed:<name>`.
    #[props(default)]
    name: Option<String>,
    /// Called with the signature after each change; `None` when blank.
    on_change: Option<EventHandler<Option<SignatureValue>>>,
) -> Element {
    let input = use_memo(move || SignaturePadInput {
        default_value: default_value.clone(),
        disabled: disabled(),
        readonly: readonly(),
        ..SignaturePadInput::default()
    });
    let state = use_signal(|| SignaturePadMachine::initial(&input.peek()));
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let typed_id = format!("{root}-typed");
    let mut surface = use_signal(|| None::<Rc<MountedData>>);
    let mut rect = use_signal(|| (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64));

    let send = move |event: SignaturePadEvent| {
        let mut state = state;
        let step = SignaturePadMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for SignaturePadCommand::Change(value) in step.commands {
            if let Some(handler) = on_change {
                handler.call(value);
            }
        }
    };
    // Dioxus pointer events carry no time stamp; samples arrive about once a
    // frame, which the machine assumes when times are equal.
    let point = move |x: f64, y: f64| h::to_point(x, y, rect(), 0.0);

    let i = input.read();
    let s = state.read();
    let locked = i.disabled || i.readonly;
    let is_disabled = i.disabled;
    let is_readonly = i.readonly;
    let mode = s.mode;
    let strokes: Vec<String> = s.strokes.iter().map(|k| k.d.clone()).collect();
    let live = h::live_path(&s, &i);
    let drawing = s.live.is_some();
    let empty = s.is_empty();
    let typed = s.typed.clone();
    let status = if drawing { String::new() } else { h::status(&s) };
    let form_value = h::form_value(&s);
    drop((s, i));
    let surface_label = format!("{label}, drawing area");

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": label,
            "data-mode": if mode == Mode::Draw { "draw" } else { "type" },
            "data-disabled": is_disabled.then_some(""),
            "data-readonly": is_readonly.then_some(""),
            "data-empty": empty.then_some(""),
            onkeydown: move |e| {
                let m = e.modifiers();
                if let Some(event) = h::root_key(&e.key().to_string(), m.ctrl() || m.meta())
                    && mode == Mode::Draw {
                        e.prevent_default();
                        send(event);
                    }
            },
            if !locked {
                div { "data-scope": h::SCOPE, "data-part": "modes", role: "group", "aria-label": "Signature method",
                    for (m , text) in [(Mode::Draw, "Draw"), (Mode::Type, "Type")] {
                        button {
                            key: "{text}",
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "mode",
                            "aria-pressed": (mode == m).to_string(),
                            "data-selected": (mode == m).then_some(""),
                            disabled: is_disabled,
                            onclick: move |_| send(SignaturePadEvent::SetMode(m)),
                            "{text}"
                        }
                    }
                }
            }
            if mode == Mode::Draw {
                svg {
                    "data-scope": h::SCOPE,
                    "data-part": "surface",
                    view_box: h::view_box(),
                    role: "img",
                    "aria-label": surface_label,
                    onmounted: move |e| surface.set(Some(e.data())),
                    onpointerdown: move |e| {
                        if locked {
                            return;
                        }
                        e.prevent_default();
                        let p = e.client_coordinates();
                        let Some(el) = surface.peek().clone() else { return };
                        spawn(async move {
                            if let Ok(r) = el.get_client_rect().await {
                                rect.set((r.origin.x, r.origin.y, r.size.width, r.size.height));
                                send(SignaturePadEvent::Start(point(p.x, p.y)));
                            }
                        });
                    },
                    line { "data-scope": h::SCOPE, "data-part": "guide", x1: "24", x2: "576", y1: "160", y2: "160", "aria-hidden": "true" }
                    if strokes.is_empty() && !drawing {
                        text { "data-scope": h::SCOPE, "data-part": "placeholder", x: "300", y: "110", text_anchor: "middle", "aria-hidden": "true",
                            if locked { "No signature" } else { "Sign here" }
                        }
                    }
                    for (n , d) in strokes.into_iter().enumerate() {
                        path { key: "{n}-{d.len()}", "data-scope": h::SCOPE, "data-part": "stroke", d }
                    }
                    path { "data-scope": h::SCOPE, "data-part": "stroke", "data-live": "", d: live }
                }
            } else {
                div { "data-scope": h::SCOPE, "data-part": "typed",
                    label { "data-scope": h::SCOPE, "data-part": "typed-label", r#for: typed_id.clone(), "Type your full name" }
                    input {
                        id: typed_id,
                        r#type: "text",
                        "data-scope": h::SCOPE,
                        "data-part": "typed-input",
                        autocomplete: "name",
                        spellcheck: "false",
                        disabled: is_disabled,
                        readonly: is_readonly,
                        value: typed.clone(),
                        oninput: move |e| send(SignaturePadEvent::SetTyped(e.value())),
                    }
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "typed-preview",
                        "aria-hidden": "true",
                        style: "font-family: {SCRIPT_FONTS};",
                        "{typed}"
                    }
                }
            }
            if !locked {
                div { "data-scope": h::SCOPE, "data-part": "actions",
                    if mode == Mode::Draw {
                        button {
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "undo",
                            title: "Undo last stroke (Ctrl+Z)",
                            disabled: empty,
                            onclick: move |_| send(SignaturePadEvent::Undo),
                            "Undo"
                        }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "clear",
                        disabled: empty,
                        onclick: move |_| send(SignaturePadEvent::Clear),
                        "Clear"
                    }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "live", "aria-live": "polite", "{status}" }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: form_value }
            }
            if drawing {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| {
                        let p = e.client_coordinates();
                        send(SignaturePadEvent::Move(point(p.x, p.y)));
                    },
                    onpointerup: move |_| send(SignaturePadEvent::End),
                    onpointercancel: move |_| send(SignaturePadEvent::Cancel),
                }
            }
        }
    }
}
