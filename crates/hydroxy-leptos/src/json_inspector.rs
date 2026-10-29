//! `<JsonInspector>`: a collapsible JSON tree with type-coloured values, search that
//! opens the way to every match, copy value / copy path, and large containers grouped
//! into ranges of 100.

use crate::drive::drive;
use std::collections::HashMap;
use std::time::Duration;

use hydroxy_headless::json_inspector::{
    self as h, CopyWhat, JsonInspectorCommand, JsonInspectorEvent, JsonInspectorInput, JsonInspectorMachine,
    JsonInspectorState, JsonKind, JsonRow, JsonValue, split_matches, visible_rows,
};
use hydroxy_statechart::Machine;
use leptos::attr::custom::custom_attribute;
use leptos::either::Either;
use leptos::prelude::*;

use crate::focus::focus_soon;
use crate::icon::{Icon, IconName};
use crate::js::copy;
use crate::modifiers::key_mods;
use crate::search_field::{SearchField, Size};

/// Visible rows: order plus lookup, so each row view finds its data in O(1).
#[derive(Clone, PartialEq, Default)]
struct Rows {
    order: Vec<String>,
    by_id: HashMap<String, JsonRow>,
}

#[derive(Clone, Copy)]
struct Ctx {
    root: StoredValue<String>,
    state: RwSignal<JsonInspectorState>,
    input: Memo<JsonInspectorInput>,
    notice: RwSignal<Option<&'static str>>,
}

impl Ctx {
    fn send(self, event: JsonInspectorEvent) {
        for command in drive::<JsonInspectorMachine, _>(self.state, self.input, event) {
            match command {
                // The row may only exist after this render.
                JsonInspectorCommand::FocusRow(id) => {
                    let dom = h::row_dom_id(&self.root.get_value(), &id);
                    focus_soon(&dom);
                }
                JsonInspectorCommand::Copy(text, what) => {
                    let notice = self.notice;
                    copy(&text, move || {
                        notice.try_set(Some(h::copied_label(what)));
                        set_timeout(
                            move || {
                                notice.try_set(None);
                            },
                            Duration::from_millis(1600),
                        );
                    });
                }
            }
        }
    }
}

/// Inspect a JSON document. Invalid JSON shows the parse error with its line and column.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn JsonInspector(
    /// The JSON text.
    #[prop(into)]
    json: Signal<String>,
    /// Levels open at start (the root is level 1). Default 2.
    #[prop(default = 2)]
    expand_depth: usize,
    /// Accessible name for the tree.
    #[prop(default = "JSON".into(), into)]
    label: String,
) -> impl IntoView {
    let parsed = Memo::new(move |_| json.with(|j| h::parse(j)));
    let input = Memo::new(move |_| JsonInspectorInput {
        root: parsed.with(|p| p.clone().unwrap_or(JsonValue::Null)),
        expand_depth,
    });
    let state = RwSignal::new(input.with_untracked(JsonInspectorMachine::initial));
    // New data starts afresh at the configured depth.
    Effect::new(move |prev: Option<()>| {
        let fresh = input.with(JsonInspectorMachine::initial);
        if prev.is_some() {
            state.set(fresh);
        }
    });
    let ctx = Ctx { root: StoredValue::new(crate::id::next_id(h::SCOPE)), state, input, notice: RwSignal::new(None) };
    let rows = Memo::new(move |_| {
        input.with(|i| {
            state.with(|s| {
                let list = visible_rows(&i.root, &s.expanded, &s.query);
                Rows {
                    order: list.iter().map(|r| r.id.clone()).collect(),
                    by_id: list.into_iter().map(|r| (r.id.clone(), r)).collect(),
                }
            })
        })
    });
    let tab_stop = Memo::new(move |_| {
        rows.with(|r| {
            state.with(|s| match &s.focused {
                Some(f) if r.by_id.contains_key(f) => Some(f.clone()),
                _ => r.order.first().cloned(),
            })
        })
    });
    let query = Memo::new(move |_| state.with(|s| s.query.clone()));
    let status = move || ctx.notice.get().map_or_else(|| state.with(h::status), str::to_string);

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <div data-scope=h::SCOPE data-part="toolbar">
                <div
                    data-scope=h::SCOPE
                    data-part="search"
                    on:keydown=move |ev| {
                        if let Some(event) = h::search_keydown(&ev.key(), key_mods(&ev)) {
                            ev.prevent_default();
                            ctx.send(event);
                        }
                    }
                >
                    <SearchField
                        label="Search keys and values"
                        placeholder="Search"
                        size=Size::Sm
                        on_input=move |q: String| ctx.send(JsonInspectorEvent::SetQuery(q))
                    />
                </div>
                <span data-scope=h::SCOPE data-part="status" role="status">{status}</span>
                <div data-scope=h::SCOPE data-part="depth" role="group" aria-label="Expand to depth">
                    {h::DEPTHS
                        .into_iter()
                        .map(|(text, depth)| {
                            let name = if depth == usize::MAX { "Expand all".to_string() } else { format!("Expand to level {depth}") };
                            view! {
                                <button
                                    type="button"
                                    data-scope=h::SCOPE
                                    data-part="depth-button"
                                    aria-label=name
                                    on:click=move |_| ctx.send(JsonInspectorEvent::ExpandToDepth(depth))
                                >
                                    {text}
                                </button>
                            }
                        })
                        .collect_view()}
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="depth-button"
                        on:click=move |_| ctx.send(JsonInspectorEvent::ExpandToDepth(0))
                    >
                        "Collapse"
                    </button>
                </div>
            </div>
            {move || match parsed.get() {
                Err(e) => Either::Left(view! {
                    <p data-scope=h::SCOPE data-part="error" role="alert">
                        <Icon name=IconName::CircleAlert size=14 />
                        {h::error_text(&e)}
                    </p>
                }),
                Ok(_) => Either::Right(view! {
                    <ul data-scope=h::SCOPE data-part="tree" role="tree" aria-label=label.clone()>
                        <For
                            each=move || rows.with(|r| r.order.clone())
                            key=String::clone
                            children=move |id| row_view(ctx, rows, tab_stop, query, id)
                        />
                    </ul>
                }),
            }}
        </div>
    }
}

