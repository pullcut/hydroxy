//! What search engines and language models read: per-page head tags, JSON-LD, and the
//! plain files (robots.txt, sitemap.xml, llms.txt, per-page markdown).

use std::fmt::Write as _;

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Script, Title};

use crate::demos::DOCS;
use crate::doc::{Doc, Section};

/// Where the site lives. Change when a custom domain is set.
pub(crate) const SITE_URL: &str = "https://hydroxy.pages.dev";

const REPO: &str = "https://github.com/pullcut/hydroxy";
const SUMMARY: &str = "Accessible, headless UI components and charts for Leptos and Dioxus, written in Rust.";

/// One indexable page: its path and what its head says about it.
pub(crate) struct Page {
    pub path: String,
    pub title: String,
    pub description: String,
    pub doc: Option<&'static Doc>,
}

impl Page {
    pub(crate) fn home() -> Self {
        Self {
            path: "/".into(),
            title: "hydroxy — Accessible UI components for Leptos and Dioxus".into(),
            description: "Accessible, headless UI components and charts for Leptos and Dioxus in Rust. \
                Tested state machines, ARIA and keyboard support, one set of design tokens."
                .into(),
            doc: None,
        }
    }

    pub(crate) fn guide() -> Self {
        Self {
            path: "/docs/getting-started".into(),
            title: "Getting started · hydroxy".into(),
            description: "Install hydroxy, add the stylesheet and render your first accessible component in Leptos or Dioxus, with server rendering and theming."
                .into(),
            doc: None,
        }
    }

    pub(crate) fn index(section: Section) -> Self {
        let (title, description) = match section {
            Section::Components => (
                "Components — hydroxy",
                "Every hydroxy component for Leptos and Dioxus: accessible, headless-first Rust UI \
                 with live examples, code for both frameworks, keyboard support and props.",
            ),
            Section::Charts => (
                "Charts — hydroxy",
                "Every hydroxy chart for Leptos and Dioxus: animated, accessible Rust charts that stay \
                 fast with large data, each with a live example and code for both frameworks.",
            ),
        };
        Self { path: section.href().into(), title: title.into(), description: description.into(), doc: None }
    }

    pub(crate) fn doc(doc: &'static Doc) -> Self {
        let (long, short, kind) = match doc.section() {
            Section::Components => {
                ("Leptos & Dioxus component · hydroxy", "Leptos & Dioxus · hydroxy", "Accessible Rust component")
            }
            Section::Charts => {
                ("Rust chart for Leptos & Dioxus · hydroxy", "Leptos & Dioxus chart · hydroxy", "Accessible Rust chart")
            }
        };
        let mut title = format!("{} — {long}", doc.name);
        if title.chars().count() > 65 {
            title = format!("{} — {short}", doc.name);
        }
        let description = clip(&format!("{} {kind} for Leptos and Dioxus.", doc.summary), 155);
        Self { path: doc.href(), title, description, doc: Some(doc) }
    }

    /// Every page worth indexing, in sitemap order.
    #[cfg(any(feature = "ssr", test))]
    pub(crate) fn all() -> Vec<Self> {
        [Self::home(), Self::guide(), Self::index(Section::Components), Self::index(Section::Charts)]
            .into_iter()
            .chain(DOCS.iter().map(Self::doc))
            .collect()
    }

    pub(crate) fn url(&self) -> String {
        if self.path == "/" { SITE_URL.to_string() } else { format!("{SITE_URL}{}", self.path) }
    }

    /// Structured data: the project on the home page, an article and breadcrumbs on a doc.
    fn json_ld(&self) -> Option<String> {
        if self.path == "/" {
            return Some(format!(
                r#"{{"@context":"https://schema.org","@graph":[{{"@type":"WebSite","name":"hydroxy","url":{url}}},{{"@type":"SoftwareSourceCode","name":"hydroxy","description":{desc},"url":{url},"codeRepository":{repo},"programmingLanguage":"Rust","license":"https://opensource.org/licenses/MIT","keywords":"Rust, Leptos, Dioxus, UI components, accessibility, headless, WebAssembly, charts"}}]}}"#,
                url = json(SITE_URL),
                desc = json(SUMMARY),
                repo = json(REPO),
            ));
        }
        let doc = self.doc?;
        let section = doc.section();
        let crumb = if section == Section::Charts { "Charts" } else { "Components" };
        Some(format!(
            r#"{{"@context":"https://schema.org","@graph":[{{"@type":"TechArticle","headline":{name},"description":{desc},"url":{url},"about":{{"@type":"Thing","name":{name}}},"isPartOf":{{"@type":"WebSite","name":"hydroxy","url":{site}}}}},{{"@type":"BreadcrumbList","itemListElement":[{{"@type":"ListItem","position":1,"name":"Home","item":{site}}},{{"@type":"ListItem","position":2,"name":{crumb},"item":{crumb_url}}},{{"@type":"ListItem","position":3,"name":{name},"item":{url}}}]}}]}}"#,
            name = json(doc.name),
            desc = json(&self.description),
            url = json(&self.url()),
            site = json(SITE_URL),
            crumb = json(crumb),
            crumb_url = json(&format!("{SITE_URL}{}", section.href())),
        ))
    }
}

