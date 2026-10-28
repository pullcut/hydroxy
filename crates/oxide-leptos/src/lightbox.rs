//! `<Lightbox>`: one image or video at a time over the page. Zoom with the
//! wheel, a pinch, a double-click or `+` / `-`; drag to pan when zoomed. Swipe,
//! the arrow keys or the strip of thumbnails change item. It is a native modal
//! `<dialog>`: focus stays inside, Escape closes, and focus goes back on close.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;
use oxide_headless::lightbox::{
    self as h, LightboxCommand, LightboxEvent, LightboxInput, LightboxMachine, LightboxState, MediaItem, MediaKind,
    StageBox,
};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::js;
use crate::overlay::{close_dialog, show_modal};

pub use oxide_headless::lightbox::{MediaItem as LightboxItem, MediaKind as LightboxKind};

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<LightboxState>,
    input: Memo<LightboxInput>,
    stage: RwSignal<StageBox>,
    on_change: Callback<Option<usize>>,
}

impl Ctx {
    fn send(self, event: LightboxEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| LightboxMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && self.state.with_untracked(|s| *s != next)
        {
            self.state.set(next);
        }
        for LightboxCommand::Change(to) in step.commands {
            self.on_change.run(to);
        }
    }

    fn size(self) -> h::Size {
        self.stage.with_untracked(|b| b.media)
    }

    fn point(self, x: i32, y: i32) -> h::Point {
        self.stage.with_untracked(|b| b.point(f64::from(x), f64::from(y)))
    }
}

/// Mirrored chevron for "previous"; CSS flips it.
fn chevron() -> impl IntoView {
    view! { <Icon name=IconName::ChevronRight size=20 /> }
}

