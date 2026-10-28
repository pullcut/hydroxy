//! Size probe: one `Button` and one `LineChart` with only the CSS they need.
//!
//! Build with `trunk build --release` (see `index.html`) and compare the `.wasm` size
//! against the budget in CI.

use hydroxy_leptos::button::Button;
use hydroxy_leptos::line_chart::{LineChart, Series};
use hydroxy_tokens::css;
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
    // Only the sheets the two components use, not `hydroxy_tokens::CSS`.
    let style = [css::BASE, css::BUTTON, css::CHART, css::CHART_CARTESIAN].concat();
    mount_to_body(move || {
        view! {
            <style>{style.clone()}</style>
            <App />
        }
    });
}
