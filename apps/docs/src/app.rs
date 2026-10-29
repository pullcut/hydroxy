//! The document shell, routes, and the docs chrome: header, sidebar, component pages.

use hydroxy_leptos::command_palette::{CommandItem, CommandPalette};
use hydroxy_leptos::icon::{Icon, IconName};
use hydroxy_leptos::id::provide_id_scope;
use hydroxy_leptos::tabs::*;
use hydroxy_leptos::theme_menu::ThemeMenu;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use leptos_router::NavigateOptions;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::{use_location, use_navigate, use_params_map};
use leptos_router::path;

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;

use crate::doc::{CHART_GROUPS, Doc, Section};
use crate::guide::GettingStarted;
use crate::home::Home;

pub(crate) use crate::demos::DOCS;

/// Groups in sidebar order, each with its pages.
pub(crate) fn groups(section: Section) -> Vec<(&'static str, Vec<&'static Doc>)> {
    let mut out: Vec<(&str, Vec<&Doc>)> = Vec::new();
    // Groups keep first-seen order, so a page listed out of place still joins its group.
    for doc in DOCS.iter().filter(|d| d.section() == section) {
        match out.iter_mut().find(|(group, _)| *group == doc.group) {
            Some((_, docs)) => docs.push(doc),
            None => out.push((doc.group, vec![doc])),
        }
    }
    // Chart families follow their declared order.
    if section == Section::Charts {
        out.sort_by_key(|(group, _)| CHART_GROUPS.iter().position(|g| g == group));
    }
    out
}

/// Pages in sidebar order, for previous / next links.
fn ordered(section: Section) -> Vec<&'static Doc> {
    groups(section).into_iter().flat_map(|(_, docs)| docs).collect()
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
    [FONT_FACES, hydroxy_tokens::CSS, include_str!("docs.css")].concat()
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
        // Same lookup as Leptos: `hash.txt` (lines `js: …`, `wasm: …`) next to the server
        // binary. It can't change while the server runs, so read it once.
        static HASHES: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let hashes = HASHES.get_or_init(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|dir| dir.join(options.hash_file.as_ref())))
                .and_then(|path| std::fs::read_to_string(path).ok())
                .unwrap_or_default()
        });
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
        <Title formatter=|t: String| if t.is_empty() { "hydroxy".to_string() } else { format!("{t} · hydroxy") } />
        <Router>
            <Header search />
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=Home />
                <ParentRoute path=path!("/docs") view=DocsLayout>
                    <Route path=path!("getting-started") view=GettingStarted />
                </ParentRoute>
                <ParentRoute path=path!("/components") view=DocsLayout>
                    <Route path=path!("") view=|| view! { <ComponentIndex section=Section::Components /> } />
                    <Route path=path!(":slug") view=ComponentPage />
                </ParentRoute>
                <ParentRoute path=path!("/charts") view=DocsLayout>
                    <Route path=path!("") view=|| view! { <ComponentIndex section=Section::Charts /> } />
                    <Route path=path!(":slug") view=ComponentPage />
                </ParentRoute>
            </Routes>
            <Search open=search />
        </Router>
    }
}

