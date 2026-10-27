//! The document shell, routes, and the docs chrome: header, sidebar, component pages.

use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_location, use_navigate, use_params_map};
use leptos_router::path;
use oxide_leptos::command_palette::{CommandItem, CommandPalette};
use oxide_leptos::icon::{Icon, IconName};
use oxide_leptos::id::provide_id_scope;
use oxide_leptos::tabs::*;
use oxide_leptos::theme_menu::ThemeMenu;

use crate::demos;
use crate::doc::Doc;
use crate::guide::GettingStarted;
use crate::home::Home;

/// Component pages in sidebar order. Pages with the same `group` must sit together.
pub(crate) const DOCS: &[Doc] = &[
    demos::button::DOC,
    demos::icon_button::DOC,
    demos::avatar::DOC,
    demos::badge::DOC,
    demos::card::DOC,
    demos::chip::DOC,
    demos::code_block::DOC,
    demos::markdown::DOC,
    demos::doc_card::DOC,
    demos::due_pill::DOC,
    demos::kbd::DOC,
    demos::priority_mark::DOC,
    demos::source_chip::DOC,
    demos::space_icon::DOC,
    demos::status_diamond::DOC,
    demos::tag::DOC,
    demos::value_diff::DOC,
    demos::carousel::DOC,
    demos::reveal::DOC,
    demos::media_timeline::DOC,
    demos::calendar::DOC,
    demos::checkbox::DOC,
    demos::combobox::DOC,
    demos::date_picker::DOC,
    demos::color_picker::DOC,
    demos::date_field::DOC,
    demos::date_range_picker::DOC,
    demos::duration_field::DOC,
    demos::quantity_stepper::DOC,
    demos::filter_chip::DOC,
    demos::input::DOC,
    demos::markdown_editor::DOC,
    demos::radio_group::DOC,
    demos::search_field::DOC,
    demos::segmented::DOC,
    demos::select::DOC,
    demos::slider::DOC,
    demos::range_slider::DOC,
    demos::rating::DOC,
    demos::toggle_button::DOC,
    demos::file_dropzone::DOC,
    demos::upload_queue::DOC,
    demos::attachment::DOC,
    demos::switch::DOC,
    demos::tags_input::DOC,
    demos::number_field::DOC,
    demos::time_field::DOC,
    demos::otp_input::DOC,
    demos::questionnaire::DOC,
    demos::availability_picker::DOC,
    demos::voice_input::DOC,
    demos::transcript_viewer::DOC,
    demos::table_of_contents::DOC,
    demos::recording_control::DOC,
    demos::master_detail::DOC,
    demos::audio_waveform::DOC,
    demos::audio_player::DOC,
    demos::app_shell::DOC,
    demos::callout::DOC,
    demos::check_list::DOC,
    demos::empty_state::DOC,
    demos::progress_bar::DOC,
    demos::toast::DOC,
    demos::approval_card::DOC,
    demos::bar_chart::DOC,
    demos::chart_frame::DOC,
    demos::scatter_chart::DOC,
    demos::heatmap_chart::DOC,
    demos::radial_charts::DOC,
    demos::bump_chart::DOC,
    demos::slope_chart::DOC,
    demos::radar_chart::DOC,
    demos::contour_chart::DOC,
    demos::geo_map::DOC,
    demos::bubble_map_chart::DOC,
    demos::hexbin_map_chart::DOC,
    demos::smith_chart::DOC,
    demos::donut_chart::DOC,
    demos::sankey_chart::DOC,
    demos::network_chart::DOC,
    demos::chord_chart::DOC,
    demos::adjacency_matrix_chart::DOC,
    demos::dependency_graph_chart::DOC,
    demos::treemap_chart::DOC,
    demos::sunburst_chart::DOC,
    demos::bubble_pack_chart::DOC,
    demos::icicle_chart::DOC,
    demos::flame_graph_chart::DOC,
    demos::tree_chart::DOC,
    demos::line_chart::DOC,
    demos::histogram::DOC,
    demos::box_plot::DOC,
    demos::candlestick_chart::DOC,
    demos::composed_chart::DOC,
    demos::strip_plot_chart::DOC,
    demos::beeswarm_chart::DOC,
    demos::density_chart::DOC,
    demos::ridgeline_chart::DOC,
    demos::violin_chart::DOC,
    demos::ohlc_chart::DOC,
    demos::depth_chart::DOC,
    demos::renko_chart::DOC,
    demos::timeline_chart::DOC,
    demos::stacked_bar_chart::DOC,
    demos::pie_chart::DOC,
    demos::polar_area_chart::DOC,
    demos::funnel_chart::DOC,
    demos::bullet_chart::DOC,
    demos::waffle_chart::DOC,
    demos::marimekko_chart::DOC,
    demos::metric_card::DOC,
    demos::stat_ring::DOC,
    demos::word_cloud_chart::DOC,
    demos::ternary_chart::DOC,
    demos::dot_plot_chart::DOC,
    demos::error_bar_chart::DOC,
    demos::pareto_chart::DOC,
    demos::waterfall_chart::DOC,
    demos::lollipop_chart::DOC,
    demos::grouped_bar_chart::DOC,
    demos::sparkline::DOC,
    demos::stat_card::DOC,
    demos::step_track::DOC,
    demos::kanban::DOC,
    demos::inline_edit::DOC,
    demos::async_validated_input::DOC,
    demos::shortcut_recorder::DOC,
    demos::sortable_list::DOC,
    demos::filter_builder::DOC,
    demos::data_table::DOC,
    demos::grid::DOC,
    demos::access_editor::DOC,
    demos::file_manager::DOC,
    demos::gantt::DOC,
    demos::log_stream::DOC,
    demos::sidebar_layout::DOC,
    demos::resizable::DOC,
    demos::dashboard_layout::DOC,
    demos::dock_layout::DOC,
    demos::accordion::DOC,
    demos::collapsible::DOC,
    demos::tabs::DOC,
    demos::wizard::DOC,
    demos::nav_item::DOC,
    demos::nav_section::DOC,
    demos::tree_view::DOC,
    demos::pagination::DOC,
    demos::alert_dialog::DOC,
    demos::menubar::DOC,
    demos::navigation_menu::DOC,
    demos::notification_center::DOC,
    demos::theme_menu::DOC,
    demos::saved_views::DOC,
    demos::command_palette::DOC,
    demos::dialog::DOC,
    demos::menu::DOC,
    demos::popover::DOC,
    demos::tooltip::DOC,
    demos::sheet::DOC,
    demos::hover_card::DOC,
    demos::context_menu::DOC,
    demos::chat_bubble::DOC,
    demos::voice_session::DOC,
    demos::composer::DOC,
    demos::thread::DOC,
    demos::thread_list::DOC,
    demos::assistant_composer::DOC,
    demos::assistant_mcp::DOC,
    demos::assistant_modal::DOC,
    demos::assistant_thread::DOC,
    demos::assistant_thread_list::DOC,
    demos::message_scroller::DOC,
    demos::insight_note::DOC,
    demos::activity::DOC,
    demos::comment_thread::DOC,
    demos::timeline::DOC,
];

