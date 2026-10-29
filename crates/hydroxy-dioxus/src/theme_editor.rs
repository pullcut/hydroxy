//! `ThemeEditor`: edit hydroxy's design tokens live. Pick a preset, then change
//! colours (with the colour picker), corner radii, the spacing scale and the
//! fonts. A preview region shows the result, contrast pairs that fall below
//! WCAG AA are flagged, and the theme exports as CSS or JSON.

use dioxus::prelude::*;
use hydroxy_headless::theme_editor::{
    self as h, COLOR_TOKENS, FONT_TOKENS, RADIUS_TOKENS, SPACE_SCALE, SPACE_SCALE_RANGE, ThemeEditorCommand,
    ThemeEditorEvent, ThemeEditorMachine, ThemeEditorState,
};
use hydroxy_statechart::Machine;

use crate::badge::{Badge, Tone};
use crate::button::{Button, ButtonVariant};
use crate::callout::{Callout, CalloutKind};
use crate::card::Card;
use crate::code_block::CodeBlock;
use crate::color_picker::ColorPicker;
use crate::icon::{Icon, IconName};
use crate::input::Input;
use crate::popover::{Popover, PopoverContent, PopoverTrigger};
use crate::segmented::{Segmented, SegmentedItem, SegmentedOption};
use crate::select::{Select, SelectOption};
use crate::slider::Slider;

/// A design-token editor with a scoped live preview.
#[component]
pub fn ThemeEditor(
    /// The preset to start from, as its `data-theme` value. Default `light`.
    #[props(default)]
    default_preset: Option<String>,
    /// Called with every token (without `--`) and its value after each change.
    on_change: Option<EventHandler<Vec<(String, String)>>>,
) -> Element {
    let input = use_hook(|| h::input(hydroxy_tokens::CSS, default_preset.as_deref().unwrap_or("light")));
    let mut state = use_signal(|| ThemeEditorState::new(&input));
    let mut format = use_signal(|| String::from("css"));
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));

    let send = use_callback(move |event: ThemeEditorEvent| {
        let step = ThemeEditorMachine::transition(&state.peek(), event, &input);
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for ThemeEditorCommand::Change(tokens) in step.commands {
            if let Some(handler) = on_change {
                handler.call(tokens);
            }
        }
    });

    let s = state.read().clone();
    let issues = s.contrast_issues();
    let summary = h::summary(&issues);
    let export = h::export(&s, &format());
    let preset_options: Vec<SelectOption> = h::presets().into_iter().map(|(v, l)| SelectOption::new(v, l)).collect();
    let heading = |part: &str| format!("{root}-{part}");

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", role: "group", "aria-label": "Theme editor",
            div { "data-scope": h::SCOPE, "data-part": "controls",
                div { "data-scope": h::SCOPE, "data-part": "toolbar",
                    Select {
                        label: "Preset",
                        options: preset_options,
                        value: s.preset.clone(),
                        on_change: move |p| send(ThemeEditorEvent::SetPreset(p)),
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        disabled: !s.is_dirty(),
                        on_click: move |_| send(ThemeEditorEvent::ResetAll),
                        "Reset all"
                    }
                }
                section { "data-scope": h::SCOPE, "data-part": "section", "aria-labelledby": heading("colours"),
                    h3 { id: heading("colours"), "Colours" }
                    for (token , label) in COLOR_TOKENS {
                        {
                            let hex = s.value(token).to_string();
                            let issue = h::token_issue(&issues, token);
                            let picked = h::picker_value(&s, token);
                            rsx! {
                                div {
                                    key: "{token}",
                                    "data-scope": h::SCOPE,
                                    "data-part": "row",
                                    "data-modified": s.modified(token).then_some(""),
                                    Popover {
                                        PopoverTrigger { variant: ButtonVariant::Ghost,
                                            span { "data-scope": h::SCOPE, "data-part": "swatch", style: "--swatch: {hex};" }
                                            span { "data-scope": h::SCOPE, "data-part": "name", "{label}" }
                                            span { "data-scope": h::SCOPE, "data-part": "value", "{hex}" }
                                        }
                                        PopoverContent {
                                            ColorPicker {
                                                label,
                                                value: Some(picked),
                                                swatches: Vec::new(),
                                                on_change: move |c| send(h::color_event(token, c)),
                                            }
                                        }
                                    }
                                    if let Some(i) = issue {
                                        span { "data-scope": h::SCOPE, "data-part": "warning", title: h::issue_text(&i),
                                            Icon { name: IconName::TriangleAlert, size: 14 }
                                            span { "data-scope": h::SCOPE, "data-part": "sr", "Low contrast " }
                                            {h::ratio_text(i.ratio)}
                                        }
                                    }
                                    ResetButton { label, token, modified: s.modified(token), send }
                                }
                            }
                        }
                    }
                }
                section { "data-scope": h::SCOPE, "data-part": "section", "aria-labelledby": heading("corners"),
                    h3 { id: heading("corners"), "Corners" }
                    for (token , label) in RADIUS_TOKENS {
                        div {
                            key: "{token}",
                            "data-scope": h::SCOPE,
                            "data-part": "row",
                            "data-modified": s.modified(token).then_some(""),
                            Slider {
                                label: "{label} radius",
                                min: 0.0,
                                max: h::RADIUS_MAX,
                                value: Some(h::radius(&s, token)),
                                on_input: move |v| send(h::radius_event(token, v)),
                                format: |v: f64| format!("{v}px"),
                            }
                            ResetButton { label, token, modified: s.modified(token), send }
                        }
                    }
                }
                section { "data-scope": h::SCOPE, "data-part": "section", "aria-labelledby": heading("spacing"),
                    h3 { id: heading("spacing"), "Spacing" }
                    div {
                        "data-scope": h::SCOPE,
                        "data-part": "row",
                        "data-modified": s.modified(SPACE_SCALE).then_some(""),
                        Slider {
                            label: "Spacing scale",
                            min: f64::from(SPACE_SCALE_RANGE.0),
                            max: f64::from(SPACE_SCALE_RANGE.1),
                            step: 5.0,
                            value: Some(f64::from(s.space_scale)),
                            on_input: move |v| send(h::space_event(v)),
                            format: |v: f64| format!("{v}%"),
                        }
                        ResetButton { label: "Spacing scale", token: SPACE_SCALE, modified: s.modified(SPACE_SCALE), send }
                    }
                }
                section { "data-scope": h::SCOPE, "data-part": "section", "aria-labelledby": heading("type"),
                    h3 { id: heading("type"), "Type" }
                    for (token , label) in FONT_TOKENS {
                        div {
                            key: "{token}",
                            "data-scope": h::SCOPE,
                            "data-part": "row",
                            "data-modified": s.modified(token).then_some(""),
                            Select {
                                label: "{label} font",
                                options: h::font_options(&s, token).into_iter().map(|o| SelectOption::new(o.clone(), o)).collect::<Vec<_>>(),
                                value: h::font_choice(s.value(token)),
                                on_change: move |choice: String| {
                                    let event = h::font_event(&state.peek(), token, &choice);
                                    send(event);
                                },
                            }
                            ResetButton { label, token, modified: s.modified(token), send }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "output",
                div {
                    role: "region",
                    "aria-label": "Preview",
                    "data-scope": h::SCOPE,
                    "data-part": "preview",
                    "data-theme": s.preset.clone(),
                    style: s.style(),
                    Preview {}
                }
                div { "data-scope": h::SCOPE, "data-part": "contrast", "data-ok": issues.is_empty().then_some(""),
                    p { "data-scope": h::SCOPE, "data-part": "live", role: "status", "aria-live": "polite", "{summary}" }
                    ul { "data-scope": h::SCOPE, "data-part": "issues",
                        for i in issues.iter() {
                            li { key: "{i.fg}-{i.bg}", {h::issue_text(i)} }
                        }
                    }
                }
                div { "data-scope": h::SCOPE, "data-part": "export",
                    Segmented {
                        label: "Export format",
                        items: h::EXPORT_FORMATS.iter().map(|(v, _)| SegmentedOption::new(*v)).collect::<Vec<_>>(),
                        default_value: "css",
                        on_value_change: move |v| format.set(v),
                        for (v , l) in h::EXPORT_FORMATS {
                            SegmentedItem { key: "{v}", value: v, label: l }
                        }
                    }
                    CodeBlock { code: export, lang: format() }
                }
            }
        }
    }
}