/// A modal viewer for `items`. Controlled: `index` says which item shows (`None`
/// closes it) and `on_change` asks for a different one.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Lightbox(
    /// The media.
    #[prop(into)]
    items: Signal<Vec<MediaItem>>,
    /// The open item, or `None` when closed.
    #[prop(into)]
    index: Signal<Option<usize>>,
    /// Called with the item to show, or `None` to close.
    #[prop(into)]
    on_change: Callback<Option<usize>>,
    /// Wrap from the last item to the first.
    #[prop(optional)]
    looping: bool,
    /// The dialog's accessible name. Default "Media viewer".
    #[prop(default = "Media viewer".into(), into)]
    label: String,
) -> impl IntoView {
    let input = Memo::new(move |_| LightboxInput { count: items.with(Vec::len), index: index.get(), looping });
    let state = RwSignal::new(input.with_untracked(LightboxMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| LightboxMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let stage_box = RwSignal::new(StageBox::default());
    let ctx = Ctx { state, input, stage: stage_box, on_change };
    let ids = StoredValue::new(h::ids(&crate::id::next_id(h::SCOPE)));

    // The dialog follows the index; the stage is watched while it is open.
    let watching = StoredValue::new_local(None::<js::Watch>);
    Effect::new(move |_| {
        let ids = ids.get_value();
        let open = index.get().is_some();
        if open {
            show_modal(&ids.content);
            if watching.with_value(Option::is_none) {
                let w = js::watch(h::WATCH_JS, &ids.stage, move |report| {
                    if let Some(b) = StageBox::parse(&report) {
                        let _ = stage_box.try_set(b);
                    }
                });
                js::keep(watching, w);
            }
            js::call("root", h::SHOW_THUMB_JS, &[JsValue::from(ids.content)]);
        } else {
            close_dialog(&ids.content);
            watching.set_value(None);
        }
    });

    let current = move || index.get().and_then(|i| items.with(|list| list.get(i).cloned()));
    let count = move || items.with(Vec::len);
    let is_image = move || current().is_some_and(|m| m.kind == MediaKind::Image);
    let zoomed = move || state.with(|s| s.zoom.is_zoomed());
    let gesture = move || state.with(|s| s.drag != 0.0);
    let transform = move || {
        state.with(|s| {
            if s.drag == 0.0 { s.zoom.css() } else { format!("translateX({:.1}px)", s.drag) }
        })
    };
    let ids_v = ids.get_value();
    let caption_id = format!("{}-caption", ids_v.content);

    let media = move || {
        let item = current()?;
        let i = index.get().unwrap_or(0);
        Some(match item.kind {
            MediaKind::Image => view! {
                <img
                    data-scope=h::SCOPE
                    data-part="media"
                    data-index=i
                    src=item.src
                    alt=item.alt
                    draggable="false"
                    decoding="async"
                    style=move || format!("transform: {};", transform())
                />
            }
            .into_any(),
            MediaKind::Video => view! {
                <video
                    data-scope=h::SCOPE
                    data-part="media"
                    data-index=i
                    src=item.src
                    poster=item.poster
                    aria-label=item.alt
                    controls=true
                    playsinline=true
                    preload="metadata"
                ></video>
            }
            .into_any(),
        })
    };

    let preload = move || {
        let Some(i) = index.get() else { return Vec::new() };
        items.with(|list| {
            h::neighbours(i, list.len(), looping)
                .into_iter()
                .filter_map(|n| list.get(n).filter(|m| m.is_image()).map(|m| m.src.clone()))
                .collect::<Vec<_>>()
        })
    };

    let thumbs = move || {
        items
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let label = h::thumb_label(i, &item);
                view! {
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="thumb"
                        aria-label=label
                        aria-current=move || (index.get() == Some(i)).then_some("true")
                        on:click=move |_| ctx.send(LightboxEvent::Open(i))
                    >
                        <img src=item.thumb_src().to_string() alt="" loading="lazy" decoding="async" draggable="false" />
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <dialog
            id=ids_v.content.clone()
            aria-label=label
            aria-describedby=caption_id.clone()
            data-scope=h::SCOPE
            data-part="content"
            data-overlay=""
            on:cancel=move |ev: leptos::ev::Event| {
                ev.prevent_default();
                ctx.send(LightboxEvent::Close);
            }
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key(), ctx.size()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            <div data-scope=h::SCOPE data-part="header">
                <p id=ids_v.counter.clone() data-scope=h::SCOPE data-part="counter" aria-live="polite">
                    {move || index.get().map(|i| h::counter(i, count()))}
                </p>
                <div data-scope=h::SCOPE data-part="actions">
                    <button
                        type="button"
                        class=ICON_BUTTON_CLASS
                        data-variant="ghost"
                        data-scope=h::SCOPE
                        data-part="zoom"
                        aria-label=move || if zoomed() { "Zoom out" } else { "Zoom in" }
                        title=move || if zoomed() { "Zoom out" } else { "Zoom in" }
                        aria-pressed=move || zoomed().to_string()
                        hidden=move || !is_image()
                        on:click=move |_| {
                            let event = if zoomed() { LightboxEvent::ResetZoom } else { LightboxEvent::DoubleTap(h::Point::default(), ctx.size()) };
                            ctx.send(event);
                        }
                    >
                        <Icon name=IconName::Search size=18 />
                    </button>
                    {move || current().and_then(|m| m.download).map(|href| view! {
                        <a
                            class=ICON_BUTTON_CLASS
                            data-variant="ghost"
                            data-scope=h::SCOPE
                            data-part="download"
                            href=href
                            download=""
                            aria-label="Download"
                            title="Download"
                        >
                            <Icon name=IconName::Download size=18 />
                        </a>
                    })}
                    <button
                        type="button"
                        class=ICON_BUTTON_CLASS
                        data-variant="ghost"
                        data-scope=h::SCOPE
                        data-part="close"
                        aria-label="Close"
                        title="Close"
                        on:click=move |_| ctx.send(LightboxEvent::Close)
                    >
                        <Icon name=IconName::X size=18 />
                    </button>
                </div>
            </div>
            <div
                id=ids_v.stage.clone()
                data-scope=h::SCOPE
                data-part="stage"
                data-zoomed=move || zoomed().then_some("")
                data-gesture=move || gesture().then_some("")
                on:wheel=move |ev| {
                    if !is_image() {
                        return;
                    }
                    ev.prevent_default();
                    let unit = if ev.delta_mode() == 1 { 16.0 } else { 1.0 };
                    let at = ctx.point(ev.client_x(), ev.client_y());
                    ctx.send(LightboxEvent::Wheel { delta: ev.delta_y() * unit, at, size: ctx.size() });
                }
                on:dblclick=move |ev| {
                    if is_image() {
                        ctx.send(LightboxEvent::DoubleTap(ctx.point(ev.client_x(), ev.client_y()), ctx.size()));
                    }
                }
                on:pointerdown=move |ev| {
                    if !is_image() || ev.button() != 0 {
                        return;
                    }
                    let el = event_target::<web_sys::Element>(&ev);
                    let _ = el.set_pointer_capture(ev.pointer_id());
                    ctx.send(LightboxEvent::PointerDown(ev.pointer_id(), ctx.point(ev.client_x(), ev.client_y())));
                }
                on:pointermove=move |ev| {
                    ctx.send(LightboxEvent::PointerMove(ev.pointer_id(), ctx.point(ev.client_x(), ev.client_y()), ctx.size()));
                }
                on:pointerup=move |ev| ctx.send(LightboxEvent::PointerUp(ev.pointer_id()))
                on:pointercancel=move |ev| ctx.send(LightboxEvent::PointerUp(ev.pointer_id()))
            >
                {media}
            </div>
            <button
                type="button"
                class=ICON_BUTTON_CLASS
                data-variant="secondary"
                data-scope=h::SCOPE
                data-part="prev"
                aria-label="Previous"
                title="Previous"
                hidden=move || count() < 2
                disabled=move || !looping && index.get() == Some(0)
                on:click=move |_| ctx.send(LightboxEvent::Prev)
            >
                {chevron()}
            </button>
            <button
                type="button"
                class=ICON_BUTTON_CLASS
                data-variant="secondary"
                data-scope=h::SCOPE
                data-part="next"
                aria-label="Next"
                title="Next"
                hidden=move || count() < 2
                disabled=move || !looping && index.get().is_some_and(|i| i + 1 >= count())
                on:click=move |_| ctx.send(LightboxEvent::Next)
            >
                {chevron()}
            </button>
            <p id=caption_id.clone() data-scope=h::SCOPE data-part="caption">
                {move || current().and_then(|m| m.caption)}
            </p>
            <div data-scope=h::SCOPE data-part="thumbs" role="group" aria-label="Thumbnails" hidden=move || count() < 2>
                {thumbs}
            </div>
            <div data-scope=h::SCOPE data-part="preload" hidden aria-hidden="true">
                {move || preload().into_iter().map(|src| view! { <img src=src alt="" /> }).collect_view()}
            </div>
        </dialog>
    }
}