/// Groups in sidebar order, each with its pages.
pub(crate) fn groups() -> Vec<(&'static str, Vec<&'static Doc>)> {
    let mut out: Vec<(&str, Vec<&Doc>)> = Vec::new();
    // Groups keep first-seen order, so a page listed out of place still joins its group.
    for doc in DOCS {
        match out.iter_mut().find(|(group, _)| *group == doc.group) {
            Some((_, docs)) => docs.push(doc),
            None => out.push((doc.group, vec![doc])),
        }
    }
    out
}

/// Sets the theme before first paint, from the saved choice or the system setting,
/// so dark-mode readers never see a light flash. Loads the extra themes' fonts only
/// once a theme that uses them is picked.
const THEME_SCRIPT: &str = "try{var r=document.documentElement,t=localStorage.getItem('ox-theme')||(matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light');r.dataset.theme=t;var f=function(){if(/^(neobrutalist|mecha)/.test(r.dataset.theme)&&!document.getElementById('ox-theme-fonts')){var l=document.createElement('link');l.id='ox-theme-fonts';l.rel='stylesheet';l.href='https://fonts.googleapis.com/css2?family=Chakra+Petch:wght@500;600&family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=Space+Grotesk:wght@400..700&display=swap';document.head.appendChild(l)}};f();new MutationObserver(f).observe(r,{attributeFilter:['data-theme']})}catch(e){}";

