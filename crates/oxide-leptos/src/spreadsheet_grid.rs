//! `<SpreadsheetGrid>`: an editable sheet with A1 columns, row numbers,
//! range selection, formulas, TSV copy and paste, column resizing, and row
//! windowing for large sheets.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use leptos::either::Either;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::spreadsheet_grid::{
    self as h, Mods, SpreadsheetGridCommand as Cmd, SpreadsheetGridEvent as Ev, SpreadsheetGridInput,
    SpreadsheetGridMachine, SpreadsheetGridState,
};
use oxide_statechart::Machine;

use crate::focus::focus_id;

pub use oxide_headless::spreadsheet_grid::{Cell, CellChange, cell_name, cells_from_a1};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<SpreadsheetGridState>,
    input: StoredValue<SpreadsheetGridInput>,
    viewport: NodeRef<Div>,
    on_change: Option<Callback<Vec<CellChange>>>,
}

impl Ctx {
    fn send(self, event: Ev) {
        let Some(step) = self
            .input
            .try_with_value(|i| self.state.try_with_untracked(|s| SpreadsheetGridMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                Cmd::FocusGrid => focus_id(&h::viewport_id(&self.id.get_value())),
                Cmd::FocusEditor => {
                    // The editor mounts when the state settles; focus it then, caret at the end.
                    let editor = h::editor_id(&self.id.get_value());
                    set_timeout(
                        move || {
                            if let Some(input) =
                                document().get_element_by_id(&editor).and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                            {
                                let _ = input.focus();
                                let _ = input.set_selection_range(u32::MAX, u32::MAX);
                            }
                        },
                        Duration::ZERO,
                    );
                }
                Cmd::ScrollTo { top, left } => {
                    if let Some(el) = self.viewport.get_untracked() {
                        el.scroll_to_with_x_and_y(left, top);
                    }
                }
                Cmd::Change(changes) => {
                    if let Some(cb) = self.on_change {
                        cb.run(changes);
                    }
                }
            }
        }
    }

    /// Report the viewport's scroll offsets and size.
    fn measure(self) {
        if let Some(el) = self.viewport.get_untracked() {
            self.send(Ev::Scroll {
                top: f64::from(el.scroll_top()),
                left: f64::from(el.scroll_left()),
                width: f64::from(el.client_width()),
                height: f64::from(el.client_height()),
            });
        }
    }

    fn editing(self) -> bool {
        self.state.with_untracked(|s| s.editing.is_some())
    }
}

