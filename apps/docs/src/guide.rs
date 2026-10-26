//! Guides.

use leptos::prelude::*;
use leptos_meta::Title;
use oxide_leptos::callout::{Callout, CalloutKind};
use oxide_leptos::tabs::*;

const LEPTOS_EXAMPLE: &str = r#"use leptos::prelude::*;
use oxide_leptos::switch::Switch;

#[component]
fn App() -> impl IntoView {
    view! {
        // Once, at the root: fonts, then tokens and component styles.
        <style inner_html=format!("{}{}", oxide_tokens::GOOGLE_FONTS, oxide_tokens::CSS)></style>
        <Switch default_checked=true>"Email me when a summary is ready"</Switch>
    }
}"#;

const DIOXUS_EXAMPLE: &str = r#"use dioxus::prelude::*;
use oxide_dioxus::switch::Switch;

#[component]
fn App() -> Element {
    rsx! {
        // Once, at the root: fonts, then tokens and component styles.
        style { {oxide_tokens::GOOGLE_FONTS} {oxide_tokens::CSS} }
        Switch { default_checked: true, "Email me when a summary is ready" }
    }
}"#;

const SSR_EXAMPLE: &str = r"#[component]
pub fn App() -> impl IntoView {
    // First thing in the root component.
    oxide_leptos::id::provide_id_scope();
    // ...
}";

const THEME_EXAMPLE: &str = r#"/* Dark theme: on <html>, or any container */
<html data-theme="dark">

/* Restyle by overriding tokens */
:root {
  --accent: var(--sapphire-500);
  --radius-md: 6px;
}"#;

/// `/docs/getting-started`.
#[component]
pub fn GettingStarted() -> impl IntoView {
    view! {
        <Title text="Getting started" />
        <div class="doc-grid">
            <article class="page prose">
                <h1>"Getting started"</h1>
                <p class="lede">
                    "oxide is a component library for Leptos and Dioxus. Every component works the same in both, because the behaviour lives below the framework."
                </p>

                <h2 id="install">"Install"</h2>
                <p>"Add the adapter for your framework and the stylesheet crate."</p>
                <Tabs tabs=vec![TabsItem::new("leptos"), TabsItem::new("dioxus")] default_value="leptos">
                    <TabsList>
                        <TabsTrigger value="leptos">"Leptos"</TabsTrigger>
                        <TabsTrigger value="dioxus">"Dioxus"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="leptos"><pre class="code"><code>"cargo add oxide-leptos oxide-tokens"</code></pre></TabsContent>
                    <TabsContent value="dioxus"><pre class="code"><code>"cargo add oxide-dioxus oxide-tokens"</code></pre></TabsContent>
                </Tabs>

                <h2 id="use">"Use a component"</h2>
                <p>
                    "Inject the stylesheet once at the root, then use components. Each component page has a live example and the code for both frameworks."
                </p>
                <Tabs tabs=vec![TabsItem::new("leptos"), TabsItem::new("dioxus")] default_value="leptos">
                    <TabsList>
                        <TabsTrigger value="leptos">"Leptos"</TabsTrigger>
                        <TabsTrigger value="dioxus">"Dioxus"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="leptos"><pre class="code"><code>{LEPTOS_EXAMPLE}</code></pre></TabsContent>
                    <TabsContent value="dioxus"><pre class="code"><code>{DIOXUS_EXAMPLE}</code></pre></TabsContent>
                </Tabs>

                <h2 id="ssr">"Server rendering"</h2>
                <p>
                    "Components generate ids to link labels, panels and focus targets. With Leptos server rendering, start an id scope in the root component so the server and the browser give every component the same id."
                </p>
                <pre class="code"><code>{SSR_EXAMPLE}</code></pre>
                <p>"This site is rendered that way: each page arrives as HTML, then hydrates."</p>

                <h2 id="theming">"Theming"</h2>
                <p>
                    "Everything is styled by CSS custom properties: six jewel colours, ink and paper neutrals, type, spacing, radii, shadows and motion. Light is the default; dark applies under "
                    <code>"data-theme=\"dark\""</code>
                    ". Override any token to restyle every component."
                </p>
                <pre class="code"><code>{THEME_EXAMPLE}</code></pre>
                <Callout kind=CalloutKind::Note title="Charts use their own order">
                    "Chart colours run amethyst, garnet, sapphire, emerald, citrine, rhodolite. That order is checked for colour-blind separation and contrast in both themes."
                </Callout>

                <h2 id="architecture">"How it's built"</h2>
                <ol class="steps">
                    <li><strong>"oxide-statechart"</strong>" — the Machine trait: initial, transition and sync return state plus commands."</li>
                    <li><strong>"oxide-machines"</strong>" — one pure machine per component, with unit tests."</li>
                    <li><strong>"oxide-headless"</strong>" — connectors: ARIA attributes, data attributes and key mapping, as plain data."</li>
                    <li><strong>"oxide-leptos, oxide-dioxus"</strong>" — adapters that render the connector and run commands such as focus."</li>
                    <li><strong>"oxide-tokens, oxide-charts"</strong>" — the stylesheet, and chart scales and geometry."</li>
                </ol>
            </article>
            <nav class="toc" aria-label="On this page">
                <p class="label">"On this page"</p>
                <a href="#install">"Install"</a>
                <a href="#use">"Use a component"</a>
                <a href="#ssr">"Server rendering"</a>
                <a href="#theming">"Theming"</a>
                <a href="#architecture">"How it's built"</a>
            </nav>
        </div>
    }
}