/// Self-hosted fonts (all SIL OFL), subset to the characters the site uses plus Latin-1;
/// Unbounded is pinned to the one weight it's used at (500). Files live in `public/fonts`;
/// a changed file gets a new name, since `/fonts` is cached for a year. Body text uses `swap`. Headings (Unbounded) and code (`JetBrains Mono`)
/// use `optional`: they're preloaded or cached so they're normally there for first paint,
/// and if one isn't, that page view keeps the fallback rather than re-laying out large
/// text, which would push LCP back and shift the layout.
const FONT_FACES: &str = "\
@font-face{font-family:'Unbounded';font-style:normal;font-weight:400 600;font-display:optional;src:url(/fonts/unbounded-500.woff2) format('woff2')}\
@font-face{font-family:'Hanken Grotesk';font-style:normal;font-weight:400 600;font-display:swap;src:url(/fonts/hanken-grotesk-subset.woff2) format('woff2')}\
@font-face{font-family:'JetBrains Mono';font-style:normal;font-weight:400 500;font-display:optional;src:url(/fonts/jetbrains-mono-subset.woff2) format('woff2')}";

/// All page CSS, inlined so first paint needs no extra request: fonts, tokens and
/// components, then the site's own layout.
fn page_css() -> String {
    [FONT_FACES, oxide_tokens::CSS, include_str!("docs.css")].concat()
}

/// Leptos's `HydrationScripts`, minus its `<link rel="preload">` tags and with a later
/// start. The page is complete HTML already, so the interactive bundle (the wasm is
/// about 1 MB compressed) waits until the page has loaded and painted. It then never
/// competes with the HTML, fonts or first paint; demos become interactive a moment later.
#[component]
fn DeferredHydration(options: LeptosOptions) -> impl IntoView {
    let mut js = options.output_name.to_string();
    let mut wasm = options.output_name.to_string();
    if options.hash_files {
        // Same lookup as Leptos: `hash.txt` (lines `js: …`, `wasm: …`) next to the server binary.
        let hashes = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join(options.hash_file.as_ref())))
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();
        for (kind, hash) in hashes.lines().filter_map(|l| l.split_once(':')) {
            match kind.trim() {
                "js" => {
                    js.push('.');
                    js.push_str(hash.trim());
                }
                "wasm" => {
                    wasm.push('.');
                    wasm.push_str(hash.trim());
                }
                _ => {}
            }
        }
    } else if option_env!("LEPTOS_OUTPUT_NAME").is_none() {
        // Same rule as Leptos: cargo-leptos names the file without `_bg`.
        wasm.push_str("_bg");
    }
    let pkg = options.site_pkg_dir.to_string();
    let script = format!(
        "(function(pkg,js,wasm){{\
function go(){{import('/'+pkg+'/'+js+'.js').then(function(m){{return m.default({{module_or_path:'/'+pkg+'/'+wasm+'.wasm'}}).then(function(){{m.hydrate()}})}})}}\
function start(){{if('requestIdleCallback' in window){{requestIdleCallback(go,{{timeout:2000}})}}else{{setTimeout(go,200)}}}}\
if(document.readyState==='complete'){{start()}}else{{addEventListener('load',start,{{once:true}})}}\
}})({pkg:?},{js:?},{wasm:?});"
    );
    view! { <script type="module" inner_html=script></script> }
}

/// The HTML document around the app. Rendered on the server for every request.
#[must_use]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="description" content="Accessible UI components for Leptos and Dioxus, built on tested state machines." />
                <link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Ccircle cx='16' cy='16' r='11' fill='none' stroke='%238772dd' stroke-width='6'/%3E%3C/svg%3E" />
                <script inner_html=THEME_SCRIPT></script>
                // The two fonts above the fold: headings and body text.
                <link rel="preload" href="/fonts/unbounded-500.woff2" r#as="font" r#type="font/woff2" crossorigin="" />
                <link rel="preload" href="/fonts/hanken-grotesk-subset.woff2" r#as="font" r#type="font/woff2" crossorigin="" />
                // inner_html: the CSS contains `>` selectors, which text escaping would break.
                <style inner_html=page_css()></style>
                <AutoReload options=options.clone() />
                <DeferredHydration options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

