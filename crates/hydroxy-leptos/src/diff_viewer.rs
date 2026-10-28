//! `<DiffViewer>`: a line diff of `before` against `after`, unified or side by side, with changed
//! words marked, syntax colouring and unchanged runs folded behind "Expand" buttons.

use leptos::either::Either;
use leptos::prelude::*;
use hydroxy_headless::diff_viewer::{
    self as h, DiffLine, DiffMode, DiffViewerCommand, DiffViewerEvent, DiffViewerInput, DiffViewerMachine, Row, SplitRow,
};
use hydroxy_statechart::Machine;

use crate::focus::focus_id;
use crate::segmented::{Segmented, SegmentedItem, SegmentedOption, SegmentedSize};

pub use hydroxy_headless::diff_viewer::DiffMode as Mode;

/// A diff of `before` against `after`.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DiffViewer(
    /// The text before.
    #[prop(into)]
    before: Signal<String>,
    /// The text after.
    #[prop(into)]
    after: Signal<String>,
    /// Shown in the header and caption, such as "src/app.rs".
    #[prop(optional, into)]
    file_name: Option<String>,
    /// Language for syntax colouring, as for `CodeBlock`.
    #[prop(default = "text".into(), into)]
    lang: String,
    /// Unchanged lines kept around each change. Default 3.
    #[prop(default = 3)]
    context: usize,
    /// Layout to start in.
    #[prop(optional)]
    mode: DiffMode,
) -> impl IntoView {
    let input = DiffViewerInput { default_mode: mode };
    let state = RwSignal::new(DiffViewerMachine::initial(&input));
    let root = StoredValue::new(crate::id::next_id(h::SCOPE));
    let lines = Memo::new(move |_| h::diff_lines(&before.get(), &after.get()));
    let counts = Memo::new(move |_| lines.with(|l| h::stats(l)));
    let send = move |event: DiffViewerEvent| {
        let Some(step) = state.try_with_untracked(|s| DiffViewerMachine::transition(s, event, &input)) else { return };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let DiffViewerCommand::FocusLine(i) = command;
            let id = h::line_id(&root.get_value(), i);
            set_timeout(move || focus_id(&id), std::time::Duration::ZERO);
        }
    };
    // New texts fold everything again; gap ids are line indices of the old diff.
    Effect::new(move |prev: Option<()>| {
        lines.track();
        if prev.is_some() {
            send(DiffViewerEvent::Reset);
        }
    });
    let rows = Memo::new(move |_| lines.with(|l| state.with(|s| h::rows(l, context, &s.expanded))));
    let lang = StoredValue::new(lang);
    let caption = {
        let file = file_name.clone();
        Memo::new(move |_| h::summary(file.as_deref(), counts.get()))
    };

    let code = move |i: usize| {
        let line = lines.with_untracked(|l| l[i].clone());
        code_cell(&line, &lang.get_value())
    };
    let num = |n: Option<usize>| view! { <td data-scope=h::SCOPE data-part="num" aria-hidden="true">{n}</td> };
    let gap = move |start: usize, len: usize, cols: &'static str| {
        view! {
            <tr data-scope=h::SCOPE data-part="gap">
                <td colspan=cols>
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="expand"
                        on:click=move |_| send(DiffViewerEvent::Expand(start))
                    >
                        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                            <path d="M8 2v4M8 14v-4M5 4l3-3 3 3M5 12l3 3 3-3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                        {h::expand_label(len)}
                    </button>
                </td>
            </tr>
        }
    };

    let unified = move || {
        view! {
            <thead data-scope=h::SCOPE data-part="sr">
                <tr><th scope="col">"Old line"</th><th scope="col">"New line"</th><th scope="col">"Change"</th></tr>
            </thead>
            <tbody>
                <For
                    each=move || rows.get()
                    key=|r| *r
                    children=move |r| match r {
                        Row::Line(i) => {
                            let l = lines.with_untracked(|l| (l[i].kind, l[i].old_no, l[i].new_no));
                            Either::Left(view! {
                                <tr id=h::line_id(&root.get_value(), i) tabindex="-1" data-scope=h::SCOPE data-part="line" data-kind=l.0.as_str()>
                                    {num(l.1)}
                                    {num(l.2)}
                                    {code(i)}
                                </tr>
                            })
                        }
                        Row::Gap { start, len } => Either::Right(gap(start, len, "3")),
                    }
                />
            </tbody>
        }
    };
    let split = move || {
        let side = move |i: Option<usize>, old_side: bool| match i {
            Some(i) => {
                let n = lines.with_untracked(|l| if old_side { l[i].old_no } else { l[i].new_no });
                Either::Left(view! { {num(n)} {code(i)} })
            }
            None => Either::Right(view! {
                <td data-scope=h::SCOPE data-part="num" aria-hidden="true"></td>
                <td data-scope=h::SCOPE data-part="code" data-empty=""></td>
            }),
        };
        view! {
            <thead data-scope=h::SCOPE data-part="sr">
                <tr>
                    <th scope="col">"Old line"</th><th scope="col">"Old text"</th>
                    <th scope="col">"New line"</th><th scope="col">"New text"</th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || lines.with(|l| rows.with(|r| h::split_rows(l, r)))
                    key=|r| *r
                    children=move |r| match r {
                        SplitRow::Pair(a, b) => Either::Left(view! {
                            <tr
                                id=a.or(b).map(|i| h::line_id(&root.get_value(), i))
                                tabindex="-1"
                                data-scope=h::SCOPE
                                data-part="line"
                            >
                                {side(a, true)}
                                {side(b, false)}
                            </tr>
                        }),
                        SplitRow::Gap { start, len } => Either::Right(gap(start, len, "4")),
                    }
                />
            </tbody>
        }
    };

    let mode_now = Memo::new(move |_| state.with(|s| s.mode));
    view! {
        <div data-scope=h::SCOPE data-part="root" data-mode=move || mode_now.get().as_str()>
            <div data-scope=h::SCOPE data-part="header">
                {file_name.map(|f| view! { <span data-scope=h::SCOPE data-part="file">{f}</span> })}
                <span data-scope=h::SCOPE data-part="stats">
                    <span data-scope=h::SCOPE data-part="added" aria-hidden="true">{move || format!("+{}", counts.get().added)}</span>
                    <span data-scope=h::SCOPE data-part="removed" aria-hidden="true">{move || format!("−{}", counts.get().removed)}</span>
                </span>
                <Segmented
                    label="Diff layout"
                    items=vec![SegmentedOption::new("unified"), SegmentedOption::new("split")]
                    default_value=mode.as_str().to_string()
                    size=SegmentedSize::Sm
                    on_value_change=move |v: String| send(DiffViewerEvent::SetMode(DiffMode::parse(&v)))
                >
                    <SegmentedItem value="unified" label="Unified" />
                    <SegmentedItem value="split" label="Split" />
                </Segmented>
            </div>
            <div data-scope=h::SCOPE data-part="body" tabindex="0" role="region" aria-label=caption>
                <table data-scope=h::SCOPE data-part="table">
                    <caption data-scope=h::SCOPE data-part="sr">{caption}</caption>
                    // Rebuild the rows when the texts change: row keys are line indices.
                    {move || {
                        lines.track();
                        match mode_now.get() {
                            DiffMode::Unified => Either::Left(unified()),
                            DiffMode::Split => Either::Right(split()),
                        }
                    }}
                </table>
            </div>
        </div>
    }
}

/// The text cell: hidden "Added:"/"Removed:" label, the gutter marker and the segments.
fn code_cell(line: &DiffLine, lang: &str) -> impl IntoView + use<> {
    let segs = h::segments(&line.text, &line.marks, lang)
        .into_iter()
        .map(|s| {
            if s.token.is_none() && !s.changed {
                Either::Left(s.text)
            } else {
                Either::Right(view! { <span data-hl=s.token data-word=s.changed.then_some("")>{s.text}</span> })
            }
        })
        .collect_view();
    view! {
        <td data-scope=h::SCOPE data-part="code" data-kind=line.kind.as_str()>
            {h::kind_label(line.kind).map(|t| view! { <span data-scope=h::SCOPE data-part="sr">{t}</span> })}
            <span data-scope=h::SCOPE data-part="marker" aria-hidden="true">{h::marker(line.kind)}</span>
            {segs}
        </td>
    }
}
