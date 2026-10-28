use dioxus::prelude::*;
use hydroxy_dioxus::button::{Button, ButtonVariant};
use hydroxy_dioxus::skeleton::{Skeleton, SkeletonCircle, SkeletonKind, SkeletonText};

#[component]
pub fn SkeletonDemo() -> Element {
    let mut loading = use_signal(|| true);

    rsx! {
        div { style: "display: grid; gap: var(--space-6);",
            div {
                Button { variant: ButtonVariant::Secondary, on_click: move |_| loading.set(!loading()),
                    if loading() { "Show content" } else { "Reload" }
                }
            }
            div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: var(--space-4);",
                // Presets are sized like the components they stand in for.
                Skeleton { loading: loading(), preset: SkeletonKind::StatCard, label: "Loading revenue",
                    div { "data-scope": "stat-card",
                        span { style: "font: var(--text-sm); color: var(--text-2);", "Monthly revenue" }
                        span { style: "font: var(--text-figure); font-size: 30px;", "$48,210" }
                        span { style: "font: var(--text-xs); color: var(--text-3);", "+12.4% on September" }
                    }
                }
                Skeleton { loading: loading(), preset: SkeletonKind::StatCard, label: "Loading signups",
                    div { "data-scope": "stat-card",
                        span { style: "font: var(--text-sm); color: var(--text-2);", "New signups" }
                        span { style: "font: var(--text-figure); font-size: 30px;", "1,284" }
                        span { style: "font: var(--text-xs); color: var(--text-3);", "+3.1% on September" }
                    }
                }
            }
            // A custom placeholder built from the shapes.
            Skeleton {
                loading: loading(),
                label: "Loading comment",
                fallback: rsx! {
                    div { style: "display: flex; gap: var(--space-3);",
                        SkeletonCircle { size: 36 }
                        div { style: "flex: 1;", SkeletonText { lines: 3, seed: 4 } }
                    }
                },
                p { style: "margin: 0; font: var(--text-body);",
                    strong { "Priya Natarajan" }
                    " The churn dip lines up with the onboarding change on the 14th. Worth a cohort split before we call it."
                }
            }
            Skeleton { loading: loading(), preset: SkeletonKind::TableRows(4, 4), label: "Loading invoices",
                p { style: "margin: 0;", "4 invoices, all paid." }
            }
        }
    }
}
