//! QR code connector. Anatomy: root, frame, svg (`role="img"`), background,
//! modules (one `<path>`), logo, download, error. Every part carries
//! `data-scope="qr-code"` and its `data-part`.
//!
//! The modules are drawn as a single path of row runs, so the code stays one
//! DOM node at any version. Colours come from `--qr-fg` and `--qr-bg` on the
//! root (dark ink on white paper by default, whatever the theme, because
//! scanners expect dark on light). A logo clears a centred square of modules
//! and forces error correction H. The download link is a `data:` URL of a
//! standalone SVG with concrete colours, so it needs no script.

use std::fmt::Write as _;

pub use hydroxy_machines::qr_code::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "qr-code";

/// Quiet zone the standard asks for, in modules.
pub const DEFAULT_MARGIN: usize = 4;

/// Default rendered width and height in px.
pub const DEFAULT_SIZE: u32 = 192;

/// Dark colour for the downloaded file when the module colour is a CSS variable.
pub const DOWNLOAD_FG: &str = "#0b1229";

/// Light colour for the downloaded file when the background is a CSS variable.
pub const DOWNLOAD_BG: &str = "#ffffff";

/// A logo hides modules, so it needs the strongest error correction.
#[must_use]
pub const fn effective_ecc(ecc: ErrorCorrection, logo: bool) -> ErrorCorrection {
    if logo { ErrorCorrection::High } else { ecc }
}

/// The square a logo covers: first module and side, centred, about 22% of the code.
#[must_use]
pub fn logo_box(modules: usize) -> (usize, usize) {
    let mut side = (modules * 22).div_ceil(100);
    // Same parity as the (odd) code so it sits dead centre.
    if side.is_multiple_of(2) {
        side += 1;
    }
    ((modules - side) / 2, side)
}

/// The `d` of the modules path: each run of dark modules in a row is one rectangle,
/// offset by `margin`. With `logo`, the logo square is left empty.
#[must_use]
pub fn path(qr: &QrCode, margin: usize, logo: bool) -> String {
    let hole = logo.then(|| logo_box(qr.size()));
    let inside = |x: usize, y: usize| hole.is_some_and(|(s, n)| (s..s + n).contains(&x) && (s..s + n).contains(&y));
    let dark = |x: usize, y: usize| qr.module(x, y) && !inside(x, y);
    let mut d = String::new();
    for y in 0..qr.size() {
        let mut x = 0;
        while x < qr.size() {
            if dark(x, y) {
                let start = x;
                while x < qr.size() && dark(x, y) {
                    x += 1;
                }
                let n = x - start;
                let _ = write!(d, "M{} {}h{n}v1h-{n}z", start + margin, y + margin);
            } else {
                x += 1;
            }
        }
    }
    d
}

/// Modules per side including the quiet zone on both sides.
#[must_use]
pub const fn total(qr: &QrCode, margin: usize) -> usize {
    qr.size() + 2 * margin
}

/// The SVG `viewBox`: one unit per module.
#[must_use]
pub fn view_box(qr: &QrCode, margin: usize) -> String {
    let t = total(qr, margin);
    format!("0 0 {t} {t}")
}

/// `style` for the logo slot: the cleared square as percentages of the whole code.
#[must_use]
pub fn logo_style(qr: &QrCode, margin: usize) -> String {
    let (start, side) = logo_box(qr.size());
    #[allow(clippy::cast_precision_loss, reason = "at most 177 + 2 × margin modules")]
    let pct = |n: usize| n as f64 / total(qr, margin) as f64 * 100.0;
    format!(
        "left: {:.3}%; top: {:.3}%; width: {:.3}%; height: {:.3}%;",
        pct(start + margin),
        pct(start + margin),
        pct(side),
        pct(side)
    )
}

/// `style` for the root: the rendered size and any colour overrides.
#[must_use]
pub fn root_style(size: u32, fg: Option<&str>, bg: Option<&str>) -> String {
    let mut style = format!("--qr-size: {size}px;");
    if let Some(fg) = fg {
        let _ = write!(style, " --qr-fg: {fg};");
    }
    if let Some(bg) = bg {
        let _ = write!(style, " --qr-bg: {bg};");
    }
    style
}

/// The accessible name: `label` if given, else `"QR code: <text>"`.
#[must_use]
pub fn aria_label(text: &str, label: Option<&str>) -> String {
    label.map_or_else(|| format!("QR code: {text}"), str::to_string)
}

