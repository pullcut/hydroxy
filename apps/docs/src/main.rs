//! oxide docs. Run from this folder: `trunk serve --open`.
//! Add a component: a demo module exporting `DOC`, listed in `DOCS`.

#![allow(clippy::wildcard_imports, reason = "demos are shown as source; prelude globs are what users write")]

use leptos::prelude::*;
use oxide_leptos::tabs::*;

mod demos;
mod doc;

use doc::Doc;

/// Pages in sidebar order. Pages with the same `group` must sit together.
const DOCS: &[Doc] = &[
    demos::button::DOC,
    demos::icon_button::DOC,
    demos::avatar::DOC,
    demos::badge::DOC,
    demos::card::DOC,
    demos::chip::DOC,
    demos::kbd::DOC,
    demos::tag::DOC,
    demos::checkbox::DOC,
    demos::combobox::DOC,
    demos::input::DOC,
    demos::radio_group::DOC,
    demos::search_field::DOC,
    demos::segmented::DOC,
    demos::select::DOC,
    demos::slider::DOC,
    demos::switch::DOC,
    demos::tags_input::DOC,
    demos::callout::DOC,
    demos::empty_state::DOC,
    demos::progress_bar::DOC,
    demos::toast::DOC,
    demos::accordion::DOC,
    demos::collapsible::DOC,
    demos::tabs::DOC,
    demos::command_palette::DOC,
    demos::dialog::DOC,
    demos::menu::DOC,
    demos::popover::DOC,
    demos::tooltip::DOC,
];

fn main() {
    mount_to_body(App);
}

fn slug_from_hash() -> String {
    window().location().hash().unwrap_or_default().trim_start_matches("#/").to_string()
}

#[component]
fn App() -> impl IntoView {
    let slug = RwSignal::new(slug_from_hash());
    let _ = window_event_listener(leptos::ev::hashchange, move |_| slug.set(slug_from_hash()));
    let doc = move || DOCS.iter().find(|d| d.slug == slug.get()).unwrap_or(&DOCS[0]);
    let dark = RwSignal::new(false);
    // Theme lives on <html> so the page background, scrollbar gutter and top-layer
    // overlays all pick it up, not just the app shell.
    Effect::new(move |_| {
        let theme = if dark.get() { "dark" } else { "light" };
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("data-theme", theme);
        }
    });

    view! {
        <style>{oxide_tokens::CSS}</style>
        <div class="shell">
            <nav class="sidebar">
                <a class="brand" href="#/">"oxide"</a>
                <Nav doc=Signal::derive(move || doc().slug) />
                <button class="theme-toggle" on:click=move |_| dark.update(|d| *d = !*d)>
                    {move || if dark.get() { "Light theme" } else { "Dark theme" }}
                </button>
            </nav>
            <main>{move || view! { <Page doc=doc() /> }}</main>
        </div>
    }
}

#[component]
fn Nav(doc: Signal<&'static str>) -> impl IntoView {
    let mut groups: Vec<&str> = DOCS.iter().map(|d| d.group).collect();
    groups.dedup();
    groups
        .into_iter()
        .map(|group| {
            view! {
                <div class="nav">
                    <p class="label">{group}</p>
                    {DOCS
                        .iter()
                        .filter(|d| d.group == group)
                        .map(|d| {
                            view! {
                                <a
                                    href=format!("#/{}", d.slug)
                                    aria-current=move || (doc.get() == d.slug).then_some("page")
                                >
                                    {d.name}
                                </a>
                            }
                        })
                        .collect_view()}
                </div>
            }
        })
        .collect_view()
}

#[component]
fn Page(doc: &'static Doc) -> impl IntoView {
    view! {
        <article class="page">
            <p class="label">{doc.group}</p>
            <h1>{doc.name}</h1>
            <p class="lede">{doc.summary}</p>
            <section class="demo">{(doc.demo)()}</section>

            <h2>"Usage"</h2>
            <Tabs tabs=vec![TabsItem::new("leptos"), TabsItem::new("dioxus")] default_value="leptos">
                <TabsList>
                    <TabsTrigger value="leptos">"Leptos"</TabsTrigger>
                    <TabsTrigger value="dioxus">"Dioxus"</TabsTrigger>
                </TabsList>
                <TabsContent value="leptos"><pre class="code"><code>{doc.leptos_code()}</code></pre></TabsContent>
                <TabsContent value="dioxus"><pre class="code"><code>{doc.dioxus}</code></pre></TabsContent>
            </Tabs>

            <Section title="Anatomy" head=&["Component", "data-part", "Renders"] rows=doc.parts />
            <Section title="Keyboard" head=&["Key", "Does"] rows=doc.keys />
            <Section title="Props" head=&["Prop", "Type", "Notes"] rows=doc.props />
        </article>
    }
}

/// A titled table, left out entirely when it has no rows.
#[component]
fn Section(
    title: &'static str,
    head: &'static [&'static str],
    rows: &'static [&'static [&'static str]],
) -> impl IntoView {
    (!rows.is_empty()).then(|| view! { <h2>{title}</h2> <Table head rows /> })
}

#[component]
fn Table(head: &'static [&'static str], rows: &'static [&'static [&'static str]]) -> impl IntoView {
    view! {
        <table>
            <thead><tr>{head.iter().map(|h| view! { <th>{*h}</th> }).collect_view()}</tr></thead>
            <tbody>
                {rows
                    .iter()
                    .map(|row| {
                        view! {
                            <tr>
                                <td><code>{row[0]}</code></td>
                                {row[1..].iter().map(|c| view! { <td>{*c}</td> }).collect_view()}
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}
