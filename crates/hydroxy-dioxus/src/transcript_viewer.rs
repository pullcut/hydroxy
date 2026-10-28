//! `TranscriptViewer`: timed speech segments that follow playback, seek on
//! click and filter by search. Following pauses when the reader scrolls.
//!
//! Hand-scroll detection and scrolling the playing segment into view run in
//! the webview through `document::eval`.

use dioxus::prelude::*;
use hydroxy_headless::transcript_viewer::{
    self as h, Segment, TranscriptViewerCommand, TranscriptViewerEvent, TranscriptViewerInput, TranscriptViewerMachine,
};
use hydroxy_statechart::Machine;

pub use h::Segment as TranscriptSegment;

/// Reports a hand scroll: the wheel, touch, a scroll key on the viewport itself, or a scrollbar drag.
const WATCH: &str = r"
const el = document.getElementById(await dioxus.recv());
if (el) {
  const keys = ['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' '];
  const tell = () => dioxus.send(true);
  el.addEventListener('wheel', tell, { passive: true });
  el.addEventListener('touchmove', tell, { passive: true });
  el.addEventListener('keydown', (e) => { if (e.target === el && keys.includes(e.key)) tell(); });
  el.addEventListener('pointerdown', (e) => { if (e.target === el) tell(); });
}
";

/// Scroll the viewport the least distance that shows the segment.
fn reveal(viewport: &str, segment: &str) {
    let _ = document::eval(&format!(
        "const v = document.getElementById({viewport:?}), s = document.getElementById({segment:?}); \
         if (v && s) {{ const a = v.getBoundingClientRect(), b = s.getBoundingClientRect(); \
           if (b.top < a.top) v.scrollTop += b.top - a.top; else if (b.bottom > a.bottom) v.scrollTop += b.bottom - a.bottom; }}"
    ));
}

/// A scrolling transcript. Pass the segments and the player's time; clicking a
/// segment calls `on_seek` with its start. Scrolling by hand stops the view
/// following playback until the reader presses "Follow playback".
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn TranscriptViewer(
    /// The segments in time order.
    segments: Vec<Segment>,
    /// Playback position in seconds. The playing segment is highlighted.
    #[props(default)]
    current_time: ReadSignal<f64>,
    /// A segment was chosen: seek here, in seconds.
    on_seek: Option<EventHandler<f64>>,
    /// Show a search field.
    #[props(default)]
    searchable: bool,
    /// Accessible name of the scrolling region. Default "Transcript".
    label: Option<String>,
    /// Height of the scrolling region in pixels. Default 320.
    #[props(default = h::MAX_HEIGHT)]
    max_height: u32,
    /// Start with following off.
    #[props(default)]
    start_suspended: bool,
    /// Following turned on or off.
    on_follow_change: Option<EventHandler<bool>>,
) -> Element {
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let viewport_id = format!("{root}-viewport");
    let input = TranscriptViewerInput { start_suspended };
    let mut state = use_signal(|| TranscriptViewerMachine::initial(&input));
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| None::<String>);

    let active = h::active_segment(&segments, current_time()).map(str::to_string);
    let send = {
        let (root, viewport_id, active) = (root.clone(), viewport_id.clone(), active.clone());
        Callback::new(move |event: TranscriptViewerEvent| {
            let step = TranscriptViewerMachine::transition(&state.peek(), event, &input);
            if let Some(next) = step.state {
                state.set(next);
            }
            for command in step.commands {
                match command {
                    TranscriptViewerCommand::FollowChange(on) => {
                        if let Some(handler) = on_follow_change {
                            handler.call(on);
                        }
                    }
                    TranscriptViewerCommand::ScrollToActive => {
                        if let Some(id) = &active {
                            reveal(&viewport_id, &h::segment_dom_id(&root, id));
                        }
                    }
                }
            }
        })
    };
    // Follow playback: when the playing segment changes and the view is following, bring it into view.
    use_effect({
        let (root, viewport_id, active) = (root.clone(), viewport_id.clone(), active.clone());
        move || {
            if let Some(id) = &active
                && state.read().following
            {
                reveal(&viewport_id, &h::segment_dom_id(&root, id));
            }
        }
    });
    let watched = viewport_id.clone();
    use_future(move || {
        let watched = watched.clone();
        async move {
            let mut eval = document::eval(WATCH);
            let _ = eval.send(watched);
            while eval.recv::<bool>().await.is_ok() {
                send.call(TranscriptViewerEvent::UserScroll);
            }
        }
    });

    let shown = h::filter(&segments, &query.read());
    let count = h::count_text(shown.len(), segments.len());
    let following = state.read().following;
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-follow": if following { "following" } else { "suspended" },
            div { "data-scope": h::SCOPE, "data-part": "toolbar",
                if searchable {
                    input {
                        r#type: "search",
                        "data-scope": h::SCOPE,
                        "data-part": "search",
                        placeholder: "Search",
                        "aria-label": h::SEARCH_LABEL,
                        value: "{query}",
                        oninput: move |e| query.set(e.value()),
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "count", role: "status", "{count}" }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "resume-follow",
                    disabled: following,
                    onclick: move |_| send.call(TranscriptViewerEvent::Resume),
                    {h::RESUME_LABEL}
                }
            }
            div {
                id: "{viewport_id}",
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                role: "region",
                tabindex: "0",
                "aria-label": label,
                style: "max-height: {max_height}px",
                ol { "data-scope": h::SCOPE, "data-part": "list",
                    for segment in shown {
                        {
                            let id = segment.id.clone();
                            let is_active = active.as_deref() == Some(id.as_str());
                            let is_selected = selected.read().as_deref() == Some(id.as_str());
                            let start = h::seek_time(segment);
                            rsx! {
                                li { key: "{id}", "data-scope": h::SCOPE, "data-part": "item",
                                    button {
                                        r#type: "button",
                                        id: h::segment_dom_id(&root, &id),
                                        "data-scope": h::SCOPE,
                                        "data-part": "segment",
                                        "data-active": is_active.then_some("true"),
                                        "data-selected": is_selected.then_some("true"),
                                        "aria-current": is_active.then_some("true"),
                                        onclick: move |_| {
                                            selected.set(Some(id.clone()));
                                            if let Some(handler) = on_seek {
                                                handler.call(start);
                                            }
                                        },
                                        span { "data-scope": h::SCOPE, "data-part": "speaker", "{segment.speaker}" }
                                        span { "data-scope": h::SCOPE, "data-part": "time", {h::format_time(segment.start)} }
                                        span { "data-scope": h::SCOPE, "data-part": "text", "{segment.text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