/// The catalogue half a path belongs to. A page's own section wins, so an old
/// `/components/<chart>` link still shows the chart sidebar.
fn section_of(path: &str) -> Section {
    let slug = path.trim_end_matches('/').rsplit('/').next().unwrap_or_default();
    DOCS.iter().find(|d| d.slug == slug).map_or_else(|| Section::of_path(path), Doc::section)
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
    hydroxy_hotkeys::leptos::use_hotkey_with(
        "Mod+K",
        hydroxy_hotkeys::HotkeyOptions::new().description("Search components").group("Site").in_inputs(true),
        move || search.update(|open| *open = !*open),
    );

    view! {
        <header class="topbar">
            <a class="brand" href="/">"hydroxy"</a>
            <nav class="topnav" aria-label="Main">
                <a href="/docs/getting-started" aria-current=section("/docs")>"Docs"</a>
                <a href="/components" aria-current=section("/components")>"Components"</a>
                <a href="/charts" aria-current=section("/charts")>"Charts"</a>
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

/// Site-wide search over every page, using hydroxy's own command palette.
#[component]
fn Search(open: RwSignal<bool>) -> impl IntoView {
    let navigate = use_navigate();
    let items = DOCS
        .iter()
        .map(|d| CommandItem::new(d.href(), d.name, d.group).keywords(d.summary))
        .chain([
            CommandItem::new("/docs/getting-started", "Getting started", "Guides").icon(IconName::FileText),
            CommandItem::new("/components", "All components", "Guides").icon(IconName::Inbox),
            CommandItem::new("/charts", "All charts", "Guides").icon(IconName::Inbox),
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
    let section = Memo::new(move |_| section_of(&location.pathname.get()));
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
                {move || match section.get() {
                    Section::Components => link("/components".into(), "All components"),
                    Section::Charts => link("/charts".into(), "All charts"),
                }}
            </div>
            {move || groups(section.get())
                .into_iter()
                .map(|(group, docs)| {
                    view! {
                        <div class="side-group">
                            <p class="label">{group}</p>
                            {docs.into_iter().map(|d| link(d.href(), d.name)).collect_view()}
                        </div>
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// `/components` and `/charts`: every page as a card with a preview, by group.
#[component]
fn ComponentIndex(section: Section) -> impl IntoView {
    let (title, lede) = match section {
        Section::Components => {
            ("Components", "each with a live example, Leptos and Dioxus code, anatomy, keyboard support and props.")
        }
        Section::Charts => {
            ("Charts", "animated, accessible and fast with large data, each with Leptos and Dioxus code.")
        }
    };
    let groups = groups(section);
    let count: usize = groups.iter().map(|(_, docs)| docs.len()).sum();
    view! {
        <Title text=title />
        <div class="page page-wide">
            <h1>{title}</h1>
            <p class="lede">{format!("{count} {}, {lede}", title.to_lowercase())}</p>
            {groups
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
                                            <a class="index-card" href=d.href()>
                                                <Preview doc=d />
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

type Watch = (web_sys::IntersectionObserver, Closure<dyn FnMut(Vec<web_sys::IntersectionObserverEntry>)>);

/// Where and how much to scale `stage` so what it draws fills `preview`, centred.
fn measure(
    preview: &web_sys::Element,
    stage: &web_sys::HtmlElement,
    only: Option<&web_sys::Element>,
) -> Option<(f64, f64, f64)> {
    let origin = stage.get_bounding_client_rect();
    // The union of everything visible: leaves, plus whole SVGs and canvases.
    let (mut left, mut top, mut right, mut bottom) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    if let Some(el) = only {
        let r = el.get_bounding_client_rect();
        (left, top, right, bottom) = (r.left(), r.top(), r.right(), r.bottom());
    }
    let all = stage.query_selector_all("*:not(svg *)").ok()?;
    for i in (0..all.length()).take_while(|_| only.is_none()) {
        let Some(el) = all.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) else { continue };
        let tag = el.tag_name().to_ascii_lowercase();
        let leaf = el.child_element_count() == 0
            || matches!(tag.as_str(), "svg" | "canvas" | "img" | "input" | "textarea" | "button");
        // Leaves count whole; in a container, only its own text runs count, measured exactly.
        let mut rects = Vec::new();
        if leaf {
            rects.push(el.get_bounding_client_rect());
        } else {
            let kids = el.child_nodes();
            for k in 0..kids.length() {
                let Some(text) = kids.item(k).filter(|n| n.node_type() == web_sys::Node::TEXT_NODE) else { continue };
                if text.text_content().is_none_or(|t| t.trim().is_empty()) {
                    continue;
                }
                if let Ok(range) = document().create_range()
                    && range.select_node_contents(&text).is_ok()
                {
                    rects.push(range.get_bounding_client_rect());
                }
            }
        }
        for r in rects {
            // Skip screen-reader-only and empty boxes.
            if r.width() <= 2.0 || r.height() <= 2.0 {
                continue;
            }
            left = left.min(r.left());
            top = top.min(r.top());
            right = right.max(r.right());
            bottom = bottom.max(r.bottom());
        }
    }
    if right <= left || bottom <= top {
        return None;
    }
    let (w, h) = (f64::from(preview.client_width()), f64::from(preview.client_height()));
    let (cw, ch) = (right - left, bottom - top);
    let scale = (w * 0.86 / cw).min(h * 0.8 / ch).min(1.6);
    let x = w / 2.0 - (left - origin.left() + cw / 2.0) * scale;
    let y = h / 2.0 - (top - origin.top() + ch / 2.0) * scale;
    Some((scale, x, y))
}

/// Scales a preview's example to fill the card. Tries a few layout widths, since wide rows
/// wrap into a better shape when narrower, and keeps the one that draws largest.
fn fit(preview: &web_sys::Element) {
    let Some(stage) = preview.first_element_child().and_then(|s| s.dyn_into::<web_sys::HtmlElement>().ok()) else {
        return;
    };
    let style = stage.style();
    let _ = style.remove_property("transform");
    // A chart page's card shows its first chart alone. Charts size themselves to their
    // box a frame later, so they keep one width.
    let chart = stage.query_selector("[data-scope='chart-frame'], [data-scope$='-chart']").ok().flatten();
    if let Some(first) = &chart {
        let mut node = first.clone();
        while let Some(parent) = node.parent_element().filter(|p| p != stage.unchecked_ref::<web_sys::Element>()) {
            let mut next = node.next_element_sibling();
            while let Some(sibling) = next {
                if let Some(el) = sibling.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = el.style().set_property("display", "none");
                }
                next = sibling.next_element_sibling();
            }
            node = parent;
        }
    }
    let widths: &[&str] = if chart.is_some() { &["640px"] } else { &["360px", "480px", "640px"] };
    let mut best: Option<(&str, (f64, f64, f64))> = None;
    for &width in widths {
        let _ = style.set_property("width", width);
        if let Some(m) = measure(preview, &stage, chart.as_ref())
            && best.is_none_or(|(_, b)| m.0 > b.0)
        {
            best = Some((width, m));
        }
    }
    let Some((width, (scale, x, y))) = best else { return };
    let _ = style.set_property("width", width);
    let _ = style.set_property("transform", &format!("translate({x}px, {y}px) scale({scale})"));
    let _ = stage.set_attribute("data-fit", "");
}

/// A card's live example: mounted when the card nears the viewport, then kept. Inert, so
/// the card stays one link, and frozen at its end state by the CSS.
#[component]
fn Preview(doc: &'static Doc) -> impl IntoView {
    let el = NodeRef::<leptos::html::Span>::new();
    let shown = RwSignal::new(false);
    // Effects only run in the browser, so the observer never exists on the server.
    Effect::new(move |_| {
        let Some(node) = el.get() else { return };
        let watch = StoredValue::new_local(None::<Watch>);
        let node_fit: web_sys::Element = (*node).clone().into();
        on_cleanup(move || {
            if let Some((observer, _)) = watch.try_update_value(Option::take).flatten() {
                observer.disconnect();
            }
        });
        let on_hit = Closure::<dyn FnMut(Vec<web_sys::IntersectionObserverEntry>)>::new(
            move |entries: Vec<web_sys::IntersectionObserverEntry>| {
                if entries.iter().any(web_sys::IntersectionObserverEntry::is_intersecting) {
                    shown.try_set(true);
                    // Fit once it has painted, and again once charts have sized themselves.
                    let node = node_fit.clone();
                    request_animation_frame(move || fit(&node));
                    let node = node_fit.clone();
                    set_timeout(move || fit(&node), std::time::Duration::from_millis(400));
                    if let Some((observer, _)) = watch.try_update_value(Option::take).flatten() {
                        observer.disconnect();
                    }
                }
            },
        );
        let options = web_sys::IntersectionObserverInit::new();
        options.set_root_margin("600px 0px");
        match web_sys::IntersectionObserver::new_with_options(on_hit.as_ref().unchecked_ref(), &options) {
            Ok(observer) => {
                observer.observe(&node);
                watch.set_value(Some((observer, on_hit)));
            }
            Err(_) => shown.set(true),
        }
    });
    view! {
        <span class="index-preview" aria-hidden="true" node_ref=el>
            <span class="index-stage" inert>
                {move || shown.get().then(|| (doc.demo)())}
            </span>
        </span>
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
    let events = doc.events;
    let sections: Vec<(&str, &str)> = [
        Some(("example", "Example")),
        Some(("usage", "Usage")),
        (!doc.parts.is_empty()).then_some(("anatomy", "Anatomy")),
        (!doc.keys.is_empty()).then_some(("keyboard", "Keyboard")),
        (!doc.props.is_empty()).then_some(("props", "Props")),
        (!events.is_empty()).then_some(("events", "Events")),
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
                <Section id="events" title="Events" head=&["Prop", "Type", "Notes"] rows=events />

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
    let section = DOCS.iter().find(|d| d.slug == slug).map_or(Section::Components, Doc::section);
    let order = ordered(section);
    let i = order.iter().position(|d| d.slug == slug).unwrap_or(0);
    let prev = i.checked_sub(1).map(|j| order[j]);
    let next = order.get(i + 1).copied();
    let card = |d: &'static Doc, dir: &'static str, class: &'static str| {
        view! {
            <a class=format!("pager-link {class}") href=d.href()>
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
