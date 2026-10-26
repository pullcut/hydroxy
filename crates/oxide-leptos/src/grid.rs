//! `<Grid>`: an ARIA data grid. One cell is in the tab order, arrow keys move
//! between cells, and Enter or F2 edits a cell.

use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::grid::{self as h, GridCommand, GridEvent, GridInput, GridMachine, GridState};
use oxide_statechart::Machine;

use crate::focus::focus_id;

pub use oxide_headless::grid::{GridColumn, GridEdit, GridRow};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<GridState>,
    input: Memo<GridInput>,
    on_edit: Option<Callback<GridEdit>>,
}

impl Ctx {
    fn send(self, event: GridEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| GridMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        self.state.set(step.state);
        for command in step.commands {
            match command {
                GridCommand::FocusCell { row, col } => focus_id(&h::cell_id(&self.id.get_value(), row, col)),
                GridCommand::FocusEditor { row, col } => {
                    // The editor mounts when the state settles; focus it then.
                    let editor = h::editor_id(&self.id.get_value(), row, col);
                    set_timeout(
                        move || {
                            if let Some(input) =
                                document().get_element_by_id(&editor).and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                            {
                                let _ = input.focus();
                                input.select();
                            }
                        },
                        std::time::Duration::ZERO,
                    );
                }
                GridCommand::Edit(edit) => {
                    if let Some(cb) = self.on_edit {
                        cb.run(edit);
                    }
                }
            }
        }
    }
}

/// A grid of text cells. Mark columns editable with [`GridColumn::editable`].
#[component]
pub fn Grid(
    /// Columns, left to right.
    columns: Vec<GridColumn>,
    /// Accessible name for the grid.
    #[prop(into)]
    label: String,
    /// Controlled rows. Pair with `on_edit`.
    #[prop(optional, into)]
    rows: MaybeProp<Vec<GridRow>>,
    /// Initial rows when uncontrolled.
    #[prop(optional, into)]
    default_rows: Vec<GridRow>,
    /// Rows a page jump skips.
    #[prop(default = 10)]
    page_size: usize,
    /// Called when a cell is edited, with the row id, column key, new and old text.
    #[prop(optional, into)]
    on_edit: Option<Callback<GridEdit>>,
) -> impl IntoView {
    let input = Memo::new(move |_| GridInput {
        columns: columns.clone(),
        rows: rows.get(),
        default_rows: default_rows.clone(),
        page_size,
    });
    let state = RwSignal::new(input.with_untracked(GridMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| GridMachine::sync(s, i)));
        state.set(synced);
    });
    let ctx = Ctx { id: StoredValue::new(crate::id::next_id(h::SCOPE)), state, input, on_edit };

    let template = Memo::new(move |_| input.with(h::template_columns));
    let col_count = Memo::new(move |_| input.with(|i| i.columns.len()));
    let row_count = Memo::new(move |_| state.with(|s| s.rows.len()));

    let headers = move || {
        input.with(|i| i.columns.iter().map(|c| c.header.clone()).collect::<Vec<_>>())
            .into_iter()
            .enumerate()
            .map(|(c, header)| {
                view! {
                    <div role="columnheader" aria-colindex=c + 1 data-scope=h::SCOPE data-part="header-cell">
                        {header}
                    </div>
                }
            })
            .collect_view()
    };

    view! {
        <div
            role="grid"
            aria-label=label
            aria-rowcount=move || state.with(h::aria_rowcount)
            aria-colcount=move || col_count.get()
            data-scope=h::SCOPE
            data-part="root"
        >
            <div role="rowgroup" data-scope=h::SCOPE data-part="header">
                <div role="row" aria-rowindex="1" style=move || template.get() data-scope=h::SCOPE data-part="header-row">
                    {headers}
                </div>
            </div>
            <div role="rowgroup" data-scope=h::SCOPE data-part="body">
                <For
                    each=move || 0..row_count.get()
                    key=|r| *r
                    children=move |r| {
                        view! {
                            <div role="row" aria-rowindex=h::aria_rowindex(r) style=move || template.get() data-scope=h::SCOPE data-part="row">
                                <For each=move || 0..col_count.get() key=|c| *c children=move |c| cell_view(ctx, r, c) />
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

fn cell_view(ctx: Ctx, row: usize, col: usize) -> impl IntoView {
    let attrs = Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::cell_attrs(&ctx.id.get_value(), s, i, row, col))));
    let text = Memo::new(move |_| {
        ctx.state.with(|s| s.rows.get(row).and_then(|r| r.cells.get(col)).cloned().unwrap_or_default())
    });
    let editing = Memo::new(move |_| attrs.with(|a| a.editing));
    view! {
        <div
            role="gridcell"
            id=move || attrs.get().id
            tabindex=move || attrs.get().tabindex
            aria-colindex=move || attrs.get().aria_colindex
            aria-readonly=move || attrs.get().readonly.then_some("true")
            data-scope=h::SCOPE
            data-part="cell"
            data-editing=move || editing.get().then_some("")
            on:focus=move |_| ctx.send(GridEvent::Focus { row, col })
            on:dblclick=move |_| ctx.send(GridEvent::StartEditAt { row, col })
            on:keydown=move |ev| {
                // Keys typed in the editor belong to the editor.
                if ev.target() != ev.current_target() {
                    return;
                }
                if let Some(event) = h::cell_keydown(&ev.key(), ev.ctrl_key() || ev.meta_key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {move || {
                if editing.get() {
                    let draft = ctx.state.with_untracked(|s| s.editing.as_ref().map(|e| e.draft.clone()).unwrap_or_default());
                    Either::Left(view! {
                        <input
                            id=h::editor_id(&ctx.id.get_value(), row, col)
                            type="text"
                            aria-label="Edit cell"
                            data-scope=h::SCOPE
                            data-part="editor"
                            prop:value=draft
                            on:input=move |ev| ctx.send(GridEvent::EditChange(event_target_value(&ev)))
                            on:keydown=move |ev| {
                                if let Some(event) = h::editor_keydown(&ev.key()) {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    ctx.send(event);
                                }
                            }
                            on:blur=move |_| ctx.send(GridEvent::Blur)
                        />
                    })
                } else {
                    Either::Right(text.get())
                }
            }}
        </div>
    }
}
