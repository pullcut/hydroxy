//! `LogStream`: a monospace log viewer. Colours lines by level, follows the tail until
//! you scroll away, holds new lines while paused, and filters by level and text.

use dioxus::prelude::*;
use hydroxy_headless::log_stream::{
    self as h, LogEntry, LogLevel, LogStreamCommand, LogStreamEvent, LogStreamInput, LogStreamMachine,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::drive::drive;
use crate::search_field::{SearchField, Size};

pub use hydroxy_headless::log_stream::{LogEntry as Entry, LogLevel as Level};

/// A live log. Feed it batches through `entries`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn LogStream(
    /// Each time this changes, its entries are appended. Set it to the lines that just
    /// arrived, not the whole log.
    #[props(default)]
    entries: ReadSignal<Vec<LogEntry>>,
    /// Most lines kept; the oldest are dropped first.
    #[props(default = 1000)]
    max_lines: usize,
    /// Controlled pause. Pair with `on_paused_change`.
    #[props(default)]
    paused: ReadSignal<Option<bool>>,
    /// Initial pause when uncontrolled.
    #[props(default)]
    default_paused: bool,
    /// Called with the new value when the pause button is pressed.
    on_paused_change: Option<EventHandler<bool>>,
    /// Accessible name for the log.
    #[props(default = "Logs".to_string())]
    label: String,
    /// Text shown when no line passes the filters.
    #[props(default = "No log lines.".to_string())]
    empty: String,
) -> Element {
    let input = use_memo(move || LogStreamInput { max_lines, paused: paused(), default_paused });
    let mut state = use_signal(|| LogStreamMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = LogStreamMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let id = use_signal(|| hydroxy_headless::next_id(h::SCOPE));
    let to_end = move || {
        let dom_id = id.cloned();
        spawn(async move {
            // Wait a tick for the new lines to render. `{dom_id:?}` quotes the id as a JS
            // string; ids from `next_id` are plain ASCII.
            let _ = document::eval(&format!(
                "await new Promise((r) => setTimeout(r, 0));
                const el = document.getElementById({dom_id:?});
                if (el) el.scrollTop = el.scrollHeight;"
            ))
            .await;
        });
    };
    let send = move |event: LogStreamEvent| {
        let relayout =
            matches!(event, LogStreamEvent::ToggleLevel(_) | LogStreamEvent::SetQuery(_) | LogStreamEvent::ToggleWrap);
        let commands = drive::<LogStreamMachine>(state, input, event);
        let following = state.peek().following;
        if relayout && following {
            to_end();
        }
        for command in commands {
            match command {
                LogStreamCommand::ScrollToEnd => to_end(),
                LogStreamCommand::PausedChange(p) => {
                    if let Some(handler) = on_paused_change {
                        handler.call(p);
                    }
                }
            }
        }
    };
    use_effect(move || {
        let batch = entries.read().clone();
        if !batch.is_empty() {
            send(LogStreamEvent::Push(batch));
        }
    });

    let s = state.read();
    let a = h::viewport_attrs(&s);
    let lines = s.visible();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-state": a.data_state,
            div { "data-scope": h::SCOPE, "data-part": "toolbar",
                div { "data-scope": h::SCOPE, "data-part": "levels", role: "group", "aria-label": "Levels",
                    for level in LogLevel::ALL {
                        button {
                            key: "{level.as_str()}",
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "level",
                            "data-level": level.as_str(),
                            "aria-pressed": s.shows(level).to_string(),
                            onclick: move |_| send(LogStreamEvent::ToggleLevel(level)),
                            "{level.tag()}"
                        }
                    }
                }
                div { "data-scope": h::SCOPE, "data-part": "search",
                    SearchField {
                        label: "Filter log text",
                        placeholder: "Filter",
                        size: Size::Sm,
                        on_input: move |q: String| send(LogStreamEvent::SetQuery(q)),
                    }
                }
                div { "data-scope": h::SCOPE, "data-part": "actions",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        on_click: move |_| {
                            let paused = state.peek().paused.value;
                            send(if paused { LogStreamEvent::Resume } else { LogStreamEvent::Pause });
                        },
                        "{h::pause_label(&s)}"
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "toggle",
                        "aria-pressed": s.following.to_string(),
                        onclick: move |_| {
                            let following = state.peek().following;
                            send(LogStreamEvent::SetFollowing(!following));
                        },
                        "Follow"
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "toggle",
                        "aria-pressed": s.wrap.to_string(),
                        onclick: move |_| send(LogStreamEvent::ToggleWrap),
                        "Wrap"
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "toggle",
                        onclick: move |_| send(LogStreamEvent::Clear),
                        "Clear"
                    }
                }
            }
            div {
                id: id.cloned(),
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                "data-wrap": s.wrap.then_some(""),
                role: a.role,
                "aria-live": a.aria_live,
                "aria-label": "{label}",
                tabindex: a.tabindex,
                onscroll: move |e| {
                    let data = e.data();
                    let end = h::at_end(
                        f64::from(data.scroll_height()),
                        data.scroll_top(),
                        f64::from(data.client_height()),
                    );
                    let following = state.peek().following;
                    if end != following {
                        send(LogStreamEvent::SetFollowing(end));
                    }
                },
                for l in lines.iter() {
                    div {
                        key: "{l.id}",
                        "data-scope": h::SCOPE,
                        "data-part": "line",
                        "data-level": l.level.as_str(),
                        span { "data-scope": h::SCOPE, "data-part": "time", "{l.time}" }
                        span { "data-scope": h::SCOPE, "data-part": "tag", "{l.level.tag()}" }
                        span { "data-scope": h::SCOPE, "data-part": "text", "{l.text}" }
                    }
                }
                if lines.is_empty() {
                    p { "data-scope": h::SCOPE, "data-part": "empty", "{empty}" }
                }
            }
        }
    }
}
