//! Text layout and the SVG and JSON export.

use std::fmt::Write as _;

use super::{
    Point, Rect, STICKY_FONT, STICKY_PAD, Shape, ShapeKind, TEXT_FONT, arrow_ends, bounds, smooth_path, swatch,
};

/// Line height for text in a shape of this kind.
#[must_use]
pub fn line_height(kind: ShapeKind) -> f64 {
    if kind == ShapeKind::Text { TEXT_FONT * 1.3 } else { STICKY_FONT * 1.35 }
}

/// A shape's text broken into lines that fit its width. Glyph width is estimated.
#[must_use]
pub fn text_lines(s: &Shape) -> Vec<String> {
    let (font, pad) = if s.kind == ShapeKind::Text { (TEXT_FONT, 4.0) } else { (STICKY_FONT, STICKY_PAD) };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a small positive count")]
    let max = (((s.w - pad * 2.0) / (font * 0.55)).floor().max(1.0)) as usize;
    let mut lines = Vec::new();
    for para in s.text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let fits = line.chars().count() + word.chars().count() + usize::from(!line.is_empty()) <= max;
            if !line.is_empty() && !fits {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// SVG markup for one shape, in world units. Colours are token variables with fallbacks.
#[must_use]
pub fn shape_svg(s: &Shape, shapes: &[Shape]) -> String {
    let colour = swatch(s.color).css();
    let tint = |pct: u8, base: &str| format!("color-mix(in srgb, {colour} {pct}%, {base})");
    let rot = if s.rotation == 0.0 {
        String::new()
    } else {
        let c = s.center();
        format!(" transform=\"rotate({:.3} {:.2} {:.2})\"", s.rotation.to_degrees(), c.x, c.y)
    };
    let text = |fill: &str, font: f64, pad: f64| {
        let lh = line_height(s.kind);
        let mut t = format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" style=\"fill: {fill}; font: {font}px var(--font-body, system-ui, sans-serif);\">",
            s.x + pad,
            s.y + pad + font
        );
        for (i, line) in text_lines(s).iter().enumerate() {
            let dy = if i == 0 { 0.0 } else { lh };
            let _ = write!(t, "<tspan x=\"{:.2}\" dy=\"{dy:.2}\">{}</tspan>", s.x + pad, xml_escape(line));
        }
        t + "</text>"
    };
    let stroke = format!("stroke: {colour}; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round;");
    match s.kind {
        ShapeKind::Rect => format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"6\" style=\"{stroke} fill: {};\"{rot}/>",
            s.x,
            s.y,
            s.w,
            s.h,
            tint(12, "transparent")
        ),
        ShapeKind::Ellipse => {
            let c = s.center();
            format!(
                "<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" style=\"{stroke} fill: {};\"{rot}/>",
                c.x,
                c.y,
                s.w / 2.0,
                s.h / 2.0,
                tint(12, "transparent")
            )
        }
        ShapeKind::Arrow => {
            let (a, b) = arrow_ends(s, shapes);
            let ang = (b.y - a.y).atan2(b.x - a.x);
            let len = 14.0_f64.min(a.dist(b) * 0.5);
            let h1 = b.plus(Point::new(-len * (ang - 0.45).cos(), -len * (ang - 0.45).sin()));
            let h2 = b.plus(Point::new(-len * (ang + 0.45).cos(), -len * (ang + 0.45).sin()));
            format!(
                "<path d=\"M{:.2} {:.2}L{:.2} {:.2}M{:.2} {:.2}L{:.2} {:.2}L{:.2} {:.2}\" style=\"{stroke} fill: none;\"/>",
                a.x, a.y, b.x, b.y, h1.x, h1.y, b.x, b.y, h2.x, h2.y
            )
        }
        ShapeKind::Pen => format!(
            "<path d=\"{}\" style=\"stroke: {colour}; stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; fill: none;\"{rot}/>",
            smooth_path(&s.ink())
        ),
        ShapeKind::Sticky => format!(
            "<g{rot}><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"4\" style=\"fill: {}; stroke: {};\"/>{}</g>",
            s.x,
            s.y,
            s.w,
            s.h,
            tint(26, "var(--surface-card, #ffffff)"),
            tint(40, "transparent"),
            text("var(--text-1, #161e35)", STICKY_FONT, STICKY_PAD)
        ),
        ShapeKind::Text => format!("<g{rot}>{}</g>", text(&colour, TEXT_FONT, 4.0)),
    }
}

/// The whole scene as a standalone SVG document.
#[must_use]
pub fn export_svg(shapes: &[Shape]) -> String {
    let r = shapes
        .iter()
        .map(|s| bounds(s, shapes))
        .reduce(Rect::union)
        .unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0))
        .inflate(24.0);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {:.2} {:.2}\" width=\"{:.0}\" height=\"{:.0}\">",
        r.x, r.y, r.w, r.h, r.w, r.h
    );
    for s in shapes {
        out.push_str(&shape_svg(s, shapes));
    }
    out + "</svg>"
}

/// The scene as JSON: `{"version":1,"shapes":[…]}`.
#[must_use]
pub fn export_json(shapes: &[Shape]) -> String {
    let opt = |o: &Option<String>| o.as_deref().map_or_else(|| "null".to_string(), crate::json_inspector::quote);
    let items: Vec<String> = shapes
        .iter()
        .map(|s| {
            let points: Vec<String> = s.points.iter().map(|p| format!("[{:.3},{:.3}]", p.x, p.y)).collect();
            format!(
                "{{\"id\":{},\"type\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"h\":{:.2},\"rotation\":{:.4},\"color\":\"{}\",\"points\":[{}],\"start\":{},\"end\":{},\"text\":{}}}",
                crate::json_inspector::quote(&s.id),
                s.kind.as_str(),
                s.x,
                s.y,
                s.w,
                s.h,
                s.rotation,
                swatch(s.color).name.to_lowercase(),
                points.join(","),
                opt(&s.start),
                opt(&s.end),
                crate::json_inspector::quote(&s.text)
            )
        })
        .collect();
    format!("{{\"version\":1,\"shapes\":[{}]}}", items.join(","))
}
