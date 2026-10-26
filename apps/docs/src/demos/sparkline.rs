use leptos::prelude::*;
use oxide_leptos::sparkline::*;

pub fn demo() -> AnyView {
    let signups = vec![12.0, 15.0, 11.0, 18.0, 22.0, 19.0, 25.0, 24.0, 29.0, 31.0, 28.0, 34.0];
    let churn = vec![4.1, 3.8, 4.4, 4.0, 3.6, 3.9, 3.2, 3.4, 3.0, 2.8];

    view! {
        <div style="display: grid; gap: var(--space-6);">
            <p style="margin: 0; font: var(--text-md); color: var(--text-2);">
                "Signups this quarter " <Sparkline values=signups.clone() label="Signups this quarter" /> " are up 183%."
            </p>
            // Bigger, filled, smooth, and recoloured with --sparkline-color.
            <span style="--sparkline-color: var(--chart-4);">
                <Sparkline values=signups label="Signups, filled" width=160 height=40 area=true curve=Curve::Monotone />
            </span>
            <span style="--sparkline-color: var(--chart-2);">
                <Sparkline
                    values=churn
                    label="Monthly churn"
                    width=160
                    height=40
                    format=|v: f64| format!("{v:.1}%")
                />
            </span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "sparkline",
    name: "Sparkline",
    group: "Data",
    summary: "A word-sized trend line for tables, stat cards and running text. No axes: it shows shape, and its accessible name gives the low, high and latest values.",
    demo,
    leptos: include_str!("sparkline.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sparkline.rs"),
    parts: &[
        &["Sparkline", "root", "svg with role=\"img\"; colour from --sparkline-color, --chart-1 by default"],
        &["", "line", "1.5px stroke; gaps where values are missing"],
        &["", "area", "Optional fill under the line"],
        &["", "dot", "The latest value"],
    ],
    keys: &[],
    props: &[
        &["values", "Vec<f64>", "Required. Oldest first; NaN leaves a gap"],
        &["label", "String", "Required. Starts the accessible name"],
        &["width", "u16", "Width in px. Default 96"],
        &["height", "u16", "Height in px. Default 24"],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["area", "bool", "Fill under the line"],
        &["dot", "bool", "Mark the latest value. Default true"],
        &["format", "Fn(f64) -> String", "Formats values in the accessible summary"],
    ],
};
