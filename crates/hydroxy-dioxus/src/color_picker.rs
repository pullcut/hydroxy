//! `ColorPicker`: a saturation/brightness area, a hue slider, an optional alpha
//! slider, a hex field and swatches. Drag the thumbs or focus one and use the
//! arrow keys (Shift for bigger steps).
//!
//! Dioxus has no pointer capture on every renderer, so while dragging a fixed
//! full-window layer catches pointer moves instead. Works on web and desktop.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::color_picker::{
    self as h, ColorPickerCommand, ColorPickerEvent, ColorPickerInput, ColorPickerMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;
use crate::modifiers::mods;

pub use hydroxy_headless::color_picker::{DEFAULT_SWATCHES, Rgba};

/// Which track a pointer is on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Track {
    Area,
    Hue,
    Alpha,
}

impl Track {
    const fn key(self) -> &'static str {
        match self {
            Self::Area => "area",
            Self::Hue => "hue",
            Self::Alpha => "alpha",
        }
    }
}

/// A colour picked with pointer, keyboard or hex text.
#[component]
pub fn ColorPicker(
    /// Accessible name for the whole picker.
    label: String,
    /// Controlled colour. Pair with `on_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<Rgba>>,
    /// Initial colour when uncontrolled. Default black.
    #[props(default)]
    default_value: Option<Rgba>,
    /// Offer an alpha slider and `#rrggbbaa` hex.
    #[props(default)]
    alpha: bool,
    /// Preset colours below the hex field. Default a palette of eight; pass an empty list for none.
    #[props(default)]
    swatches: Option<Vec<Rgba>>,
    /// Block changes.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name. Submits the hex text.
    #[props(default)]
    name: Option<String>,
    /// Called with the colour as it changes.
    on_change: Option<EventHandler<Rgba>>,
) -> Element {
    let input = use_memo(move || ColorPickerInput { value: value(), default_value, alpha, disabled: disabled() });
    let mut state = use_signal(|| ColorPickerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ColorPickerMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let thumbs = use_hook(FocusMap::new);
    let mut tracks = use_signal(|| [None::<Rc<MountedData>>, None, None]);
    // Track rectangle (left, top, width, height), read once per press and reused while dragging.
    let mut rect = use_signal(|| (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64));
    let mut dragging = use_signal(|| None::<Track>);

    let send = move |event: ColorPickerEvent| {
        for ColorPickerCommand::ValueChange(color) in drive::<ColorPickerMachine>(state, input, event) {
            if let Some(handler) = on_change {
                handler.call(color);
            }
        }
    };
    let to_event = move |track: Track, x: f64, y: f64| {
        let (left, top, width, height) = rect();
        match track {
            Track::Area => h::area_event(x, y, left, top, width, height),
            Track::Hue => h::hue_event(x, left, width),
            Track::Alpha => h::alpha_event(x, left, width),
        }
    };
    let mut press = move |track: Track, x: f64, y: f64| {
        if input.peek().disabled {
            return;
        }
        let index = track as usize;
        let Some(el) = tracks.peek()[index].clone() else { return };
        rect.set((0.0, 0.0, 0.0, 0.0));
        dragging.set(Some(track));
        thumbs.focus(track.key());
        spawn(async move {
            if let Ok(r) = el.get_client_rect().await {
                rect.set((r.origin.x, r.origin.y, r.size.width, r.size.height));
                send(to_event(track, x, y));
            }
        });
    };

    let s = state.read();
    let i = input.read();
    let style = h::root_style(&s, &i);
    let current = s.rgba(&i);
    let draft = s.draft.clone();
    let invalid = s.is_invalid();
    let form_value = h::form_value(&s, &i);
    let (area_thumb, hue_thumb, alpha_thumb) =
        (h::area_thumb_style(&s), h::hue_thumb_style(&s), h::alpha_thumb_style(&s));
    let (area_text, area_now) = (h::area_valuetext(&s), h::percent(s.hsv.s));
    let (hue_now, alpha_now) = (h::hue_degrees(&s), h::percent(s.alpha));
    let (hue_text, alpha_text) = (h::hue_valuetext(&s), h::alpha_valuetext(&s));
    let is_disabled = i.disabled;
    drop((s, i));
    let swatches = swatches.unwrap_or_else(|| h::DEFAULT_SWATCHES.to_vec());
    let tab = if is_disabled { "-1" } else { "0" };

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": label,
            style,
            "data-disabled": is_disabled.then_some(""),
            div {
                "data-scope": h::SCOPE,
                "data-part": "area",
                onmounted: move |e| tracks.write()[0] = Some(e.data()),
                onpointerdown: move |e| {
                    e.prevent_default();
                    let p = e.client_coordinates();
                    press(Track::Area, p.x, p.y);
                },
                div {
                    id: ids.area.clone(),
                    role: "slider",
                    tabindex: tab,
                    "aria-label": "Colour",
                    "aria-roledescription": "2D slider",
                    "aria-valuemin": "0",
                    "aria-valuemax": "100",
                    "aria-valuenow": area_now,
                    "aria-valuetext": area_text,
                    "aria-disabled": is_disabled.then_some("true"),
                    "data-scope": h::SCOPE,
                    "data-part": "area-thumb",
                    style: area_thumb,
                    onmounted: move |e| thumbs.register(Track::Area.key().to_string(), e.data()),
                    onkeydown: move |e| {
                        if let Some(event) = h::area_keydown(&e.key().to_string(), mods(e.modifiers())) {
                            e.prevent_default();
                            send(event);
                        }
                    },
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "controls",
                div { "data-scope": h::SCOPE, "data-part": "preview", "aria-hidden": "true" }
                div { "data-scope": h::SCOPE, "data-part": "sliders",
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "hue",
                        onmounted: move |e| tracks.write()[1] = Some(e.data()),
                        onpointerdown: move |e| {
                            e.prevent_default();
                            let p = e.client_coordinates();
                            press(Track::Hue, p.x, p.y);
                        },
                        div {
                            id: ids.hue.clone(),
                            role: "slider",
                            tabindex: tab,
                            "aria-label": "Hue",
                            "aria-valuemin": "0",
                            "aria-valuemax": "360",
                            "aria-valuenow": hue_now,
                            "aria-valuetext": hue_text,
                            "aria-orientation": "horizontal",
                            "aria-disabled": is_disabled.then_some("true"),
                            "data-scope": h::SCOPE,
                            "data-part": "hue-thumb",
                            style: hue_thumb,
                            onmounted: move |e| thumbs.register(Track::Hue.key().to_string(), e.data()),
                            onkeydown: move |e| {
                                if let Some(event) = h::hue_keydown(&e.key().to_string(), mods(e.modifiers())) {
                                    e.prevent_default();
                                    send(event);
                                }
                            },
                        }
                    }
                    if alpha {
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "alpha",
                            onmounted: move |e| tracks.write()[2] = Some(e.data()),
                            onpointerdown: move |e| {
                                e.prevent_default();
                                let p = e.client_coordinates();
                                press(Track::Alpha, p.x, p.y);
                            },
                            div {
                                id: ids.alpha.clone(),
                                role: "slider",
                                tabindex: tab,
                                "aria-label": "Opacity",
                                "aria-valuemin": "0",
                                "aria-valuemax": "100",
                                "aria-valuenow": alpha_now,
                                "aria-valuetext": alpha_text,
                                "aria-orientation": "horizontal",
                                "aria-disabled": is_disabled.then_some("true"),
                                "data-scope": h::SCOPE,
                                "data-part": "alpha-thumb",
                                style: alpha_thumb,
                                onmounted: move |e| thumbs.register(Track::Alpha.key().to_string(), e.data()),
                                onkeydown: move |e| {
                                    if let Some(event) = h::alpha_keydown(&e.key().to_string(), mods(e.modifiers())) {
                                        e.prevent_default();
                                        send(event);
                                    }
                                },
                            }
                        }
                    }
                }
                input {
                    r#type: "text",
                    id: ids.hex,
                    "data-scope": h::SCOPE,
                    "data-part": "hex",
                    "aria-label": "Hex colour",
                    autocomplete: "off",
                    spellcheck: "false",
                    maxlength: "9",
                    disabled: is_disabled,
                    "aria-invalid": invalid.then_some("true"),
                    value: draft,
                    oninput: move |e| send(ColorPickerEvent::HexInput(e.value())),
                    onblur: move |_| send(ColorPickerEvent::HexCommit),
                    onkeydown: move |e| {
                        if let Some(event) = h::hex_keydown(&e.key().to_string()) {
                            e.prevent_default();
                            send(event);
                        }
                    },
                }
            }
            if !swatches.is_empty() {
                div { "data-scope": h::SCOPE, "data-part": "swatches", role: "group", "aria-label": "Swatches",
                    for color in swatches {
                        {
                            let hex = color.hex(false);
                            let chosen = (current.r, current.g, current.b) == (color.r, color.g, color.b);
                            rsx! {
                                button {
                                    key: "{hex}",
                                    r#type: "button",
                                    "data-scope": h::SCOPE,
                                    "data-part": "swatch",
                                    "aria-label": hex.clone(),
                                    "aria-pressed": chosen.to_string(),
                                    "data-selected": chosen.then_some(""),
                                    style: "--swatch: {hex};",
                                    disabled: is_disabled,
                                    onclick: move |_| send(ColorPickerEvent::SetColor(Rgba { a: current.a, ..color })),
                                }
                            }
                        }
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: form_value }
            }
            if let Some(track) = dragging() {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| {
                        if rect().2 > 0.0 {
                            let p = e.client_coordinates();
                            send(to_event(track, p.x, p.y));
                        }
                    },
                    onpointerup: move |_| dragging.set(None),
                    onpointercancel: move |_| dragging.set(None),
                }
            }
        }
    }
}
