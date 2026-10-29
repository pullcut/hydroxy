//! `RangeSlider`: pick a low and high value on one track. Two `role="slider"`
//! thumbs, each bounded by the other. Drag a thumb, press the track to move
//! the nearest thumb there, or use the arrow keys.
//!
//! Dioxus has no pointer capture on every renderer, so while dragging a
//! fixed full-window layer catches pointer moves instead. Works on web and desktop.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::range_slider::{
    self as h, RangeSliderCommand, RangeSliderEvent, RangeSliderInput, RangeSliderMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;

/// A labelled two-thumb range.
#[component]
pub fn RangeSlider(
    /// Visible label. Also names the group.
    label: String,
    /// Lowest value.
    #[props(default = 0.0)]
    min: f64,
    /// Highest value.
    #[props(default = 100.0)]
    max: f64,
    /// Step between values.
    #[props(default = 1.0)]
    step: f64,
    /// Smallest distance kept between the thumbs.
    #[props(default = 0.0)]
    min_gap: f64,
    /// Controlled `[low, high]`. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<[f64; 2]>>,
    /// Initial `[low, high]` when uncontrolled. Defaults to `[min, max]`.
    default_value: Option<[f64; 2]>,
    /// Called with the new pair as either thumb moves.
    on_value_change: Option<EventHandler<[f64; 2]>>,
    /// Turns a value into text, such as `"£1,200"`. Used for the readout and `aria-valuetext`.
    format: Option<Callback<f64, String>>,
    /// Accessible names for the low and high thumbs.
    #[props(default = ["Minimum".to_string(), "Maximum".to_string()])]
    thumb_labels: [String; 2],
    /// Block changes.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name. Both values are submitted under it, low first.
    name: Option<String>,
) -> Element {
    let id = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let input = use_memo(move || RangeSliderInput {
        min,
        max,
        step,
        min_gap,
        value: value(),
        default_value,
        disabled: disabled(),
    });
    let mut state = use_signal(|| RangeSliderMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = RangeSliderMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let thumbs = use_hook(FocusMap::new);
    let mut track = use_signal(|| None::<Rc<MountedData>>);
    // Track position, read once per press and reused while dragging.
    let mut rect = use_signal(|| (0.0_f64, 0.0_f64));
    // The pointer is down. A press can end before the async rect read returns.
    let mut pressed = use_signal(|| false);

    let send = move |event: RangeSliderEvent| {
        for command in drive::<RangeSliderMachine>(state, input, event) {
            match command {
                RangeSliderCommand::FocusThumb(thumb) => thumbs.focus(&thumb.to_string()),
                RangeSliderCommand::ValueChange { value, .. } => {
                    if let Some(handler) = on_value_change {
                        handler.call(value);
                    }
                }
            }
        }
    };
    let pointer_value = move |x: f64| {
        let (left, width) = rect();
        h::value_from_pointer(x, left, width, &input.peek())
    };
    let mut release = move || {
        pressed.set(false);
        send(RangeSliderEvent::PointerUp);
    };

    let s = state.read();
    let i = input.read();
    let text = |v: f64| format.map_or_else(|| v.to_string(), |f| f.call(v));
    let readout = format!("{} – {}", text(s.value.value[0]), text(s.value.value[1]));
    let range_style = h::range_style(&s, &i);
    let dragging = s.dragging.is_some();
    let [low, high] = s.value.value;
    let label_id = h::label_id(&id);
    let thumb_attrs = [h::thumb_attrs(&s, &i, 0), h::thumb_attrs(&s, &i, 1)];

    rsx! {
        div {
            role: "group",
            "aria-labelledby": label_id.clone(),
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": i.disabled.then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "header",
                span { id: label_id, "data-scope": h::SCOPE, "data-part": "label", {label} }
                span { "data-scope": h::SCOPE, "data-part": "value", "aria-hidden": "true", {readout} }
            }
            div {
                "data-scope": h::SCOPE,
                "data-part": "track",
                onmounted: move |e| track.set(Some(e.data())),
                onpointerdown: move |e| {
                    if input.peek().disabled {
                        return;
                    }
                    // Keep the thumb focus the machine asks for, not the browser's default.
                    e.prevent_default();
                    pressed.set(true);
                    let x = e.client_coordinates().x;
                    let Some(el) = track.peek().clone() else { return };
                    spawn(async move {
                        if let Ok(r) = el.get_client_rect().await {
                            rect.set((r.origin.x, r.size.width));
                        }
                        send(RangeSliderEvent::PointerDown(pointer_value(x)));
                        if !pressed() {
                            send(RangeSliderEvent::PointerUp);
                        }
                    });
                },
                onpointerup: move |_| release(),
                div { "data-scope": h::SCOPE, "data-part": "range", style: range_style }
                for (thumb, (a, aria_label)) in thumb_attrs.into_iter().zip(thumb_labels).enumerate() {
                    div {
                        key: "{thumb}",
                        id: h::thumb_id(&id, thumb),
                        role: a.role,
                        tabindex: a.tabindex,
                        "aria-label": aria_label,
                        "aria-orientation": "horizontal",
                        "aria-valuemin": a.aria_valuemin,
                        "aria-valuemax": a.aria_valuemax,
                        "aria-valuenow": a.aria_valuenow,
                        "aria-valuetext": format.map(|f| f.call(a.aria_valuenow)),
                        "aria-disabled": a.aria_disabled.then_some("true"),
                        "data-scope": h::SCOPE,
                        "data-part": "thumb",
                        "data-thumb": a.data_thumb,
                        "data-dragging": a.dragging.then_some(""),
                        style: "left: {a.percent}%;",
                        onmounted: move |e| thumbs.register(thumb.to_string(), e.data()),
                        onkeydown: move |e| {
                            let event = h::keydown(&e.key().to_string(), thumb, &state.peek(), &input.peek());
                            if let Some(event) = event {
                                e.prevent_default();
                                send(event);
                            }
                        },
                    }
                }
            }
            if dragging {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| send(RangeSliderEvent::PointerMove(pointer_value(e.client_coordinates().x))),
                    onpointerup: move |_| release(),
                    onpointercancel: move |_| release(),
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: name.clone(), value: "{low}" }
                input { r#type: "hidden", name, value: "{high}" }
            }
        }
    }
}
