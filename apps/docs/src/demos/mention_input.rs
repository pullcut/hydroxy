use leptos::prelude::*;
use oxide_leptos::mention_input::{Mention, MentionInput, MentionItem, MentionSource, MentionValue};

fn sources() -> Vec<MentionSource> {
    let people = vec![
        MentionItem::new("u_ana", "Ana Lima").description("Design lead"),
        MentionItem::new("u_bo", "Bo Chen").description("Backend"),
        MentionItem::new("u_dana", "Dana Ortiz").description("Product"),
        MentionItem::new("u_joan", "Joan Park").description("Support"),
        MentionItem::new("u_kofi", "Kofi Mensah").description("Data"),
        MentionItem::new("u_mira", "Mira Shah").description("Frontend"),
        MentionItem::new("u_sam", "Sam Okafor").description("QA"),
    ];
    let commands = vec![
        MentionItem::new("assign", "assign").description("Assign to someone"),
        MentionItem::new("due", "due").description("Set a due date"),
        MentionItem::new("label", "label").description("Add a label"),
        MentionItem::new("estimate", "estimate").description("Set points"),
        MentionItem::new("close", "close").description("Close the issue"),
    ];
    vec![MentionSource::new('@', "People", people), MentionSource::new('/', "Commands", commands)]
}

pub fn demo() -> AnyView {
    let start = "Checkout fails on Safari for some cards. @Bo Chen can you look at the logs? ";
    let value = RwSignal::new(MentionValue {
        text: start.into(),
        mentions: vec![Mention { id: "u_bo".into(), label: "Bo Chen".into(), trigger: '@', start: 41, end: 49 }],
    });

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 560px;">
            <MentionInput
                label="Comment"
                sources=sources()
                default_value=value.get_untracked()
                placeholder="Write a comment. @ mentions someone, / runs a command"
                on_change=move |v: MentionValue| value.set(v)
            />
            <div style="display: grid; gap: var(--space-1); font: var(--text-xs); color: var(--text-3);">
                <span>{move || value.with(|v| format!("{} characters", v.text.chars().count()))}</span>
                {move || {
                    value
                        .with(|v| v.mentions.clone())
                        .into_iter()
                        .map(|m| {
                            view! {
                                <code style="font-family: var(--font-mono);">
                                    {format!("{}{} → {} at {}..{}", m.trigger, m.label, m.id, m.start, m.end)}
                                </code>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "mention-input",
    name: "Mention input",
    group: "Forms",
    summary: "A text field where @ lists people and / lists commands, filtered as you type. A picked row becomes a mention that carries an id and is deleted as one piece.",
    demo,
    leptos: include_str!("mention_input.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/mention_input.rs"),
    parts: &[
        &["MentionInput", "root", "Wrapper. data-open while a list shows, data-disabled"],
        &["MentionInput", "label", "Names the field"],
        &["MentionInput", "field", "The bordered box. Holds the mirror, the textarea and the list"],
        &["MentionInput", "mirror", "aria-hidden copy of the text behind the textarea, with the same metrics"],
        &["MentionInput", "mention", "Span in the mirror that draws a mention's pill"],
        &["MentionInput", "anchor", "Empty span at the open trigger. The list is placed under it"],
        &["MentionInput", "input", "Native <textarea role=combobox>. Editing, selection and IME stay native"],
        &["MentionInput", "popover", "The floating list. Positioned from --mention-x and --mention-y"],
        &["MentionInput", "listbox", "role=listbox, named after the source (People, Commands)"],
        &["MentionInput", "option", "role=option. data-highlighted on the active row"],
        &["MentionInput", "option-mark, option-label, option-description", "Trigger badge, name and secondary text"],
        &["MentionInput", "empty", "Shown when nothing matches"],
        &["MentionInput", "status", "Visually hidden role=status with the result count"],
    ],
    keys: &[
        &["@ or /", "At the start of a word: open that list. Keep typing to filter"],
        &["ArrowDown / ArrowUp", "With a list open: move the highlight, wrapping"],
        &["Enter / Tab", "With a list open: insert the highlighted row as a mention"],
        &["Escape", "Close the list. It stays closed until you leave that trigger"],
        &["Backspace / Delete", "Next to a mention: remove the whole mention"],
        &["ArrowLeft / ArrowRight", "Step over a mention in one press"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["sources", "Vec<MentionSource>", "Required. Each trigger character with its list name and items. MentionSource::new('@', \"People\", items); .start_only() for chat-style /commands"],
        &["default_value", "MentionValue", "Initial text and mentions. Mentions that do not match the text are dropped"],
        &["on_change", "Callback<MentionValue>", "The text plus each mention's id, label, trigger and char range"],
        &["placeholder", "String", "Hint shown when empty"],
        &["rows", "u32", "Visible text rows. Default 3. The field can be resized taller"],
        &["disabled", "bool", "Greyed out and not editable"],
    ],
};