/// Text with search hits wrapped in `<mark>`.
fn marked(text: &str, query: &str) -> impl IntoView + use<> {
    split_matches(text, query)
        .into_iter()
        .map(|(hit, run)| {
            if hit {
                Either::Left(view! { <mark data-scope=h::SCOPE data-part="mark">{run.to_string()}</mark> })
            } else {
                Either::Right(run.to_string())
            }
        })
        .collect_view()
}

#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
fn row_view(
    ctx: Ctx,
    rows: Memo<Rows>,
    tab_stop: Memo<Option<String>>,
    query: Memo<String>,
    id: String,
) -> impl IntoView {
    let row = {
        let id = id.clone();
        Memo::new(move |_| rows.with(|r| r.by_id.get(&id).cloned()))
    };
    let a = move || row.with(|r| r.as_ref().map(|r| h::item_attrs(&ctx.root.get_value(), r)));
    let tabindex = {
        let id = id.clone();
        move || tab_stop.with(|t| h::tabindex(&id, t.as_deref()))
    };
    let (focused, keyed, clicked) = (id.clone(), id.clone(), id);
    let content = move || {
        let r = row.get()?;
        let q = query.get();
        let primitive = !matches!(r.kind, JsonKind::Array | JsonKind::Object | JsonKind::Range);
        let key = h::key_text(&r).map(|k| {
            let k = if r.kind == JsonKind::Range { Either::Left(k) } else { Either::Right(marked(&k, &q)) };
            view! {
                <span data-scope=h::SCOPE data-part="key" data-index=r.index.then_some("")>{k}</span>
                <span data-scope=h::SCOPE data-part="colon" aria-hidden="true">":"</span>
            }
        });
        let value = if primitive { Either::Left(marked(&r.text, &q)) } else { Either::Right(r.text.clone()) };
        let (copy_value, copy_path) = (r.id.clone(), r.id.clone());
        let range = r.kind == JsonKind::Range;
        Some(view! {
            <span data-scope=h::SCOPE data-part="toggle" aria-hidden="true">
                {r.expandable.then(|| view! { <Icon name=IconName::ChevronRight size=14 /> })}
            </span>
            {key}
            <span data-scope=h::SCOPE data-part="value" data-type=r.kind.as_str()>{value}</span>
            <span data-scope=h::SCOPE data-part="actions">
                <button
                    type="button"
                    tabindex="-1"
                    data-scope=h::SCOPE
                    data-part="copy"
                    aria-label="Copy value"
                    title="Copy value (Ctrl+C)"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        ctx.send(JsonInspectorEvent::Copy(copy_value.clone(), CopyWhat::Value));
                    }
                >
                    <Icon name=IconName::Copy size=13 />
                </button>
                {(!range).then(|| view! {
                    <button
                        type="button"
                        tabindex="-1"
                        data-scope=h::SCOPE
                        data-part="copy"
                        aria-label="Copy path"
                        title=format!("Copy path {} (Ctrl+Shift+C)", r.path)
                        on:click=move |ev| {
                            ev.stop_propagation();
                            ctx.send(JsonInspectorEvent::Copy(copy_path.clone(), CopyWhat::Path));
                        }
                    >
                        "$."
                    </button>
                })}
            </span>
        })
    };
    view! {
        <li
            role="treeitem"
            id=move || a().map(|a| a.id)
            aria-label=move || a().map(|a| a.aria_label)
            aria-setsize=move || a().map(|a| a.aria_setsize)
            aria-posinset=move || a().map(|a| a.aria_posinset)
            aria-expanded=move || a().and_then(|a| a.aria_expanded).map(|e| e.to_string())
            tabindex=tabindex
            style=move || a().map(|a| a.style)
            data-scope=h::SCOPE
            data-part="item"
            data-state=move || a().and_then(|a| a.data_state)
            data-matched=move || row.with(|r| r.as_ref().is_some_and(|r| r.matched)).then_some("")
            on:click=move |_| {
                if row.with_untracked(|r| r.as_ref().is_some_and(|r| r.expandable)) {
                    ctx.send(JsonInspectorEvent::Toggle(clicked.clone()));
                } else {
                    ctx.send(JsonInspectorEvent::Focus(clicked.clone()));
                }
            }
            on:focus=move |_| ctx.send(JsonInspectorEvent::Focus(focused.clone()))
            on:keydown=move |ev| {
                if ev.alt_key() {
                    return;
                }
                if let Some(event) = h::keydown(&ev.key(), &keyed, key_mods(&ev)) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {content}
        </li>
    }
    // tachys has no typed `aria-level`.
    .add_any_attr(custom_attribute("aria-level", move || a().map(|a| a.aria_level.to_string())))
}
