use std::fmt::Write as _;

use hydroxy_leptos::terminal::Terminal;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (output, set_output) = signal(
        "\x1b[1;32mwelcome\x1b[0m to the hydroxy demo shell. Type \x1b[1mhelp\x1b[0m to see what it knows.".to_string(),
    );
    view! {
        <div style="height: 360px; max-width: 760px;">
            <Terminal output=output prompt="~/hydroxy $" label="Demo shell" on_command=move |line: String| set_output.set(respond(&line)) />
        </div>
    }
    .into_any()
}

/// A tiny pretend shell: what each command prints.
fn respond(line: &str) -> String {
    let (cmd, rest) = line.trim().split_once(' ').unwrap_or((line.trim(), ""));
    match cmd {
        "help" => "Commands: \x1b[1mls\x1b[0m, \x1b[1mgit status\x1b[0m, \x1b[1mcargo test\x1b[0m, \x1b[1mcolors\x1b[0m, \x1b[1mecho\x1b[0m, \x1b[1mclear\x1b[0m\nUp and Down recall earlier commands; Ctrl+L clears the screen.".into(),
        "ls" => "\x1b[1;34mcrates\x1b[0m  \x1b[1;34mapps\x1b[0m  \x1b[1;34mexamples\x1b[0m  Cargo.toml  Cargo.lock  \x1b[32mbuild.sh\x1b[0m  README.md".into(),
        "git" if rest == "status" => "On branch \x1b[1mmain\x1b[0m\nChanges not staged for commit:\n  \x1b[31mmodified:   crates/hydroxy-machines/src/terminal.rs\x1b[0m\n  \x1b[31mmodified:   apps/docs/src/app.rs\x1b[0m\nUntracked files:\n  \x1b[31mcrates/hydroxy-tokens/css/terminal.css\x1b[0m".into(),
        "cargo" if rest == "test" => "\x1b[1;32m   Compiling\x1b[0m hydroxy-machines v0.1.0\n\x1b[1;32m    Finished\x1b[0m test profile in 4.21s\n\x1b[1;32m     Running\x1b[0m unittests src/lib.rs\ntest terminal::tests::history_walks_and_restores_draft ... \x1b[32mok\x1b[0m\ntest terminal::tests::sgr_256_and_truecolour ... \x1b[32mok\x1b[0m\ntest parser::tests::nested_quotes ... \x1b[1;31mFAILED\x1b[0m\n\x1b[2mnote: run with `RUST_BACKTRACE=1` for a backtrace\x1b[0m\ntest result: \x1b[31mFAILED\x1b[0m. 2 passed; 1 failed".into(),
        "colors" => {
            let basic = (0..16).fold(String::new(), |mut s, i| {
                let _ = write!(s, "\x1b[{}m {i:>2} ", if i < 8 { 40 + i } else { 92 + i });
                s
            });
            let cube = (16..52).fold(String::new(), |mut s, i| {
                let _ = write!(s, "\x1b[48;5;{i}m  ");
                s
            });
            let ramp = (0..36).fold(String::new(), |mut s, i| {
                let _ = write!(s, "\x1b[48;2;{};{};200m  ", i * 7, 220 - i * 5);
                s
            });
            format!("{basic}\x1b[0m\n{cube}\x1b[0m\n{ramp}\x1b[0m\n\x1b[1mbold\x1b[0m \x1b[2mdim\x1b[0m \x1b[3mitalic\x1b[0m \x1b[4munderline\x1b[0m \x1b[7minverse\x1b[0m")
        }
        "echo" => rest.to_string(),
        "clear" => "\x1b[2J".into(),
        _ => format!("\x1b[31m{cmd}: command not found\x1b[0m. Try \x1b[1mhelp\x1b[0m."),
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "terminal",
    name: "Terminal",
    group: "Data",
    summary: "Terminal-style output with a prompt. ANSI colours (16, 256 and truecolour) and bold, dim, italic, underline and inverse render as styled, copyable text. Up and Down recall history, Ctrl+L clears, and the view follows new output until you scroll up. Your code runs the commands.",
    demo,
    leptos: include_str!("terminal.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/terminal.rs"),
    parts: &[
        &["Terminal", "root", "The panel. data-state: following or scrolled"],
        &["", "viewport", "The scrolling area. A click focuses the prompt unless you are selecting text"],
        &["", "output", "The role=log list of lines, announced politely"],
        &["", "line", "One line of styled spans. data-echo on echoed commands"],
        &["", "prompt-row / prompt / input", "The prompt label and the command field"],
    ],
    keys: &[
        &["Enter", "Echo and run the command"],
        &["Up / Down", "Older and newer commands; Down past the newest restores your draft"],
        &["Ctrl+L", "Clear the screen"],
    ],
    props: &[
        &[
            "output",
            "String",
            "Each change writes this text. \\n splits lines; ANSI SGR escapes style it; ESC[2J clears",
        ],
        &["on_command", "Callback<String>", "Called with each non-blank command line"],
        &["prompt", "String", "Text before the input, echoed with commands. Default $"],
        &["max_lines", "usize", "Most lines kept; the oldest go first. Default 2000"],
        &["label", "String", "Accessible name for the output"],
    ],
};
