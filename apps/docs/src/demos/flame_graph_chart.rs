use hydroxy_leptos::flame_graph_chart::*;
use hydroxy_leptos::search_field::SearchField;
use leptos::prelude::*;

// A frame's value is its total time; whatever its callees don't cover is self time.
fn frame(name: &str, total: f64, calls: Vec<Tree>) -> Tree {
    Tree { name: name.into(), value: total, children: calls }
}

#[component]
pub fn Demo() -> impl IntoView {
    // CPU time for one checkout request, in ms.
    let profile = frame(
        "handle_request",
        412.0,
        vec![
            frame("authenticate", 38.0, vec![Tree::leaf("verify_jwt", 21.0), Tree::leaf("load_session", 12.0)]),
            frame(
                "load_cart",
                146.0,
                vec![
                    frame("db_query", 98.0, vec![Tree::leaf("pool_acquire", 14.0), Tree::leaf("parse_rows", 31.0)]),
                    Tree::leaf("price_items", 36.0),
                ],
            ),
            frame(
                "render",
                188.0,
                vec![
                    frame("serialize_json", 72.0, vec![Tree::leaf("escape_strings", 19.0)]),
                    frame("template", 94.0, vec![Tree::leaf("parse_json", 22.0), Tree::leaf("layout", 51.0)]),
                ],
            ),
            Tree::leaf("log_access", 17.0),
        ],
    );
    let (query, set_query) = signal("json".to_string());

    view! {
        <div style="display: grid; gap: var(--space-5);">
            <div style="max-width: 280px;">
                <SearchField label="Highlight frames" placeholder="Highlight frames" value=query on_input=move |v| set_query.set(v) />
            </div>
            <FlameGraphChart root=profile label="CPU time for a checkout request" highlight=query format=|v: f64| format!("{v:.0} ms") />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "flame-graph-chart",
    name: "Flame graph chart",
    group: "Parts of a whole",
    summary: "A profile as stacked frames, callers below callees, each frame as wide as its total time. Frames colour by name, the tooltip splits self from total time, and a search highlights matching frames.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("flame_graph_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/flame_graph_chart.rs"),
    parts: &[
        &["FlameGraphChart", "root", "figure; `data-searching` while a highlight is set"],
        &["", "breadcrumb", "Path from the root frame to the current zoom"],
        &["", "marks", "Focusable SVG; arrow keys move between frames"],
        &["", "cell", "One frame; `data-match` when it matches the highlight, `data-active`"],
        &["", "tooltip", "Total time, self time and share of the zoomed frame"],
        &["", "sr-table", "Every frame with its total and self time"],
    ],
    keys: &[
        &["Arrow keys", "Move between frames in call order"],
        &["Home / End", "Jump to the first or last frame"],
        &["Enter / Space", "Zoom into the focused frame, or out from the bottom row"],
        &["Escape", "Clear the focused frame, then zoom out"],
    ],
    props: &[
        &["root", "Tree", "Required. A branch's value is its total time; the rest after its callees is self time"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["highlight", "String", "Marks frames whose name contains it, ignoring case; the rest dim"],
        &["height", "u16", "Plot height in px at an 800px drawing width. Default 320"],
        &["format", "Fn(f64) -> String", "Formats times in the tooltip and table"],
    ],
    events: crate::doc::NODE_EVENTS,
};
