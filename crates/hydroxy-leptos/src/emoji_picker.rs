//! `<EmojiPicker>`: search, category tabs, a keyboard-navigable grid, a skin-tone
//! choice and a recently-used tab. The host stores the recent list and the tone
//! (both come back through callbacks), so they survive reloads.

use hydroxy_headless::emoji_picker::{
    self as h, Cell, EmojiPickerCommand, EmojiPickerEvent, EmojiPickerInput, EmojiPickerMachine, EmojiPickerState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::focus::focus_id;
use crate::search_field::{SearchField, Size};

pub use hydroxy_headless::emoji_picker::SkinTone;

#[derive(Clone, Copy)]
struct Ctx {
    ids: StoredValue<h::Ids>,
    state: RwSignal<EmojiPickerState>,
    input: Memo<EmojiPickerInput>,
    on_select: Option<Callback<(String, String)>>,
    on_recent_change: Option<Callback<Vec<String>>>,
    on_skin_tone_change: Option<Callback<SkinTone>>,
}

impl Ctx {
    fn send(self, event: EmojiPickerEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| EmojiPickerMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && self.state.with_untracked(|s| *s != next)
        {
            self.state.set(next);
        }
        let ids = self.ids.get_value();
        for command in step.commands {
            match command {
                EmojiPickerCommand::Select(Cell { emoji, name }) => {
                    if let Some(cb) = self.on_select {
                        cb.run((emoji, name.to_string()));
                    }
                }
                EmojiPickerCommand::RecentChange(list) => {
                    if let Some(cb) = self.on_recent_change {
                        cb.run(list);
                    }
                }
                EmojiPickerCommand::ToneChange(tone) => {
                    if let Some(cb) = self.on_skin_tone_change {
                        cb.run(tone);
                    }
                }
                EmojiPickerCommand::FocusCell(i) => focus_id(&h::cell_id(&ids, i)),
                EmojiPickerCommand::FocusSearch => focus_search(&ids.search),
                // The tab exists before and after the switch, so it can take focus now.
                EmojiPickerCommand::FocusTab(section) => focus_id(&h::tab_id(&ids, section)),
            }
        }
    }
}

fn focus_search(wrap: &str) {
    if let Ok(Some(el)) = document().query_selector(&format!("#{wrap} input")) {
        let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
    }
}

/// Focus the search field of the picker inside the element with this id. For
/// hosts that open the picker in a popover.
pub fn focus_picker_search(container_id: &str) {
    if let Ok(Some(el)) = document().query_selector(&format!("#{container_id} [data-scope=\"emoji-picker\"] input")) {
        let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
    }
}

/// An emoji picker.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn EmojiPicker(
    /// Called with the emoji (in the chosen skin tone) and its name.
    #[prop(optional, into)]
    on_select: Option<Callback<(String, String)>>,
    /// Recently used emoji, newest first. Store what `on_recent_change` gives you.
    #[prop(optional, into)]
    recent: Signal<Vec<String>>,
    /// Called with the new recent list after each pick.
    #[prop(optional, into)]
    on_recent_change: Option<Callback<Vec<String>>>,
    /// Controlled skin tone. Pair with `on_skin_tone_change`.
    #[prop(optional, into)]
    skin_tone: MaybeProp<SkinTone>,
    /// Initial skin tone when uncontrolled.
    #[prop(optional)]
    default_skin_tone: SkinTone,
    /// Called when a skin tone is chosen.
    #[prop(optional, into)]
    on_skin_tone_change: Option<Callback<SkinTone>>,
    /// Emoji per row. Default 9.
    #[prop(default = 9)]
    columns: usize,
    /// How many recent emoji to keep. Default 27 (three rows).
    #[prop(default = 27)]
    max_recent: usize,
) -> impl IntoView {
    let input = Memo::new(move |_| EmojiPickerInput {
        recent: recent.get(),
        max_recent,
        columns,
        skin_tone: skin_tone.get(),
        default_skin_tone,
    });
    let state = RwSignal::new(input.with_untracked(EmojiPickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| EmojiPickerMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let ctx =
        Ctx { ids: StoredValue::new(ids.clone()), state, input, on_select, on_recent_change, on_skin_tone_change };

    // What the grid lists depends on the query, tab and tone, not on the active cell.
    let view_key = Memo::new(move |_| state.with(|s| (s.query.clone(), s.section, s.tone)));
    let cells = Memo::new(move |_| {
        let (query, section, tone) = view_key.get();
        let probe = EmojiPickerState { query, section, active: 0, tone, controlled_tone: false };
        input.with(|i| h::cells(&probe, i))
    });
    let rows = Memo::new(move |_| {
        let cols = columns.max(1);
        cells.with(|c| {
            c.chunks(cols)
                .enumerate()
                .map(|(r, row)| {
                    (r, row.iter().cloned().enumerate().map(|(j, cell)| (r * cols + j, cell)).collect::<Vec<_>>())
                })
                .collect::<Vec<_>>()
        })
    });
    let active = Memo::new(move |_| state.with(|s| s.active));
    let searching = Memo::new(move |_| state.with(|s| !s.query.trim().is_empty()));
    let status = move || state.with(|s| h::result_status(&s.query, cells.with(Vec::len)));
    let preview = move || {
        cells
            .with(|c| c.get(active.get()).cloned())
            .map_or_else(|| (String::new(), "Pick an emoji".to_string()), |c| (c.emoji, c.name.to_string()))
    };
    // A new list starts at the top.
    let body = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        view_key.track();
        if let Some(el) = body.get_untracked() {
            el.set_scroll_top(0);
        }
    });

    let tabs = {
        let ids = ids.clone();
        move || {
            input.with(h::sections).into_iter().map(|section| {
                let selected = move || !searching.get() && state.with(|s| s.section == section);
                view! {
                    <button
                        type="button"
                        role="tab"
                        id=h::tab_id(&ids, section)
                        aria-selected=move || selected().to_string()
                        aria-controls=ids.grid.clone()
                        aria-label=section.label()
                        title=section.label()
                        tabindex=move || if selected() || (searching.get() && section == state.with(|s| s.section)) { 0 } else { -1 }
                        data-scope=h::SCOPE
                        data-part="tab"
                        data-state=move || if selected() { "active" } else { "inactive" }
                        on:click=move |_| ctx.send(EmojiPickerEvent::ShowSection(section))
                        on:keydown=move |ev| {
                            if let Some(event) = h::tab_keydown(&ev.key()) {
                                ev.prevent_default();
                                ctx.send(event);
                            }
                        }
                    >
                        <span aria-hidden="true">{section.icon()}</span>
                    </button>
                }
            }).collect_view()
        }
    };

    let row_view = {
        let ids = ids.clone();
        move |(_, row): (usize, Vec<(usize, Cell)>)| {
            let cells = row
                .into_iter()
                .map(|(i, cell)| {
                    view! {
                        <button
                            type="button"
                            role="gridcell"
                            id=h::cell_id(&ids, i)
                            tabindex=move || if active.get() == i { 0 } else { -1 }
                            aria-label=cell.name
                            data-scope=h::SCOPE
                            data-part="cell"
                            data-active=move || (active.get() == i).then_some("")
                            on:click=move |_| ctx.send(EmojiPickerEvent::Pick(i))
                            on:focus=move |_| ctx.send(EmojiPickerEvent::Hover(i))
                            on:pointerenter=move |_| ctx.send(EmojiPickerEvent::Hover(i))
                            on:keydown=move |ev| {
                                if let Some(event) = h::grid_keydown(&ev.key(), ev.ctrl_key() || ev.meta_key()) {
                                    ev.prevent_default();
                                    ctx.send(event);
                                }
                            }
                        >
                            {cell.emoji}
                        </button>
                    }
                })
                .collect_view();
            view! { <div role="row" data-scope=h::SCOPE data-part="row">{cells}</div> }
        }
    };

    let tones = {
        let ids = ids.clone();
        h::SkinTone::ALL
            .into_iter()
            .map(|tone| {
                let checked = move || state.with(|s| s.tone == tone);
                view! {
                    <button
                        type="button"
                        role="radio"
                        id=h::tone_id(&ids, tone)
                        aria-checked=move || checked().to_string()
                        aria-label=tone.label()
                        title=tone.label()
                        tabindex=move || if checked() { 0 } else { -1 }
                        data-scope=h::SCOPE
                        data-part="tone"
                        on:click=move |_| ctx.send(EmojiPickerEvent::Tone(tone))
                        on:keydown=move |ev| {
                            if let Some(next) = h::tone_keydown(&ev.key(), tone) {
                                ev.prevent_default();
                                ctx.send(EmojiPickerEvent::Tone(next));
                                focus_id(&h::tone_id(&ctx.ids.get_value(), next));
                            }
                        }
                    >
                        {h::tone_swatch(tone)}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" style=format!("--emoji-columns: {columns};")>
            <div
                data-scope=h::SCOPE
                data-part="search"
                id=ids.search.clone()
                on:keydown=move |ev| {
                    if let Some(event) = h::search_keydown(&ev.key()) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
            >
                <SearchField
                    label="Search emoji"
                    placeholder="Search emoji"
                    size=Size::Sm
                    value=Signal::derive(move || Some(state.with(|s| s.query.clone())))
                    on_input=move |q: String| ctx.send(EmojiPickerEvent::Query(q))
                />
            </div>
            <div data-scope=h::SCOPE data-part="tabs" role="tablist" aria-label="Emoji categories">{tabs}</div>
            <div data-scope=h::SCOPE data-part="body" node_ref=body>
                <div
                    role="grid"
                    id=ids.grid.clone()
                    aria-label=move || state.with(h::grid_label)
                    aria-colcount=columns
                    data-scope=h::SCOPE
                    data-part="grid"
                >
                    <For each=move || rows.get() key=|(r, row)| (*r, row.iter().map(|(_, c)| c.emoji.as_str()).collect::<String>()) children=row_view />
                </div>
                {move || cells.with(Vec::is_empty).then(|| view! {
                    <p data-scope=h::SCOPE data-part="empty">
                        {move || if searching.get() { "No emoji found. Try another word." } else { "Nothing here yet." }}
                    </p>
                })}
            </div>
            <div data-scope=h::SCOPE data-part="footer">
                <span data-scope=h::SCOPE data-part="preview" aria-hidden="true">
                    <span data-scope=h::SCOPE data-part="preview-emoji">{move || preview().0}</span>
                    <span data-scope=h::SCOPE data-part="preview-name">{move || preview().1}</span>
                </span>
                <div data-scope=h::SCOPE data-part="tones" role="radiogroup" aria-label="Skin tone">{tones}</div>
            </div>
            <span data-scope=h::SCOPE data-part="status" role="status">{status}</span>
        </div>
    }
}
