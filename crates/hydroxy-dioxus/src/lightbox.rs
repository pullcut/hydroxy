//! `Lightbox`: one image or video at a time over the page. Zoom with the
//! wheel, a pinch, a double-click or `+` / `-`; drag to pan when zoomed. Swipe,
//! the arrow keys or the strip of thumbnails change item. It is a native modal
//! `<dialog>`: focus stays inside, Escape closes, and focus goes back on close.

use dioxus::prelude::*;
use hydroxy_headless::lightbox::{
    self as h, LightboxCommand, LightboxEvent, LightboxInput, LightboxMachine, LightboxState, MediaItem, MediaKind,
    StageBox,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::js;
use crate::overlay::{close_dialog, show_modal};

pub use hydroxy_headless::lightbox::{MediaItem as LightboxItem, MediaKind as LightboxKind};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<LightboxState>,
    input: Memo<LightboxInput>,
    stage: Signal<StageBox>,
    on_change: EventHandler<Option<usize>>,
}

impl Ctx {
    fn send(self, event: LightboxEvent) {
        for LightboxCommand::Change(to) in drive::<LightboxMachine>(self.state, self.input, event) {
            self.on_change.call(to);
        }
    }

    fn size(self) -> h::Size {
        self.stage.peek().media
    }

    fn point(self, x: f64, y: f64) -> h::Point {
        self.stage.peek().point(x, y)
    }
}

