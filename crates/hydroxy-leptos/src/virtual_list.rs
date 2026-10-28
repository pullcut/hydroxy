//! `<VirtualList>`: a listbox that renders only the rows in view, so it stays smooth
//! with hundreds of thousands of rows. Rows are a fixed height or measured as they
//! render; group headers stick to the top until the next group pushes them off.
//!
//! Focus stays on the listbox and the active option is announced with
//! `aria-activedescendant`. The active row is always rendered, even when scrolled away.

use hydroxy_headless::virtual_list::{
    self as h, Block, Layout, VirtualListCommand, VirtualListEvent, VirtualListInput, VirtualListMachine,
    VirtualListState,
};
use hydroxy_statechart::Machine;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use web_sys::{ResizeObserver, ResizeObserverEntry};

use crate::drive::drive;

pub use hydroxy_headless::virtual_list::{Align, DEFAULT_OVERSCAN};

/// The size observer and the closure it calls, kept alive together.
struct Observer {
    ro: ResizeObserver,
    _on_resize: Closure<dyn FnMut(Vec<ResizeObserverEntry>)>,
}

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<VirtualListState>,
    input: Memo<VirtualListInput>,
    /// Row sizes. Kept out of signals so a measurement never clones it; `version` says
    /// when it changed.
    layout: StoredValue<Layout>,
    version: RwSignal<u64>,
    /// `(scrollTop, clientHeight)` of the viewport.
    view: RwSignal<(f64, f64)>,
    viewport: NodeRef<Div>,
    blocks: Memo<Vec<Block>>,
    row: Callback<usize, AnyView>,
    observer: StoredValue<Option<Observer>, LocalStorage>,
    fixed: bool,
    on_select: Option<Callback<usize>>,
}

impl Ctx {
    fn send(self, event: VirtualListEvent) {
        // Events can still arrive after unmount.
        for command in drive::<VirtualListMachine, _>(self.state, self.input, event) {
            match command {
                VirtualListCommand::ScrollTo(index) => self.scroll_to(index, Align::Auto),
                VirtualListCommand::SelectedChange(index) => {
                    if let Some(cb) = self.on_select {
                        cb.run(index);
                    }
                }
            }
        }
    }

    /// Read the viewport's scroll position and height into `view`.
    fn sync_view(self) {
        let Some(el) = self.viewport.try_get_untracked().flatten() else { return };
        let next = (f64::from(el.scroll_top()), f64::from(el.client_height()));
        if self.view.try_get_untracked().is_some_and(|v| v != next) {
            self.view.set(next);
        }
    }

    #[allow(clippy::cast_possible_truncation, reason = "scroll offsets fit in i32 pixels")]
    fn scroll_to(self, index: usize, align: Align) {
        let Some(el) = self.viewport.try_get_untracked().flatten() else { return };
        let (scroll, viewport) = (f64::from(el.scroll_top()), f64::from(el.client_height()));
        let target = self.input.try_with_untracked(|i| {
            self.layout.try_with_value(|l| h::scroll_target(l, &i.headers, index, align, scroll, viewport))
        });
        if let Some(target) = target.flatten() {
            el.set_scroll_top(target.round() as i32);
            self.sync_view();
        }
    }

    /// Apply a batch of size reports. Rows above the top visible row that change size
    /// shift the scroll position by the same amount, so the view doesn't jump.
    #[allow(clippy::cast_possible_truncation, reason = "scroll offsets fit in i32 pixels")]
    fn measured(self, entries: &[ResizeObserverEntry]) {
        let Some((scroll, _)) = self.view.try_get_untracked() else { return };
        let sizes = entries.iter().filter_map(|entry| {
            let target = entry.target();
            // Rows that just unmounted report a zero size; skip them.
            if !target.is_connected() {
                return None;
            }
            let index = target.get_attribute("data-index").and_then(|v| v.parse::<usize>().ok())?;
            let height = target.get_bounding_client_rect().height();
            (height > 0.0).then_some((index, height))
        });
        let (moved, shift) = self.layout.try_update_value(|l| h::anchor_shift(l, scroll, sizes)).unwrap_or_default();
        if moved {
            self.version.update(|v| *v = v.wrapping_add(1));
            if scroll > 0.0
                && shift.abs() > 0.5
                && let Some(el) = self.viewport.get_untracked()
            {
                el.set_scroll_top((scroll + shift).round() as i32);
            }
        }
        // The viewport itself is observed too: pick up a new height.
        self.sync_view();
    }

    /// The shared size observer, created on first use.
    fn resize_observer(self) -> Option<ResizeObserver> {
        if let Some(ro) = self.observer.try_with_value(|o| o.as_ref().map(|o| o.ro.clone())).flatten() {
            return Some(ro);
        }
        let on_resize =
            Closure::<dyn FnMut(Vec<ResizeObserverEntry>)>::new(move |entries: Vec<ResizeObserverEntry>| {
                self.measured(&entries);
            });
        let ro = ResizeObserver::new(on_resize.as_ref().unchecked_ref()).ok()?;
        self.observer.set_value(Some(Observer { ro: ro.clone(), _on_resize: on_resize }));
        Some(ro)
    }

