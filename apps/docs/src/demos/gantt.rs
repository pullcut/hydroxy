use hydroxy_leptos::gantt::*;
use leptos::prelude::*;

fn d(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).expect("a real date")
}

fn plan() -> Vec<GanttTask> {
    vec![
        GanttTask::new("research", "Customer research", d(2026, 10, 19), d(2026, 10, 23)),
        GanttTask::new("synthesis", "Synthesis", d(2026, 10, 26), d(2026, 10, 28)).depends_on(["research"]),
        GanttTask::new("design", "Design the digest", d(2026, 10, 27), d(2026, 11, 3)).depends_on(["synthesis"]),
        GanttTask::new("build", "Build and test", d(2026, 11, 4), d(2026, 11, 12)).depends_on(["design"]),
        GanttTask::new("launch", "Launch", d(2026, 11, 13), d(2026, 11, 13)).depends_on(["build", "design"]),
    ]
}

pub fn demo() -> AnyView {
    // Controlled: the owner applies each change. Today is fixed so the page is the same every day.
    let (tasks, set_tasks) = signal(plan());
    let (last, set_last) = signal(String::from("Drag a bar or an edge, or focus one and press Space."));

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <Gantt
                tasks=tasks
                today=d(2026, 10, 30)
                on_change=move |change: GanttChange| {
                    set_last.set(format!("{}: {} to {}", change.id, change.start, change.end));
                    set_tasks.update(|list| apply_change(list, &change));
                }
            />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">{last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "gantt",
    name: "Gantt chart",
    group: "Data",
    summary: "A task list beside a timeline of bars on a day or week scale. Drag a bar to move it or an edge to resize it, snapping to days, or pick a bar up with Space and use the arrow keys. Dependencies are drawn as connectors; they are not enforced.",
    demo,
    leptos: include_str!("gantt.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/gantt.rs"),
    parts: &[
        &["Gantt", "root", "Wrapper. data-scale=day|week, data-dragging while a bar moves"],
        &["(inside)", "toolbar, scale-toggle", "The Day and Week buttons, aria-pressed"],
        &["(inside)", "list, list-header, rows, row", "The task names and dates on the left, one row per task"],
        &["(inside)", "scroller, timeline", "The horizontally scrolling chart"],
        &[
            "(inside)",
            "header, months, month, ticks, tick",
            "Month names over day or week columns. data-weekend and data-major on ticks",
        ],
        &["(inside)", "grid", "role=group. Column and row lines are its background"],
        &["(inside)", "connectors, connector, arrow", "SVG dependency lines with an arrowhead at the dependent task"],
        &["(inside)", "today", "The vertical line at the today date"],
        &[
            "(inside)",
            "bar",
            "<div role=button tabindex=0>. data-state=idle|dragging|grabbed, aria-pressed while picked up",
        ],
        &["(inside)", "bar-label, handle", "The title, and the two edge handles (data-edge=start|end)"],
        &[
            "(inside)",
            "instructions, live",
            "Visually hidden keyboard help linked by aria-describedby, and a role=status region",
        ],
    ],
    keys: &[
        &["Space / Enter", "Picks up the focused bar, or drops the bar you're carrying"],
        &["ArrowLeft / ArrowRight", "Moves the picked-up bar a day"],
        &["Shift + ArrowLeft / ArrowRight", "Moves the bar's end a day"],
        &["Alt + ArrowLeft / ArrowRight", "Moves the bar's start a day"],
        &["Escape", "Puts the bar back where it started"],
        &["ArrowUp / ArrowDown", "Moves focus to the previous or next bar"],
    ],
    props: &[
        &["tasks", "Option<Vec<GanttTask>>", "Controlled tasks. Pair with on_change and apply_change"],
        &[
            "default_tasks",
            "Vec<GanttTask>",
            "Initial tasks when uncontrolled. GanttTask::new(id, title, start, end).depends_on([ids])",
        ],
        &["scale", "Option<GanttScale>", "Controlled scale. Pair with on_scale_change"],
        &["default_scale", "GanttScale", "Day (default) or Week"],
        &["today", "Option<Date>", "Where to draw the today line. Pass the date in; the chart never reads a clock"],
        &["on_change", "Callback<GanttChange>", "A bar landed on new dates: id, start, end (both inclusive)"],
        &["on_scale_change", "Callback<GanttScale>", "The scale toggle was used"],
        &["disabled", "bool", "Block every move"],
    ],
    events: &[],
};
