//! `JsonInspector`: a collapsible JSON tree with type-coloured values, search that
//! opens the way to every match, copy value / copy path, and large containers grouped
//! into ranges of 100.

use dioxus::prelude::*;
use oxide_headless::json_inspector::{
    self as h, CopyWhat, JsonInspectorCommand, JsonInspectorEvent, JsonInspectorInput, JsonInspectorMachine,
    JsonInspectorState, JsonKind, JsonValue, quote, split_matches, visible_rows,
};
use oxide_statechart::Machine;

use crate::icon::{Icon, IconName};
use crate::search_field::{SearchField, Size};

#[derive(Clone, Copy)]
struct Ctx {
    root: Signal<String>,
    state: Signal<JsonInspectorState>,
    input: Memo<JsonInspectorInput>,
    notice: Signal<Option<&'static str>>,
}

impl Ctx {
    fn send(mut self, event: JsonInspectorEvent) {
        let step = JsonInspectorMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                JsonInspectorCommand::FocusRow(id) => {
                    // Wait a tick: the row may only exist after this render. Ids are hex.
                    let dom = h::row_dom_id(&self.root.peek(), &id);
                    let _ = document::eval(&format!(
                        "await new Promise((r) => setTimeout(r, 0)); document.getElementById('{dom}')?.focus();"
                    ));
                }
                JsonInspectorCommand::Copy(text, what) => {
                    // `quote` gives a JSON string, which is a valid JS string literal.
                    let mut done = document::eval(&format!(
                        "try {{ await navigator.clipboard.writeText({}); }} catch (_) {{}} await new Promise(r => setTimeout(r, 1600)); dioxus.send(true);",
                        quote(&text)
                    ));
                    self.notice.set(Some(h::copied_label(what)));
                    let mut notice = self.notice;
                    spawn(async move {
                        let _ = done.recv::<bool>().await;
                        notice.set(None);
                    });
                }
            }
        }
    }
}

/// Text with search hits wrapped in `<mark>`.
fn marked(text: &str, query: &str) -> Element {
    let runs: Vec<(bool, String)> = split_matches(text, query).into_iter().map(|(m, r)| (m, r.to_string())).collect();
    rsx! {
        for (hit, run) in runs {
            if hit {
                mark { "data-scope": h::SCOPE, "data-part": "mark", "{run}" }
            } else {
                "{run}"
            }
        }
    }
}

