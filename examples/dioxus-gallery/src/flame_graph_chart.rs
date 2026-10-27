use dioxus::prelude::*;
use oxide_dioxus::flame_graph_chart::{FlameGraphChart, Tree};
use oxide_dioxus::search_field::SearchField;

// A frame's value is its total time; whatever its callees don't cover is self time.
fn frame(name: &str, total: f64, calls: Vec<Tree>) -> Tree {
    Tree { name: name.into(), value: total, children: calls }
}

#[component]
pub fn FlameGraphChartDemo() -> Element {
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
    let mut query = use_signal(|| "json".to_string());

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            div { style: "max-width: 280px;",
                SearchField {
                    label: "Highlight frames",
                    placeholder: "Highlight frames",
                    value: Some(query()),
                    on_input: move |v| query.set(v),
                }
            }
            FlameGraphChart {
                root: profile,
                label: "CPU time for a checkout request",
                highlight: query,
                format: |v: f64| format!("{v:.0} ms"),
            }
        }
    }
}