    /// Watch a row's size while it is mounted. Fixed sizes need no watching.
    fn observe(self, node: NodeRef<Div>) {
        if self.fixed {
            return;
        }
        let held = StoredValue::new_local(None::<web_sys::Element>);
        Effect::new(move |_| {
            let Some(el) = node.get() else { return };
            if let Some(ro) = self.resize_observer() {
                ro.observe(&el);
            }
            held.set_value(Some(el.into()));
        });
        on_cleanup(move || {
            let Some(el) = held.try_update_value(Option::take).flatten() else { return };
            if let Some(Some(ro)) = self.observer.try_with_value(|o| o.as_ref().map(|o| o.ro.clone())) {
                ro.unobserve(&el);
            }
        });
    }

    /// The block for `header`, read reactively.
    fn block_with<T>(self, header: Option<usize>, f: impl FnOnce(&Block) -> T) -> Option<T> {
        self.blocks.with(|bs| bs.iter().find(|b| b.header == header).map(f))
    }
}

/// A virtualised listbox over `count` rows. Only the rows in view (plus `overscan` each
/// side) are in the DOM. Give it a height through its parent.
#[component]
#[allow(clippy::too_many_lines, reason = "one component: props, wiring and one view! tree")]
pub fn VirtualList(
    /// Number of rows, headers included.
    #[prop(into)]
    count: Signal<usize>,
    /// Renders row `index` (an option or a header). Called once each time a row mounts.
    #[prop(into)]
    row: Callback<usize, AnyView>,
    /// Accessible name for the listbox.
    #[prop(into)]
    label: String,
    /// Row height in pixels: exact when fixed, the first guess when `measure` is on.
    #[prop(default = 36.0)]
    row_height: f64,
    /// Header height in pixels. Defaults to `row_height`.
    #[prop(optional)]
    header_height: Option<f64>,
    /// Indexes of group header rows. They stick while their group is in view, are skipped
    /// by the keyboard and can't be selected.
    #[prop(optional, into)]
    headers: Signal<Vec<usize>>,
    /// Measure each row as it renders instead of trusting `row_height`. For rows whose
    /// height depends on their content.
    #[prop(optional)]
    measure: bool,
    /// Rows rendered beyond each edge of the viewport.
    #[prop(default = DEFAULT_OVERSCAN)]
    overscan: usize,
    /// The option selected at first.
    #[prop(optional)]
    default_selected: Option<usize>,
    /// Called with the option chosen by click, Enter or Space.
    #[prop(optional, into)]
    on_select: Option<Callback<usize>>,
    /// Each time this is set to `Some((index, align))`, the list scrolls that row into view.
    #[prop(optional, into)]
    scroll_to: Signal<Option<(usize, Align)>>,
    /// Text shown when there are no rows.
    #[prop(default = "Nothing to show.".into(), into)]
    empty: String,
) -> impl IntoView {
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let header_height = header_height.unwrap_or(row_height);
    let fixed = !measure;
    let input = Memo::new(move |_| {
        let mut hs = headers.get();
        hs.sort_unstable();
        hs.dedup();
        VirtualListInput { count: count.get(), headers: hs, default_selected }
    });
    let state = RwSignal::new(input.with_untracked(VirtualListMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| VirtualListMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let layout =
        StoredValue::new(input.with_untracked(|i| h::layout_for(i.count, &i.headers, row_height, header_height)));
    let version = RwSignal::new(0_u64);
    // New rows keep measured sizes; new headers start over from the estimates.
    Effect::new(move |prev: Option<(usize, Vec<usize>)>| {
        let (n, hs) = input.with(|i| (i.count, i.headers.clone()));
        if let Some((pn, ph)) = &prev
            && (*pn != n || *ph != hs)
        {
            if measure && *ph == hs {
                layout.update_value(|l| l.set_len(n));
            } else {
                layout.set_value(h::layout_for(n, &hs, row_height, header_height));
            }
            version.update(|v| *v = v.wrapping_add(1));
        }
        (n, hs)
    });

    let view = RwSignal::new((0.0_f64, 0.0_f64));
    let window = Memo::new(move |_| {
        version.track();
        let (scroll, viewport) = view.get();
        layout.with_value(|l| l.range(scroll, viewport, overscan))
    });
    let blocks = Memo::new(move |_| {
        version.track();
        let range = window.get();
        let pinned = state.with(|s| s.active);
        input.with(|i| layout.with_value(|l| h::blocks(l, &i.headers, range, pinned)))
    });

    let viewport = NodeRef::<Div>::new();
    let ctx = Ctx {
        id,
        state,
        input,
        layout,
        version,
        view,
        viewport,
        blocks,
        row,
        observer: StoredValue::new_local(None),
        fixed,
        on_select,
    };
    Effect::new(move |_| {
        let Some(el) = viewport.get() else { return };
        if let Some(ro) = ctx.resize_observer() {
            ro.observe(&el);
        }
        ctx.sync_view();
    });
    // The observer only exists in the browser; on the server the owner may drop on another thread.
    if cfg!(target_arch = "wasm32") {
        on_cleanup(move || {
            if let Some(o) = ctx.observer.try_update_value(Option::take).flatten() {
                o.ro.disconnect();
            }
        });
    }
    Effect::new(move |_| {
        if let Some((index, align)) = scroll_to.get() {
            ctx.scroll_to(index, align);
        }
    });

    let groups = Memo::new(move |_| blocks.with(|bs| bs.iter().map(|b| b.header).collect::<Vec<_>>()));
    let is_empty = Memo::new(move |_| input.with(|i| i.count == 0));

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <div
                node_ref=viewport
                id=id.get_value()
                role="listbox"
                aria-label=label
                tabindex="0"
                aria-activedescendant=move || state.with(|s| h::activedescendant(&id.get_value(), s))
                data-scope=h::SCOPE
                data-part="viewport"
                data-measured=measure.then_some("")
                on:scroll=move |_| ctx.sync_view()
                on:focus=move |_| {
                    let (scroll, _) = view.get_untracked();
                    let first = layout.with_value(|l| l.index_at(scroll));
                    ctx.send(VirtualListEvent::Focus(first));
                }
                on:keydown=move |ev| {
                    if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
                        return;
                    }
                    let (scroll, height) = view.get_untracked();
                    let page = layout.with_value(|l| h::page_rows(l, scroll, height));
                    if let Some(event) = h::keydown(&ev.key(), page) {
                        ev.prevent_default();
                        ctx.send(event);
                    }
                }
            >
                <div
                    role="presentation"
                    data-scope=h::SCOPE
                    data-part="spacer"
                    style=move || {
                        version.track();
                        layout.with_value(h::spacer_style)
                    }
                >
                    <For each=move || groups.get() key=|g| *g children=move |g| group_view(ctx, g) />
                </div>
            </div>
            {move || is_empty.get().then(|| view! { <p data-scope=h::SCOPE data-part="empty">{empty.clone()}</p> })}
        </div>
    }
}