/// The page's head tags. Site-wide tags (image, locale, icons) are in the shell.
pub(crate) fn head(page: Page) -> impl IntoView {
    let url = page.url();
    let og_type = if page.path == "/" { "website" } else { "article" };
    let markdown = page.doc.map(|d| format!("{}.md", d.href()));
    let json_ld = page.json_ld();
    let Page { title, description, .. } = page;
    view! {
        <Title text=title.clone() />
        <Meta name="description" content=description.clone() />
        <Link rel="canonical" href=url.clone() />
        <Meta property="og:type" content=og_type />
        <Meta property="og:title" content=title.clone() />
        <Meta property="og:description" content=description.clone() />
        <Meta property="og:url" content=url />
        <Meta name="twitter:title" content=title />
        <Meta name="twitter:description" content=description />
        {markdown.map(|href| view! { <Link rel="alternate" type_="text/markdown" href /> })}
        {json_ld.map(|j| view! { <Script type_="application/ld+json">{j}</Script> })}
    }
}

/// `s` as a JSON string. `<`, `>` and `&` are escaped too, so it can't end the
/// `<script>` it sits in, and HTML text escaping leaves it alone.
fn json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' | '>' | '&' | '\u{0}'..='\u{1f}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// At most `max` characters, cut at a word boundary with an ellipsis.
fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max - 1).collect();
    let cut = head.rfind(' ').unwrap_or(head.len());
    format!("{}…", head[..cut].trim_end_matches([',', ';', ':', '.', ' ']))
}

#[cfg(feature = "ssr")]
pub use files::file;

/// The plain files, generated from the same page list. Server only.
#[cfg(feature = "ssr")]
mod files {
    use std::borrow::Cow;
    use std::fmt::Write as _;

    use super::{Page, SITE_URL, SUMMARY};
    use crate::demos::DOCS;
    use crate::doc::{Doc, Section};
    use crate::guide::{DIOXUS_EXAMPLE, LEPTOS_EXAMPLE, SSR_EXAMPLE, THEME_EXAMPLE};

    /// A generated file for `path`, with its content type, or `None` to fall through to pages.
    pub fn file(path: &str) -> Option<(&'static str, Cow<'static, str>)> {
        const TEXT: &str = "text/plain; charset=utf-8";
        const MARKDOWN: &str = "text/markdown; charset=utf-8";
        let (href, css) = crate::app::stylesheet();
        Some(match path {
            "/robots.txt" => (TEXT, format!("User-agent: *\nAllow: /\n\nSitemap: {SITE_URL}/sitemap.xml\n").into()),
            "/sitemap.xml" => ("application/xml; charset=utf-8", sitemap().into()),
            "/llms.txt" => (TEXT, llms().into()),
            "/llms-full.txt" => (TEXT, llms_full().into()),
            p if p == href => ("text/css; charset=utf-8", Cow::Borrowed(css)),
            p => {
                let page = p.strip_suffix(".md")?;
                let doc = DOCS.iter().find(|d| d.href() == page)?;
                (MARKDOWN, markdown(doc).into())
            }
        })
    }

    fn sitemap() -> String {
        let mut out = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        for page in Page::all() {
            // Paths are slugs and fixed words, so nothing needs XML escaping.
            let _ = writeln!(out, "  <url><loc>{}</loc></url>", page.url());
        }
        out.push_str("</urlset>\n");
        out
    }

    fn llms() -> String {
        let mut out = format!(
            "# hydroxy\n\n> {SUMMARY}\n\n\
             hydroxy is a Rust UI library with the same components for Leptos and Dioxus. Each component's \
             behaviour is a pure, unit-tested state machine; a headless layer turns it into ARIA attributes, \
             focus and keyboard handling; thin adapters render it in either framework, styled by one set of \
             CSS design tokens. Charts are accessible too, with keyboard navigation and text alternatives.\n\n\
             Everything below in one file: [llms-full.txt]({SITE_URL}/llms-full.txt)\n\n## Docs\n\n"
        );
        let guide = Page::guide();
        let _ = writeln!(out, "- [Getting started]({}): {}", guide.url(), guide.description);
        for section in [Section::Components, Section::Charts] {
            let page = Page::index(section);
            let name = if section == Section::Charts { "All charts" } else { "All components" };
            let _ = writeln!(out, "- [{name}]({}): {}", page.url(), page.description);
        }
        for (section, heading) in [(Section::Components, "Components"), (Section::Charts, "Charts")] {
            let _ = write!(out, "\n## {heading}\n\n");
            for (_, docs) in crate::app::groups(section) {
                for d in docs {
                    let _ = writeln!(out, "- [{}]({SITE_URL}{}.md): {}", d.name, d.href(), d.summary);
                }
            }
        }
        out
    }

