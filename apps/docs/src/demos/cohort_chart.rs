use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::cohort_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // Customers by signup month, and how many were still paying each month after.
    let cohorts = vec![
        Cohort::new("Jan 2026", 1_204.0, [1_204.0, 662.0, 518.0, 457.0, 421.0, 398.0, 381.0, 370.0, 362.0]),
        Cohort::new("Feb 2026", 1_118.0, [1_118.0, 637.0, 503.0, 449.0, 413.0, 392.0, 377.0, 366.0]),
        Cohort::new("Mar 2026", 1_356.0, [1_356.0, 801.0, 646.0, 579.0, 538.0, 511.0, 494.0]),
        Cohort::new("Apr 2026", 1_290.0, [1_290.0, 787.0, 640.0, 576.0, 537.0, 514.0]),
        Cohort::new("May 2026", 1_472.0, [1_472.0, 927.0, 762.0, 690.0, 646.0]),
        Cohort::new("Jun 2026", 1_538.0, [1_538.0, 984.0, 818.0, 741.0]),
        Cohort::new("Jul 2026", 1_611.0, [1_611.0, 1_047.0, 880.0]),
        Cohort::new("Aug 2026", 1_704.0, [1_704.0, 1_125.0]),
        Cohort::new("Sep 2026", 1_689.0, [1_689.0]),
    ];
    let periods: Vec<String> = (0..9).map(|m| format!("Month {m}")).collect();

    view! {
        <ChartFrame title="Paying customers by signup month" description="Share of each month's signups still paying. Later cohorts hold on better after the March onboarding change." export=true>
            <CohortChart cohorts periods label="Monthly retention by signup month" noun="customers" />
        </ChartFrame>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "cohort-chart",
    name: "Cohort chart",
    group: "Grids and tables",
    summary: "A retention triangle: one row per cohort (everyone who started in the same period) with its size, and one cell per period since, shaded by the share still active. The ramp tops out at the best rate after period 0, so differences in later periods stay visible. Cells fade in along the diagonals.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("cohort_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/cohort_chart.rs"),
    parts: &[
        &["CohortChart", "root", "figure with a screen-reader table of every rate"],
        &["", "row-label", "Cohort name and its size"],
        &["", "cells", "Focusable grid; arrow keys move between cells"],
        &["", "cell", "Shaded by --level, labelled with its percentage; --i is its diagonal"],
        &["", "tooltip", "Cohort, period and the count behind the rate"],
        &["", "scale", "The colour ramp from 0% to the best later rate"],
        &["", "sr-live", "Announces the active cell"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move along a cohort; the first press lands on the top-left cell"],
        &["ArrowUp / ArrowDown", "Move between cohorts, staying inside the triangle"],
        &["Home / End", "First or latest period of the cohort"],
        &["Escape", "Clear the active cell"],
    ],
    props: &[
        &["cohorts", "Vec<Cohort>", "Required. Cohort::new(name, size, counts per period so far)"],
        &["periods", "Vec<String>", "Required. One label per period"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["noun", "String", "What a count is of, for the tooltip. Default \"people\""],
        &["cell_height", "u16", "Cell height in px. Default 32"],
        &["format", "Fn(f64) -> String", "Formats sizes and counts"],
    ],
    events: &[],
};
