//! `<MediaTimeline>`: a scrubber over a native `<audio>` or `<video>` element.

use crate::drive::drive;
use crate::icon::{Icon, IconName};
use hydroxy_headless::media_timeline::{
    self as h, MediaTimelineCommand, MediaTimelineEvent, MediaTimelineInput, MediaTimelineMachine, MediaTimelineState,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

pub use h::{Marker, MediaKind};

/// The media element with this id, if it is mounted.
fn media(id: &str) -> Option<web_sys::HtmlMediaElement> {
    document().get_element_by_id(id)?.dyn_into().ok()
}

/// A play button, a scrubber with chapter markers, and a native media element
/// underneath. Click or drag the track to seek; the thumb takes arrow keys.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "one view! tree; props are owned by the macro"
)]
pub fn MediaTimeline(
    /// URL of the audio or video file.
    #[prop(into)]
    src: String,
    /// Audio (default) or video. Video shows the picture above the controls.
    #[prop(optional)]
    kind: MediaKind,
    /// Accessible name for the scrubber, such as "Episode 12".
    #[prop(into)]
    label: String,
    /// Chapters or points of interest, as ticks on the track.
    #[prop(optional)]
    markers: Vec<Marker>,
    /// Seconds an arrow key moves. Default 5.
    #[prop(default = 5.0)]
    step: f64,
    /// Block seeking and playback controls.
    #[prop(optional, into)]
    disabled: Signal<bool>,
) -> impl IntoView {
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| MediaTimelineInput { step, disabled: disabled.get() });
    let state = RwSignal::new(input.with_untracked(MediaTimelineMachine::initial));
    let markers = StoredValue::new(markers);

    let send = {
        move |event: MediaTimelineEvent| {
            let commands = drive::<MediaTimelineMachine, _>(state, input, event);
            let Some(el) = id.try_get_value().and_then(|id| media(&id)) else { return };
            for command in commands {
                match command {
                    MediaTimelineCommand::Seek(t) => el.set_current_time(t),
                    MediaTimelineCommand::Play => {
                        let _ = el.play();
                    }
                    MediaTimelineCommand::Pause => {
                        let _ = el.pause();
                    }
                }
            }
        }
    };
    // Mirror the media element into the machine. Runs on every media event and once on mount.
    let sync = {
        move || {
            let Some(el) = id.try_get_value().and_then(|id| media(&id)) else { return };
            send(MediaTimelineEvent::Duration(el.duration()));
            send(MediaTimelineEvent::Time(el.current_time()));
            send(MediaTimelineEvent::Playing(!el.paused() && !el.ended()));
            let ranges = el.buffered();
            if ranges.length() > 0
                && let Ok(end) = ranges.end(ranges.length() - 1)
            {
                send(MediaTimelineEvent::Buffered(end));
            }
        }
    };
    Effect::new(move |_| sync());

    let at = move |ev: &web_sys::PointerEvent| -> Option<f64> {
        let el = ev.current_target()?.dyn_into::<web_sys::Element>().ok()?;
        let rect = el.get_bounding_client_rect();
        let fraction = (f64::from(ev.client_x()) - rect.left()) / rect.width().max(1.0);
        Some(h::time_at(fraction, state.with_untracked(|s| s.duration)))
    };
    let down = move |ev: web_sys::PointerEvent| {
        if let Some(t) = at(&ev) {
            if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                let _ = el.set_pointer_capture(ev.pointer_id());
            }
            send(MediaTimelineEvent::ScrubStart(t));
        }
    };
    let moved = move |ev: web_sys::PointerEvent| {
        if let Some(t) = at(&ev) {
            send(MediaTimelineEvent::Hover(Some(t)));
            send(MediaTimelineEvent::ScrubMove(t));
        }
    };

    let percent =
        move |seconds: f64| state.with(|s: &MediaTimelineState| hydroxy_charts::plot::pct(s.fraction(seconds)));
    let on_media = move |_: web_sys::Event| sync();
    let on_progress = move |_: web_sys::ProgressEvent| sync();
    let element = match kind {
        MediaKind::Audio => Either::Left(view! {
            <audio
                id=id.get_value()
                data-scope=h::SCOPE
                data-part="media"
                src=src.clone()
                preload="metadata"
                on:timeupdate=on_media
                on:durationchange=on_media
                on:loadedmetadata=on_media
                on:progress=on_progress
                on:play=on_media
                on:pause=on_media
                on:ended=on_media
            ></audio>
        }),
        MediaKind::Video => Either::Right(view! {
            <video
                id=id.get_value()
                data-scope=h::SCOPE
                data-part="media"
                src=src.clone()
                preload="metadata"
                playsinline=true
                on:timeupdate=on_media
                on:durationchange=on_media
                on:loadedmetadata=on_media
                on:progress=on_progress
                on:play=on_media
                on:pause=on_media
                on:ended=on_media
            ></video>
        }),
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-kind=match kind { MediaKind::Audio => "audio", MediaKind::Video => "video" }
            data-state=move || if state.with(|s| s.playing) { "playing" } else { "paused" }
            data-disabled=move || disabled.get().then_some("")
        >
            {element}
            <div data-scope=h::SCOPE data-part="controls">
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="play"
                    aria-label=move || h::play_label(state.with(|s| s.playing))
                    disabled=move || disabled.get()
                    on:click=move |_| send(MediaTimelineEvent::TogglePlay)
                >
                    {move || {
                        let name = if state.with(|s| s.playing) { IconName::Pause } else { IconName::Play };
                        view! { <Icon name=name size=18 /> }
                    }}
                </button>
                <span data-scope=h::SCOPE data-part="current">{move || h::format_time(state.with(|s| s.current))}</span>
                <div
                    data-scope=h::SCOPE
                    data-part="track"
                    data-scrubbing=move || state.with(|s| s.scrubbing).then_some("")
                    on:pointerdown=down
                    on:pointermove=moved
                    on:pointerup=move |_| send(MediaTimelineEvent::ScrubEnd)
                    on:pointercancel=move |_| send(MediaTimelineEvent::ScrubEnd)
                    on:pointerleave=move |_| send(MediaTimelineEvent::Hover(None))
                >
                    <span data-scope=h::SCOPE data-part="buffered" style:width=move || percent(state.with(|s| s.buffered))></span>
                    <span data-scope=h::SCOPE data-part="played" style:width=move || percent(state.with(|s| s.current))></span>
                    {markers
                        .get_value()
                        .into_iter()
                        .map(|m| {
                            view! {
                                <span
                                    data-scope=h::SCOPE
                                    data-part="marker"
                                    title=m.label.clone()
                                    style:left=move || percent(m.time)
                                ></span>
                            }
                        })
                        .collect_view()}
                    {move || {
                        state
                            .with(|s| s.hover)
                            .map(|t| {
                                view! {
                                    <span data-scope=h::SCOPE data-part="preview" style:left=percent(t)>
                                        {h::format_time(t)}
                                    </span>
                                }
                            })
                    }}
                    <div
                        data-scope=h::SCOPE
                        data-part="thumb"
                        role="slider"
                        tabindex=move || if disabled.get() { "-1" } else { "0" }
                        aria-label=label
                        aria-valuemin="0"
                        aria-valuemax=move || state.with(|s| s.duration).to_string()
                        aria-valuenow=move || state.with(|s| s.current).floor().to_string()
                        aria-valuetext=move || state.with(|s| h::value_text(s.current, s.duration))
                        aria-disabled=move || disabled.get().then_some("true")
                        style:left=move || percent(state.with(|s| s.current))
                        on:keydown=move |ev| {
                            if let Some(event) = h::keydown(&ev.key(), ev.shift_key(), step) {
                                ev.prevent_default();
                                send(event);
                            }
                        }
                    ></div>
                </div>
                <span data-scope=h::SCOPE data-part="duration">{move || h::format_time(state.with(|s| s.duration))}</span>
            </div>
            <p data-scope=h::SCOPE data-part="chapter" hidden=move || markers.with_value(Vec::is_empty)>
                {move || {
                    let current = state.with(|s| s.current);
                    markers.with_value(|m| h::chapter_at(m, current).map(|i| m[i].label.clone()))
                }}
            </p>
        </div>
    }
}
