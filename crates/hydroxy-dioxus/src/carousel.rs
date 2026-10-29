//! `Carousel`: slides in a CSS scroll-snap strip, with previous and next
//! buttons, dots and optional autoplay that pauses on hover and focus.

use dioxus::prelude::*;
use hydroxy_headless::carousel::{
    self as h, CarouselCommand, CarouselEvent, CarouselInput, CarouselMachine, CarouselState,
};
use hydroxy_headless::icons::Icon as IconName;
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::Icon;
use crate::js;

/// One slide: whatever it should show.
#[derive(Clone, PartialEq)]
pub struct Slide(Element);

impl Slide {
    /// A slide showing `content`.
    #[must_use]
    pub fn new(content: Element) -> Self {
        Self(content)
    }
}

/// How long after the last scroll event the strip counts as settled.
const SETTLE_MS: u32 = 120;

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<CarouselState>,
    input: Memo<CarouselInput>,
    on_index_change: Option<EventHandler<usize>>,
}

impl Ctx {
    fn send(self, event: CarouselEvent) {
        let commands = drive::<CarouselMachine>(self.state, self.input, event);
        for command in commands {
            match command {
                CarouselCommand::IndexChange { index, .. } => {
                    if let Some(handler) = self.on_index_change {
                        handler.call(index);
                    }
                }
                // The clock runs through eval, so it works on web and desktop alike.
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                CarouselCommand::Wait { id, ms } => {
                    spawn(async move {
                        js::sleep(ms).await;
                        self.send(CarouselEvent::Elapsed(id));
                    });
                }
            }
        }
    }
}

/// A strip of slides. Swipe, scroll, use the arrow keys, or press the buttons and dots.
#[component]
pub fn Carousel(
    /// The slides, in order.
    slides: Vec<Slide>,
    /// Accessible name for the carousel.
    #[props(default = "Carousel".to_string())]
    label: String,
    /// Controlled current slide. Pair with `on_index_change`.
    #[props(default)]
    index: ReadSignal<Option<usize>>,
    /// Initial slide when uncontrolled.
    #[props(default)]
    default_index: usize,
    /// Wrap from the last slide to the first, and back.
    #[props(default)]
    looping: bool,
    /// Advance every this many milliseconds. Pauses on hover and focus, has a
    /// pause button, and never starts for people who prefer reduced motion.
    autoplay_ms: Option<u32>,
    /// Called with the new slide index as it changes.
    on_index_change: Option<EventHandler<usize>>,
) -> Element {
    let count = slides.len();
    let input = use_memo(move || CarouselInput { index: index(), default_index, count, looping, autoplay_ms });
    let mut state = use_signal(|| CarouselMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CarouselMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, on_index_change };

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let viewport = h::viewport_id(&root);

    // Autoplay starts once mounted, and not at all under reduced motion.
    use_future(move || async move {
        if !js::prefers_reduced_motion().await {
            ctx.send(CarouselEvent::Start);
        }
    });

    // The strip follows the index: buttons, dots, keys and autoplay all scroll it.
    // CSS `scroll-behavior` decides whether that animates.
    let follow = viewport.clone();
    use_effect(move || {
        let at = state.read().index.value;
        let _ = document::eval(&format!(
            "const el = document.getElementById({follow}); \
             if (el && el.clientWidth && Math.round(el.scrollLeft / el.clientWidth) !== {at}) el.scrollLeft = {at} * el.clientWidth;",
            follow = crate::js::js_string(&follow)
        ));
    });

    // A swipe or wheel scroll moves the strip; once it settles the index follows.
    let mut scroll_generation = use_signal(|| 0_u32);
    let on_scroll = move |e: ScrollEvent| {
        let (left, width) = (e.scroll_left(), f64::from(e.client_width()));
        let generation = scroll_generation.peek().wrapping_add(1);
        scroll_generation.set(generation);
        spawn(async move {
            js::sleep(SETTLE_MS).await;
            if *scroll_generation.peek() != generation {
                return;
            }
            let at = h::scroll_index(left, width, count);
            if state.peek().index.value != at {
                ctx.send(CarouselEvent::Go(at, h::ChangeReason::Pointer));
            }
        });
    };

    let s = state.read().clone();
    let inp = input.read().clone();
    let (prev_off, next_off) = (h::prev_disabled(&s, &inp), h::next_disabled(&s, &inp));

    rsx! {
        section {
            role: "region",
            "aria-roledescription": "carousel",
            "aria-label": label,
            "data-scope": h::SCOPE,
            "data-part": "root",
            onkeydown: move |e| {
                if let Some(event) = h::key_event(&e.key().to_string(), count) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            onpointerenter: move |_| ctx.send(CarouselEvent::Hover(true)),
            onpointerleave: move |_| ctx.send(CarouselEvent::Hover(false)),
            onfocusin: move |_| ctx.send(CarouselEvent::Focus(true)),
            onfocusout: move |_| ctx.send(CarouselEvent::Focus(false)),
            div {
                id: viewport,
                tabindex: "0",
                "aria-live": h::aria_live(&s),
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                onscroll: on_scroll,
                for (i, slide) in slides.into_iter().enumerate() {
                    {
                        let a = h::slide_attrs(&root, &s, &inp, i);
                        rsx! {
                            div {
                                key: "slide-{i}",
                                role: "group",
                                "aria-roledescription": "slide",
                                id: a.id,
                                "aria-label": a.aria_label,
                                "inert": a.inert.then_some(""),
                                "data-scope": h::SCOPE,
                                "data-part": "slide",
                                "data-state": a.data_state,
                                {slide.0}
                            }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "controls",
                button {
                    r#type: "button",
                    "aria-label": "Previous slide",
                    disabled: prev_off,
                    "data-scope": h::SCOPE,
                    "data-part": "prev",
                    onclick: move |_| ctx.send(CarouselEvent::Prev(h::ChangeReason::Pointer)),
                    Icon { name: IconName::ChevronRight, size: 18 }
                }
                div { "data-scope": h::SCOPE, "data-part": "dots",
                    for i in 0..count {
                        button {
                            key: "dot-{i}",
                            r#type: "button",
                            "aria-label": "Go to slide {i + 1}",
                            "aria-current": (s.index.value == i).then_some("true"),
                            "data-scope": h::SCOPE,
                            "data-part": "dot",
                            onclick: move |_| ctx.send(CarouselEvent::Go(i, h::ChangeReason::Pointer)),
                        }
                    }
                }
                if autoplay_ms.is_some() {
                    button {
                        r#type: "button",
                        "aria-label": h::play_label(&s),
                        "data-scope": h::SCOPE,
                        "data-part": "play",
                        "data-state": if s.playing { "playing" } else { "paused" },
                        onclick: move |_| ctx.send(CarouselEvent::TogglePlay),
                        dangerous_inner_html: if s.playing { h::PAUSE_MARKUP } else { h::PLAY_MARKUP },
                    }
                }
                button {
                    r#type: "button",
                    "aria-label": "Next slide",
                    disabled: next_off,
                    "data-scope": h::SCOPE,
                    "data-part": "next",
                    onclick: move |_| ctx.send(CarouselEvent::Next(h::ChangeReason::Pointer)),
                    Icon { name: IconName::ChevronRight, size: 18 }
                }
            }
        }
    }
}
