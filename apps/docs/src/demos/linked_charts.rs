use hydroxy_charts::event::{Datum as EventDatum, View};
use hydroxy_leptos::bar_chart::{BarChart, Datum};
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::line_chart::{LineChart, Series};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    // Two line charts share one view: zoom or pan either and the other follows.
    let weeks: Vec<String> = (1..=24).map(|w| format!("Wk {w}")).collect();
    let signups: Vec<f64> =
        (0..24u32).map(|i| 300.0 + f64::from(i) * 9.0 + (f64::from(i) * 0.7).sin() * 30.0).collect();
    let churn: Vec<f64> = (0..24u32).map(|i| 40.0 - f64::from(i) * 0.4 + (f64::from(i) * 0.9).cos() * 5.0).collect();
    let signups = vec![Series::new("Signups", signups)];
    let churn = vec![Series::new("Cancellations", churn)];
    let weeks_a = weeks.clone();
    let view = RwSignal::new(View::FULL);
    let shared: Signal<View> = view.into();
    let sync = Callback::new(move |v: View| view.set(v));

    // Clicking a bar filters the table to that team's tickets.
    let teams = vec![
        Datum::new("Research", 4.0),
        Datum::new("Support", 3.0),
        Datum::new("Sales", 3.0),
        Datum::new("Product", 2.0),
    ];
    let tickets = [
        ("Research", "Interview backlog"),
        ("Research", "Tag the September calls"),
        ("Research", "Survey wording"),
        ("Research", "Recruit five users"),
        ("Support", "Refund wording"),
        ("Support", "Macro cleanup"),
        ("Support", "Weekend cover"),
        ("Sales", "Pricing page copy"),
        ("Sales", "Trial follow-ups"),
        ("Sales", "Renewal list"),
        ("Product", "Export to CSV"),
        ("Product", "Dark mode contrast"),
    ];
    let picked = RwSignal::new(None::<String>);
    let on_select = Callback::new(move |d: EventDatum| picked.set(d.label));

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <div data-testid="linked-a">
                <ChartFrame title="Signups" description="Scroll or press plus on the plot to zoom. The chart below follows.">
                    <LineChart
                        series=signups
                        labels=weeks_a
                        label="Signups per week"
                        zoom=true
                        view=shared
                        on_view_change=sync
                    />
                </ChartFrame>
            </div>
            <div data-testid="linked-b">
                <ChartFrame title="Cancellations" description="Shares the view above.">
                    <LineChart
                        series=churn
                        labels=weeks
                        label="Cancellations per week"
                        zoom=true
                        view=shared
                        on_view_change=sync
                    />
                </ChartFrame>
            </div>

            <ChartFrame title="Open tickets" description="Click a bar to list that team's tickets.">
                <BarChart data=teams label="Open tickets by team" highlight=hydroxy_leptos::bar_chart::Highlight::Off on_select=on_select />
            </ChartFrame>
            <div class="table-wrap" data-testid="linked-table">
                <table>
                    <caption>{move || picked.get().map_or_else(|| "All teams".to_owned(), |t| format!("{t} tickets"))}</caption>
                    <tbody>
                        {move || {
                            let team = picked.get();
                            tickets
                                .iter()
                                .filter(|(t, _)| team.as_deref().is_none_or(|p| p == *t))
                                .map(|(t, title)| view! { <tr><td>{*t}</td><td>{*title}</td></tr> })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "linked-charts",
    name: "Linked charts",
    group: "Lines and areas",
    summary: "Charts report what the user does. Two line charts share one view, so zooming either moves both, and a bar chart's on_select filters a table.",
    demo,
    leptos: include_str!("linked_charts.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/linked_charts.rs"),
    parts: &[],
    keys: &[],
    props: &[
        &[
            "view",
            "Signal<View> (Leptos), ReadSignal<Option<View>> (Dioxus)",
            "Controlled zoom window. Leave it off for an uncontrolled chart",
        ],
        &["on_view_change", "Callback<View>", "Called once a zoom, pan or reset settles"],
        &["on_select", "Callback<Datum>", "Click, tap or Enter on a mark"],
    ],
};
