//! `DockLayout`: an IDE-style workspace. Splits you can resize, tab groups
//! whose tabs you can reorder and move between groups by drag or keyboard.

use dioxus::prelude::*;
use hydroxy_headless::dock_layout::{
    self as h, DockCommand, DockEvent, DockGroup, DockInput, DockLayoutMachine, DockNode, DockState, announcement_text,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};
use crate::modifiers::mods;
use crate::resizable::{Panel, ResizablePanels};

pub use hydroxy_headless::dock_layout::{DockGroup as Group, DockNode as Node, SplitAxis};

/// One panel: an id that appears in the tree, a tab title and its content.
#[derive(Clone, PartialEq)]
pub struct DockPanel {
    id: String,
    title: String,
    content: Element,
}

impl DockPanel {
    /// A panel showing `content` under a tab titled `title`.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: Element) -> Self {
        Self { id: id.into(), title: title.into(), content }
    }
}

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<DockState>,
    input: Memo<DockInput>,
    live: Signal<String>,
    titles: CopyValue<Vec<(String, String)>>,
    tabs: FocusMap,
    /// A tab to focus when it mounts: a move can re-create it in another group.
    pending_focus: Signal<Option<String>>,
    on_layout_change: Option<EventHandler<DockNode>>,
    on_close: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(mut self, event: DockEvent) {
        let commands = drive::<DockLayoutMachine>(self.state, self.input, event);
        for command in commands {
            match command {
                DockCommand::LayoutChange(tree) => {
                    if let Some(handler) = self.on_layout_change {
                        handler.call(tree);
                    }
                }
                DockCommand::Closed(panel) => {
                    if let Some(handler) = self.on_close {
                        handler.call(panel);
                    }
                }
                DockCommand::FocusTab(panel) => {
                    self.tabs.focus(&panel);
                    self.pending_focus.set(Some(panel));
                }
                DockCommand::Announce(a) => {
                    let h::Announcement::Moved { panel, .. } = &a;
                    let title = self
                        .titles
                        .read()
                        .iter()
                        .find(|(id, _)| id == panel)
                        .map_or_else(|| panel.clone(), |(_, t)| t.clone());
                    self.live.set(announcement_text(&a, &title));
                }
            }
        }
    }
}

/// Everything a render pass reads.
struct Env {
    ctx: Ctx,
    root: String,
    state: DockState,
    input: DockInput,
    panels: Vec<DockPanel>,
}

/// A workspace of resizable splits and tab groups.
#[component]
pub fn DockLayout(
    /// Every panel the tree can name.
    panels: Vec<DockPanel>,
    /// Panels that cannot be closed.
    #[props(default)]
    locked: Vec<String>,
    /// Controlled tree. Pair with `on_layout_change`.
    #[props(default)]
    tree: ReadSignal<Option<DockNode>>,
    /// Initial tree when uncontrolled.
    default_tree: DockNode,
    /// Called with the whole tree when tabs move, close or change.
    on_layout_change: Option<EventHandler<DockNode>>,
    /// Called with the panel's id when it is closed.
    on_close: Option<EventHandler<String>>,
) -> Element {
    let input =
        use_memo(move || DockInput { tree: tree(), default_tree: default_tree.clone(), locked: locked.clone() });
    let mut state = use_signal(|| DockLayoutMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DockLayoutMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let titles: Vec<(String, String)> = panels.iter().map(|p| (p.id.clone(), p.title.clone())).collect();
    let ctx = use_hook(|| Ctx {
        state,
        input,
        live: Signal::new(String::new()),
        titles: CopyValue::new(titles.clone()),
        tabs: FocusMap::new(),
        pending_focus: Signal::new(None),
        on_layout_change,
        on_close,
    });

    let env = Env { ctx, root: root.clone(), state: state.read().clone(), input: input.read().clone(), panels };
    let body = node_view(&env, &env.state.tree.value);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            p { "data-scope": h::SCOPE, "data-part": "instructions", id: h::instructions_id(&root), {h::INSTRUCTIONS} }
            {body}
            div { "data-scope": h::SCOPE, "data-part": "live", role: "status", "aria-live": "polite", "aria-atomic": "true", "{ctx.live}" }
        }
    }
}

