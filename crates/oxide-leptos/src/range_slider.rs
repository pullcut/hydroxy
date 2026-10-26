//! `<RangeSlider>`: pick a low and high value on one track. Two `role="slider"`
//! thumbs, each bounded by the other. Drag a thumb, press the track to move
//! the nearest thumb there, or use the arrow keys.

use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::range_slider::{
    self as h, RangeSliderCommand, RangeSliderEvent, RangeSliderInput, RangeSliderMachine,
};
use oxide_statechart::Machine;

use crate::focus::focus_id;

/// A labelled two-thumb range.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RangeSlider(
    /// Visible label. Also names the group.
    #[prop(into)]
    label: String,
    /// Lowest value.
    #[prop(default = 0.0)]
    min: f64,
    /// Highest value.
    #[prop(default = 100.0)]
    max: f64,
    /// Step between values.
    #[prop(default = 1.0)]
    step: f64,
    /// Smallest distance kept between the thumbs.
    #[prop(default = 0.0)]
    min_gap: f64,
    /// Controlled `[low, high]`. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<[f64; 2]>,
    /// Initial `[low, high]` when uncontrolled. Defaults to `[min, max]`.
    #[prop(optional, into)]
    default_value: Option<[f64; 2]>,
    /// Called with the new pair as either thumb moves.
    #[prop(optional, into)]
    on_value_change: Option<Callback<[f64; 2]>>,
    /// Turns a value into text, such as `"£1,200"`. Used for the readout and `aria-valuetext`.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Accessible names for the low and high thumbs.
    #[prop(default = ["Minimum".into(), "Maximum".into()])]
    thumb_labels: [String; 2],
    /// Block changes.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name. Both values are submitted under it, low first.
    #[prop(optional, into)]
    name: Option<String>,
) -> impl IntoView {
    let id = crate::id::next_id(h::SCOPE);
    let input = Memo::new(move |_| RangeSliderInput {
        min,
        max,
        step,
        min_gap,
        value: value.get(),
        default_value,
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(RangeSliderMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| RangeSliderMachine::sync(s, i)));
        state.set(synced);
    });

    let thumb_id = {
        let id = id.clone();
        move |thumb: usize| format!("{id}-thumb-{thumb}")
    };
    let send = {
        let thumb_id = thumb_id.clone();
        move |event: RangeSliderEvent| {
            let Some(step) = input
                .try_with_untracked(|i| state.try_with_untracked(|s| RangeSliderMachine::transition(s, event, i)))
                .flatten()
            else {
                return;
            };
            state.set(step.state);
            for command in step.commands {
                match command {
                    RangeSliderCommand::FocusThumb(thumb) => focus_id(&thumb_id(thumb)),
                    RangeSliderCommand::ValueChange { value, .. } => {
                        if let Some(cb) = on_value_change {
                            cb.run(value);
                        }
                    }
                }
            }
        }
    };

    let track = NodeRef::<Div>::new();
    let pointer_value = move |ev: &web_sys::PointerEvent| {
        let rect = track.get_untracked()?.get_bounding_client_rect();
        Some(input.with_untracked(|i| h::value_from_pointer(f64::from(ev.client_x()), rect.left(), rect.width(), i)))
    };
    let text = move |v: f64| format.map_or_else(|| v.to_string(), |f| f.run(v));
    let readout = move || state.with(|s| format!("{} – {}", text(s.value[0]), text(s.value[1])));

    let thumb = {
        let send = send.clone();
        move |thumb: usize, aria_label: String| {
            let attrs = Memo::new(move |_| input.with(|i| state.with(|s| h::thumb_attrs(s, i, thumb))));
            let send = send.clone();
            view! {
                <div
                    id=thumb_id(thumb)
                    role="slider"
                    tabindex=move || attrs.get().tabindex
                    aria-label=aria_label
                    aria-orientation="horizontal"
                    aria-valuemin=move || attrs.get().aria_valuemin
                    aria-valuemax=move || attrs.get().aria_valuemax
                    aria-valuenow=move || attrs.get().aria_valuenow
                    aria-valuetext=move || format.map(|f| f.run(attrs.get().aria_valuenow))
                    aria-disabled=move || attrs.get().aria_disabled.then_some("true")
                    data-scope=h::SCOPE
                    data-part="thumb"
                    data-thumb=move || attrs.get().data_thumb
                    data-dragging=move || attrs.get().dragging.then_some("")
                    style=move || format!("left: {}%;", attrs.get().percent)
                    on:keydown=move |ev| {
                        let event = input.with_untracked(|i| state.with_untracked(|s| h::keydown(&ev.key(), thumb, s, i)));
                        if let Some(event) = event {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                ></div>
            }
        }
    };
    let [low_label, high_label] = thumb_labels;
    let hidden = move || {
        let name = name.clone()?;
        let [low, high] = state.get().value;
        Some(view! {
            <input type="hidden" name=name.clone() value=low.to_string() />
            <input type="hidden" name=name value=high.to_string() />
        })
    };
    let label_id = format!("{id}-label");
    let labelledby = label_id.clone();

    view! {
        <div
            role="group"
            aria-labelledby=labelledby
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || input.with(|i| i.disabled).then_some("")
        >
            <div data-scope=h::SCOPE data-part="header">
                <span id=label_id data-scope=h::SCOPE data-part="label">{label}</span>
                <span data-scope=h::SCOPE data-part="value" aria-hidden="true">{readout}</span>
            </div>
            <div
                node_ref=track
                data-scope=h::SCOPE
                data-part="track"
                on:pointerdown={
                    let send = send.clone();
                    move |ev| {
                        if ev.button() != 0 || input.with_untracked(|i| i.disabled) {
                            return;
                        }
                        // Keep the thumb focus the machine asks for, not the browser's default.
                        ev.prevent_default();
                        if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                            let _ = el.set_pointer_capture(ev.pointer_id());
                        }
                        if let Some(v) = pointer_value(&ev) {
                            send(RangeSliderEvent::PointerDown(v));
                        }
                    }
                }
                on:pointermove={
                    let send = send.clone();
                    move |ev| {
                        if state.with_untracked(|s| s.dragging.is_some()) && let Some(v) = pointer_value(&ev) {
                            send(RangeSliderEvent::PointerMove(v));
                        }
                    }
                }
                on:pointerup={
                    let send = send.clone();
                    move |_| send(RangeSliderEvent::PointerUp)
                }
                on:pointercancel={
                    let send = send.clone();
                    move |_| send(RangeSliderEvent::PointerUp)
                }
            >
                <div
                    data-scope=h::SCOPE
                    data-part="range"
                    style=move || input.with(|i| state.with(|s| h::range_style(s, i)))
                ></div>
                {thumb(0, low_label)}
                {thumb(1, high_label)}
            </div>
            {hidden}
        </div>
    }
}
