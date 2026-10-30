use hydroxy_leptos::diff_viewer::DiffViewer;
use leptos::prelude::*;

const OLD: &str = r"use std::time::Duration;

/// Retry policy for outbound webhook deliveries.
pub struct Retry {
    pub attempts: u32,
    pub delay: Duration,
}

impl Retry {
    pub fn new(attempts: u32) -> Self {
        Self { attempts, delay: Duration::from_millis(500) }
    }

    /// How long to wait before attempt `n`.
    pub fn backoff(&self, n: u32) -> Duration {
        self.delay * n
    }

    pub fn exhausted(&self, n: u32) -> bool {
        n >= self.attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_linearly() {
        let r = Retry::new(3);
        assert_eq!(r.backoff(2), Duration::from_millis(1000));
    }
}
";

const NEW: &str = r"use std::time::Duration;

/// Retry policy for outbound webhook deliveries.
pub struct Retry {
    pub attempts: u32,
    pub delay: Duration,
    pub max_delay: Duration,
}

impl Retry {
    pub fn new(attempts: u32) -> Self {
        Self { attempts, delay: Duration::from_millis(250), max_delay: Duration::from_secs(30) }
    }

    /// How long to wait before attempt `n`: doubles each time, capped.
    pub fn backoff(&self, n: u32) -> Duration {
        (self.delay * 2u32.saturating_pow(n)).min(self.max_delay)
    }

    pub fn exhausted(&self, n: u32) -> bool {
        n >= self.attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_exponentially_up_to_the_cap() {
        let r = Retry::new(8);
        assert_eq!(r.backoff(2), Duration::from_millis(1000));
        assert_eq!(r.backoff(12), Duration::from_secs(30));
    }
}
";

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="max-width: 900px; max-height: 520px; display: flex; flex-direction: column;">
            <DiffViewer before=OLD after=NEW file_name="src/webhooks/retry.rs" lang="rust" />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "diff-viewer",
    name: "Diff viewer",
    group: "Data",
    summary: "Compare two versions of a text or source file. A Myers line diff with changed words marked inside each line, unified or side-by-side, with unchanged runs folded away until you expand them.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("diff_viewer.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/diff_viewer.rs"),
    parts: &[
        &["DiffViewer", "root", "Header over the diff. data-mode: unified or split"],
        &["", "header", "File name, +/− line counts and the layout toggle"],
        &["", "body", "The scrolling region, labelled with the change summary"],
        &["", "table", "A native table with a hidden caption and column headers"],
        &["", "line", "One row. data-kind: added, removed or context (unified)"],
        &["", "num", "Old or new line number, hidden from screen readers"],
        &["", "code", "Marker, a hidden Added/Removed label and the coloured text. data-word on changed words"],
        &["", "gap / expand", "A folded run of unchanged lines and the button that opens it"],
    ],
    keys: &[
        &["Tab", "Move to the layout toggle, the diff region and each expand button"],
        &["Arrow keys", "In the layout toggle: switch layout. In the diff region: scroll"],
        &["Enter / Space", "On an expand button: show the folded lines and focus the first"],
    ],
    props: &[
        &["before / after", "String", "The two texts. Reactive; a change folds everything again"],
        &["file_name", "Option<String>", "Shown in the header and the summary"],
        &["lang", "String", "Syntax colouring, as CodeBlock: rust, ts, json, sql, bash, yaml, toml. Default text"],
        &["context", "usize", "Unchanged lines kept around each change. Default 3"],
        &["mode", "DiffMode", "Unified or Split to start with"],
    ],
    events: &[],
};
