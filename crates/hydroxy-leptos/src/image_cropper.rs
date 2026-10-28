//! `<ImageCropper>`: an image with a crop box you drag, resize from eight
//! handles, lock to an aspect ratio, nudge from the keyboard, zoom and turn in
//! quarter turns. Reports the crop in pixels of the original file.

use hydroxy_headless::image_cropper::{
    self as h, Handle, ImageCropperCommand, ImageCropperEvent, ImageCropperInput, ImageCropperMachine,
};
use hydroxy_statechart::Machine;
use leptos::html::Img;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

pub use hydroxy_headless::image_cropper::{Crop, PRESETS};

/// Crop an image with pointer or keyboard.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn ImageCropper(
    /// Image URL.
    #[prop(into)]
    src: Signal<String>,
    /// Alternative text for the image.
    #[prop(into)]
    alt: String,
    /// Accessible name for the whole cropper.
    #[prop(default = "Crop image".into(), into)]
    label: String,
    /// Aspect lock (width / height) to start with. Default free.
    #[prop(optional)]
    aspect: Option<f64>,
    /// Show the aspect preset buttons.
    #[prop(default = true)]
    presets: bool,
    /// Largest zoom. Default 4.
    #[prop(default = 4.0)]
    max_zoom: f64,
    /// Block every change.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Called with the crop, in source pixels, whenever it changes.
    #[prop(optional, into)]
    on_change: Option<Callback<Crop>>,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let hint_id = format!("{root}-hint");
    let zoom_id = format!("{root}-zoom");
    let box_hint = hint_id.clone();
    let input = Memo::new(move |_| ImageCropperInput {
        aspect,
        max_zoom,
        disabled: disabled.get().unwrap_or(false),
        ..ImageCropperInput::default()
    });
    let state = RwSignal::new(input.with_untracked(ImageCropperMachine::initial));
    let img = NodeRef::<Img>::new();
    let rect = StoredValue::new((0.0, 0.0, 0.0, 0.0));

    let send = move |event: ImageCropperEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| ImageCropperMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && state.with_untracked(|s| *s != next)
        {
            state.set(next);
        }
        for ImageCropperCommand::Change(crop) in step.commands {
            if let Some(cb) = on_change {
                cb.run(crop);
            }
        }
    };
    let read_size = move || {
        if let Some(el) = img.get_untracked()
            && el.complete()
            && el.natural_width() > 0
        {
            send(ImageCropperEvent::Loaded {
                width: f64::from(el.natural_width()),
                height: f64::from(el.natural_height()),
            });
        }
    };
    // An image cached before hydration may have loaded already, without a load event.
    Effect::new(move |_| {
        if img.get().is_some() {
            read_size();
        }
    });

    let view_px = move |ev: &web_sys::PointerEvent| {
        state.with_untracked(|s| h::to_view_px(s, f64::from(ev.client_x()), f64::from(ev.client_y()), rect.get_value()))
    };
    let on_down = move |ev: web_sys::PointerEvent| {
        if (ev.pointer_type() == "mouse" && ev.button() != 0) || input.with_untracked(|i| i.disabled) {
            return;
        }
        let handle = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("[data-handle]").ok().flatten())
            .and_then(|el| el.get_attribute("data-handle"))
            .and_then(|s| Handle::parse(&s));
        let (Some(handle), Some(stage_el)) =
            (handle, ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()))
        else {
            return;
        };
        ev.prevent_default();
        let _ = stage_el.set_pointer_capture(ev.pointer_id());
        let r = stage_el.get_bounding_client_rect();
        rect.set_value((r.left(), r.top(), r.width(), r.height()));
        if let Some(b) = stage_el
            .query_selector("[data-part='box']")
            .ok()
            .flatten()
            .and_then(|b| b.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = b.focus();
        }
        let (x, y) = view_px(&ev);
        send(ImageCropperEvent::Press { handle, x, y });
    };

    let is_disabled = move || input.with(|i| i.disabled);
    let loaded = move || state.with(h::ImageCropperState::loaded);
    let dragging = move || state.with(|s| s.drag.is_some());
    let handles = || {
        Handle::RESIZE
            .into_iter()
            .map(|hd| view! { <span data-scope=h::SCOPE data-part="handle" data-handle=hd.as_str() aria-hidden="true"></span> })
            .collect_view()
    };
    let preset_buttons = PRESETS
        .into_iter()
        .map(|(text, value)| {
            let on = move || state.with(|s| h::preset_active(value, s.aspect));
            view! {
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="aspect"
                    aria-pressed=move || on().to_string()
                    data-selected=move || on().then_some("")
                    disabled=move || is_disabled() || !loaded()
                    on:click=move |_| send(ImageCropperEvent::SetAspect(value))
                >
                    {text}
                </button>
            }
        })
        .collect_view();

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=label
            data-disabled=move || is_disabled().then_some("")
            data-dragging=move || dragging().then_some("")
        >
            <div data-scope=h::SCOPE data-part="viewport">
                <div
                    data-scope=h::SCOPE
                    data-part="stage"
                    style=move || state.with(h::stage_style)
                    on:pointerdown=on_down
                    on:pointermove=move |ev: web_sys::PointerEvent| {
                        if state.with_untracked(|s| s.drag.is_some()) {
                            let (x, y) = view_px(&ev);
                            send(ImageCropperEvent::Drag { x, y });
                        }
                    }
                    on:pointerup=move |_| send(ImageCropperEvent::Release)
                    on:pointercancel=move |_| send(ImageCropperEvent::Release)
                >
                    <img
                        node_ref=img
                        data-scope=h::SCOPE
                        data-part="image"
                        src=move || src.get()
                        alt=alt
                        draggable="false"
                        style=move || state.with(h::image_style)
                        on:load=move |_| read_size()
                    />
                    <Show when=loaded>
                        <div
                            data-scope=h::SCOPE
                            data-part="box"
                            data-handle="move"
                            tabindex=move || if is_disabled() { "-1" } else { "0" }
                            role="group"
                            aria-roledescription="crop area"
                            aria-label=move || format!("Crop area, {}", state.with(h::describe))
                            aria-describedby=box_hint.clone()
                            aria-disabled=move || is_disabled().then_some("true")
                            data-dragging=move || dragging().then_some("")
                            style=move || state.with(h::box_style)
                            on:keydown=move |ev| {
                                let event = state.with_untracked(|s| h::box_key(&ev.key(), ev.shift_key(), ev.alt_key(), s));
                                if let Some(event) = event {
                                    ev.prevent_default();
                                    send(event);
                                }
                            }
                        >
                            <span data-scope=h::SCOPE data-part="grid" aria-hidden="true"></span>
                            {handles()}
                        </div>
                    </Show>
                </div>
            </div>
            <span id=hint_id.clone() data-scope=h::SCOPE data-part="hint">
                "Arrow keys move the crop, Shift for bigger steps. Alt with arrows resizes. Plus and minus zoom; brackets rotate."
            </span>
            <div data-scope=h::SCOPE data-part="toolbar">
                {presets.then(|| view! {
                    <div data-scope=h::SCOPE data-part="aspects" role="group" aria-label="Aspect ratio">
                        {preset_buttons}
                    </div>
                })}
                <div data-scope=h::SCOPE data-part="tools">
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="rotate-left"
                        aria-label="Rotate left"
                        title="Rotate left ([)"
                        disabled=move || is_disabled() || !loaded()
                        on:click=move |_| send(ImageCropperEvent::Rotate(false))
                    >
                        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path d="M4 9a8 8 0 1 1 1.5 8M4 4v5h5" /></svg>
                    </button>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="rotate-right"
                        aria-label="Rotate right"
                        title="Rotate right (])"
                        disabled=move || is_disabled() || !loaded()
                        on:click=move |_| send(ImageCropperEvent::Rotate(true))
                    >
                        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path d="M20 9a8 8 0 1 0-1.5 8M20 4v5h-5" /></svg>
                    </button>
                    <label data-scope=h::SCOPE data-part="zoom-label" for=zoom_id.clone()>"Zoom"</label>
                    <input
                        id=zoom_id
                        type="range"
                        data-scope=h::SCOPE
                        data-part="zoom"
                        min="1"
                        max=max_zoom.to_string()
                        step="0.25"
                        aria-valuetext=move || format!("{}%", (state.with(|s| s.zoom) * 100.0).round())
                        disabled=move || is_disabled() || !loaded()
                        prop:value=move || state.with(|s| s.zoom.to_string())
                        on:input=move |ev| {
                            if let Ok(z) = event_target_value(&ev).parse::<f64>() {
                                send(ImageCropperEvent::SetZoom(z));
                            }
                        }
                    />
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="reset"
                        disabled=move || is_disabled() || !loaded()
                        on:click=move |_| send(ImageCropperEvent::Reset)
                    >
                        "Reset"
                    </button>
                </div>
            </div>
            <span data-scope=h::SCOPE data-part="readout" aria-hidden="true">{move || state.with(h::describe)}</span>
            <span data-scope=h::SCOPE data-part="live" aria-live="polite">
                {move || state.with(|s| if s.drag.is_some() { String::new() } else { h::describe(s) })}
            </span>
        </div>
    }
}