/// The app: routes and the chrome shared by every page.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_id_scope();
    let search = RwSignal::new(false);

    view! {
        <Title formatter=|t: String| if t.is_empty() { "oxide".to_string() } else { format!("{t} · oxide") } />
        <Router>
            <Header search />
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=Home />
                <ParentRoute path=path!("/docs") view=DocsLayout>
                    <Route path=path!("getting-started") view=GettingStarted />
                </ParentRoute>
                <ParentRoute path=path!("/components") view=DocsLayout>
                    <Route path=path!("") view=ComponentIndex />
                    <Route path=path!(":slug") view=ComponentPage />
                </ParentRoute>
            </Routes>
            <Search open=search />
        </Router>
    }
}

/// Is `href` the current page, or (for section links) a parent of it?
fn is_current(path: &str, href: &str, exact: bool) -> bool {
    if exact { path.trim_end_matches('/') == href } else { path.starts_with(href) }
}

#[component]
fn Header(search: RwSignal<bool>) -> impl IntoView {
    let location = use_location();
    let section = move |href: &'static str| move || is_current(&location.pathname.get(), href, false).then_some("page");
    // ⌘K / Ctrl+K opens search from anywhere. No-op on the server.
    let shortcut = window_event_listener(leptos::ev::keydown, move |ev| {
        if (ev.meta_key() || ev.ctrl_key()) && ev.key().eq_ignore_ascii_case("k") {
            ev.prevent_default();
            search.update(|open| *open = !*open);
        }
    });
    on_cleanup(move || shortcut.remove());

    view! {
        <header class="topbar">
            <a class="brand" href="/">"oxide"</a>
            <nav class="topnav" aria-label="Main">
                <a href="/docs/getting-started" aria-current=section("/docs")>"Docs"</a>
                <a href="/components" aria-current=section("/components")>"Components"</a>
            </nav>
            <button class="search-trigger" type="button" on:click=move |_| search.set(true)>
                <Icon name=IconName::Search size=15 />
                <span class="search-label">"Search components"</span>
                <kbd aria-hidden="true">"⌘K"</kbd>
            </button>
            <ThemeMenu />
        </header>
    }
}

/// Site-wide search over every page, using oxide's own command palette.
#[component]
fn Search(open: RwSignal<bool>) -> impl IntoView {
    let navigate = use_navigate();
    let items = DOCS
        .iter()
        .map(|d| CommandItem::new(format!("/components/{}", d.slug), d.name, d.group).keywords(d.summary))
        .chain([
            CommandItem::new("/docs/getting-started", "Getting started", "Guides").icon(IconName::FileText),
            CommandItem::new("/components", "All components", "Guides").icon(IconName::Inbox),
        ])
        .collect::<Vec<_>>();
    view! {
        <CommandPalette
            items
            open=open
            on_open_change=move |o| open.set(o)
            on_select=move |href: String| navigate(&href, NavigateOptions::default())
            placeholder="Search components and guides…"
            label="Search the docs"
        />
    }
}

/// Guides and component pages share the sidebar layout.
#[component]
fn DocsLayout() -> impl IntoView {
    view! {
        <div class="docs">
            <aside class="sidebar" aria-label="Documentation">
                <Sidebar />
            </aside>
            <main class="docs-main" id="main">
                <Outlet />
            </main>
        </div>
    }
}

