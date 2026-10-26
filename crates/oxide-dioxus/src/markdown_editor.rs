//! `MarkdownEditor`: a textarea with a formatting toolbar and Write / Preview tabs.
//! Toolbar buttons and Cmd/Ctrl+B, I, K wrap or prefix the selection.
//!
//! The selection is read and written through `document::eval`, so this works on web
//! and in the desktop webview alike.

use dioxus::prelude::*;
use oxide_headless::markdown_editor::{self as h, Format};

use crate::markdown::Markdown;
use crate::tabs::{Tabs, TabsContent, TabsItem, TabsList, TabsTrigger};

/// Markdown text field with a toolbar and a rendered preview.
#[component]
pub fn MarkdownEditor(
    /// Visible label.
    label: String,
    /// Controlled value. Pair with `on_input`.
    value: Option<String>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Called with the new text on every change, typed or from the toolbar.
    on_input: Option<EventHandler<String>>,
    /// Placeholder text.
    placeholder: Option<String>,
    /// Form field name.
    name: Option<String>,
    /// Visible text rows. Default 8.
    #[props(default = 8)]
    rows: u32,
) -> Element {
    let id = use_hook(|| CopyValue::new(oxide_headless::next_id(h::SCOPE)));
    let mut text = use_signal(|| default_value.clone());
    let mut writing = use_signal(|| true);
    let current = value.clone().unwrap_or_else(&*text);

    let mut changed = move |t: String| {
        text.set(t.clone());
        if let Some(handler) = on_input {
            handler.call(t);
        }
    };
    let run = move |format: Format| {
        let id = id.cloned();
        spawn(async move {
            // `{id:?}` quotes the id as a JS string. Ids from `next_id` are plain ASCII.
            let mut js = document::eval(&format!(
                "const el = document.getElementById({id:?});
                if (el) {{
                    dioxus.send([el.selectionStart, el.selectionEnd, el.value]);
                    const [t, s, e] = await dioxus.recv();
                    el.value = t;
                    el.focus();
                    el.setSelectionRange(s, e);
                }}"
            ));
            let Ok((start, end, value)) = js.recv::<(usize, usize, String)>().await else { return };
            let edit = h::apply(format, &value, start, end);
            let _ = js.send((edit.text.clone(), edit.start, edit.end));
            changed(edit.text);
        });
    };

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            label { "data-scope": h::SCOPE, "data-part": "label", r#for: id.cloned(), "{label}" }
            Tabs {
                tabs: vec![TabsItem::new("write"), TabsItem::new("preview")],
                default_value: "write".to_string(),
                on_value_change: move |v: String| writing.set(v == "write"),
                div { "data-scope": h::SCOPE, "data-part": "bar",
                    TabsList {
                        TabsTrigger { value: "write", "Write" }
                        TabsTrigger { value: "preview", "Preview" }
                    }
                    div {
                        role: "toolbar",
                        "aria-label": "Formatting",
                        "aria-controls": id.cloned(),
                        "data-scope": h::SCOPE,
                        "data-part": "toolbar",
                        hidden: !writing(),
                        for format in Format::ALL {
                            button {
                                r#type: "button",
                                "data-scope": h::SCOPE,
                                "data-part": "action",
                                "data-format": format.as_str(),
                                "aria-label": format.label(),
                                title: format.label(),
                                // Keep focus (and the selection) in the textarea.
                                onmousedown: |e| e.prevent_default(),
                                onclick: move |_| run(format),
                                "{format.glyph()}"
                            }
                        }
                    }
                }
                TabsContent { value: "write",
                    textarea {
                        id: id.cloned(),
                        "data-scope": h::SCOPE,
                        "data-part": "control",
                        rows: rows,
                        placeholder,
                        name,
                        value: "{current}",
                        oninput: move |e| changed(e.value()),
                        onkeydown: move |e| {
                            let m = e.modifiers();
                            if let Some(format) = h::shortcut(&e.key().to_string(), m.meta() || m.ctrl()) {
                                e.prevent_default();
                                run(format);
                            }
                        },
                    }
                }
                TabsContent { value: "preview",
                    div { "data-scope": h::SCOPE, "data-part": "preview",
                        Markdown { source: current.clone() }
                    }
                }
            }
        }
    }
}
