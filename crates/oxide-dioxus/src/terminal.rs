//! `Terminal`: terminal-style output with ANSI colours and a prompt. The host runs
//! commands from `on_command` and writes results back through `output`. Up/Down walk
//! history, Ctrl+L clears, and the view follows new output until you scroll up.

use dioxus::prelude::*;
use oxide_headless::terminal::{self as h, TerminalCommand, TerminalEvent, TerminalInput, TerminalMachine};
use oxide_statechart::Machine;

/// A terminal. Feed it output through `output`.
#[component]
pub fn Terminal(
    /// Each time this changes, its text is written. Set it to what just arrived; `\n`
    /// splits lines and ANSI SGR escapes colour them.
    #[props(default)]
    output: ReadSignal<String>,
    /// Called with each submitted, non-blank command line.
    on_command: Option<EventHandler<String>>,
    /// Text before the input, echoed with each command.
    #[props(default = "$".to_string())]
    prompt: String,
    /// Most lines kept; the oldest go first.
    #[props(default = 2000)]
    max_lines: usize,
    /// Accessible name for the output.
    #[props(default = "Terminal".to_string())]
    label: String,
) -> Element {
    let input = use_hook(|| CopyValue::new(TerminalInput { max_lines, prompt: prompt.clone() }));
    let mut state = use_signal(|| TerminalMachine::initial(&input.read()));
    let ids = use_hook(|| h::ids(&oxide_headless::next_id(h::SCOPE)));
    let viewport_id = use_hook(|| CopyValue::new(ids.viewport.clone()));
    let to_end = move || {
        let dom_id = viewport_id.cloned();
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
    let mut send = move |event: TerminalEvent| {
        let step = TerminalMachine::transition(&state.peek(), event, &input.read());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                TerminalCommand::ScrollToEnd => to_end(),
                TerminalCommand::Run(line) => {
                    if let Some(handler) = on_command {
                        handler.call(line);
                    }
                }
            }
        }
    };
    use_effect(move || {
        let text = output.read().clone();
        if !text.is_empty() {
            send(TerminalEvent::Write(text));
        }
    });
    let input_id = ids.input.clone();

    let s = state.read();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-state": if s.following { "following" } else { "scrolled" },
            div {
                id: ids.viewport.clone(),
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                // Focus the prompt unless the reader is selecting text to copy.
                onclick: move |_| {
                    let _ = document::eval(&format!(
                        "if (window.getSelection()?.isCollapsed !== false) document.getElementById({input_id:?})?.focus();"
                    ));
                },
                onscroll: move |e| {
                    let data = e.data();
                    let end = h::at_end(f64::from(data.scroll_height()), data.scroll_top(), f64::from(data.client_height()));
                    let following = state.peek().following;
                    if end != following {
                        send(TerminalEvent::SetFollowing(end));
                    }
                },
                div { "data-scope": h::SCOPE, "data-part": "output", role: "log", "aria-live": "polite", "aria-label": "{label}",
                    for l in s.lines.iter() {
                        div { key: "{l.id}", "data-scope": h::SCOPE, "data-part": "line", "data-echo": l.echo.then_some(""),
                            for span in l.spans.iter() {
                                span { style: h::span_css(&span.style), "{span.text}" }
                            }
                        }
                    }
                }
                div { "data-scope": h::SCOPE, "data-part": "prompt-row",
                    label { "data-scope": h::SCOPE, "data-part": "prompt", r#for: ids.input.clone(), "{prompt}" }
                    input {
                        id: ids.input.clone(),
                        "data-scope": h::SCOPE,
                        "data-part": "input",
                        r#type: "text",
                        "aria-label": "Command",
                        autocomplete: "off",
                        "autocapitalize": "off",
                        spellcheck: "false",
                        value: s.draft.clone(),
                        oninput: move |e| send(TerminalEvent::Input(e.value())),
                        onkeydown: move |e| {
                            if let Some(event) = h::input_keydown(&e.key().to_string(), e.modifiers().ctrl()) {
                                e.prevent_default();
                                send(event);
                            }
                        },
                    }
                }
            }
        }
    }
}