/// A colour a standalone file can use: `color` unless missing or a CSS `var(...)`.
#[must_use]
pub fn concrete<'a>(color: Option<&'a str>, fallback: &'a str) -> &'a str {
    match color {
        Some(c) if !c.trim_start().starts_with("var(") && !c.trim().is_empty() => c,
        _ => fallback,
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}

/// A standalone SVG file of the code (no logo) in concrete colours.
#[must_use]
pub fn svg_document(qr: &QrCode, margin: usize, fg: Option<&str>, bg: Option<&str>) -> String {
    let t = total(qr, margin);
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {t} {t}" width="{w}" height="{w}" shape-rendering="crispEdges"><rect width="{t}" height="{t}" fill="{bg}"/><path d="{d}" fill="{fg}"/></svg>"#,
        w = t * 8,
        bg = escape_attr(concrete(bg, DOWNLOAD_BG)),
        fg = escape_attr(concrete(fg, DOWNLOAD_FG)),
        d = path(qr, margin, false),
    )
}

/// `href` for a download link: the SVG as a percent-encoded `data:` URL.
#[must_use]
pub fn data_url(svg: &str) -> String {
    let mut url = String::from("data:image/svg+xml;charset=utf-8,");
    for b in svg.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~/:=,;'".contains(&b) {
            url.push(char::from(b));
        } else {
            let _ = write!(url, "%{b:02X}");
        }
    }
    url
}

/// Shown in place of the code when the text doesn't fit.
pub const TOO_LONG: &str = "Too much text for a QR code. Shorten it or lower the error correction.";

#[cfg(test)]
mod tests {
    use super::*;

    /// The (x, y, width) of each run rectangle in a path.
    fn rects(d: &str) -> Vec<(usize, usize, usize)> {
        d.split('M')
            .skip(1)
            .map(|r| {
                let (xy, rest) = r.split_once('h').unwrap();
                let (x, y) = xy.split_once(' ').unwrap();
                (x.parse().unwrap(), y.parse().unwrap(), rest.split('v').next().unwrap().parse().unwrap())
            })
            .collect()
    }

    #[test]
    fn path_merges_runs_and_respects_the_margin() {
        let qr = QrCode::encode("oxide", ErrorCorrection::Low).unwrap();
        let d = path(&qr, 4, false);
        // The top-left finder's first row is one 7-wide run at (4, 4).
        assert!(d.starts_with("M4 4h7v1h-7z"), "{d}");
        let area: usize = rects(&d).iter().map(|r| r.2).sum();
        let dark =
            (0..qr.size()).flat_map(|y| (0..qr.size()).map(move |x| (x, y))).filter(|&(x, y)| qr.module(x, y)).count();
        assert_eq!(area, dark);
        assert_eq!(view_box(&qr, 4), "0 0 29 29");
    }

    #[test]
    fn logo_clears_a_centred_square() {
        let qr = QrCode::encode("https://oxide.rs", effective_ecc(ErrorCorrection::Low, true)).unwrap();
        assert_eq!((qr.ecc(), qr.size()), (ErrorCorrection::High, 29));
        let (start, side) = logo_box(qr.size());
        assert_eq!((start, side), (11, 7));
        for (x, y, n) in rects(&path(&qr, 0, true)) {
            let rows = (start..start + side).contains(&y);
            assert!(!rows || x + n <= start || x >= start + side, "run at ({x}, {y}) crosses the logo");
        }
        assert_eq!(logo_style(&qr, 0), "left: 37.931%; top: 37.931%; width: 24.138%; height: 24.138%;");
    }

    #[test]
    fn download_uses_concrete_colours() {
        assert_eq!(concrete(Some("var(--ink-900)"), DOWNLOAD_FG), DOWNLOAD_FG);
        assert_eq!(concrete(Some("#123456"), DOWNLOAD_FG), "#123456");
        assert_eq!(concrete(None, DOWNLOAD_BG), DOWNLOAD_BG);
        let qr = QrCode::encode("1", ErrorCorrection::Low).unwrap();
        let svg = svg_document(&qr, 2, Some("var(--x)"), Some("#fafafa"));
        assert!(svg.contains(r##"fill="#fafafa""##) && svg.contains(r##"fill="#0b1229""##));
        let url = data_url(&svg);
        assert!(url.starts_with("data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns=%22http://www.w3.org/2000/svg%22"));
        assert!(!url.contains('#') && !url.contains('<') && !url.contains(' '));
    }

    #[test]
    fn styles_and_labels() {
        assert_eq!(root_style(160, None, Some("white")), "--qr-size: 160px; --qr-bg: white;");
        assert_eq!(aria_label("WIFI:T:WPA;", None), "QR code: WIFI:T:WPA;");
        assert_eq!(aria_label("x", Some("Ticket")), "Ticket");
    }
}