/// A modal viewer for `items`. Controlled: `index` says which item shows (`None`
/// closes it) and `on_change` asks for a different one.
#[component]
pub fn Lightbox(
    /// The media.
    items: ReadSignal<Vec<MediaItem>>,
    /// The open item, or `None` when closed.
    index: ReadSignal<Option<usize>>,
    /// Called with the item to show, or `None` to close.
    on_change: EventHandler<Option<usize>>,
    /// Wrap from the last item to the first.
    #[props(default)]
    looping: bool,
    /// The dialog's accessible name. Default "Media viewer".
    #[props(default = "Media viewer".to_string())]
    label: String,
) -> Element {
    let input = use_memo(move || LightboxInput { count: items.read().len(), index: index(), looping });
    let mut state = use_signal(|| LightboxMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = LightboxMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let mut stage = use_signal(StageBox::default);
    let ctx = use_hook(|| Ctx { state, input, stage, on_change });
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let watch = js::use_watch(h::WATCH_JS, ids.stage.clone(), move |report| {
        if let Ok(b) = report.parse::<StageBox>() {
            stage.set(b);
        }
    });

    let content_id = ids.content.clone();
    use_effect(move || {
        if index().is_some() {
            show_modal(&content_id);
            js::run(&[("root", &content_id)], h::SHOW_THUMB_JS);
        } else {
            close_dialog(&content_id);
        }
    });

    let list = items();
    let at = index();
    let count = list.len();
    let current = at.and_then(|i| list.get(i).cloned());
    let is_image = current.as_ref().is_some_and(MediaItem::is_image);
    let s = state.read().clone();
    let zoomed = s.zoom.is_zoomed();
    let transform = if s.drag == 0.0 { s.zoom.css() } else { format!("translateX({:.1}px)", s.drag) };
    let zoom_label = if zoomed { "Zoom out" } else { "Zoom in" };
    let caption_id = format!("{}-caption", ids.content);
    let stage_id = ids.stage.clone();
    let preload: Vec<String> = at
        .map(|i| {
            h::neighbours(i, count, looping)
                .into_iter()
                .filter_map(|n| list.get(n).filter(|m| m.is_image()).map(|m| m.src.clone()))
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        dialog {
            id: ids.content.clone(),
            "aria-label": label,
            "aria-describedby": caption_id.clone(),
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-overlay": "",
            oncancel: move |e| {
                e.prevent_default();
                ctx.send(LightboxEvent::Close);
            },
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string(), ctx.size()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            div { "data-scope": h::SCOPE, "data-part": "header",
                p { id: ids.counter.clone(), "data-scope": h::SCOPE, "data-part": "counter", "aria-live": "polite",
                    if let Some(i) = at { {h::counter(i, count)} }
                }
                div { "data-scope": h::SCOPE, "data-part": "actions",
                    button {
                        r#type: "button",
                        class: ICON_BUTTON_CLASS,
                        "data-variant": "ghost",
                        "data-scope": h::SCOPE,
                        "data-part": "zoom",
                        "aria-label": zoom_label,
                        title: zoom_label,
                        "aria-pressed": zoomed.to_string(),
                        hidden: !is_image,
                        onclick: move |_| {
                            let event = if zoomed { LightboxEvent::ResetZoom } else { LightboxEvent::DoubleTap(h::Point::default(), ctx.size()) };
                            ctx.send(event);
                        },
                        Icon { name: IconName::Search, size: 18 }
                    }
                    if let Some(href) = current.as_ref().and_then(|m| m.download.clone()) {
                        a {
                            class: ICON_BUTTON_CLASS,
                            "data-variant": "ghost",
                            "data-scope": h::SCOPE,
                            "data-part": "download",
                            href,
                            "download": "",
                            "aria-label": "Download",
                            title: "Download",
                            Icon { name: IconName::Download, size: 18 }
                        }
                    }
                    button {
                        r#type: "button",
                        class: ICON_BUTTON_CLASS,
                        "data-variant": "ghost",
                        "data-scope": h::SCOPE,
                        "data-part": "close",
                        "aria-label": "Close",
                        title: "Close",
                        onclick: move |_| ctx.send(LightboxEvent::Close),
                        Icon { name: IconName::X, size: 18 }
                    }
                }
            }
            div {
                id: ids.stage,
                "data-scope": h::SCOPE,
                "data-part": "stage",
                "data-zoomed": zoomed.then_some(""),
                "data-gesture": (s.drag != 0.0).then_some(""),
                onmounted: move |_| {
                    let _ = watch.send("start");
                },
                onwheel: move |e| {
                    if !is_image {
                        return;
                    }
                    e.prevent_default();
                    let p = e.data().client_coordinates();
                    let at = ctx.point(p.x, p.y);
                    ctx.send(LightboxEvent::Wheel { delta: e.data().delta().strip_units().y, at, size: ctx.size() });
                },
                ondoubleclick: move |e| {
                    if is_image {
                        let p = e.client_coordinates();
                        ctx.send(LightboxEvent::DoubleTap(ctx.point(p.x, p.y), ctx.size()));
                    }
                },
                onpointerdown: move |e| {
                    if !is_image || e.trigger_button().is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary) {
                        return;
                    }
                    js::run(
                        &[("id", &stage_id), ("p", &e.pointer_id().to_string())],
                        "try { document.getElementById(id)?.setPointerCapture(Number(p)); } catch (_) {}",
                    );
                    let p = e.client_coordinates();
                    ctx.send(LightboxEvent::PointerDown(e.pointer_id(), ctx.point(p.x, p.y)));
                },
                onpointermove: move |e| {
                    let p = e.client_coordinates();
                    ctx.send(LightboxEvent::PointerMove(e.pointer_id(), ctx.point(p.x, p.y), ctx.size()));
                },
                onpointerup: move |e| ctx.send(LightboxEvent::PointerUp(e.pointer_id())),
                onpointercancel: move |e| ctx.send(LightboxEvent::PointerUp(e.pointer_id())),
                if let (Some(item), Some(i)) = (current.clone(), at) {
                    if item.kind == MediaKind::Image {
                        img {
                            key: "{i}",
                            "data-scope": h::SCOPE,
                            "data-part": "media",
                            "data-index": "{i}",
                            src: item.src,
                            alt: item.alt,
                            draggable: "false",
                            decoding: "async",
                            style: "transform: {transform};",
                        }
                    } else {
                        video {
                            key: "{i}",
                            "data-scope": h::SCOPE,
                            "data-part": "media",
                            "data-index": "{i}",
                            src: item.src,
                            poster: item.poster,
                            "aria-label": item.alt,
                            controls: true,
                            "playsinline": "true",
                            preload: "metadata",
                        }
                    }
                }
            }
            button {
                r#type: "button",
                class: ICON_BUTTON_CLASS,
                "data-variant": "secondary",
                "data-scope": h::SCOPE,
                "data-part": "prev",
                "aria-label": "Previous",
                title: "Previous",
                hidden: count < 2,
                disabled: !looping && at == Some(0),
                onclick: move |_| ctx.send(LightboxEvent::Prev),
                Icon { name: IconName::ChevronRight, size: 20 }
            }
            button {
                r#type: "button",
                class: ICON_BUTTON_CLASS,
                "data-variant": "secondary",
                "data-scope": h::SCOPE,
                "data-part": "next",
                "aria-label": "Next",
                title: "Next",
                hidden: count < 2,
                disabled: !looping && at.is_some_and(|i| i + 1 >= count),
                onclick: move |_| ctx.send(LightboxEvent::Next),
                Icon { name: IconName::ChevronRight, size: 20 }
            }
            p { id: caption_id, "data-scope": h::SCOPE, "data-part": "caption",
                if let Some(c) = current.as_ref().and_then(|m| m.caption.clone()) { "{c}" }
            }
            div { "data-scope": h::SCOPE, "data-part": "thumbs", role: "group", "aria-label": "Thumbnails", hidden: count < 2,
                for (i, item) in list.iter().enumerate() {
                    button {
                        key: "{i}",
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "thumb",
                        "aria-label": h::thumb_label(i, item),
                        "aria-current": (at == Some(i)).then_some("true"),
                        onclick: move |_| ctx.send(LightboxEvent::Open(i)),
                        img { src: item.thumb_src().to_string(), alt: "", loading: "lazy", decoding: "async", draggable: "false" }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "preload", hidden: true, "aria-hidden": "true",
                for src in preload { img { src, alt: "" } }
            }
        }
    }
}