/// Inspect a JSON document. Invalid JSON shows the parse error with its line and column.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn JsonInspector(
    /// The JSON text.
    json: ReadSignal<String>,
    /// Levels open at start (the root is level 1). Default 2.
    #[props(default = 2)]
    expand_depth: usize,
    /// Accessible name for the tree.
    #[props(default = "JSON".to_string())]
    label: String,
) -> Element {
    let parsed = use_memo(move || h::parse(&json.read()));
    let input = use_memo(move || JsonInspectorInput {
        root: parsed.read().clone().unwrap_or(JsonValue::Null),
        expand_depth,
    });
    let mut state = use_signal(|| JsonInspectorMachine::initial(&input.peek()));
    // New data starts afresh at the configured depth.
    use_effect(move || state.set(JsonInspectorMachine::initial(&input.read())));
    let ctx = use_hook(|| Ctx {
        root: Signal::new(oxide_headless::next_id(h::SCOPE)),
        state,
        input,
        notice: Signal::new(None),
    });

    let s = state.read();
    let status = ctx.notice.read().map_or_else(|| h::status(&s), str::to_string);
    let root_id = ctx.root.read().clone();
    let parsed = parsed.read();
    let i = input.read();
    let rows = if parsed.is_ok() { visible_rows(&i.root, &s.expanded, &s.query) } else { Vec::new() };
    let stop = s.tab_stop(&rows).map(str::to_string);
    let query = s.query.clone();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            div { "data-scope": h::SCOPE, "data-part": "toolbar",
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "search",
                    onkeydown: move |e| {
                        if let Some(event) = h::search_keydown(&e.key().to_string(), e.modifiers().shift()) {
                            e.prevent_default();
                            ctx.send(event);
                        }
                    },
                    SearchField {
                        label: "Search keys and values",
                        placeholder: "Search",
                        size: Size::Sm,
                        on_input: move |q: String| ctx.send(JsonInspectorEvent::SetQuery(q)),
                    }
                }
                span { "data-scope": h::SCOPE, "data-part": "status", role: "status", "{status}" }
                div { "data-scope": h::SCOPE, "data-part": "depth", role: "group", "aria-label": "Expand to depth",
                    for (text, depth) in h::DEPTHS {
                        button {
                            key: "{text}",
                            r#type: "button",
                            "data-scope": h::SCOPE,
                            "data-part": "depth-button",
                            "aria-label": if depth == usize::MAX { "Expand all".to_string() } else { format!("Expand to level {depth}") },
                            onclick: move |_| ctx.send(JsonInspectorEvent::ExpandToDepth(depth)),
                            "{text}"
                        }
                    }
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "depth-button",
                        onclick: move |_| ctx.send(JsonInspectorEvent::ExpandToDepth(0)),
                        "Collapse"
                    }
                }
            }
            if let Err(e) = &*parsed {
                p { "data-scope": h::SCOPE, "data-part": "error", role: "alert",
                    Icon { name: IconName::CircleAlert, size: 14 }
                    "{h::error_text(e)}"
                }
            } else {
                ul { "data-scope": h::SCOPE, "data-part": "tree", role: "tree", "aria-label": "{label}",
                    for r in rows {
                        {
                            let a = h::item_attrs(&root_id, &r);
                            let primitive = !matches!(r.kind, JsonKind::Array | JsonKind::Object | JsonKind::Range);
                            let range = r.kind == JsonKind::Range;
                            let key = h::key_text(&r);
                            let (focused, keyed, clicked, copy_value, copy_path) =
                                (r.id.clone(), r.id.clone(), r.id.clone(), r.id.clone(), r.id.clone());
                            let expandable = r.expandable;
                            rsx! {
                                li {
                                    key: "{r.id}",
                                    role: "treeitem",
                                    id: a.id,
                                    "aria-label": a.aria_label,
                                    "aria-level": a.aria_level,
                                    "aria-setsize": a.aria_setsize,
                                    "aria-posinset": a.aria_posinset,
                                    "aria-expanded": a.aria_expanded.map(|e| e.to_string()),
                                    tabindex: h::tabindex(&r.id, stop.as_deref()),
                                    style: a.style,
                                    "data-scope": h::SCOPE,
                                    "data-part": "item",
                                    "data-state": a.data_state,
                                    "data-matched": r.matched.then_some(""),
                                    onclick: move |_| {
                                        let event = if expandable {
                                            JsonInspectorEvent::Toggle(clicked.clone())
                                        } else {
                                            JsonInspectorEvent::Focus(clicked.clone())
                                        };
                                        ctx.send(event);
                                    },
                                    onfocus: move |_| ctx.send(JsonInspectorEvent::Focus(focused.clone())),
                                    onkeydown: move |e| {
                                        let m = e.modifiers();
                                        if m.alt() {
                                            return;
                                        }
                                        if let Some(event) = h::keydown(&e.key().to_string(), &keyed, m.ctrl() || m.meta(), m.shift()) {
                                            e.prevent_default();
                                            ctx.send(event);
                                        }
                                    },
                                    span { "data-scope": h::SCOPE, "data-part": "toggle", "aria-hidden": "true",
                                        if expandable {
                                            Icon { name: IconName::ChevronRight, size: 14 }
                                        }
                                    }
                                    if let Some(k) = key {
                                        span { "data-scope": h::SCOPE, "data-part": "key", "data-index": r.index.then_some(""),
                                            if range { "{k}" } else { {marked(&k, &query)} }
                                        }
                                        span { "data-scope": h::SCOPE, "data-part": "colon", "aria-hidden": "true", ":" }
                                    }
                                    span { "data-scope": h::SCOPE, "data-part": "value", "data-type": a.data_type,
                                        if primitive { {marked(&r.text, &query)} } else { "{r.text}" }
                                    }
                                    span { "data-scope": h::SCOPE, "data-part": "actions",
                                        button {
                                            r#type: "button",
                                            tabindex: "-1",
                                            "data-scope": h::SCOPE,
                                            "data-part": "copy",
                                            "aria-label": "Copy value",
                                            title: "Copy value (Ctrl+C)",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                ctx.send(JsonInspectorEvent::Copy(copy_value.clone(), CopyWhat::Value));
                                            },
                                            Icon { name: IconName::Copy, size: 13 }
                                        }
                                        if !range {
                                            button {
                                                r#type: "button",
                                                tabindex: "-1",
                                                "data-scope": h::SCOPE,
                                                "data-part": "copy",
                                                "aria-label": "Copy path",
                                                title: "Copy path {r.path} (Ctrl+Shift+C)",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    ctx.send(JsonInspectorEvent::Copy(copy_path.clone(), CopyWhat::Path));
                                                },
                                                "$."
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
