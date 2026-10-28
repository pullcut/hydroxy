//! `<SignaturePad>`: draw a signature with a mouse, pen or finger, or type a
//! name instead. Strokes are smoothed and thin as the pen speeds up; the value
//! is SVG path data (or the typed name), ready to store or export with
//! [`to_svg`].

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use hydroxy_headless::signature_pad::{
    self as h, Mode, SignaturePadCommand, SignaturePadEvent, SignaturePadInput, SignaturePadMachine,
};
use hydroxy_statechart::Machine;

pub use hydroxy_headless::signature_pad::{SCRIPT_FONTS, SignatureValue, to_svg};

/// A signature drawn with a pointer, or typed as the keyboard alternative.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
#[component]
pub fn SignaturePad(
    /// Accessible name, such as "Signature".
    #[prop(into)]
    label: String,
    /// A signature to start with, such as one saved earlier.
    #[prop(optional)]
    default_value: Option<SignatureValue>,
    /// Block every change and dim the pad.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Show the signature with no way to change it.
    #[prop(optional, into)]
    readonly: MaybeProp<bool>,
    /// Form field name. Submits `drawn:<path data>` or `typed:<name>`.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the signature after each change; `None` when blank.
    #[prop(optional, into)]
    on_change: Option<Callback<Option<SignatureValue>>>,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let typed_id = format!("{root}-typed");
    let input = Memo::new(move |_| SignaturePadInput {
        default_value: default_value.clone(),
        disabled: disabled.get().unwrap_or(false),
        readonly: readonly.get().unwrap_or(false),
        ..SignaturePadInput::default()
    });
    let state = RwSignal::new(input.with_untracked(SignaturePadMachine::initial));
    let rect = StoredValue::new((0.0, 0.0, 0.0, 0.0));

    let send = move |event: SignaturePadEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| SignaturePadMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && state.with_untracked(|s| *s != next)
        {
            state.set(next);
        }
        for SignaturePadCommand::Change(value) in step.commands {
            if let Some(cb) = on_change {
                cb.run(value);
            }
        }
    };
    let point = move |ev: &web_sys::PointerEvent| {
        h::to_point(f64::from(ev.client_x()), f64::from(ev.client_y()), rect.get_value(), ev.time_stamp())
    };

    let locked = move || input.with(|i| i.disabled || i.readonly);
    let mode = move || state.with(|s| s.mode);
    let mode_button = move |m: Mode, text: &'static str| {
        view! {
            <button
                type="button"
                data-scope=h::SCOPE
                data-part="mode"
                aria-pressed=move || (mode() == m).to_string()
                data-selected=move || (mode() == m).then_some("")
                disabled=move || input.with(|i| i.disabled)
                on:click=move |_| send(SignaturePadEvent::SetMode(m))
            >
                {text}
            </button>
        }
    };
    let surface_label = format!("{label}, drawing area");

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=label.clone()
            data-mode=move || if mode() == Mode::Draw { "draw" } else { "type" }
            data-disabled=move || input.with(|i| i.disabled).then_some("")
            data-readonly=move || input.with(|i| i.readonly).then_some("")
            data-empty=move || state.with(h::SignaturePadState::is_empty).then_some("")
            on:keydown=move |ev| {
                if let Some(event) = h::root_key(&ev.key(), ev.ctrl_key() || ev.meta_key())
                    && mode() == Mode::Draw {
                        ev.prevent_default();
                        send(event);
                    }
            }
        >
            <Show when=move || !locked()>
                <div data-scope=h::SCOPE data-part="modes" role="group" aria-label="Signature method">
                    {mode_button(Mode::Draw, "Draw")}
                    {mode_button(Mode::Type, "Type")}
                </div>
            </Show>
            <Show
                when=move || mode() == Mode::Draw
                fallback=move || {
                    let typed_id = typed_id.clone();
                    view! {
                        <div data-scope=h::SCOPE data-part="typed">
                            <label data-scope=h::SCOPE data-part="typed-label" for=typed_id.clone()>"Type your full name"</label>
                            <input
                                id=typed_id
                                type="text"
                                data-scope=h::SCOPE
                                data-part="typed-input"
                                autocomplete="name"
                                spellcheck="false"
                                disabled=move || input.with(|i| i.disabled)
                                readonly=move || input.with(|i| i.readonly)
                                prop:value=move || state.with(|s| s.typed.clone())
                                on:input=move |ev| send(SignaturePadEvent::SetTyped(event_target_value(&ev)))
                            />
                            <div data-scope=h::SCOPE data-part="typed-preview" aria-hidden="true" style=format!("font-family: {SCRIPT_FONTS};")>
                                {move || state.with(|s| s.typed.clone())}
                            </div>
                        </div>
                    }
                }
            >
                <svg
                    data-scope=h::SCOPE
                    data-part="surface"
                    viewBox=h::view_box()
                    role="img"
                    aria-label=surface_label.clone()
                    on:pointerdown=move |ev: web_sys::PointerEvent| {
                        if !ev.is_primary() || (ev.pointer_type() == "mouse" && ev.button() != 0) || locked() {
                            return;
                        }
                        ev.prevent_default();
                        if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                            let _ = el.set_pointer_capture(ev.pointer_id());
                            let r = el.get_bounding_client_rect();
                            rect.set_value((r.left(), r.top(), r.width(), r.height()));
                        }
                        send(SignaturePadEvent::Start(point(&ev)));
                    }
                    on:pointermove=move |ev: web_sys::PointerEvent| {
                        if ev.is_primary() && state.with_untracked(|s| s.live.is_some()) {
                            send(SignaturePadEvent::Move(point(&ev)));
                        }
                    }
                    on:pointerup=move |_| send(SignaturePadEvent::End)
                    on:pointercancel=move |_| send(SignaturePadEvent::Cancel)
                >
                    <line data-scope=h::SCOPE data-part="guide" x1="24" x2="576" y1="160" y2="160" aria-hidden="true" />
                    <Show when=move || state.with(|s| s.strokes.is_empty() && s.live.is_none())>
                        <text data-scope=h::SCOPE data-part="placeholder" x="300" y="110" text-anchor="middle" aria-hidden="true">
                            {move || if locked() { "No signature" } else { "Sign here" }}
                        </text>
                    </Show>
                    <For
                        each=move || state.with(|s| s.strokes.iter().map(|k| k.d.clone()).enumerate().collect::<Vec<_>>())
                        key=|(i, d)| (*i, d.clone())
                        children=|(_, d)| view! { <path data-scope=h::SCOPE data-part="stroke" d=d /> }
                    />
                    <path data-scope=h::SCOPE data-part="stroke" data-live="" d=move || input.with(|i| state.with(|s| h::live_path(s, i))) />
                </svg>
            </Show>
            <Show when=move || !locked()>
                <div data-scope=h::SCOPE data-part="actions">
                    <Show when=move || mode() == Mode::Draw>
                        <button
                            type="button"
                            data-scope=h::SCOPE
                            data-part="undo"
                            title="Undo last stroke (Ctrl+Z)"
                            disabled=move || state.with(|s| s.strokes.is_empty())
                            on:click=move |_| send(SignaturePadEvent::Undo)
                        >
                            "Undo"
                        </button>
                    </Show>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="clear"
                        disabled=move || state.with(h::SignaturePadState::is_empty)
                        on:click=move |_| send(SignaturePadEvent::Clear)
                    >
                        "Clear"
                    </button>
                </div>
            </Show>
            <span data-scope=h::SCOPE data-part="live" aria-live="polite">
                {move || state.with(|s| if s.live.is_some() { String::new() } else { h::status(s) })}
            </span>
            {name.map(|name| view! { <input type="hidden" name=name prop:value=move || state.with(h::form_value) /> })}
        </div>
    }
}
