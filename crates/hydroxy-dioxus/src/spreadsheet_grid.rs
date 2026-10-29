//! `SpreadsheetGrid`: an editable sheet with A1 columns, row numbers, range
//! selection, formulas, TSV copy and paste, column resizing, and row
//! windowing for large sheets.
//!
//! Paste arrives through a `paste` listener installed with `document::eval`
//! (Dioxus clipboard events carry no data); Ctrl/Cmd+C and X write the
//! selection with `navigator.clipboard.writeText`.

use std::collections::BTreeMap;
use std::sync::Arc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use hydroxy_headless::spreadsheet_grid::{
    self as h, KeyPress, SpreadsheetGridCommand as Cmd, SpreadsheetGridEvent as Ev, SpreadsheetGridInput,
    SpreadsheetGridMachine, SpreadsheetGridState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::focus_id;
use crate::js;
use crate::modifiers::mods;

pub use hydroxy_headless::spreadsheet_grid::{Cell, CellChange, cell_name, cells_from_a1};

/// Forwards pastes on the viewport itself (not the editor) to Rust.
const PASTE: &str = r"
const el = document.getElementById(await dioxus.recv());
if (el) {
  el.addEventListener('paste', (e) => {
    if (e.target !== el) return;
    e.preventDefault();
    dioxus.send(e.clipboardData ? e.clipboardData.getData('text/plain') : '');
  });
}
";

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<SpreadsheetGridState>,
    input: Signal<SpreadsheetGridInput>,
    root: Signal<String>,
    on_change: Option<EventHandler<Vec<CellChange>>>,
}

impl Ctx {
    fn send(self, event: Ev) {
        let commands = drive::<SpreadsheetGridMachine>(self.state, self.input, event);
        let viewport = h::viewport_id(&self.root.peek());
        for command in commands {
            match command {
                Cmd::FocusGrid => {
                    focus_id(&viewport);
                }
                // The editor focuses itself when it mounts.
                Cmd::FocusEditor => {}
                // `inf` isn't JS, so only finite offsets go through.
                Cmd::ScrollTo { top, left } => {
                    if top.is_finite() && left.is_finite() {
                        let _ = document::eval(&format!(
                            "document.getElementById({viewport})?.scrollTo({left}, {top});",
                            viewport = crate::js::js_string(&viewport)
                        ));
                    }
                }
                Cmd::Change(changes) => {
                    if let Some(handler) = self.on_change {
                        handler.call(changes);
                    }
                }
            }
        }
    }

    fn editing(self) -> bool {
        self.state.peek().editing.is_some()
    }
}

