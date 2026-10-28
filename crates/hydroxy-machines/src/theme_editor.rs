//! Theme editor: live edits to the hydroxy design tokens on top of a preset.
//!
//! Presets are read from the token stylesheet itself ([`theme_declarations`]),
//! so they never drift from `themes.css`. The state is the chosen preset plus
//! overrides; every view (scoped style, contrast checks, CSS and JSON export) is
//! derived from it. Contrast uses the WCAG 2.x relative-luminance formula.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use hydroxy_statechart::{Machine, Transition};

use crate::color::Rgba;

/// Editable colour tokens (without the leading `--`) with their labels.
pub const COLOR_TOKENS: [(&str, &str); 19] = [
    ("surface-page", "Page"),
    ("surface-card", "Card"),
    ("surface-sunken", "Sunken"),
    ("surface-raised", "Raised"),
    ("text-1", "Text"),
    ("text-2", "Secondary text"),
    ("text-3", "Muted text"),
    ("text-link", "Link"),
    ("border-1", "Border"),
    ("border-2", "Strong border"),
    ("accent", "Accent"),
    ("action-primary-bg", "Primary button"),
    ("action-primary-fg", "Primary button text"),
    ("action-accent-bg", "Accent button"),
    ("action-accent-fg", "Accent button text"),
    ("status-success", "Success"),
    ("status-warning", "Warning"),
    ("status-danger", "Danger"),
    ("status-info", "Info"),
];

/// Editable corner radii with their labels.
pub const RADIUS_TOKENS: [(&str, &str); 5] = [
    ("radius-xs", "Extra small"),
    ("radius-sm", "Small"),
    ("radius-md", "Medium"),
    ("radius-lg", "Large"),
    ("radius-xl", "Extra large"),
];

/// Editable font stacks with their labels.
pub const FONT_TOKENS: [(&str, &str); 3] = [("font-display", "Display"), ("font-body", "Body"), ("font-mono", "Mono")];

/// The spacing steps the scale regenerates.
pub const SPACE_STEPS: u8 = 14;

/// The pseudo-token name [`ThemeEditorEvent::Reset`] takes for the spacing scale.
pub const SPACE_SCALE: &str = "space-scale";

/// Lowest and highest spacing scale, in percent.
pub const SPACE_SCALE_RANGE: (u16, u16) = (50, 200);

/// Foreground, background and minimum ratio for every contrast check.
pub const CONTRAST_CHECKS: [(&str, &str, f64); 12] = [
    ("text-1", "surface-page", 4.5),
    ("text-1", "surface-card", 4.5),
    ("text-2", "surface-card", 4.5),
    ("text-3", "surface-card", 4.5),
    ("text-link", "surface-card", 4.5),
    ("action-primary-fg", "action-primary-bg", 4.5),
    ("action-accent-fg", "action-accent-bg", 4.5),
    ("accent", "surface-card", 3.0),
    ("status-success", "surface-card", 4.5),
    ("status-warning", "surface-card", 4.5),
    ("status-danger", "surface-card", 4.5),
    ("status-info", "surface-card", 4.5),
];

// ---- CSS reading ----

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start + 2..].find("*/").map_or("", |end| &rest[start + 2 + end + 2..]);
    }
    out.push_str(rest);
    out
}

/// Top-level `(selector list, body)` pairs in source order. At-rules
/// (`@import`, `@media` and friends) are skipped whole.
fn rules(css: &str) -> Vec<(String, String)> {
    let css = strip_comments(css);
    let bytes = css.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        // Read a prelude up to `{`, or `;` for a block-less at-rule.
        while i < bytes.len() && bytes[i] != b'{' && bytes[i] != b';' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let prelude = css[start..i].trim();
        if bytes[i] == b';' {
            i += 1;
            continue;
        }
        let body_start = i + 1;
        let mut depth = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        let body = &css[body_start..i.min(bytes.len())];
        i += 1;
        if !prelude.starts_with('@') {
            out.push((prelude.to_string(), body.to_string()));
        }
    }
    out
}

fn selects_theme(selector: &str, theme: &str) -> bool {
    selector.split(',').map(str::trim).any(|s| {
        s == ":root"
            || s.strip_prefix("[data-theme=\"").and_then(|r| r.strip_suffix("\"]")) == Some(theme)
            || s.strip_prefix("[data-theme^=\"")
                .and_then(|r| r.strip_suffix("\"]"))
                .is_some_and(|p| theme.starts_with(p))
    })
}

