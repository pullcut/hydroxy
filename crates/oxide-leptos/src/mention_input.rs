//! `<MentionInput>`: a text field where `@`, `/` or any configured character opens a
//! filtered list, and a picked row becomes an atomic mention. The field is a native
//! `<textarea>`; a mirror behind it draws the mention pills and anchors the list at the
//! trigger.

use leptos::html::Textarea;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::mention_input::{
    self as h, KeyPress, MentionInputCommand, MentionInputEvent, MentionInputMachine, MentionInputState, Segment,
};
use oxide_statechart::Machine;

pub use oxide_headless::mention_input::{Mention, MentionItem, MentionSource, MentionValue, TriggerSpec};

/// A text field with `@` mentions and `/` commands (or any triggers in `sources`). Typing a
/// trigger at the start of a word lists that source's items, filtered as you type; Up and
/// Down move, Enter or Tab inserts, Escape closes. An inserted mention is one token: it is
/// deleted whole and the caret steps over it. `on_change` gets the text plus every
/// mention's id and range.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn MentionInput(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Triggers and the items each one lists.
    #[prop(into)]
    sources: Signal<Vec<MentionSource>>,
    /// Initial text and mentions.
    #[prop(optional, into)]
    default_value: Option<MentionValue>,
    /// Called with the new value after every change.
    #[prop(optional, into)]
    on_change: Option<Callback<MentionValue>>,
    /// Placeholder text.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Visible text rows.
    #[prop(default = 3)]
    rows: u32,
    /// Greyed out and not editable.
    #[prop(optional)]
    disabled: bool,
) -> impl IntoView {
    let root = crate::id::next_id(h::SCOPE);
    let ids = h::ids(&root);
    let state = RwSignal::new(default_value.map(MentionInputState::from_value).unwrap_or_default());
    let area = NodeRef::<Textarea>::new();
    let pos = RwSignal::new((0, 0));

    // Write the machine's text and selection into the textarea.
    let place = move |start: usize, end: usize| {
        let Some(el) = area.get_untracked() else { return };
        let text = state.with_untracked(|s| s.text.clone());
        el.set_value(&text);
        let a = u32::try_from(h::char_to_utf16(&text, start)).unwrap_or(u32::MAX);
        let b = u32::try_from(h::char_to_utf16(&text, end)).unwrap_or(u32::MAX);
        let _ = el.set_selection_range(a, b);
    };
    let send = move |event: MentionInputEvent| {
        let Some(step) = state.try_with_untracked(|s| {
            sources.with_untracked(|src| MentionInputMachine::transition(s, event, &h::machine_input(src)))
        }) else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                MentionInputCommand::Changed(value) => {
                    if let Some(cb) = on_change {
                        cb.run(value);
                    }
                }
                MentionInputCommand::PlaceCaret { start, end } => place(start, end),
            }
        }
    };

    let list = Memo::new(move |_| state.with(|s| sources.with(|src| h::rows(s, src))));
    let open = move || state.with(MentionInputState::is_open);

    let read_input = move || {
        let Some(el) = area.get_untracked() else { return };
        let text = el.value();
        let units = el.selection_end().ok().flatten().map_or(usize::MAX, |n| n as usize);
        send(MentionInputEvent::SetText { caret: h::utf16_to_char(&text, units), text });
    };
    let read_select = move || {
        let Some(el) = area.get_untracked() else { return };
        let text = el.value();
        // A pending edit arrives through `input`; only report pure caret moves here.
        if state.with_untracked(|s| s.text != text) {
            return;
        }
        let at = |n: Option<u32>| h::utf16_to_char(&text, n.map_or(0, |n| n as usize));
        let (start, end) = (at(el.selection_start().ok().flatten()), at(el.selection_end().ok().flatten()));
        send(MentionInputEvent::Select { start, end });
    };

    // Place the list under the trigger: the anchor span's spot in the mirror, less its scroll.
    let measure = {
        let (anchor_id, mirror_id) = (ids.anchor.clone(), ids.mirror.clone());
        move || {
            let doc = document();
            let (Some(anchor), Some(mirror)) = (doc.get_element_by_id(&anchor_id), doc.get_element_by_id(&mirror_id))
            else {
                return;
            };
            let anchor: web_sys::HtmlElement = anchor.unchecked_into();
            let p = (anchor.offset_left(), anchor.offset_top() + anchor.offset_height() - mirror.scroll_top());
            if pos.get_untracked() != p {
                pos.set(p);
            }
        }
    };
    Effect::new({
        let measure = measure.clone();
        move |_| {
            if open() {
                measure();
            }
        }
    });
    let (scroll_list, scroll_root) = (ids.listbox.clone(), root.clone());
    Effect::new(move |_| {
        if let Some(i) = state.with(|s| s.is_open().then_some(s.active)) {
            crate::combobox::scroll_into_list(&scroll_list, &h::option_id(&scroll_root, i));
        }
    });

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        let press = KeyPress { key: &key, shift: ev.shift_key(), composing: ev.is_composing() };
        let event = list.with_untracked(|r| state.with_untracked(|s| h::input_key(&press, s, r.len(), r.get(s.active))));
        if let Some(e) = event {
            ev.prevent_default();
            send(e);
        }
    };
    let on_scroll = {
        let mirror_id = ids.mirror.clone();
        move |_| {
            let (Some(el), Some(mirror)) = (area.get_untracked(), document().get_element_by_id(&mirror_id)) else {
                return;
            };
            mirror.set_scroll_top(el.scroll_top());
            if open() {
                measure();
            }
        }
    };

    let anchor_id = ids.anchor.clone();
    let active_root = root.clone();
    let option_root = root.clone();
    let list_id = ids.listbox.clone();

    view! {
        <div
            id=root.clone()
            data-scope=h::SCOPE
            data-part="root"
            data-open=move || open().then_some("")
            data-disabled=disabled.then_some("")
        >
            <label id=ids.label.clone() for=ids.input.clone() data-scope=h::SCOPE data-part="label">
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field">
                <div id=ids.mirror.clone() data-scope=h::SCOPE data-part="mirror" aria-hidden="true">
                    {move || {
                        state
                            .with(h::segments)
                            .into_iter()
                            .map(|seg| match seg {
                                Segment::Text(t) => t.into_any(),
                                Segment::Mention(t) => {
                                    view! { <span data-scope=h::SCOPE data-part="mention">{t}</span> }.into_any()
                                }
                                Segment::Anchor => {
                                    view! { <span id=anchor_id.clone() data-scope=h::SCOPE data-part="anchor"></span> }
                                        .into_any()
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <textarea
                    node_ref=area
                    id=ids.input.clone()
                    data-scope=h::SCOPE
                    data-part="input"
                    rows=rows.to_string()
                    role="combobox"
                    aria-autocomplete="list"
                    aria-haspopup="listbox"
                    aria-expanded=move || open().to_string()
                    aria-controls=ids.listbox.clone()
                    aria-activedescendant=move || {
                        let has_rows = list.with(|r| !r.is_empty());
                        (open() && has_rows).then(|| h::option_id(&active_root, state.with(|s| s.active)))
                    }
                    placeholder=placeholder
                    disabled=disabled
                    prop:value=move || state.with(|s| s.text.clone())
                    on:input=move |_| read_input()
                    on:keydown=on_keydown
                    on:keyup=move |ev| {
                        let moves = matches!(
                            ev.key().as_str(),
                            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" | "PageUp" | "PageDown"
                        );
                        if moves {
                            read_select();
                        }
                    }
                    on:pointerup=move |_| read_select()
                    on:select=move |_| read_select()
                    on:blur=move |_| {
                        if open() {
                            send(MentionInputEvent::Dismiss);
                        }
                    }
                    on:scroll=on_scroll
                ></textarea>
                <Show when=open>
                    <div
                        data-scope=h::SCOPE
                        data-part="popover"
                        style=move || {
                            let (x, y) = pos.get();
                            format!("--mention-x: {x}px; --mention-y: {y}px;")
                        }
                    >
                        <ul
                            id=list_id.clone()
                            role="listbox"
                            aria-label=move || {
                                state.with(|s| sources.with(|src| h::open_source(s, src).map(|o| o.name.clone())))
                            }
                            data-scope=h::SCOPE
                            data-part="listbox"
                        >
                            <For
                                each={move || list.get().into_iter().enumerate().collect::<Vec<_>>()}
                                key=|(i, item)| (*i, item.id.clone())
                                children={
                                    let root = option_root.clone();
                                    move |(i, item): (usize, MentionItem)| {
                                        let MentionItem { id, label, description } = item;
                                        let pick = (id, label.clone());
                                        let active = move || state.with(|s| s.active) == i;
                                        let mark = move || state.with(|s| s.trigger.as_ref().map(|t| t.char.to_string()));
                                        view! {
                                            <li
                                                id=h::option_id(&root, i)
                                                role="option"
                                                data-scope=h::SCOPE
                                                data-part="option"
                                                aria-selected=move || active().to_string()
                                                data-highlighted=move || active().then_some("")
                                                on:mousedown=|ev| ev.prevent_default()
                                                on:click=move |_| {
                                                    send(MentionInputEvent::Accept {
                                                        id: pick.0.clone(),
                                                        label: pick.1.clone(),
                                                    });
                                                    if let Some(el) = area.get_untracked() {
                                                        let _ = el.focus();
                                                    }
                                                }
                                            >
                                                <span data-scope=h::SCOPE data-part="option-mark" aria-hidden="true">
                                                    {mark}
                                                </span>
                                                <span data-scope=h::SCOPE data-part="option-label">{label}</span>
                                                {(!description.is_empty())
                                                    .then(|| {
                                                        view! {
                                                            <span data-scope=h::SCOPE data-part="option-description">
                                                                {description}
                                                            </span>
                                                        }
                                                    })}
                                            </li>
                                        }
                                    }
                                }
                            />
                        </ul>
                        <Show when=move || list.with(Vec::is_empty)>
                            <p data-scope=h::SCOPE data-part="empty">{h::EMPTY_TEXT}</p>
                        </Show>
                    </div>
                </Show>
            </div>
            <span id=ids.status.clone() role="status" data-scope=h::SCOPE data-part="status">
                {move || list.with(|r| state.with(|s| h::status_text(s, r.len())))}
            </span>
        </div>
    }
}
