use dioxus::prelude::*;
use hydroxy_dioxus::icicle_chart::{IcicleChart, Tree};

fn module(name: &str, files: &[(&str, f64)]) -> Tree {
    Tree::branch(name, files.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

#[component]
pub fn IcicleChartDemo() -> Element {
    // A web app's JavaScript bundle, in kB after minification.
    let bundle = Tree::branch(
        "main.js",
        vec![
            Tree::branch(
                "node_modules",
                vec![
                    module("react-dom", &[("client", 118.0), ("server", 12.0)]),
                    module("date-fns", &[("locale", 46.0), ("format", 9.0), ("parse", 7.0)]),
                    module("chart.js", &[("controllers", 31.0), ("scales", 24.0), ("core", 38.0)]),
                    Tree::leaf("lodash", 24.0),
                ],
            ),
            Tree::branch(
                "src",
                vec![
                    module("pages", &[("dashboard", 28.0), ("settings", 14.0), ("billing", 11.0)]),
                    module("components", &[("table", 17.0), ("forms", 13.0), ("charts", 9.0)]),
                    Tree::leaf("api", 8.0),
                ],
            ),
            Tree::leaf("polyfills", 19.0),
        ],
    );

    rsx! {
        IcicleChart { root: bundle, label: "Bundle size by module", format: |v: f64| format!("{v:.0} kB") }
    }
}