/// Every declaration that applies to an element with `data-theme="theme"` at the
/// document root, unresolved, in cascade order (later rules win; `:root` and the
/// attribute selectors have equal specificity). Keys keep a leading `--` for
/// custom properties.
#[must_use]
pub fn theme_declarations(css: &str, theme: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (selector, body) in rules(css) {
        if !selects_theme(&selector, theme) {
            continue;
        }
        for decl in body.split(';') {
            if let Some((name, value)) = decl.split_once(':') {
                let (name, value) = (name.trim(), value.trim());
                if !name.is_empty() && !value.is_empty() {
                    out.insert(name.to_string(), value.to_string());
                }
            }
        }
    }
    out
}

/// `--name`'s value with every `var(--x)` substituted, or `None` if one is undefined.
#[must_use]
pub fn resolve(decls: &BTreeMap<String, String>, name: &str) -> Option<String> {
    resolve_value(decls, decls.get(name)?, 0)
}

fn resolve_value(decls: &BTreeMap<String, String>, value: &str, depth: u8) -> Option<String> {
    if depth > 16 {
        return None;
    }
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("var(") {
        out.push_str(&rest[..start]);
        let inner = &rest[start + 4..];
        let end = inner.find(')')?;
        let name = inner[..end].split(',').next()?.trim();
        out.push_str(&resolve_value(decls, decls.get(name)?, depth + 1)?);
        rest = &inner[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// Declarations that reference one of `names` (directly or through each other),
/// with their raw values. Re-declaring these inside a scoped container makes
/// composites like `--text-sm` pick up an edited `--font-body`.
#[must_use]
pub fn dependents(decls: &BTreeMap<String, String>, names: &[&str]) -> Vec<(String, String)> {
    let mut found: Vec<String> = Vec::new();
    let mut changed = true;
    while changed {
        changed = false;
        for (name, value) in decls {
            if !name.starts_with("--") || names.contains(&&name[2..]) || found.contains(name) {
                continue;
            }
            let uses = |n: &str| value.contains(&format!("var({n})")) || value.contains(&format!("var({n},"));
            if names.iter().any(|n| uses(&format!("--{n}"))) || found.iter().any(|n| uses(n)) {
                found.push(name.clone());
                changed = true;
            }
        }
    }
    found
        .into_iter()
        .map(|n| {
            let value = decls[&n].clone();
            (n, value)
        })
        .collect()
}

// ---- contrast ----

/// WCAG 2.x relative luminance of an sRGB colour (alpha ignored).
#[must_use]
pub fn luminance(c: Rgba) -> f64 {
    let lin = |v: u8| {
        let s = f64::from(v) / 255.0;
        if s <= 0.040_45 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// WCAG contrast ratio between two colours, 1 to 21.
#[must_use]
pub fn contrast(a: Rgba, b: Rgba) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// One failing contrast check.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastIssue {
    /// The foreground token.
    pub fg: &'static str,
    /// The background token.
    pub bg: &'static str,
    /// The ratio now.
    pub ratio: f64,
    /// The ratio it needs.
    pub min: f64,
}

/// A token's label, or the name itself.
#[must_use]
pub fn label(token: &str) -> &str {
    COLOR_TOKENS.iter().chain(&RADIUS_TOKENS).chain(&FONT_TOKENS).find(|(n, _)| *n == token).map_or(token, |(_, l)| l)
}

// ---- machine ----

/// Props for [`ThemeEditorMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeEditorInput {
    /// The token stylesheet presets are read from (hydroxy-tokens' `CSS`).
    pub css: &'static str,
    /// The preset to start from, as its `data-theme` value. `light` is the `:root` default.
    pub default_preset: String,
}

/// What [`ThemeEditorMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeEditorState {
    /// The preset's `data-theme` value.
    pub preset: String,
    /// The preset's values for every editable token, plus `space-1`… and `color-scheme`.
    pub base: BTreeMap<String, String>,
    /// The preset's raw declarations, for [`dependents`].
    pub raw: BTreeMap<String, String>,
    /// Edited tokens and their values.
    pub overrides: BTreeMap<String, String>,
    /// Spacing scale in percent; 100 is the preset.
    pub space_scale: u16,
}

fn editable() -> impl Iterator<Item = &'static str> {
    COLOR_TOKENS.iter().chain(&RADIUS_TOKENS).chain(&FONT_TOKENS).map(|(n, _)| *n)
}

fn load(css: &str, preset: &str) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let raw = theme_declarations(css, preset);
    let mut base = BTreeMap::new();
    let steps = (1..=SPACE_STEPS).map(|n| format!("space-{n}"));
    for name in editable().map(str::to_string).chain(steps) {
        if let Some(v) = resolve(&raw, &format!("--{name}")) {
            base.insert(name, v);
        }
    }
    base.insert("color-scheme".into(), raw.get("color-scheme").cloned().unwrap_or_else(|| "light".into()));
    (base, raw)
}

