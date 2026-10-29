//! `EmojiPicker`: search, category tabs, a keyboard-navigable grid, a skin-tone
//! choice and a recently-used tab. The host stores the recent list and the tone
//! (both come back through callbacks), so they survive reloads.

use dioxus::prelude::*;
use hydroxy_headless::emoji_picker::{
    self as h, Cell, EmojiPickerCommand, EmojiPickerEvent, EmojiPickerInput, EmojiPickerMachine, EmojiPickerState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::{focus_first, focus_id};
use crate::modifiers::mods;
use crate::search_field::{SearchField, Size};

pub use hydroxy_headless::emoji_picker::SkinTone;

#[derive(Clone, Copy)]
struct Ctx {
    ids: CopyValue<h::Ids>,
    state: Signal<EmojiPickerState>,
    input: Memo<EmojiPickerInput>,
    on_select: Option<EventHandler<(String, String)>>,
    on_recent_change: Option<EventHandler<Vec<String>>>,
    on_skin_tone_change: Option<EventHandler<SkinTone>>,
}

impl Ctx {
    fn send(self, event: EmojiPickerEvent) {
        let ids = self.ids.peek().clone();
        for command in drive::<EmojiPickerMachine>(self.state, self.input, event) {
            match command {
                EmojiPickerCommand::Select(Cell { emoji, name }) => {
                    if let Some(handler) = self.on_select {
                        handler.call((emoji, name.to_string()));
                    }
                }
                EmojiPickerCommand::RecentChange(list) => {
                    if let Some(handler) = self.on_recent_change {
                        handler.call(list);
                    }
                }
                EmojiPickerCommand::ToneChange(tone) => {
                    if let Some(handler) = self.on_skin_tone_change {
                        handler.call(tone);
                    }
                }
                // Eval runs after this render's DOM update, so new cells exist by then.
                EmojiPickerCommand::FocusCell(i) => focus_id(&h::cell_id(&ids, i)),
                EmojiPickerCommand::FocusSearch => focus_first(&ids.search, "input"),
                EmojiPickerCommand::FocusTab(section) => focus_id(&h::tab_id(&ids, section)),
            }
        }
    }
}

/// An emoji picker.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn EmojiPicker(
    /// Called with the emoji (in the chosen skin tone) and its name.
    on_select: Option<EventHandler<(String, String)>>,
    /// Recently used emoji, newest first. Store what `on_recent_change` gives you.
    #[props(default)]
    recent: ReadSignal<Vec<String>>,
    /// Called with the new recent list after each pick.
    on_recent_change: Option<EventHandler<Vec<String>>>,
    /// Controlled skin tone. Pair with `on_skin_tone_change`.
    #[props(default)]
    skin_tone: ReadSignal<Option<SkinTone>>,
    /// Initial skin tone when uncontrolled.
    #[props(default)]
    default_skin_tone: SkinTone,
    /// Called when a skin tone is chosen.
    on_skin_tone_change: Option<EventHandler<SkinTone>>,
    /// Emoji per row. Default 9.
    #[props(default = 9)]
    columns: usize,
    /// How many recent emoji to keep. Default 27 (three rows).
    #[props(default = 27)]
    max_recent: usize,
) -> Element {
    let input = use_memo(move || EmojiPickerInput {
        recent: recent(),
        max_recent,
        columns,
        skin_tone: skin_tone(),
        default_skin_tone,
    });
    let mut state = use_signal(|| EmojiPickerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = EmojiPickerMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let ctx = use_hook(|| Ctx {
        ids: CopyValue::new(ids.clone()),
        state,
        input,
        on_select,
        on_recent_change,
        on_skin_tone_change,
    });

    // What the grid lists depends on the query, tab and tone, not on the active cell.
    let view_key = use_memo(move || {
        let s = state.read();
        (s.query.clone(), s.section, s.tone)
    });
    let cells = use_memo(move || {
        let (query, section, tone) = view_key();
        let probe = EmojiPickerState { query, section, active: 0, tone };
        h::cells(&probe, &input.read())
    });
    // A new list starts at the top.
    let body_id = format!("{}-body", ids.root);
    use_effect({
        let body_id = body_id.clone();
        move || {
            view_key.read();
            let _ = document::eval(&format!(
                "const b = document.getElementById({body_id}); if (b) b.scrollTop = 0;",
                body_id = crate::js::js_string(&body_id)
            ));
        }
    });

    let s = state.read();
    let searching = !s.query.trim().is_empty();
    let active = s.active;
    let tone = s.tone.value;
    let list = cells.read();
    let cols = columns.max(1);
    let status = h::result_status(&s.query, list.len());
    let (preview_emoji, preview_name) = list
        .get(active)
        .map_or_else(|| (String::new(), "Pick an emoji".to_string()), |c| (c.emoji.clone(), c.name.to_string()));
    let sections = h::sections(&input.read());
    let current_section = s.section;
    let query = s.query.clone();
    let rows: Vec<(usize, Vec<(usize, Cell)>)> = list
        .chunks(cols)
        .enumerate()
        .map(|(r, row)| (r, row.iter().cloned().enumerate().map(|(j, c)| (r * cols + j, c)).collect()))
        .collect();
    let empty = list.is_empty();
    drop(list);
    drop(s);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", style: "--emoji-columns: {columns};",
            div {
                "data-scope": h::SCOPE,
                "data-part": "search",
                id: ids.search.clone(),
                onkeydown: move |e| {
                    if let Some(event) = h::search_keydown(&e.key().to_string()) {
                        e.prevent_default();
                        ctx.send(event);
                    }
                },
                SearchField {
                    label: "Search emoji",
                    placeholder: "Search emoji",
                    size: Size::Sm,
                    value: Some(query),
                    on_input: move |q: String| ctx.send(EmojiPickerEvent::Query(q)),
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "tabs", role: "tablist", "aria-label": "Emoji categories",
                for section in sections {
                    button {
                        key: "{section.as_str()}",
                        r#type: "button",
                        role: "tab",
                        id: h::tab_id(&ids, section),
                        "aria-selected": (!searching && section == current_section).to_string(),
                        "aria-controls": ids.grid.clone(),
                        "aria-label": section.label(),
                        title: section.label(),
                        tabindex: if section == current_section { "0" } else { "-1" },
                        "data-scope": h::SCOPE,
                        "data-part": "tab",
                        "data-state": if !searching && section == current_section { "active" } else { "inactive" },
                        onclick: move |_| ctx.send(EmojiPickerEvent::ShowSection(section)),
                        onkeydown: move |e| {
                            if let Some(event) = h::tab_keydown(&e.key().to_string()) {
                                e.prevent_default();
                                ctx.send(event);
                            }
                        },
                        span { "aria-hidden": "true", {section.icon()} }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "body", id: body_id,
                div {
                    role: "grid",
                    id: ids.grid.clone(),
                    "aria-label": h::grid_label(&state.read()),
                    "aria-colcount": "{columns}",
                    "data-scope": h::SCOPE,
                    "data-part": "grid",
                    for (r, row) in rows {
                        div { key: "{r}", role: "row", "data-scope": h::SCOPE, "data-part": "row",
                            for (i, cell) in row {
                                button {
                                    key: "{i}-{cell.emoji}",
                                    r#type: "button",
                                    role: "gridcell",
                                    id: h::cell_id(&ids, i),
                                    tabindex: if i == active { "0" } else { "-1" },
                                    "aria-label": cell.name,
                                    "data-scope": h::SCOPE,
                                    "data-part": "cell",
                                    "data-active": (i == active).then_some(""),
                                    onclick: move |_| ctx.send(EmojiPickerEvent::Pick(i)),
                                    onfocus: move |_| ctx.send(EmojiPickerEvent::Hover(i)),
                                    onpointerenter: move |_| ctx.send(EmojiPickerEvent::Hover(i)),
                                    onkeydown: move |e| {
                                        let m = e.modifiers();
                                        if let Some(event) = h::grid_keydown(&e.key().to_string(), mods(m)) {
                                            e.prevent_default();
                                            ctx.send(event);
                                        }
                                    },
                                    "{cell.emoji}"
                                }
                            }
                        }
                    }
                }
                if empty {
                    p { "data-scope": h::SCOPE, "data-part": "empty",
                        if searching { "No emoji found. Try another word." } else { "Nothing here yet." }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "footer",
                span { "data-scope": h::SCOPE, "data-part": "preview", "aria-hidden": "true",
                    span { "data-scope": h::SCOPE, "data-part": "preview-emoji", "{preview_emoji}" }
                    span { "data-scope": h::SCOPE, "data-part": "preview-name", "{preview_name}" }
                }
                div { "data-scope": h::SCOPE, "data-part": "tones", role: "radiogroup", "aria-label": "Skin tone",
                    for t in SkinTone::ALL {
                        button {
                            key: "{t.as_str()}",
                            r#type: "button",
                            role: "radio",
                            id: h::tone_id(&ids, t),
                            "aria-checked": (t == tone).to_string(),
                            "aria-label": t.label(),
                            title: t.label(),
                            tabindex: if t == tone { "0" } else { "-1" },
                            "data-scope": h::SCOPE,
                            "data-part": "tone",
                            onclick: move |_| ctx.send(EmojiPickerEvent::Tone(t)),
                            onkeydown: move |e| {
                                if let Some(next) = h::tone_keydown(&e.key().to_string(), t) {
                                    e.prevent_default();
                                    ctx.send(EmojiPickerEvent::Tone(next));
                                    focus_id(&h::tone_id(&ctx.ids.peek(), next));
                                }
                            },
                            {h::tone_swatch(t)}
                        }
                    }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "status", role: "status", "{status}" }
        }
    }
}
