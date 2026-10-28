use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::raincloud_chart::{Annotation, Group, Orientation, RaincloudChart};

/// `n` page load times in ms around `median`, skewed right like real loads. A fixed
/// generator, so the sample is the same on every render.
fn loads(seed: u32, n: usize, median: f64) -> Vec<f64> {
    let mut s = seed;
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let u = f64::from(s >> 8) / f64::from(1u32 << 24);
            (median * (0.6 + 1.3 * u * u)).round()
        })
        .collect()
}

fn regions() -> Vec<Group> {
    vec![
        Group::new("Europe", loads(7, 60, 820.0)),
        Group::new("North America", loads(11, 60, 760.0)),
        Group::new("Asia Pacific", loads(23, 48, 1080.0)),
        Group::new("South America", loads(41, 36, 1290.0)),
    ]
}

#[component]
pub fn RaincloudChartDemo() -> Element {
    rsx! {
        ChartFrame { title: "Page load time by region", description: "Last week's checkout page loads: the shape, the quartiles and every visit.",
            RaincloudChart {
                groups: regions(),
                orientation: Orientation::Horizontal,
                label: "Checkout page load time by region",
                annotations: vec![Annotation::x_line(1000.0, "1 s budget")],
                format: |v: f64| format!("{v:.0} ms"),
            }
        }
    }
}