/// Pixels in a length like `10px` or `0`.
#[must_use]
pub fn px(value: &str) -> Option<f64> {
    let v = value.trim();
    v.strip_suffix("px").unwrap_or(v).trim().parse().ok()
}

/// The first family in a font stack, unquoted.
#[must_use]
pub fn first_family(stack: &str) -> &str {
    stack.split(',').next().unwrap_or("").trim().trim_matches(['\'', '"'])
}

impl ThemeEditorState {
    /// The starting state: the input's preset, unedited.
    #[must_use]
    pub fn new(input: &ThemeEditorInput) -> Self {
        let (base, raw) = load(input.css, &input.default_preset);
        Self { preset: input.default_preset.clone(), base, raw, overrides: BTreeMap::new(), space_scale: 100 }
    }

    /// A token's value: the override, else the preset's.
    #[must_use]
    pub fn value(&self, token: &str) -> &str {
        self.overrides.get(token).or_else(|| self.base.get(token)).map_or("", String::as_str)
    }

    /// A colour token as a colour.
    #[must_use]
    pub fn color(&self, token: &str) -> Option<Rgba> {
        Rgba::parse_hex(self.value(token))
    }

    /// Whether a token (or [`SPACE_SCALE`]) differs from the preset.
    #[must_use]
    pub fn modified(&self, token: &str) -> bool {
        if token == SPACE_SCALE { self.space_scale != 100 } else { self.overrides.contains_key(token) }
    }

    /// Whether anything differs from the preset.
    #[must_use]
    pub fn dirty(&self) -> bool {
        !self.overrides.is_empty() || self.space_scale != 100
    }

    /// Whether the preset renders dark.
    #[must_use]
    pub fn dark(&self) -> bool {
        self.base.get("color-scheme").is_some_and(|s| s == "dark")
    }

    /// Every token the theme sets, `--`-less, with its current value (spacing scaled).
    #[must_use]
    pub fn tokens(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = editable().map(|n| (n.to_string(), self.value(n).to_string())).collect();
        for n in 1..=SPACE_STEPS {
            let name = format!("space-{n}");
            let base = self.base.get(&name).and_then(|v| px(v)).unwrap_or(0.0);
            let scaled = (base * f64::from(self.space_scale) / 100.0).round();
            out.push((name, if scaled == 0.0 { "0".into() } else { format!("{scaled}px") }));
        }
        out
    }

    /// Inline `style` for the preview container: every token plus the composites
    /// that read them, so the edits land inside the container only.
    #[must_use]
    pub fn style(&self) -> String {
        let mut out = format!("color-scheme: {};", self.value("color-scheme"));
        let tokens = self.tokens();
        for (name, value) in &tokens {
            let _ = write!(out, " --{name}: {value};");
        }
        let names: Vec<&str> = tokens.iter().map(|(n, _)| n.as_str()).collect();
        for (name, value) in dependents(&self.raw, &names) {
            let _ = write!(out, " {name}: {value};");
        }
        out
    }

    /// Failing contrast checks.
    #[must_use]
    pub fn contrast_issues(&self) -> Vec<ContrastIssue> {
        CONTRAST_CHECKS
            .iter()
            .filter_map(|&(fg, bg, min)| {
                let ratio = contrast(self.color(fg)?, self.color(bg)?);
                (ratio < min).then_some(ContrastIssue { fg, bg, ratio, min })
            })
            .collect()
    }

    /// The theme as a CSS rule under `[data-theme="custom"]`.
    #[must_use]
    pub fn export_css(&self) -> String {
        let mut out = format!("[data-theme=\"custom\"] {{\n  color-scheme: {};\n", self.value("color-scheme"));
        for (name, value) in self.tokens() {
            let _ = writeln!(out, "  --{name}: {value};");
        }
        out.push('}');
        out
    }

