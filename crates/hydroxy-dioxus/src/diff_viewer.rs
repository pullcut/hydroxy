//! `DiffViewer`: a line diff of `before` against `after`, unified or side by side, with changed
//! words marked, syntax colouring and unchanged runs folded behind "Expand" buttons.

use dioxus::prelude::*;
use hydroxy_headless::diff_viewer::{
    self as h, DiffLine, DiffMode, DiffViewerCommand, DiffViewerEvent, DiffViewerInput, DiffViewerMachine, Row,
    SplitRow,
};
use hydroxy_statechart::Machine;

use crate::segmented::{Segmented, SegmentedItem, SegmentedOption, SegmentedSize};

pub use hydroxy_headless::diff_viewer::DiffMode as Mode;

/// A diff of `before` against `after`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn DiffViewer(
    /// The text before.
    before: ReadSignal<String>,
    /// The text after.
    after: ReadSignal<String>,
    /// Shown in the header and caption, such as "src/app.rs".
    file_name: Option<String>,
    /// Language for syntax colouring, as for `CodeBlock`.
    #[props(default = "text".to_string())]
    lang: String,
    /// Unchanged lines kept around each change. Default 3.
    #[props(default = 3)]
    context: usize,
    /// Layout to start in.
    #[props(default)]
    mode: DiffMode,
) -> Element {
    let input = DiffViewerInput { default_mode: mode };
    let mut state = use_signal(|| DiffViewerMachine::initial(&input));
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let lines = use_memo(move || h::diff_lines(&before.read(), &after.read()));
    // New texts fold everything again; gap ids are line indices of the old diff.
    use_effect(move || {
        lines.read();
        if !state.peek().expanded.is_empty() {
            state.write().expanded.clear();
        }
    });
    let send = {
        let root = root.clone();
        move |event: DiffViewerEvent| {
            let step = DiffViewerMachine::transition(&state.peek(), event, &input);
            if let Some(next) = step.state {
                state.set(next);
            }
            for command in step.commands {
                let DiffViewerCommand::FocusLine(i) = command;
                let id = h::line_id(&root, i);
                // `{id:?}` quotes the id as a JS string; ids are plain ASCII.
                spawn(async move {
                    let _ = document::eval(&format!(
                        "await new Promise((r) => setTimeout(r, 0)); document.getElementById({id:?})?.focus();"
                    ))
                    .await;
                });
            }
        }
    };

    let l = lines.read();
    let s = state.read();
    let rows = h::rows(&l, context, &s.expanded);
    let stats = h::stats(&l);
    let caption = h::summary(file_name.as_deref(), stats);
    let split = s.mode == DiffMode::Split;
    let split_rows = if split { h::split_rows(&l, &rows) } else { Vec::new() };
    let cols = if split { "4" } else { "3" };

    let gap = |start: usize, len: usize| {
        let mut send = send.clone();
        rsx! {
            tr { key: "gap-{start}", "data-scope": h::SCOPE, "data-part": "gap",
                td { colspan: cols,
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "expand",
                        onclick: move |_| send(DiffViewerEvent::Expand(start)),
                        svg { view_box: "0 0 16 16", width: "14", height: "14", "aria-hidden": "true",
                            path {
                                d: "M8 2v4M8 14v-4M5 4l3-3 3 3M5 12l3 3 3-3",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.5",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                            }
                        }
                        "{h::expand_label(len)}"
                    }
                }
            }
        }
    };
    let num = |n: Option<usize>| {
        rsx! {
            td { "data-scope": h::SCOPE, "data-part": "num", "aria-hidden": "true",
                if let Some(n) = n { "{n}" }
            }
        }
    };
    let side = |i: Option<usize>, old_side: bool| match i {
        Some(i) => rsx! {
            {num(if old_side { l[i].old_no } else { l[i].new_no })}
            {code_cell(&l[i], &lang)}
        },
        None => rsx! {
            td { "data-scope": h::SCOPE, "data-part": "num", "aria-hidden": "true" }
            td { "data-scope": h::SCOPE, "data-part": "code", "data-empty": "" }
        },
    };

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-mode": s.mode.as_str(),
            div { "data-scope": h::SCOPE, "data-part": "header",
                if let Some(f) = file_name.clone() {
                    span { "data-scope": h::SCOPE, "data-part": "file", "{f}" }
                }
                span { "data-scope": h::SCOPE, "data-part": "stats",
                    span { "data-scope": h::SCOPE, "data-part": "added", "aria-hidden": "true", "+{stats.added}" }
                    span { "data-scope": h::SCOPE, "data-part": "removed", "aria-hidden": "true", "−{stats.removed}" }
                }
                Segmented {
                    label: "Diff layout",
                    items: vec![SegmentedOption::new("unified"), SegmentedOption::new("split")],
                    default_value: mode.as_str().to_string(),
                    size: SegmentedSize::Sm,
                    on_value_change: {
                        let mut send = send.clone();
                        move |v: String| send(DiffViewerEvent::SetMode(DiffMode::parse(&v)))
                    },
                    SegmentedItem { value: "unified", label: "Unified" }
                    SegmentedItem { value: "split", label: "Split" }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "body", tabindex: "0", role: "region", "aria-label": "{caption}",
                table { "data-scope": h::SCOPE, "data-part": "table",
                    caption { "data-scope": h::SCOPE, "data-part": "sr", "{caption}" }
                    if split {
                        thead { "data-scope": h::SCOPE, "data-part": "sr",
                            tr {
                                th { scope: "col", "Old line" }
                                th { scope: "col", "Old text" }
                                th { scope: "col", "New line" }
                                th { scope: "col", "New text" }
                            }
                        }
                        tbody {
                            for r in split_rows {
                                match r {
                                    SplitRow::Pair(a, b) => rsx! {
                                        tr {
                                            key: "{a:?}-{b:?}",
                                            id: a.or(b).map(|i| h::line_id(&root, i)),
                                            tabindex: "-1",
                                            "data-scope": h::SCOPE,
                                            "data-part": "line",
                                            {side(a, true)}
                                            {side(b, false)}
                                        }
                                    },
                                    SplitRow::Gap { start, len } => gap(start, len),
                                }
                            }
                        }
                    } else {
                        thead { "data-scope": h::SCOPE, "data-part": "sr",
                            tr {
                                th { scope: "col", "Old line" }
                                th { scope: "col", "New line" }
                                th { scope: "col", "Change" }
                            }
                        }
                        tbody {
                            for r in rows {
                                match r {
                                    Row::Line(i) => rsx! {
                                        tr {
                                            key: "{i}",
                                            id: h::line_id(&root, i),
                                            tabindex: "-1",
                                            "data-scope": h::SCOPE,
                                            "data-part": "line",
                                            "data-kind": l[i].kind.as_str(),
                                            {num(l[i].old_no)}
                                            {num(l[i].new_no)}
                                            {code_cell(&l[i], &lang)}
                                        }
                                    },
                                    Row::Gap { start, len } => gap(start, len),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The text cell: hidden "Added:"/"Removed:" label, the gutter marker and the segments.
fn code_cell(line: &DiffLine, lang: &str) -> Element {
    let segs = h::segments(&line.text, &line.marks, lang);
    rsx! {
        td { "data-scope": h::SCOPE, "data-part": "code", "data-kind": line.kind.as_str(),
            if let Some(t) = h::kind_label(line.kind) {
                span { "data-scope": h::SCOPE, "data-part": "sr", "{t}" }
            }
            span { "data-scope": h::SCOPE, "data-part": "marker", "aria-hidden": "true", "{h::marker(line.kind)}" }
            for s in segs {
                if s.token.is_none() && !s.changed {
                    "{s.text}"
                } else {
                    span { "data-hl": s.token, "data-word": s.changed.then_some(""), "{s.text}" }
                }
            }
        }
    }
}
