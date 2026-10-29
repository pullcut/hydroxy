//! `<MasterDetail>` and `<MasterDetailRow>`: a list beside its detail on wide
//! screens, one pane at a time on narrow ones. The host owns the selection.
//!
//! Parts panic if rendered outside their root `MasterDetail`.

use crate::drive::drive;
use hydroxy_headless::master_detail::{
    self as h, MasterDetailCommand, MasterDetailEvent, MasterDetailInput, MasterDetailMachine, MasterDetailState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

pub use h::{Pane, PaneMode};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    selected: Signal<Option<String>>,
    send: Callback<MasterDetailEvent>,
}

/// Focus an element by id once the browser has painted the pane that holds it.
fn focus_later(id: String) {
    request_animation_frame(move || {
        if let Some(el) = document().get_element_by_id(&id).and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok()) {
            let _ = el.focus();
        }
    });
}

/// A selectable list row. Render these inside the `list`.
#[component]
pub fn MasterDetailRow(
    /// The row's id, reported to `on_select`.
    #[prop(into)]
    id: String,
    children: Children,
) -> impl IntoView {
    let ctx = use_context::<Ctx>().expect("MasterDetailRow must be rendered inside <MasterDetail>");
    let dom_id = h::row_dom_id(&ctx.id.get_value(), &id);
    let row = id.clone();
    let selected = Memo::new(move |_| ctx.selected.get().as_deref() == Some(row.as_str()));
    view! {
        <button
            type="button"
            id=dom_id
            data-scope=h::SCOPE
            data-part="row"
            data-selected=move || selected.get().then_some("")
            aria-current=move || selected.get().then_some("true")
            on:click=move |_| ctx.send.run(MasterDetailEvent::Select(id.clone()))
        >
            {children()}
        </button>
    }
}

/// A list and its detail. Pick rows with [`MasterDetailRow`]; show the chosen
/// item in `detail`. Selection changes go through `on_select`, so the host
/// decides (and can route). On narrow screens Back asks for a `None` selection.
#[component]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
pub fn MasterDetail(
    /// The host's selected row id.
    #[prop(into)]
    selected: Signal<Option<String>>,
    /// The list pane: a heading is optional, rows are [`MasterDetailRow`]s.
    #[prop(into)]
    list: ViewFn,
    /// The detail pane. Stays mounted; CSS picks the narrow-screen pane.
    #[prop(into)]
    detail: ViewFn,
    /// A heading for the list. It takes focus when the selected row is gone.
    #[prop(optional, into)]
    list_heading: Option<ViewFn>,
    /// `Auto` follows the selection; `List` or `Detail` is host-controlled.
    #[prop(optional)]
    mobile_pane: PaneMode,
    /// A row was chosen (`Some`) or Back was pressed (`None`).
    #[prop(optional, into)]
    on_select: Option<Callback<Option<String>>>,
    /// The host should show this narrow-screen pane (explicit modes only).
    #[prop(optional, into)]
    on_pane_change: Option<Callback<Pane>>,
    /// Back was pressed on this row. Route here.
    #[prop(optional, into)]
    on_back: Option<Callback<String>>,
) -> impl IntoView {
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| MasterDetailInput { selected: selected.get(), mode: mobile_pane });
    let state = RwSignal::new(input.with_untracked(MasterDetailMachine::initial));

    let send = move |event: MasterDetailEvent| {
        for command in drive::<MasterDetailMachine, _>(state, input, event) {
            match command {
                MasterDetailCommand::SelectionChange(next, _) => {
                    if let Some(cb) = on_select {
                        cb.run(next);
                    }
                }
                MasterDetailCommand::PaneChange(pane) => {
                    if let Some(cb) = on_pane_change {
                        cb.run(pane);
                    }
                }
                MasterDetailCommand::Back(row) => {
                    if let Some(cb) = on_back {
                        cb.run(row);
                    }
                }
                MasterDetailCommand::FocusRow(row) => {
                    let root = id.get_value();
                    let target = if document().get_element_by_id(&h::row_dom_id(&root, &row)).is_some() {
                        h::row_dom_id(&root, &row)
                    } else {
                        h::heading_dom_id(&root)
                    };
                    focus_later(target);
                }
            }
        }
    };
    Effect::new(move |_| {
        input.track();
        send(MasterDetailEvent::Sync);
    });
    provide_context(Ctx { id, selected, send: Callback::new(send) });

    let pane = move || state.with(|s: &MasterDetailState| input.with(|i| s.pane(i))).as_str();
    view! {
        <div data-scope=h::SCOPE data-part="root" data-mobile-pane=pane>
            <section data-scope=h::SCOPE data-part="list">
                {list_heading
                    .map(|heading| {
                        view! {
                            <h2 id=h::heading_dom_id(&id.get_value()) tabindex="-1" data-scope=h::SCOPE data-part="list-heading">
                                {heading.run()}
                            </h2>
                        }
                    })} {list.run()}
            </section>
            <section data-scope=h::SCOPE data-part="detail">
                <button type="button" data-scope=h::SCOPE data-part="back" on:click=move |_| send(MasterDetailEvent::Back)>
                    {h::BACK_LABEL}
                </button>
                {detail.run()}
            </section>
        </div>
    }
}