    /// The theme as a JSON object of custom property names to values.
    #[must_use]
    pub fn export_json(&self) -> String {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        let body: Vec<String> =
            self.tokens().iter().map(|(n, v)| format!("  \"--{}\": \"{}\"", esc(n), esc(v))).collect();
        format!("{{\n{}\n}}", body.join(",\n"))
    }
}

/// Inputs to [`ThemeEditorMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeEditorEvent {
    /// Start from another preset. Edits are dropped.
    SetPreset(String),
    /// Set a token. The preset's own value clears the override.
    Set {
        /// Token name without `--`.
        token: String,
        /// CSS value.
        value: String,
    },
    /// Set the spacing scale in percent (clamped to [`SPACE_SCALE_RANGE`]).
    SetSpaceScale(u16),
    /// Put a token (or [`SPACE_SCALE`]) back to the preset.
    Reset(String),
    /// Put everything back to the preset.
    ResetAll,
}

/// Side effects from [`ThemeEditorMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeEditorCommand {
    /// The theme changed: every token and its value now.
    Change(Vec<(String, String)>),
}

/// Edits a theme on top of a preset.
#[derive(Debug, Clone, Copy)]
pub struct ThemeEditorMachine;

impl Machine for ThemeEditorMachine {
    type Input = ThemeEditorInput;
    type State = ThemeEditorState;
    type Event = ThemeEditorEvent;
    type Command = ThemeEditorCommand;

    fn initial(input: &ThemeEditorInput) -> ThemeEditorState {
        ThemeEditorState::new(input)
    }

