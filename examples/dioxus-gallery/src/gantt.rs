use dioxus::prelude::*;
use hydroxy_dioxus::gantt::{Date, Gantt, GanttChange, GanttTask, apply_change};

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

#[component]
pub fn GanttDemo() -> Element {
    // Controlled: the owner applies each change. Today is fixed so the page is the same every day.
    let mut tasks = use_signal(plan);
    let mut last = use_signal(|| String::from("Drag a bar or an edge, or focus one and press Space."));

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            Gantt {
                tasks: Some(tasks()),
                today: Some(d(2026, 10, 30)),
                on_change: move |change: GanttChange| {
                    last.set(format!("{}: {} to {}", change.id, change.start, change.end));
                    apply_change(&mut tasks.write(), &change);
                },
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "{last}" }
        }
    }
}