fn node_view(env: &Env, node: &DockNode) -> Element {
    match node {
        DockNode::Tabs(group) => group_view(env, group),
        DockNode::Split { id, axis, children } => {
            let panels: Vec<Panel> = children.iter().map(|child| Panel::new(node_view(env, child)).min(12.0)).collect();
            rsx! {
                div { key: "{id}", "data-scope": h::SCOPE, "data-part": "split", "data-id": "{id}",
                    ResizablePanels { panels, orientation: h::orientation(*axis) }
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
fn group_view(env: &Env, group: &DockGroup) -> Element {
    let ctx = env.ctx;
    // Only panels the owner defined; an unknown id in the tree is skipped.
    let known: Vec<&DockPanel> = group.panels.iter().filter_map(|id| env.panels.iter().find(|p| p.id == *id)).collect();
    let (over_end, group_id) = (group.id.clone(), group.id.clone());
    rsx! {
        div {
            key: "{group.id}",
            "data-scope": h::SCOPE,
            "data-part": "group",
            "data-id": "{group.id}",
            "data-empty": known.is_empty().then_some(""),
            div {
                role: "tablist",
                "data-scope": h::SCOPE,
                "data-part": "tablist",
                "data-drop-end": h::drop_at_end(&env.state, &group.id).then_some(""),
                ondragover: move |e| {
                    e.prevent_default();
                    ctx.send(DockEvent::DragOver { group: over_end.clone(), before: None });
                },
                ondrop: move |e| {
                    e.prevent_default();
                    ctx.send(DockEvent::Drop);
                },
                for p in known.iter() {
                    {
                        let a = h::tab_attrs(&env.root, &env.state, group, &p.id);
                        let closable = !env.input.locked.contains(&p.id);
                        let (mounted, started, hovered, keyed, activate, close) =
                            (p.id.clone(), p.id.clone(), p.id.clone(), p.id.clone(), p.id.clone(), p.id.clone());
                        let (state, input, gid) = (ctx.state, ctx.input, group_id.clone());
                        rsx! {
                            div {
                                key: "{p.id}",
                                role: "tab",
                                id: a.id,
                                tabindex: a.tabindex.to_string(),
                                "aria-selected": a.aria_selected.to_string(),
                                "aria-controls": a.aria_controls,
                                "aria-describedby": h::instructions_id(&env.root),
                                draggable: "true",
                                "data-scope": h::SCOPE,
                                "data-part": "tab",
                                "data-state": a.data_state,
                                "data-dragging": a.dragging.then_some(""),
                                "data-drop-before": a.drop_before.then_some(""),
                                onmounted: move |e| {
                                    let el = e.data();
                                    ctx.tabs.register(mounted.clone(), el.clone());
                                    let mut pending = ctx.pending_focus;
                                    if pending.peek().as_deref() == Some(mounted.as_str()) {
                                        pending.set(None);
                                        spawn(async move {
                                            let _ = el.set_focus(true).await;
                                        });
                                    }
                                },
                                onclick: move |_| ctx.send(DockEvent::Activate(activate.clone())),
                                ondragstart: move |e| {
                                    let dt = e.data_transfer();
                                    // Firefox only starts a drag that carries data.
                                    let _ = dt.set_data("text/plain", &started);
                                    dt.set_effect_allowed("move");
                                    ctx.send(DockEvent::DragStart(started.clone()));
                                },
                                ondragover: move |e| {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    ctx.send(DockEvent::DragOver { group: gid.clone(), before: Some(hovered.clone()) });
                                },
                                ondragend: move |_| ctx.send(DockEvent::Cancel),
                                onkeydown: move |e| {
                                    let event = h::tab_keydown(&e.key().to_string(), mods(e.modifiers()), &keyed, &state.peek(), &input.peek());
                                    if let Some(event) = event {
                                        e.prevent_default();
                                        ctx.send(event);
                                    }
                                },
                                span { "data-scope": h::SCOPE, "data-part": "title", "{p.title}" }
                                if closable {
                                    button {
                                        r#type: "button",
                                        tabindex: "-1",
                                        "aria-label": "Close {p.title}",
                                        "data-scope": h::SCOPE,
                                        "data-part": "close",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            ctx.send(DockEvent::Close(close.clone()));
                                        },
                                        Icon { name: IconName::X, size: 12 }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for p in known.iter() {
                {
                    let a = h::panel_attrs(&env.root, group, &p.id);
                    rsx! {
                        div {
                            key: "{p.id}",
                            role: "tabpanel",
                            tabindex: "0",
                            id: a.id,
                            "aria-labelledby": a.aria_labelledby,
                            hidden: a.hidden,
                            "data-scope": h::SCOPE,
                            "data-part": "panel",
                            {p.content.clone()}
                        }
                    }
                }
            }
        }
    }
}