    fn llms_full() -> String {
        let mut out = format!("# hydroxy\n\n> {SUMMARY}\n\n{}", guide());
        for section in [Section::Components, Section::Charts] {
            for (_, docs) in crate::app::groups(section) {
                for d in docs {
                    out.push_str("\n---\n\n");
                    out.push_str(&markdown(d));
                }
            }
        }
        out
    }

    /// The getting started page as markdown.
    fn guide() -> String {
        format!(
            "## Getting started\n\n\
             Source: {SITE_URL}/docs/getting-started\n\n\
             hydroxy is a component library for Leptos and Dioxus. Every component works the same in both, \
             because the behaviour lives below the framework.\n\n\
             ### Install\n\n\
             Leptos: `cargo add hydroxy-leptos hydroxy-tokens`\n\n\
             Dioxus: `cargo add hydroxy-dioxus hydroxy-tokens`\n\n\
             ### Use a component\n\n\
             Inject the stylesheet once at the root, then use components.\n\n\
             ```rust\n{LEPTOS_EXAMPLE}\n```\n\n```rust\n{DIOXUS_EXAMPLE}\n```\n\n\
             ### Server rendering\n\n\
             Components generate ids to link labels, panels and focus targets. With Leptos server rendering, \
             start an id scope in the root component so the server and the browser give every component the same id.\n\n\
             ```rust\n{SSR_EXAMPLE}\n```\n\n\
             ### Theming\n\n\
             Everything is styled by CSS custom properties. Light is the default; dark applies under \
             `data-theme=\"dark\"`. Override any token to restyle every component.\n\n\
             ```css\n{THEME_EXAMPLE}\n```\n\n\
             ### How it's built\n\n\
             1. hydroxy-statechart: the Machine trait.\n\
             2. hydroxy-machines: one pure machine per component, with unit tests.\n\
             3. hydroxy-headless: connectors, giving ARIA attributes, data attributes and key mapping as plain data.\n\
             4. hydroxy-leptos, hydroxy-dioxus: adapters that render the connector and run commands such as focus.\n\
             5. hydroxy-tokens, hydroxy-charts: the stylesheet, and chart scales and geometry.\n"
        )
    }

    /// One component or chart page as markdown: what it is, its code, and its tables.
    fn markdown(doc: &Doc) -> String {
        let kind = if doc.section() == Section::Charts { "Chart" } else { "Component" };
        let mut out = format!(
            "# {}\n\n{}\n\n{kind} in the {} group of hydroxy, for Leptos and Dioxus. Page: {SITE_URL}{}\n\n\
             ## Usage (Leptos)\n\n```rust\n{}\n```\n",
            doc.name,
            doc.summary,
            doc.group,
            doc.href(),
            doc.leptos_code()
        );
        for (_, title, head, rows) in doc.tables() {
            if rows.is_empty() {
                continue;
            }
            let _ = write!(out, "\n## {title}\n\n| {} |\n|{}\n", head.join(" | "), " --- |".repeat(head.len()));
            for row in rows {
                let cells: Vec<String> = row
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let c = c.replace('|', "\\|").replace('\n', " ");
                        if i == 0 { format!("`{c}`") } else { c }
                    })
                    .collect();
                let _ = writeln!(out, "| {} |", cells.join(" | "));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{Page, clip, json};

    #[test]
    fn pages_have_unique_titles_descriptions_and_urls() {
        let pages = Page::all();
        let (mut titles, mut descriptions, mut urls) = (HashSet::new(), HashSet::new(), HashSet::new());
        for p in &pages {
            assert!(p.title.len() > 10, "{} has a short title", p.path);
            assert!((50..=160).contains(&p.description.chars().count()), "{} description length", p.path);
            assert!(titles.insert(&p.title), "duplicate title {}", p.title);
            assert!(descriptions.insert(&p.description), "duplicate description on {}", p.path);
            assert!(urls.insert(p.url()), "duplicate canonical {}", p.url());
            assert!(!p.url().ends_with('/'), "{} canonical ends in a slash", p.path);
        }
    }

    #[test]
    fn json_escapes_script_breakers() {
        assert_eq!(json("a</script>\"&\n"), r#""a\u003c/script\u003e\"\u0026\u000a""#);
    }

    #[test]
    fn clip_cuts_at_a_word() {
        assert_eq!(clip("one two three", 9), "one two…");
        assert_eq!(clip("short", 9), "short");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn files_are_served() {
        let (href, _) = crate::app::stylesheet();
        for path in ["/robots.txt", "/sitemap.xml", "/llms.txt", "/llms-full.txt", href, "/components/button.md"] {
            assert!(super::file(path).is_some(), "{path} not served");
        }
        assert!(super::file("/components/button").is_none());
        assert!(super::file("/charts/button.md").is_none());
    }
}
