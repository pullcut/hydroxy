//! Size probe: one `Button` and one `LineChart`. `index.html` links only the CSS they
//! need as plain stylesheets, so none of it is compiled into the wasm.
//!
//! CI builds it with the `wasm-release` profile, runs wasm-bindgen, and compares the
//! `_bg.wasm` size against `wasm-size-budget.txt`. `trunk serve` runs it in a browser.

use hydroxy_leptos::button::Button;
use hydroxy_leptos::line_chart::{LineChart, Series};
use leptos::mount::mount_to_body;
use leptos::prelude::*;

/// A button that adds a point to the chart.
#[component]
fn App() -> impl IntoView {
    let values = RwSignal::new(vec![3.0, 5.0, 4.0, 8.0, 6.0]);
    let series = Signal::derive(move || vec![Series::new("Visits", values.get())]);
    let labels = Signal::derive(move || (1..=values.with(Vec::len)).map(|i| i.to_string()).collect::<Vec<_>>());
    let add = Callback::new(move |_| values.update(|v| v.push(v.last().copied().unwrap_or(0.0) + 1.0)));

    view! {
        <Button on_click=add>"Add point"</Button>
        <LineChart series labels label="Visits".to_owned() />
    }
}

fn main() {
    mount_to_body(App);
}