#[component]
fn ResetButton(label: &'static str, token: &'static str, modified: bool, send: Callback<ThemeEditorEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            "data-scope": h::SCOPE,
            "data-part": "reset",
            "aria-label": "Reset {label}",
            disabled: !modified,
            onclick: move |_| send(ThemeEditorEvent::Reset(token.to_string())),
            "Reset"
        }
    }
}

/// Realistic UI built from hydroxy components, so every token shows somewhere.
#[component]
fn Preview() -> Element {
    rsx! {
        Card {
            div { "data-scope": h::SCOPE, "data-part": "sample",
                h4 { "Invoice INV-2041" }
                p {
                    "Northwind Traders · due 14 November · "
                    a { href: "#preview", onclick: |e| e.prevent_default(), "View PDF" }
                }
                div { "data-scope": h::SCOPE, "data-part": "badges",
                    Badge { tone: Tone::Success, dot: true, "Paid" }
                    Badge { tone: Tone::Warning, dot: true, "Partial" }
                    Badge { tone: Tone::Danger, dot: true, "Overdue" }
                    Badge { tone: Tone::Info, dot: true, "Draft" }
                }
                Input { label: "Reference", default_value: "PO-88123", hint: "Shown on the customer's statement" }
                Callout { kind: CalloutKind::Warning, title: "Card expires soon", "The card on file expires at the end of the month." }
                div { "data-scope": h::SCOPE, "data-part": "actions",
                    Button { variant: ButtonVariant::Primary, "Send invoice" }
                    Button { variant: ButtonVariant::Accent, "Record payment" }
                    Button { variant: ButtonVariant::Secondary, "Cancel" }
                }
                small {
                    "Amounts in GBP · "
                    code { "£1,240.00" }
                }
            }
        }
    }
}
