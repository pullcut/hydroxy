//! `<ResizablePanels>`: a row or column of panels with draggable,
//! keyboard-operable separators between them.

use hydroxy_headless::resizable::{self as h, ResizableCommand, ResizableEvent, ResizableInput, ResizableMachine};
use hydroxy_statechart::Machine;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

pub use hydroxy_headless::resizable::{Orientation, PanelSpec};

/// One panel: its content and limits, in percent of the group.
#[derive(Clone)]
pub struct Panel {
    spec: PanelSpec,
    label: Option<String>,
    content: ViewFn,
}

impl Panel {
    /// A panel showing `content`.
    pub fn new(content: impl Into<ViewFn>) -> Self {
        Self { spec: PanelSpec::default(), label: None, content: content.into() }
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

/// Where a pointer drag started: pointer coordinate, divider position, group length.
#[derive(Clone, Copy)]
struct DragOrigin {
    pointer: f64,
    divider: f64,
    length: f64,
}

/// A group of resizable panels. Fills its container; give it a height for rows.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn ResizablePanels(
    /// Panels in order.
    panels: Vec<Panel>,
    /// Side by side (`Horizontal`, default) or stacked.
    #[prop(optional)]
    orientation: Orientation,
    /// Controlled sizes in percent. Pair with `on_sizes_change`.
    #[prop(optional, into)]
    sizes: MaybeProp<Vec<f64>>,
    /// Called with the new sizes as they change.
    #[prop(optional, into)]
    on_sizes_change: Option<Callback<Vec<f64>>>,
    /// Keyboard step in percent.
    #[prop(default = 5.0)]
    step: f64,
) -> impl IntoView {
    let specs: Vec<PanelSpec> = panels.iter().map(|p| p.spec).collect();
    let names: Vec<String> = (0..panels.len().saturating_sub(1))
        .map(|d| h::separator_label(panels[h::controlled_panel(&specs, d)].label.as_deref()))
        .collect();
    let input = Memo::new(move |_| ResizableInput { panels: specs.clone(), sizes: sizes.get(), step });
    let state = RwSignal::new(input.with_untracked(ResizableMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ResizableMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let send = move |event: ResizableEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| ResizableMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let ResizableCommand::SizesChange(sizes) = command;
            if let Some(cb) = on_sizes_change {
                cb.run(sizes);
            }
        }
    };

    let root_id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let root = NodeRef::<Div>::new();
    let origin = StoredValue::new(None::<DragOrigin>);
    let vertical = orientation == Orientation::Vertical;
    let coord = move |ev: &leptos::ev::PointerEvent| f64::from(if vertical { ev.client_y() } else { ev.client_x() });
    let count = panels.len();

    let children = panels
        .into_iter()
        .enumerate()
        .map(|(i, panel)| {
            let separator = (i + 1 < count).then(|| {
                let d = i;
                let name = names[d].clone();
                let attrs = Memo::new(move |_| {
                    input.with(|inp| state.with(|s| h::separator_attrs(&root_id.read_value(), d, orientation, s, inp)))
                });
                view! {
                    <div
                        role="separator"
                        tabindex="0"
                        aria-label=name
                        aria-controls=move || attrs.get().aria_controls
                        aria-orientation=move || attrs.get().aria_orientation
                        aria-valuenow=move || attrs.get().aria_valuenow
                        aria-valuemin=move || attrs.get().aria_valuemin
                        aria-valuemax=move || attrs.get().aria_valuemax
                        data-scope=h::SCOPE
                        data-part="separator"
                        data-orientation=orientation.as_str()
                        data-dragging=move || state.with(|s| s.dragging == Some(d)).then_some("")
                        on:pointerdown=move |ev| {
                            if ev.button() != 0 {
                                return;
                            }
                            ev.prevent_default();
                            if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                                let _ = el.set_pointer_capture(ev.pointer_id());
                            }
                            let length = root.get_untracked().map_or(0, |r| if vertical { r.client_height() } else { r.client_width() });
                            origin.set_value(Some(DragOrigin {
                                pointer: coord(&ev),
                                divider: state.with_untracked(|s| h::divider_position(s, d)),
                                length: f64::from(length),
                            }));
                            send(ResizableEvent::DragStart(d));
                        }
                        on:pointermove=move |ev| {
                            if let Some(o) = origin.get_value() {
                                send(ResizableEvent::DragTo(h::drag_position(o.divider, coord(&ev) - o.pointer, o.length)));
                            }
                        }
                        on:pointerup=move |_| {
                            origin.set_value(None);
                            send(ResizableEvent::DragEnd);
                        }
                        on:pointercancel=move |_| {
                            origin.set_value(None);
                            send(ResizableEvent::DragEnd);
                        }
                        on:keydown=move |ev| {
                            if let Some(event) = h::key_event(&ev.key(), d, orientation) {
                                ev.prevent_default();
                                send(event);
                            }
                        }
                    ></div>
                }
            });
            view! {
                <div
                    id=move || h::panel_id(&root_id.read_value(), i)
                    data-scope=h::SCOPE
                    data-part="panel"
                    data-orientation=orientation.as_str()
                    data-state=move || state.with(|s| h::panel_state(s, i))
                    style=move || state.with(|s| h::panel_style(s, i))
                >
                    {panel.content.run()}
                </div>
                {separator}
            }
        })
        .collect_view();

    view! {
        <div node_ref=root data-scope=h::SCOPE data-part="root" data-orientation=orientation.as_str()>
            {children}
        </div>
    }
}
