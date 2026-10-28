//! The homepage: what oxide is, a live preview, how it's built, and the catalogue.

use leptos::prelude::*;
use leptos_meta::Title;
use oxide_leptos::badge::{Badge, Tone};
use oxide_leptos::bar_chart::{BarChart, Datum};
use oxide_leptos::card::{Card, CardTone, Jewel};
use oxide_leptos::segmented::{Segmented, SegmentedItem, SegmentedOption};
use oxide_leptos::switch::Switch;
use oxide_leptos::tabs::*;

use crate::app::{DOCS, groups};
use crate::doc::Section;

const INSTALL: &str = "cargo add oxide-leptos oxide-tokens";

/// `/`.
#[component]
pub fn Home() -> impl IntoView {
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
        <Title text="" />
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
                <span>"oxide"</span>
                <span>"MIT licensed · Leptos 0.8 · Dioxus 0.7"</span>
            </footer>
        </main>
    }
}

/// Real components, rendered on the server and interactive after hydration.
#[component]
fn HeroPreview() -> impl IntoView {
    let weeks = vec![
        Datum::new("Mon", 6.0),
        Datum::new("Tue", 9.0),
        Datum::new("Wed", 7.0),
        Datum::new("Thu", 12.0),
        Datum::new("Fri", 15.0),
    ];
    view! {
        <div class="hero-preview" aria-label="Live components">
            <Tabs tabs=vec![TabsItem::new("summary"), TabsItem::new("sources"), TabsItem::new("activity")] default_value="summary">
                <TabsList>
                    <TabsTrigger value="summary">"Summary"</TabsTrigger>
                    <TabsTrigger value="sources">"Sources"</TabsTrigger>
                    <TabsTrigger value="activity">"Activity"</TabsTrigger>
                </TabsList>
                <TabsContent value="summary">
                    <div class="preview-row">
                        <Badge tone=Tone::Success dot=true>"Ready"</Badge>
                        <Badge tone=Tone::Warning>"2 flags"</Badge>
                    </div>
                    <BarChart data=weeks label="Interviews read this week" height=120 />
                </TabsContent>
                <TabsContent value="sources">
                    <p class="preview-text">"interviews-q3.pdf · tickets-sept.csv · survey-results.xlsx"</p>
                </TabsContent>
                <TabsContent value="activity">
                    <p class="preview-text">"Summary drafted 26 October. Shared with the workspace."</p>
                </TabsContent>
            </Tabs>
            <div class="preview-controls">
                <Switch default_checked=true>"Email me when it's ready"</Switch>
                <Segmented
                    label="Summary length"
                    items=vec![SegmentedOption::new("short"), SegmentedOption::new("long")]
                    default_value="short"
                >
                    <SegmentedItem value="short" label="Short" />
                    <SegmentedItem value="long" label="Long" />
                </Segmented>
            </div>
        </div>
    }
}

fn ordered_count(section: Section) -> usize {
    DOCS.iter().filter(|d| d.section() == section).count()
}
