//! `ImageCropper`: an image with a crop box you drag, resize from eight
//! handles, lock to an aspect ratio, nudge from the keyboard, zoom and turn in
//! quarter turns. Reports the crop in pixels of the original file.
//!
//! While dragging, a fixed full-window layer catches pointer moves, since not
//! every Dioxus renderer has pointer capture. The image's natural size is read
//! with a small script when it loads.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::image_cropper::{
    self as h, Handle, ImageCropperCommand, ImageCropperEvent, ImageCropperInput, ImageCropperMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::modifiers::mods;

pub use hydroxy_headless::image_cropper::{Crop, PRESETS};

/// Crop an image with pointer or keyboard.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn ImageCropper(
    /// Image URL.
    src: ReadSignal<String>,
    /// Alternative text for the image.
    alt: String,
    /// Accessible name for the whole cropper.
    #[props(default = "Crop image".to_string())]
    label: String,
    /// Aspect lock (width / height) to start with. Default free.
    #[props(default)]
    aspect: Option<f64>,
    /// Show the aspect preset buttons.
    #[props(default = true)]
    presets: bool,
    /// Largest zoom. Default 4.
    #[props(default = 4.0)]
    max_zoom: f64,
    /// Block every change.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the crop, in source pixels, whenever it changes.
    on_change: Option<EventHandler<Crop>>,
) -> Element {
    let input =
        use_memo(move || ImageCropperInput { aspect, max_zoom, disabled: disabled(), ..ImageCropperInput::default() });
    let state = use_signal(|| ImageCropperMachine::initial(&input.peek()));
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let h::Ids { img: img_id, hint: hint_id, zoom: zoom_id } = h::ids(&root);
    let mut stage = use_signal(|| None::<Rc<MountedData>>);
    let mut crop_box = use_signal(|| None::<Rc<MountedData>>);
    let mut rect = use_signal(|| (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64));

    let send = move |event: ImageCropperEvent| {
        for ImageCropperCommand::Change(crop) in drive::<ImageCropperMachine>(state, input, event) {
            if let Some(handler) = on_change {
                handler.call(crop);
            }
        }
    };
    let read_size = {
        let img_id = img_id.clone();
        move || {
            let img_id = img_id.clone();
            spawn(async move {
                let mut js = document::eval(&format!(
                    "const el = document.getElementById({img_id});
                    dioxus.send(el && el.complete ? [el.naturalWidth, el.naturalHeight] : [0, 0]);",
                    img_id = crate::js::js_string(&img_id)
                ));
                if let Ok((width, height)) = js.recv::<(f64, f64)>().await
                    && width > 0.0
                {
                    send(ImageCropperEvent::Loaded { width, height });
                }
            });
        }
    };
    // An image cached before the first render may have loaded already, without a load event.
    let first_read = read_size.clone();
    use_effect(first_read);

    let press = move |handle: Handle, x: f64, y: f64| {
        if input.peek().disabled {
            return;
        }
        let Some(el) = stage.peek().clone() else { return };
        if let Some(b) = crop_box.peek().clone() {
            spawn(async move {
                let _ = b.set_focus(true).await;
            });
        }
        spawn(async move {
            if let Ok(r) = el.get_client_rect().await {
                rect.set((r.origin.x, r.origin.y, r.size.width, r.size.height));
                let (vx, vy) = h::to_view_px(&state.peek(), x, y, rect());
                send(ImageCropperEvent::Press { handle, x: vx, y: vy });
            }
        });
    };

    let s = state.read();
    let is_disabled = input.read().disabled;
    let loaded = s.is_loaded();
    let dragging = s.drag.is_some();
    let stage_style = h::stage_style(&s);
    let image_style = h::image_style(&s);
    let box_style = h::box_style(&s);
    let readout = h::describe(&s);
    let live = if dragging { String::new() } else { readout.clone() };
    let zoom = s.zoom;
    let current_aspect = s.aspect;
    let zoom_text = h::zoom_valuetext(&s);
    drop(s);
    let off = is_disabled || !loaded;
    let src_now = src();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": label,
            "data-disabled": is_disabled.then_some(""),
            "data-dragging": dragging.then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "viewport",
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "stage",
                    style: stage_style,
                    onmounted: move |e| stage.set(Some(e.data())),
                    img {
                        id: img_id,
                        "data-scope": h::SCOPE,
                        "data-part": "image",
                        src: src_now,
                        alt,
                        draggable: "false",
                        style: image_style,
                        onload: move |_| read_size(),
                    }
                    if loaded {
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "box",
                            "data-handle": "move",
                            tabindex: if is_disabled { "-1" } else { "0" },
                            role: "group",
                            "aria-roledescription": "crop area",
                            "aria-label": "Crop area, {readout}",
                            "aria-describedby": hint_id.clone(),
                            "aria-disabled": is_disabled.then_some("true"),
                            "data-dragging": dragging.then_some(""),
                            style: box_style,
                            onmounted: move |e| crop_box.set(Some(e.data())),
                            onpointerdown: move |e| {
                                e.prevent_default();
                                let p = e.client_coordinates();
                                press(Handle::Move, p.x, p.y);
                            },
                            onkeydown: move |e| {
                                let m = e.modifiers();
                                let event = h::box_key(&e.key().to_string(), mods(m), &state.peek());
                                if let Some(event) = event {
                                    e.prevent_default();
                                    send(event);
                                }
                            },
                            span { "data-scope": h::SCOPE, "data-part": "grid", "aria-hidden": "true" }
                            for hd in Handle::RESIZE {
                                span {
                                    key: "{hd.as_str()}",
                                    "data-scope": h::SCOPE,
                                    "data-part": "handle",
                                    "data-handle": hd.as_str(),
                                    "aria-hidden": "true",
                                    onpointerdown: move |e| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let p = e.client_coordinates();
                                        press(hd, p.x, p.y);
                                    },
                                }
                            }
                        }
                    }
                }
            }
            span { id: hint_id, "data-scope": h::SCOPE, "data-part": "hint",
                "Arrow keys move the crop, Shift for bigger steps. Alt with arrows resizes. Plus and minus zoom; brackets rotate."
            }
            div { "data-scope": h::SCOPE, "data-part": "toolbar",
                if presets {
                    div { "data-scope": h::SCOPE, "data-part": "aspects", role: "group", "aria-label": "Aspect ratio",
                        for (text , value) in PRESETS {
                            {
                                let on = h::preset_active(value, current_aspect);
                                rsx! {
                                    button {
                                        key: "{text}",
                                        r#type: "button",
                                        "data-scope": h::SCOPE,
                                        "data-part": "aspect",
                                        "aria-pressed": on.to_string(),
                                        "data-selected": on.then_some(""),
                                        disabled: off,
                                        onclick: move |_| send(ImageCropperEvent::SetAspect(value)),
                                        "{text}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { "data-scope": h::SCOPE, "data-part": "tools",
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "rotate-left",
                        "aria-label": "Rotate left",
                        title: "Rotate left ([)",
                        disabled: off,
                        onclick: move |_| send(ImageCropperEvent::Rotate(false)),
                        Icon { name: IconName::RotateCcw }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "rotate-right",
                        "aria-label": "Rotate right",
                        title: "Rotate right (])",
                        disabled: off,
                        onclick: move |_| send(ImageCropperEvent::Rotate(true)),
                        Icon { name: IconName::RotateCw }
                    }
                    label { "data-scope": h::SCOPE, "data-part": "zoom-label", r#for: zoom_id.clone(), "Zoom" }
                    input {
                        id: zoom_id,
                        r#type: "range",
                        "data-scope": h::SCOPE,
                        "data-part": "zoom",
                        min: "1",
                        max: max_zoom.to_string(),
                        step: "0.25",
                        "aria-valuetext": zoom_text,
                        disabled: off,
                        value: zoom.to_string(),
                        oninput: move |e| {
                            if let Ok(z) = e.value().parse::<f64>() {
                                send(ImageCropperEvent::SetZoom(z));
                            }
                        },
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "reset",
                        disabled: off,
                        onclick: move |_| send(ImageCropperEvent::Reset),
                        "Reset"
                    }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "readout", "aria-hidden": "true", "{readout}" }
            span { "data-scope": h::SCOPE, "data-part": "live", "aria-live": "polite", "{live}" }
            if dragging {
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "capture",
                    "aria-hidden": "true",
                    onpointermove: move |e| {
                        if rect().2 > 0.0 {
                            let p = e.client_coordinates();
                            let (x, y) = h::to_view_px(&state.peek(), p.x, p.y, rect());
                            send(ImageCropperEvent::Drag { x, y });
                        }
                    },
                    onpointerup: move |_| send(ImageCropperEvent::Release),
                    onpointercancel: move |_| send(ImageCropperEvent::Release),
                }
            }
        }
    }
}
