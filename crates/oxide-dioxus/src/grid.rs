//! `Grid`: an ARIA data grid. One cell is in the tab order, arrow keys move
//! between cells, and Enter or F2 edits a cell.

use dioxus::prelude::*;
use oxide_headless::grid::{self as h, GridCommand, GridEvent, GridInput, GridMachine, GridState};
use oxide_statechart::Machine;

use crate::focus::FocusMap;

pub use oxide_headless::grid::{GridColumn, GridEdit, GridRow};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<GridState>,
    input: Memo<GridInput>,
    cells: FocusMap,
    on_edit: Option<EventHandler<GridEdit>>,
}

impl Ctx {
    fn send(mut self, event: GridEvent) {
        let step = GridMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                GridCommand::FocusCell { row, col } => self.cells.focus(&format!("{row}-{col}")),
                // The editor focuses itself when it mounts.
                GridCommand::FocusEditor { .. } => {}
                GridCommand::Edit(edit) => {
                    if let Some(handler) = self.on_edit {
                        handler.call(edit);
                    }
                }
            }
        }
    }
}

/// A grid of text cells. Mark columns editable with [`GridColumn::editable`].
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Grid(
    /// Columns, left to right.
    columns: Vec<GridColumn>,
    /// Accessible name for the grid.
    label: String,
    /// Controlled rows. Pair with `on_edit`.
    #[props(default)]
    rows: ReadSignal<Option<Vec<GridRow>>>,
    /// Initial rows when uncontrolled.
    #[props(default)]
    default_rows: Vec<GridRow>,
    /// Rows a page jump skips.
    #[props(default = 10)]
    page_size: usize,
    /// Called when a cell is edited, with the row id, column key, new and old text.
    on_edit: Option<EventHandler<GridEdit>>,
) -> Element {
    let mut columns_signal = use_signal(|| columns.clone());
    if *columns_signal.peek() != columns {
        columns_signal.set(columns.clone());
    }
    let input = use_memo(move || GridInput {
        columns: columns_signal(),
        rows: rows(),
        default_rows: default_rows.clone(),
        page_size,
    });
    let mut state = use_signal(|| GridMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = GridMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx { state, input, cells: FocusMap::new(), on_edit });

    let s = state.read().clone();
    let inp = input.read().clone();
    let template = h::template_columns(&inp);

    rsx! {
        div {
            role: "grid",
            "aria-label": label,
            "aria-rowcount": h::aria_rowcount(&s),
            "aria-colcount": columns.len(),
            "data-scope": h::SCOPE,
            "data-part": "root",
            div { role: "rowgroup", "data-scope": h::SCOPE, "data-part": "header",
                div { role: "row", "aria-rowindex": "1", style: template.clone(), "data-scope": h::SCOPE, "data-part": "header-row",
                    for (c, column) in columns.iter().enumerate() {
                        div {
                            key: "{column.key}",
                            role: "columnheader",
                            "aria-colindex": c + 1,
                            "data-scope": h::SCOPE,
                            "data-part": "header-cell",
                            "{column.header}"
                        }
                    }
                }
            }
            div { role: "rowgroup", "data-scope": h::SCOPE, "data-part": "body",
                for (r, row) in s.rows.iter().enumerate() {
                    div {
                        key: "{row.id}",
                        role: "row",
                        "aria-rowindex": h::aria_rowindex(r),
                        style: template.clone(),
                        "data-scope": h::SCOPE,
                        "data-part": "row",
                        for (c, text) in row.cells.iter().enumerate() {
                            {
                                let a = h::cell_attrs(&root, &s, &inp, r, c);
                                let editor = h::editor_id(&root, r, c);
                                let mounted_key = format!("{r}-{c}");
                                rsx! {
                                    div {
                                        key: "{c}",
                                        role: "gridcell",
                                        id: a.id,
                                        tabindex: a.tabindex.to_string(),
                                        "aria-colindex": a.aria_colindex,
                                        "aria-readonly": a.readonly.then_some("true"),
                                        "data-scope": h::SCOPE,
                                        "data-part": "cell",
                                        "data-editing": a.editing.then_some(""),
                                        onmounted: move |e| ctx.cells.register(mounted_key.clone(), e.data()),
                                        onfocus: move |_| ctx.send(GridEvent::Focus { row: r, col: c }),
                                        ondoubleclick: move |_| ctx.send(GridEvent::StartEditAt { row: r, col: c }),
                                        onkeydown: move |e| {
                                            // Keys typed in the editor belong to the editor.
                                            if ctx.state.peek().editing.is_some() {
                                                return;
                                            }
                                            let ctrl = e.modifiers().ctrl() || e.modifiers().meta();
                                            if let Some(event) = h::cell_keydown(&e.key().to_string(), ctrl) {
                                                e.prevent_default();
                                                ctx.send(event);
                                            }
                                        },
                                        if a.editing {
                                            input {
                                                id: editor.clone(),
                                                r#type: "text",
                                                "aria-label": "Edit cell",
                                                "data-scope": h::SCOPE,
                                                "data-part": "editor",
                                                value: s.editing.as_ref().map(|ed| ed.draft.clone()).unwrap_or_default(),
                                                onmounted: move |_| {
                                                    let _ = document::eval(&format!(
                                                        "const el = document.getElementById({editor:?}); if (el) {{ el.focus(); el.select(); }}"
                                                    ));
                                                },
                                                oninput: move |e| ctx.send(GridEvent::EditChange(e.value())),
                                                onkeydown: move |e| {
                                                    if let Some(event) = h::editor_keydown(&e.key().to_string()) {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        ctx.send(event);
                                                    }
                                                },
                                                onblur: move |_| ctx.send(GridEvent::Blur),
                                            }
                                        } else {
                                            "{text}"
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
