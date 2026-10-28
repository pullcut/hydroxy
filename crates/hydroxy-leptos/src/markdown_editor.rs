//! `<MarkdownEditor>`: a textarea with a formatting toolbar and Write / Preview tabs.
//! Toolbar buttons and Cmd/Ctrl+B, I, K wrap or prefix the selection.

use leptos::html::Textarea;
use leptos::prelude::*;
use hydroxy_headless::markdown_editor::{self as h, Format};

use crate::markdown::Markdown;
use crate::tabs::{Tabs, TabsContent, TabsItem, TabsList, TabsTrigger};

/// Markdown text field with a toolbar and a rendered preview.
#[component]
pub fn MarkdownEditor(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_input`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Called with the new text on every change, typed or from the toolbar.
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,
    /// Placeholder text.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Visible text rows. Default 8.
    #[prop(optional, default = 8)]
    rows: u32,
) -> impl IntoView {
    let id = crate::id::next_id(h::SCOPE);
    let text = RwSignal::new(value.get_untracked().unwrap_or(default_value));
    Effect::new(move |_| {
        if let Some(v) = value.get() {
            text.set(v);
        }
    });
    let writing = RwSignal::new(true);
    let area = NodeRef::<Textarea>::new();

    let changed = move |t: String| {
        text.set(t.clone());
        if let Some(cb) = on_input {
            cb.run(t);
        }
    };
    let run = move |format: Format| {
        let Some(el) = area.get_untracked() else { return };
        let pos = |p: Result<Option<u32>, _>| p.ok().flatten().unwrap_or(0) as usize;
        let edit = h::apply(format, &el.value(), pos(el.selection_start()), pos(el.selection_end()));
        // Write the DOM first so the selection lands on the new text.
        el.set_value(&edit.text);
        let _ = el.focus();
        let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(to_u32(edit.start), to_u32(edit.end));
        changed(edit.text);
    };

    let buttons = Format::ALL
        .into_iter()
        .map(|format| {
            view! {
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="action"
                    data-format=format.as_str()
                    aria-label=format.label()
                    title=format.label()
                    // Keep focus (and the selection) in the textarea.
                    on:mousedown=|ev| ev.prevent_default()
                    on:click=move |_| run(format)
                >
                    {format.glyph()}
                </button>
            }
        })
        .collect_view();

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <label data-scope=h::SCOPE data-part="label" for=id.clone()>{label}</label>
            <Tabs
                tabs=vec![TabsItem::new("write"), TabsItem::new("preview")]
                default_value="write".to_string()
                on_value_change=move |v: String| writing.set(v == "write")
            >
                <div data-scope=h::SCOPE data-part="bar">
                    <TabsList>
                        <TabsTrigger value="write">"Write"</TabsTrigger>
                        <TabsTrigger value="preview">"Preview"</TabsTrigger>
                    </TabsList>
                    <div
                        role="toolbar"
                        aria-label="Formatting"
                        aria-controls=id.clone()
                        data-scope=h::SCOPE
                        data-part="toolbar"
                        hidden=move || !writing.get()
                    >
                        {buttons}
                    </div>
                </div>
                <TabsContent value="write">
                    <textarea
                        node_ref=area
                        id=id
                        data-scope=h::SCOPE
                        data-part="control"
                        rows=rows
                        placeholder=placeholder
                        name=name
                        prop:value=move || text.get()
                        on:input=move |ev| changed(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if let Some(format) = h::shortcut(&ev.key(), ev.meta_key() || ev.ctrl_key()) {
                                ev.prevent_default();
                                run(format);
                            }
                        }
                    ></textarea>
                </TabsContent>
                <TabsContent value="preview">
                    <div data-scope=h::SCOPE data-part="preview">
                        <Markdown source=text />
                    </div>
                </TabsContent>
            </Tabs>
        </div>
    }
}
