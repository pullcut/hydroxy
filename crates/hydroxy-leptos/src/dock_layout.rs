//! `<DockLayout>`: an IDE-style workspace. Splits you can resize, tab groups
//! whose tabs you can reorder and move between groups by drag or keyboard.

use hydroxy_headless::dock_layout::{
    self as h, DockCommand, DockEvent, DockGroup, DockInput, DockLayoutMachine, DockNode, DockState, announcement_text,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::modifiers::key_mods;
use crate::resizable::{Panel, ResizablePanels};

pub use hydroxy_headless::dock_layout::{DockGroup as Group, DockNode as Node, SplitAxis};

/// One panel: an id that appears in the tree, a tab title and its content.
#[derive(Clone)]
pub struct DockPanel {
    id: String,
    title: String,
    content: ViewFn,
}

impl DockPanel {
    /// A panel showing `content` under a tab titled `title`.
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<ViewFn>) -> Self {
        Self { id: id.into(), title: title.into(), content: content.into() }
    }
}

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<DockState>,
    input: Memo<DockInput>,
    live: RwSignal<String>,
    panels: StoredValue<Vec<DockPanel>>,
    on_layout_change: Option<Callback<DockNode>>,
    on_close: Option<Callback<String>>,
}

impl Ctx {
    fn title(self, panel: &str) -> String {
        self.panels
            .with_value(|p| p.iter().find(|d| d.id == panel).map(|d| d.title.clone()))
            .unwrap_or_else(|| panel.to_string())
    }

    fn send(self, event: DockEvent) {
        for command in drive::<DockLayoutMachine, _>(self.state, self.input, event) {
            match command {
                DockCommand::LayoutChange(tree) => {
                    if let Some(cb) = self.on_layout_change {
                        cb.run(tree);
                    }
                }
                DockCommand::Closed(panel) => {
                    if let Some(cb) = self.on_close {
                        cb.run(panel);
                    }
                }
                DockCommand::FocusTab(panel) => {
                    // The tab may have moved to another group; focus it once the DOM has caught up.
                    let target = h::tab_id(&self.id.get_value(), &panel);
                    crate::focus::focus_soon(&target);
                }
                DockCommand::Announce(a) => {
                    let h::Announcement::Moved { panel, .. } = &a;
                    self.live.set(announcement_text(&a, &self.title(panel)));
                }
            }
        }
    }
}

/// A workspace of resizable splits and tab groups.
#[component]
pub fn DockLayout(
    /// Every panel the tree can name.
    panels: Vec<DockPanel>,
    /// Panels that cannot be closed.
    #[prop(optional, into)]
    locked: Vec<String>,
    /// Controlled tree. Pair with `on_layout_change`.
    #[prop(optional, into)]
    tree: MaybeProp<DockNode>,
    /// Initial tree when uncontrolled.
    default_tree: DockNode,
    /// Called with the whole tree when tabs move, close or change.
    #[prop(optional, into)]
    on_layout_change: Option<Callback<DockNode>>,
    /// Called with the panel's id when it is closed.
    #[prop(optional, into)]
    on_close: Option<Callback<String>>,
) -> impl IntoView {
    let input =
        Memo::new(move |_| DockInput { tree: tree.get(), default_tree: default_tree.clone(), locked: locked.clone() });
    let state = RwSignal::new(input.with_untracked(DockLayoutMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DockLayoutMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx {
        id: StoredValue::new(crate::id::next_id(h::SCOPE)),
        state,
        input,
        live: RwSignal::new(String::new()),
        panels: StoredValue::new(panels),
        on_layout_change,
        on_close,
    };
    let skeleton = Memo::new(move |_| state.with(|s| s.tree.value.skeleton()));
    let root = ctx.id.get_value();

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <p data-scope=h::SCOPE data-part="instructions" id=h::instructions_id(&root)>{h::INSTRUCTIONS}</p>
            {move || node_view(ctx, skeleton.get())}
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || ctx.live.get()}
            </div>
        </div>
    }
}

fn node_view(ctx: Ctx, node: DockNode) -> AnyView {
    match node {
        DockNode::Tabs(g) => group_view(ctx, g.id).into_any(),
        DockNode::Split { id, axis, children } => {
            let panels =
                children.into_iter().map(|child| Panel::new(move || node_view(ctx, child.clone())).min(12.0)).collect();
            view! {
                <div data-scope=h::SCOPE data-part="split" data-id=id>
                    <ResizablePanels panels=panels orientation=h::orientation(axis) />
                </div>
            }
            .into_any()
        }
    }
}

