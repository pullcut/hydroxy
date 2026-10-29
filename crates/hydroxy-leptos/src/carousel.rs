//! `<Carousel>`: slides in a CSS scroll-snap strip, with previous and next
//! buttons, dots and optional autoplay that pauses on hover and focus.

use std::time::Duration;

use hydroxy_headless::carousel::{self as h, CarouselCommand, CarouselEvent, CarouselInput, CarouselMachine};
use hydroxy_headless::icons::Icon as IconName;
use hydroxy_pacer::Debouncer;
use hydroxy_pacer::leptos::use_debounced_callback;
use hydroxy_statechart::Machine;
use leptos::html::Div;
use leptos::prelude::*;

use crate::drive::drive;
use crate::icon::Icon;

/// One slide: whatever it should show.
#[derive(Clone)]
pub struct Slide(ViewFn);

impl Slide {
    /// A slide showing `content`.
    pub fn new(content: impl Into<ViewFn>) -> Self {
        Self(content.into())
    }
}

/// How long after the last scroll event the strip counts as settled.
const SETTLE: Duration = Duration::from_millis(120);

/// A strip of slides. Swipe, scroll, use the arrow keys, or press the buttons and dots.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn Carousel(
    /// The slides, in order.
    slides: Vec<Slide>,
    /// Accessible name for the carousel.
    #[prop(into, default = "Carousel".to_string())]
    label: String,
    /// Controlled current slide. Pair with `on_index_change`.
    #[prop(optional, into)]
    index: MaybeProp<usize>,
    /// Initial slide when uncontrolled.
    #[prop(optional)]
    default_index: usize,
    /// Wrap from the last slide to the first, and back.
    #[prop(optional)]
    looping: bool,
    /// Advance every this many milliseconds. Pauses on hover and focus, has a
    /// pause button, and never starts for people who prefer reduced motion.
    #[prop(optional)]
    autoplay_ms: Option<u32>,
    /// Called with the new slide index as it changes.
    #[prop(optional, into)]
    on_index_change: Option<Callback<usize>>,
) -> impl IntoView {
    let count = slides.len();
    let input = Memo::new(move |_| CarouselInput { index: index.get(), default_index, count, looping, autoplay_ms });
    let state = RwSignal::new(input.with_untracked(CarouselMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CarouselMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let root_id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let viewport = NodeRef::<Div>::new();

    // The machine is re-entered from timers, so the sender has to be reachable from itself.
    let send_cell = StoredValue::new(None::<Callback<CarouselEvent>>);
    let send = Callback::new(move |event: CarouselEvent| {
        for command in drive::<CarouselMachine, _>(state, input, event) {
            match command {
                CarouselCommand::IndexChange { index, .. } => {
                    if let Some(cb) = on_index_change {
                        cb.run(index);
                    }
                }
                CarouselCommand::Wait { id, ms } => {
                    set_timeout(
                        move || {
                            if let Some(send) = send_cell.try_get_value().flatten() {
                                send.run(CarouselEvent::Elapsed(id));
                            }
                        },
                        Duration::from_millis(ms.into()),
                    );
                }
            }
        }
    });
    send_cell.set_value(Some(send));

    // Autoplay starts once mounted, and not at all under reduced motion.
    Effect::new(move |prev: Option<()>| {
        if prev.is_none() && !crate::js::prefers_reduced_motion() {
            send.run(CarouselEvent::Start);
        }
    });

    // The strip follows the index: buttons, dots, keys and autoplay all scroll it.
    // CSS `scroll-behavior` decides whether that animates.
    Effect::new(move |_| {
        let at = state.with(|s| s.index.value);
        if let Some(el) = viewport.get() {
            let width = f64::from(el.client_width());
            if h::scroll_index(f64::from(el.scroll_left()), width, count) != at {
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_precision_loss,
                    reason = "slide counts and pixel offsets are small"
                )]
                el.set_scroll_left((at as f64 * width) as i32);
            }
        }
    });

    // A swipe or wheel scroll moves the strip; once it settles the index follows.
    let settle = use_debounced_callback(Debouncer::new(SETTLE), move |()| {
        let Some(Some(el)) = viewport.try_get_untracked() else { return };
        let at = h::scroll_index(f64::from(el.scroll_left()), f64::from(el.client_width()), count);
        if state.try_with_untracked(|s| s.index.value).is_some_and(|i| i != at) {
            send.run(CarouselEvent::Go(at, h::ChangeReason::Pointer));
        }
    });
    let on_scroll = move |_| {
        settle.call(());
    };

    let slide_views = slides
        .into_iter()
        .enumerate()
        .map(|(i, slide)| {
            let attrs =
                Memo::new(move |_| input.with(|inp| state.with(|s| h::slide_attrs(&root_id.read_value(), s, inp, i))));
            view! {
                <div
                    role="group"
                    aria-roledescription="slide"
                    id=move || attrs.with(|x| x.id.clone())
                    aria-label=move || attrs.with(|x| x.aria_label.clone())
                    inert=move || attrs.with(|x| x.inert)
                    data-scope=h::SCOPE
                    data-part="slide"
                    data-state=move || attrs.with(|x| x.data_state)
                >
                    {slide.0.run()}
                </div>
            }
        })
        .collect_view();

    let dots = (0..count)
        .map(|i| {
            view! {
                <button
                    type="button"
                    aria-label=format!("Go to slide {}", i + 1)
                    aria-current=move || state.with(|s| s.index.value == i).then_some("true")
                    data-scope=h::SCOPE
                    data-part="dot"
                    on:click=move |_| send.run(CarouselEvent::Go(i, h::ChangeReason::Pointer))
                ></button>
            }
        })
        .collect_view();

    let (prev_off, next_off) = (
        move || input.with(|i| state.with(|s| h::prev_disabled(s, i))),
        move || input.with(|i| state.with(|s| h::next_disabled(s, i))),
    );

    view! {
        <section
            role="region"
            aria-roledescription="carousel"
            aria-label=label
            data-scope=h::SCOPE
            data-part="root"
            on:keydown=move |ev| {
                if let Some(event) = h::key_event(&ev.key(), count) {
                    ev.prevent_default();
                    send.run(event);
                }
            }
            on:pointerenter=move |_| send.run(CarouselEvent::Hover(true))
            on:pointerleave=move |_| send.run(CarouselEvent::Hover(false))
            on:focusin=move |_| send.run(CarouselEvent::Focus(true))
            on:focusout=move |_| send.run(CarouselEvent::Focus(false))
        >
            <div
                node_ref=viewport
                id=move || h::viewport_id(&root_id.read_value())
                tabindex="0"
                aria-live=move || state.with(h::aria_live)
                data-scope=h::SCOPE
                data-part="viewport"
                on:scroll=on_scroll
            >
                {slide_views}
            </div>
            <div data-scope=h::SCOPE data-part="controls">
                <button
                    type="button"
                    aria-label="Previous slide"
                    disabled=prev_off
                    data-scope=h::SCOPE
                    data-part="prev"
                    on:click=move |_| send.run(CarouselEvent::Prev(h::ChangeReason::Pointer))
                >
                    <Icon name=IconName::ChevronRight size=18 />
                </button>
                <div data-scope=h::SCOPE data-part="dots">{dots}</div>
                {autoplay_ms.map(|_| {
                    view! {
                        <button
                            type="button"
                            aria-label=move || state.with(h::play_label)
                            data-scope=h::SCOPE
                            data-part="play"
                            data-state=move || if state.with(|s| s.playing) { "playing" } else { "paused" }
                            on:click=move |_| send.run(CarouselEvent::TogglePlay)
                            inner_html=move || if state.with(|s| s.playing) { h::PAUSE_MARKUP } else { h::PLAY_MARKUP }
                        ></button>
                    }
                })}
                <button
                    type="button"
                    aria-label="Next slide"
                    disabled=next_off
                    data-scope=h::SCOPE
                    data-part="next"
                    on:click=move |_| send.run(CarouselEvent::Next(h::ChangeReason::Pointer))
                >
                    <Icon name=IconName::ChevronRight size=18 />
                </button>
            </div>
        </section>
    }
}
