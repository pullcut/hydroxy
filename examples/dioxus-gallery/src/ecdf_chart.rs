use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::ecdf_chart::{Annotation, EcdfChart, Sample};

/// Page loads in seconds around `median`, skewed right like real load times.
fn loads(median: f64, spread: f64, seed: u32) -> Vec<f64> {
    let mut s = seed;
    let mut next = move || {
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(s >> 8) / f64::from(1u32 << 24)
    };
    (0..240).map(|_| median * (spread * (next() + next() + next() + next() - 2.0) * 1.7).exp()).collect()
}

#[component]
pub fn EcdfChartDemo() -> Element {
    let regions = vec![Sample::new("Europe", loads(1.4, 0.28, 7)), Sample::new("Americas", loads(1.7, 0.32, 11)), Sample::new("Asia Pacific", loads(2.3, 0.4, 23))];

    rsx! {
        ChartFrame { title: "Page load time by region", description: "Share of page loads finished within each time. Dashed lines are normal fits.",
            EcdfChart {
                data: regions,
                label: "Cumulative distribution of page load time by region",
                reference: true,
                format: move |v: f64| format!("{v:.1} s"),
                annotations: vec![Annotation::x_line(3.0, "3 s budget")],
            }
        }
    }
}