fn group_view(ctx: Ctx, group: String) -> impl IntoView {
    let live_group = {
        let group = group.clone();
        Memo::new(move |_| ctx.state.with(|s| s.tree.value.groups().into_iter().find(|g| g.id == group).cloned()))
    };
    // Only panels the owner defined; an unknown id in the tree is skipped.
    let ids = Memo::new(move |_| {
        let known: Vec<String> = ctx.panels.with_value(|p| p.iter().map(|d| d.id.clone()).collect());
        live_group.with(|g| {
            g.as_ref()
                .map(|g| g.panels.iter().filter(|p| known.contains(p)).cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        })
    });
    let over_end = group.clone();
    let marker = group.clone();
    let tab_group = group.clone();
    view! {
        <div data-scope=h::SCOPE data-part="group" data-id=group data-empty=move || ids.with(Vec::is_empty).then_some("")>
            <div
                role="tablist"
                data-scope=h::SCOPE
                data-part="tablist"
                data-drop-end=move || ctx.state.with(|s| h::drop_at_end(s, &marker)).then_some("")
                on:dragover=move |ev| {
                    ev.prevent_default();
                    ctx.send(DockEvent::DragOver { group: over_end.clone(), before: None });
                }
                on:drop=move |ev| {
                    ev.prevent_default();
                    ctx.send(DockEvent::Drop);
                }
            >
                <For each=move || ids.get() key=String::clone children=move |p| tab_view(ctx, tab_group.clone(), live_group, &p) />
            </div>
            <For each=move || ids.get() key=String::clone children=move |p| panel_view(ctx, live_group, &p) />
        </div>
    }
}

fn tab_view(ctx: Ctx, group_id: String, group: Memo<Option<DockGroup>>, panel: &str) -> impl IntoView + use<> {
    let attrs = {
        let panel = panel.to_string();
        Memo::new(move |_| {
            group.with(|g| g.as_ref().map(|g| ctx.state.with(|s| h::tab_attrs(&ctx.id.get_value(), s, g, &panel))))
        })
    };
    let title = ctx.title(panel);
    let closable = ctx.input.with_untracked(|i| !i.locked.iter().any(|l| l == panel));
    let (started, over, keyed, activate, close) =
        (panel.to_string(), panel.to_string(), panel.to_string(), panel.to_string(), panel.to_string());
    let close_label = format!("Close {title}");
    let describedby = h::instructions_id(&ctx.id.get_value());
    view! {
        <div
            role="tab"
            id=move || attrs.get().map(|a| a.id)
            tabindex=move || attrs.get().map(|a| a.tabindex)
            aria-selected=move || attrs.get().map(|a| a.aria_selected.to_string())
            aria-controls=move || attrs.get().map(|a| a.aria_controls)
            aria-describedby=describedby
            draggable="true"
            data-scope=h::SCOPE
            data-part="tab"
            data-state=move || attrs.get().map(|a| a.data_state)
            data-dragging=move || attrs.get().is_some_and(|a| a.dragging).then_some("")
            data-drop-before=move || attrs.get().is_some_and(|a| a.drop_before).then_some("")
            on:click=move |_| ctx.send(DockEvent::Activate(activate.clone()))
            on:dragstart=move |ev| {
                if let Some(dt) = ev.data_transfer() {
                    // Firefox only starts a drag that carries data.
                    let _ = dt.set_data("text/plain", &started);
                    dt.set_effect_allowed("move");
                }
                ctx.send(DockEvent::DragStart(started.clone()));
            }
            on:dragover=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                ctx.send(DockEvent::DragOver { group: group_id.clone(), before: Some(over.clone()) });
            }
            on:dragend=move |_| ctx.send(DockEvent::Cancel)
            on:keydown=move |ev| {
                let event = ctx.input.with_untracked(|i| {
                    ctx.state.with_untracked(|s| h::tab_keydown(&ev.key(), key_mods(&ev), &keyed, s, i))
                });
                if let Some(event) = event {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            <span data-scope=h::SCOPE data-part="title">{title}</span>
            {closable.then(|| view! {
                <button
                    type="button"
                    tabindex="-1"
                    aria-label=close_label
                    data-scope=h::SCOPE
                    data-part="close"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        ctx.send(DockEvent::Close(close.clone()));
                    }
                >
                    <Icon name=IconName::X size=12 />
                </button>
            })}
        </div>
    }
}

fn panel_view(ctx: Ctx, group: Memo<Option<DockGroup>>, panel: &str) -> impl IntoView + use<> {
    let attrs = {
        let panel = panel.to_string();
        Memo::new(move |_| group.with(|g| g.as_ref().map(|g| h::panel_attrs(&ctx.id.get_value(), g, &panel))))
    };
    let content = ctx.panels.with_value(|p| p.iter().find(|d| d.id == panel).map(|d| d.content.clone()));
    view! {
        <div
            role="tabpanel"
            tabindex="0"
            id=move || attrs.get().map(|a| a.id)
            aria-labelledby=move || attrs.get().map(|a| a.aria_labelledby)
            hidden=move || attrs.get().is_none_or(|a| a.hidden)
            data-scope=h::SCOPE
            data-part="panel"
        >
            {content.map(|c| c.run())}
        </div>
    }
}
