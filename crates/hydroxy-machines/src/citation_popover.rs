//! Citations: numbered markers in text that preview their source on hover or
//! focus. This module turns text plus cited spans into runs to render, and pulls
//! the display bits (host, quote highlight) out of a source. Open and close timing
//! is the hover card's (see [`crate::hover_card`]).

/// A source a citation points to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The page or document title.
    pub title: String,
    /// Where it lives.
    pub url: String,
    /// A passage from the source.
    pub snippet: String,
    /// The part of `snippet` the claim rests on; highlighted when found.
    pub quote: Option<String>,
    /// Icon URL. Without one, the host's first letter stands in.
    pub favicon: Option<String>,
}

impl Source {
    /// A source from its title, URL and snippet.
    #[must_use]
    pub fn new(title: impl Into<String>, url: impl Into<String>, snippet: impl Into<String>) -> Self {
        Self { title: title.into(), url: url.into(), snippet: snippet.into(), quote: None, favicon: None }
    }

    /// The same source with the quoted part of the snippet to highlight.
    #[must_use]
    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    /// The same source with an icon.
    #[must_use]
    pub fn favicon(mut self, url: impl Into<String>) -> Self {
        self.favicon = Some(url.into());
        self
    }
}

/// A cited stretch of text: byte offsets into the text and the sources (indexes
/// into the source list) that back it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitedSpan {
    /// Start byte offset.
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
    /// Indexes into the source list.
    pub sources: Vec<usize>,
}

impl CitedSpan {
    /// A span from `start` to `end` backed by `sources`.
    #[must_use]
    pub fn new(start: usize, end: usize, sources: &[usize]) -> Self {
        Self { start, end, sources: sources.to_vec() }
    }

    /// The span covering the first `needle` in `text`, if it is there. Handy for hosts
    /// that know the cited sentence but not its offsets.
    #[must_use]
    pub fn find(text: &str, needle: &str, sources: &[usize]) -> Option<Self> {
        let start = text.find(needle)?;
        Some(Self::new(start, start + needle.len(), sources))
    }
}

/// A run of text to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Run {
    /// Plain text.
    Text(String),
    /// Cited text, followed by a marker per source.
    Cited {
        /// The cited text.
        text: String,
        /// Indexes into the source list.
        sources: Vec<usize>,
    },
}

/// Split `text` into plain and cited runs. Spans are sorted; ones that overlap an
/// earlier span, fall outside the text or cut a character in half are dropped,
/// as are source indexes not below `source_count`.
#[must_use]
pub fn runs(text: &str, spans: &[CitedSpan], source_count: usize) -> Vec<Run> {
    let mut spans: Vec<&CitedSpan> = spans.iter().collect();
    spans.sort_by_key(|s| (s.start, s.end));
    let mut out = Vec::new();
    let mut at = 0;
    for span in spans {
        let valid = span.start >= at
            && span.start < span.end
            && span.end <= text.len()
            && text.is_char_boundary(span.start)
            && text.is_char_boundary(span.end);
        let sources: Vec<usize> = span.sources.iter().copied().filter(|&i| i < source_count).collect();
        if !valid || sources.is_empty() {
            continue;
        }
        if span.start > at {
            out.push(Run::Text(text[at..span.start].to_string()));
        }
        out.push(Run::Cited { text: text[span.start..span.end].to_string(), sources });
        at = span.end;
    }
    if at < text.len() {
        out.push(Run::Text(text[at..].to_string()));
    }
    out
}

/// The host part of a URL without "www.": `https://www.rust-lang.org/learn` → `rust-lang.org`.
#[must_use]
pub fn host(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    let host = host.split(':').next().unwrap_or(host);
    host.strip_prefix("www.").unwrap_or(host).to_lowercase()
}

/// The letter standing in for a favicon: the host's first letter, upper case.
#[must_use]
pub fn initial(url: &str) -> String {
    host(url).chars().find(|c| c.is_alphanumeric()).map_or_else(|| "?".into(), |c| c.to_uppercase().collect())
}

/// The snippet split around the quote: `(before, quote, after)`. The quote is
/// matched ignoring case; when missing, the whole snippet is `before`.
#[must_use]
pub fn highlight(source: &Source) -> (String, String, String) {
    let snippet = &source.snippet;
    let found = source.quote.as_deref().filter(|q| !q.is_empty()).and_then(|q| {
        // Lower-casing can change byte lengths, so match on the original when it's ASCII-safe.
        snippet.find(q).map(|i| (i, i + q.len())).or_else(|| {
            let (hay, needle) = (snippet.to_lowercase(), q.to_lowercase());
            (hay.len() == snippet.len()).then(|| hay.find(&needle).map(|i| (i, i + needle.len()))).flatten()
        })
    });
    match found {
        Some((a, b)) => (snippet[..a].to_string(), snippet[a..b].to_string(), snippet[b..].to_string()),
        None => (snippet.clone(), String::new(), String::new()),
    }
}

/// The marker's accessible name: "Source 2: Rust 2024 edition guide".
#[must_use]
pub fn marker_label(index: usize, source: &Source) -> String {
    format!("Source {}: {}", index + 1, source.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_text_around_spans() {
        let text = "Rust is fast. It has no GC.";
        let spans = [CitedSpan::new(14, 27, &[1]), CitedSpan::new(0, 13, &[0, 1])];
        assert_eq!(
            runs(text, &spans, 2),
            vec![
                Run::Cited { text: "Rust is fast.".into(), sources: vec![0, 1] },
                Run::Text(" ".into()),
                Run::Cited { text: "It has no GC.".into(), sources: vec![1] },
            ]
        );
    }

    #[test]
    fn drops_bad_spans() {
        let text = "héllo world";
        let spans = [
            CitedSpan::new(0, 2, &[0]),   // cuts "é" in half
            CitedSpan::new(7, 99, &[0]),  // past the end
            CitedSpan::new(0, 6, &[5]),   // unknown source
            CitedSpan::new(0, 6, &[0]),   // fine
            CitedSpan::new(3, 9, &[0]),   // overlaps
        ];
        assert_eq!(runs(text, &spans, 1), vec![Run::Cited { text: "héllo".into(), sources: vec![0] }, Run::Text(" world".into())]);
        assert_eq!(CitedSpan::find(text, "world", &[0]), Some(CitedSpan::new(7, 12, &[0])));
    }

    #[test]
    fn hosts_and_initials() {
        assert_eq!(host("https://www.Rust-Lang.org/learn?x=1"), "rust-lang.org");
        assert_eq!(host("http://user@docs.rs:8080/serde"), "docs.rs");
        assert_eq!(host("example.com/a"), "example.com");
        assert_eq!(initial("https://docs.rs"), "D");
        assert_eq!(initial(""), "?");
    }

    #[test]
    fn quote_is_highlighted_ignoring_case() {
        let s = Source::new("t", "u", "Ownership rules: each value has one owner.").quote("Each value has one owner");
        assert_eq!(highlight(&s), ("Ownership rules: ".into(), "each value has one owner".into(), ".".into()));
        let none = Source::new("t", "u", "abc").quote("zzz");
        assert_eq!(highlight(&none), ("abc".into(), String::new(), String::new()));
        assert_eq!(marker_label(1, &s), "Source 2: t");
    }
}