#[component]
fn Sidebar() -> impl IntoView {
    let location = use_location();
    let link = move |href: String, name: &'static str| {
        let current = {
            let href = href.clone();
            move || is_current(&location.pathname.get(), &href, true).then_some("page")
        };
        view! { <a href=href aria-current=current>{name}</a> }
    };
    view! {
        <nav class="side-nav">
            <div class="side-group">
                <p class="label">"Overview"</p>
                {link("/docs/getting-started".into(), "Getting started")}
                {link("/components".into(), "All components")}
            </div>
            {groups()
                .into_iter()
                .map(|(group, docs)| {
                    view! {
                        <div class="side-group">
                            <p class="label">{group}</p>
                            {docs.into_iter().map(|d| link(format!("/components/{}", d.slug), d.name)).collect_view()}
                        </div>
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// `/components`: every component as a card, by group.
#[component]
fn ComponentIndex() -> impl IntoView {
    view! {
        <Title text="Components" />
        <div class="page page-wide">
            <h1>"Components"</h1>
            <p class="lede">
                {format!("{} components, each with a live example, Leptos and Dioxus code, anatomy, keyboard support and props.", DOCS.len())}
            </p>
            {groups()
                .into_iter()
                .map(|(group, docs)| {
                    view! {
                        <section class="index-group">
                            <h2>{group}</h2>
                            <div class="index-grid">
                                {docs
                                    .into_iter()
                                    .map(|d| {
                                        view! {
                                            <a class="index-card" href=format!("/components/{}", d.slug)>
                                                <span class="index-name">{d.name}</span>
                                                <span class="index-summary">{d.summary}</span>
                                            </a>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </section>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// `/components/:slug`.
#[component]
fn ComponentPage() -> impl IntoView {
    let params = use_params_map();
    let doc = move || params.with(|p| p.get("slug")).and_then(|slug| DOCS.iter().find(|d| d.slug == slug));
    move || match doc() {
        Some(doc) => view! { <ComponentDoc doc /> }.into_any(),
        None => view! { <NotFound /> }.into_any(),
    }
}

#[component]
fn ComponentDoc(doc: &'static Doc) -> impl IntoView {
    let sections: Vec<(&str, &str)> = [
        Some(("example", "Example")),
        Some(("usage", "Usage")),
        (!doc.parts.is_empty()).then_some(("anatomy", "Anatomy")),
        (!doc.keys.is_empty()).then_some(("keyboard", "Keyboard")),
        (!doc.props.is_empty()).then_some(("props", "Props")),
    ]
    .into_iter()
    .flatten()
    .collect();

    view! {
        <Title text=doc.name />
        <div class="doc-grid">
            <article class="page">
                <h1>{doc.name}</h1>
                <p class="lede">{doc.summary}</p>

                <section class="demo" id="example" aria-label="Example">{(doc.demo)()}</section>

                <h2 id="usage">"Usage"</h2>
                <Tabs tabs=vec![TabsItem::new("leptos"), TabsItem::new("dioxus")] default_value="leptos">
                    <TabsList>
                        <TabsTrigger value="leptos">"Leptos"</TabsTrigger>
                        <TabsTrigger value="dioxus">"Dioxus"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="leptos"><pre class="code"><code>{doc.leptos_code()}</code></pre></TabsContent>
                    <TabsContent value="dioxus"><pre class="code"><code>{doc.dioxus}</code></pre></TabsContent>
                </Tabs>

                <Section id="anatomy" title="Anatomy" head=&["Component", "data-part", "Renders"] rows=doc.parts />
                <Section id="keyboard" title="Keyboard" head=&["Key", "Does"] rows=doc.keys />
                <Section id="props" title="Props" head=&["Prop", "Type", "Notes"] rows=doc.props />

                <PrevNext slug=doc.slug />
            </article>
            <nav class="toc" aria-label="On this page">
                <p class="label">"On this page"</p>
                {sections.into_iter().map(|(id, name)| view! { <a href=format!("#{id}")>{name}</a> }).collect_view()}
            </nav>
        </div>
    }
}

/// Links to the previous and next component in sidebar order.
#[component]
fn PrevNext(slug: &'static str) -> impl IntoView {
    let i = DOCS.iter().position(|d| d.slug == slug).unwrap_or(0);
    let prev = i.checked_sub(1).map(|j| &DOCS[j]);
    let next = DOCS.get(i + 1);
    let card = |d: &'static Doc, dir: &'static str, class: &'static str| {
        view! {
            <a class=format!("pager-link {class}") href=format!("/components/{}", d.slug)>
                <span class="label">{dir}</span>
                <span class="pager-name">{d.name}</span>
            </a>
        }
    };
    view! {
        <nav class="pager" aria-label="More components">
            {prev.map(|d| card(d, "Previous", "prev"))}
            {next.map(|d| card(d, "Next", "next"))}
        </nav>
    }
}

/// A titled table with an anchor, left out entirely when it has no rows.
#[component]
fn Section(
    id: &'static str,
    title: &'static str,
    head: &'static [&'static str],
    rows: &'static [&'static [&'static str]],
) -> impl IntoView {
    (!rows.is_empty()).then(|| {
        view! {
            <h2 id=id>{title}</h2>
            <div class="table-wrap">
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
            </div>
        }
    })
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(resp) = use_context::<leptos_axum::ResponseOptions>() {
            resp.set_status(axum::http::StatusCode::NOT_FOUND);
        }
    }
    view! {
        <Title text="Not found" />
        <div class="page not-found">
            <h1>"No page here"</h1>
            <p class="lede">"The link may be old. Search finds every component and guide."</p>
            <p><a class="ox-button" data-variant="secondary" data-size="md" href="/components">"Browse components"</a></p>
        </div>
    }
}