/// An editable spreadsheet. Type a number, text, or a formula starting with
/// `=` (`=SUM(B2:B9)`, `=A1*1.2`). Props are read once, at mount.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn SpreadsheetGrid(
    /// Accessible name for the grid.
    #[prop(into)]
    label: String,
    /// Number of rows. Only the rows in view are rendered, so thousands are fine.
    #[prop(default = 1000)]
    rows: usize,
    /// Number of columns.
    #[prop(default = 10)]
    cols: usize,
    /// Initial cell text by (row, column), 0-based. See [`cells_from_a1`].
    #[prop(optional)]
    default_cells: BTreeMap<Cell, String>,
    /// Initial column width in pixels.
    #[prop(default = 112.0)]
    col_width: f64,
    /// Height of the scrolling area in pixels.
    #[prop(default = 360.0)]
    height: f64,
    /// Called with every batch of changed cells: one edit, a paste, a cut or a clear.
    #[prop(optional, into)]
    on_change: Option<Callback<Vec<CellChange>>>,
) -> impl IntoView {
    let input = SpreadsheetGridInput { rows, cols, default_cells: Arc::new(default_cells), col_width, height };
    let state = RwSignal::new(SpreadsheetGridMachine::initial(&input));
    let viewport_style = h::viewport_style(&input);
    let (rowcount, colcount) = (h::aria_rowcount(&input), h::aria_colcount(&input));
    let root = crate::id::next_id(h::SCOPE);
    let ctx = Ctx {
        id: StoredValue::new(root.clone()),
        state,
        input: StoredValue::new(input),
        viewport: NodeRef::new(),
        on_change,
    };
    Effect::new(move |_| {
        if ctx.viewport.get().is_some() {
            ctx.measure();
        }
    });

    let template = Memo::new(move |_| state.with(h::template_columns));
    let body_style = Memo::new(move |_| ctx.input.with_value(|i| state.with(|s| h::body_style(s, i))));
    let rendered = Memo::new(move |_| ctx.input.with_value(|i| state.with(|s| h::rendered_rows(s, i))));
    let selection = Memo::new(move |_| state.with(SpreadsheetGridState::selection));
    let active_id = {
        let root = root.clone();
        move || state.with(|s| h::cell_id(&root, s.active))
    };

    let headers = (0..cols)
        .map(|c| {
            let attrs = Memo::new(move |_| state.with(|s| h::resize_attrs(s, c)));
            let selected = move || selection.with(|r| (r.left..=r.right).contains(&c)).then_some("");
            view! {
                <div
                    role="columnheader"
                    aria-colindex=h::aria_colindex(c)
                    data-scope=h::SCOPE
                    data-part="column-header"
                    data-selected=selected
                >
                    {h::column_name(c)}
                    <div
                        role="separator"
                        tabindex="0"
                        aria-orientation="vertical"
                        aria-label=move || attrs.with(|a| a.label.clone())
                        aria-valuenow=move || attrs.with(|a| a.valuenow).to_string()
                        aria-valuemin=h::MIN_COL_WIDTH.to_string()
                        aria-valuemax=h::MAX_COL_WIDTH.to_string()
                        data-scope=h::SCOPE
                        data-part="resize-handle"
                        data-dragging=move || attrs.with(|a| a.dragging).then_some("")
                        on:pointerdown=move |ev| {
                            if ev.button() != 0 {
                                return;
                            }
                            ev.prevent_default();
                            ev.stop_propagation();
                            if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                                let _ = el.set_pointer_capture(ev.pointer_id());
                            }
                            ctx.send(Ev::ResizeStart { col: c, x: f64::from(ev.client_x()) });
                        }
                        on:pointermove=move |ev| {
                            if attrs.with_untracked(|a| a.dragging) {
                                ctx.send(Ev::ResizeMove(f64::from(ev.client_x())));
                            }
                        }
                        on:pointerup=move |_| ctx.send(Ev::ResizeEnd)
                        on:pointercancel=move |_| ctx.send(Ev::ResizeEnd)
                        on:keydown=move |ev| {
                            ev.stop_propagation();
                            if let Some(event) = h::resize_keydown(&ev.key(), ev.shift_key(), c) {
                                ev.prevent_default();
                                ctx.send(event);
                            }
                        }
                    ></div>
                </div>
            }
        })
        .collect_view();

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <div data-scope=h::SCOPE data-part="formula-bar">
                <output data-scope=h::SCOPE data-part="name-box" aria-label="Selection">
                    {move || state.with(h::name_box)}
                </output>
                <div data-scope=h::SCOPE data-part="formula" aria-label="Cell contents">
                    {move || state.with(h::formula_text)}
                </div>
            </div>
            <div
                node_ref=ctx.viewport
                id=h::viewport_id(&root)
                role="grid"
                tabindex="0"
                aria-label=label
                aria-multiselectable="true"
                aria-rowcount=rowcount
                aria-colcount=colcount
                aria-activedescendant=active_id
                style=viewport_style
                data-scope=h::SCOPE
                data-part="viewport"
                data-resizing=move || state.with(|s| s.resizing.is_some()).then_some("")
                on:scroll=move |_| ctx.measure()
                on:pointerup=move |_| ctx.send(Ev::PointerUp)
                on:keydown=move |ev| {
                    // Keys in the editor and on resize handles belong to them.
                    if ev.target() != ev.current_target() {
                        return;
                    }
                    let mods = Mods { shift: ev.shift_key(), ctrl: ev.ctrl_key() || ev.meta_key(), alt: ev.alt_key() };
                    if let Some(event) = h::grid_keydown(&ev.key(), mods) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
                on:copy=move |ev| {
                    if ev.target() != ev.current_target() || ctx.editing() {
                        return;
                    }
                    ev.prevent_default();
                    if let Some(data) = ev.clipboard_data() {
                        let _ = data.set_data("text/plain", &state.with_untracked(SpreadsheetGridState::selection_tsv));
                    }
                }
                on:cut=move |ev| {
                    if ev.target() != ev.current_target() || ctx.editing() {
                        return;
                    }
                    ev.prevent_default();
                    if let Some(data) = ev.clipboard_data() {
                        let _ = data.set_data("text/plain", &state.with_untracked(SpreadsheetGridState::selection_tsv));
                    }
                    ctx.send(Ev::Clear);
                }
                on:paste=move |ev| {
                    if ev.target() != ev.current_target() || ctx.editing() {
                        return;
                    }
                    ev.prevent_default();
                    let text = ev.clipboard_data().and_then(|d| d.get_data("text/plain").ok()).unwrap_or_default();
                    ctx.send(Ev::Paste(text));
                }
            >
                <div role="row" aria-rowindex="1" style=move || template.get() data-scope=h::SCOPE data-part="header-row">
                    <div role="columnheader" aria-colindex="1" data-scope=h::SCOPE data-part="corner"></div>
                    {headers}
                </div>
                <div role="rowgroup" style=move || body_style.get() data-scope=h::SCOPE data-part="body">
                    <div
                        aria-hidden="true"
                        data-scope=h::SCOPE
                        data-part="selection"
                        data-range=move || (selection.with(h::Rect::count) > 1).then_some("")
                        style=move || state.with(h::selection_style)
                    ></div>
                    <For each=move || rendered.get() key=|r| *r children=move |r| row_view(ctx, template, selection, cols, r) />
                </div>
            </div>
            <div role="status" aria-live="polite" data-scope=h::SCOPE data-part="status">
                {move || state.with(|s| s.status.clone())}
            </div>
        </div>
    }
}

