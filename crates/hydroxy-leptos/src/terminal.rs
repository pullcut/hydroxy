//! `<Terminal>`: terminal-style output with ANSI colours and a prompt. The host runs
//! commands from `on_command` and writes results back through `output`. Up/Down walk
//! history, Ctrl+L clears, and the view follows new output until you scroll up.

use std::time::Duration;

use leptos::prelude::*;
use hydroxy_headless::terminal::{
    self as h, TerminalCommand, TerminalEvent, TerminalInput, TerminalMachine,
};
use hydroxy_statechart::Machine;

use crate::focus::focus_id;

/// A terminal. Feed it output through `output`.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Terminal(
    /// Each time this changes, its text is written. Set it to what just arrived; `\n`
    /// splits lines and ANSI SGR escapes colour them.
    #[prop(optional, into)]
    output: Signal<String>,
    /// Called with each submitted, non-blank command line.
    #[prop(optional, into)]
    on_command: Option<Callback<String>>,
    /// Text before the input, echoed with each command.
    #[prop(default = "$".into(), into)]
    prompt: String,
    /// Most lines kept; the oldest go first.
    #[prop(default = 2000)]
    max_lines: usize,
    /// Accessible name for the output.
    #[prop(default = "Terminal".into(), into)]
    label: String,
) -> impl IntoView {
    let input = StoredValue::new(TerminalInput { max_lines, prompt: prompt.clone() });
    let state = RwSignal::new(input.with_value(TerminalMachine::initial));
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let viewport_id = StoredValue::new(ids.viewport.clone());
    let to_end = move || {
        // Wait for the new lines to render before measuring.
        set_timeout(
            move || {
                if let Some(el) = document().get_element_by_id(&viewport_id.get_value()) {
                    el.set_scroll_top(el.scroll_height());
                }
            },
            Duration::ZERO,
        );
    };
    let send = move |event: TerminalEvent| {
        let Some(step) = input.try_with_value(|i| state.try_with_untracked(|s| TerminalMachine::transition(s, event, i))).flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                TerminalCommand::ScrollToEnd => to_end(),
                TerminalCommand::Run(line) => {
                    if let Some(cb) = on_command {
                        cb.run(line);
                    }
                }
            }
        }
    };
    Effect::new(move |_| {
        let text = output.get();
        if !text.is_empty() {
            send(TerminalEvent::Write(text));
        }
    });
    let input_id = ids.input.clone();
    let focus_input = move |_| {
        // Leave a text selection alone so it can be copied.
        let selecting = window().get_selection().ok().flatten().is_some_and(|s| !s.is_collapsed());
        if !selecting {
            focus_id(&input_id);
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-state=move || if state.with(|s| s.following) { "following" } else { "scrolled" }>
            <div
                id=ids.viewport.clone()
                data-scope=h::SCOPE
                data-part="viewport"
                on:click=focus_input
                on:scroll=move |_| {
                    let Some(el) = document().get_element_by_id(&viewport_id.get_value()) else { return };
                    let end = h::at_end(f64::from(el.scroll_height()), f64::from(el.scroll_top()), f64::from(el.client_height()));
                    if end != state.with_untracked(|s| s.following) {
                        send(TerminalEvent::SetFollowing(end));
                    }
                }
            >
                <div data-scope=h::SCOPE data-part="output" role="log" aria-live="polite" aria-label=label>
                    <For
                        each=move || state.with(|s| s.lines.iter().cloned().collect::<Vec<_>>())
                        key=|l| l.id
                        children=|l| {
                            view! {
                                <div data-scope=h::SCOPE data-part="line" data-echo=l.echo.then_some("")>
                                    {l
                                        .spans
                                        .into_iter()
                                        .map(|s| view! { <span style=h::span_css(&s.style)>{s.text}</span> })
                                        .collect_view()}
                                </div>
                            }
                        }
                    />
                </div>
                <div data-scope=h::SCOPE data-part="prompt-row">
                    <label data-scope=h::SCOPE data-part="prompt" for=ids.input.clone()>
                        {prompt}
                    </label>
                    <input
                        id=ids.input.clone()
                        data-scope=h::SCOPE
                        data-part="input"
                        type="text"
                        aria-label="Command"
                        autocomplete="off"
                        autocapitalize="off"
                        spellcheck="false"
                        prop:value=move || state.with(|s| s.draft.clone())
                        on:input=move |ev| send(TerminalEvent::Input(event_target_value(&ev)))
                        on:keydown=move |ev| {
                            if let Some(event) = h::input_keydown(&ev.key(), ev.ctrl_key()) {
                                ev.prevent_default();
                                send(event);
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