/// An editable spreadsheet. Type a number, text, or a formula starting with
/// `=` (`=SUM(B2:B9)`, `=A1*1.2`). Props are read once, at mount.
#[component]
pub fn SpreadsheetGrid(
    /// Accessible name for the grid.
    label: String,
    /// Number of rows. Only the rows in view are rendered, so thousands are fine.
    #[props(default = 1000)]
    rows: usize,
    /// Number of columns.
    #[props(default = 10)]
    cols: usize,
    /// Initial cell text by (row, column), 0-based. See [`cells_from_a1`].
    #[props(default)]
    default_cells: BTreeMap<Cell, String>,
    /// Initial column width in pixels.
    #[props(default = 112.0)]
    col_width: f64,
    /// Height of the scrolling area in pixels.
    #[props(default = 360.0)]
    height: f64,
    /// Called with every batch of changed cells: one edit, a paste, a cut or a clear.
    on_change: Option<EventHandler<Vec<CellChange>>>,
) -> Element {
    let input = use_signal(|| SpreadsheetGridInput {
        rows,
        cols,
        default_cells: Arc::new(default_cells.clone()),
        col_width,
        height,
    });
    let state = use_signal(|| SpreadsheetGridMachine::initial(&input.peek()));
    let root = use_signal(|| hydroxy_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx { state, input, root, on_change });
    let viewport_id = h::viewport_id(&root.peek());

    use_future(move || async move {
        let mut eval = document::eval(PASTE);
        let _ = eval.send(h::viewport_id(&root.peek()));
        while let Ok(text) = eval.recv::<String>().await {
            ctx.send(Ev::Paste(text));
        }
    });

    let s = state.read().clone();
    let inp = input.peek().clone();
    let template = h::template_columns(&s);
    let sel = s.selection();
    let root_id = root.peek().clone();
    let resizing = s.resizing.is_some();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            div { "data-scope": h::SCOPE, "data-part": "formula-bar",
                output { "data-scope": h::SCOPE, "data-part": "name-box", "aria-label": "Selection", {h::name_box(&s)} }
                div { "data-scope": h::SCOPE, "data-part": "formula", "aria-label": "Cell contents", {h::formula_text(&s)} }
            }
            div {
                id: viewport_id,
                role: "grid",
                tabindex: "0",
                "aria-label": label,
                "aria-multiselectable": "true",
                "aria-rowcount": h::aria_rowcount(&inp),
                "aria-colcount": h::aria_colcount(&inp),
                "aria-activedescendant": h::cell_id(&root_id, s.active),
                style: h::viewport_style(&inp),
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                "data-resizing": resizing.then_some(""),
                onmounted: move |e| {
                    let el = e.data();
                    spawn(async move {
                        if let Ok(rect) = el.get_client_rect().await {
                            ctx.send(Ev::Scroll { top: 0.0, left: 0.0, width: rect.width(), height: rect.height() });
                        }
                    });
                },
                onscroll: move |e| {
                    let d = e.data();
                    ctx.send(Ev::Scroll {
                        top: d.scroll_top(),
                        left: d.scroll_left(),
                        width: f64::from(d.client_width()),
                        height: f64::from(d.client_height()),
                    });
                },
                onpointerup: move |_| ctx.send(Ev::PointerUp),
                onkeydown: move |e| {
                    // Keys typed in the editor belong to the editor.
                    if ctx.editing() {
                        return;
                    }
                    let m = e.modifiers();
                    let press = KeyPress {
                        key: e.key().to_string(),
                        shift: m.shift(),
                        ctrl: m.ctrl(),
                        meta: m.meta(),
                        alt: m.alt(),
                        ..KeyPress::default()
                    };
                    if let Some(cut) = h::copy_shortcut(&press) {
                        js::copy(&ctx.state.peek().selection_tsv());
                        if cut {
                            ctx.send(Ev::Clear);
                        }
                        return;
                    }
                    if let Some(event) = h::grid_keydown(&press) {
                        e.prevent_default();
                        ctx.send(event);
                    }
                },
                div { role: "row", "aria-rowindex": "1", style: template, "data-scope": h::SCOPE, "data-part": "header-row",
                    div { role: "columnheader", "aria-colindex": "1", "data-scope": h::SCOPE, "data-part": "corner" }
                    for c in 0..inp.cols {
                        {
                            let a = h::resize_attrs(&s, c);
                            let handle_id = format!("{root_id}-resize-{c}");
                            rsx! {
                                div {
                                    key: "{c}",
                                    role: "columnheader",
                                    "aria-colindex": h::aria_colindex(c),
                                    "data-scope": h::SCOPE,
                                    "data-part": "column-header",
                                    "data-selected": (sel.left()..=sel.right()).contains(&c).then_some(""),
                                    {h::column_name(c)}
                                    div {
                                        id: handle_id.clone(),
                                        role: "separator",
                                        tabindex: "0",
                                        "aria-orientation": "vertical",
                                        "aria-label": a.label,
                                        "aria-valuenow": a.valuenow.to_string(),
                                        "aria-valuemin": h::MIN_COL_WIDTH.to_string(),
                                        "aria-valuemax": h::MAX_COL_WIDTH.to_string(),
                                        "data-scope": h::SCOPE,
                                        "data-part": "resize-handle",
                                        "data-dragging": a.dragging.then_some(""),
                                        onpointerdown: move |e| {
                                            if e.trigger_button() != Some(MouseButton::Primary) {
                                                return;
                                            }
                                            e.prevent_default();
                                            e.stop_propagation();
                                            let _ = document::eval(&format!(
                                                "try {{ document.getElementById({handle_id})?.setPointerCapture({}); }} catch (_) {{}}",
                                                e.pointer_id(), handle_id = crate::js::js_string(&handle_id)));
                                            ctx.send(Ev::ResizeStart { col: c, x: e.client_coordinates().x });
                                        },
                                        onpointermove: move |e| {
                                            if ctx.state.peek().resizing.is_some() {
                                                ctx.send(Ev::ResizeMove(e.client_coordinates().x));
                                            }
                                        },
                                        onpointerup: move |_| ctx.send(Ev::ResizeEnd),
                                        onpointercancel: move |_| ctx.send(Ev::ResizeEnd),
                                        onkeydown: move |e| {
                                            e.stop_propagation();
                                            if let Some(event) = h::resize_keydown(&e.key().to_string(), mods(e.modifiers()), c) {
                                                e.prevent_default();
                                                ctx.send(event);
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                div { role: "rowgroup", style: h::body_style(&s, &inp), "data-scope": h::SCOPE, "data-part": "body",
                    div {
                        "aria-hidden": "true",
                        "data-scope": h::SCOPE,
                        "data-part": "selection",
                        "data-range": (sel.count() > 1).then_some(""),
                        style: h::selection_style(&s),
                    }
                    for r in h::rendered_rows(&s, &inp) {
                        div {
                            key: "{r}",
                            role: "row",
                            "aria-rowindex": h::aria_rowindex(r),
                            style: h::row_style(r, &template),
                            "data-scope": h::SCOPE,
                            "data-part": "row",
                            div {
                                role: "rowheader",
                                "aria-colindex": "1",
                                "data-scope": h::SCOPE,
                                "data-part": "row-header",
                                "data-selected": (sel.top()..=sel.bottom()).contains(&r).then_some(""),
                                "{r + 1}"
                            }
                            for c in 0..inp.cols {
                                {
                                    let cell = (r, c);
                                    let a = h::cell_attrs(&root_id, &s, cell);
                                    let editing = a.active && s.editing.is_some();
                                    let editor = h::editor_id(&root_id);
                                    rsx! {
                                        div {
                                            key: "{c}",
                                            role: "gridcell",
                                            id: a.id,
                                            "aria-colindex": a.aria_colindex,
                                            "aria-selected": a.selected.to_string(),
                                            "data-scope": h::SCOPE,
                                            "data-part": "cell",
                                            "data-kind": a.kind,
                                            "data-active": a.active.then_some(""),
                                            "data-editing": editing.then_some(""),
                                            onpointerdown: move |e| {
                                                // Clicks inside the open editor move its caret.
                                                if e.trigger_button() != Some(MouseButton::Primary) || editing {
                                                    return;
                                                }
                                                ctx.send(Ev::PointerDown { cell, shift: e.modifiers().shift() });
                                            },
                                            onpointerenter: move |e| {
                                                if e.held_buttons().contains(MouseButton::Primary) {
                                                    ctx.send(Ev::PointerEnter(cell));
                                                }
                                            },
                                            ondoubleclick: move |_| {
                                                if !editing {
                                                    ctx.send(Ev::EditAt(cell));
                                                }
                                            },
                                            if editing {
                                                input {
                                                    id: editor.clone(),
                                                    r#type: "text",
                                                    autocomplete: "off",
                                                    spellcheck: "false",
                                                    "aria-label": format!("Edit {}", h::cell_name(cell)),
                                                    "data-scope": h::SCOPE,
                                                    "data-part": "editor",
                                                    value: s.editing.clone().unwrap_or_default(),
                                                    onmounted: move |_| {
                                                        let _ = document::eval(&format!(
                                                            "const el = document.getElementById({editor}); \
                                                             if (el) {{ el.focus(); const n = el.value.length; el.setSelectionRange(n, n); }}", editor = crate::js::js_string(&editor)));
                                                    },
                                                    oninput: move |e| ctx.send(Ev::EditChange(e.value())),
                                                    onkeydown: move |e| {
                                                        if let Some(event) = h::editor_keydown(&e.key().to_string(), mods(e.modifiers())) {
                                                            e.prevent_default();
                                                            e.stop_propagation();
                                                            ctx.send(event);
                                                        }
                                                    },
                                                    onblur: move |_| ctx.send(Ev::Blur),
                                                }
                                            } else {
                                                "{a.text}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { role: "status", "aria-live": "polite", "data-scope": h::SCOPE, "data-part": "status", "{s.status}" }
        }
    }
}
