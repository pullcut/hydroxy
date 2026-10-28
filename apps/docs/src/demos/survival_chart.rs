use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::survival_chart::*;

pub fn demo() -> AnyView {
    // Weeks in remission from the 6-MP leukaemia trial (Freireich et al., 1963).
    // `false` marks a patient still in remission when last seen (censored).
    let drug = SurvivalGroup::new(
        "6-MP",
        [
            (6.0, true), (6.0, true), (6.0, true), (6.0, false), (7.0, true), (9.0, false), (10.0, true), (10.0, false), (11.0, false),
            (13.0, true), (16.0, true), (17.0, false), (19.0, false), (20.0, false), (22.0, true), (23.0, true), (25.0, false),
            (32.0, false), (32.0, false), (34.0, false), (35.0, false),
        ],
    );
    let placebo = SurvivalGroup::new(
        "Placebo",
        [1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 8.0, 8.0, 8.0, 8.0, 11.0, 11.0, 12.0, 12.0, 15.0, 17.0, 22.0, 23.0].map(|w| (w, true)),
    );

    view! {
        <ChartFrame title="Remission by treatment" description="Share of patients still in remission, with 95% intervals. Ticks mark patients last seen in remission.">
            <SurvivalChart
                data=vec![drug, placebo]
                label="Kaplan-Meier curves of weeks in remission"
                time_label="Week"
                show_ci=true
                risk_table=true
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "survival-chart",
    name: "Survival chart",
    group: "Distributions",
    summary: "Show how long things last when some are still going. Kaplan-Meier curves step down at each event, ticks mark censored subjects, and a table counts who is still at risk.",
    demo,
    leptos: include_str!("survival_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/survival_chart.rs"),
    parts: &[
        &["SurvivalChart", "root", "figure with a screen-reader table of each group's estimate at every time; data-variant=ci with intervals"],
        &["", "line", "One step curve per group, drawn left to right"],
        &["", "band", "Greenwood 95% interval, log-log transformed (show_ci)"],
        &["", "censor", "Tick where a subject left the study event-free"],
        &["", "crosshair", "The hovered time, with a dot on each curve"],
        &["", "risk-table", "Number at risk at each time tick (risk_table), after the figure"],
        &["", "tooltip", "Each group's estimate at the hovered time"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next observed time"],
        &["Home / End", "First or last time"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<SurvivalGroup>", "Required. SurvivalGroup::new(name, [(time, event)]); event is false when censored. Up to six"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["time_label", "String", "What time is measured in. Default \"Time\""],
        &["show_ci", "bool", "Shade each curve's 95% confidence interval"],
        &["risk_table", "bool", "Number at risk under the axis"],
        &["height", "u16", "Plot height in px. Default 260"],
        &["format", "Fn(f64) -> String", "Formats times"],
    ],
};
