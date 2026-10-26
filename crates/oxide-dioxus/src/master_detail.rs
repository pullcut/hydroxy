//! `MasterDetail` and `MasterDetailRow`: a list beside its detail on wide
//! screens, one pane at a time on narrow ones. The host owns the selection.

use dioxus::prelude::*;
use oxide_headless::master_detail::{
    self as h, MasterDetailCommand, MasterDetailEvent, MasterDetailInput, MasterDetailMachine,
};
use oxide_statechart::Machine;

pub use h::{Pane, PaneMode};

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    selected: ReadSignal<Option<String>>,
    send: Callback<MasterDetailEvent>,
}

/// A selectable list row. Render these inside the `list`.
#[component]
pub fn MasterDetailRow(
    /// The row's id, reported to `on_select`.
    id: String,
    children: Element,
) -> Element {
    let ctx = try_use_context::<Ctx>().expect("MasterDetailRow must be rendered inside MasterDetail");
    let selected = ctx.selected.read().as_deref() == Some(id.as_str());
    let dom_id = h::row_dom_id(&ctx.id.read(), &id);
    rsx! {
        button {
            r#type: "button",
            id: "{dom_id}",
            "data-scope": h::SCOPE,
            "data-part": "row",
            "data-selected": selected.then_some(""),
            "aria-current": selected.then_some("true"),
            onclick: move |_| ctx.send.call(MasterDetailEvent::Select(id.clone())),
            {children}
        }
    }
}

/// A list and its detail. Pick rows with `MasterDetailRow`; show the chosen
/// item in `detail`. Selection changes go through `on_select`, so the host
/// decides (and can route). On narrow screens Back asks for a `None` selection.
#[component]
pub fn MasterDetail(
    /// The host's selected row id.
    #[props(default)]
    selected: ReadSignal<Option<String>>,
    /// The list pane: a heading is optional, rows are `MasterDetailRow`s.
    list: Element,
    /// The detail pane. Stays mounted; CSS picks the narrow-screen pane.
    detail: Element,
    /// A heading for the list. It takes focus when the selected row is gone.
    list_heading: Option<Element>,
    /// `Auto` follows the selection; `List` or `Detail` is host-controlled.
    #[props(default)]
    mobile_pane: PaneMode,
    /// A row was chosen (`Some`) or Back was pressed (`None`).
    on_select: Option<EventHandler<Option<String>>>,
    /// The host should show this narrow-screen pane (explicit modes only).
    on_pane_change: Option<EventHandler<Pane>>,
    /// Back was pressed on this row. Route here.
    on_back: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| CopyValue::new(oxide_headless::next_id(h::SCOPE)));
    let input = use_memo(move || MasterDetailInput { selected: selected(), mode: mobile_pane });
    let mut state = use_signal(|| MasterDetailMachine::initial(&input.peek()));

    let send = Callback::new(move |event: MasterDetailEvent| {
        let step = MasterDetailMachine::transition(&state.peek(), event, &input.peek());
        state.set(step.state);
        for command in step.commands {
            match command {
                MasterDetailCommand::SelectionChange(next, _) => {
                    if let Some(handler) = on_select {
                        handler.call(next);
                    }
                }
                MasterDetailCommand::PaneChange(pane) => {
                    if let Some(handler) = on_pane_change {
                        handler.call(pane);
                    }
                }
                MasterDetailCommand::Back(row) => {
                    if let Some(handler) = on_back {
                        handler.call(row);
                    }
                }
                MasterDetailCommand::FocusRow(row) => {
                    let (row_id, heading) = (h::row_dom_id(&id.peek(), &row), h::heading_dom_id(&id.peek()));
                    // Wait a frame so the list pane is shown before focusing into it.
                    let _ = document::eval(&format!(
                        "requestAnimationFrame(() => (document.getElementById({row_id:?}) ?? document.getElementById({heading:?}))?.focus());"
                    ));
                }
            }
        }
    });
    use_effect(move || {
        input.read();
        send.call(MasterDetailEvent::Sync);
    });
    use_context_provider(|| Ctx { id, selected, send });

    let pane = state.read().pane(&input.read()).as_str();
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-mobile-pane": pane,
            section { "data-scope": h::SCOPE, "data-part": "list",
                if let Some(heading) = list_heading {
                    h2 { id: h::heading_dom_id(&id.read()), tabindex: "-1", "data-scope": h::SCOPE, "data-part": "list-heading", {heading} }
                }
                {list}
            }
            section { "data-scope": h::SCOPE, "data-part": "detail",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "back",
                    onclick: move |_| send.call(MasterDetailEvent::Back),
                    {h::BACK_LABEL}
                }
                {detail}
            }
        }
    }
}
