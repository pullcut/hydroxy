//! `ResizablePanels`: a row or column of panels with draggable,
//! keyboard-operable separators between them.

use std::rc::Rc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use oxide_headless::resizable::{
    self as h, ResizableCommand, ResizableEvent, ResizableInput, ResizableMachine, ResizableState,
};
use oxide_statechart::Machine;

pub use oxide_headless::resizable::{Orientation, PanelSpec};

/// One panel: its content and limits, in percent of the group.
#[derive(Clone, PartialEq)]
pub struct Panel {
    spec: PanelSpec,
    label: Option<String>,
    content: Element,
}

impl Panel {
    /// A panel showing `content`.
    #[must_use]
    pub fn new(content: Element) -> Self {
        Self { spec: PanelSpec::default(), label: None, content }
    }

    /// Smallest size while expanded.
    #[must_use]
    pub fn min(mut self, percent: f64) -> Self {
        self.spec.min = percent;
        self
    }

    /// Largest size.
    #[must_use]
    pub fn max(mut self, percent: f64) -> Self {
        self.spec.max = percent;
        self
    }

    /// Initial size. Panels without one share what is left.
    #[must_use]
    pub fn default_size(mut self, percent: f64) -> Self {
        self.spec.default_size = Some(percent);
        self
    }

    /// Allow collapsing to zero.
    #[must_use]
    pub fn collapsible(mut self) -> Self {
        self.spec.collapsible = true;
        self
    }

    /// Names the separator that controls this panel: "Resize {label}".
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Where a pointer drag started: pointer coordinate and divider position.
#[derive(Clone, Copy)]
struct DragOrigin {
    pointer: f64,
    divider: f64,
}

fn send(
    mut state: Signal<ResizableState>,
    input: Memo<ResizableInput>,
    on_sizes_change: Option<EventHandler<Vec<f64>>>,
    event: ResizableEvent,
) {
    let step = ResizableMachine::transition(&state.peek(), event, &input.peek());
    state.set(step.state);
    for command in step.commands {
        let ResizableCommand::SizesChange(sizes) = command;
        if let Some(handler) = on_sizes_change {
            handler.call(sizes);
        }
    }
}

/// A group of resizable panels. Fills its container; give it a height for rows.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ResizablePanels(
    /// Panels in order.
    panels: Vec<Panel>,
    /// Side by side (`Horizontal`, default) or stacked.
    #[props(default)]
    orientation: Orientation,
    /// Controlled sizes in percent. Pair with `on_sizes_change`.
    #[props(default)]
    sizes: ReadSignal<Option<Vec<f64>>>,
    /// Called with the new sizes as they change.
    on_sizes_change: Option<EventHandler<Vec<f64>>>,
    /// Keyboard step in percent.
    #[props(default = 5.0)]
    step: f64,
) -> Element {
    let specs: Vec<PanelSpec> = panels.iter().map(|p| p.spec).collect();
    let mut specs_signal = use_signal(|| specs.clone());
    if *specs_signal.peek() != specs {
        specs_signal.set(specs.clone());
    }
    let input = use_memo(move || ResizableInput { panels: specs_signal(), sizes: sizes(), step });
    let mut state = use_signal(|| ResizableMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ResizableMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });

    let root_id = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut origin = use_signal(|| None::<DragOrigin>);
    let mut length = use_signal(|| 0.0_f64);
    let vertical = orientation == Orientation::Vertical;
    let coord = move |e: &PointerEvent| {
        let p = e.client_coordinates();
        if vertical { p.y } else { p.x }
    };
    let mut end = move || {
        if origin.peek().is_some() {
            origin.set(None);
            send(state, input, on_sizes_change, ResizableEvent::DragEnd);
        }
    };

    let st = state.read().clone();
    let inp = input.read().clone();
    let count = panels.len();
    let names: Vec<String> = (0..count.saturating_sub(1))
        .map(|d| h::separator_label(panels[h::controlled_panel(&specs, d)].label.as_deref()))
        .collect();

    rsx! {
        div {
            id: root_id.clone(),
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-orientation": orientation.as_str(),
            onmounted: move |e| mounted.set(Some(e.data())),
            // Moves land on the root: the separator captures the pointer, and
            // the events bubble here.
            onpointermove: move |e| {
                if let Some(o) = origin() {
                    let at = h::drag_position(o.divider, coord(&e) - o.pointer, length());
                    send(state, input, on_sizes_change, ResizableEvent::DragTo(at));
                }
            },
            onpointerup: move |_| end(),
            onpointercancel: move |_| end(),
            for (i, panel) in panels.into_iter().enumerate() {
                div {
                    key: "panel-{i}",
                    id: h::panel_id(&root_id, i),
                    "data-scope": h::SCOPE,
                    "data-part": "panel",
                    "data-orientation": orientation.as_str(),
                    "data-state": h::panel_state(&st, i),
                    style: h::panel_style(&st, i),
                    {panel.content}
                }
                if i + 1 < count {
                    {
                        let a = h::separator_attrs(&root_id, i, orientation, &st, &inp);
                        let sep_id = format!("{root_id}-separator-{i}");
                        rsx! {
                            div {
                                key: "separator-{i}",
                                id: sep_id.clone(),
                                role: "separator",
                                tabindex: "0",
                                "aria-label": names[i].clone(),
                                "aria-controls": a.aria_controls,
                                "aria-orientation": a.aria_orientation,
                                "aria-valuenow": a.aria_valuenow,
                                "aria-valuemin": a.aria_valuemin,
                                "aria-valuemax": a.aria_valuemax,
                                "data-scope": h::SCOPE,
                                "data-part": "separator",
                                "data-orientation": orientation.as_str(),
                                "data-dragging": (st.dragging == Some(i)).then_some(""),
                                onpointerdown: move |e| {
                                    if e.trigger_button() != Some(MouseButton::Primary) {
                                        return;
                                    }
                                    e.prevent_default();
                                    let _ = document::eval(&format!(
                                        "try {{ document.getElementById({sep_id:?})?.setPointerCapture({}); }} catch (_) {{}}",
                                        e.pointer_id()
                                    ));
                                    origin.set(Some(DragOrigin {
                                        pointer: coord(&e),
                                        divider: h::divider_position(&state.peek(), i),
                                    }));
                                    if let Some(el) = mounted() {
                                        spawn(async move {
                                            if let Ok(rect) = el.get_client_rect().await {
                                                length.set(if vertical { rect.height() } else { rect.width() });
                                            }
                                        });
                                    }
                                    send(state, input, on_sizes_change, ResizableEvent::DragStart(i));
                                },
                                onkeydown: move |e| {
                                    if let Some(event) = h::key_event(&e.key().to_string(), i, orientation) {
                                        e.prevent_default();
                                        send(state, input, on_sizes_change, event);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