fn row_view(ctx: Ctx, template: Memo<String>, selection: Memo<h::Rect>, cols: usize, row: usize) -> impl IntoView {
    let style = Memo::new(move |_| template.with(|t| h::row_style(row, t)));
    view! {
        <div role="row" aria-rowindex=h::aria_rowindex(row) style=move || style.get() data-scope=h::SCOPE data-part="row">
            <div
                role="rowheader"
                aria-colindex="1"
                data-scope=h::SCOPE
                data-part="row-header"
                data-selected=move || selection.with(|r| (r.top..=r.bottom).contains(&row)).then_some("")
            >
                {row + 1}
            </div>
            {(0..cols).map(|col| cell_view(ctx, (row, col))).collect_view()}
        </div>
    }
}

fn cell_view(ctx: Ctx, cell: Cell) -> impl IntoView {
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::cell_attrs(&ctx.id.read_value(), s, cell)));
    let editing = Memo::new(move |_| ctx.state.with(|s| s.editing.is_some() && s.active == cell));
    view! {
        <div
            role="gridcell"
            id=h::cell_id(&ctx.id.get_value(), cell)
            aria-colindex=h::aria_colindex(cell.1)
            aria-selected=move || attrs.with(|a| a.selected).to_string()
            data-scope=h::SCOPE
            data-part="cell"
            data-kind=move || attrs.with(|a| a.kind)
            data-active=move || attrs.with(|a| a.active).then_some("")
            data-editing=move || editing.get().then_some("")
            on:pointerdown=move |ev| {
                // Clicks inside the open editor move its caret.
                if ev.button() != 0 || editing.get_untracked() {
                    return;
                }
                ctx.send(Ev::PointerDown { cell, shift: ev.shift_key() });
            }
            on:pointerenter=move |ev| {
                if ev.buttons() & 1 == 1 {
                    ctx.send(Ev::PointerEnter(cell));
                }
            }
            on:dblclick=move |_| {
                if !editing.get_untracked() {
                    ctx.send(Ev::EditAt(cell));
                }
            }
        >
            {move || {
                if editing.get() {
                    let draft = ctx.state.with_untracked(|s| s.editing.clone().unwrap_or_default());
                    Either::Left(view! {
                        <input
                            id=h::editor_id(&ctx.id.get_value())
                            type="text"
                            autocomplete="off"
                            spellcheck="false"
                            aria-label=format!("Edit {}", h::cell_name(cell))
                            data-scope=h::SCOPE
                            data-part="editor"
                            prop:value=draft
                            on:input=move |ev| ctx.send(Ev::EditChange(event_target_value(&ev)))
                            on:keydown=move |ev| {
                                if let Some(event) = h::editor_keydown(&ev.key(), ev.shift_key()) {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    ctx.send(event);
                                }
                            }
                            on:blur=move |_| ctx.send(Ev::Blur)
                        />
                    })
                } else {
                    Either::Right(attrs.with(|a| a.text.clone()))
                }
            }}
        </div>
    }
}