fn group_view(ctx: Ctx, header: Option<usize>) -> impl IntoView {
    let root = ctx.id.get_value();
    let label = header.map(|index| {
        let node = NodeRef::<Div>::new();
        ctx.observe(node);
        view! {
            <div
                node_ref=node
                id=h::item_id(&root, index)
                role="presentation"
                data-scope=h::SCOPE
                data-part="group-label"
                data-index=index
                style=move || {
                    ctx.version.track();
                    ctx.layout.with_value(|l| h::header_style(l, index, ctx.fixed))
                }
            >
                {ctx.row.run(index)}
            </div>
        }
    });
    let rows = move || ctx.block_with(header, |b| b.rows.clone()).unwrap_or_default();
    let role = if header.is_some() { "group" } else { "presentation" };
    view! {
        <div
            role=role
            aria-labelledby=header.map(|index| h::item_id(&root, index))
            data-scope=h::SCOPE
            data-part="group"
            style=move || ctx.block_with(header, h::block_style)
        >
            {label}
            <For each=rows key=|i| *i children=move |i| item_view(ctx, header, i) />
        </div>
    }
}

fn item_view(ctx: Ctx, header: Option<usize>, index: usize) -> impl IntoView {
    let node = NodeRef::<Div>::new();
    ctx.observe(node);
    let attrs =
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::item_attrs(&ctx.id.get_value(), s, i, index))));
    let style = move || {
        let top = ctx.block_with(header, |b| b.top).unwrap_or(0.0);
        ctx.version.track();
        ctx.layout.with_value(|l| h::row_style(l, top, index, ctx.fixed))
    };
    view! {
        <div
            node_ref=node
            id=move || attrs.get().id
            role="option"
            aria-selected=move || attrs.get().selected.to_string()
            aria-setsize=move || attrs.get().aria_setsize
            aria-posinset=move || attrs.get().aria_posinset
            data-scope=h::SCOPE
            data-part="item"
            data-index=index
            data-active=move || attrs.get().active.then_some("")
            data-selected=move || attrs.get().selected.then_some("")
            style=style
            on:pointerdown=move |_| ctx.send(VirtualListEvent::Point(index))
            on:click=move |_| ctx.send(VirtualListEvent::Click(index))
        >
            {ctx.row.run(index)}
        </div>
    }
}