    fn transition(
        state: &ThemeEditorState,
        event: ThemeEditorEvent,
        input: &ThemeEditorInput,
    ) -> Transition<ThemeEditorState, ThemeEditorCommand> {
        let mut next = state.clone();
        match event {
            ThemeEditorEvent::SetPreset(preset) => {
                if preset != state.preset {
                    (next.base, next.raw) = load(input.css, &preset);
                    next.preset = preset;
                }
                next.overrides.clear();
                next.space_scale = 100;
            }
            ThemeEditorEvent::Set { token, value } => {
                if !editable().any(|n| n == token) {
                    return Transition::to(next);
                }
                if state.base.get(&token) == Some(&value) {
                    next.overrides.remove(&token);
                } else {
                    next.overrides.insert(token, value);
                }
            }
            ThemeEditorEvent::SetSpaceScale(scale) => {
                next.space_scale = scale.clamp(SPACE_SCALE_RANGE.0, SPACE_SCALE_RANGE.1);
            }
            ThemeEditorEvent::Reset(token) => {
                if token == SPACE_SCALE {
                    next.space_scale = 100;
                } else {
                    next.overrides.remove(&token);
                }
            }
            ThemeEditorEvent::ResetAll => {
                next.overrides.clear();
                next.space_scale = 100;
            }
        }
        let commands = if next == *state { Vec::new() } else { vec![ThemeEditorCommand::Change(next.tokens())] };
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    const CSS: &str = r#"
        @import url('x;y');
        /* a comment with { braces } */
        :root { --ink: #0b1229; --paper: #ffffff; --text-1: var(--ink); --surface-card: var(--paper);
                --font-body: 'Hanken Grotesk', sans-serif; --text-sm: 400 14px/1.5 var(--font-body);
                --font-text: var(--font-body); --space-3: 8px; --radius-md: 10px; color-scheme: light; }
        [data-theme="dark"], [data-theme="noir"] { --text-1: var(--paper); --surface-card: var(--ink); color-scheme: dark; }
        [data-theme^="noir"] { --radius-md: 0; }
        [data-theme="dark"] .button { --text-1: red; }
        @media (prefers-reduced-motion: reduce) { :root { --text-1: blue; } }
    "#;

    fn input(preset: &str) -> ThemeEditorInput {
        ThemeEditorInput { css: CSS, default_preset: preset.into() }
    }

    #[test]
    fn cascade_follows_selectors_and_source_order() {
        let d = theme_declarations(CSS, "dark");
        assert_eq!(resolve(&d, "--text-1").as_deref(), Some("#ffffff"));
        assert_eq!(d.get("--radius-md").map(String::as_str), Some("10px"));
        let n = theme_declarations(CSS, "noir-dark");
        assert_eq!(n.get("--radius-md").map(String::as_str), Some("0"));
        assert_eq!(resolve(&n, "--text-1").as_deref(), Some("#0b1229"), "noir-dark is not noir");
        let l = theme_declarations(CSS, "light");
        assert_eq!(resolve(&l, "--text-1").as_deref(), Some("#0b1229"));
        assert_eq!(resolve(&l, "--missing"), None);
    }

    #[test]
    fn dependents_follow_references() {
        let l = theme_declarations(CSS, "light");
        let deps = dependents(&l, &["font-body"]);
        let names: Vec<&str> = deps.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"--text-sm") && names.contains(&"--font-text"));
        assert!(!names.contains(&"--text-1"));
    }

    #[test]
    fn contrast_matches_wcag() {
        let (black, white) = (Rgba::rgb(0, 0, 0), Rgba::rgb(255, 255, 255));
        assert!((contrast(black, white) - 21.0).abs() < 1e-9);
        assert!((contrast(white, white) - 1.0).abs() < 1e-9);
        let grey = Rgba::rgb(0x77, 0x77, 0x77);
        assert!((contrast(grey, white) - 4.48).abs() < 0.01);
        assert!((contrast(Rgba::rgb(0x76, 0x76, 0x76), white) - 4.54).abs() < 0.01);
    }

    #[test]
    fn edits_reset_and_presets() {
        let i = input("light");
        let s = ThemeEditorMachine::initial(&i);
        assert_eq!(s.value("text-1"), "#0b1229");
        let t = step::<ThemeEditorMachine>(
            &s,
            ThemeEditorEvent::Set { token: "text-1".into(), value: "#777777".into() },
            &i,
        );
        assert!(t.state.modified("text-1"));
        assert_eq!(t.commands.len(), 1);
        let issues = t.state.contrast_issues();
        assert_eq!(issues.len(), 1);
        assert_eq!((issues[0].fg, issues[0].bg), ("text-1", "surface-card"));
        // Setting the preset value back clears the override.
        let back = step::<ThemeEditorMachine>(
            &t.state,
            ThemeEditorEvent::Set { token: "text-1".into(), value: "#0b1229".into() },
            &i,
        );
        assert!(!back.state.dirty());
        let reset = step::<ThemeEditorMachine>(&t.state, ThemeEditorEvent::Reset("text-1".into()), &i).state;
        assert_eq!(reset.value("text-1"), "#0b1229");
        // Unknown tokens are ignored.
        let t2 = step::<ThemeEditorMachine>(&s, ThemeEditorEvent::Set { token: "evil".into(), value: "x".into() }, &i);
        assert!(t2.commands.is_empty());
        let dark = step::<ThemeEditorMachine>(&t.state, ThemeEditorEvent::SetPreset("dark".into()), &i).state;
        assert!(dark.dark() && !dark.dirty());
        assert_eq!(dark.value("text-1"), "#ffffff");
    }

    #[test]
    fn spacing_scales_and_exports() {
        let i = input("light");
        let s = ThemeEditorMachine::initial(&i);
        let s = step::<ThemeEditorMachine>(&s, ThemeEditorEvent::SetSpaceScale(150), &i).state;
        assert!(s.tokens().contains(&("space-3".into(), "12px".into())));
        assert!(s.tokens().contains(&("space-1".into(), "0".into())), "undefined steps are 0");
        let clamped = step::<ThemeEditorMachine>(&s, ThemeEditorEvent::SetSpaceScale(999), &i).state;
        assert_eq!(clamped.space_scale, 200);
        let style = s.style();
        assert!(style.starts_with("color-scheme: light;"));
        assert!(style.contains("--font-body: 'Hanken Grotesk', sans-serif;"));
        assert!(style.contains("--text-sm: 400 14px/1.5 var(--font-body);"));
        let css = s.export_css();
        assert!(css.starts_with("[data-theme=\"custom\"] {\n  color-scheme: light;\n") && css.ends_with('}'));
        assert!(css.contains("  --space-3: 12px;\n"));
        let json = s.export_json();
        assert!(json.contains("\"--text-1\": \"#0b1229\""));
        assert_eq!(px("10px"), Some(10.0));
        assert_eq!(px("0"), Some(0.0));
        assert_eq!(first_family("'JetBrains Mono', monospace"), "JetBrains Mono");
    }
}
