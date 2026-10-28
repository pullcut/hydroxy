//! `<TranscriptViewer>`: timed speech segments that follow playback, seek on
//! click and filter by search. Following pauses when the reader scrolls.

use leptos::prelude::*;
use oxide_headless::transcript_viewer::{
    self as h, Segment, TranscriptViewerCommand, TranscriptViewerEvent, TranscriptViewerInput,
    TranscriptViewerMachine, TranscriptViewerState,
};
use oxide_statechart::Machine;

pub use h::Segment as TranscriptSegment;

/// Scroll `viewport` the least distance that brings `segment` fully into view.
fn reveal(viewport_id: &str, segment_id: &str) {
    let doc = document();
    let (Some(viewport), Some(segment)) = (doc.get_element_by_id(viewport_id), doc.get_element_by_id(segment_id)) else {
        return;
    };
    let (v, s) = (viewport.get_bounding_client_rect(), segment.get_bounding_client_rect());
    let top = s.top() - v.top();
    let bottom = s.bottom() - v.bottom();
    #[allow(clippy::cast_possible_truncation, reason = "pixel offsets")]
    if top < 0.0 {
        viewport.set_scroll_top(viewport.scroll_top() + top.floor() as i32);
    } else if bottom > 0.0 {
        viewport.set_scroll_top(viewport.scroll_top() + bottom.ceil() as i32);
    }
}

/// A scrolling transcript. Pass the segments and the player's time; clicking a
/// segment calls `on_seek` with its start. Scrolling by hand stops the view
/// following playback until the reader presses "Follow playback".
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
pub fn TranscriptViewer(
    /// The segments in time order.
    #[prop(into)]
    segments: Signal<Vec<Segment>>,
    /// Playback position in seconds. The playing segment is highlighted.
    #[prop(optional, into)]
    current_time: Signal<f64>,
    /// A segment was chosen: seek here, in seconds.
    #[prop(optional, into)]
    on_seek: Option<Callback<f64>>,
    /// Show a search field.
    #[prop(optional)]
    searchable: bool,
    /// Accessible name of the scrolling region. Default "Transcript".
    #[prop(optional, into)]
    label: Option<String>,
    /// Height of the scrolling region in pixels. Default 320.
    #[prop(default = h::MAX_HEIGHT)]
    max_height: u32,
    /// Start with following off.
    #[prop(optional)]
    start_suspended: bool,
    /// Following turned on or off.
    #[prop(optional, into)]
    on_follow_change: Option<Callback<bool>>,
) -> impl IntoView {
    let root = StoredValue::new(crate::id::next_id(h::SCOPE));
    let viewport_id = format!("{}-viewport", root.get_value());
    let viewport_id = StoredValue::new(viewport_id);
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    let input = TranscriptViewerInput { start_suspended };
    let state = RwSignal::new(TranscriptViewerMachine::initial(&input));
    let query = RwSignal::new(String::new());
    let selected = RwSignal::new(None::<String>);

    let active = Memo::new(move |_| segments.with(|s| h::active_segment(s, current_time.get()).map(str::to_string)));
    let scroll_to_active = move || {
        if let Some(id) = active.get_untracked() {
            reveal(&viewport_id.get_value(), &h::segment_dom_id(&root.get_value(), &id));
        }
    };
    let send = move |event: TranscriptViewerEvent| {
        let Some(step) = state.try_with_untracked(|s| TranscriptViewerMachine::transition(s, event, &input)) else { return };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                TranscriptViewerCommand::FollowChange(on) => {
                    if let Some(cb) = on_follow_change {
                        cb.run(on);
                    }
                }
                TranscriptViewerCommand::ScrollToActive => scroll_to_active(),
            }
        }
    };
    Effect::new(move |_| {
        let id = active.get();
        if let Some(id) = id
            && state.with_untracked(|s: &TranscriptViewerState| s.following)
        {
            reveal(&viewport_id.get_value(), &h::segment_dom_id(&root.get_value(), &id));
        }
    });

    let visible = move || {
        segments.with(|all| {
            let q = query.get();
            h::filter(all, &q).into_iter().cloned().collect::<Vec<_>>()
        })
    };
    let following = move || state.with(|s| s.following);

    view! {
        <div data-scope=h::SCOPE data-part="root" data-follow=move || if following() { "following" } else { "suspended" }>
            <div data-scope=h::SCOPE data-part="toolbar">
                {searchable
                    .then(|| {
                        view! {
                            <input
                                type="search"
                                data-scope=h::SCOPE
                                data-part="search"
                                placeholder="Search"
                                aria-label=h::SEARCH_LABEL
                                prop:value=move || query.get()
                                on:input=move |ev| query.set(event_target_value(&ev))
                            />
                        }
                    })}
                <span data-scope=h::SCOPE data-part="count" role="status">
                    {move || segments.with(|all| h::count_text(visible().len(), all.len()))}
                </span>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="resume-follow"
                    disabled=following
                    on:click=move |_| send(TranscriptViewerEvent::Resume)
                >
                    {h::RESUME_LABEL}
                </button>
            </div>
            <div
                id=viewport_id.get_value()
                data-scope=h::SCOPE
                data-part="viewport"
                role="region"
                tabindex="0"
                aria-label=label
                style=format!("max-height: {max_height}px")
                on:wheel=move |_| send(TranscriptViewerEvent::UserScroll)
                on:touchmove=move |_| send(TranscriptViewerEvent::UserScroll)
                on:keydown=move |ev| {
                    if h::is_scroll_key(&ev.key()) && ev.target() == ev.current_target() {
                        send(TranscriptViewerEvent::UserScroll);
                    }
                }
                // Dragging the scrollbar targets the viewport itself.
                on:pointerdown=move |ev| {
                    if ev.target() == ev.current_target() {
                        send(TranscriptViewerEvent::UserScroll);
                    }
                }
            >
                <ol data-scope=h::SCOPE data-part="list">
                    <For each=visible key=|s| s.id.clone() let:segment>
                        {
                            let id = segment.id.clone();
                            let is_active = {
                                let id = id.clone();
                                Memo::new(move |_| active.with(|a| a.as_deref() == Some(id.as_str())))
                            };
                            let is_selected = {
                                let id = id.clone();
                                Memo::new(move |_| selected.with(|s| s.as_deref() == Some(id.as_str())))
                            };
                            let start = h::seek_time(&segment);
                            view! {
                                <li data-scope=h::SCOPE data-part="item">
                                    <button
                                        type="button"
                                        id=h::segment_dom_id(&root.get_value(), &id)
                                        data-scope=h::SCOPE
                                        data-part="segment"
                                        data-active=move || is_active.get().then_some("true")
                                        data-selected=move || is_selected.get().then_some("true")
                                        aria-current=move || is_active.get().then_some("true")
                                        on:click=move |_| {
                                            selected.set(Some(id.clone()));
                                            if let Some(cb) = on_seek {
                                                cb.run(start);
                                            }
                                        }
                                    >
                                        <span data-scope=h::SCOPE data-part="speaker">{segment.speaker.clone()}</span>
                                        <span data-scope=h::SCOPE data-part="time">{h::format_time(segment.start)}</span>
                                        <span data-scope=h::SCOPE data-part="text">{segment.text.clone()}</span>
                                    </button>
                                </li>
                            }
                        }
                    </For>
                </ol>
            </div>
        </div>
    }
}
