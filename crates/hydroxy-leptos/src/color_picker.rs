//! `<ColorPicker>`: a saturation/brightness area, a hue slider, an optional
//! alpha slider, a hex field and swatches. Drag the thumbs or focus one and use
//! the arrow keys (Shift for bigger steps).

use hydroxy_headless::color_picker::{
    self as h, ColorPickerCommand, ColorPickerEvent, ColorPickerInput, ColorPickerMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::drive::drive;
use crate::focus::focus_id;

pub use hydroxy_headless::color_picker::{DEFAULT_SWATCHES, Rgba};

#[derive(Clone, Copy)]
enum Track {
    Area,
    Hue,
    Alpha,
}

/// A colour picked with pointer, keyboard or hex text.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn ColorPicker(
    /// Accessible name for the whole picker.
    #[prop(into)]
    label: String,
    /// Controlled colour. Pair with `on_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<Rgba>,
    /// Initial colour when uncontrolled. Default black.
    #[prop(optional)]
    default_value: Option<Rgba>,
    /// Offer an alpha slider and `#rrggbbaa` hex.
    #[prop(optional)]
    alpha: bool,
    /// Preset colours below the hex field. Default a palette of eight; pass an empty list for none.
    #[prop(optional)]
    swatches: Option<Vec<Rgba>>,
    /// Block changes.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name. Submits the hex text.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the colour as it changes.
    #[prop(optional, into)]
    on_change: Option<Callback<Rgba>>,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let ids = h::ids(&root);
    let input = Memo::new(move |_| ColorPickerInput {
        value: value.get(),
        default_value,
        alpha,
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(ColorPickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ColorPickerMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let send = move |event: ColorPickerEvent| {
        for ColorPickerCommand::ValueChange(color) in drive::<ColorPickerMachine, _>(state, input, event) {
            if let Some(cb) = on_change {
                cb.run(color);
            }
        }
    };

    // Pointer on a track: focus its thumb, then turn the position into an event.
    let thumb_ids = StoredValue::new([ids.area.clone(), ids.hue.clone(), ids.alpha.clone()]);
    let to_event = move |ev: &web_sys::PointerEvent, track: Track| {
        let el = ev.current_target()?.dyn_into::<web_sys::Element>().ok()?;
        let r = el.get_bounding_client_rect();
        let (x, y) = (f64::from(ev.client_x()), f64::from(ev.client_y()));
        Some(match track {
            Track::Area => h::area_event(x, y, r.left(), r.top(), r.width(), r.height()),
            Track::Hue => h::hue_event(x, r.left(), r.width()),
            Track::Alpha => h::alpha_event(x, r.left(), r.width()),
        })
    };
    let press = move |ev: web_sys::PointerEvent, track: Track| {
        if ev.button() != 0 || input.with_untracked(|i| i.disabled) {
            return;
        }
        // Keep the thumb focus we ask for, not the browser's default.
        ev.prevent_default();
        if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        focus_id(&thumb_ids.with_value(|t| t[track as usize].clone()));
        if let Some(event) = to_event(&ev, track) {
            send(event);
        }
    };
    let drag = move |ev: web_sys::PointerEvent, track: Track| {
        if ev.buttons() & 1 == 1
            && let Some(event) = to_event(&ev, track)
        {
            send(event);
        }
    };

    let style = move || input.with(|i| state.with(|s| h::root_style(s, i)));
    let is_disabled = move || input.with(|i| i.disabled);
    let swatches = swatches.unwrap_or_else(|| h::DEFAULT_SWATCHES.to_vec());
    let current = move || input.with(|i| state.with(|s| s.rgba(i)));
    let has_swatches = !swatches.is_empty();
    let swatch_views = swatches
        .into_iter()
        .map(|color| {
            let hex = color.hex(false);
            let chosen = move || {
                let c = current();
                (c.r, c.g, c.b) == (color.r, color.g, color.b)
            };
            view! {
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="swatch"
                    aria-label=hex.clone()
                    aria-pressed=move || chosen().to_string()
                    data-selected=move || chosen().then_some("")
                    style=format!("--swatch: {hex};")
                    disabled=is_disabled
                    on:click=move |_| send(ColorPickerEvent::SetColor(Rgba { a: current().a, ..color }))
                ></button>
            }
        })
        .collect_view();

    let hue_now = move || state.with(h::hue_degrees);
    let alpha_now = move || state.with(|s| h::percent(s.alpha));

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=label
            style=style
            data-disabled=move || is_disabled().then_some("")
        >
            <div
                data-scope=h::SCOPE
                data-part="area"
                on:pointerdown=move |ev| press(ev, Track::Area)
                on:pointermove=move |ev| drag(ev, Track::Area)
            >
                <div
                    id=ids.area.clone()
                    role="slider"
                    tabindex=move || if is_disabled() { "-1" } else { "0" }
                    aria-label="Colour"
                    aria-roledescription="2D slider"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow=move || state.with(|s| h::percent(s.hsv.s))
                    aria-valuetext=move || state.with(h::area_valuetext)
                    aria-disabled=move || is_disabled().then_some("true")
                    data-scope=h::SCOPE
                    data-part="area-thumb"
                    style=move || state.with(h::area_thumb_style)
                    on:keydown=move |ev| {
                        if let Some(event) = h::area_keydown(&ev.key(), ev.shift_key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                ></div>
            </div>
            <div data-scope=h::SCOPE data-part="controls">
                <div data-scope=h::SCOPE data-part="preview" aria-hidden="true"></div>
                <div data-scope=h::SCOPE data-part="sliders">
                    <div
                        data-scope=h::SCOPE
                        data-part="hue"
                        on:pointerdown=move |ev| press(ev, Track::Hue)
                        on:pointermove=move |ev| drag(ev, Track::Hue)
                    >
                        <div
                            id=ids.hue.clone()
                            role="slider"
                            tabindex=move || if is_disabled() { "-1" } else { "0" }
                            aria-label="Hue"
                            aria-valuemin="0"
                            aria-valuemax="360"
                            aria-valuenow=hue_now
                            aria-valuetext=move || state.with(h::hue_valuetext)
                            aria-orientation="horizontal"
                            aria-disabled=move || is_disabled().then_some("true")
                            data-scope=h::SCOPE
                            data-part="hue-thumb"
                            style=move || state.with(h::hue_thumb_style)
                            on:keydown=move |ev| {
                                if let Some(event) = h::hue_keydown(&ev.key(), ev.shift_key()) {
                                    ev.prevent_default();
                                    send(event);
                                }
                            }
                        ></div>
                    </div>
                    {alpha.then(|| view! {
                        <div
                            data-scope=h::SCOPE
                            data-part="alpha"
                            on:pointerdown=move |ev| press(ev, Track::Alpha)
                            on:pointermove=move |ev| drag(ev, Track::Alpha)
                        >
                            <div
                                id=ids.alpha.clone()
                                role="slider"
                                tabindex=move || if is_disabled() { "-1" } else { "0" }
                                aria-label="Opacity"
                                aria-valuemin="0"
                                aria-valuemax="100"
                                aria-valuenow=alpha_now
                                aria-valuetext=move || state.with(h::alpha_valuetext)
                                aria-orientation="horizontal"
                                aria-disabled=move || is_disabled().then_some("true")
                                data-scope=h::SCOPE
                                data-part="alpha-thumb"
                                style=move || state.with(h::alpha_thumb_style)
                                on:keydown=move |ev| {
                                    if let Some(event) = h::alpha_keydown(&ev.key(), ev.shift_key()) {
                                        ev.prevent_default();
                                        send(event);
                                    }
                                }
                            ></div>
                        </div>
                    })}
                </div>
                <input
                    type="text"
                    id=ids.hex.clone()
                    data-scope=h::SCOPE
                    data-part="hex"
                    aria-label="Hex colour"
                    autocomplete="off"
                    spellcheck="false"
                    maxlength="9"
                    disabled=is_disabled
                    aria-invalid=move || state.with(hydroxy_headless::color_picker::ColorPickerState::invalid).then_some("true")
                    prop:value=move || state.with(|s| s.draft.clone())
                    on:input=move |ev| send(ColorPickerEvent::HexInput(event_target_value(&ev)))
                    on:blur=move |_| send(ColorPickerEvent::HexCommit)
                    on:keydown=move |ev| {
                        if let Some(event) = h::hex_keydown(&ev.key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                />
            </div>
            {has_swatches.then(|| view! {
                <div data-scope=h::SCOPE data-part="swatches" role="group" aria-label="Swatches">
                    {swatch_views}
                </div>
            })}
            {name.map(|name| {
                view! {
                    <input
                        type="hidden"
                        name=name
                        prop:value=move || input.with(|i| state.with(|s| h::form_value(s, i)))
                    />
                }
            })}
        </div>
    }
}
