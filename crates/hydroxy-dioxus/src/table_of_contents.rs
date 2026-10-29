//! `TableOfContents`: an in-page outline of real `#hash` links, with an optional scroll spy.
//!
//! The spy runs in the webview through `document::eval`: it reports the heading
//! tops on scroll and resize, and Rust picks the current one.

use dioxus::prelude::*;
use hydroxy_headless::table_of_contents as h;

pub use h::Item;

/// Waits for the ids, then reports `[tops, at_bottom, ids]` on scroll and resize
/// until the next message. Anything but an id list (spy off) just stops.
const SCRIPT: &str = r"
const ids = await dioxus.recv();
if (!Array.isArray(ids)) return;
const post = () => {
  const tops = [], found = [];
  for (const id of ids) {
    const el = document.getElementById(id);
    if (el) { tops.push(el.getBoundingClientRect().top); found.push(id); }
  }
  const root = document.documentElement;
  const atBottom = root.scrollHeight > innerHeight && root.scrollHeight - (scrollY + innerHeight) <= 2;
  dioxus.send([tops, atBottom, found]);
};
addEventListener('scroll', post, { passive: true });
addEventListener('resize', post);
post();
await dioxus.recv();
removeEventListener('scroll', post);
removeEventListener('resize', post);
";

/// A heading outline. Links are plain anchors, so it works without script. The
/// host passes `active_id`, or turns on `spy` and the outline follows the page.
#[component]
pub fn TableOfContents(
    /// Headings in reading order: id, label and depth.
    items: Vec<Item>,
    /// The current heading, owned by the host. Wins over the spy.
    active_id: Option<String>,
    /// Accessible name for the outline. Default "On this page".
    label: Option<String>,
    /// Pixels of sticky-header clearance: the spy's reading line. Default 72.
    #[props(default = h::DEFAULT_HEADING_OFFSET)]
    heading_offset: f64,
    /// Follow the page's scroll position to find the current heading.
    #[props(default)]
    spy: bool,
    /// Called with the heading id when a link is followed.
    on_navigate: Option<EventHandler<String>>,
) -> Element {
    let mut spied = use_signal(|| None::<String>);
    let ids: Vec<String> = h::resolve(&items, None).0.into_iter().map(|i| i.id).collect();
    let eval = use_hook(|| document::eval(SCRIPT));
    use_future(move || {
        let ids = ids.clone();
        async move {
            if !spy {
                return;
            }
            let mut eval = eval;
            let _ = eval.send(ids);
            while let Ok((tops, at_bottom, found)) = eval.recv::<(Vec<f64>, bool, Vec<String>)>().await {
                spied.set(h::pick_active(&tops, heading_offset, at_bottom).map(|i| found[i].clone()));
            }
        }
    });
    // Ends the script (spy off) or removes its listeners.
    use_drop(move || {
        let _ = eval.send("stop");
    });

    let current = active_id.or_else(|| spied.cloned());
    let (resolved, _) = h::resolve(&items, current.as_deref());
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    rsx! {
        nav { "data-scope": h::SCOPE, "data-part": "root", "aria-label": label, style: h::root_style(heading_offset),
            ol { "data-scope": h::SCOPE, "data-part": "list",
                for item in resolved {
                    li {
                        key: "{item.id}",
                        "data-scope": h::SCOPE,
                        "data-part": "item",
                        "data-level": item.level.to_string(),
                        style: format!("--ox-toc-level: {}", item.level),
                        a {
                            "data-scope": h::SCOPE,
                            "data-part": "link",
                            "data-table-of-contents-item": item.id.clone(),
                            "data-active": item.active.then_some(""),
                            "aria-current": h::aria_current(item.active),
                            href: item.href.clone(),
                            onclick: {
                                let id = item.id.clone();
                                move |_| {
                                    if let Some(handler) = on_navigate {
                                        handler.call(id.clone());
                                    }
                                }
                            },
                            "{item.label}"
                        }
                    }
                }
            }
        }
    }
}
