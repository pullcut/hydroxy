//! `<LogStream>`: a monospace log viewer. Colours lines by level, follows the tail until
//! you scroll away, holds new lines while paused, and filters by level and text.

use std::time::Duration;

use leptos::html::Div;
use leptos::prelude::*;
use oxide_headless::log_stream::{
    self as h, LogEntry, LogLevel, LogStreamCommand, LogStreamEvent, LogStreamInput, LogStreamMachine,
};
use oxide_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::search_field::{SearchField, Size};

pub use oxide_headless::log_stream::{LogEntry as Entry, LogLevel as Level};

/// A live log. Feed it batches through `entries`.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn LogStream(
    /// Each time this changes, its entries are appended. Set it to the lines that just
    /// arrived, not the whole log.
    #[prop(optional, into)]
    entries: Signal<Vec<LogEntry>>,
    /// Most lines kept; the oldest are dropped first.
    #[prop(default = 1000)]
    max_lines: usize,
    /// Controlled pause. Pair with `on_paused_change`.
    #[prop(optional, into)]
    paused: MaybeProp<bool>,
    /// Initial pause when uncontrolled.
    #[prop(optional)]
    default_paused: bool,
    /// Called with the new value when the pause button is pressed.
    #[prop(optional, into)]
    on_paused_change: Option<Callback<bool>>,
    /// Accessible name for the log.
    #[prop(default = "Logs".into(), into)]
    label: String,
    /// Text shown when no line passes the filters.
    #[prop(default = "No log lines.".into(), into)]
    empty: String,
) -> impl IntoView {
    let input = Memo::new(move |_| LogStreamInput { max_lines, paused: paused.get(), default_paused });
    let state = RwSignal::new(input.with_untracked(LogStreamMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| LogStreamMachine::sync(s, i)));
        state.set(synced);
    });
    let viewport = NodeRef::<Div>::new();
    let to_end = move || {
        // Wait for the new lines to render before measuring.
        set_timeout(
            move || {
                if let Some(el) = viewport.try_get_untracked().flatten() {
                    el.set_scroll_top(el.scroll_height());
                }
            },
            Duration::ZERO,
        );
    };
    let send = move |event: LogStreamEvent| {
        let relayout = matches!(
            event,
            LogStreamEvent::ToggleLevel(_) | LogStreamEvent::SetQuery(_) | LogStreamEvent::ToggleWrap
        );
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| LogStreamMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        let following = step.state.following;
        state.set(step.state);
        if relayout && following {
            to_end();
        }
        for command in step.commands {
            match command {
                LogStreamCommand::ScrollToEnd => to_end(),
                LogStreamCommand::PausedChange(p) => {
                    if let Some(cb) = on_paused_change {
                        cb.run(p);
                    }
                }
            }
        }
    };
    Effect::new(move |_| {
        let batch = entries.get();
        if !batch.is_empty() {
            send(LogStreamEvent::Push(batch));
        }
    });

    let lines = Memo::new(move |_| state.with(|s| s.visible().into_iter().cloned().collect::<Vec<_>>()));
    let attrs = Memo::new(move |_| state.with(h::viewport_attrs));

    view! {
        <div data-scope=h::SCOPE data-part="root" data-state=move || attrs.get().data_state>
            <div data-scope=h::SCOPE data-part="toolbar">
                <div data-scope=h::SCOPE data-part="levels" role="group" aria-label="Levels">
                    {LogLevel::ALL
                        .into_iter()
                        .map(|level| {
                            view! {
                                <button
                                    type="button"
                                    data-scope=h::SCOPE
                                    data-part="level"
                                    data-level=level.as_str()
                                    aria-pressed=move || state.with(|s| s.shows(level)).to_string()
                                    on:click=move |_| send(LogStreamEvent::ToggleLevel(level))
                                >
                                    {level.tag()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <div data-scope=h::SCOPE data-part="search">
                    <SearchField
                        label="Filter log text"
                        placeholder="Filter"
                        size=Size::Sm
                        on_input=move |q: String| send(LogStreamEvent::SetQuery(q))
                    />
                </div>
                <div data-scope=h::SCOPE data-part="actions">
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=move |_| {
                            let paused = state.with(|s| s.paused);
                            send(if paused { LogStreamEvent::Resume } else { LogStreamEvent::Pause });
                        }
                    >
                        {move || state.with(h::pause_label)}
                    </Button>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="toggle"
                        aria-pressed=move || state.with(|s| s.following).to_string()
                        on:click=move |_| send(LogStreamEvent::SetFollowing(!state.with(|s| s.following)))
                    >
                        "Follow"
                    </button>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="toggle"
                        aria-pressed=move || state.with(|s| s.wrap).to_string()
                        on:click=move |_| send(LogStreamEvent::ToggleWrap)
                    >
                        "Wrap"
                    </button>
                    <button type="button" data-scope=h::SCOPE data-part="toggle" on:click=move |_| send(LogStreamEvent::Clear)>
                        "Clear"
                    </button>
                </div>
            </div>
            <div
                node_ref=viewport
                data-scope=h::SCOPE
                data-part="viewport"
                data-wrap=move || state.with(|s| s.wrap).then_some("")
                role=move || attrs.get().role
                aria-live=move || attrs.get().aria_live
                aria-label=label
                tabindex=move || attrs.get().tabindex
                on:scroll=move |_| {
                    let Some(el) = viewport.get_untracked() else { return };
                    let end = h::at_end(
                        f64::from(el.scroll_height()),
                        f64::from(el.scroll_top()),
                        f64::from(el.client_height()),
                    );
                    if end != state.with_untracked(|s| s.following) {
                        send(LogStreamEvent::SetFollowing(end));
                    }
                }
            >
                <For
                    each=move || lines.get()
                    key=|l| l.id
                    children=|l| {
                        view! {
                            <div data-scope=h::SCOPE data-part="line" data-level=l.level.as_str()>
                                <span data-scope=h::SCOPE data-part="time">{l.time}</span>
                                <span data-scope=h::SCOPE data-part="tag">{l.level.tag()}</span>
                                <span data-scope=h::SCOPE data-part="text">{l.text}</span>
                            </div>
                        }
                    }
                />
                {move || lines.with(Vec::is_empty).then(|| view! { <p data-scope=h::SCOPE data-part="empty">{empty.clone()}</p> })}
            </div>
        </div>
    }
}
