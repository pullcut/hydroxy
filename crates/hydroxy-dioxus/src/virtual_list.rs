//! `VirtualList`: a listbox that renders only the rows in view, so it stays smooth with
//! hundreds of thousands of rows. Rows are a fixed height or measured as they render;
//! group headers stick to the top until the next group pushes them off.
//!
//! Focus stays on the listbox and the active option is announced with
//! `aria-activedescendant`. The active row is always rendered, even when scrolled away.
//! Scrolling and measuring talk to the page through `eval`, so it works on web and
//! desktop alike.

use dioxus::prelude::*;
use hydroxy_headless::virtual_list::{
    self as h, Block, Layout, VirtualListCommand, VirtualListEvent, VirtualListInput, VirtualListMachine,
    VirtualListState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;

pub use hydroxy_headless::virtual_list::{Align, DEFAULT_OVERSCAN};

/// Waits for "start" (the viewport is mounted; anything sent earlier is dropped), then reports
/// `[scrollTop, clientHeight, [[index, height], ...]]` at most once a frame while
/// scrolling, and whenever the viewport or (with MEASURE) a row changes size. A number
/// sent in sets `scrollTop`; "stop" tears everything down.
const WATCH: &str = r#"
for (;;) {
  const msg = await dioxus.recv();
  if (msg === "start") break;
  if (msg === "stop") return;
}
const el = document.getElementById(ROOT_ID);
if (!el) return;
let frame = 0;
let sizes = [];
const report = () => {
  frame = 0;
  dioxus.send([el.scrollTop, el.clientHeight, sizes]);
  sizes = [];
};
const onScroll = () => {
  if (!frame) frame = requestAnimationFrame(report);
};
el.addEventListener("scroll", onScroll, { passive: true });
const resize = new ResizeObserver((entries) => {
  for (const e of entries) {
    const t = e.target;
    if (t === el || !t.isConnected) continue;
    const i = Number(t.dataset.index);
    const h = t.getBoundingClientRect().height;
    if (Number.isInteger(i) && h > 0) sizes.push([i, h]);
  }
  report();
});
resize.observe(el);
const watched = new Set();
const scan = () => {
  for (const n of watched) {
    if (!n.isConnected) {
      watched.delete(n);
      resize.unobserve(n);
    }
  }
  for (const n of el.querySelectorAll("[data-index]")) {
    if (!watched.has(n)) {
      watched.add(n);
      resize.observe(n);
    }
  }
};
const rows = new MutationObserver(scan);
if (MEASURE) {
  rows.observe(el, { childList: true, subtree: true });
  scan();
}
report();
for (;;) {
  const msg = await dioxus.recv();
  if (typeof msg !== "number") break;
  el.scrollTop = msg;
  report();
}
el.removeEventListener("scroll", onScroll);
if (frame) cancelAnimationFrame(frame);
resize.disconnect();
rows.disconnect();
"#;

/// A virtualised listbox over `count` rows. Only the rows in view (plus `overscan` each
/// side) are in the DOM. Give it a height through its parent.
#[component]
#[allow(clippy::too_many_lines, reason = "one component: props, wiring and one rsx! tree")]
pub fn VirtualList(
    /// Number of rows, headers included.
    count: ReadSignal<usize>,
    /// Renders row `index` (an option or a header).
    row: Callback<usize, Element>,
    /// Accessible name for the listbox.
    label: String,
    /// Row height in pixels: exact when fixed, the first guess when `measure` is on.
    #[props(default = 36.0)]
    row_height: f64,
    /// Header height in pixels. Defaults to `row_height`.
    header_height: Option<f64>,
    /// Indexes of group header rows. They stick while their group is in view, are skipped
    /// by the keyboard and can't be selected.
    #[props(default)]
    headers: ReadSignal<Vec<usize>>,
    /// Measure each row as it renders instead of trusting `row_height`. For rows whose
    /// height depends on their content.
    #[props(default)]
    measure: bool,
    /// Rows rendered beyond each edge of the viewport.
    #[props(default = DEFAULT_OVERSCAN)]
    overscan: usize,
    /// The option selected at first.
    default_selected: Option<usize>,
    /// Called with the option chosen by click, Enter or Space.
    on_select: Option<EventHandler<usize>>,
    /// Each time this is set to `Some((index, align))`, the list scrolls that row into view.
    #[props(default)]
    scroll_to: ReadSignal<Option<(usize, Align)>>,
    /// Text shown when there are no rows.
    #[props(default = "Nothing to show.".to_string())]
    empty: String,
) -> Element {
    let id = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let header_height = header_height.unwrap_or(row_height);
    let fixed = !measure;
    let input = use_memo(move || {
        let mut hs = headers();
        hs.sort_unstable();
        hs.dedup();
        VirtualListInput { count: count(), headers: hs, default_selected }
    });
    let mut state = use_signal(|| VirtualListMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = VirtualListMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    // Row sizes live outside signals so a measurement never clones them; `version` says
    // when they changed.
    let mut layout = use_hook(|| {
        let i = input.peek();
        CopyValue::new(h::layout_for(i.count, &i.headers, row_height, header_height))
    });
    let mut version = use_signal(|| 0_u64);
    let mut shape = use_hook(|| {
        let i = input.peek();
        CopyValue::new((i.count, i.headers.clone()))
    });
    // New rows keep measured sizes; new headers start over from the estimates.
    use_effect(move || {
        let i = input.read();
        let next = (i.count, i.headers.clone());
        let same_headers = shape.read().1 == next.1;
        if *shape.read() == next {
            return;
        }
        if measure && same_headers {
            layout.write().set_len(next.0);
        } else {
            *layout.write() = h::layout_for(next.0, &next.1, row_height, header_height);
        }
        shape.set(next);
        *version.write() += 1;
    });

    // `(scrollTop, clientHeight)` of the viewport.
    let mut view = use_signal(|| (0.0_f64, 0.0_f64));
    let window = use_memo(move || {
        version.read();
        let (scroll, height) = view();
        layout.read().range(scroll, height, overscan)
    });

    let watch = use_hook(|| {
        document::eval(
            &WATCH
                .replace("ROOT_ID", &crate::js::js_string(&id))
                .replace("MEASURE", if measure { "true" } else { "false" }),
        )
    });
    use_future(move || async move {
        let mut watch = watch;
        while let Ok((scroll, height, sizes)) = watch.recv::<(f64, f64, Vec<(usize, f64)>)>().await {
            let mut moved = false;
            let mut shift = 0.0;
            if !sizes.is_empty() {
                (moved, shift) = h::anchor_shift(&mut layout.write(), scroll, sizes);
            }
            if moved {
                *version.write() += 1;
                // Rows above the top one changed size: move with them so the view holds still.
                if scroll > 0.0 && shift.abs() > 0.5 {
                    let _ = watch.send((scroll + shift).round());
                }
            }
            if *view.peek() != (scroll, height) {
                view.set((scroll, height));
            }
        }
    });
    use_drop(move || {
        let _ = watch.send("stop");
    });

    let scroll_into_view = move |index: usize, align: Align| {
        let (scroll, height) = *view.peek();
        let target = h::scroll_target(&layout.read(), &input.peek().headers, index, align, scroll, height);
        let _ = watch.send(target.round());
    };
    let send = use_callback(move |event: VirtualListEvent| {
        for command in drive::<VirtualListMachine>(state, input, event) {
            match command {
                VirtualListCommand::ScrollTo(index) => scroll_into_view(index, Align::Auto),
                VirtualListCommand::SelectedChange(index) => {
                    if let Some(handler) = on_select {
                        handler.call(index);
                    }
                }
            }
        }
    });
    use_effect(move || {
        if let Some((index, align)) = scroll_to() {
            scroll_into_view(index, align);
        }
    });

    version.read();
    let range = window();
    let s = state.read();
    let i = input.read();
    let l = layout.read();
    let blocks = h::blocks(&l, &i.headers, range, s.active);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            div {
                id: "{id}",
                role: "listbox",
                "aria-label": "{label}",
                tabindex: "0",
                "aria-activedescendant": h::activedescendant(&id, &s),
                "data-scope": h::SCOPE,
                "data-part": "viewport",
                "data-measured": measure.then_some(""),
                onmounted: move |_| {
                    let _ = watch.send("start");
                },
                onfocus: move |_| {
                    let (scroll, _) = *view.peek();
                    let first = layout.read().index_at(scroll);
                    send.call(VirtualListEvent::Focus(first));
                },
                onkeydown: move |e: KeyboardEvent| {
                    let m = e.modifiers();
                    if m.ctrl() || m.meta() || m.alt() {
                        return;
                    }
                    let (scroll, height) = *view.peek();
                    let page = h::page_rows(&layout.read(), scroll, height);
                    if let Some(event) = h::keydown(&e.key().to_string(), page) {
                        e.prevent_default();
                        send.call(event);
                    }
                },
                div {
                    role: "presentation",
                    "data-scope": h::SCOPE,
                    "data-part": "spacer",
                    style: h::spacer_style(&l),
                    for b in blocks.iter() {
                        {group(&id, &l, &s, &i, b, fixed, row, send)}
                    }
                }
            }
            if i.count == 0 {
                p { "data-scope": h::SCOPE, "data-part": "empty", "{empty}" }
            }
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "plain render helper over the component's locals")]
fn group(
    root: &str,
    layout: &Layout,
    state: &VirtualListState,
    input: &VirtualListInput,
    block: &Block,
    fixed: bool,
    row: Callback<usize, Element>,
    send: Callback<VirtualListEvent>,
) -> Element {
    let key = block.header.map_or_else(|| "lead".to_string(), |index| index.to_string());
    let role = if block.header.is_some() { "group" } else { "presentation" };
    rsx! {
        div {
            key: "{key}",
            role: "{role}",
            "aria-labelledby": block.header.map(|index| h::item_id(root, index)),
            "data-scope": h::SCOPE,
            "data-part": "group",
            style: h::block_style(block),
            if let Some(index) = block.header {
                div {
                    id: h::item_id(root, index),
                    role: "presentation",
                    "data-scope": h::SCOPE,
                    "data-part": "group-label",
                    "data-index": index,
                    style: h::header_style(layout, index, fixed),
                    {row.call(index)}
                }
            }
            for &index in &block.rows {
                {option(root, layout, state, input, block.top, index, fixed, row, send)}
            }
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "plain render helper over the component's locals")]
fn option(
    root: &str,
    layout: &Layout,
    state: &VirtualListState,
    input: &VirtualListInput,
    top: f64,
    index: usize,
    fixed: bool,
    row: Callback<usize, Element>,
    send: Callback<VirtualListEvent>,
) -> Element {
    let a = h::item_attrs(root, state, input, index);
    rsx! {
        div {
            key: "{index}",
            id: a.id,
            role: "option",
            "aria-selected": a.selected.to_string(),
            "aria-setsize": a.aria_setsize,
            "aria-posinset": a.aria_posinset,
            "data-scope": h::SCOPE,
            "data-part": "item",
            "data-index": index,
            "data-active": a.active.then_some(""),
            "data-selected": a.selected.then_some(""),
            style: h::row_style(layout, top, index, fixed),
            onpointerdown: move |_| send.call(VirtualListEvent::Point(index)),
            onclick: move |_| send.call(VirtualListEvent::Click(index)),
            {row.call(index)}
        }
    }
}
