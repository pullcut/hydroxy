//! Property test: [`safe_url`] never lets a script or data URL through, however it is
//! cased, padded or split by control characters.
#![cfg(feature = "markdown")]

use hydroxy_headless::markdown::safe_url;
use proptest::prelude::*;

/// The scheme a browser would read (WHATWG URL parsing): leading and trailing C0
/// controls and spaces trimmed, tab and newlines removed anywhere, lowercased.
fn browser_scheme(url: &str) -> String {
    let c0 = |c: char| c <= ' ';
    let squashed: String = url.trim_matches(c0).chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect();
    squashed.split(':').next().unwrap_or_default().to_ascii_lowercase()
}

/// A dangerous scheme with random case, leading padding a browser trims, and a tab or
/// newline (which browsers delete) somewhere inside it.
fn hostile() -> impl Strategy<Value = String> {
    let scheme = prop::sample::select(vec!["javascript", "vbscript", "data"]);
    let pad = prop::sample::select(vec!["", " ", "  ", "\u{0}", "\u{1f}", " \u{c}"]);
    let inner = prop::sample::select(vec!["", "\t", "\n", "\r"]);
    (scheme, prop::collection::vec(any::<bool>(), 10), pad, inner, any::<u8>(), ".{0,12}").prop_map(
        |(scheme, upper, pad, inner, at, rest)| {
            let mut s: String = scheme
                .chars()
                .zip(upper.iter().cycle())
                .map(|(c, &u)| if u { c.to_ascii_uppercase() } else { c })
                .collect();
            s.insert_str(usize::from(at) % (s.len() + 1), inner);
            format!("{pad}{s}:{rest}")
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn hostile_schemes_are_refused(url in hostile()) {
        prop_assert_eq!(safe_url(&url), None);
    }

    #[test]
    fn nothing_accepted_runs_script(url in "\\PC{0,24}") {
        if let Some(out) = safe_url(&url) {
            let scheme = browser_scheme(&out);
            prop_assert!(!["javascript", "vbscript", "data"].contains(&scheme.as_str()), "{out:?}");
        }
    }
}
