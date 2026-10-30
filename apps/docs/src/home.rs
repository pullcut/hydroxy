//! The homepage: what hydroxy is, a live preview, how it's built, and the catalogue.

use hydroxy_leptos::badge::{Badge, Tone};
use hydroxy_leptos::card::{Card, CardTone, Jewel};
use hydroxy_leptos::line_chart::{Curve, LineChart, Series};
use hydroxy_leptos::presence_bar::{Collaborator, PresenceBar, PresenceStatus};
use hydroxy_leptos::segmented::{Segmented, SegmentedItem, SegmentedOption};
use hydroxy_leptos::stat_ring::StatRing;
use hydroxy_leptos::switch::Switch;
use leptos::prelude::*;

use crate::app::{DOCS, groups};
use crate::doc::Section;
use crate::seo::{self, Page};

const INSTALL: &str = "cargo add hydroxy-leptos hydroxy-tokens";

/// `/`.
#[allow(unreachable_pub, reason = "#[component] makes the generated props struct pub")]
#[component]
pub(crate) fn Home() -> impl IntoView {
    let layers: [(&str, &str, Jewel); 4] = [
        (
            "Machines",
            "Each component's behaviour is a pure Rust state machine: events in, state and commands out. Unit-tested without a browser.",
            Jewel::Amethyst,
        ),
        (
            "Connectors",
            "A headless layer turns machine state into ARIA attributes, roving focus and key handling, as plain data.",
            Jewel::Sapphire,
        ),
        (
            "Adapters",
            "Thin Leptos and Dioxus components render the connector. Same behaviour, same markup, either framework.",
            Jewel::Emerald,
        ),
        (
            "Tokens",
            "One stylesheet of design tokens and component styles, light and dark. Override a variable to restyle everything.",
            Jewel::Citrine,
        ),
    ];

    view! {
        {seo::head(Page::home())}
        <main class="home" id="main">
            <section class="hero">
                <div class="hero-copy">
                    <h1 class="display">"Accessible components for Leptos and Dioxus."</h1>
                    <p class="hero-lede">
                        "Behaviour lives in tested state machines. A headless layer turns it into ARIA, focus and keyboard support. Thin adapters render it in either framework, styled by one set of design tokens."
                    </p>
                    <div class="hero-actions">
                        <a class="ox-button" data-variant="primary" data-size="lg" href="/docs/getting-started">"Get started"</a>
                        <a class="ox-button" data-variant="secondary" data-size="lg" href="/components">"Browse components"</a>
                    </div>
                    <div class="install">
                        <span class="install-prompt" aria-hidden="true">"$"</span>
                        <code>{INSTALL}</code>
                    </div>
                </div>
                <HeroPreview />
            </section>

            <section class="home-section">
                <h2 class="section-title">"Four layers, each with one job"</h2>
                <div class="layer-grid">
                    {layers
                        .into_iter()
                        .map(|(title, body, jewel)| {
                            view! {
                                <Card tone=CardTone::Jewel(jewel) padding=28>
                                    <h3 class="layer-title">{title}</h3>
                                    <p class="layer-body">{body}</p>
                                </Card>
                            }
                        })
                        .collect_view()}
                </div>
            </section>

            <section class="home-section">
                <div class="section-head">
                    <div>
                        <h2 class="section-title">{format!("{} components and {} charts", ordered_count(Section::Components), ordered_count(Section::Charts))}</h2>
                    </div>
                    <a class="ox-button" data-variant="ghost" data-size="md" href="/components">"See them all"</a>
                </div>
                <div class="catalogue">
                    {groups(Section::Components)
                        .into_iter()
                        .chain(groups(Section::Charts))
                        .map(|(group, docs)| {
                            view! {
                                <div class="catalogue-group">
                                    <p class="label">{group}</p>
                                    <ul>
                                        {docs
                                            .into_iter()
                                            .map(|d| view! { <li><a href=d.href()>{d.name}</a></li> })
                                            .collect_view()}
                                    </ul>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </section>

            <footer class="site-footer">
                <span>"hydroxy"</span>
                <span>"MIT licensed · Leptos 0.8 · Dioxus 0.7"</span>
            </footer>
        </main>
    }
}

/// One time range of the hero chart.
struct Window {
    id: &'static str,
    ticks: &'static [&'static str],
    now: &'static [f64],
    before: &'static [f64],
    total: &'static str,
    delta: &'static str,
}

/// Three ranges of the same metric, so the range switch visibly reshapes the chart.
const WINDOWS: [Window; 3] = [
    Window {
        id: "7d",
        ticks: &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        now: &[1820.0, 1940.0, 2010.0, 1985.0, 2140.0, 1710.0, 1655.0],
        before: &[1700.0, 1760.0, 1795.0, 1810.0, 1880.0, 1590.0, 1540.0],
        total: "13,260",
        delta: "+8.4%",
    },
    Window {
        id: "30d",
        ticks: &["1 Oct", "4 Oct", "7 Oct", "10 Oct", "13 Oct", "16 Oct", "19 Oct", "22 Oct", "25 Oct", "28 Oct"],
        now: &[5200.0, 5480.0, 5390.0, 5820.0, 6110.0, 5990.0, 6430.0, 6710.0, 6650.0, 7120.0],
        before: &[4800.0, 4920.0, 5010.0, 4980.0, 5150.0, 5230.0, 5190.0, 5380.0, 5460.0, 5520.0],
        total: "59,880",
        delta: "+18.2%",
    },
    Window {
        id: "90d",
        ticks: &["Aug", "", "", "", "Sep", "", "", "", "Oct", "", "", ""],
        now: &[
            14_200.0, 14_900.0, 15_100.0, 16_400.0, 16_000.0, 17_300.0, 18_100.0, 18_000.0, 19_600.0, 20_400.0,
            21_200.0, 22_900.0,
        ],
        before: &[
            13_100.0, 13_400.0, 13_200.0, 13_900.0, 14_100.0, 14_000.0, 14_600.0, 14_900.0, 15_200.0, 15_100.0,
            15_800.0, 16_000.0,
        ],
        total: "212k",
        delta: "+31.5%",
    },
];

/// Axis and tooltip numbers: 1.8k, 12k.
fn compact(v: f64) -> String {
    if v.abs() < 1000.0 {
        return format!("{v:.0}");
    }
    let k = format!("{:.1}", v / 1000.0);
    format!("{}k", k.strip_suffix(".0").unwrap_or(&k))
}

/// Real components, rendered on the server and interactive after hydration.
#[component]
fn HeroPreview() -> impl IntoView {
    let range = RwSignal::new("30d".to_string());
    let window = move || WINDOWS.iter().find(|w| range.with(|r| r == w.id)).unwrap_or(&WINDOWS[1]);
    let series = Signal::derive(move || {
        let w = window();
        vec![Series::new("This period", w.now.to_vec()), Series::new("Previous period", w.before.to_vec())]
    });
    let labels = Signal::derive(move || window().ticks.iter().map(|t| (*t).to_string()).collect::<Vec<_>>());
    let range_items: Vec<SegmentedOption> = WINDOWS.iter().map(|w| SegmentedOption::new(w.id)).collect();
    let team = vec![
        Collaborator::new("u-maya", "Maya Okafor"),
        Collaborator::new("u-jonas", "Jonas Berg"),
        Collaborator::new("u-priya", "Priya Raman").status(PresenceStatus::Idle),
        Collaborator::new("u-leo", "Leo Martins"),
        Collaborator::new("u-sana", "Sana Haddad"),
    ];

    view! {
        <div class="hero-stage" role="group" aria-label="Live components">
            <div class="hero-card hero-main">
                <div class="hero-card-head">
                    <div>
                        <p class="hero-metric-label">"Active users"</p>
                        <p class="hero-metric">
                            <span class="hero-metric-value">{move || window().total}</span>
                            <Badge tone=Tone::Success>{move || window().delta}</Badge>
                        </p>
                    </div>
                    <Segmented
                        label="Time range"
                        items=range_items
                        default_value="30d"
                        on_value_change=move |v: String| range.set(v)
                    >
                        <SegmentedItem value="7d" label="7D" />
                        <SegmentedItem value="30d" label="30D" />
                        <SegmentedItem value="90d" label="90D" />
                    </Segmented>
                </div>
                <LineChart series labels label="Active users" height=190 curve=Curve::Monotone format=compact />
            </div>

            <div class="hero-card hero-float hero-release">
                <StatRing value=41.0 goal=50.0 label="Checks" size=80 thickness=7 />
                <div class="hero-float-body">
                    <p class="hero-float-title">"Release 0.4"</p>
                    <p class="hero-float-sub">"41 of 50 checks passed"</p>
                    <Switch default_checked=true>"Auto-deploy"</Switch>
                </div>
            </div>

            <div class="hero-card hero-float hero-team">
                <PresenceBar people=team max=4 size=32 />
                <p class="hero-float-sub">"5 people viewing"</p>
            </div>
        </div>
    }
}

fn ordered_count(section: Section) -> usize {
    DOCS.iter().filter(|d| d.section() == section).count()
}
